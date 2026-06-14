//! Structured motif & block builders (`CONE-DECOMPOSITION.7`).
//!
//! The "vocabulary" the recursion draws from: comb-mux / case / casez /
//! for-fold blocks (both the recursive form, called by
//! `process_signal_frame`, and the pool-only form, called by
//! `grow_pool_one_unit`), the priority encoder, the linear-combination /
//! coefficient compound, the constant-shift and constant-comparand
//! motifs, and the small shared mux helpers `make_none_selected` /
//! `or_reduce_terms` / `is_comparison_op`. These mutually recurse with the
//! strategy core's `build_cone` (which stays in the cone root) through the
//! cone-root re-exports. Extracted verbatim from `cone.rs`; behaviour is
//! unchanged.

use super::{
    build_comparison_gate, build_cone, ceil_log2, make_and, make_constant, make_eq_const, make_mul,
    make_mux, make_nary_add, make_nary_mul, make_sub, node_deps, pick_datas_with_dup_cap,
    pick_signals_with_dup_rate, pick_terminal, pick_terminal_dep_bearing, replicate_to_width,
    roll_knob, width_mask, FlopWorklist,
};
use crate::gen::pool::SignalPool;
use crate::gen::Generator;
use crate::ir::{DepSet, ForFoldKind, GateOp, KnobId, Module, MuxArm, NodeId};
use rand::Rng;
use tracing::{instrument, warn};

/// Pool-only comb-mux assembly (mirrors `build_comb_mux` but
/// sub-cones are pool picks instead of recursive builds).
pub(crate) fn build_comb_mux_pool_only(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);

    let encoded = roll_knob(
        g,
        m,
        KnobId::CombMuxEncodingProb,
        g.cfg.comb_mux_encoding_prob,
    );
    if encoded {
        m.comb_mux_encoded_built += 1;
        let sel_width = ceil_log2(n_arms);
        let sel = pick_terminal_dep_bearing(g, m, pool, sel_width, None);
        let datas: Vec<NodeId> = pick_datas_with_dup_cap(g, m, pool, width, n_arms as usize, None);
        let fall_through = make_constant(m, pool, width, 0);
        let mut tail = fall_through;
        for idx_rev in 0..n_arms {
            let idx = n_arms - 1 - idx_rev;
            let eq = make_eq_const(m, pool, sel, sel_width, idx as u128);
            tail = make_mux(m, pool, eq, datas[idx as usize], tail, width);
        }
        tail
    } else {
        m.comb_mux_one_hot_built += 1;
        let datas = pick_datas_with_dup_cap(g, m, pool, width, n_arms as usize, None);
        let mut arms: Vec<MuxArm> = Vec::with_capacity(n_arms as usize);
        for data in datas {
            let sel = pick_terminal_dep_bearing(g, m, pool, 1, None);
            arms.push(MuxArm { data, sel });
        }
        let mut term_nodes: Vec<NodeId> = Vec::with_capacity(arms.len());
        for arm in &arms {
            let mask = replicate_to_width(m, pool, arm.sel, width);
            term_nodes.push(make_and(m, pool, mask, arm.data, width));
        }
        or_reduce_terms(m, pool, &term_nodes, width)
    }
}

/// Pool-only procedural case-mux assembly. Semantics match the
/// encoded comb-mux shape, but emission uses `always_comb case`.
pub(crate) fn build_case_mux_pool_only(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);
    let sel_width = ceil_log2(n_arms);
    let sel = pick_terminal_dep_bearing(g, m, pool, sel_width, None);
    let datas: Vec<NodeId> = (0..n_arms)
        .map(|_| pick_terminal(g, m, pool, width, None))
        .collect();
    let root = make_case_mux(m, pool, sel, &datas, width);
    m.case_mux_built += 1;
    root
}

/// Pool-only procedural casez-mux assembly. Semantics match a
/// wildcarded indexed mux, but emission uses `always_comb casez`.
/// The generated patterns are non-overlapping by construction.
pub(crate) fn build_casez_mux_pool_only(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);
    let (sel_width, patterns) = build_casez_patterns(n_arms);
    let sel = pick_terminal_dep_bearing(g, m, pool, sel_width, None);
    let datas: Vec<NodeId> = (0..n_arms)
        .map(|_| pick_terminal(g, m, pool, width, None))
        .collect();
    let root = make_casez_mux(m, pool, sel, &patterns, &datas, width);
    m.casez_mux_built += 1;
    root
}

/// Pool-only procedural for-fold assembly. Emits a statically bounded
/// `always_comb` loop over fixed-width packed chunks when the target
/// width admits a bounded trip count.
pub(crate) fn build_for_fold_pool_only(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
) -> Option<NodeId> {
    let trip_count = pick_for_fold_trip_count(g, width)?;
    let src_width = width.checked_mul(trip_count)?;
    let src = pick_terminal_dep_bearing(g, m, pool, src_width, None);
    let kind = pick_for_fold_kind(g);
    let root = make_for_fold(m, pool, src, kind, trip_count, width);
    m.for_fold_built += 1;
    Some(root)
}
/// Draw a strictly positive coefficient from the configured range,
/// clamped to fit the target operand width. The returned value is
/// guaranteed to satisfy `1 <= c <= 2^width - 1`, so it always fits in
/// a `width`-bit constant literal without truncation.
pub(crate) fn pick_coefficient(g: &mut Generator, width: u32) -> u128 {
    let width_max: u128 = if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    };
    let coef_min = u128::from(g.cfg.min_coefficient.max(1)).min(width_max);
    let coef_max = u128::from(g.cfg.max_coefficient.max(g.cfg.min_coefficient.max(1)))
        .min(width_max)
        .max(coef_min);
    g.rng.gen_range(coef_min..=coef_max)
}

/// Pick the term count N for the Add/Sub linear-combination motif.
/// Drawn from `[min_gate_arity, max_gate_arity]`.
pub(crate) fn pick_linear_combination_arity(g: &mut Generator) -> u32 {
    let min_n = g.cfg.min_gate_arity;
    let max_n = g.cfg.max_gate_arity.max(min_n);
    g.rng.gen_range(min_n..=max_n)
}

/// For Mul: pick coefficient and signal count jointly. `c == 1` forces
/// `n >= 2` (otherwise `1 * s1 = s1` is structurally dead). Returns
/// `(coef, n_signals)`.
pub(crate) fn pick_mul_coefficient_and_arity(g: &mut Generator, width: u32) -> (u128, u32) {
    let coef = pick_coefficient(g, width);
    let min_n = if coef == 1 {
        g.cfg.min_gate_arity.max(2)
    } else {
        g.cfg.min_gate_arity.max(1)
    };
    let max_n = g.cfg.max_gate_arity.max(min_n);
    let n = g.rng.gen_range(min_n..=max_n);
    (coef, n)
}

/// Assemble `y = (s1*c1) + (s2*c2) + ... + (sn*cn)` — N `Mul` nodes
/// plus one N-arity `Add`. Coefficients drawn per-term.
pub(crate) fn assemble_add_linear_combination(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    signals: &[NodeId],
) -> NodeId {
    debug_assert!(!signals.is_empty());
    let mut terms: Vec<NodeId> = Vec::with_capacity(signals.len());
    for &s in signals {
        let coef = pick_coefficient(g, width);
        let const_node = make_constant(m, pool, width, coef);
        terms.push(make_mul(m, pool, s, const_node, width));
    }
    // Under strict operand-uniqueness, dedup the Mul terms so the
    // outer Add doesn't see duplicate NodeIds. Two terms can be
    // identical when both the signal and the coefficient coincide
    // (same signal by CSE + same random coef) → `make_mul` CSE-
    // dedupes to one NodeId, appearing twice in `terms`.
    if m.operand_duplication_rate < 1.0 {
        let mut seen = std::collections::HashSet::new();
        terms.retain(|t| seen.insert(*t));
    }
    if terms.len() == 1 {
        return terms[0];
    }
    make_nary_add(m, pool, &terms, width)
}

/// Assemble `y = (s1*c1) - (s2*c2) - ... - (sn*cn)` — N `Mul` nodes
/// plus `N-1` chained 2-arity `Sub` nodes (left-associative).
/// Coefficients strictly positive per-term.
pub(crate) fn assemble_sub_linear_combination(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    signals: &[NodeId],
) -> NodeId {
    debug_assert!(!signals.is_empty());
    let mut terms: Vec<NodeId> = Vec::with_capacity(signals.len());
    for &s in signals {
        let coef = pick_coefficient(g, width);
        let const_node = make_constant(m, pool, width, coef);
        terms.push(make_mul(m, pool, s, const_node, width));
    }
    if terms.len() == 1 {
        return terms[0];
    }
    let mut acc = terms[0];
    for &t in &terms[1..] {
        acc = make_sub(m, pool, acc, t, width);
    }
    acc
}

/// Assemble `y = c * s1 * s2 * ... * sN` as a single N+1-arity `Mul`
/// node. Caller supplies the pre-drawn coefficient (must be >= 1) and
/// signal list (must have `>= 2` entries when `c == 1`).
pub(crate) fn assemble_mul_linear_combination(
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    coef: u128,
    signals: &[NodeId],
) -> NodeId {
    debug_assert!(!signals.is_empty());
    debug_assert!(
        coef != 1 || signals.len() >= 2,
        "c == 1 requires >= 2 signals"
    );
    // When operand-uniqueness is strict, dedup the signals list
    // before interning. `c * x * x * y` becomes `c * x * y`, which
    // is semantically different (loses the x² factor) but honours
    // the user's explicit no-duplicates contract. At rate 1.0 the
    // user opts in to the x² shape.
    let deduped_signals: Vec<NodeId> = if m.operand_duplication_rate < 1.0 {
        let mut seen = std::collections::HashSet::new();
        signals
            .iter()
            .copied()
            .filter(|s| seen.insert(*s))
            .collect()
    } else {
        signals.to_vec()
    };
    // Preserve the `coef == 1 ⇒ n >= 2` invariant after dedup.
    if coef == 1 && deduped_signals.len() < 2 {
        // Only one distinct signal → `1 * x * x = x * x = x²`, which
        // is degenerate under strict uniqueness. Fall through to the
        // single signal passthrough.
        return deduped_signals[0];
    }
    let const_node = make_constant(m, pool, width, coef);
    let mut operands: Vec<NodeId> = Vec::with_capacity(deduped_signals.len() + 1);
    operands.push(const_node);
    operands.extend_from_slice(&deduped_signals);
    // Final dedup: the coefficient constant can collide with a
    // signal that happens to be the same-value, same-width constant
    // (via CSE). `deduped_signals` only deduped among signals; the
    // coef was added after. Under strict operand uniqueness, drop
    // any repeat. If that collapses operands to < 2, the remaining
    // single operand IS the product (when coef == x and signal ==
    // x, x * x was forbidden by operand uniqueness, so returning x
    // alone is the Rule-8-consistent resolution).
    if m.operand_duplication_rate < 1.0 {
        let mut seen = std::collections::HashSet::new();
        operands.retain(|o| seen.insert(*o));
        if operands.len() < 2 {
            return operands[0];
        }
    }
    make_nary_mul(m, pool, &operands, width)
}

/// Dispatch for the coefficient motif when signal picking is via the
/// recursive `build_cone` path (sequential / shuffled / interleaved
/// block-internals). Selects N (and coefficient for Mul), builds
/// signals recursively, then calls the appropriate assembler.
pub(crate) fn build_linear_combination_recursive(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    op: GateOp,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    match op {
        GateOp::Add => {
            let n = pick_linear_combination_arity(g);
            let signals: Vec<NodeId> = (0..n)
                .map(|_| build_cone(g, m, pool, worklist, width, depth + 1, exclude))
                .collect();
            assemble_add_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Sub => {
            let n = pick_linear_combination_arity(g);
            let signals: Vec<NodeId> = (0..n)
                .map(|_| build_cone(g, m, pool, worklist, width, depth + 1, exclude))
                .collect();
            assemble_sub_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Mul => {
            let (coef, n) = pick_mul_coefficient_and_arity(g, width);
            let signals: Vec<NodeId> = (0..n)
                .map(|_| build_cone(g, m, pool, worklist, width, depth + 1, exclude))
                .collect();
            assemble_mul_linear_combination(m, pool, width, coef, &signals)
        }
        _ => unreachable!("build_linear_combination_recursive: op must be Add/Sub/Mul"),
    }
}

/// Pick a constant shift amount for a W-bit shift. Drawn uniformly
/// from `[min_shift_amount, max_shift_amount]` clamped to `[0, W-1]`.
/// A shift by `>= W` on an unsigned W-bit value is always zero; we
/// restrict to in-range amounts so the shift has semantic weight.
pub(crate) fn pick_shift_amount(g: &mut Generator, value_width: u32) -> u128 {
    let max_meaningful = value_width.saturating_sub(1);
    let lo = g.cfg.min_shift_amount.min(max_meaningful);
    let hi = g.cfg.max_shift_amount.min(max_meaningful).max(lo);
    u128::from(g.rng.gen_range(lo..=hi))
}

/// Build a shift (`Shl`/`Shr`) with a constant shift amount:
/// `value_signal OP constant`. The shift-amount constant width is
/// chosen small (just enough to hold the value) — typically 1..8 bits.
pub(crate) fn build_shift_const_amount(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    op: GateOp,
    value_node: NodeId,
    value_width: u32,
) -> NodeId {
    debug_assert!(matches!(op, GateOp::Shl | GateOp::Shr));
    let amount = pick_shift_amount(g, value_width);
    // Choose a compact constant width: enough bits to hold `amount`.
    // `leading_zeros` on a u128 returns in 0..=128.
    let const_width = (128u32 - amount.max(1).leading_zeros()).max(1);
    let const_node = make_constant(m, pool, const_width, amount);
    let deps = node_deps(m, value_node);
    let (node_id, is_new) =
        m.intern_gate(op, vec![value_node, const_node], value_width, deps.clone());
    if is_new {
        pool.add(node_id, value_width, deps);
    }
    node_id
}

/// Pick the internal operand width K for a comparison. Matches
/// `input_widths_for`'s draw range (1..=8).
pub(crate) fn pick_comparison_operand_width(g: &mut Generator) -> u32 {
    g.rng.gen_range(1..=8)
}

/// Draw a constant comparand value for a K-bit comparison operand.
/// Clamped to `[0, 2^K - 1]` to fit the operand width.
pub(crate) fn pick_comparand_value(g: &mut Generator, operand_width: u32) -> u128 {
    let width_max: u128 = if operand_width >= 128 {
        u128::MAX
    } else {
        (1u128 << operand_width) - 1
    };
    let hi = u128::from(g.cfg.max_comparand).min(width_max);
    let lo = u128::from(g.cfg.min_comparand).min(hi);
    g.rng.gen_range(lo..=hi)
}

/// Build a comparison with a constant RHS: `lhs_signal OP const`.
/// Output is always 1-bit (comparisons reduce to a flag).
pub(crate) fn build_comparison_const_comparand(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    op: GateOp,
    lhs: NodeId,
    operand_width: u32,
) -> NodeId {
    debug_assert!(matches!(
        op,
        GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
    ));
    let value = pick_comparand_value(g, operand_width);
    let const_node = make_constant(m, pool, operand_width, value);
    build_comparison_gate(m, pool, op, lhs, const_node)
}

/// Find an N (number of request inputs) for a priority-encoder block
/// such that `ceil_log2(N) == target_width`, constrained to the
/// configured `[min_mux_arms, max_mux_arms]` range. Returns None if
/// no N in range produces an output matching `target_width`.
pub(crate) fn pick_priority_encoder_n(g: &mut Generator, target_width: u32) -> Option<u32> {
    if target_width == 0 || target_width > 32 {
        return None;
    }
    // For W-bit output, N is in [2^(W-1) + 1 .. 2^W], except W=1 where
    // N=2 (ceil_log2(2) == 1).
    let n_min = if target_width == 1 {
        2
    } else {
        (1u32 << (target_width - 1)) + 1
    };
    let n_max = if target_width == 32 {
        u32::MAX
    } else {
        1u32 << target_width
    };
    let knob_min = g.cfg.min_mux_arms.max(2);
    let knob_max = g.cfg.max_mux_arms.max(knob_min);
    let eff_min = n_min.max(knob_min);
    let eff_max = n_max.min(knob_max);
    if eff_min > eff_max {
        return None;
    }
    Some(g.rng.gen_range(eff_min..=eff_max))
}

/// Assemble a priority encoder as a chained ternary:
///   `y = req_0 ? 0 : req_1 ? 1 : ... : req_{N-1} ? N-1 : 0`
/// The fall-through 0 when no request is asserted. All request bits
/// are 1-bit signals; the output is `target_width`-bit.
pub(crate) fn assemble_priority_encoder(
    m: &mut Module,
    pool: &mut SignalPool,
    target_width: u32,
    req_bits: &[NodeId],
) -> NodeId {
    debug_assert!(!req_bits.is_empty());
    let n = req_bits.len() as u32;
    let fall_through = make_constant(m, pool, target_width, 0);
    let mut tail = fall_through;
    for idx_rev in 0..n {
        let idx = n - 1 - idx_rev;
        let index_const = make_constant(m, pool, target_width, u128::from(idx));
        tail = make_mux(
            m,
            pool,
            req_bits[idx as usize],
            index_const,
            tail,
            target_width,
        );
    }
    tail
}

/// Build a priority-encoder block via recursive signal picking for
/// the request bits. Returns None if the caller's target width is
/// incompatible with any N in the configured block-arity range.
pub(crate) fn build_priority_encoder_recursive(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    target_width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> Option<NodeId> {
    let n = pick_priority_encoder_n(g, target_width)?;
    let req_bits: Vec<NodeId> = (0..n)
        .map(|_| build_cone(g, m, pool, worklist, 1, depth + 1, exclude))
        .collect();
    let root = assemble_priority_encoder(m, pool, target_width, &req_bits);
    m.priority_encoder_built += 1;
    Some(root)
}

/// Pool-only variant for the graph-first strategy.
pub(crate) fn build_priority_encoder_pool(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    target_width: u32,
) -> Option<NodeId> {
    let n = pick_priority_encoder_n(g, target_width)?;
    let req_bits: Vec<NodeId> = (0..n)
        .map(|_| pick_terminal_dep_bearing(g, m, pool, 1, None))
        .collect();
    let root = assemble_priority_encoder(m, pool, target_width, &req_bits);
    m.priority_encoder_built += 1;
    Some(root)
}

pub(crate) fn is_comparison_op(op: GateOp) -> bool {
    matches!(
        op,
        GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
    )
}

/// Dispatch for the coefficient motif when signal picking is pool-only
/// (graph-first strategy). Same shapes as the recursive variant, but
/// signals come from `pick_terminal` instead of `build_cone`.
pub(crate) fn build_linear_combination_pool(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    op: GateOp,
    width: u32,
) -> NodeId {
    match op {
        GateOp::Add => {
            let n = pick_linear_combination_arity(g);
            let signals = pick_signals_with_dup_rate(g, m, pool, width, n as usize, None);
            assemble_add_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Sub => {
            let n = pick_linear_combination_arity(g);
            let signals = pick_signals_with_dup_rate(g, m, pool, width, n as usize, None);
            assemble_sub_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Mul => {
            let (coef, n) = pick_mul_coefficient_and_arity(g, width);
            let signals = pick_signals_with_dup_rate(g, m, pool, width, n as usize, None);
            assemble_mul_linear_combination(m, pool, width, coef, &signals)
        }
        _ => unreachable!("build_linear_combination_pool: op must be Add/Sub/Mul"),
    }
}

/// `none_selected = ~(sel_0 | sel_1 | ... | sel_{M-1})`.
/// 1-bit output, 1 when no select is asserted.
pub(crate) fn make_none_selected(m: &mut Module, pool: &mut SignalPool, arms: &[MuxArm]) -> NodeId {
    debug_assert!(!arms.is_empty());
    let sels: Vec<NodeId> = arms.iter().map(|a| a.sel).collect();
    let acc = or_reduce_terms(m, pool, &sels, 1);
    let acc_deps = node_deps(m, acc);
    let (node_id, is_new) = m.intern_gate(GateOp::Not, vec![acc], 1, acc_deps.clone());
    if is_new {
        pool.add(node_id, 1, acc_deps);
    }
    node_id
}

pub(crate) fn or_reduce_terms(
    m: &mut Module,
    pool: &mut SignalPool,
    terms: &[NodeId],
    width: u32,
) -> NodeId {
    debug_assert!(!terms.is_empty());
    // Dedup in first-occurrence order. `x | x = x` at any scale.
    // When all terms are identical the reduce is a single passthrough.
    let mut unique: Vec<NodeId> = Vec::with_capacity(terms.len());
    for &t in terms {
        if !unique.contains(&t) {
            unique.push(t);
        }
    }
    let mut acc = unique[0];
    for &t in &unique[1..] {
        let deps = DepSet::union(&[&node_deps(m, acc), &node_deps(m, t)]);
        let (node_id, is_new) = m.intern_gate(GateOp::Or, vec![acc, t], width, deps.clone());
        if is_new {
            pool.add(node_id, width, deps);
        }
        acc = node_id;
    }
    acc
}
/// Build an M-to-1 combinational mux block.
///
/// A *block* (not an operator — see `book/src/structural-rules.md`):
/// ports are M data inputs (width W) + 1 select (1-bit × M for
/// OneHot, ceil(log2(M))-bit for Encoded). No Q-feedback axis because
/// combinational muxes have no state.
///
/// When no select asserts (OneHot) or select is out of range
/// (Encoded, when M is not a power of 2), output is 0.
#[instrument(level = "trace", skip(g, m, pool, worklist))]
pub(crate) fn build_comb_mux(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);

    let encoded = roll_knob(
        g,
        m,
        KnobId::CombMuxEncodingProb,
        g.cfg.comb_mux_encoding_prob,
    );
    if encoded {
        m.comb_mux_encoded_built += 1;
        build_comb_mux_encoded(g, m, pool, worklist, width, depth, exclude, n_arms)
    } else {
        m.comb_mux_one_hot_built += 1;
        build_comb_mux_one_hot(g, m, pool, worklist, width, depth, exclude, n_arms)
    }
}

#[instrument(level = "trace", skip(g, m, pool, worklist))]
pub(crate) fn build_case_mux_recursive(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);
    let sel_width = ceil_log2(n_arms);
    let sel = build_cone(g, m, pool, worklist, sel_width, depth + 1, exclude);
    let datas: Vec<NodeId> = (0..n_arms)
        .map(|_| build_cone(g, m, pool, worklist, width, depth + 1, exclude))
        .collect();
    let root = make_case_mux(m, pool, sel, &datas, width);
    m.case_mux_built += 1;
    root
}

#[instrument(level = "trace", skip(g, m, pool, worklist))]
pub(crate) fn build_casez_mux_recursive(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    let min_arms = g.cfg.min_mux_arms.max(2);
    let max_arms = g.cfg.max_mux_arms.max(min_arms);
    let n_arms = g.rng.gen_range(min_arms..=max_arms);
    let (sel_width, patterns) = build_casez_patterns(n_arms);
    let sel = build_cone(g, m, pool, worklist, sel_width, depth + 1, exclude);
    let datas: Vec<NodeId> = (0..n_arms)
        .map(|_| build_cone(g, m, pool, worklist, width, depth + 1, exclude))
        .collect();
    let root = make_casez_mux(m, pool, sel, &patterns, &datas, width);
    m.casez_mux_built += 1;
    root
}

#[instrument(level = "trace", skip(g, m, pool, worklist))]
pub(crate) fn build_for_fold_recursive(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
) -> Option<NodeId> {
    let trip_count = pick_for_fold_trip_count(g, width)?;
    let src_width = width.checked_mul(trip_count)?;
    let src = build_cone(g, m, pool, worklist, src_width, depth + 1, exclude);
    let kind = pick_for_fold_kind(g);
    let root = make_for_fold(m, pool, src, kind, trip_count, width);
    m.for_fold_built += 1;
    Some(root)
}

pub(crate) fn build_comb_mux_one_hot(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
    n_arms: u32,
) -> NodeId {
    let mut arms: Vec<MuxArm> = Vec::with_capacity(n_arms as usize);
    for _ in 0..n_arms {
        let data = build_cone(g, m, pool, worklist, width, depth + 1, exclude);
        let sel = build_cone(g, m, pool, worklist, 1, depth + 1, exclude);
        arms.push(MuxArm { data, sel });
    }
    // Assemble D = OR_i({W{sel_i}} & data_i). No Q-feedback term —
    // combinational muxes have no state, so "no select fires" yields 0.
    let mut term_nodes: Vec<NodeId> = Vec::with_capacity(arms.len());
    for arm in &arms {
        let mask = replicate_to_width(m, pool, arm.sel, width);
        let term = make_and(m, pool, mask, arm.data, width);
        term_nodes.push(term);
    }
    or_reduce_terms(m, pool, &term_nodes, width)
}

pub(crate) fn build_comb_mux_encoded(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
    n_arms: u32,
) -> NodeId {
    let sel_width = ceil_log2(n_arms);
    let sel = build_cone(g, m, pool, worklist, sel_width, depth + 1, exclude);
    let mut datas: Vec<NodeId> = Vec::with_capacity(n_arms as usize);
    for _ in 0..n_arms {
        datas.push(build_cone(g, m, pool, worklist, width, depth + 1, exclude));
    }
    // Assemble chained ternary: (sel==0)? data_0 : (sel==1)? data_1 : ... : 0.
    let fall_through = make_constant(m, pool, width, 0);
    let mut tail = fall_through;
    for idx_rev in 0..n_arms {
        let idx = n_arms - 1 - idx_rev;
        let eq = make_eq_const(m, pool, sel, sel_width, idx as u128);
        tail = make_mux(m, pool, eq, datas[idx as usize], tail, width);
    }
    tail
}

pub(crate) fn make_case_mux(
    m: &mut Module,
    pool: &mut SignalPool,
    sel: NodeId,
    datas: &[NodeId],
    width: u32,
) -> NodeId {
    debug_assert!(datas.len() >= 2);
    let mut operands = Vec::with_capacity(datas.len() + 1);
    operands.push(sel);
    operands.extend_from_slice(datas);
    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let (node_id, is_new) = m.intern_gate(GateOp::CaseMux, operands, width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

pub(crate) fn make_casez_mux(
    m: &mut Module,
    pool: &mut SignalPool,
    sel: NodeId,
    patterns: &[(u128, u128)],
    datas: &[NodeId],
    width: u32,
) -> NodeId {
    debug_assert!(patterns.len() >= 2);
    debug_assert_eq!(patterns.len(), datas.len());
    let sel_width = m.nodes[sel as usize].width();
    let mut operands = Vec::with_capacity(1 + patterns.len() * 3);
    operands.push(sel);
    for ((value, wild_mask), data) in patterns.iter().copied().zip(datas.iter().copied()) {
        operands.push(make_constant(m, pool, sel_width, value));
        operands.push(make_constant(m, pool, sel_width, wild_mask));
        operands.push(data);
    }
    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let (node_id, is_new) = m.intern_gate(GateOp::CasezMux, operands, width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

pub(crate) fn make_for_fold(
    m: &mut Module,
    pool: &mut SignalPool,
    src: NodeId,
    kind: ForFoldKind,
    trip_count: u32,
    chunk_width: u32,
) -> NodeId {
    let operands = vec![src];
    let deps = node_deps(m, src);
    let (node_id, is_new) = m.intern_gate(
        GateOp::ForFold {
            kind,
            trip_count,
            chunk_width,
        },
        operands,
        chunk_width,
        deps.clone(),
    );
    if is_new {
        pool.add(node_id, chunk_width, deps);
    }
    node_id
}

pub(crate) fn pick_for_fold_trip_count(g: &mut Generator, width: u32) -> Option<u32> {
    let min_iters = g.cfg.min_gate_arity.max(2);
    let max_iters = g.cfg.max_gate_arity.max(min_iters);
    let valid: Vec<u32> = (min_iters..=max_iters)
        .filter(|iters| {
            width
                .checked_mul(*iters)
                .map(|src_width| src_width <= 128)
                .unwrap_or(false)
        })
        .collect();
    (!valid.is_empty()).then(|| valid[g.rng.gen_range(0..valid.len())])
}

pub(crate) fn pick_for_fold_kind(g: &mut Generator) -> ForFoldKind {
    match g.rng.gen_range(0..4) {
        0 => ForFoldKind::Xor,
        1 => ForFoldKind::Or,
        2 => ForFoldKind::And,
        _ => ForFoldKind::Add,
    }
}

pub(crate) fn build_casez_patterns(n_arms: u32) -> (u32, Vec<(u128, u128)>) {
    debug_assert!(n_arms >= 2);
    let wildcard_bits = 1;
    let sel_width = ceil_log2(n_arms) + wildcard_bits;
    let wildcard_mask = width_mask(wildcard_bits);
    let patterns = (0..n_arms)
        .map(|idx| (u128::from(idx) << wildcard_bits, wildcard_mask))
        .collect();
    (sel_width, patterns)
}
