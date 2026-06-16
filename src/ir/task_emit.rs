//! `STRUCTURED-EMISSION-EXPANSION.6b.1` — post-construction annotation
//! that marks selected combinational gates for the combinational
//! `task automatic` emit-projection (decision `0014` + the `.6a`
//! design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The third richer-structured surface: a marked gate is rendered by the
//! emitter as a behaviour-preserving combinational `task automatic`
//!
//! ```systemverilog
//! task automatic <gate>__t(output logic [W-1:0] o, input logic [W0-1:0] a0, ...);
//!     o = a0 <op> a1 ...;
//! endtask
//! ...
//! logic [W-1:0] <gate>__tv;
//! always_comb <gate>__t(<gate>__tv, <operand refs>);
//! assign <gate> = <gate>__tv;   // the gate's net, unchanged downstream
//! ```
//!
//! instead of the inline `assign <gate> = <op>;`. The task writes exactly
//! the gate's value into the `<gate>__tv` output var (the body is the same
//! operation over positional parameters bound to the gate's direct
//! operands), and the existing `<gate>` net is driven from it — so the
//! projection is **behaviour-preserving by construction**. This is the
//! decision `0012` single-gate `function automatic` parallel, but expressed
//! as a *procedural* `task` with an `output` argument called from
//! `always_comb` rather than a value-returning `function`. Positional
//! parameters — not node-id-mapped — so a gate with duplicate operands
//! (e.g. `x & x`) renders one parameter per operand slot.
//!
//! **Rules-first, never generate-then-filter.** The task re-expresses a
//! gate that is already valid in the flat emission; selection happens here
//! at construction time. There is nothing to check-and-discard.
//!
//! **Non-rolling annotation, rolled at the call site like every other
//! knob.** The per-gate decision is a seeded `gen_bool(prob)` here
//! (reproducible; never `thread_rng`). The generator guards the call on
//! `Config::task_emit_prob > 0.0`, so the default (`0.0`) draws nothing
//! from the RNG and marks nothing ⇒ byte-identical stream + output. The
//! annotation is an emitter-surface marker only: the flat IR body,
//! validators, CSE keys and `canonical_module_signature` are all untouched.
//! Mirrors `crate::ir::function_emit::annotate_function_emit_gates`.
//!
//! **Mutually exclusive with the sibling projections.** A gate is projected
//! by at most one of `function_emit` / `generate_loop` / `task_emit` /
//! `soft_union`. This pass runs **after** the others and excludes any gate
//! already marked there (the established "later pass excludes earlier
//! marks" ordering — `function_emit` runs after `soft_union`,
//! `generate_loop` runs after `function_emit`).

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;

/// True iff the gate at `id` qualifies for the combinational
/// `task automatic` projection: a *computational* `Node::Gate` that is
///
/// - **not** a procedural structured block (`CaseMux` / `CasezMux` /
///   `ForFold` — those already have their own `always_comb` rendering);
/// - **not** a `Slice` bit-select — like `function_emit`, a full-width task
///   parameter for a bit-select would leave the unused bits flagged
///   `UNUSEDSIGNAL` under `verilator -Wall`. `Slice` still emits inline —
///   nothing is retired;
/// - has at least one operand;
/// - is **not** already marked for any sibling emit-projection
///   (`function_emit_gates` / `generate_loop_gates` / `soft_union_slice_gates`
///   — the projections are mutually exclusive on a gate, and this pass runs
///   after the others).
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
    if m.function_emit_gates.contains(&id) {
        return false;
    }
    if m.generate_loop_gates.contains(&id) {
        return false;
    }
    if m.soft_union_slice_gates.contains(&id) {
        return false;
    }
    true
}

/// Mark qualifying combinational gates for the `task automatic`
/// emit-projection by rolling `prob` per qualifying gate on the seeded
/// generator RNG. Returns the number newly marked. Callers must gate on
/// `prob > 0.0` so the default path is byte-identical (draws nothing).
/// Single-call per module (mirrors the `function_emit` / `generate_loop` /
/// `soft_union` call-site roll). Must run **after**
/// `annotate_function_emit_gates` and `annotate_generate_loop_gates` so
/// those marks are visible and excluded here.
pub fn annotate_task_emit_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (their emitted
    // widths are symbolic; the param/structured cross-product is out of
    // scope). Mirrors the function_emit pass scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does
    // not overlap the mutable insert into `m.task_emit_gates`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| gate_qualifies(m, *i as NodeId, n))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut marked = 0usize;
    for id in candidates {
        if rng.gen_bool(p) && m.task_emit_gates.insert(id) {
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
    /// gate (the task-emit candidate).
    fn module_and_gate() -> Module {
        let mut m = Module {
            name: "te".into(),
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
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert!(m.task_emit_gates.contains(&2));
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_and_gate();
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
    }

    #[test]
    fn structured_gate_does_not_qualify() {
        // A CaseMux is a procedural structured block, emitted via
        // always_comb — never a task-emit candidate.
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
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
    }

    #[test]
    fn slice_gate_does_not_qualify() {
        // A `Slice` bit-select uses only a sub-range of its operand, so a
        // full-width task parameter would leave unused bits — excluded from
        // the candidate set (still emitted inline).
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
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
    }

    #[test]
    fn function_emit_marked_gate_is_excluded() {
        // A gate already marked for the `function automatic` projection is
        // never also task-emitted (the projections are mutually exclusive on
        // a gate; this pass runs after function_emit).
        let mut m = module_and_gate();
        m.function_emit_gates.insert(2);
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
    }

    #[test]
    fn generate_loop_marked_gate_is_excluded() {
        // A gate already marked for the `generate for` loop projection is
        // never also task-emitted (mutually exclusive; this pass runs after
        // generate_loop).
        let mut m = module_and_gate();
        m.generate_loop_gates.insert(2);
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
    }

    #[test]
    fn soft_union_marked_gate_is_excluded() {
        // A gate already marked for the `union soft` overlay is never also
        // task-emitted.
        let mut m = module_and_gate();
        m.soft_union_slice_gates.insert(2);
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.task_emit_gates.is_empty());
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
        let n = annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR
        // node and does not change `canonical_module_signature`.
        let mut m = module_and_gate();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_task_emit_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// The end-to-end emit proof: a marked gate renders a behaviour-
    /// preserving `task automatic` declaration + an `always_comb` call into a
    /// `<wire>__tv` output var + a passthrough `assign`, and the default
    /// (unmarked) emission is the plain inline assign — proving the
    /// projection is opt-in and byte-identical by default.
    #[test]
    fn marked_gate_emits_task_and_call_unmarked_is_inline() {
        use crate::emit::to_sv;

        // Unmarked baseline: the plain inline assign, no task.
        let base = to_sv(&module_and_gate());
        assert!(
            !base.contains("task automatic"),
            "default-off emission has no task:\n{base}"
        );
        assert!(
            base.contains("assign and_0 = a & b;"),
            "default-off emission is the inline gate assign:\n{base}"
        );

        // Marked: the gate is projected to a `task automatic` + always_comb
        // call + passthrough assign.
        let mut marked = module_and_gate();
        marked.task_emit_gates.insert(2);
        let out = to_sv(&marked);
        assert!(
            out.contains("task automatic and_0__t(output logic [3:0] o, input logic [3:0] a0, input logic [3:0] a1);"),
            "marked gate declares a task with an output var + operand params:\n{out}"
        );
        assert!(
            out.contains("o = a0 & a1;"),
            "task body is the behaviour-preserving op over positional params:\n{out}"
        );
        assert!(
            out.contains("endtask"),
            "task declaration is closed:\n{out}"
        );
        assert!(
            out.contains("logic [3:0] and_0__tv;"),
            "the task-output var is declared:\n{out}"
        );
        assert!(
            out.contains("always_comb and_0__t(and_0__tv, a, b);"),
            "the task is called from always_comb into the output var:\n{out}"
        );
        assert!(
            out.contains("assign and_0 = and_0__tv;"),
            "the gate's assign becomes a passthrough from the task var:\n{out}"
        );
        // The inline gate op is suppressed for the marked gate.
        assert!(
            !out.contains("assign and_0 = a & b;"),
            "the inline gate assign is suppressed:\n{out}"
        );
        // The output port is still driven from the gate wire unchanged.
        assert!(
            out.contains("assign y = and_0;"),
            "the output drive is unchanged:\n{out}"
        );
    }

    /// Duplicate operands get distinct positional parameters (the reason the
    /// body is positional, not node-id-mapped).
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
        m.task_emit_gates.insert(1);
        let out = to_sv(&m);
        assert!(
            out.contains("o = a0 & a1;"),
            "duplicate operands render as distinct positional params:\n{out}"
        );
        assert!(
            out.contains("always_comb and_0__t(and_0__tv, a, a);"),
            "the call passes the same ref into both positions:\n{out}"
        );
    }
}
