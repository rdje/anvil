//! `STRUCTURED-EMISSION-EXPANSION.15b.1` — post-construction annotation
//! that marks selected 2:1 `Mux` gates for the procedural `always_comb`
//! `if`/`else` emit-projection (decision `0027` + the `.15a` design-detail
//! in `DEVELOPMENT_NOTES.md`).
//!
//! The seventh richer-structured surface: a marked `Mux` gate is rendered by
//! the emitter as a behaviour-preserving procedural conditional
//!
//! ```systemverilog
//! logic [W-1:0] <gate>__cv;
//! always_comb begin
//!     if (<sel>) <gate>__cv = <a>;
//!     else <gate>__cv = <b>;
//! end
//! assign <gate> = <gate>__cv;   // the gate's net, unchanged downstream
//! ```
//!
//! instead of the inline ternary `assign <gate> = (<sel>) ? (<a>) : (<b>);`.
//! The `if`/`else` writes exactly the mux's value (`sel == 1 ⇒ a` — operand 1,
//! `sel == 0 ⇒ b` — operand 2; the same operand mapping the ternary uses), and
//! the existing `<gate>` net is driven from the `<gate>__cv` output var — so
//! the projection is **behaviour-preserving by construction**. This is the
//! decision `0014` single-gate-`task` **output-var + passthrough** mechanism,
//! but a bare `always_comb` `if`/`else` rather than a `task` call. It is the
//! first procedural-conditional construct in the lane — none of the six prior
//! surfaces emits a procedural `if`/`else` (the `Mux` is a continuous-assign
//! ternary; `CaseMux`/`CasezMux` are `case`/`casez`).
//!
//! **Rules-first, never generate-then-filter.** The `always_comb` block
//! re-expresses a `Mux` that is already valid in the flat emission; selection
//! happens here at construction time. There is nothing to check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-gate decision is a seeded `gen_bool(prob)` here (reproducible;
//! never `thread_rng`). The generator guards the call on
//! `Config::mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing from
//! the RNG and marks nothing ⇒ byte-identical stream + output. The annotation
//! is an emitter-surface marker only: the flat IR body, validators, CSE keys
//! and `canonical_module_signature` are all untouched. Mirrors
//! `crate::ir::task_emit::annotate_task_emit_gates`.
//!
//! **Mutually exclusive with the sibling projections.** A gate is projected by
//! at most one of `function_emit` / `generate_loop` / `task_emit` /
//! `multi_output_task` / `cone_function` / `soft_union` / `mux_if`. This pass
//! runs **last** (after `cone_function`) and excludes any gate already marked
//! there — so the exclusion set is the union of every sibling mark, and this
//! set is disjoint from all of them by construction (the established "later
//! pass excludes earlier marks" ordering).

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the procedural `always_comb`
/// `if`/`else` projection: a 2:1 `Node::Gate` whose op is `GateOp::Mux`
/// (`[sel, a, b]`, a 1-bit selector by IR invariant — checked defensively as
/// exactly three operands) that is **not** already marked by any sibling
/// emit-projection. Because this pass runs last, the exclusion set is the union
/// of `function_emit_gates` / `generate_loop_gates` / `task_emit_gates` /
/// `soft_union_slice_gates`, the multi-output-task members
/// (`multi_output_task_groups` keys ∪ values) and the cone-function roots ∪
/// interiors (`cone_function_gates` keys ∪ flattened values). In practice only
/// `function_emit` / `task_emit` / `multi_output_task` / `cone_function` can
/// ever mark a `Mux` (generate_loop targets `{N{x}}` `Concat`, soft_union
/// targets `Slice`), but all six are excluded for robustness.
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate { op, operands, .. } = node else {
        return false;
    };
    if !matches!(op, GateOp::Mux) {
        return false;
    }
    if operands.len() != 3 {
        return false;
    }
    if m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
    {
        return false;
    }
    // Multi-output task members: a leader key or any partner value.
    if m.multi_output_task_groups.contains_key(&id)
        || m.multi_output_task_groups
            .values()
            .flatten()
            .any(|&x| x == id)
    {
        return false;
    }
    // Cone-function gates: a cone root key or any absorbed interior value.
    if m.cone_function_gates.contains_key(&id)
        || m.cone_function_gates.values().flatten().any(|&x| x == id)
    {
        return false;
    }
    true
}

/// Mark qualifying 2:1 `Mux` gates for the procedural `always_comb` `if`/`else`
/// emit-projection by rolling `prob` per qualifying gate on the seeded
/// generator RNG. Returns the number newly marked. Callers must gate on
/// `prob > 0.0` so the default path is byte-identical (draws nothing).
/// Single-call per module (mirrors the `task_emit` / `function_emit`
/// call-site roll). Must run **last** — after every sibling projection pass —
/// so their marks are visible and excluded here.
pub fn annotate_mux_if_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted widths are
    // symbolic; the param/structured cross-product is out of scope). Mirrors the
    // task_emit pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.mux_if_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.mux_if_gates.insert(id) {
            marked += 1;
        }
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, GateOp, Module, Node, Port};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(0)
    }

    /// `y = sel ? a : b` over a 1-bit selector + two 4-bit inputs — node 3 is a
    /// 2:1 `Mux` (the mux-if candidate).
    fn module_mux_gate() -> Module {
        let mut m = Module {
            name: "mi".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "sel".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 2,
            name: "b".into(),
            width: 4,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 3,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // id 0 (sel)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // id 1 (a)
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // id 2 (b)
        m.nodes.push(Node::Gate {
            op: GateOp::Mux,
            operands: vec![0, 1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 3
        m.drives.push((3, 3));
        m
    }

    #[test]
    fn prob_one_marks_a_mux_gate() {
        let mut m = module_mux_gate();
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.mux_if_gates.contains(&3));
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_mux_gate();
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.mux_if_gates.is_empty());
    }

    #[test]
    fn non_mux_gate_does_not_qualify() {
        // An `and` gate is not a Mux — never a mux-if candidate.
        let mut m = Module {
            name: "ng".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // id 1
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 2
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.mux_if_gates.is_empty());
    }

    #[test]
    fn function_emit_marked_mux_is_excluded() {
        // A Mux already marked for the `function automatic` projection is never
        // also mux-if'd (the projections are mutually exclusive; this pass runs
        // after function_emit).
        let mut m = module_mux_gate();
        m.function_emit_gates.insert(3);
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.mux_if_gates.is_empty());
    }

    #[test]
    fn task_emit_marked_mux_is_excluded() {
        let mut m = module_mux_gate();
        m.task_emit_gates.insert(3);
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.mux_if_gates.is_empty());
    }

    #[test]
    fn cone_function_root_or_interior_mux_is_excluded() {
        // A Mux that is a cone-function root (a key) is excluded.
        let mut root = module_mux_gate();
        root.cone_function_gates.insert(3, vec![]);
        assert_eq!(annotate_mux_if_gates(&mut root, &mut rng(), 1.0), 0);
        assert!(root.mux_if_gates.is_empty());
        // A Mux that is an absorbed cone interior (a value) is excluded too.
        let mut interior = module_mux_gate();
        interior.cone_function_gates.insert(99, vec![3]);
        assert_eq!(annotate_mux_if_gates(&mut interior, &mut rng(), 1.0), 0);
        assert!(interior.mux_if_gates.is_empty());
    }

    #[test]
    fn multi_output_task_member_mux_is_excluded() {
        // A Mux that is a multi-output-task leader (a key) is excluded.
        let mut leader = module_mux_gate();
        leader.multi_output_task_groups.insert(3, vec![99]);
        assert_eq!(annotate_mux_if_gates(&mut leader, &mut rng(), 1.0), 0);
        assert!(leader.mux_if_gates.is_empty());
        // A Mux that is a multi-output-task partner (a value) is excluded too.
        let mut partner = module_mux_gate();
        partner.multi_output_task_groups.insert(99, vec![3]);
        assert_eq!(annotate_mux_if_gates(&mut partner, &mut rng(), 1.0), 0);
        assert!(partner.mux_if_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_mux_gate();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR node and
        // does not change `canonical_module_signature`.
        let mut m = module_mux_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked Mux renders a behaviour-preserving
    /// `always_comb` `if`/`else` block writing a `<wire>__cv` output var + a
    /// passthrough `assign`, and the default (unmarked) emission is the plain
    /// inline ternary — proving the projection is opt-in and byte-identical by
    /// default.
    #[test]
    fn marked_mux_emits_if_else_block_unmarked_is_inline_ternary() {
        use crate::emit::to_sv;

        // Unmarked baseline: the plain inline ternary, no procedural block.
        let base = to_sv(&module_mux_gate());
        assert!(
            base.contains("assign mux_0 = (sel) ? (a) : (b);"),
            "default-off emission is the inline mux ternary:\n{base}"
        );
        assert!(
            !base.contains("mux_0__cv"),
            "default-off emission has no procedural conditional var:\n{base}"
        );

        // Marked: the gate is projected to an `always_comb if/else` + `<wire>__cv`
        // var + passthrough assign.
        let mut marked = module_mux_gate();
        marked.mux_if_gates.insert(3);
        let out = to_sv(&marked);
        assert!(
            out.contains("logic [3:0] mux_0__cv;"),
            "the conditional output var is declared:\n{out}"
        );
        assert!(
            out.contains("    always_comb begin"),
            "a procedural always_comb block is emitted:\n{out}"
        );
        assert!(
            out.contains("if (sel) mux_0__cv = a;"),
            "the sel==1 arm writes operand 1 (a):\n{out}"
        );
        assert!(
            out.contains("else mux_0__cv = b;"),
            "the sel==0 arm writes operand 2 (b):\n{out}"
        );
        assert!(
            out.contains("assign mux_0 = mux_0__cv;"),
            "the gate's assign becomes a passthrough from the conditional var:\n{out}"
        );
        // The inline ternary is suppressed for the marked gate.
        assert!(
            !out.contains("assign mux_0 = (sel) ? (a) : (b);"),
            "the inline ternary is suppressed:\n{out}"
        );
        // The output port is still driven from the gate wire unchanged.
        assert!(
            out.contains("assign y = mux_0;"),
            "the output drive is unchanged:\n{out}"
        );
    }
}
