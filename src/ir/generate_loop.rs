//! `STRUCTURED-EMISSION-EXPANSION.4b` — post-construction annotation that
//! marks selected `{N{x}}` replication gates for the `generate for` loop
//! emit-projection (decision `0013` + the `.4a` design-detail in
//! `DEVELOPMENT_NOTES.md`).
//!
//! The second richer-structured surface: a marked replication gate is
//! rendered by the emitter as a behaviour-preserving single-level
//! `generate for` loop
//!
//! ```systemverilog
//! genvar <gate>__gi;
//! generate
//!     for (<gate>__gi = 0; <gate>__gi < N; <gate>__gi = <gate>__gi + 1) begin : <gate>__gen
//!         assign <gate>[<gate>__gi] = <x>;
//!     end
//! endgenerate
//! ```
//!
//! instead of the inline `assign <gate> = {N{x}};`. The candidate is a
//! `GateOp::Concat` of the `{N{x}}` form — `>= 2` operands that are all the
//! *same* `NodeId`, with a **1-bit lane** (the replicated operand is 1 bit
//! wide). With a 1-bit lane the result width is exactly `N == operands.len()`
//! and bit `g` of the result is exactly the lane `x`, so the unrolled loop is
//! **byte-equivalent** to the inline replication — the projection is
//! behaviour-preserving by construction.
//!
//! **Rules-first, never generate-then-filter.** The loop re-expresses a
//! replication that is already valid in the flat emission; selection happens
//! here at construction time. There is nothing to check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-gate decision is a seeded `gen_bool(prob)` here (reproducible;
//! never `thread_rng`). The generator guards the call on
//! `Config::generate_loop_emit_prob > 0.0`, so the default (`0.0`) draws
//! nothing from the RNG and marks nothing ⇒ byte-identical stream + output.
//! The annotation is an emitter-surface marker only: the flat IR body,
//! validators, CSE keys and `canonical_module_signature` are all untouched.
//! Mirrors `crate::ir::function_emit::annotate_function_emit_gates`.
//!
//! **Mutually exclusive with the `function automatic` projection.** A
//! replication `Concat` is also a function-emit candidate. This pass runs
//! **after** `annotate_function_emit_gates` and excludes any gate already
//! marked there (the established "later pass excludes earlier marks"
//! ordering — `function_emit` itself runs after `soft_union` and excludes its
//! marks).

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the `generate for` loop
/// projection: a `GateOp::Concat` of the `{N{x}}` form —
///
/// - `>= 2` operands that are **all the same `NodeId`** (an N-fold
///   replication of one operand);
/// - the replicated operand (the *lane*) is **1 bit wide**, so the result
///   width is exactly `N` and bit `g` of the result is exactly the lane (the
///   loop body `assign <wire>[gi] = <x>;` is byte-faithful). A wider lane
///   would need a part-select body (`<wire>[gi*LW +: LW]`) — a recorded
///   follow-up; a wider replication still emits inline, nothing retired;
/// - **not** already marked for the `function automatic` projection (the two
///   emit-projections are mutually exclusive on a gate; this pass runs after
///   `function_emit`);
/// - **not** marked for the `union soft` overlay (defensive — `soft_union`
///   only marks `Slice` gates, never `Concat`s).
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate {
        op: GateOp::Concat,
        operands,
        width,
        ..
    } = node
    else {
        return false;
    };
    if operands.len() < 2 {
        return false;
    }
    let first = operands[0];
    if !operands.iter().all(|o| *o == first) {
        return false;
    }
    // 1-bit lane ⇒ result width == N (each result bit is the lane).
    let Some(lane) = m.nodes.get(first as usize) else {
        return false;
    };
    if lane.width() != 1 || *width as usize != operands.len() {
        return false;
    }
    if m.function_emit_gates.contains(&id) {
        return false;
    }
    if m.soft_union_slice_gates.contains(&id) {
        return false;
    }
    true
}

/// Mark qualifying `{N{x}}` replication gates for the `generate for` loop
/// emit-projection by rolling `prob` per qualifying gate on the seeded
/// generator RNG. Returns the number newly marked. Callers must gate on
/// `prob > 0.0` so the default path is byte-identical (draws nothing).
/// Single-call per module (mirrors the `function_emit` / `soft_union`
/// call-site roll). Must run **after** `annotate_function_emit_gates` so the
/// function-emit marks are visible and excluded here.
pub fn annotate_generate_loop_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted widths
    // are symbolic). Mirrors the soft_union / function_emit pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.generate_loop_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.generate_loop_gates.insert(id) {
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

    /// `y = {4{sel}}` over a 1-bit input — node 1 is a `{N{x}}` replication
    /// with a 1-bit lane (the generate-loop candidate).
    fn module_1bit_replication(n: usize) -> Module {
        let mut m = Module {
            name: "gl".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "sel".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: n as u32,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // id 0
        m.nodes.push(Node::Gate {
            op: GateOp::Concat,
            operands: vec![0; n],
            width: n as u32,
            deps: DepSet::from_port(0),
        }); // id 1
        m.drives.push((1, 1));
        m
    }

    #[test]
    fn prob_one_marks_a_1bit_replication_gate() {
        let mut m = module_1bit_replication(4);
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 1);
        assert!(m.generate_loop_gates.contains(&1));
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_1bit_replication(4);
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(marked, 0);
        assert!(m.generate_loop_gates.is_empty());
    }

    #[test]
    fn single_operand_concat_does_not_qualify() {
        // A single-operand Concat is the identity, not a replication.
        let mut m = module_1bit_replication(1);
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 0);
    }

    #[test]
    fn non_replication_concat_does_not_qualify() {
        // `{a, b}` — distinct operands, not a replication of one lane.
        let mut m = Module {
            name: "cc".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 1,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // id 0
        m.nodes.push(Node::PrimaryInput { port: 1, width: 1 }); // id 1
        m.nodes.push(Node::Gate {
            op: GateOp::Concat,
            operands: vec![0, 1],
            width: 2,
            deps: DepSet::new(),
        }); // id 2
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 0);
    }

    #[test]
    fn wide_lane_replication_does_not_qualify() {
        // `{4{byte}}` — lane is 8-bit, so a `<wire>[gi]` body would be wrong;
        // excluded from the first cut (part-select is a recorded follow-up).
        let mut m = Module {
            name: "wl".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "byte".into(),
            width: 8,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // id 0
        m.nodes.push(Node::Gate {
            op: GateOp::Concat,
            operands: vec![0, 0, 0, 0],
            width: 32,
            deps: DepSet::from_port(0),
        }); // id 1
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 0);
    }

    #[test]
    fn function_emit_marked_gate_is_excluded() {
        // A replication already marked for the `function automatic` projection
        // is never also generate-loop-emitted (mutually exclusive on a gate).
        let mut m = module_1bit_replication(4);
        m.function_emit_gates.insert(1);
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 0);
        assert!(m.generate_loop_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_1bit_replication(4);
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let marked = annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(marked, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR node
        // and does not change `canonical_module_signature`.
        let mut m = module_1bit_replication(4);
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_generate_loop_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked `{N{x}}` replication renders a
    /// behaviour-preserving `generate for` loop, and the default (unmarked)
    /// emission is the plain inline `{N{x}}` assign — proving the projection
    /// is opt-in and byte-identical by default.
    #[test]
    fn marked_gate_emits_generate_loop_unmarked_is_inline() {
        use crate::emit::to_sv;

        // Unmarked baseline: the plain inline replication assign, no generate.
        let base = to_sv(&module_1bit_replication(4));
        assert!(
            !base.contains("generate"),
            "default-off emission has no generate block:\n{base}"
        );
        assert!(
            base.contains("assign concat_0 = {4{sel}};"),
            "default-off emission is the inline replication assign:\n{base}"
        );

        // Marked: the replication is projected to a `generate for` loop.
        let mut marked = module_1bit_replication(4);
        marked.generate_loop_gates.insert(1);
        let out = to_sv(&marked);
        assert!(
            out.contains("genvar concat_0__gi;"),
            "marked gate declares a genvar:\n{out}"
        );
        assert!(
            out.contains("for (concat_0__gi = 0; concat_0__gi < 4; concat_0__gi = concat_0__gi + 1) begin : concat_0__gen"),
            "the generate for loop spans the replication count:\n{out}"
        );
        assert!(
            out.contains("assign concat_0[concat_0__gi] = sel;"),
            "the loop body drives each bit from the lane:\n{out}"
        );
        assert!(
            out.contains("endgenerate"),
            "the generate region is closed:\n{out}"
        );
        // The inline replication assign is suppressed for the marked gate.
        assert!(
            !out.contains("assign concat_0 = {4{sel}};"),
            "the inline replication assign is suppressed:\n{out}"
        );
        // The output port is still driven from the gate wire unchanged.
        assert!(
            out.contains("assign y = concat_0;"),
            "the output drive is unchanged:\n{out}"
        );
    }
}
