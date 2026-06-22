//! `STRUCTURED-EMISSION-EXPANSION.17b.1` — post-construction annotation
//! that marks selected dynamic-selector `CaseMux` gates for the procedural
//! `always_comb` `if`/`else if` **priority-chain** emit-projection (decision
//! `0028` + the `.17a` design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The eighth richer-structured surface: a marked `CaseMux` gate, instead of
//! the parallel `case` statement it renders today
//!
//! ```systemverilog
//! always_comb begin
//!     case (sel)
//!         W'd0: casemux_0 = arm_0;
//!         W'd1: casemux_0 = arm_1;
//!         default: casemux_0 = D'h0;
//!     endcase
//! end
//! ```
//!
//! is re-expressed as a behaviour-preserving `if`/`else if` priority chain
//!
//! ```systemverilog
//! always_comb begin
//!     if (sel == W'd0) casemux_0 = arm_0;
//!     else if (sel == W'd1) casemux_0 = arm_1;
//!     else casemux_0 = D'h0;
//! end
//! ```
//!
//! The chain tests each `case` label `W'd{i}` in ascending arm order and falls
//! through to the same `default` value. Because the labels are **distinct
//! constants by construction** (arm index `i` ⇒ label `W'd{i}`), at most one
//! equality is ever true, so the priority chain and the parallel `case` produce
//! identical results for every selector value — the projection is
//! **behaviour-preserving by construction**.
//!
//! Unlike the seventh surface (the 2:1 `Mux`, `mux_if_emit`), a `CaseMux` is
//! **already** declared as an `always_comb`-written `logic` var, so this surface
//! needs **no** `<gate>__cv` output var + passthrough — only the `always_comb`
//! *body* swaps `case … endcase` → `if … else if`. It is the first N-way
//! procedural priority-chain construct in the lane (the seventh surface emits a
//! single 2:1 `if`/`else`; `CaseMux`/`CasezMux` are parallel `case`/`casez`).
//!
//! **Dynamic selectors only.** A `CaseMux` with a *constant* selector is
//! statically collapsed by the emitter to a continuous `assign` of the selected
//! arm (`render_static_structured_gate`), so it never emits an `always_comb`
//! block and is **not** a candidate. The predicate excludes it directly (its
//! selector operand is a `Node::Constant`), which keeps the
//! `num_emitted_case_mux_if_chains` metric (`= case_mux_if_gates.len()`) exact.
//!
//! **Rules-first, never generate-then-filter.** The `always_comb` block
//! re-expresses a `CaseMux` that is already valid in the flat emission;
//! selection happens here at construction time. There is nothing to
//! check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-gate decision is a seeded `gen_bool(prob)` here (reproducible; never
//! `thread_rng`). The generator guards the call on
//! `Config::case_mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing
//! from the RNG and marks nothing ⇒ byte-identical stream + output. The
//! annotation is an emitter-surface marker only: the flat IR body, validators,
//! CSE keys and `canonical_module_signature` are all untouched. Mirrors
//! `crate::ir::mux_if_emit::annotate_mux_if_gates`.
//!
//! **Mutually exclusive with the sibling projections.** A gate is projected by
//! at most one of the eight emit-surfaces. This pass runs **last** (after
//! `mux_if`) and excludes any gate already marked by a sibling projection — in
//! practice vacuous (no other pass marks a `CaseMux`; they target plain gates,
//! `{N{x}}` `Concat`, or `Slice`), but kept for robustness and the established
//! "later pass excludes earlier marks" convention.

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the procedural `always_comb`
/// `if`/`else if` priority-chain projection: a `Node::Gate` whose op is
/// `GateOp::CaseMux`, with a **non-constant (dynamic) selector** (`operands[0]`
/// is not a `Node::Constant` — a constant selector would render as a static
/// continuous `assign`, never an `always_comb` block), at least one arm, and not
/// already marked by any sibling emit-projection.
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate { op, operands, .. } = node else {
        return false;
    };
    if !matches!(op, GateOp::CaseMux) {
        return false;
    }
    // A selector + at least one arm.
    if operands.len() < 2 {
        return false;
    }
    // Dynamic selector only: a constant selector is statically collapsed to a
    // continuous assign (`render_static_structured_gate`) and never emits an
    // `always_comb` block, so it cannot host the chain.
    if matches!(
        m.nodes.get(operands[0] as usize),
        Some(Node::Constant { .. })
    ) {
        return false;
    }
    // Sibling-projection exclusion (the pass runs last). Vacuous for a `CaseMux`
    // in practice — no other pass marks one — but kept for robustness.
    if m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
        || m.mux_if_gates.contains(&id)
    {
        return false;
    }
    if m.multi_output_task_groups.contains_key(&id)
        || m.multi_output_task_groups
            .values()
            .flatten()
            .any(|&x| x == id)
    {
        return false;
    }
    if m.cone_function_gates.contains_key(&id)
        || m.cone_function_gates.values().flatten().any(|&x| x == id)
    {
        return false;
    }
    true
}

/// Mark qualifying dynamic-selector `CaseMux` gates for the procedural
/// `always_comb` `if`/`else if` priority-chain emit-projection by rolling `prob`
/// per qualifying gate on the seeded generator RNG. Returns the number newly
/// marked. Callers must gate on `prob > 0.0` so the default path is
/// byte-identical (draws nothing). Single-call per module (mirrors the
/// `mux_if_emit` call-site roll). Must run **last** — after every sibling
/// projection pass — so their marks are visible and excluded here.
pub fn annotate_case_mux_if_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted widths are
    // symbolic; the param/structured cross-product is out of scope). Mirrors the
    // mux_if pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.case_mux_if_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.case_mux_if_gates.insert(id) {
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

    /// `y = case (sel) 0:a 1:b 2:c` over a 2-bit dynamic selector + three 4-bit
    /// inputs — node 4 is the `CaseMux` (the chain candidate).
    fn module_case_mux_gate() -> Module {
        let mut m = Module {
            name: "cm".into(),
            ..Module::default()
        };
        for (id, name, width) in [(0u32, "sel", 2u32), (1, "a", 4), (2, "b", 4), (3, "c", 4)] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width,
                dir: Direction::In,
            });
        }
        m.outputs.push(Port {
            id: 4,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 }); // id 0 (sel)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // id 1 (a)
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // id 2 (b)
        m.nodes.push(Node::PrimaryInput { port: 3, width: 4 }); // id 3 (c)
        m.nodes.push(Node::Gate {
            op: GateOp::CaseMux,
            operands: vec![0, 1, 2, 3],
            width: 4,
            deps: DepSet::new(),
        }); // id 4
        m.drives.push((4, 4));
        m
    }

    /// The same shape but with a **constant** selector — statically collapsed by
    /// the emitter, never a chain candidate.
    fn module_static_case_mux_gate() -> Module {
        let mut m = module_case_mux_gate();
        // Replace the selector (node 0) with a constant. The CaseMux still
        // references operand 0; node 0 becomes a Constant.
        m.nodes[0] = Node::Constant { value: 1, width: 2 };
        m
    }

    #[test]
    fn prob_one_marks_a_dynamic_case_mux() {
        let mut m = module_case_mux_gate();
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.case_mux_if_gates.contains(&4));
    }

    #[test]
    fn constant_selector_case_mux_is_excluded() {
        let mut m = module_static_case_mux_gate();
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a constant-selector CaseMux is statically collapsed");
        assert!(m.case_mux_if_gates.is_empty());
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_case_mux_gate();
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.case_mux_if_gates.is_empty());
    }

    #[test]
    fn non_case_mux_gate_does_not_qualify() {
        // A plain 2:1 Mux is the seventh surface's candidate, not this one.
        let mut m = Module {
            name: "ng".into(),
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
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 });
        m.nodes.push(Node::Gate {
            op: GateOp::Mux,
            operands: vec![0, 1, 2],
            width: 4,
            deps: DepSet::new(),
        });
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.case_mux_if_gates.is_empty());
    }

    #[test]
    fn casez_mux_gate_does_not_qualify() {
        // CasezMux carries ?-wildcard patterns — a masked comparison, not a plain
        // equality chain (the recorded follow-up); never this surface's candidate.
        let mut m = Module {
            name: "cz".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "sel".into(),
            width: 2,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        m.nodes.push(Node::Constant { value: 0, width: 2 }); // pattern
        m.nodes.push(Node::Constant { value: 3, width: 2 }); // wildcard mask
        m.nodes.push(Node::Gate {
            op: GateOp::CasezMux,
            operands: vec![0, 2, 3, 1], // [sel, pattern, mask, data]
            width: 4,
            deps: DepSet::new(),
        });
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.case_mux_if_gates.is_empty());
    }

    #[test]
    fn sibling_marked_case_mux_is_excluded() {
        // A CaseMux already marked by a sibling projection (artificial — no real
        // pass marks a CaseMux) is never also chained: the robustness exclusion.
        let mut m = module_case_mux_gate();
        m.mux_if_gates.insert(4);
        assert_eq!(annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0), 0);
        assert!(m.case_mux_if_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_case_mux_gate();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR node and
        // does not change `canonical_module_signature`.
        let mut m = module_case_mux_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_case_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked dynamic `CaseMux` renders a
    /// behaviour-preserving `always_comb` `if`/`else if` priority chain, and the
    /// default (unmarked) emission is the parallel `case` — proving the
    /// projection is opt-in and byte-identical by default.
    #[test]
    fn marked_case_mux_emits_priority_chain_unmarked_is_case() {
        use crate::emit::to_sv;

        // Unmarked baseline: the parallel `case` statement.
        let base = to_sv(&module_case_mux_gate());
        assert!(
            base.contains("case (sel)"),
            "default-off emission is the parallel case:\n{base}"
        );
        assert!(
            base.contains("endcase"),
            "default-off emission closes with endcase:\n{base}"
        );

        // Marked: the gate is projected to an `if`/`else if` priority chain.
        let mut marked = module_case_mux_gate();
        marked.case_mux_if_gates.insert(4);
        let out = to_sv(&marked);
        assert!(
            out.contains("    always_comb begin"),
            "a procedural always_comb block is emitted:\n{out}"
        );
        assert!(
            out.contains("if (sel == 2'd0)"),
            "the first arm is an `if` equality test:\n{out}"
        );
        assert!(
            out.contains("else if (sel == 2'd1)"),
            "subsequent arms are `else if` equality tests:\n{out}"
        );
        assert!(
            out.contains("else if (sel == 2'd2)"),
            "every arm is chained in order:\n{out}"
        );
        // The trailing `else` carries the former `default` value.
        assert!(
            out.contains("'h0;") && out.contains("        else "),
            "the trailing else carries the default value:\n{out}"
        );
        // The parallel `case`/`endcase` is suppressed for the marked gate.
        assert!(
            !out.contains("case (sel)"),
            "the parallel case is suppressed:\n{out}"
        );
        assert!(
            !out.contains("endcase"),
            "no endcase in the chain form:\n{out}"
        );
        // The output port is still driven from the gate wire unchanged.
        assert!(
            out.contains("assign y ="),
            "the output drive is unchanged:\n{out}"
        );
    }
}
