//! `STRUCTURED-EMISSION-EXPANSION.19b.1` — post-construction annotation
//! that marks selected dynamic-selector `CasezMux` gates for the procedural
//! `always_comb` `if`/`else if` **masked** priority-chain emit-projection
//! (decision `0029` + the `.19a` design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The ninth richer-structured surface, the wildcard generalization of the
//! eighth (`case_mux_if`). A marked `CasezMux` gate, instead of the parallel
//! `casez` statement it renders today
//!
//! ```systemverilog
//! always_comb begin
//!     casez (sel)
//!         2'b0?: casez_mux_0 = a;
//!         2'b1?: casez_mux_0 = b;
//!         default: casez_mux_0 = D'h0;
//!     endcase
//! end
//! ```
//!
//! is re-expressed as a behaviour-preserving **masked** `if`/`else if` priority
//! chain
//!
//! ```systemverilog
//! always_comb begin
//!     if ((sel & 2'h2) == 2'h0) casez_mux_0 = a;
//!     else if ((sel & 2'h2) == 2'h2) casez_mux_0 = b;
//!     else casez_mux_0 = D'h0;
//! end
//! ```
//!
//! Each arm tests `(sel & care_mask) == value_masked`, where `care_mask =
//! ~wildcard_mask & sel_mask` and `value_masked = pattern_value & care_mask` —
//! exactly the `casez_pattern_matches` predicate the emitter / metrics / compact
//! paths already use (`((sel ^ pattern) & care_mask) == 0`). Because a priority
//! `if`/`else if` chain is first-match-wins, exactly like `casez`, the chain
//! selects the same arm as the parallel `casez` for **every** selector value,
//! and the trailing `else` covers exactly the `default` — the projection is
//! **behaviour-preserving by construction**. (ANVIL's `casez` arms are
//! non-overlapping by construction — `build_casez_patterns` uses
//! `wildcard_bits = 1`, so each arm carries a distinct care-bit value and no
//! arm degenerates to an all-wildcard constant-true condition.)
//!
//! It is the **masked** sibling of the eighth surface (`case_mux_if`): that one
//! projects a plain `CaseMux` into a chain of **bare** equalities `sel ==
//! SW'd{i}`; this one projects a wildcard `CasezMux` into a chain of **masked**
//! equalities `(sel & care_mask) == value_masked`. The masked-AND form is the
//! shipped one because the concise wildcard-equality operator (`sel ==?
//! pattern`) is rejected by Yosys (decision `0029` probe); the masked-AND form
//! is accepted warning-clean by every repo tool and sim-equivalent to the
//! `casez`.
//!
//! Unlike the seventh surface (the 2:1 `Mux`, `mux_if_emit`), a `CasezMux` is
//! **already** declared as an `always_comb`-written `logic` var, so this surface
//! needs **no** `<gate>__cv` output var + passthrough — only the `always_comb`
//! *body* swaps `casez … endcase` → masked `if … else if`.
//!
//! **Dynamic selectors only.** A `CasezMux` with a *constant* selector is
//! statically collapsed by the emitter to a continuous `assign` of the matching
//! arm (`render_static_structured_gate`), so it never emits an `always_comb`
//! block and is **not** a candidate. The predicate excludes it directly (its
//! selector operand is a `Node::Constant`), which keeps the
//! `num_emitted_casez_mux_if_chains` metric (`= casez_mux_if_gates.len()`) exact.
//!
//! **Rules-first, never generate-then-filter.** The `always_comb` block
//! re-expresses a `CasezMux` that is already valid in the flat emission;
//! selection happens here at construction time. There is nothing to
//! check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-gate decision is a seeded `gen_bool(prob)` here (reproducible; never
//! `thread_rng`). The generator guards the call on
//! `Config::casez_mux_if_emit_prob > 0.0`, so the default (`0.0`) draws nothing
//! from the RNG and marks nothing ⇒ byte-identical stream + output. The
//! annotation is an emitter-surface marker only: the flat IR body, validators,
//! CSE keys and `canonical_module_signature` are all untouched. Mirrors
//! `crate::ir::case_mux_if_emit::annotate_case_mux_if_gates`.
//!
//! **Mutually exclusive with the sibling projections.** A gate is projected by
//! at most one of the nine emit-surfaces. This pass runs **last** (after
//! `case_mux_if`) and excludes any gate already marked by a sibling projection —
//! in practice vacuous (no other pass marks a `CasezMux`; they target plain
//! gates, `{N{x}}` `Concat`, `Slice`, or `CaseMux`), but kept for robustness and
//! the established "later pass excludes earlier marks" convention.

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the procedural `always_comb`
/// `if`/`else if` **masked** priority-chain projection: a `Node::Gate` whose op
/// is `GateOp::CasezMux`, with a **non-constant (dynamic) selector**
/// (`operands[0]` is not a `Node::Constant` — a constant selector would render
/// as a static continuous `assign`, never an `always_comb` block), at least one
/// `(value, mask, data)` arm (`operands.len() >= 4`), and not already marked by
/// any sibling emit-projection.
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate { op, operands, .. } = node else {
        return false;
    };
    if !matches!(op, GateOp::CasezMux) {
        return false;
    }
    // A selector + at least one full `(value, mask, data)` arm.
    if operands.len() < 4 {
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
    // Sibling-projection exclusion (the pass runs last, after `case_mux_if`).
    // Vacuous for a `CasezMux` in practice — no other pass marks one — but kept
    // for robustness.
    if m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
        || m.mux_if_gates.contains(&id)
        || m.case_mux_if_gates.contains(&id)
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

/// Mark qualifying dynamic-selector `CasezMux` gates for the procedural
/// `always_comb` `if`/`else if` **masked** priority-chain emit-projection by
/// rolling `prob` per qualifying gate on the seeded generator RNG. Returns the
/// number newly marked. Callers must gate on `prob > 0.0` so the default path is
/// byte-identical (draws nothing). Single-call per module (mirrors the
/// `case_mux_if_emit` call-site roll). Must run **last** — after every sibling
/// projection pass — so their marks are visible and excluded here.
pub fn annotate_casez_mux_if_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted widths are
    // symbolic; the param/structured cross-product is out of scope). Mirrors the
    // case_mux_if pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.casez_mux_if_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.casez_mux_if_gates.insert(id) {
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

    /// `y = casez (sel) 2'b0?:a 2'b1?:b` over a 2-bit dynamic selector + two
    /// 4-bit inputs — node 6 is the `CasezMux` (the chain candidate). The arms
    /// use `wildcard_bits = 1` (mask `2'b01`), exactly like `build_casez_patterns`.
    fn module_casez_mux_gate() -> Module {
        let mut m = Module {
            name: "cz".into(),
            ..Module::default()
        };
        for (id, name, width) in [(0u32, "sel", 2u32), (1, "a", 4), (2, "b", 4)] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width,
                dir: Direction::In,
            });
        }
        m.outputs.push(Port {
            id: 3,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 }); // id 0 (sel)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // id 1 (a)
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // id 2 (b)
        m.nodes.push(Node::Constant { value: 0, width: 2 }); // id 3 (arm0 pattern 2'b00)
        m.nodes.push(Node::Constant { value: 1, width: 2 }); // id 4 (wildcard mask 2'b01, shared)
        m.nodes.push(Node::Constant { value: 2, width: 2 }); // id 5 (arm1 pattern 2'b10)
        m.nodes.push(Node::Gate {
            op: GateOp::CasezMux,
            // [sel, (value0, mask0, data0), (value1, mask1, data1)]
            operands: vec![0, 3, 4, 1, 5, 4, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 6
        m.drives.push((3, 6));
        m
    }

    /// The same shape but with a **constant** selector — statically collapsed by
    /// the emitter, never a chain candidate.
    fn module_static_casez_mux_gate() -> Module {
        let mut m = module_casez_mux_gate();
        // Replace the selector (node 0) with a constant. The CasezMux still
        // references operand 0; node 0 becomes a Constant.
        m.nodes[0] = Node::Constant { value: 1, width: 2 };
        m
    }

    #[test]
    fn prob_one_marks_a_dynamic_casez_mux() {
        let mut m = module_casez_mux_gate();
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.casez_mux_if_gates.contains(&6));
    }

    #[test]
    fn constant_selector_casez_mux_is_excluded() {
        let mut m = module_static_casez_mux_gate();
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a constant-selector CasezMux is statically collapsed");
        assert!(m.casez_mux_if_gates.is_empty());
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_casez_mux_gate();
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.casez_mux_if_gates.is_empty());
    }

    #[test]
    fn case_mux_gate_does_not_qualify() {
        // A plain (non-wildcard) CaseMux is the eighth surface's candidate, not
        // this one.
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
        m.nodes.push(Node::PrimaryInput { port: 0, width: 2 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 3, width: 4 });
        m.nodes.push(Node::Gate {
            op: GateOp::CaseMux,
            operands: vec![0, 1, 2, 3],
            width: 4,
            deps: DepSet::new(),
        });
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.casez_mux_if_gates.is_empty());
    }

    #[test]
    fn plain_mux_gate_does_not_qualify() {
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
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.casez_mux_if_gates.is_empty());
    }

    #[test]
    fn sibling_marked_casez_mux_is_excluded() {
        // A CasezMux already marked by a sibling projection (artificial — no real
        // pass marks a CasezMux) is never also chained: the robustness exclusion.
        // Includes the eighth surface's `case_mux_if_gates`.
        let mut m = module_casez_mux_gate();
        m.case_mux_if_gates.insert(6);
        assert_eq!(annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0), 0);
        assert!(m.casez_mux_if_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_casez_mux_gate();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR node and
        // does not change `canonical_module_signature`.
        let mut m = module_casez_mux_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_casez_mux_if_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked dynamic `CasezMux` renders a
    /// behaviour-preserving `always_comb` masked `if`/`else if` priority chain,
    /// and the default (unmarked) emission is the parallel `casez` — proving the
    /// projection is opt-in and byte-identical by default.
    #[test]
    fn marked_casez_mux_emits_masked_priority_chain_unmarked_is_casez() {
        use crate::emit::to_sv;

        // Unmarked baseline: the parallel `casez` statement.
        let base = to_sv(&module_casez_mux_gate());
        assert!(
            base.contains("casez (sel)"),
            "default-off emission is the parallel casez:\n{base}"
        );
        assert!(
            base.contains("endcase"),
            "default-off emission closes with endcase:\n{base}"
        );

        // Marked: the gate is projected to a masked `if`/`else if` priority chain.
        // arm0: value 2'b00, mask 2'b01 ⇒ care_mask = ~01 & 11 = 2'b10 = 2'h2,
        //       value_masked = 00 & 10 = 0 ⇒ `(sel & 2'h2) == 2'h0`.
        // arm1: value 2'b10, mask 2'b01 ⇒ care_mask = 2'h2,
        //       value_masked = 10 & 10 = 2'b10 = 2'h2 ⇒ `(sel & 2'h2) == 2'h2`.
        let mut marked = module_casez_mux_gate();
        marked.casez_mux_if_gates.insert(6);
        let out = to_sv(&marked);
        assert!(
            out.contains("    always_comb begin"),
            "a procedural always_comb block is emitted:\n{out}"
        );
        assert!(
            out.contains("if ((sel & 2'h2) == 2'h0)"),
            "the first arm is a masked equality test:\n{out}"
        );
        assert!(
            out.contains("else if ((sel & 2'h2) == 2'h2)"),
            "subsequent arms are masked `else if` tests:\n{out}"
        );
        // The trailing `else` carries the former `default` value.
        assert!(
            out.contains("'h0;") && out.contains("        else "),
            "the trailing else carries the default value:\n{out}"
        );
        // The parallel `casez`/`endcase` is suppressed for the marked gate.
        assert!(
            !out.contains("casez (sel)"),
            "the parallel casez is suppressed:\n{out}"
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
