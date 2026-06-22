//! `STRUCTURED-EMISSION-EXPANSION.12b.1` (pair) + `.13b` (wider `k > 2` groups) —
//! post-construction annotation that groups a co-supported set of combinational
//! gates for the multi-output combinational `task automatic` emit-projection
//! (decision `0025` + the `.12a` / `.13a` design-details in `DEVELOPMENT_NOTES.md`).
//!
//! The sixth richer-structured surface: a co-supported group of gates is
//! co-emitted by the emitter as one behaviour-preserving `task automatic` with
//! several `output` arguments and a **deduplicated** `input` list:
//!
//! ```systemverilog
//! task automatic <leader>__mt(output logic [W0-1:0] o0, output logic [W1-1:0] o1,
//!                             input logic [..] a0, input logic [..] a1, ...);
//!     o0 = <op0 over the shared formals a*>;
//!     o1 = <op1 over the shared formals a*>;
//! endtask
//! ...
//! logic [W0-1:0] <m0>__mtv;
//! logic [W1-1:0] <m1>__mtv;
//! always_comb <leader>__mt(<m0>__mtv, <m1>__mtv, <deduped operand refs>);
//! assign <m0> = <m0>__mtv;   // each member's net, unchanged downstream
//! assign <m1> = <m1>__mtv;
//! ```
//!
//! instead of the two inline `assign`s. A **shared** non-constant operand becomes
//! **one** input formal feeding multiple outputs (the genuine "co-supported
//! sink"); a `Constant` operand folds inline as a literal (the cone-function
//! precedent). Each output is the member gate's exact operation over those
//! formals, so the task is **behaviour-preserving by construction.**
//!
//! **Group size (`STRUCTURED-EMISSION-EXPANSION.13`).** The group is a `k >= 2`
//! co-supported set, bounded by [`MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS`]. The first
//! cut shipped a **pair** (`k = 2`, decision `0025`); `.13` deepens it to wider
//! groups (the recorded `.13` follow-up). The emitter was already written
//! k-agnostic (`render_multi_output_task_decl` / `render_multi_output_task_call` /
//! `multi_output_task_params` iterate `members` of any length) and the carrier
//! `Module.multi_output_task_groups: BTreeMap<NodeId, Vec<NodeId>>` holds `k - 1`
//! partners, so the widening lives entirely in this pass.
//!
//! **The soundness rule (mutual fan-in independence).** A member is admissible only
//! when no member lies in another member's transitive fan-in. If member `gb` were
//! in `ga`'s fan-in, `gb`'s net — driven by the shared task's `<gb>__mtv`
//! passthrough — would feed, through gates *outside* the task, into a direct
//! operand the task reads, closing a combinational cycle through the single
//! `always_comb` task call (a Verilator `UNOPTFLAT` even though it converges
//! functionally). Requiring **pairwise** mutual fan-in independence (each new
//! member checked against *every* current member, [`independent_of_all`]) makes the
//! co-emitted task cycle-free by construction at any group size — the multi-output
//! analogue of the cone function's single-use rule. The check is a bounded backward
//! DFS over the operand DAG;
//! because `Module::intern_gate` appends a gate after its operands, a gate's
//! operands always have strictly smaller `NodeId`, so for `ga < gb` the direction
//! `gb ∈ fanin(ga)` is automatically false — but both directions are checked for
//! robustness against any future invariant change.
//!
//! **Rules-first, never generate-then-filter.** The task re-expresses two gates
//! that are already valid in the flat emission; selection happens here at
//! construction time. There is nothing to check-and-discard.
//!
//! **One roll per leader, then greedy extension.** For the lowest ungrouped
//! candidate the pass rolls a seeded `gen_bool(prob)` once (reproducible; never
//! `thread_rng`); if it fires it greedily admits **every** ungrouped higher-`NodeId`
//! candidate that (1) shares a non-constant operand with at least one current member
//! ([`connected_co_support`]) and (2) is mutually fan-in-independent with every
//! current member ([`independent_of_all`]), up to
//! [`MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS`]. A group forms iff at least one partner is
//! admitted, so `k = 2` is the exact subset (the pair behaviour is unchanged when
//! only one partner qualifies). The generator guards the call on
//! `Config::multi_output_task_emit_prob > 0.0`, so the default (`0.0`) draws
//! nothing and groups nothing ⇒ byte-identical stream + output. The annotation is
//! an emitter-surface marker only: the flat IR body, validators, CSE keys and
//! `canonical_module_signature` are all untouched. Mirrors
//! `crate::ir::task_emit::annotate_task_emit_gates`.
//!
//! **Mutually exclusive with the sibling projections.** A gate is projected by at
//! most one of `function_emit` / `generate_loop` / `task_emit` /
//! `multi_output_task` / `cone_function` / `soft_union`. This pass runs **after**
//! `task_emit` (excludes its single-gate marks) and **before** `cone_function`
//! (which excludes multi-output members as roots / interiors).

use crate::ir::{GateOp, Module, Node, NodeId};
use rand::Rng;
use std::collections::BTreeSet;

/// Upper bound on the number of gates co-emitted into a single multi-output
/// `task automatic` (`STRUCTURED-EMISSION-EXPANSION.13`): the leader plus up to
/// `MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS - 1` partners. A bounded, reviewable first
/// cut for the `k > 2` widening — it keeps any one task readable and lets
/// **multiple** groups form per module (a leader stops absorbing at the cap; the
/// next ungrouped leader forms its own group) rather than collapsing a dense comb
/// module into one giant task. Raising or removing it is a one-constant follow-up.
const MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS: usize = 8;

/// True iff `node` is a *computational* `Node::Gate` admissible as a multi-output
/// task member: not a procedural structured block (`CaseMux` / `CasezMux` /
/// `ForFold`), not a `Slice` bit-select (a full-width formal would trip `-Wall
/// UNUSEDSIGNAL`), and with at least one operand. The same admissible op set as
/// `crate::ir::task_emit::gate_qualifies` (replicated per the per-pass
/// convention).
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

/// True iff the gate at `id` is already marked for one of the four prior sibling
/// emit-projections (function / generate-loop / task / soft-union). Such a gate is
/// never a multi-output task member (this pass runs after those and is mutually
/// exclusive with them).
fn sibling_marked(m: &Module, id: NodeId) -> bool {
    m.function_emit_gates.contains(&id)
        || m.generate_loop_gates.contains(&id)
        || m.task_emit_gates.contains(&id)
        || m.soft_union_slice_gates.contains(&id)
}

/// The non-constant direct operands of the gate at `id` (constants fold inline as
/// literals, so they never contribute a shared input formal).
fn nonconst_operands(m: &Module, id: NodeId) -> Vec<NodeId> {
    match &m.nodes[id as usize] {
        Node::Gate { operands, .. } => operands
            .iter()
            .copied()
            .filter(|&o| !matches!(m.nodes[o as usize], Node::Constant { .. }))
            .collect(),
        _ => Vec::new(),
    }
}

/// True iff gates `a` and `b` share at least one non-constant direct operand — so
/// the co-emitted task genuinely has a shared input formal feeding both outputs
/// (without this it would merely be two unrelated tasks fused, no new
/// interaction).
fn shares_nonconst_operand(m: &Module, a: NodeId, b: NodeId) -> bool {
    let sa: BTreeSet<NodeId> = nonconst_operands(m, a).into_iter().collect();
    nonconst_operands(m, b).into_iter().any(|o| sa.contains(&o))
}

/// True iff `target` is in the transitive operand (fan-in) cone of `root`: a
/// bounded backward DFS over `Node::Gate` operands with a visited set (each node
/// expanded once ⇒ bounded by the cone size). Leaves (primary inputs / flop `Q`s /
/// instance outputs / constants) have no operands and terminate a branch.
fn in_fanin(m: &Module, target: NodeId, root: NodeId) -> bool {
    let mut stack: Vec<NodeId> = Vec::new();
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    if let Node::Gate { operands, .. } = &m.nodes[root as usize] {
        stack.extend(operands.iter().copied());
    }
    while let Some(n) = stack.pop() {
        if n == target {
            return true;
        }
        if !visited.insert(n) {
            continue;
        }
        if let Node::Gate { operands, .. } = &m.nodes[n as usize] {
            stack.extend(operands.iter().copied());
        }
    }
    false
}

/// True iff candidate `gc` shares at least one non-constant direct operand with at
/// least one gate already in `members` — the **connected co-support** rule that
/// generalizes the pair's [`shares_nonconst_operand`] to a `k > 2` group. It keeps
/// the deduplicated task a genuine shared-formal structure: every shared formal
/// still feeds `>= 2` outputs (the "co-supported sink" essence), and the group
/// stays connected through shared operands rather than fusing unrelated gates.
fn connected_co_support(m: &Module, gc: NodeId, members: &[NodeId]) -> bool {
    members.iter().any(|&gm| shares_nonconst_operand(m, gm, gc))
}

/// True iff candidate `gc` is mutually fan-in-independent with **every** gate in
/// `members` — the generalized soundness rule (the multi-output analogue of the
/// cone-function single-use rule). Checked against all current members (both
/// `in_fanin` directions for robustness), so as the group grows greedily the
/// invariant "all members are pairwise fan-in-independent" is maintained
/// inductively ⇒ the co-emitted task is cycle-free by construction at any size.
fn independent_of_all(m: &Module, gc: NodeId, members: &[NodeId]) -> bool {
    members
        .iter()
        .all(|&gm| !in_fanin(m, gm, gc) && !in_fanin(m, gc, gm))
}

/// Group admissible, non-sibling-marked combinational gates into co-supported
/// `k >= 2` groups for the multi-output `task automatic` emit-projection by rolling
/// `prob` once per leader on the seeded generator RNG, then greedily extending the
/// group up to [`MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS`]. Returns the number of
/// groups newly formed. Callers must gate on `prob > 0.0` so the default path is
/// byte-identical (draws nothing). Single-call per module (mirrors the
/// `task_emit` / `cone_function` call-site roll). Must run **after**
/// `annotate_task_emit_gates` and **before** `annotate_cone_function_gates`.
pub fn annotate_multi_output_task_groups(m: &mut Module, rng: &mut impl Rng, prob: f64) -> usize {
    // Scope: leave Phase 5 parameterized modules out (symbolic widths; the
    // param/structured cross-product is out of scope). Mirrors the task_emit pass.
    if m.param_env.is_some() {
        return 0;
    }
    let p = prob.clamp(0.0, 1.0);
    // Collect candidates first so the immutable scan over `m.nodes` does not
    // overlap the mutable insert into `m.multi_output_task_groups`.
    let candidates: Vec<NodeId> = m
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| admissible(n) && !sibling_marked(m, *i as NodeId))
        .map(|(i, _)| i as NodeId)
        .collect();
    let mut used: BTreeSet<NodeId> = BTreeSet::new();
    let mut groups = 0usize;
    for &ga in &candidates {
        if used.contains(&ga) {
            continue;
        }
        // One roll per ungrouped leader (the per-leader roll is unchanged by the
        // k>2 widening; at the gate's prob=1.0 `gen_bool` short-circuits and draws
        // nothing, and the default prob=0.0 path never calls this pass).
        if !rng.gen_bool(p) {
            continue;
        }
        // Greedily build the group: start with the leader, then admit each
        // ungrouped, higher-NodeId candidate (ascending) that (1) is connected to
        // the current group through a shared non-constant operand and (2) is
        // mutually fan-in-independent with every current member, up to the cap.
        // Scanning ascending keeps the result deterministic.
        let mut members: Vec<NodeId> = vec![ga];
        for &gc in &candidates {
            if members.len() >= MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS {
                break;
            }
            if gc <= ga || used.contains(&gc) {
                continue;
            }
            if !connected_co_support(m, gc, &members) {
                continue;
            }
            if !independent_of_all(m, gc, &members) {
                continue;
            }
            members.push(gc);
        }
        // A group forms iff at least one partner was admitted (so k=2 is the exact
        // subset). Keyed by the leader (lowest NodeId) → the partner members
        // (ascending). Every member is consumed.
        if members.len() >= 2 {
            let partners: Vec<NodeId> = members[1..].to_vec();
            for &g in &members {
                used.insert(g);
            }
            m.multi_output_task_groups.insert(ga, partners);
            groups += 1;
        }
    }
    groups
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

    /// Two independent sibling gates over shared inputs:
    /// `y0 = a & b`, `y1 = b | c` (operand `b` is shared, neither feeds the
    /// other). Nodes: 0=a 1=b 2=c, 3=(a&b), 4=(b|c).
    fn module_two_co_supported_gates() -> Module {
        let mut m = Module {
            name: "mo".into(),
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
            name: "y0".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 4,
            name: "y1".into(),
            width: 4,
            dir: Direction::Out,
        });
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 3
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 4
        m.drives.push((3, 3));
        m.drives.push((4, 4));
        m
    }

    #[test]
    fn prob_one_groups_a_co_supported_pair() {
        let mut m = module_two_co_supported_gates();
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1);
        // Keyed by the lower-NodeId leader (3) → the partner (4).
        assert_eq!(m.multi_output_task_groups.get(&3), Some(&vec![4]));
    }

    #[test]
    fn prob_zero_groups_nothing_byte_identical() {
        let mut m = module_two_co_supported_gates();
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 0.0);
        assert_eq!(n, 0);
        assert!(m.multi_output_task_groups.is_empty());
    }

    #[test]
    fn gates_without_a_shared_nonconst_operand_do_not_pair() {
        // `y0 = a & b`, `y1 = c | d` — disjoint supports, no shared operand.
        let mut m = Module {
            name: "ns".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b"), (2, "c"), (3, "d")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 4
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![2, 3],
            width: 4,
            deps: DepSet::new(),
        }); // id 5
        m.drives.push((0, 4));
        m.drives.push((1, 5));
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(
            n, 0,
            "no shared non-constant operand ⇒ no co-supported group"
        );
    }

    #[test]
    fn a_shared_constant_operand_alone_does_not_pair() {
        // `y0 = a & K`, `y1 = b & K` — they share only the constant K (which
        // folds inline, so no shared *formal*); not co-supported.
        let mut m = Module {
            name: "kc".into(),
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
        m.nodes.push(Node::Constant { width: 4, value: 3 }); // id 2 (K)
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 3
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 4
        m.drives.push((0, 3));
        m.drives.push((1, 4));
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a shared constant is not a shared formal");
    }

    #[test]
    fn fan_in_dependent_gates_do_not_pair() {
        // `g0 = a & b`, `g1 = g0 | b` — g0 is in g1's fan-in; pairing them would
        // close a combinational cycle through the shared task. They share `b`, so
        // only the independence rule excludes them.
        let mut m = Module {
            name: "dep".into(),
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
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 2 (g0)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![2, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 3 (g1 = g0 | b)
        m.drives.push((0, 2));
        m.drives.push((1, 3));
        assert!(in_fanin(&m, 2, 3), "g0 is in g1's fan-in");
        assert!(!in_fanin(&m, 3, 2), "g1 is not in g0's fan-in");
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "fan-in-dependent members must not pair (cycle)");
    }

    #[test]
    fn structured_and_slice_gates_do_not_qualify() {
        let mut m = module_two_co_supported_gates();
        // Replace gate 3 with a Slice — no longer admissible.
        m.nodes[3] = Node::Gate {
            op: GateOp::Slice { hi: 3, lo: 0 },
            operands: vec![0],
            width: 4,
            deps: DepSet::new(),
        };
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "a Slice member is excluded ⇒ no pair");
    }

    #[test]
    fn sibling_marked_gate_is_excluded() {
        // A gate already marked for a sibling projection is never a member.
        let mut m = module_two_co_supported_gates();
        m.task_emit_gates.insert(3);
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(
            n, 0,
            "a task-emit-marked gate cannot also be a multi-output member"
        );
    }

    #[test]
    fn param_env_module_is_skipped() {
        use crate::ir::ParamEnv;
        let mut m = module_two_co_supported_gates();
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 8,
            design_value: 4,
        });
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 0, "parameterized modules are out of scope");
    }

    /// The end-to-end emit proof: a grouped pair renders ONE multi-output
    /// `task automatic` with two `output`s + a deduplicated `input` list (the
    /// shared operand `b` is one formal), an `always_comb` call into two
    /// `<member>__mtv` vars, and two passthrough `assign`s; the default (ungrouped)
    /// emission is the plain inline assigns — proving the projection is opt-in and
    /// byte-identical by default.
    #[test]
    fn grouped_pair_emits_multi_output_task_unmarked_is_inline() {
        use crate::emit::to_sv;

        // Unmarked baseline: plain inline assigns, no task.
        let base = to_sv(&module_two_co_supported_gates());
        assert!(
            !base.contains("__mt("),
            "default-off emission has no multi-output task:\n{base}"
        );

        // Marked: the pair is co-emitted as one multi-output task.
        let mut m = module_two_co_supported_gates();
        m.multi_output_task_groups.insert(3, vec![4]);
        let out = to_sv(&m);
        // The body is deterministic regardless of the gates' auto-assigned names:
        // params = [a(0), b(1), c(2)] ⇒ o0 = node3 (a & b) = a0 & a1; o1 = node4
        // (b | c) = a1 | a2 (the shared operand b is the single formal a1).
        assert!(
            out.contains("o0 = a0 & a1;"),
            "first member is its op over the shared deduped formals:\n{out}"
        );
        assert!(
            out.contains("o1 = a1 | a2;"),
            "second member reuses the shared formal a1 (operand b):\n{out}"
        );
        assert!(
            out.contains("__mt("),
            "a multi-output task is declared/called:\n{out}"
        );
        assert!(out.contains("endtask"), "the task is closed:\n{out}");
        assert!(
            out.contains("always_comb"),
            "the task is called from always_comb:\n{out}"
        );
        // Two per-member output vars + two passthrough assigns.
        assert!(
            out.contains("__mtv;"),
            "per-member output vars declared:\n{out}"
        );
        // The inline gate ops are suppressed for both members.
        assert!(
            !out.contains("= a & b;"),
            "the first member's inline assign is suppressed:\n{out}"
        );
        assert!(
            !out.contains("= b | c;"),
            "the second member's inline assign is suppressed:\n{out}"
        );
    }

    #[test]
    fn grouping_leaves_identity_and_node_count_untouched() {
        let mut m = module_two_co_supported_gates();
        let nodes_before = m.nodes.len();
        let sig_before = crate::metrics::canonical_module_signature(&m);
        annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(m.nodes.len(), nodes_before, "no new IR node");
        assert_eq!(
            crate::metrics::canonical_module_signature(&m),
            sig_before,
            "identity is unaffected by the emitter-surface mark"
        );
    }

    // ----- STRUCTURED-EMISSION-EXPANSION.13: wider (k>2) co-supported groups -----

    /// Three pairwise-independent gates connected by shared operands:
    /// `y0 = a & b`, `y1 = b | c`, `y2 = c ^ d` (`b` shared by y0,y1; `c` shared by
    /// y1,y2; none feeds another). Nodes: 0=a 1=b 2=c 3=d, 4=(a&b), 5=(b|c), 6=(c^d).
    fn module_three_co_supported_gates() -> Module {
        let mut m = Module {
            name: "mo3".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b"), (2, "c"), (3, "d")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        for (id, name) in [(4u32, "y0"), (5, "y1"), (6, "y2")] {
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
        }); // id 4
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 5
        m.nodes.push(Node::Gate {
            op: GateOp::Xor,
            operands: vec![2, 3],
            width: 4,
            deps: DepSet::new(),
        }); // id 6
        m.drives.push((4, 4));
        m.drives.push((5, 5));
        m.drives.push((6, 6));
        m
    }

    /// `n + 1` inputs (a shared `s` plus `x1..xn`) and `n` gates `s & x_i` — every
    /// gate shares the operand `s` (connected) and reads only inputs (so all are
    /// mutually fan-in-independent). Used to exercise the group-size cap.
    fn module_n_co_supported_independent_gates(n: u32) -> Module {
        let mut m = Module {
            name: "cap".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "s".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        for i in 1..=n {
            m.inputs.push(Port {
                id: i,
                name: format!("x{i}"),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: i, width: 4 });
        }
        // Gate `i` (i in 1..=n) lands at node id `n + i` = `s & x_i`.
        for i in 1..=n {
            m.nodes.push(Node::Gate {
                op: GateOp::And,
                operands: vec![0, i],
                width: 4,
                deps: DepSet::new(),
            });
        }
        m
    }

    #[test]
    fn prob_one_groups_a_co_supported_triple() {
        let mut m = module_three_co_supported_gates();
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(n, 1, "the three connected independent gates form ONE group");
        // Keyed by the lowest-NodeId leader (4) → the two higher partners (5, 6),
        // ascending. The k=2 first cut would have stopped at [5]; the widening
        // admits 6 (shares c with member 5, independent of both).
        assert_eq!(m.multi_output_task_groups.get(&4), Some(&vec![5, 6]));
    }

    #[test]
    fn group_extends_only_to_connected_co_support() {
        // y0=a&b, y1=b|c (connected via b), y2=e&f (disjoint support).
        // Nodes: 0=a 1=b 2=c 3=e 4=f, 5=(a&b), 6=(b|c), 7=(e&f).
        let mut m = Module {
            name: "conn".into(),
            ..Module::default()
        };
        for (id, name) in [(0u32, "a"), (1, "b"), (2, "c"), (3, "e"), (4, "f")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 4,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 4 });
        }
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 5 (a&b)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 6 (b|c)
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![3, 4],
            width: 4,
            deps: DepSet::new(),
        }); // id 7 (e&f) — shares no operand with 5 or 6
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(
            n, 1,
            "only the connected pair groups; the disjoint gate cannot"
        );
        assert_eq!(
            m.multi_output_task_groups.get(&5),
            Some(&vec![6]),
            "the disjoint gate 7 is NOT admitted into the group"
        );
        assert!(
            !m.multi_output_task_groups.contains_key(&7),
            "the disjoint gate forms no group of its own (no eligible partner)"
        );
    }

    #[test]
    fn group_excludes_fan_in_dependent_member_when_widening() {
        // g0=a&b, g1=b|c (independent, shares b ⇒ pairs), g2=g0|b (shares b with g0
        // AND has g0 in its fan-in ⇒ excluded by the soundness rule even though it
        // co-supports). Nodes: 0=a 1=b 2=c, 3=(a&b), 4=(b|c), 5=(g0|b)=(3|1).
        let mut m = Module {
            name: "dep3".into(),
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
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 3 (g0)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 4,
            deps: DepSet::new(),
        }); // id 4 (g1)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![3, 1],
            width: 4,
            deps: DepSet::new(),
        }); // id 5 (g2 = g0 | b)
        assert!(in_fanin(&m, 3, 5), "g0 (3) is in g2 (5)'s fan-in");
        assert!(
            shares_nonconst_operand(&m, 3, 5),
            "g2 co-supports g0 (both read b) — only independence excludes it"
        );
        let n = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(
            n, 1,
            "the independent pair groups; the dependent gate is excluded"
        );
        assert_eq!(
            m.multi_output_task_groups.get(&3),
            Some(&vec![4]),
            "the fan-in-dependent member 5 must NOT join — would close a cycle"
        );
    }

    #[test]
    fn group_respects_the_member_cap() {
        // MAX + 1 mutually-independent co-supported gates: the leader's group caps
        // at MAX members (leader + MAX-1 partners); the surplus gate is left
        // ungrouped (it has no remaining un-used partner).
        let n_gates = (MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS + 1) as u32;
        let mut m = module_n_co_supported_independent_gates(n_gates);
        let groups = annotate_multi_output_task_groups(&mut m, &mut rng(), 1.0);
        assert_eq!(
            groups, 1,
            "one capped group; the surplus gate has no partner"
        );
        // The first gate node id is `n_gates + 1` (after the shared `s` + n inputs).
        let leader = n_gates + 1;
        let partners = m
            .multi_output_task_groups
            .get(&leader)
            .expect("the leader heads the capped group");
        assert_eq!(
            partners.len() + 1,
            MAX_MULTI_OUTPUT_TASK_GROUP_MEMBERS,
            "the group is capped at MAX members (leader + MAX-1 partners)"
        );
        // The last gate (the surplus beyond the cap) is grouped nowhere.
        let surplus = n_gates + n_gates; // node id of gate #n_gates
        assert!(
            !m.multi_output_task_groups.contains_key(&surplus) && !partners.contains(&surplus),
            "the over-cap surplus gate is left ungrouped"
        );
    }

    /// The end-to-end emit proof for a 3-member group: it renders ONE `__mt(` task
    /// with three `output`s (`o0`/`o1`/`o2`), a deduplicated `input` list, an
    /// `always_comb` call into three `<member>__mtv` vars, and three passthrough
    /// `assign`s — exercising the already-k-agnostic emitter at `k = 3`.
    #[test]
    fn grouped_triple_emits_three_output_task() {
        use crate::emit::to_sv;
        let mut m = module_three_co_supported_gates();
        m.multi_output_task_groups.insert(4, vec![5, 6]);
        let out = to_sv(&m);
        assert!(
            out.contains("__mt("),
            "a multi-output task is declared:\n{out}"
        );
        assert!(out.contains("o0 ="), "first member output:\n{out}");
        assert!(out.contains("o1 ="), "second member output:\n{out}");
        assert!(
            out.contains("o2 ="),
            "THIRD member output — the k>2 widening:\n{out}"
        );
        assert!(out.contains("endtask"), "the task is closed:\n{out}");
        assert!(
            out.contains("always_comb"),
            "the task is called from always_comb:\n{out}"
        );
        // Exactly three task output formals ⇒ a genuine k=3 group (o2 present, no
        // o3). The body statements `o0..o2 = …` already drive each member.
        assert!(
            !out.contains("o3 ="),
            "exactly three task outputs — no fourth member:\n{out}"
        );
        // Three per-member output vars + their three passthrough assigns
        // (each line ends in `__mtv;`).
        assert_eq!(
            out.matches("__mtv;").count(),
            6,
            "three `__mtv` var decls + three passthrough assigns:\n{out}"
        );
    }
}
