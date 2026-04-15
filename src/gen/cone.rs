//! Fanin cone recursion. See `book/src/algorithm.md` for the full spec.
//!
//! Combinational + sequential. Recursion is the core principle:
//! - Q is a leaf in the *current* cone (terminates the descent).
//! - D opens a *new* cone, queued on the worklist for later draining.
//! - The same `build_cone` function constructs both.

use super::{pool::SignalPool, Generator};
use crate::ir::{DepSet, Flop, FlopId, FlopKind, GateOp, Module, MuxArm, Node, NodeId, ResetKind};
use rand::Rng;

/// Worklist of flops whose D-input cone has not yet been built.
pub type FlopWorklist = Vec<FlopId>;

/// Retry loop around `build_cone` that rejects trivial (empty dep-set)
/// roots. Bounded to avoid pathological infinite retries; if we exceed
/// the budget, the last attempt is accepted.
///
/// `exclude` lets callers forbid a specific `NodeId` from terminal
/// selection. Used for flop D-cone construction to forbid the flop's
/// own Q from appearing in its data or select sub-cones (the only
/// permitted Q→D path is the all-zeros-select feedback term in
/// `FlopKind::QFeedback`).
pub fn build_cone_with_retry(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    const MAX_RETRIES: u32 = 4;
    for _ in 0..MAX_RETRIES {
        let snapshot = (m.nodes.len(), m.flops.len(), pool.clone(), worklist.clone());
        let node = build_cone(g, m, pool, worklist, width, 0, exclude);
        let deps = node_deps(m, node);
        if !deps.is_empty() {
            return node;
        }
        m.nodes.truncate(snapshot.0);
        m.flops.truncate(snapshot.1);
        *pool = snapshot.2;
        *worklist = snapshot.3;
    }
    build_cone(g, m, pool, worklist, width, 0, exclude)
}

/// Drain the flop worklist.
///
/// For each pending flop:
/// - Pick M (number of mux arms) from {0, 2, 3, ..., max_mux_arms}.
/// - If M == 0: D is driven directly by a recursive cone (no mux).
/// - If M >= 2: build M data sub-cones (width N) + M select sub-cones
///   (1-bit), every one a recursion point. Assemble D as a one-hot mux:
///   `D = OR_i({N{sel_i}} & data_i)`, plus an optional Q-feedback term
///   for `FlopKind::QFeedback`.
///
/// All sub-cones (data, select, or the M==0 direct D-cone) forbid this
/// flop's own Q from being a leaf — the *only* permitted Q→D path is
/// the all-zeros-select feedback term in `FlopKind::QFeedback`.
///
/// The drain may itself enqueue more flops; the loop handles that
/// until quiescence.
pub fn drain_flop_worklist(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) {
    while let Some(flop_id) = worklist.pop() {
        let width = m.flops[flop_id as usize].width;
        let kind = m.flops[flop_id as usize].kind;
        let q_node = m.flops[flop_id as usize].q;
        let exclude = Some(q_node);

        let m_arms = pick_mux_arm_count(g);
        if m_arms == 0 {
            // No mux: D is a direct recursive cone of width N.
            let d_node = build_cone_with_retry(g, m, pool, worklist, width, exclude);
            // If this flop is QFeedback with M==0, the contract degenerates to
            // "D is whatever the cone produces" — there is no all-zeros-select
            // term to fall back to. Both kinds collapse to the same shape here.
            m.flops[flop_id as usize].d = Some(d_node);
            m.flops[flop_id as usize].arms = Vec::new();
            continue;
        }

        let mut arms: Vec<MuxArm> = Vec::with_capacity(m_arms as usize);
        for _ in 0..m_arms {
            let data = build_cone_with_retry(g, m, pool, worklist, width, exclude);
            let sel = build_cone_with_retry(g, m, pool, worklist, 1, exclude);
            arms.push(MuxArm { data, sel });
        }

        let d_node = assemble_flop_d(m, pool, width, &arms, kind, q_node);
        m.flops[flop_id as usize].arms = arms;
        m.flops[flop_id as usize].d = Some(d_node);
    }
}

/// Pick M from {0, 2, 3, ..., max_mux_arms}. M = 1 is excluded by
/// design — a 1-arm mux is just a wire.
fn pick_mux_arm_count(g: &mut Generator) -> u32 {
    let max = g.cfg.max_mux_arms;
    // Build the legal set: 0, then 2..=max (skipping 1).
    // min_mux_arms still controls the *minimum non-zero* M.
    let min = g.cfg.min_mux_arms.max(2);
    let max = max.max(min);
    // Coin flip: M == 0 with probability 1/(max - min + 2) baseline, plus
    // a uniform pick among 2..=max. Simplest: 1-in-(max-min+2) for M=0,
    // otherwise uniform in [min, max].
    let n_choices = max - min + 2;
    let pick = g.rng.gen_range(0..n_choices);
    if pick == 0 {
        0
    } else {
        min + (pick - 1)
    }
}

/// Build the gate tree for D from M one-hot mux arms.
/// `D = OR_i ({N{sel_i}} & data_i)` (Kind1)
/// `D = OR_i ({N{sel_i}} & data_i) | ({N{none_selected}} & Q)` (Kind2)
fn assemble_flop_d(
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    arms: &[MuxArm],
    kind: FlopKind,
    q_node: NodeId,
) -> NodeId {
    let mut term_nodes: Vec<NodeId> = Vec::with_capacity(arms.len() + 1);
    for arm in arms {
        let mask = replicate_to_width(m, pool, arm.sel, width);
        let term = make_and(m, pool, mask, arm.data, width);
        term_nodes.push(term);
    }
    if matches!(kind, FlopKind::QFeedback) {
        let none_sel = make_none_selected(m, pool, arms);
        let mask = replicate_to_width(m, pool, none_sel, width);
        let term = make_and(m, pool, mask, q_node, width);
        term_nodes.push(term);
    }
    or_reduce_terms(m, pool, &term_nodes, width)
}

/// `{N{bit}}` — replicate a 1-bit signal to N bits via Concat. If N == 1,
/// returns the bit unchanged.
fn replicate_to_width(m: &mut Module, pool: &mut SignalPool, bit: NodeId, width: u32) -> NodeId {
    if width == 1 {
        return bit;
    }
    let bit_deps = node_deps(m, bit);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Concat,
        operands: vec![bit; width as usize],
        width,
        deps: bit_deps.clone(),
    });
    pool.add(node_id, width, bit_deps);
    node_id
}

fn make_and(m: &mut Module, pool: &mut SignalPool, a: NodeId, b: NodeId, width: u32) -> NodeId {
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::And,
        operands: vec![a, b],
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

/// `none_selected = ~(sel_0 | sel_1 | ... | sel_{M-1})`.
/// 1-bit output, 1 when no select is asserted.
fn make_none_selected(m: &mut Module, pool: &mut SignalPool, arms: &[MuxArm]) -> NodeId {
    debug_assert!(!arms.is_empty());
    let mut acc = arms[0].sel;
    for arm in &arms[1..] {
        let deps = DepSet::union(&[&node_deps(m, acc), &node_deps(m, arm.sel)]);
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![acc, arm.sel],
            width: 1,
            deps: deps.clone(),
        });
        pool.add(node_id, 1, deps);
        acc = node_id;
    }
    let acc_deps = node_deps(m, acc);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Not,
        operands: vec![acc],
        width: 1,
        deps: acc_deps.clone(),
    });
    pool.add(node_id, 1, acc_deps);
    node_id
}

fn or_reduce_terms(m: &mut Module, pool: &mut SignalPool, terms: &[NodeId], width: u32) -> NodeId {
    debug_assert!(!terms.is_empty());
    let mut acc = terms[0];
    for &t in &terms[1..] {
        let deps = DepSet::union(&[&node_deps(m, acc), &node_deps(m, t)]);
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![acc, t],
            width,
            deps: deps.clone(),
        });
        pool.add(node_id, width, deps);
        acc = node_id;
    }
    acc
}

pub fn build_cone(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    let leaf_prob = (depth as f64) / (g.cfg.max_depth as f64);
    let force_leaf = depth >= g.cfg.max_depth || g.rng.gen_bool(leaf_prob.min(1.0));

    if force_leaf {
        return pick_terminal(g, m, pool, width, exclude);
    }

    // Recursion fork: gate vs flop. Flop is allowed up to a per-module cap.
    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    let pick_flop = flop_allowed && g.rng.gen_bool(g.cfg.flop_prob.min(1.0));
    if pick_flop {
        return build_flop_leaf(g, m, pool, worklist, width);
    }

    let op = pick_gate(g, width);
    let operand_widths = input_widths_for(op, width, &mut g.rng);
    let mut operands = Vec::with_capacity(operand_widths.len());
    for w in operand_widths {
        operands.push(build_cone(g, m, pool, worklist, w, depth + 1, exclude));
    }

    if violates_anti_collapse(op, &operands, m) {
        return pick_terminal(g, m, pool, width, exclude);
    }

    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());

    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op,
        operands,
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

/// Allocate a flop and a `FlopQ` node. The Q is returned (and added to
/// the pool) as the leaf for the current cone. The flop's D-cone is
/// queued for later construction by `drain_flop_worklist`.
fn build_flop_leaf(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
) -> NodeId {
    let flop_id = m.flops.len() as FlopId;
    let q_node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::FlopQ {
        flop: flop_id,
        width,
    });
    let reset_val = pick_reset_value(g, width);
    let kind = if g.rng.gen_bool(g.cfg.flop_qfeedback_prob.min(1.0)) {
        FlopKind::QFeedback
    } else {
        FlopKind::ZeroDefault
    };
    m.flops.push(Flop {
        id: flop_id,
        width,
        d: None,
        q: q_node_id,
        reset_val,
        // Fully synchronous design discipline: every flop uses the
        // module's single CLK (posedge) and single RST_N (async, active-low).
        reset_kind: ResetKind::Async,
        kind,
        arms: Vec::new(),
    });
    let virtual_deps = DepSet::from_flop_virtual(flop_id);
    pool.add(q_node_id, width, virtual_deps);
    worklist.push(flop_id);
    q_node_id
}

fn pick_reset_value(g: &mut Generator, width: u32) -> u128 {
    // Bias toward zero (most common in real designs).
    let r = g.rng.gen_range(0..4);
    if r < 2 || width >= 128 {
        0
    } else if r == 2 {
        (1u128 << width) - 1 // all ones
    } else {
        g.rng.gen::<u128>() & ((1u128 << width) - 1)
    }
}

fn pick_terminal(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    let not_excluded = |e: &&crate::gen::pool::PoolEntry| Some(e.node) != exclude;

    let with_deps: Vec<_> = pool
        .of_width(width)
        .filter(not_excluded)
        .filter(|e| !e.deps.is_empty())
        .map(|e| e.node)
        .collect();
    if !with_deps.is_empty() {
        let idx = g.rng.gen_range(0..with_deps.len());
        return with_deps[idx];
    }

    let any_match: Vec<_> = pool
        .of_width(width)
        .filter(not_excluded)
        .map(|e| e.node)
        .collect();
    if !any_match.is_empty() {
        let idx = g.rng.gen_range(0..any_match.len());
        return any_match[idx];
    }

    let source: Option<(NodeId, u32, DepSet)> = pool
        .iter()
        .filter(not_excluded)
        .filter(|e| !e.deps.is_empty())
        .max_by_key(|e| e.width)
        .map(|e| (e.node, e.width, e.deps.clone()));
    if let Some((src_node, src_width, src_deps)) = source {
        return make_width_adapter(m, pool, src_node, src_width, src_deps, width);
    }

    let value = if width >= 128 {
        0
    } else {
        g.rng.gen::<u128>() & ((1u128 << width) - 1)
    };
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Constant { width, value });
    pool.add(node_id, width, DepSet::new());
    node_id
}

/// Build a Slice or replicating Concat (+ Slice) that adapts a source
/// signal of width `src_width` to `target_width`, propagating deps.
fn make_width_adapter(
    m: &mut Module,
    pool: &mut SignalPool,
    src_node: NodeId,
    src_width: u32,
    src_deps: DepSet,
    target_width: u32,
) -> NodeId {
    if src_width == target_width {
        return src_node;
    }
    if src_width > target_width {
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op: GateOp::Slice {
                hi: target_width - 1,
                lo: 0,
            },
            operands: vec![src_node],
            width: target_width,
            deps: src_deps.clone(),
        });
        pool.add(node_id, target_width, src_deps);
        return node_id;
    }
    let copies = target_width.div_ceil(src_width);
    let concat_width = copies * src_width;
    let concat_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Concat,
        operands: vec![src_node; copies as usize],
        width: concat_width,
        deps: src_deps.clone(),
    });
    pool.add(concat_id, concat_width, src_deps.clone());
    if concat_width == target_width {
        return concat_id;
    }
    let slice_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Slice {
            hi: target_width - 1,
            lo: 0,
        },
        operands: vec![concat_id],
        width: target_width,
        deps: src_deps.clone(),
    });
    pool.add(slice_id, target_width, src_deps);
    slice_id
}

fn pick_gate(g: &mut Generator, target_width: u32) -> GateOp {
    use GateOp::*;
    let bitwise: &[GateOp] = &[And, Or, Xor, Not];
    let arith: &[GateOp] = &[Add, Sub];
    let structured: &[GateOp] = &[Mux];
    let compare: &[GateOp] = if target_width == 1 {
        &[Eq, Neq, Lt]
    } else {
        &[]
    };

    let w = &g.cfg;
    let buckets: [(u32, &[GateOp]); 4] = [
        (w.gate_bitwise_weight, bitwise),
        (w.gate_arith_weight, arith),
        (w.gate_struct_weight, structured),
        (w.gate_compare_weight, compare),
    ];
    let total: u32 = buckets
        .iter()
        .filter(|(_, gs)| !gs.is_empty())
        .map(|(wt, _)| *wt)
        .sum();
    if total == 0 {
        return And;
    }
    let mut pick = g.rng.gen_range(0..total);
    for (wt, gs) in buckets.iter() {
        if gs.is_empty() {
            continue;
        }
        if pick < *wt {
            return gs[g.rng.gen_range(0..gs.len())];
        }
        pick -= *wt;
    }
    And
}

fn input_widths_for(op: GateOp, out_w: u32, rng: &mut impl Rng) -> Vec<u32> {
    use GateOp::*;
    match op {
        And | Or | Xor | Add | Sub | Mul => vec![out_w, out_w],
        Not => vec![out_w],
        Mux => vec![1, out_w, out_w],
        Eq | Neq | Lt | Gt | Le | Ge => {
            let w = rng.gen_range(1..=8);
            vec![w, w]
        }
        RedAnd | RedOr | RedXor => {
            let w = rng.gen_range(2..=8);
            vec![w]
        }
        Shl | Shr => vec![out_w, 8],
        Slice { .. } => vec![out_w.saturating_add(1)],
        Concat => vec![out_w],
    }
}

fn violates_anti_collapse(op: GateOp, operands: &[NodeId], _m: &Module) -> bool {
    use GateOp::*;
    match op {
        Xor | Sub | Eq | Neq if operands.len() == 2 => operands[0] == operands[1],
        Mux if operands.len() == 3 => operands[1] == operands[2],
        _ => false,
    }
}

fn node_deps(m: &Module, id: NodeId) -> DepSet {
    match &m.nodes[id as usize] {
        Node::PrimaryInput { port, .. } => DepSet::from_port(*port),
        Node::Constant { .. } => DepSet::new(),
        Node::FlopQ { flop, .. } => DepSet::from_flop_virtual(*flop),
        Node::Gate { deps, .. } => deps.clone(),
    }
}
