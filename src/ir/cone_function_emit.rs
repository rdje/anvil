//! `STRUCTURED-EMISSION-EXPANSION.10b` — post-construction annotation that
//! marks a combinational **cone** (a root gate plus its single-use interior
//! gates) for the multi-gate-cone `function automatic` emit-projection
//! (decision `0016` + the `.10a` design-detail in `DEVELOPMENT_NOTES.md`).
//!
//! The fifth richer-structured surface: a marked cone is rendered by the
//! emitter as one behaviour-preserving `function automatic`
//!
//! ```systemverilog
//! function automatic logic [W-1:0] <root>__cf(input logic [..] a0, ...);
//!     logic [..] <g0>;            // one function-local per interior gate,
//!     logic [..] <g1>;            // declared up front
//!     <g0> = <expr over params / earlier locals / literals>;
//!     <g1> = <expr ...>;
//!     <root>__cf = <expr ...>;    // returns the root
//! endfunction
//! ...
//! assign <root> = <root>__cf(<boundary-leaf refs>);
//! ```
//!
//! instead of the inline per-gate `assign` chain. The function's parameters
//! are the cone's **boundary leaves** (every operand of the root or an interior
//! gate that is not itself an interior gate and not a constant), its body is a
//! topo-ordered sequence of function-locals (one per absorbed interior gate,
//! constants folded inline as literals), and it returns the root — so the
//! function evaluates to exactly the cone's value: **behaviour-preserving by
//! construction**.
//!
//! This **deepens** the decision `0012` single-gate `function_emit` surface
//! (one gate over its direct operands, a one-line body) to a whole cone (a
//! multi-statement body with function-local declarations) — a genuinely new
//! emitted shape. It uses its **own** knob (`Config::cone_function_emit_prob`),
//! so the shipped single-gate surface stays byte-identical; nothing is retired.
//!
//! **Rules-first, never generate-then-filter.** The function wraps a cone that
//! is already valid in the flat emission; selection happens here at construction
//! time. There is nothing to check-and-discard.
//!
//! **Single-use absorption (the soundness rule).** An interior gate is absorbed
//! only when it is used **exactly once in the entire module** — so its sole
//! consumer is the cone edge that reached it, and suppressing both its
//! module-level `wire` declaration and its inline `assign` (the gate now lives
//! only inside the function) is provably safe: nothing else reads it. A
//! multi-use (DAG-shared) gate stays a boundary parameter, keeping its own
//! module wire + assign. This is the conservative realization of the
//! `.10a` "single-use-within-cone" rule (global-use-count `== 1` is a sufficient
//! subset; broadening to true within-cone sharing is a recorded follow-up).
//!
//! **Non-rolling annotation, rolled at the call site like every other knob.**
//! The per-cone decision is a seeded `gen_bool(prob)` here (reproducible; never
//! `thread_rng`). The generator guards the call on
//! `Config::cone_function_emit_prob > 0.0`, so the default (`0.0`) draws nothing
//! and marks nothing ⇒ byte-identical stream + output. The annotation is an
//! emitter-surface marker only: the flat IR body, validators, CSE keys and
//! `canonical_module_signature` are all untouched. Mirrors
//! `crate::ir::function_emit::annotate_function_emit_gates`. Runs **last** in
//! the call-site projection chain (after `soft_union` / `function_emit` /
//! `generate_loop` / `task_emit`) so those marks are visible and excluded — a
//! sibling-marked gate is never a cone root and never an absorbed interior.

use crate::ir::{FlopMux, GateOp, Module, Node, NodeId};
use rand::Rng;
use std::collections::BTreeSet;

/// True iff `node` is a *computational* `Node::Gate` admissible as a cone root
/// or an absorbed interior: not a procedural structured block
/// (`CaseMux` / `CasezMux` / `ForFold`), not a `Slice` bit-select (a full-width
/// parameter would trip `-Wall UNUSEDSIGNAL`; a `Slice` is naturally inline),
/// and with at least one operand. The same admissible op set as
/// `crate::ir::function_emit::gate_qualifies`.
fn admissible(node: &Node) -> bool {
    matches!(
        node,
        Node::Gate { op, operands, .. }
            if !matches!(
                op,
                GateOp::CaseMux | GateOp::CasezMux | GateOp::ForFold { .. } | GateOp::Slice { .. }
            ) && !operands.is_empty()
    )
}

/// True iff the gate at `id` is already marked for one of the four sibling
/// emit-projections (function / generate-loop / task / soft-union). Such a gate
/// is never a cone root and never an absorbed interior (the cone pass runs last
/// and is mutually exclusive with the siblings).
fn sibling_marked(m: &Module, id: NodeId) -> bool {
    m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
}

/// Count every value-consumer reference to each node across the whole module:
/// gate operands, output drives, flop `D` / mux refs, and instance inputs. A
/// gate with a count of exactly `1` is used by a single consumer, so it can be
/// safely absorbed into a cone (its module wire + assign are then suppressed).
fn compute_use_counts(m: &Module) -> Vec<u32> {
    let mut uc = vec![0u32; m.nodes.len()];
    let mut bump = |id: NodeId| {
        let i = id as usize;
        if i < uc.len() {
            uc[i] += 1;
        }
    };
    for node in &m.nodes {
        if let Node::Gate { operands, .. } = node {
            for &op in operands {
                bump(op);
            }
        }
    }
    for &(_port, nid) in &m.drives {
        bump(nid);
    }
    for f in &m.flops {
        if let Some(d) = f.d {
            bump(d);
        }
        match &f.mux {
            FlopMux::None => {}
            FlopMux::OneHot(arms) => {
                for arm in arms {
                    bump(arm.data);
                    bump(arm.sel);
                }
            }
            FlopMux::Encoded { sel, data } => {
                bump(*sel);
                for &d in data {
                    bump(d);
                }
            }
        }
    }
    for inst in &m.instances {
        for &(_p, nid) in &inst.inputs {
            bump(nid);
        }
    }
    uc
}

/// True iff operand `o` may be absorbed as a cone interior local: an admissible
/// gate, used exactly once in the module, not already claimed by another cone,
/// and not sibling-marked.
fn should_absorb(m: &Module, o: NodeId, use_count: &[u32], claimed: &BTreeSet<NodeId>) -> bool {
    admissible(&m.nodes[o as usize])
        && use_count[o as usize] == 1
        && !claimed.contains(&o)
        && !sibling_marked(m, o)
}

/// Post-order fan-in walk from `node`, pushing every absorbable interior gate
/// into `interior` *after* its absorbable children — yielding a topological
/// order (children before parents) for the function body. The root is never
/// pushed (it is the return value). Boundary operands (inputs / flop `Q`s /
/// instance outputs / constants / multi-use or sibling-marked gates) stop the
/// walk and become parameters.
fn absorb_children(
    m: &Module,
    node: NodeId,
    use_count: &[u32],
    claimed: &BTreeSet<NodeId>,
    visited: &mut BTreeSet<NodeId>,
    interior: &mut Vec<NodeId>,
) {
    let operands: Vec<NodeId> = match &m.nodes[node as usize] {
        Node::Gate { operands, .. } => operands.clone(),
        _ => return,
    };
    for op in operands {
        if !visited.contains(&op) && should_absorb(m, op, use_count, claimed) {
            visited.insert(op);
            absorb_children(m, op, use_count, claimed, visited, interior);
            interior.push(op);
        }
    }
}

/// The topo-ordered absorbed interior gates of the cone rooted at `root`.
fn collect_cone(
    m: &Module,
    root: NodeId,
    use_count: &[u32],
    claimed: &BTreeSet<NodeId>,
) -> Vec<NodeId> {
    let mut interior = Vec::new();
    let mut visited = BTreeSet::new();
    absorb_children(m, root, use_count, claimed, &mut visited, &mut interior);
    interior
}

/// Mark qualifying combinational cones for the multi-gate-cone `function
/// automatic` emit-projection by rolling `prob` per qualifying cone on the
/// seeded generator RNG. Returns the number of cones newly marked. Callers must
/// gate on `prob > 0.0` so the default path is byte-identical (draws nothing).
/// Must run **after** the sibling projection passes (`soft_union` /
/// `function_emit` / `generate_loop` / `task_emit`) so their marks are visible
/// and excluded. Mirrors the `annotate_function_emit_gates` call-site roll.
pub fn annotate_cone_function_gates(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Phase-5 parameterized modules are out of scope (symbolic widths; the
    // param/structured cross-product is out of scope). Mirrors the sibling
    // passes' scoping.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    let use_count = compute_use_counts(m);
    // Greedy in node order: a committed cone claims its root + interiors so a
    // later root cannot re-absorb them (a single-use interior can only be
    // reached by one consumer anyway, so this is belt-and-suspenders for the
    // cross-cone case). The plan is collected first (immutable scan), then
    // committed into `m.cone_function_gates` so the scan does not overlap the
    // mutable insert.
    let mut claimed: BTreeSet<NodeId> = BTreeSet::new();
    let mut plans: Vec<(NodeId, Vec<NodeId>)> = Vec::new();
    for idx in 0..m.nodes.len() {
        let root = idx as NodeId;
        if !admissible(&m.nodes[idx]) || claimed.contains(&root) || sibling_marked(m, root) {
            continue;
        }
        let interior = collect_cone(m, root, &use_count, &claimed);
        // A cone needs at least one absorbed interior gate, else it is just the
        // single-gate surface (left to `function_emit`).
        if interior.is_empty() {
            continue;
        }
        if rng.gen_bool(p) {
            for &g in &interior {
                claimed.insert(g);
            }
            claimed.insert(root);
            plans.push((root, interior));
        }
    }
    let marked = plans.len();
    for (root, interior) in plans {
        m.cone_function_gates.insert(root, interior);
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

    /// `y = (a & b) | c` — node 3 (`and_0`) is a single-use interior gate, node
    /// 4 (`or_0`) is the cone root (driven to output `y`).
    fn module_cone() -> Module {
        let mut m = Module {
            name: "cf".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b"), (2, "c")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        m.outputs.push(Port {
            id: 3,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 3 — interior
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![3, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 4 — root
        m.drives.push((3, 4)); // output y <- node 4
        m
    }

    #[test]
    fn prob_one_marks_a_cone_with_its_interior() {
        let mut m = module_cone();
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        assert_eq!(
            m.cone_function_gates.get(&4),
            Some(&vec![3]),
            "root 4 absorbs single-use interior gate 3"
        );
    }

    #[test]
    fn prob_zero_marks_nothing_byte_identical() {
        let mut m = module_cone();
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.cone_function_gates.is_empty());
    }

    #[test]
    fn a_zero_interior_root_is_not_marked() {
        // `y = a & b` — the root's only operands are inputs, no interior gate to
        // absorb, so the cone is empty (left to the single-gate `function_emit`).
        let mut m = Module {
            name: "flat".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        m.outputs.push(Port {
            id: 2,
            name: "y".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 2 — root, no interior
        m.drives.push((2, 2));
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a single-gate cone has no interior to absorb");
        assert!(m.cone_function_gates.is_empty());
    }

    #[test]
    fn a_multi_use_interior_gate_is_not_absorbed() {
        // `g = a & b` feeds BOTH outputs y0 and y1, so `g` is used twice and
        // stays a boundary param — neither cone has an absorbable interior.
        let mut m = Module {
            name: "shared".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        for (id, name) in [(2u32, "y0"), (3, "y1")] {
            m.outputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::Out,
            });
        }
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 2 — g (shared)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![2, 0],
            width: 4,
            deps: DepSet::new(),
        }); // id 3 — root0
        m.nodes.push(Node::Gate {
            op: GateOp::Xor,
            operands: vec![2, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 4 — root1
        m.drives.push((2, 3));
        m.drives.push((3, 4));
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a multi-use interior gate stays a boundary param");
        assert!(m.cone_function_gates.is_empty());
    }

    #[test]
    fn a_sibling_marked_gate_is_excluded() {
        // The interior gate is already function-emit-marked, so it is neither a
        // cone root nor an absorbable interior — the cone is empty.
        let mut m = module_cone();
        m.function_emit_gates.insert(3);
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0);
        assert!(m.cone_function_gates.is_empty());
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_cone();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    #[test]
    fn marking_leaves_identity_and_node_count_untouched() {
        // The mark is an emitter-surface annotation only: it adds no IR node and
        // does not change `canonical_module_signature`.
        let mut m = module_cone();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_cone_function_gates(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    /// End-to-end emit proof: a marked cone renders a behaviour-preserving
    /// multi-statement `function automatic` with a function-local + a call, the
    /// interior gate's inline assign and module wire are suppressed, and the
    /// default (unmarked) emission is the plain inline chain.
    #[test]
    fn marked_cone_emits_multi_statement_function_unmarked_is_inline() {
        use crate::emit::to_sv;

        // Unmarked baseline: the inline per-gate chain, no function.
        let base = to_sv(&module_cone());
        assert!(
            !base.contains("__cf"),
            "default-off emission has no cone function:\n{base}"
        );
        assert!(
            base.contains("assign and_0 = a & b;"),
            "default-off emission has the inline interior assign:\n{base}"
        );
        assert!(
            base.contains("assign or_0 = and_0 | c;"),
            "default-off emission has the inline root assign:\n{base}"
        );

        // Marked: the cone is projected to one `function automatic`.
        let mut marked = module_cone();
        marked.cone_function_gates.insert(4, vec![3]);
        let out = to_sv(&marked);
        assert!(
            out.contains("function automatic logic [3:0] or_0__cf(input logic [3:0] a0, input logic [3:0] a1, input logic [3:0] a2);"),
            "cone root declares a function over the boundary leaves:\n{out}"
        );
        assert!(
            out.contains("        logic [3:0] and_0;"),
            "the interior gate becomes a function-local:\n{out}"
        );
        assert!(
            out.contains("        and_0 = a0 & a1;"),
            "interior body statement uses the params:\n{out}"
        );
        assert!(
            out.contains("        or_0__cf = and_0 | a2;"),
            "the return uses the local + a param:\n{out}"
        );
        assert!(
            out.contains("assign or_0 = or_0__cf(a, b, c);"),
            "the root's assign becomes a call over the boundary refs:\n{out}"
        );
        // The interior gate's module wire + inline assign are suppressed.
        assert!(
            !out.contains("wire [3:0] and_0;"),
            "the absorbed interior's module wire is suppressed:\n{out}"
        );
        assert!(
            !out.contains("assign and_0 = a & b;"),
            "the absorbed interior's inline assign is suppressed:\n{out}"
        );
        // The output drive is unchanged.
        assert!(
            out.contains("assign y = or_0;"),
            "the output drive is unchanged:\n{out}"
        );
    }
}
