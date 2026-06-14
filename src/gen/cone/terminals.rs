//! Terminal & signal-pool selection + gate-shape policy
//! (`CONE-DECOMPOSITION.5`).
//!
//! How a recursion point picks an *existing* signal (or a width-adapted
//! one) instead of growing the graph: `pick_terminal` and its dep-bearing
//! variant, the dup-capped data pickers, the `make_width_adapter`
//! width-coercion helper, `try_share` for DAG reuse, the `pick_gate` /
//! structured-gate / operand-width policy, the anti-collapse guards, and
//! the `node_deps` accessor. `pick_terminal_dep_bearing`,
//! `make_width_adapter`, and `node_deps` are used by `src/gen/module.rs`
//! and `src/gen/hierarchy.rs` via the cone-root re-export, so they are
//! `pub(crate)`. Extracted verbatim from `cone.rs`; behaviour is unchanged.

use super::{ceil_log2, roll_knob};
use crate::config::Config;
use crate::gen::pool::SignalPool;
use crate::gen::Generator;
use crate::ir::{DepSet, GateOp, KnobId, Module, Node, NodeId};
use rand::Rng;
use tracing::{instrument, trace, warn};

#[instrument(level = "trace", skip(g, m, pool))]
pub(crate) fn pick_terminal(
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
    let any_match: Vec<_> = pool
        .of_width(width)
        .filter(not_excluded)
        .map(|e| e.node)
        .collect();

    if !with_deps.is_empty() || !any_match.is_empty() {
        if roll_knob(g, m, KnobId::TerminalReuseProb, g.cfg.terminal_reuse_prob) {
            let (tier, candidates) = if !with_deps.is_empty() {
                (1, &with_deps)
            } else {
                (2, &any_match)
            };
            let idx = g.rng.gen_range(0..candidates.len());
            let node = candidates[idx];
            if tier == 1 {
                trace!(tier, node, "pick_terminal: dep-bearing pool entry");
            } else {
                trace!(tier, node, "pick_terminal: any matching-width pool entry");
            }
            return node;
        }
        trace!(
            tier = 2,
            width,
            "pick_terminal: terminal_reuse_prob missed; emit fresh constant"
        );
        return emit_terminal_constant(g, m, pool, width);
    }

    let source: Option<(NodeId, u32, DepSet)> = pool
        .iter()
        .filter(not_excluded)
        .filter(|e| !e.deps.is_empty())
        .max_by_key(|e| e.width)
        .map(|e| (e.node, e.width, e.deps.clone()));
    if let Some((src_node, src_width, src_deps)) = source {
        if roll_knob(g, m, KnobId::ConstantProb, g.cfg.constant_prob) {
            trace!(
                tier = 3,
                width,
                "pick_terminal: constant_prob fired; emit fresh constant"
            );
            return emit_terminal_constant(g, m, pool, width);
        }
        trace!(
            tier = 3,
            src_node,
            src_width,
            target_width = width,
            "pick_terminal: width-adapter fallback"
        );
        return make_width_adapter(m, pool, src_node, src_width, src_deps, width);
    }

    warn!(
        tier = 4,
        width, "⚠️ pick_terminal: fresh-constant fallback (no reusable source)"
    );
    emit_terminal_constant(g, m, pool, width)
}

pub(crate) fn emit_terminal_constant(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    let value = if width >= 128 {
        0
    } else {
        g.rng.gen::<u128>() & ((1u128 << width) - 1)
    };
    let (node_id, is_new) = m.intern_constant(width, value);
    if is_new {
        pool.add(node_id, width, DepSet::new());
    }
    node_id
}

/// Pick `count` data signals of the given `width` for the arms of an
/// N-to-1 mux, honoring `m.mux_arm_duplication_rate`. At each pick,
/// if the candidate would duplicate a signal already picked for this
/// mux, it is kept with probability `mux_arm_duplication_rate` and
/// rejected (re-pick) otherwise. Rate 0.0 → every arm distinct;
/// rate 1.0 → no constraint. Bounded retry budget (8 tries) — after
/// exhaustion the candidate is accepted, to avoid pathological
/// re-pick loops when the pool is too small to satisfy uniqueness.
pub(crate) fn pick_datas_with_dup_cap(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    count: usize,
    exclude: Option<NodeId>,
) -> Vec<NodeId> {
    use std::collections::HashSet;
    let rate = m.mux_arm_duplication_rate.clamp(0.0, 1.0);
    let mut picked: HashSet<NodeId> = HashSet::new();
    let mut arms: Vec<NodeId> = Vec::with_capacity(count);
    for _ in 0..count {
        let mut candidate = pick_terminal(g, m, pool, width, exclude);
        let mut tries = 0u32;
        while picked.contains(&candidate) && !g.rng.gen_bool(rate) && tries < 8 {
            candidate = pick_terminal(g, m, pool, width, exclude);
            tries += 1;
        }
        picked.insert(candidate);
        arms.push(candidate);
    }
    arms
}

/// Pick `count` operator-gate operand signals honouring
/// `m.operand_duplication_rate`. Mirrors `pick_datas_with_dup_cap`
/// but reads the operand-duplication knob instead of the mux-arm
/// knob. At rate 0.0 (default) retries duplicates up to 8 tries;
/// at rate 1.0 accepts duplicates freely.
pub(crate) fn pick_signals_with_dup_rate(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    count: usize,
    exclude: Option<NodeId>,
) -> Vec<NodeId> {
    use std::collections::HashSet;
    let rate = m.operand_duplication_rate.clamp(0.0, 1.0);
    let mut picked: HashSet<NodeId> = HashSet::new();
    let mut arms: Vec<NodeId> = Vec::with_capacity(count);
    for _ in 0..count {
        let mut candidate = pick_terminal(g, m, pool, width, exclude);
        let mut tries = 0u32;
        while picked.contains(&candidate) && !g.rng.gen_bool(rate) && tries < 8 {
            candidate = pick_terminal(g, m, pool, width, exclude);
            tries += 1;
        }
        picked.insert(candidate);
        arms.push(candidate);
    }
    arms
}

/// Strict variant of `pick_terminal`: guaranteed to return a
/// dep-bearing node (transitively driven by a primary input or flop Q).
/// Use at positions where a constant source would make the surrounding
/// logic fold at elaboration time — mux selects, priority-encoder
/// request bits, LHS of the constant-comparand comparison, value
/// operand of the constant-shift-amount motif. See Rule 20 in
/// `book/src/structural-rules.md`.
///
/// Tiers (subset of `pick_terminal`):
/// 1. Random dep-bearing matching-width pool entry.
/// 2. Width-adapter from the widest dep-bearing pool entry of any
///    width.
///
/// Panics if the pool contains no dep-bearing entry at all. Since
/// primary inputs are always added to the pool with non-empty deps,
/// this is unreachable in normal generator flow.
#[instrument(level = "trace", skip(g, m, pool))]
pub(crate) fn pick_terminal_dep_bearing(
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

    let source: Option<(NodeId, u32, DepSet)> = pool
        .iter()
        .filter(not_excluded)
        .filter(|e| !e.deps.is_empty())
        .max_by_key(|e| e.width)
        .map(|e| (e.node, e.width, e.deps.clone()));
    if let Some((src_node, src_width, src_deps)) = source {
        return make_width_adapter(m, pool, src_node, src_width, src_deps, width);
    }

    panic!(
        "pick_terminal_dep_bearing: pool has no dep-bearing entry; \
         generator invariant violated (primary inputs should always \
         be present in the pool with non-empty deps)"
    );
}

/// Build a Slice or replicating Concat (+ Slice) that adapts a source
/// signal of width `src_width` to `target_width`, propagating deps.
pub(crate) fn make_width_adapter(
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
        let (node_id, is_new) = m.intern_gate(
            GateOp::Slice {
                hi: target_width - 1,
                lo: 0,
            },
            vec![src_node],
            target_width,
            src_deps.clone(),
        );
        if is_new {
            pool.add(node_id, target_width, src_deps);
        }
        return node_id;
    }

    // Expand to the exact target width instead of materialising a
    // wider replicated Concat and slicing it back down. The old
    // shape was semantically fine, but it left dead high bits in the
    // intermediate Concat (`{src, src, ...}[target-1:0]`), which
    // downstream linters quite reasonably flagged as unused.
    let full_copies = target_width / src_width;
    let remainder = target_width % src_width;
    let mut operands: Vec<NodeId> =
        Vec::with_capacity(full_copies as usize + usize::from(remainder > 0));
    if remainder > 0 {
        let (slice_id, slice_is_new) = m.intern_gate(
            GateOp::Slice {
                hi: remainder - 1,
                lo: 0,
            },
            vec![src_node],
            remainder,
            src_deps.clone(),
        );
        if slice_is_new {
            pool.add(slice_id, remainder, src_deps.clone());
        }
        operands.push(slice_id);
    }
    operands.extend(vec![src_node; full_copies as usize]);
    let (concat_id, concat_is_new) =
        m.intern_gate(GateOp::Concat, operands, target_width, src_deps.clone());
    if concat_is_new {
        pool.add(concat_id, target_width, src_deps);
    }
    concat_id
}

pub(crate) fn pick_gate(g: &mut Generator, target_width: u32) -> GateOp {
    use GateOp::*;
    #[derive(Clone, Copy)]
    enum GateBucket {
        Bitwise,
        Arith,
        Structured,
        Compare,
        Reduce,
        Shift,
    }

    let bitwise: &[GateOp] = &[And, Or, Xor, Not];
    let arith: &[GateOp] = &[Add, Sub, Mul];

    let w = &g.cfg;
    let buckets: [(u32, GateBucket); 6] = [
        (w.gate_bitwise_weight, GateBucket::Bitwise),
        (w.gate_arith_weight, GateBucket::Arith),
        (w.gate_struct_weight, GateBucket::Structured),
        (w.gate_compare_weight, GateBucket::Compare),
        (w.gate_reduce_weight, GateBucket::Reduce),
        (w.gate_shift_weight, GateBucket::Shift),
    ];
    let bucket_live = |bucket: GateBucket| match bucket {
        GateBucket::Bitwise | GateBucket::Arith | GateBucket::Structured => true,
        GateBucket::Compare | GateBucket::Reduce => target_width == 1,
        GateBucket::Shift => target_width > 1,
    };
    let total: u32 = buckets
        .iter()
        .filter(|(_, bucket)| bucket_live(*bucket))
        .map(|(wt, _)| *wt)
        .sum();
    if total == 0 {
        return And;
    }
    let mut pick = g.rng.gen_range(0..total);
    for (wt, bucket) in buckets.iter() {
        if !bucket_live(*bucket) {
            continue;
        }
        if pick < *wt {
            return match bucket {
                GateBucket::Bitwise => bitwise[g.rng.gen_range(0..bitwise.len())],
                GateBucket::Arith => arith[g.rng.gen_range(0..arith.len())],
                GateBucket::Structured => pick_structured_gate(g, target_width),
                GateBucket::Compare => match g.rng.gen_range(0..6) {
                    0 => Eq,
                    1 => Neq,
                    2 => Lt,
                    3 => Gt,
                    4 => Le,
                    _ => Ge,
                },
                GateBucket::Reduce => match g.rng.gen_range(0..3) {
                    0 => RedAnd,
                    1 => RedOr,
                    _ => RedXor,
                },
                GateBucket::Shift => {
                    if g.rng.gen_bool(0.5) {
                        Shl
                    } else {
                        Shr
                    }
                }
            };
        }
        pick -= *wt;
    }
    And
}

pub(crate) fn pick_structured_gate(g: &mut Generator, target_width: u32) -> GateOp {
    // Keep the selectable Slice/Concat surfaces explicitly
    // non-degenerate: selectable Slice must not be a full-width
    // identity, and selectable Concat must have >= 2 operands.
    if target_width >= 2 {
        match g.rng.gen_range(0..3) {
            0 => GateOp::Mux,
            1 => pick_slice_gate(g, target_width),
            _ => GateOp::Concat,
        }
    } else {
        match g.rng.gen_range(0..2) {
            0 => GateOp::Mux,
            _ => pick_slice_gate(g, target_width),
        }
    }
}

pub(crate) fn pick_slice_gate(g: &mut Generator, target_width: u32) -> GateOp {
    debug_assert!(target_width >= 1);
    let lo: u32 = g.rng.gen_range(0..=3);
    let hi = lo
        .checked_add(target_width - 1)
        .expect("slice hi must fit in u32");
    GateOp::Slice { hi, lo }
}

pub(crate) fn pick_concat_operand_widths(out_w: u32, cfg: &Config, rng: &mut impl Rng) -> Vec<u32> {
    debug_assert!(out_w >= 2);
    let max_parts = cfg.max_gate_arity.max(2).min(out_w);
    let n_parts = rng.gen_range(2..=max_parts);
    let mut remaining = out_w;
    let mut widths = Vec::with_capacity(n_parts as usize);
    for idx in 0..(n_parts - 1) {
        let min_remaining_after = n_parts - idx - 1;
        let max_this = remaining - min_remaining_after;
        let w = rng.gen_range(1..=max_this);
        widths.push(w);
        remaining -= w;
    }
    widths.push(remaining);
    widths
}

pub(crate) fn input_widths_for(
    op: GateOp,
    out_w: u32,
    cfg: &Config,
    rng: &mut impl Rng,
) -> Vec<u32> {
    use GateOp::*;
    match op {
        // N-arity associative operators: And/Or/Xor/Add/Mul. N picked from
        // [min_gate_arity, max_gate_arity]; all operands width == out_w.
        // N = 2 recovers the classic binary gate.
        //
        // Sub is NOT associative — (a - b) - c ≠ a - (b - c). It stays
        // strictly 2-arity. Chains like `a - b - c` can still arise in
        // emitted output because multiple 2-arity Sub gates cascade, but
        // the IR represents each subtraction as its own binary node.
        And | Or | Xor | Add | Mul => {
            let n = rng.gen_range(cfg.min_gate_arity..=cfg.max_gate_arity);
            vec![out_w; n as usize]
        }
        Sub => vec![out_w, out_w],
        Not => vec![out_w],
        Mux => vec![1, out_w, out_w],
        CaseMux => {
            let min_arms = cfg.min_mux_arms.max(2);
            let max_arms = cfg.max_mux_arms.max(min_arms);
            let n = rng.gen_range(min_arms..=max_arms);
            let mut widths = Vec::with_capacity(n as usize + 1);
            widths.push(ceil_log2(n));
            widths.extend(std::iter::repeat_n(out_w, n as usize));
            widths
        }
        CasezMux => {
            let min_arms = cfg.min_mux_arms.max(2);
            let max_arms = cfg.max_mux_arms.max(min_arms);
            let n = rng.gen_range(min_arms..=max_arms);
            let sel_w = ceil_log2(n) + 1;
            let mut widths = Vec::with_capacity(1 + n as usize * 3);
            widths.push(sel_w);
            for _ in 0..n {
                widths.push(sel_w);
                widths.push(sel_w);
                widths.push(out_w);
            }
            widths
        }
        ForFold {
            trip_count,
            chunk_width,
            ..
        } => vec![trip_count.saturating_mul(chunk_width)],
        Eq | Neq | Lt | Gt | Le | Ge => {
            let w = rng.gen_range(1..=8);
            vec![w, w]
        }
        RedAnd | RedOr | RedXor => {
            let w = rng.gen_range(2..=8);
            vec![w]
        }
        Shl | Shr => vec![out_w, 8],
        Slice { hi, .. } => vec![hi.saturating_add(2)],
        Concat => {
            if out_w >= 2 {
                pick_concat_operand_widths(out_w, cfg, rng)
            } else {
                vec![out_w]
            }
        }
    }
}

pub(crate) fn violates_anti_collapse(op: GateOp, operands: &[NodeId], m: &Module) -> bool {
    use crate::config::FactorizationLevel;
    use GateOp::*;
    // Operand-uniqueness checks (And/Or/Xor and conditionally
    // Add/Mul) are gated on `factorization_level >= OperandUnique`.
    // At level `cse` / `none` we do NOT reject operand duplicates —
    // the user has opted out of that layer. The 2-operand
    // algebraic-degeneracy cases (Sub / Eq / Neq) are base Rule 8
    // and fire regardless of the level (they'd break correctness
    // otherwise). Mux is gated on `mux_arm_duplication_rate` as
    // before.
    let operand_unique_enabled =
        m.effective_factorization_level() >= FactorizationLevel::OperandUnique;
    match op {
        And | Or | Xor if operand_unique_enabled => has_duplicate_operand(operands),
        Add | Mul if operand_unique_enabled && m.operand_duplication_rate < 1.0 => {
            has_duplicate_operand(operands)
        }
        Sub if operands.len() == 2 => operands[0] == operands[1],
        Eq | Neq if operands.len() == 2 => operands[0] == operands[1],
        Mux if operands.len() == 3 && m.mux_arm_duplication_rate < 1.0 => {
            operands[1] == operands[2]
        }
        _ => false,
    }
}

/// True iff any `NodeId` appears more than once in `operands`.
/// O(N²) in operand count — acceptable because N is bounded by
/// `cfg.max_gate_arity` (typically ≤ 8).
pub(crate) fn has_duplicate_operand(operands: &[NodeId]) -> bool {
    for i in 0..operands.len() {
        for j in (i + 1)..operands.len() {
            if operands[i] == operands[j] {
                return true;
            }
        }
    }
    false
}

/// DAG-sharing operand picker. Returns an existing pool entry of the
/// requested width with non-empty deps, honoring the `exclude` filter.
/// None if no shareable candidate exists — the caller falls back to
/// normal recursion.
pub(crate) fn try_share(
    g: &mut Generator,
    pool: &SignalPool,
    width: u32,
    exclude: Option<NodeId>,
) -> Option<NodeId> {
    let candidates: Vec<NodeId> = pool
        .of_width(width)
        .filter(|e| Some(e.node) != exclude)
        .filter(|e| !e.deps.is_empty())
        .map(|e| e.node)
        .collect();
    if candidates.is_empty() {
        None
    } else {
        let idx = g.rng.gen_range(0..candidates.len());
        Some(candidates[idx])
    }
}

pub(crate) fn node_deps(m: &Module, id: NodeId) -> DepSet {
    match &m.nodes[id as usize] {
        Node::PrimaryInput { port, .. } => DepSet::from_port(*port),
        Node::Constant { .. } => DepSet::new(),
        Node::FlopQ { flop, .. } => DepSet::from_flop_virtual(*flop),
        Node::MemRead { mem, .. } => DepSet::from_mem_virtual(*mem),
        Node::FsmOut { fsm, .. } => DepSet::from_fsm_virtual(*fsm),
        Node::InstanceOutput { instance, port, .. } => {
            DepSet::from_instance_output_virtual(*instance, *port)
        }
        Node::Gate { deps, .. } => deps.clone(),
    }
}
