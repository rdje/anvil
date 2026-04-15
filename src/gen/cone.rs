//! Fanin cone recursion. See `book/src/algorithm.md` for the full spec.
//!
//! Phase 1 scope: combinational only, tree-shaped (no sharing beyond
//! primary inputs), no flops. Structural anti-collapse rules apply.

use super::{pool::SignalPool, Generator};
use crate::ir::{DepSet, GateOp, Module, Node, NodeId};
use rand::Rng;

/// Retry loop around `build_cone` that rejects trivial (empty dep-set)
/// roots. Bounded to avoid pathological infinite retries; if we exceed
/// the budget, the last attempt is accepted.
pub fn build_cone_with_retry(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    const MAX_RETRIES: u32 = 4;
    for _ in 0..MAX_RETRIES {
        let snapshot = (m.nodes.len(), pool.clone());
        let node = build_cone(g, m, pool, width, 0);
        let deps = node_deps(m, node);
        if !deps.is_empty() {
            return node;
        }
        // Rewind and retry.
        m.nodes.truncate(snapshot.0);
        *pool = snapshot.1;
    }
    // Final attempt without retry budget.
    build_cone(g, m, pool, width, 0)
}

pub fn build_cone(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    depth: u32,
) -> NodeId {
    let leaf_prob = (depth as f64) / (g.cfg.max_depth as f64);
    let force_leaf = depth >= g.cfg.max_depth || g.rng.gen_bool(leaf_prob.min(1.0));

    if force_leaf {
        return pick_terminal(g, m, pool, width);
    }

    // Pick a gate category, then a specific op.
    let op = pick_gate(g, width);
    let operand_widths = input_widths_for(op, width, &mut g.rng);
    let mut operands = Vec::with_capacity(operand_widths.len());
    for w in operand_widths {
        operands.push(build_cone(g, m, pool, w, depth + 1));
    }

    // Structural anti-collapse: reject obvious patterns by regenerating.
    if violates_anti_collapse(op, &operands, m) {
        // Fall back to a terminal rather than looping.
        return pick_terminal(g, m, pool, width);
    }

    let deps = DepSet::union(
        &operands
            .iter()
            .map(|id| node_deps(m, *id))
            .collect::<Vec<_>>()
            .iter()
            .collect::<Vec<_>>(),
    );

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

fn pick_terminal(g: &mut Generator, m: &mut Module, pool: &mut SignalPool, width: u32) -> NodeId {
    // 1. Prefer matching-width pool entries with non-empty deps.
    let with_deps: Vec<_> = pool
        .of_width(width)
        .filter(|e| !e.deps.is_empty())
        .map(|e| e.node)
        .collect();
    if !with_deps.is_empty() {
        let idx = g.rng.gen_range(0..with_deps.len());
        return with_deps[idx];
    }

    // 2. Fall back to any matching-width entry (may be a constant).
    let any_match: Vec<_> = pool.of_width(width).map(|e| e.node).collect();
    if !any_match.is_empty() {
        let idx = g.rng.gen_range(0..any_match.len());
        return any_match[idx];
    }

    // 3. No matching width. Build a width-adapter from the best pool entry
    //    with non-empty deps. This preserves dep-set propagation.
    let source: Option<(NodeId, u32, DepSet)> = pool
        .iter()
        .filter(|e| !e.deps.is_empty())
        .max_by_key(|e| e.width)
        .map(|e| (e.node, e.width, e.deps.clone()));
    if let Some((src_node, src_width, src_deps)) = source {
        return make_width_adapter(m, pool, src_node, src_width, src_deps, width);
    }

    // 4. Last resort: emit a constant. The cone-root non-triviality check
    //    will reject this and the retry loop will regenerate.
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
    // src_width < target_width: replicate via Concat, then slice if needed.
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
    // Phase 1 gate menu. Weights from config.
    let bitwise: &[GateOp] = &[And, Or, Xor, Not];
    let arith: &[GateOp] = &[Add, Sub];
    let structured: &[GateOp] = &[Mux];
    // Comparisons only legal when target_width == 1
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
        return And; // degenerate but legal
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
            // Output is 1. Pick an internal operand width.
            let w = rng.gen_range(1..=8);
            vec![w, w]
        }
        RedAnd | RedOr | RedXor => {
            let w = rng.gen_range(2..=8);
            vec![w]
        }
        Shl | Shr => vec![out_w, 8],
        Slice { .. } => vec![out_w.saturating_add(1)],
        Concat => vec![out_w], // placeholder; real concat is variadic
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
