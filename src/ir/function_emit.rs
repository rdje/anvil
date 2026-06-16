//! `STRUCTURED-EMISSION-EXPANSION.2b.1` — post-construction annotation
//! that marks selected combinational gates for the combinational
//! `function automatic` emit-projection (decision `0012` + the `.2a`
//! design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The first richer-structured surface: a marked gate is rendered by the
//! emitter as a behaviour-preserving combinational `function automatic`
//!
//! ```systemverilog
//! function automatic logic [W-1:0] <gate>__f(input logic [W0-1:0] a0, ...);
//!     <gate>__f = a0 <op> a1 ...;
//! endfunction
//! ...
//! assign <gate> = <gate>__f(<operand refs>);
//! ```
//!
//! instead of the inline `assign <gate> = <op>;`. The function returns
//! exactly the gate's value (the body is the same operation over
//! positional parameters bound to the gate's direct operands), so the
//! projection is **behaviour-preserving by construction**. Positional
//! parameters — not node-id-mapped — so a gate with duplicate operands
//! (e.g. `x & x`, or a replicated `Concat`) renders one parameter per
//! operand slot.
//!
//! **Rules-first, never generate-then-filter.** The function wraps a cone
//! that is already valid in the flat emission; selection happens here at
//! construction time. There is nothing to check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other
//! knob.** The per-gate decision is a seeded `gen_bool(prob)` here
//! (reproducible; never `thread_rng`). The generator guards the call on
//! `Config::function_emit_prob > 0.0`, so the default (`0.0`) draws
//! nothing from the RNG and marks nothing ⇒ byte-identical stream +
//! output. The annotation is an emitter-surface marker only: the flat IR
//! body, validators, CSE keys and `canonical_module_signature` are all
//! untouched. Mirrors `crate::ir::soft_union::annotate_soft_union_slices`.

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the combinational
/// `function automatic` projection: a *computational* `Node::Gate` that is
///
/// - **not** a procedural structured block (`CaseMux` / `CasezMux` /
///   `ForFold` — those are emitted via `always_comb`, not a continuous
///   assign);
/// - **not** a `Slice` bit-select — `Slice` is the one operation that uses
///   only a *sub-range* of its operand, so a full-width function parameter
///   would leave the unused bits flagged `UNUSEDSIGNAL` under
///   `verilator -Wall`. A bit-select is naturally an inline construct; a
///   slice-aware projection (passing only the used sub-range) is a recorded
///   follow-up. Excluding it here keeps the first cut robustly
///   warning-clean. `Slice` still emits inline — nothing is retired;
/// - has at least one operand;
/// - is **not** already marked for the `union soft` overlay (the two
///   emit-projections are mutually exclusive on a gate).
fn gate_qualifies(m: &Module, id: NodeId, node: &Node) -> bool {
    let Node::Gate { op, operands, .. } = node else {
        return false;
    };
    if matches!(
        op,
        GateOp::CaseMux | GateOp::CasezMux | GateOp::ForFold { .. } | GateOp::Slice { .. }
    ) {
        return false;
    }
    if operands.is_empty() {
        return false;
    }
    if m.soft_union_slice_gates.contains(&id) {
        return false;
    }
    true
}

/// Mark qualifying combinational gates for the `function automatic`
/// emit-projection by rolling `prob` per qualifying gate on the seeded
/// generator RNG. Returns the number newly marked. Callers must gate on
/// `prob > 0.0` so the default path is byte-identical (draws nothing).
/// Single-call per module (mirrors the `aggregate_prob` / soft_union
/// call-site roll). Must run **after**
/// `annotate_soft_union_slices` so the `union soft` marks are visible and
/// excluded here.
pub fn annotate_function_emit_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted
    // widths are symbolic; the param/structured cross-product is out of
    // scope). Mirrors the soft_union pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does
    // not overlap the mutable insert into `m.function_emit_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.function_emit_gates.insert(id) {
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

    /// `y = a & b` over two 4-bit inputs — node 2 is a plain combinational
    /// gate (the function-emit candidate).
    fn module_and_gate() -> Module {
        let mut m = Module {
            name: "fe".into(),
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
        m.outputs.push(Port {
            id: 2,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 }); // id 1
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 2
        m.drives.push((2, 2));
        m
    }

    #[test]
    fn prob_one_marks_a_candidate_gate() {
        let mut m = module_and_gate();
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.function_emit_gates.contains(&2));
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_and_gate();
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.function_emit_gates.is_empty());
    }

    #[test]
    fn structured_gate_does_not_qualify() {
        // A CaseMux is a procedural structured block, emitted via
        // always_comb — never a function-emit candidate.
        let mut m = Module {
            name: "cm".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "s".into(),
            width: 1,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // id 0
        m.nodes.push(Node::Constant { width: 4, value: 1 }); // id 1
        m.nodes.push(Node::Constant { width: 4, value: 2 }); // id 2
        m.nodes.push(Node::Gate {
            op: GateOp::CaseMux,
            operands: vec![0, 1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 3
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.function_emit_gates.is_empty());
    }

    #[test]
    fn slice_gate_does_not_qualify() {
        // A `Slice` bit-select uses only a sub-range of its operand, so a
        // full-width function parameter would leave unused bits — excluded
        // from the first-cut candidate set (still emitted inline).
        let mut m = Module {
            name: "sl".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // id 0
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 3, lo: 0 },
            operands: vec![0],
            width: 4,
            deps: DepSet::new(),
        }); // id 1
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.function_emit_gates.is_empty());
    }

    #[test]
    fn soft_union_marked_gate_is_excluded() {
        // A gate already marked for the `union soft` overlay is never also
        // function-emitted (the two emit-projections are mutually
        // exclusive on a gate).
        let mut m = module_and_gate();
        m.soft_union_slice_gates.insert(2);
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.function_emit_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_and_gate();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR
        // node and does not change `canonical_module_signature`.
        let mut m = module_and_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_function_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked gate renders a behaviour-
    /// preserving `function automatic` declaration + a call, and the
    /// default (unmarked) emission is the plain inline assign — proving the
    /// projection is opt-in and byte-identical by default.
    #[test]
    fn marked_gate_emits_function_and_call_unmarked_is_inline() {
        use crate::emit::to_sv;

        // Unmarked baseline: the plain inline assign, no function.
        let base = to_sv(&module_and_gate());
        assert!(
            !base.contains("function automatic"),
            "default-off emission has no function:\n{base}"
        );
        assert!(
            base.contains("assign and_0 = a & b;"),
            "default-off emission is the inline gate assign:\n{base}"
        );

        // Marked: the gate is projected to a `function automatic` + call.
        let mut marked = module_and_gate();
        marked.function_emit_gates.insert(2);
        let out = to_sv(&marked);
        assert!(
            out.contains("function automatic logic [3:0] and_0__f(input logic [3:0] a0, input logic [3:0] a1);"),
            "marked gate declares a function over its operand widths:\n{out}"
        );
        assert!(
            out.contains("and_0__f = a0 & a1;"),
            "function body is the behaviour-preserving op over positional params:\n{out}"
        );
        assert!(
            out.contains("endfunction"),
            "function declaration is closed:\n{out}"
        );
        assert!(
            out.contains("assign and_0 = and_0__f(a, b);"),
            "the gate's assign becomes a call passing the operand refs:\n{out}"
        );
        // The output port is still driven from the gate wire unchanged.
        assert!(
            out.contains("assign y = and_0;"),
            "the output drive is unchanged:\n{out}"
        );
    }

    /// Duplicate operands get distinct positional parameters (the reason
    /// the body is positional, not node-id-mapped).
    #[test]
    fn duplicate_operands_use_distinct_positional_params() {
        use crate::emit::to_sv;
        // y = a & a — the same input feeds both operand slots.
        let mut m = Module {
            name: "dup".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // id 0
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 0],
            width: 4,
            deps: DepSet::new(),
        }); // id 1
        m.drives.push((1, 1));
        m.function_emit_gates.insert(1);
        let out = to_sv(&m);
        assert!(
            out.contains("and_0__f = a0 & a1;"),
            "duplicate operands render as distinct positional params:\n{out}"
        );
        assert!(
            out.contains("assign and_0 = and_0__f(a, a);"),
            "the call passes the same ref into both positions:\n{out}"
        );
    }
}
