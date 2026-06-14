//! Fanin cone recursion. See `book/src/algorithm.md` for the full spec.
//!
//! Combinational + sequential. Recursion is the core principle:
//! - Q is a leaf in the *current* cone (terminates the descent).
//! - D opens a *new* cone, queued on the worklist for later draining.
//! - The same `build_cone` function constructs both.

#![allow(clippy::too_many_arguments)]

use super::{pool::SignalPool, Generator};
use crate::config::Config;
use crate::ir::{
    DepSet, Flop, FlopId, FlopKind, FlopMux, ForFoldKind, GateOp, KnobId, Module, MuxArm, Node,
    NodeId, ResetKind,
};
use rand::Rng;
use tracing::{debug, instrument, trace, warn};

// cone submodules (CONE-DECOMPOSITION). The `pub(crate) use <sub>::*`
// re-exports keep every `crate::gen::cone::<symbol>` path stable for
// external callers and give each submodule's `use super::*;` visibility
// of its siblings — preserving the original single-file namespace.
mod primitives;
mod semantic;
mod snapshot;
pub(crate) use primitives::*;
pub(crate) use semantic::*;
pub(crate) use snapshot::*;

/// Worklist of flops whose D-input cone has not yet been built.
pub type FlopWorklist = Vec<FlopId>;

/// Perform a probability-roll against a named knob and record the
/// attempt + outcome in `m.knob_rolls`. Single place to add
/// telemetry — every `gen_bool(cfg.<prob>)` site in this module
/// routes through here so the empirical fire-rate
/// `fires / attempts` can be compared against the configured
/// probability (knob-effectiveness validation per the
/// measurability doctrine).
fn roll_knob(g: &mut Generator, m: &mut Module, knob: KnobId, prob: f64) -> bool {
    let fired = g.rng.gen_bool(prob.min(1.0));
    m.knob_rolls.record(knob, fired);
    fired
}

/// Per-module construction-time node-budget check
/// (`WORKLOAD-MEMORY-SAFETY.3`). True once the module's node arena has
/// reached `cfg.max_nodes_per_module`. The sentinel `0` means *unlimited*
/// and is the default, so this returns `false` on the default path and
/// the budget never perturbs construction or RNG consumption — generated
/// RTL stays byte-identical. When non-zero, callers use this to force
/// terminal selection (steer to existing signals; never truncate a
/// finished cone), bounding peak per-module memory.
#[inline]
fn node_budget_reached(g: &Generator, m: &Module) -> bool {
    let budget = g.cfg.max_nodes_per_module;
    budget != 0 && m.nodes.len() >= budget as usize
}

/// Retry loop around `build_cone` that rejects trivial (empty dep-set)
/// roots. Bounded to avoid pathological infinite retries; if we exceed
/// the budget, the last attempt is accepted.
///
/// `exclude` lets rare callers forbid a specific `NodeId` from
/// terminal selection. Flop D-cones deliberately pass `None`: Rule 2
/// allows a flop's own Q to appear freely in direct-D, data, and select
/// sub-cones.
#[instrument(level = "debug", skip(g, m, pool, worklist))]
pub fn build_cone_with_retry(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    exclude: Option<NodeId>,
) -> NodeId {
    const MAX_RETRIES: u32 = 4;
    for attempt in 0..MAX_RETRIES {
        let snapshot = take_construction_snapshot(m, pool, worklist);
        let node = build_cone(g, m, pool, worklist, width, 0, exclude);
        let deps = node_deps(m, node);
        if !deps.is_empty() {
            debug!(attempt, node, "cone root dep-bearing ✅");
            return node;
        }
        warn!(attempt, "🔁 cone root empty-dep, retrying");
        rollback_construction_snapshot(m, pool, worklist, snapshot);
    }
    warn!("⚠️ cone retry budget exhausted, accepting last attempt");
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
/// All sub-cones (data, select, or the M==0 direct D-cone) pass
/// `exclude = None`. A flop's own Q may appear freely as a leaf in its
/// D-cone; `FlopKind::QFeedback` is an additional structured feedback
/// idiom, not the only legal Q→D path.
///
/// The drain may itself enqueue more flops; the loop handles that
/// until quiescence.
// ------------------------------------------------------------------
// Interleaved construction: frame state machine for output cones.
// See book/src/construction-strategies.md.
// ------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Dest {
    Output(usize),
    Slot { frame_id: usize, slot: usize },
}

struct SignalFrame {
    width: u32,
    depth: u32,
    exclude: Option<NodeId>,
    dest: Dest,
}

struct GateFrame {
    op: GateOp,
    operands: Vec<Option<NodeId>>,
    pending: usize,
    width: u32,
    dest: Dest,
}

// ------------------------------------------------------------------
// Graph-first construction: no per-output cone recursion. Grow a
// gate pool with no output attribution; operands of each new unit
// are picked from the existing pool. Flop D-cones resolved after
// pool growth using pool-only picks. Output drive-roots picked from
// the pool at the end. See book/src/construction-strategies.md.
// ------------------------------------------------------------------

/// Grow a gate pool and pick drive-roots for each output from it.
/// Returns the drive-root NodeId per output, in declaration order.
#[instrument(level = "info", skip(g, m, pool, worklist))]
pub fn build_graph_first(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> Vec<NodeId> {
    debug!(
        target = g.cfg.graph_first_pool_size,
        "graph-first: growing pool"
    );
    // Phase 1 — grow the pool by `graph_first_pool_size` top-level
    // units. A unit is one operator gate, one flop (with deferred D),
    // or one comb-mux block. Comb-mux and flop-mux assembly internally
    // creates multiple primitive gates; those are NOT counted toward
    // the pool size target. The counter only advances on successful
    // unit emission; skipped emissions (e.g., anti-collapse rejects)
    // do not advance it but consume an iteration budget to prevent
    // pathological infinite loops.
    let target = g.cfg.graph_first_pool_size.max(1) as usize;
    let mut emitted: usize = 0;
    let mut iterations: usize = 0;
    let iter_budget = target.saturating_mul(8);
    while emitted < target && iterations < iter_budget {
        // Node-budget governor (`WORKLOAD-MEMORY-SAFETY.3`): stop growing
        // the pool once the module's node arena reaches the budget.
        // Sentinel 0 = unlimited ⇒ never fires ⇒ byte-identical.
        if node_budget_reached(g, m) {
            break;
        }
        iterations += 1;
        if grow_pool_one_unit(g, m, pool, worklist) {
            emitted += 1;
        }
    }

    debug!(
        emitted,
        iterations,
        pending_flops = worklist.len(),
        "graph-first: pool grown, draining flops"
    );

    // Phase 2 — resolve flop D-cones using pool-only picks. By this
    // point the pool is fully grown, so every flop has the full pool
    // to pick its D-mux operands from. Q-feedback is permitted freely
    // (Rule 2) — `exclude` is None throughout.
    drain_flop_worklist_pool_only(g, m, pool, worklist);

    // Phase 3 — pick a drive-root for each output from the pool.
    // `pick_terminal` handles the adapter fallback when no matching-
    // width entry exists.
    debug!("graph-first: picking drive-roots");
    (0..m.outputs.len())
        .map(|i| pick_terminal(g, m, pool, m.outputs[i].width, None))
        .collect()
}

#[instrument(level = "trace", skip(g, m, pool, worklist))]
fn grow_pool_one_unit(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> bool {
    let width = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);

    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    if flop_allowed && roll_knob(g, m, g.active_flop_knob, g.cfg.flop_prob) {
        trace!(width, "🧱 flop block");
        build_flop_leaf(g, m, pool, worklist, width);
        return true;
    }

    if roll_knob(g, m, KnobId::CombMuxProb, g.cfg.comb_mux_prob) {
        trace!(width, "🧱 comb-mux block");
        build_comb_mux_pool_only(g, m, pool, width);
        return true;
    }

    if roll_knob(g, m, KnobId::CaseMuxProb, g.cfg.case_mux_prob) {
        trace!(width, "🧱 case-mux block");
        build_case_mux_pool_only(g, m, pool, width);
        return true;
    }

    if roll_knob(g, m, KnobId::CasezMuxProb, g.cfg.casez_mux_prob) {
        trace!(width, "🧱 casez-mux block");
        build_casez_mux_pool_only(g, m, pool, width);
        return true;
    }

    if roll_knob(g, m, KnobId::ForFoldProb, g.cfg.for_fold_prob)
        && build_for_fold_pool_only(g, m, pool, width).is_some()
    {
        trace!(width, "🧱 for-fold block");
        return true;
    }

    // Priority-encoder block (pool-only). Skip if no N compatible with
    // target width.
    if roll_knob(
        g,
        m,
        KnobId::PriorityEncoderProb,
        g.cfg.priority_encoder_prob,
    ) && build_priority_encoder_pool(g, m, pool, width).is_some()
    {
        trace!(width, "🧱 priority-encoder block");
        return true;
    }

    let op = pick_gate(g, width);
    trace!(?op, width, "🔧 operator gate");

    // Coefficient motif (pool-only signal picks). Same doctrine as the
    // recursive path: Add/Sub/Mul with coefficient_prob probability
    // becomes a linear-combination compound.
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && roll_knob(g, m, KnobId::CoefficientProb, g.cfg.coefficient_prob)
    {
        trace!(?op, "➕ linear-combination motif");
        build_linear_combination_pool(g, m, pool, op, width);
        return true;
    }

    // Constant shift-amount motif (pool-only). Value operand is a
    // pool pick; shift amount is a literal constant.
    if matches!(op, GateOp::Shl | GateOp::Shr)
        && roll_knob(
            g,
            m,
            KnobId::ConstShiftAmountProb,
            g.cfg.const_shift_amount_prob,
        )
    {
        trace!(?op, "⏩ const-shift-amount motif");
        let value = pick_terminal_dep_bearing(g, m, pool, width, None);
        build_shift_const_amount(g, m, pool, op, value, width);
        return true;
    }

    // Constant comparand motif (pool-only). LHS is a pool pick of
    // internal operand width K; RHS is a literal constant. Output
    // is 1-bit.
    if is_comparison_op(op)
        && roll_knob(g, m, KnobId::ConstComparandProb, g.cfg.const_comparand_prob)
    {
        trace!(?op, "🔍 const-comparand motif");
        let k = pick_comparison_operand_width(g);
        let lhs = pick_terminal_dep_bearing(g, m, pool, k, None);
        build_comparison_const_comparand(g, m, pool, op, lhs, k);
        return true;
    }

    let operand_widths = input_widths_for(op, width, &g.cfg, &mut g.rng);
    for attempt in 0..4 {
        let operands: Vec<NodeId> = operand_widths
            .iter()
            .map(|w| pick_terminal(g, m, pool, *w, None))
            .collect();
        if !violates_anti_collapse(op, &operands, m) {
            if is_comparison_op(op) {
                debug_assert_eq!(operands.len(), 2);
                build_comparison_gate(m, pool, op, operands[0], operands[1]);
            } else {
                let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
                let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
                let (node_id, is_new) = m.intern_gate(op, operands, width, deps.clone());
                if is_new {
                    pool.add(node_id, width, deps);
                }
            }
            return true;
        }
        warn!(?op, attempt, "🔁 anti-collapse hit, retrying operand pick");
    }
    warn!(?op, "❌ anti-collapse retries exhausted, unit skipped");
    false
}

/// Pool-only comb-mux assembly (mirrors `build_comb_mux` but
/// sub-cones are pool picks instead of recursive builds).
fn build_comb_mux_pool_only(
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
fn build_case_mux_pool_only(
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
fn build_casez_mux_pool_only(
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
fn build_for_fold_pool_only(
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

/// Pool-only flop D-cone drain (mirrors `drain_flop_worklist` but
/// operand sub-cones are pool picks). Reuses `assemble_flop_d_one_hot`
/// and `assemble_flop_d_encoded` for the mux-tree assembly.
fn drain_flop_worklist_pool_only(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) {
    while let Some(flop_id) = worklist.pop() {
        let width = m.flops[flop_id as usize].width;
        let kind = m.flops[flop_id as usize].kind;
        let q_node = m.flops[flop_id as usize].q;
        let exclude: Option<NodeId> = None;

        let m_arms = pick_mux_arm_count(g);
        if m_arms == 0 {
            let d_node = pick_terminal(g, m, pool, width, exclude);
            m.flops[flop_id as usize].d = Some(d_node);
            m.flops[flop_id as usize].mux = FlopMux::None;
            continue;
        }

        let encoded = roll_knob(
            g,
            m,
            KnobId::FlopMuxEncodingProb,
            g.cfg.flop_mux_encoding_prob,
        );
        if encoded {
            let sel_width = ceil_log2(m_arms);
            let sel = pick_terminal_dep_bearing(g, m, pool, sel_width, exclude);
            let n_data_slots = match kind {
                FlopKind::ZeroDefault => m_arms as usize,
                FlopKind::QFeedback => (m_arms - 1) as usize,
            };
            let datas = pick_datas_with_dup_cap(g, m, pool, width, n_data_slots, exclude);
            let d = assemble_flop_d_encoded(m, pool, width, sel, sel_width, &datas, kind, q_node);
            m.flops[flop_id as usize].d = Some(d);
            m.flops[flop_id as usize].mux = FlopMux::Encoded { sel, data: datas };
        } else {
            let datas = pick_datas_with_dup_cap(g, m, pool, width, m_arms as usize, exclude);
            let mut arms: Vec<MuxArm> = Vec::with_capacity(m_arms as usize);
            for data in datas {
                let sel = pick_terminal_dep_bearing(g, m, pool, 1, exclude);
                arms.push(MuxArm { data, sel });
            }
            let d = assemble_flop_d_one_hot(m, pool, width, &arms, kind, q_node);
            m.flops[flop_id as usize].d = Some(d);
            m.flops[flop_id as usize].mux = FlopMux::OneHot(arms);
        }
    }
}

/// Build every output cone via a global frame queue. At each step a
/// random `SignalFrame` is popped and processed: blocks (flop,
/// comb-mux) and leaf terminals resolve immediately; operator gates
/// push a `GateFrame` into the in-flight table and enqueue one
/// `SignalFrame` per operand slot. When a gate's last operand
/// resolves, the gate finalizes — the `Node::Gate` is created, added
/// to the pool, and its result is delivered to the gate's own
/// destination (possibly another gate slot, recursing).
///
/// Flop D-cones are *not* interleaved here — they are queued on
/// `worklist` and drained synchronously after all output frames are
/// processed, the same as under `Sequential` and `Shuffled`. That is
/// the "near-symmetric" scope: output-cone construction interleaves,
/// flop D-cones are built depth-first per flop. `GraphFirst` remains a
/// compatibility alias for `Interleaved`; fuller symmetry is future
/// work rather than the meaning of that retired strategy name.
#[instrument(level = "info", skip(g, m, pool, worklist))]
pub fn build_outputs_interleaved(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> Vec<NodeId> {
    let n_out = m.outputs.len();
    let mut per_output_drive: Vec<Option<NodeId>> = vec![None; n_out];
    let mut signal_queue: Vec<SignalFrame> = (0..n_out)
        .map(|idx| SignalFrame {
            width: m.outputs[idx].width,
            depth: 0,
            exclude: None,
            dest: Dest::Output(idx),
        })
        .collect();
    let mut gate_frames: Vec<Option<GateFrame>> = Vec::new();

    while !signal_queue.is_empty() {
        let i = g.rng.gen_range(0..signal_queue.len());
        let frame = signal_queue.swap_remove(i);
        process_signal_frame(
            g,
            m,
            pool,
            worklist,
            frame,
            &mut signal_queue,
            &mut gate_frames,
            &mut per_output_drive,
        );
    }

    per_output_drive
        .into_iter()
        .map(|r| r.expect("interleaved: every output must have a drive root"))
        .collect()
}

#[instrument(
    level = "trace",
    skip(g, m, pool, worklist, frame, signal_queue, gate_frames, per_output_drive),
    fields(depth = frame.depth, width = frame.width)
)]
fn process_signal_frame(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    frame: SignalFrame,
    signal_queue: &mut Vec<SignalFrame>,
    gate_frames: &mut Vec<Option<GateFrame>>,
    per_output_drive: &mut [Option<NodeId>],
) {
    let leaf_prob = (frame.depth as f64) / (g.cfg.max_depth as f64);
    // Node-budget governor (`WORKLOAD-MEMORY-SAFETY.3`): once the module's
    // node arena reaches the budget, force every further recursion point
    // to a terminal — steering to existing signals instead of opening new
    // sub-cones (rules-first; never a truncation). Sentinel 0 = unlimited
    // ⇒ this term is false ⇒ the decision (and RNG consumption) is
    // byte-identical to the historical path.
    let over_budget = node_budget_reached(g, m);
    let force_leaf =
        over_budget || frame.depth >= g.cfg.max_depth || g.rng.gen_bool(leaf_prob.min(1.0));

    if force_leaf {
        let node = pick_terminal(g, m, pool, frame.width, frame.exclude);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Flop block: allocates a Flop and enqueues its D-cone on the worklist.
    // The FlopQ node is returned immediately and the frame resolves.
    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    if flop_allowed && roll_knob(g, m, g.active_flop_knob, g.cfg.flop_prob) {
        let node = build_flop_leaf(g, m, pool, worklist, frame.width);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Comb-mux block: builds its internal sub-cones depth-first within
    // this frame step. Block placement interleaves with other cones;
    // block internals do not. This matches the "near-symmetric" scope.
    if roll_knob(g, m, KnobId::CombMuxProb, g.cfg.comb_mux_prob) {
        let node = build_comb_mux(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth,
            frame.exclude,
        );
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    if roll_knob(g, m, KnobId::CaseMuxProb, g.cfg.case_mux_prob) {
        let node = build_case_mux_recursive(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth,
            frame.exclude,
        );
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    if roll_knob(g, m, KnobId::CasezMuxProb, g.cfg.casez_mux_prob) {
        let node = build_casez_mux_recursive(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth,
            frame.exclude,
        );
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    if roll_knob(g, m, KnobId::ForFoldProb, g.cfg.for_fold_prob) {
        if let Some(node) = build_for_fold_recursive(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth,
            frame.exclude,
        ) {
            deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
            return;
        }
    }

    // Priority-encoder block: compatible only when the frame's target
    // width matches ceil_log2(N) for some N in the block-arity range.
    if roll_knob(
        g,
        m,
        KnobId::PriorityEncoderProb,
        g.cfg.priority_encoder_prob,
    ) {
        if let Some(node) = build_priority_encoder_recursive(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth,
            frame.exclude,
        ) {
            deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
            return;
        }
    }

    // Operator gate: push a GateFrame into the in-flight table, enqueue
    // one SignalFrame per operand slot. The gate finalizes when its
    // last operand resolves (see `deliver`).
    let op = pick_gate(g, frame.width);
    crate::trace_verbose!(
        ?op,
        depth = frame.depth,
        width = frame.width,
        "🎲 interleaved pick_gate"
    );

    // Coefficient motif: Add/Sub/Mul with coefficient_prob becomes a
    // compound linear-combination tree. Built synchronously within
    // this frame step (the tree itself is atomic; its signal leaves
    // come from recursive build_cone just like block internals).
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && roll_knob(g, m, KnobId::CoefficientProb, g.cfg.coefficient_prob)
    {
        let node = build_linear_combination_recursive(
            g,
            m,
            pool,
            worklist,
            op,
            frame.width,
            frame.depth,
            frame.exclude,
        );
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Constant shift-amount motif: Shl/Shr with const_shift_amount_prob.
    // Built synchronously within this frame step; the value operand
    // comes from a recursive build_cone call.
    if matches!(op, GateOp::Shl | GateOp::Shr)
        && roll_knob(
            g,
            m,
            KnobId::ConstShiftAmountProb,
            g.cfg.const_shift_amount_prob,
        )
    {
        let value = build_cone(
            g,
            m,
            pool,
            worklist,
            frame.width,
            frame.depth + 1,
            frame.exclude,
        );
        let node = build_shift_const_amount(g, m, pool, op, value, frame.width);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Constant comparand motif: comparison with const_comparand_prob.
    if is_comparison_op(op)
        && roll_knob(g, m, KnobId::ConstComparandProb, g.cfg.const_comparand_prob)
    {
        let k = pick_comparison_operand_width(g);
        let lhs = build_cone(g, m, pool, worklist, k, frame.depth + 1, frame.exclude);
        let node = build_comparison_const_comparand(g, m, pool, op, lhs, k);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    let operand_widths = input_widths_for(op, frame.width, &g.cfg, &mut g.rng);
    let n_ops = operand_widths.len();
    let frame_id = gate_frames.len();
    gate_frames.push(Some(GateFrame {
        op,
        operands: vec![None; n_ops],
        pending: n_ops,
        width: frame.width,
        dest: frame.dest,
    }));
    for (slot, w) in operand_widths.into_iter().enumerate() {
        // DAG-sharing fork (Rule 16 / share_prob): same as recursive path.
        let shared = if roll_knob(g, m, KnobId::ShareProb, g.cfg.share_prob) {
            try_share(g, pool, w, frame.exclude)
        } else {
            None
        };
        if let Some(shared_id) = shared {
            deliver(
                g,
                m,
                pool,
                shared_id,
                Dest::Slot { frame_id, slot },
                gate_frames,
                per_output_drive,
            );
        } else {
            signal_queue.push(SignalFrame {
                width: w,
                depth: frame.depth + 1,
                exclude: frame.exclude,
                dest: Dest::Slot { frame_id, slot },
            });
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn deliver(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    node: NodeId,
    dest: Dest,
    gate_frames: &mut Vec<Option<GateFrame>>,
    per_output_drive: &mut [Option<NodeId>],
) {
    match dest {
        Dest::Output(idx) => {
            per_output_drive[idx] = Some(node);
        }
        Dest::Slot { frame_id, slot } => {
            let gf = gate_frames[frame_id].as_mut().expect("gate frame live");
            gf.operands[slot] = Some(node);
            gf.pending -= 1;
            if gf.pending == 0 {
                let gf = gate_frames[frame_id].take().unwrap();
                let operands: Vec<NodeId> = gf.operands.into_iter().map(|o| o.unwrap()).collect();

                // Structural anti-collapse. Unlike the recursive path
                // (`build_cone`), the frame machine has already
                // committed each operand sub-tree to `m.nodes` by the
                // time all operand slots resolve — there is no
                // per-frame snapshot to roll back to. Instead, when
                // the parent gate's shape is rejected, deliver an
                // *existing* operand as the fallback so we introduce
                // no new node (pick_terminal would create one) and
                // the operand subtrees remain consumed by their
                // representative. For idempotent / self-inverse /
                // comparison collapses all operands are the same
                // NodeId, so any choice works. For the `mux(s,a,a)`
                // case we choose `operands[1]` (= operands[2]); the
                // `sel` operand may be orphaned if it had no other
                // consumers, which is a bounded edge case tracked in
                // the Rule-18 audit.
                if violates_anti_collapse(gf.op, &operands, m) {
                    // Fallback must have the gate's output width, not
                    // the operand width. For most ops the two match
                    // (And/Or/Xor/Add/Mul/Sub/Mux — operand and output
                    // widths are equal). For comparisons the output is
                    // 1-bit while the operand width is the comparand
                    // width, so `operands[0]` would be wrong-width.
                    // Emit a width-correct constant representing the
                    // algebraic truth value of the collapsed shape:
                    //   Eq(a, a) = 1, Neq(a, a) = 0.
                    let fallback = match gf.op {
                        GateOp::Mux if operands.len() == 3 => operands[1],
                        GateOp::Eq => make_constant(m, pool, gf.width, 1),
                        GateOp::Neq => make_constant(m, pool, gf.width, 0),
                        _ => operands[0],
                    };
                    trace!(
                        op = ?gf.op,
                        fallback,
                        "🔁 anti-collapse: reusing existing operand as fallback (interleaved)"
                    );
                    deliver(g, m, pool, fallback, gf.dest, gate_frames, per_output_drive);
                    return;
                }

                let node_id = if is_comparison_op(gf.op) {
                    debug_assert_eq!(operands.len(), 2);
                    build_comparison_gate(m, pool, gf.op, operands[0], operands[1])
                } else {
                    let deps_vec: Vec<DepSet> =
                        operands.iter().map(|id| node_deps(m, *id)).collect();
                    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
                    let (node_id, is_new) = m.intern_gate(gf.op, operands, gf.width, deps.clone());
                    if is_new {
                        pool.add(node_id, gf.width, deps);
                    }
                    node_id
                };
                deliver(g, m, pool, node_id, gf.dest, gate_frames, per_output_drive);
            }
        }
    }
}

#[instrument(level = "debug", skip(g, m, pool, worklist))]
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
        // Q-feedback is freely permitted in this flop's D-cone: this
        // flop's own Q may appear any number of times as a leaf in any
        // data / select / direct-D sub-cone. The clock edge breaks the
        // loop temporally, so Q→D feedback is a legal sequential pattern.
        // Combinational self-reference is impossible by construction
        // (pool entries pre-date each recursion call — arena-index
        // monotonicity).
        let exclude: Option<NodeId> = None;

        let m_arms = pick_mux_arm_count(g);
        if m_arms == 0 {
            let d_node = build_cone_with_retry(g, m, pool, worklist, width, exclude);
            m.flops[flop_id as usize].d = Some(d_node);
            m.flops[flop_id as usize].mux = FlopMux::None;
            continue;
        }

        let encoded = roll_knob(
            g,
            m,
            KnobId::FlopMuxEncodingProb,
            g.cfg.flop_mux_encoding_prob,
        );
        if encoded {
            let (d_node, mux) =
                drain_flop_encoded(g, m, pool, worklist, width, kind, q_node, m_arms);
            m.flops[flop_id as usize].d = Some(d_node);
            m.flops[flop_id as usize].mux = mux;
        } else {
            let (d_node, mux) =
                drain_flop_one_hot(g, m, pool, worklist, width, kind, q_node, m_arms);
            m.flops[flop_id as usize].d = Some(d_node);
            m.flops[flop_id as usize].mux = mux;
        }
    }
}

fn drain_flop_one_hot(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    kind: FlopKind,
    q_node: NodeId,
    m_arms: u32,
) -> (NodeId, FlopMux) {
    // Q may appear in sub-cones (see drain_flop_worklist note).
    let exclude: Option<NodeId> = None;
    let mut arms: Vec<MuxArm> = Vec::with_capacity(m_arms as usize);
    for _ in 0..m_arms {
        let data = build_cone_with_retry(g, m, pool, worklist, width, exclude);
        let sel = build_cone_with_retry(g, m, pool, worklist, 1, exclude);
        arms.push(MuxArm { data, sel });
    }
    let d_node = assemble_flop_d_one_hot(m, pool, width, &arms, kind, q_node);
    (d_node, FlopMux::OneHot(arms))
}

fn drain_flop_encoded(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    width: u32,
    kind: FlopKind,
    q_node: NodeId,
    m_arms: u32,
) -> (NodeId, FlopMux) {
    // Q may appear in sub-cones (see drain_flop_worklist note).
    let exclude: Option<NodeId> = None;
    let sel_width = ceil_log2(m_arms);
    let sel = build_cone_with_retry(g, m, pool, worklist, sel_width, exclude);

    // For QFeedback the slot at index 0 is Q, not a recursive cone.
    // For ZeroDefault all M slots are recursive cones.
    let datas: Vec<NodeId> = match kind {
        FlopKind::ZeroDefault => (0..m_arms)
            .map(|_| build_cone_with_retry(g, m, pool, worklist, width, exclude))
            .collect(),
        FlopKind::QFeedback => (1..m_arms)
            .map(|_| build_cone_with_retry(g, m, pool, worklist, width, exclude))
            .collect(),
    };

    let d_node = assemble_flop_d_encoded(m, pool, width, sel, sel_width, &datas, kind, q_node);
    (d_node, FlopMux::Encoded { sel, data: datas })
}

/// Ceiling of log2(n). Defined so that `2^ceil_log2(n) >= n` for n >= 1.
fn ceil_log2(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        32 - (n - 1).leading_zeros()
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
fn assemble_flop_d_one_hot(
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

/// Build the gate tree for D from an encoded-select mux.
///
/// ZeroDefault: `D = (sel==0)? data_0 : (sel==1)? data_1 : ... : (sel==M-1)? data_{M-1} : 0`.
/// QFeedback:   `D = (sel==0)? Q      : (sel==1)? data_1 : ... : (sel==M-1)? data_{M-1} : Q`.
///
/// When M is not a power of 2, `sel` can take values outside `[0, M)`;
/// the final else-branch (0 or Q) handles those.
fn assemble_flop_d_encoded(
    m: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    sel: NodeId,
    sel_width: u32,
    datas: &[NodeId],
    kind: FlopKind,
    q_node: NodeId,
) -> NodeId {
    let fall_through: NodeId = match kind {
        FlopKind::ZeroDefault => make_constant(m, pool, width, 0),
        FlopKind::QFeedback => q_node,
    };
    // Iterate indices 0..M in reverse, wrapping the previous tail in a Mux.
    // For QFeedback, index 0 uses Q (not datas[0]); datas has length M-1
    // and corresponds to indices 1..M.
    let m_arms = match kind {
        FlopKind::ZeroDefault => datas.len() as u32,
        FlopKind::QFeedback => datas.len() as u32 + 1,
    };
    let mut tail = fall_through;
    for idx_rev in 0..m_arms {
        let idx = m_arms - 1 - idx_rev;
        let eq = make_eq_const(m, pool, sel, sel_width, idx as u128);
        let data_node = match kind {
            FlopKind::ZeroDefault => datas[idx as usize],
            FlopKind::QFeedback => {
                if idx == 0 {
                    q_node
                } else {
                    datas[(idx - 1) as usize]
                }
            }
        };
        tail = make_mux(m, pool, eq, data_node, tail, width);
    }
    tail
}

/// Draw a strictly positive coefficient from the configured range,
/// clamped to fit the target operand width. The returned value is
/// guaranteed to satisfy `1 <= c <= 2^width - 1`, so it always fits in
/// a `width`-bit constant literal without truncation.
fn pick_coefficient(g: &mut Generator, width: u32) -> u128 {
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
fn pick_linear_combination_arity(g: &mut Generator) -> u32 {
    let min_n = g.cfg.min_gate_arity;
    let max_n = g.cfg.max_gate_arity.max(min_n);
    g.rng.gen_range(min_n..=max_n)
}

/// For Mul: pick coefficient and signal count jointly. `c == 1` forces
/// `n >= 2` (otherwise `1 * s1 = s1` is structurally dead). Returns
/// `(coef, n_signals)`.
fn pick_mul_coefficient_and_arity(g: &mut Generator, width: u32) -> (u128, u32) {
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
fn assemble_add_linear_combination(
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
fn assemble_sub_linear_combination(
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
fn assemble_mul_linear_combination(
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
fn build_linear_combination_recursive(
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
fn pick_shift_amount(g: &mut Generator, value_width: u32) -> u128 {
    let max_meaningful = value_width.saturating_sub(1);
    let lo = g.cfg.min_shift_amount.min(max_meaningful);
    let hi = g.cfg.max_shift_amount.min(max_meaningful).max(lo);
    u128::from(g.rng.gen_range(lo..=hi))
}

/// Build a shift (`Shl`/`Shr`) with a constant shift amount:
/// `value_signal OP constant`. The shift-amount constant width is
/// chosen small (just enough to hold the value) — typically 1..8 bits.
fn build_shift_const_amount(
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
fn pick_comparison_operand_width(g: &mut Generator) -> u32 {
    g.rng.gen_range(1..=8)
}

/// Draw a constant comparand value for a K-bit comparison operand.
/// Clamped to `[0, 2^K - 1]` to fit the operand width.
fn pick_comparand_value(g: &mut Generator, operand_width: u32) -> u128 {
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
fn build_comparison_const_comparand(
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
fn pick_priority_encoder_n(g: &mut Generator, target_width: u32) -> Option<u32> {
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
fn assemble_priority_encoder(
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
fn build_priority_encoder_recursive(
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
fn build_priority_encoder_pool(
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

fn is_comparison_op(op: GateOp) -> bool {
    matches!(
        op,
        GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
    )
}

/// Dispatch for the coefficient motif when signal picking is pool-only
/// (graph-first strategy). Same shapes as the recursive variant, but
/// signals come from `pick_terminal` instead of `build_cone`.
fn build_linear_combination_pool(
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
fn make_none_selected(m: &mut Module, pool: &mut SignalPool, arms: &[MuxArm]) -> NodeId {
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

fn or_reduce_terms(m: &mut Module, pool: &mut SignalPool, terms: &[NodeId], width: u32) -> NodeId {
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

#[instrument(level = "trace", skip(g, m, pool, worklist))]
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
    // Node-budget governor (`WORKLOAD-MEMORY-SAFETY.3`); see
    // `process_signal_frame`. Sentinel 0 = unlimited ⇒ byte-identical.
    let over_budget = node_budget_reached(g, m);
    let force_leaf = over_budget || depth >= g.cfg.max_depth || g.rng.gen_bool(leaf_prob.min(1.0));

    if force_leaf {
        trace!(depth, width, "🍃 leaf via pick_terminal");
        return pick_terminal(g, m, pool, width, exclude);
    }

    // Recursion fork: flop block, comb-mux block, or operator gate.
    // Blocks take priority over operator gates. Ordering between flop
    // and comb-mux is first-come by their independent probability rolls.
    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    let pick_flop = flop_allowed && roll_knob(g, m, g.active_flop_knob, g.cfg.flop_prob);
    if pick_flop {
        trace!(depth, width, "🧱 flop block");
        return build_flop_leaf(g, m, pool, worklist, width);
    }

    let pick_comb_mux = roll_knob(g, m, KnobId::CombMuxProb, g.cfg.comb_mux_prob);
    if pick_comb_mux {
        return build_comb_mux(g, m, pool, worklist, width, depth, exclude);
    }

    if roll_knob(g, m, KnobId::CaseMuxProb, g.cfg.case_mux_prob) {
        return build_case_mux_recursive(g, m, pool, worklist, width, depth, exclude);
    }

    if roll_knob(g, m, KnobId::CasezMuxProb, g.cfg.casez_mux_prob) {
        return build_casez_mux_recursive(g, m, pool, worklist, width, depth, exclude);
    }

    if roll_knob(g, m, KnobId::ForFoldProb, g.cfg.for_fold_prob) {
        if let Some(node) = build_for_fold_recursive(g, m, pool, worklist, width, depth, exclude) {
            return node;
        }
    }

    // Priority-encoder block: compatible only when target width matches
    // ceil_log2(N) for some N in the block-arity range.
    if roll_knob(
        g,
        m,
        KnobId::PriorityEncoderProb,
        g.cfg.priority_encoder_prob,
    ) {
        if let Some(node) =
            build_priority_encoder_recursive(g, m, pool, worklist, width, depth, exclude)
        {
            return node;
        }
    }

    let op = pick_gate(g, width);
    crate::trace_verbose!(?op, depth, width, "🎲 build_cone pick_gate");

    // Coefficient motif: when the picked op is Add / Sub / Mul and the
    // per-op probability fires, emit a linear-combination compound tree
    // (see `book/src/structural-rules.md` "Roles of constants in RTL").
    // Signals are picked via the usual recursive path.
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && roll_knob(g, m, KnobId::CoefficientProb, g.cfg.coefficient_prob)
    {
        crate::trace_verbose!(?op, depth, width, "➕ linear-combination motif (recursive)");
        return build_linear_combination_recursive(g, m, pool, worklist, op, width, depth, exclude);
    }

    // Constant shift-amount motif: when the picked op is Shl/Shr and
    // the per-shift probability fires, emit `value OP const` with a
    // literal shift amount instead of a barrel shifter.
    if matches!(op, GateOp::Shl | GateOp::Shr)
        && roll_knob(
            g,
            m,
            KnobId::ConstShiftAmountProb,
            g.cfg.const_shift_amount_prob,
        )
    {
        let value = build_cone(g, m, pool, worklist, width, depth + 1, exclude);
        return build_shift_const_amount(g, m, pool, op, value, width);
    }

    // Constant comparand motif: when the picked op is a comparison
    // and the per-comparison probability fires, emit `lhs OP const`
    // instead of recursing on both operands.
    if is_comparison_op(op)
        && roll_knob(g, m, KnobId::ConstComparandProb, g.cfg.const_comparand_prob)
    {
        let k = pick_comparison_operand_width(g);
        let lhs = build_cone(g, m, pool, worklist, k, depth + 1, exclude);
        return build_comparison_const_comparand(g, m, pool, op, lhs, k);
    }

    // Snapshot construction state BEFORE building operands. If the
    // operator's shape is rejected by `violates_anti_collapse` after
    // its operands are built, the newly-created operand sub-trees
    // must be rolled back — otherwise they stay in `m.nodes` with no
    // consumer (orphans). This is the α construction-rule enforcement
    // of Rule 18: a gate comes into existence only when it and its
    // operands will actually be consumed.
    let snapshot = take_construction_snapshot(m, pool, worklist);

    let operand_widths = input_widths_for(op, width, &g.cfg, &mut g.rng);
    let mut operands = Vec::with_capacity(operand_widths.len());
    for w in operand_widths {
        // DAG-sharing fork (Phase 2): with probability share_prob, terminate
        // this operand at an existing matching-width pool entry instead of
        // recursing to create fresh logic. Falls back to recursion if no
        // shareable candidate exists. Share/recurse is decided per-operand,
        // so a single gate's operands can mix shared and freshly-built sub-cones.
        let share = roll_knob(g, m, KnobId::ShareProb, g.cfg.share_prob);
        let shared = if share {
            try_share(g, pool, w, exclude)
        } else {
            None
        };
        let operand =
            shared.unwrap_or_else(|| build_cone(g, m, pool, worklist, w, depth + 1, exclude));
        operands.push(operand);
    }

    if violates_anti_collapse(op, &operands, m) {
        trace!(?op, "🔁 anti-collapse reject, rolling back operand subtree");
        rollback_construction_snapshot(m, pool, worklist, snapshot);
        return pick_terminal(g, m, pool, width, exclude);
    }

    if is_comparison_op(op) {
        debug_assert_eq!(operands.len(), 2);
        return build_comparison_gate(m, pool, op, operands[0], operands[1]);
    }

    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());

    let (node_id, is_new) = m.intern_gate(op, operands, width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

/// Allocate a flop and a `FlopQ` node. The Q is returned (and added to
/// the pool) as the leaf for the current cone. The flop's D-cone is
/// queued for later construction by `drain_flop_worklist`.
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
fn build_comb_mux(
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
fn build_case_mux_recursive(
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
fn build_casez_mux_recursive(
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
fn build_for_fold_recursive(
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

fn build_comb_mux_one_hot(
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

fn build_comb_mux_encoded(
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

fn make_case_mux(
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

fn make_casez_mux(
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

fn make_for_fold(
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

fn pick_for_fold_trip_count(g: &mut Generator, width: u32) -> Option<u32> {
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

fn pick_for_fold_kind(g: &mut Generator) -> ForFoldKind {
    match g.rng.gen_range(0..4) {
        0 => ForFoldKind::Xor,
        1 => ForFoldKind::Or,
        2 => ForFoldKind::And,
        _ => ForFoldKind::Add,
    }
}

fn build_casez_patterns(n_arms: u32) -> (u32, Vec<(u128, u128)>) {
    debug_assert!(n_arms >= 2);
    let wildcard_bits = 1;
    let sel_width = ceil_log2(n_arms) + wildcard_bits;
    let wildcard_mask = width_mask(wildcard_bits);
    let patterns = (0..n_arms)
        .map(|idx| (u128::from(idx) << wildcard_bits, wildcard_mask))
        .collect();
    (sel_width, patterns)
}

#[instrument(level = "trace", skip(g, m, pool, worklist))]
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
    let kind = if roll_knob(g, m, KnobId::FlopQFeedbackProb, g.cfg.flop_qfeedback_prob) {
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
        mux: FlopMux::None,
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

#[instrument(level = "trace", skip(g, m, pool))]
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

fn emit_terminal_constant(
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
fn pick_datas_with_dup_cap(
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
fn pick_signals_with_dup_rate(
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
pub(super) fn pick_terminal_dep_bearing(
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
pub(super) fn make_width_adapter(
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

fn pick_gate(g: &mut Generator, target_width: u32) -> GateOp {
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

fn pick_structured_gate(g: &mut Generator, target_width: u32) -> GateOp {
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

fn pick_slice_gate(g: &mut Generator, target_width: u32) -> GateOp {
    debug_assert!(target_width >= 1);
    let lo: u32 = g.rng.gen_range(0..=3);
    let hi = lo
        .checked_add(target_width - 1)
        .expect("slice hi must fit in u32");
    GateOp::Slice { hi, lo }
}

fn pick_concat_operand_widths(out_w: u32, cfg: &Config, rng: &mut impl Rng) -> Vec<u32> {
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

fn input_widths_for(op: GateOp, out_w: u32, cfg: &Config, rng: &mut impl Rng) -> Vec<u32> {
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

fn violates_anti_collapse(op: GateOp, operands: &[NodeId], m: &Module) -> bool {
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
fn has_duplicate_operand(operands: &[NodeId]) -> bool {
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
fn try_share(
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

pub(super) fn node_deps(m: &Module, id: NodeId) -> DepSet {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, FactorizationLevel, IdentityMode};
    use crate::ir::{Direction, Flop, FlopKind, MuxArm, Port, ResetKind};
    use std::collections::HashMap;

    /// Build a minimal test fixture with `n_wide` primary inputs of
    /// the given width + `n_1bit` primary inputs of 1 bit. Returns
    /// (module, pool). All primary inputs added to pool with correct deps.
    fn fixture_with_inputs(n_wide: u32, wide_w: u32, n_1bit: u32) -> (Module, SignalPool) {
        let mut m = Module::default();
        let mut pool = SignalPool::new();
        for i in 0..n_wide {
            let port_id = i;
            m.inputs.push(Port {
                id: port_id,
                name: format!("w_in{i}"),
                width: wide_w,
                dir: Direction::In,
            });
            let nid = m.nodes.len() as NodeId;
            m.nodes.push(Node::PrimaryInput {
                port: port_id,
                width: wide_w,
            });
            pool.add(nid, wide_w, DepSet::from_port(port_id));
        }
        for i in 0..n_1bit {
            let port_id = n_wide + i;
            m.inputs.push(Port {
                id: port_id,
                name: format!("b_in{i}"),
                width: 1,
                dir: Direction::In,
            });
            let nid = m.nodes.len() as NodeId;
            m.nodes.push(Node::PrimaryInput {
                port: port_id,
                width: 1,
            });
            pool.add(nid, 1, DepSet::from_port(port_id));
        }
        (m, pool)
    }

    #[test]
    fn rollback_snapshot_truncates_pool_and_prunes_stale_dedup_entries() {
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 0);
        m.identity_mode = IdentityMode::NodeId;
        m.factorization_level = FactorizationLevel::Cse;
        m.max_ast_instances = 1;
        let mut worklist = Vec::new();

        let a = 0;
        let b = 1;
        let deps = DepSet::union(&[&node_deps(&m, a), &node_deps(&m, b)]);

        let (old_const, old_const_new) = m.intern_constant(4, 1);
        assert!(old_const_new);
        pool.add(old_const, 4, DepSet::new());

        let (old_gate, old_gate_new) = m.intern_gate(GateOp::Add, vec![a, b], 4, deps.clone());
        assert!(old_gate_new);
        pool.add(old_gate, 4, deps.clone());

        let snapshot = take_construction_snapshot(&m, &pool, &worklist);

        let (new_const, new_const_new) = m.intern_constant(4, 2);
        assert!(new_const_new);
        pool.add(new_const, 4, DepSet::new());

        let (new_gate, new_gate_new) = m.intern_gate(GateOp::Xor, vec![a, b], 4, deps.clone());
        assert!(new_gate_new);
        pool.add(new_gate, 4, deps);
        worklist.push(7);

        rollback_construction_snapshot(&mut m, &mut pool, &mut worklist, snapshot);

        assert_eq!(m.nodes.len(), snapshot.nodes_len);
        assert_eq!(m.flops.len(), snapshot.flops_len);
        assert_eq!(pool.len(), snapshot.pool_len);
        assert_eq!(worklist.len(), snapshot.worklist_len);

        let add_key = (GateOp::Add, vec![a, b], 4);
        let xor_key = (GateOp::Xor, vec![a, b], 4);
        assert_eq!(m.gate_instances.get(&add_key), Some(&vec![old_gate]));
        assert!(!m.gate_instances.contains_key(&xor_key));
        assert_eq!(m.const_instances.get(&(4, 1)), Some(&vec![old_const]));
        assert!(!m.const_instances.contains_key(&(4, 2)));
    }

    /// Allocate a flop and its FlopQ node. Returns the FlopQ NodeId
    /// (which is also `flop.q`) and the flop id.
    fn alloc_flop(m: &mut Module, pool: &mut SignalPool, width: u32, kind: FlopKind) -> NodeId {
        let flop_id = m.flops.len() as FlopId;
        let q_node = m.nodes.len() as NodeId;
        m.nodes.push(Node::FlopQ {
            flop: flop_id,
            width,
        });
        m.flops.push(Flop {
            id: flop_id,
            width,
            d: None,
            q: q_node,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind,
            mux: FlopMux::None,
        });
        pool.add(q_node, width, DepSet::from_flop_virtual(flop_id));
        q_node
    }

    #[test]
    fn assemble_flop_d_one_hot_zero_default_top_is_or() {
        // 2 data inputs (width 4) + 2 sel inputs (1-bit) + 1 flop.
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 2);
        let q = alloc_flop(&mut m, &mut pool, 4, FlopKind::ZeroDefault);
        // PrimaryInput nodes: 0 (data0 w=4), 1 (data1 w=4), 2 (sel0 1b), 3 (sel1 1b).
        let arms = vec![MuxArm { data: 0, sel: 2 }, MuxArm { data: 1, sel: 3 }];
        let d = assemble_flop_d_one_hot(&mut m, &mut pool, 4, &arms, FlopKind::ZeroDefault, q);
        match &m.nodes[d as usize] {
            Node::Gate { op, width, .. } => {
                assert_eq!(
                    *op,
                    GateOp::Or,
                    "top-level of OneHot ZeroDefault should be Or"
                );
                assert_eq!(*width, 4);
            }
            other => panic!("expected Gate, got {other:?}"),
        }
    }

    #[test]
    fn assemble_flop_d_one_hot_qfeedback_includes_q_term() {
        // QFeedback adds an extra `{W{~(OR sels)}} & Q` term to the OR-reduce.
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 2);
        let q = alloc_flop(&mut m, &mut pool, 4, FlopKind::QFeedback);
        let arms = vec![MuxArm { data: 0, sel: 2 }, MuxArm { data: 1, sel: 3 }];
        let pre_len = m.nodes.len();
        let d = assemble_flop_d_one_hot(&mut m, &mut pool, 4, &arms, FlopKind::QFeedback, q);
        // Top-level is still an Or (OR-reduce over arm terms + Q-feedback term).
        match &m.nodes[d as usize] {
            Node::Gate { op, width, .. } => {
                assert_eq!(*op, GateOp::Or);
                assert_eq!(*width, 4);
            }
            other => panic!("expected Gate, got {other:?}"),
        }
        // QFeedback variant emits strictly more gates than ZeroDefault would
        // (it adds Not, OR-reduce of sels, a replicate, an And for the Q term,
        //  and an extra Or to fold Q in). Strong inequality check would be
        //  fragile; just confirm at least one Not appears (the ~(OR sels) node).
        let post_len = m.nodes.len();
        let created_slice = &m.nodes[pre_len..post_len];
        let has_not = created_slice.iter().any(|n| {
            matches!(
                n,
                Node::Gate {
                    op: GateOp::Not,
                    ..
                }
            )
        });
        assert!(
            has_not,
            "QFeedback OneHot should emit a Not for ~(OR of sels)"
        );
    }

    #[test]
    fn assemble_flop_d_encoded_zero_default_top_is_mux() {
        // 2 data (width 4) + 1 sel bus (sel_width = ceil_log2(M=2) = 1) + 1 flop.
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 1);
        let q = alloc_flop(&mut m, &mut pool, 4, FlopKind::ZeroDefault);
        // Nodes: 0=data0, 1=data1, 2=sel (1 bit), 3=Q.
        let datas = vec![0, 1];
        let d =
            assemble_flop_d_encoded(&mut m, &mut pool, 4, 2, 1, &datas, FlopKind::ZeroDefault, q);
        // Top-level of the chained ternary is a Mux.
        match &m.nodes[d as usize] {
            Node::Gate { op, width, .. } => {
                assert_eq!(*op, GateOp::Mux);
                assert_eq!(*width, 4);
            }
            other => panic!("expected Gate, got {other:?}"),
        }
    }

    #[test]
    fn assemble_flop_d_encoded_qfeedback_fallthrough_is_q() {
        // QFeedback + Encoded: index 0 is routed from Q (not from a data
        // sub-cone). Caller passes M-1 data NodeIds; assemble builds a
        // chained ternary where sel==0 picks Q and fall-through is also Q.
        let (mut m, mut pool) = fixture_with_inputs(1, 4, 1); // only 1 data + 1 sel
        let q = alloc_flop(&mut m, &mut pool, 4, FlopKind::QFeedback);
        // Nodes: 0=data1 (for index 1), 1=sel, 2=Q.
        let datas = vec![0]; // only index 1 data; index 0 becomes Q
        let d = assemble_flop_d_encoded(&mut m, &mut pool, 4, 1, 1, &datas, FlopKind::QFeedback, q);
        // Top-level is a Mux. Walking the chain we should find a Mux
        // whose false-branch operand is the Q node or a downstream one.
        match &m.nodes[d as usize] {
            Node::Gate {
                op: GateOp::Mux,
                width,
                ..
            } => {
                assert_eq!(*width, 4);
            }
            other => panic!("expected top-level Mux, got {other:?}"),
        }
    }

    #[test]
    fn ceil_log2_expected_values() {
        // Guard against 2^k >= n invariant.
        assert_eq!(ceil_log2(1), 1);
        assert_eq!(ceil_log2(2), 1);
        assert_eq!(ceil_log2(3), 2);
        assert_eq!(ceil_log2(4), 2);
        assert_eq!(ceil_log2(5), 3);
        assert_eq!(ceil_log2(8), 3);
        assert_eq!(ceil_log2(9), 4);
        for n in 2..64u32 {
            let bits = ceil_log2(n);
            assert!(
                (1u32 << bits) >= n,
                "ceil_log2({n}) = {bits}, but 2^{bits} = {} < {n}",
                1u32 << bits
            );
        }
    }

    #[test]
    fn pick_priority_encoder_n_rejects_target_widths_above_u32_domain() {
        let mut g = Generator::new(Config::default());
        assert_eq!(pick_priority_encoder_n(&mut g, 33), None);
        assert_eq!(pick_priority_encoder_n(&mut g, 128), None);
    }

    #[test]
    fn node_budget_caps_and_shrinks_module_but_stays_valid() {
        // WORKLOAD-MEMORY-SAFETY.3: a non-zero `max_nodes_per_module`
        // bounds the per-module node arena (cone construction steers to
        // existing terminals once the budget is reached — rules-first,
        // never truncating a finished cone) while the module stays
        // valid-by-construction. The default sentinel `0` is unlimited.
        let base = |seed: u64| Config {
            seed,
            // Default max_depth (6) keeps the unbounded reference safely
            // small; four outputs + no sharing make it reliably exceed
            // the tight budget below.
            min_outputs: 4,
            max_outputs: 4,
            share_prob: 0.0,
            constant_prob: 0.0,
            ..Config::default()
        };

        // Default is the unlimited sentinel.
        assert_eq!(Config::default().max_nodes_per_module, 0);

        // Unbounded reference (explicit sentinel 0 = same as default).
        let mut unb = base(123);
        unb.max_nodes_per_module = 0;
        let big = Generator::new(unb).generate_module();

        // Budgeted: identical knobs except a tight node budget.
        let budget: u32 = 48;
        let mut bnd = base(123);
        bnd.max_nodes_per_module = budget;
        let small = Generator::new(bnd).generate_module();

        // The cap has a real effect: the budgeted module is strictly
        // smaller than the unbounded one.
        assert!(
            small.nodes.len() < big.nodes.len(),
            "budget must shrink the module: {} !< {}",
            small.nodes.len(),
            big.nodes.len()
        );
        // And it is genuinely bounded (soft ceiling: a bounded number of
        // terminal/adapter nodes may close already-open frames past the
        // budget, hence the generous slack rather than an exact equality).
        assert!(
            small.nodes.len() <= budget as usize * 6,
            "budget must keep the module bounded: {} > {}",
            small.nodes.len(),
            budget * 6
        );

        // Both remain valid-by-construction.
        crate::ir::validate::validate(&small).expect("budgeted module must be valid");
        crate::ir::validate::validate(&big).expect("unbounded module must be valid");
    }

    fn make_generator(flop_prob: f64) -> Generator {
        let cfg = Config {
            seed: 42,
            flop_prob,
            ..Config::default()
        };
        Generator::new(cfg)
    }

    #[test]
    fn pick_gate_exercises_all_live_category_ops() {
        use std::collections::HashSet;

        fn collect_ops(cfg: Config, target_width: u32, draws: usize) -> HashSet<GateOp> {
            let mut g = Generator::new(cfg);
            let mut out = HashSet::new();
            for _ in 0..draws {
                out.insert(pick_gate(&mut g, target_width));
            }
            out
        }

        let category_cfg = |seed: u64| Config {
            seed,
            gate_bitwise_weight: 0,
            gate_arith_weight: 0,
            gate_struct_weight: 0,
            gate_compare_weight: 0,
            gate_reduce_weight: 0,
            gate_shift_weight: 0,
            ..Config::default()
        };

        let bitwise = collect_ops(
            Config {
                gate_bitwise_weight: 1,
                ..category_cfg(1)
            },
            4,
            512,
        );
        assert_eq!(
            bitwise,
            HashSet::from([GateOp::And, GateOp::Or, GateOp::Xor, GateOp::Not])
        );

        let arith = collect_ops(
            Config {
                gate_arith_weight: 1,
                ..category_cfg(2)
            },
            4,
            512,
        );
        assert_eq!(
            arith,
            HashSet::from([GateOp::Add, GateOp::Sub, GateOp::Mul])
        );

        let structured = collect_ops(
            Config {
                gate_struct_weight: 1,
                ..category_cfg(3)
            },
            4,
            128,
        );
        assert!(structured.contains(&GateOp::Mux));
        assert!(structured.contains(&GateOp::Concat));
        assert!(
            structured
                .iter()
                .any(|op| matches!(op, GateOp::Slice { .. })),
            "structured bucket must include selectable Slice"
        );

        let compare = collect_ops(
            Config {
                gate_compare_weight: 1,
                ..category_cfg(4)
            },
            1,
            1024,
        );
        assert_eq!(
            compare,
            HashSet::from([
                GateOp::Eq,
                GateOp::Neq,
                GateOp::Lt,
                GateOp::Gt,
                GateOp::Le,
                GateOp::Ge,
            ])
        );

        let reduce = collect_ops(
            Config {
                gate_reduce_weight: 1,
                ..category_cfg(5)
            },
            1,
            512,
        );
        assert_eq!(
            reduce,
            HashSet::from([GateOp::RedAnd, GateOp::RedOr, GateOp::RedXor])
        );

        let shifts = collect_ops(
            Config {
                gate_shift_weight: 1,
                ..category_cfg(6)
            },
            4,
            256,
        );
        assert_eq!(shifts, HashSet::from([GateOp::Shl, GateOp::Shr]));
    }

    #[test]
    fn selectable_slice_gate_never_degenerates_to_identity_shape() {
        let cfg = Config::default();
        let mut g = Generator::new(cfg.clone());
        for _ in 0..128 {
            let op = pick_slice_gate(&mut g, 8);
            let widths = input_widths_for(op, 8, &cfg, &mut g.rng);
            match op {
                GateOp::Slice { hi, lo } => {
                    assert_eq!(hi - lo + 1, 8);
                    assert_eq!(widths.len(), 1);
                    assert!(
                        widths[0] > hi,
                        "selectable Slice must have a wider source than its high bit"
                    );
                }
                other => panic!("expected Slice, got {other:?}"),
            }
        }
    }

    #[test]
    fn selectable_concat_widths_partition_output_width() {
        let cfg = Config::default();
        let mut g = Generator::new(cfg.clone());
        for _ in 0..128 {
            let widths = pick_concat_operand_widths(8, &cfg, &mut g.rng);
            assert!(
                widths.len() >= 2,
                "selectable Concat must have at least 2 operands"
            );
            assert_eq!(widths.iter().sum::<u32>(), 8);
            assert!(
                widths.iter().all(|w| *w >= 1),
                "every Concat operand width must be positive"
            );
        }
    }

    #[test]
    fn pick_mux_arm_count_never_returns_one() {
        let mut g = make_generator(0.0);
        for _ in 0..10_000 {
            let m = pick_mux_arm_count(&mut g);
            assert_ne!(m, 1, "pick_mux_arm_count must never return 1");
            assert!(m == 0 || (2..=g.cfg.max_mux_arms).contains(&m));
        }
    }

    fn scaffold_module_with_input(width: u32) -> (Module, SignalPool, NodeId, DepSet) {
        let mut m = Module::default();
        m.inputs.push(crate::ir::Port {
            id: 0,
            name: "a".into(),
            width,
            dir: crate::ir::Direction::In,
        });
        let node_id = 0;
        m.nodes.push(Node::PrimaryInput { port: 0, width });
        let deps = DepSet::from_port(0);
        let mut pool = SignalPool::new();
        pool.add(node_id, width, deps.clone());
        (m, pool, node_id, deps)
    }

    #[test]
    fn width_adapter_identity() {
        let (mut m, mut pool, src, deps) = scaffold_module_with_input(8);
        let out = make_width_adapter(&mut m, &mut pool, src, 8, deps, 8);
        assert_eq!(out, src, "src==target must be a passthrough");
        assert_eq!(m.nodes.len(), 1, "no nodes should be added on identity");
    }

    #[test]
    fn pick_terminal_reuse_knob_controls_exact_width_leaf_choice() {
        let cfg_reuse = Config {
            seed: 7,
            terminal_reuse_prob: 1.0,
            ..Config::default()
        };
        let mut g_reuse = Generator::new(cfg_reuse);
        let (mut m_reuse, mut pool_reuse, src_reuse, _) = scaffold_module_with_input(4);
        let picked_reuse = pick_terminal(&mut g_reuse, &mut m_reuse, &mut pool_reuse, 4, None);
        assert_eq!(
            picked_reuse, src_reuse,
            "terminal_reuse_prob=1.0 must reuse the matching-width pool signal"
        );

        let cfg_fresh = Config {
            seed: 7,
            terminal_reuse_prob: 0.0,
            ..Config::default()
        };
        let mut g_fresh = Generator::new(cfg_fresh);
        let (mut m_fresh, mut pool_fresh, src_fresh, _) = scaffold_module_with_input(4);
        let picked_fresh = pick_terminal(&mut g_fresh, &mut m_fresh, &mut pool_fresh, 4, None);
        assert_ne!(
            picked_fresh, src_fresh,
            "terminal_reuse_prob=0.0 must not reuse the matching-width pool signal"
        );
        assert!(
            matches!(m_fresh.nodes[picked_fresh as usize], Node::Constant { .. }),
            "terminal_reuse_prob=0.0 fallback should emit a constant leaf"
        );
    }

    #[test]
    fn pick_terminal_constant_prob_controls_width_adapter_fallback() {
        let cfg_adapter = Config {
            seed: 11,
            constant_prob: 0.0,
            ..Config::default()
        };
        let mut g_adapter = Generator::new(cfg_adapter);
        let (mut m_adapter, mut pool_adapter, _, _) = scaffold_module_with_input(8);
        let picked_adapter =
            pick_terminal(&mut g_adapter, &mut m_adapter, &mut pool_adapter, 4, None);
        assert!(
            matches!(
                m_adapter.nodes[picked_adapter as usize],
                Node::Gate {
                    op: GateOp::Slice { .. },
                    ..
                }
            ),
            "constant_prob=0.0 must use a width-adapter when no matching-width signal exists"
        );

        let cfg_constant = Config {
            seed: 11,
            constant_prob: 1.0,
            ..Config::default()
        };
        let mut g_constant = Generator::new(cfg_constant);
        let (mut m_constant, mut pool_constant, _, _) = scaffold_module_with_input(8);
        let picked_constant = pick_terminal(
            &mut g_constant,
            &mut m_constant,
            &mut pool_constant,
            4,
            None,
        );
        assert!(
            matches!(
                m_constant.nodes[picked_constant as usize],
                Node::Constant { .. }
            ),
            "constant_prob=1.0 must emit a constant when width-adapter fallback is available"
        );
    }

    #[test]
    fn width_adapter_slice_shrinks() {
        let (mut m, mut pool, src, deps) = scaffold_module_with_input(16);
        let out = make_width_adapter(&mut m, &mut pool, src, 16, deps, 8);
        assert_ne!(out, src);
        match &m.nodes[out as usize] {
            Node::Gate {
                op: GateOp::Slice { hi, lo },
                operands,
                width,
                ..
            } => {
                assert_eq!(*hi, 7);
                assert_eq!(*lo, 0);
                assert_eq!(*width, 8);
                assert_eq!(operands, &vec![src]);
            }
            other => panic!("expected Slice, got {other:?}"),
        }
    }

    #[test]
    fn width_adapter_concat_expands_exact_multiple() {
        let (mut m, mut pool, src, deps) = scaffold_module_with_input(4);
        // target = 16 = 4 * 4, so a single Concat with 4 copies suffices.
        let out = make_width_adapter(&mut m, &mut pool, src, 4, deps, 16);
        match &m.nodes[out as usize] {
            Node::Gate {
                op: GateOp::Concat,
                operands,
                width,
                ..
            } => {
                assert_eq!(*width, 16);
                assert_eq!(operands.len(), 4);
                assert!(operands.iter().all(|&id| id == src));
            }
            other => panic!("expected Concat, got {other:?}"),
        }
    }

    #[test]
    fn comb_mux_block_produces_valid_output() {
        // Force every non-leaf recursion point into a comb-mux block.
        // Verify it still produces an IR-valid module (width rules
        // correct, no trivial outputs, etc.) across a seed sweep.
        let base = Config {
            comb_mux_prob: 1.0,
            flop_prob: 0.0,
            share_prob: 0.0,
            max_depth: 3,
            min_inputs: 3,
            max_inputs: 3,
            min_outputs: 2,
            max_outputs: 2,
            min_width: 4,
            max_width: 8,
            min_mux_arms: 2,
            max_mux_arms: 3,
            ..Config::default()
        };
        for seed in 0..10u64 {
            for enc_prob in [0.0, 1.0] {
                let cfg = Config {
                    seed,
                    comb_mux_encoding_prob: enc_prob,
                    ..base.clone()
                };
                let mut gen = Generator::new(cfg);
                let m = gen.generate_module();
                crate::ir::validate::validate(&m).unwrap_or_else(|e| {
                    panic!("seed {seed} enc={enc_prob} comb-mux: validation failed: {e}")
                });
            }
        }
    }

    #[test]
    fn share_prob_high_shares_internal_gates() {
        // With high share_prob the non-leaf sharing path fires. Statistically,
        // across a handful of seeds, at least one run must show an internal
        // *Gate* (not a primary input) being consumed as an operand by 2+
        // other gates. This is the Phase 2 DAG-cone mechanism working —
        // without it, an internal gate has exactly one consumer (its parent).
        //
        // The test sweeps seeds rather than asserting on one, because
        // `try_share` picks uniformly over pool entries (which include
        // primary inputs and adapter nodes) and may not hit a mid-tree
        // gate on a given seed. Over a small sweep it reliably does.
        let base = Config {
            share_prob: 0.9,
            flop_prob: 0.0,
            max_depth: 6,
            min_inputs: 4,
            max_inputs: 4,
            min_outputs: 4,
            max_outputs: 4,
            // Same width everywhere so the pool has many matching candidates.
            min_width: 4,
            max_width: 4,
            ..Config::default()
        };
        let found_gate_sharing = (0..32u64).any(|seed| {
            let cfg = Config {
                seed,
                ..base.clone()
            };
            let mut gen = Generator::new(cfg);
            let m = gen.generate_module();
            let fanout = count_gate_fanout(&m);
            m.nodes
                .iter()
                .enumerate()
                .any(|(idx, n)| matches!(n, Node::Gate { .. }) && fanout[idx] >= 2)
        });
        assert!(
            found_gate_sharing,
            "high share_prob must produce at least one Gate with fanout >= 2 \
             across a 32-seed sweep (internal-gate sharing is the DAG-cone mechanism)"
        );
    }

    /// For each node index, how many other gates reference it as an operand.
    fn count_gate_fanout(m: &Module) -> Vec<u32> {
        let mut fanout = vec![0u32; m.nodes.len()];
        for node in &m.nodes {
            if let Node::Gate { operands, .. } = node {
                for &op in operands {
                    fanout[op as usize] += 1;
                }
            }
        }
        fanout
    }

    #[test]
    fn width_adapter_concat_expands_non_multiple_exactly() {
        let (mut m, mut pool, src, deps) = scaffold_module_with_input(3);
        // target = 8 = 3 + 3 + 2, so the adapter should build the
        // exact-width shape `{src[1:0], src, src}` with no oversized
        // intermediate Concat and no outer Slice.
        let out = make_width_adapter(&mut m, &mut pool, src, 3, deps, 8);
        match &m.nodes[out as usize] {
            Node::Gate {
                op: GateOp::Concat,
                operands,
                width,
                ..
            } => {
                assert_eq!(*width, 8);
                assert_eq!(operands.len(), 3);
                assert_eq!(operands[1], src);
                assert_eq!(operands[2], src);
                match &m.nodes[operands[0] as usize] {
                    Node::Gate {
                        op: GateOp::Slice { hi, lo },
                        operands: slice_ops,
                        width: slice_width,
                        ..
                    } => {
                        assert_eq!(*hi, 1);
                        assert_eq!(*lo, 0);
                        assert_eq!(*slice_width, 2);
                        assert_eq!(slice_ops, &vec![src]);
                    }
                    other => panic!("expected leading remainder Slice, got {other:?}"),
                }
            }
            other => panic!("expected exact-width Concat, got {other:?}"),
        }
        assert!(
            !m.nodes.iter().any(|n| matches!(
                n,
                Node::Gate {
                    op: GateOp::Slice { hi: 7, lo: 0 },
                    width: 8,
                    ..
                }
            )),
            "non-multiple expansion should not build an outer Slice"
        );
    }

    /// `violates_anti_collapse` must catch N-arity duplicates on
    /// idempotent / self-inverse operators, not just pairwise 2-arity
    /// cases. Regression guard for the `i_2 ^ i_2 ^ i_2 ^ i_2 = 0`
    /// defect observed in sample output.
    #[test]
    fn anti_collapse_catches_nary_duplicates() {
        use GateOp::*;
        let m = Module::default();
        // Xor/And/Or with any duplicate operand at any arity.
        for op in [Xor, And, Or] {
            assert!(
                violates_anti_collapse(op, &[7, 7, 7, 7], &m),
                "{op:?}: 4-repeat not caught"
            );
            assert!(
                violates_anti_collapse(op, &[1, 2, 1], &m),
                "{op:?}: 3-arity with duplicate not caught"
            );
            assert!(
                violates_anti_collapse(op, &[3, 4, 5, 3], &m),
                "{op:?}: 4-arity with single duplicate not caught"
            );
            assert!(
                !violates_anti_collapse(op, &[1, 2, 3, 4], &m),
                "{op:?}: all-distinct flagged falsely"
            );
        }
        // Add / Mul: under default `operand_duplication_rate = 0.0`
        // (module default from `Module::default`), duplicates ARE
        // flagged. User opts in by raising the rate toward 1.0 to
        // allow `x + x = 2x` / `x * x = x²` shapes.
        for op in [Add, Mul] {
            assert!(
                violates_anti_collapse(op, &[1, 1, 1], &m),
                "{op:?}: duplicates must be flagged at default rate 0.0"
            );
        }
        // With the knob at 1.0 the flag is disabled: duplicates pass.
        let m_relaxed = Module {
            operand_duplication_rate: 1.0,
            ..Module::default()
        };
        for op in [Add, Mul] {
            assert!(
                !violates_anti_collapse(op, &[1, 1, 1], &m_relaxed),
                "{op:?}: duplicates must pass at rate 1.0"
            );
        }
    }

    /// `pick_terminal_dep_bearing` must never return a dep-empty node
    /// (a constant). Regression guard for the `2'h2 == 2'h2` constant-
    /// select defect observed in sample output.
    #[test]
    fn pick_terminal_dep_bearing_rejects_constants() {
        let cfg = Config {
            seed: 0xDEADBEEF,
            ..Config::default()
        };
        let mut g = Generator::new(cfg);
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 0);
        // Pollute the pool with a dep-empty constant of the target width.
        let const_id = make_constant(&mut m, &mut pool, 4, 5);
        for _ in 0..100 {
            let picked = pick_terminal_dep_bearing(&mut g, &mut m, &mut pool, 4, None);
            assert_ne!(
                picked, const_id,
                "dep-bearing picker returned the dep-empty constant"
            );
            assert!(
                !node_deps(&m, picked).is_empty(),
                "dep-bearing picker returned a node with empty deps"
            );
        }
    }

    /// `pick_coefficient` must never return a value that overflows the
    /// target operand width, even when `max_coefficient` is larger than
    /// `2^width - 1`. Regression guard against the `1'h6` bug observed
    /// in sample output.
    #[test]
    fn pick_coefficient_respects_target_width() {
        let cfg = Config {
            seed: 0xC0FFEE,
            min_coefficient: 1,
            max_coefficient: 15,
            ..Config::default()
        };
        let mut g = Generator::new(cfg);
        for _ in 0..200 {
            let c1 = pick_coefficient(&mut g, 1);
            assert_eq!(c1, 1, "width=1: only legal coefficient is 1, got {c1}");
            let c2 = pick_coefficient(&mut g, 2);
            assert!(
                (1..=3).contains(&c2),
                "width=2: coef must be in [1,3], got {c2}"
            );
            let c4 = pick_coefficient(&mut g, 4);
            assert!(
                (1..=15).contains(&c4),
                "width=4: coef must be in [1,15], got {c4}"
            );
            let c8 = pick_coefficient(&mut g, 8);
            assert!(
                (1..=15).contains(&c8),
                "width=8: coef bounded by max_coefficient=15, got {c8}"
            );
        }
    }

    #[test]
    fn make_nary_add_dedups_duplicate_terms_at_strict_rate() {
        let (mut m, mut pool) = fixture_with_inputs(0, 8, 0);
        m.identity_mode = IdentityMode::NodeId;
        m.factorization_level = FactorizationLevel::OperandUnique;
        m.operand_duplication_rate = 0.0;

        let a = make_constant(&mut m, &mut pool, 8, 3);
        let b = make_constant(&mut m, &mut pool, 8, 5);
        let sum = make_nary_add(&mut m, &mut pool, &[a, b, a], 8);

        match &m.nodes[sum as usize] {
            Node::Gate {
                op: GateOp::Add,
                operands,
                width,
                ..
            } => {
                assert_eq!(*width, 8);
                assert_eq!(operands, &vec![a, b]);
            }
            other => panic!("expected Add gate, got {other:?}"),
        }
    }

    #[test]
    fn comparison_range_fold_rejects_gt_all_ones_even_without_peephole() {
        let (mut m, mut pool) = fixture_with_inputs(1, 2, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let max = make_constant(&mut m, &mut pool, 2, 0b11);
        let cmp = build_comparison_gate(&mut m, &mut pool, GateOp::Gt, x, max);
        assert!(
            matches!(
                &m.nodes[cmp as usize],
                Node::Constant { width: 1, value: 0 }
            ),
            "x > all-ones must fold to 1'b0 even below peephole"
        );
    }

    #[test]
    fn comparison_range_fold_proves_overshift_rhs_is_zero() {
        let (mut m, mut pool) = fixture_with_inputs(1, 7, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let huge_shift = make_constant(&mut m, &mut pool, 8, 0xD5);
        let deps = node_deps(&m, x);
        let (rhs, is_new) = m.intern_gate(GateOp::Shl, vec![x, huge_shift], 7, deps.clone());
        if is_new {
            pool.add(rhs, 7, deps);
        }
        let cmp = build_comparison_gate(&mut m, &mut pool, GateOp::Ge, x, rhs);
        assert!(
            matches!(
                &m.nodes[cmp as usize],
                Node::Constant { width: 1, value: 1 }
            ),
            "x >= (y << huge_const) must fold when the shift is provably zero"
        );
    }

    #[test]
    fn comparison_range_fold_keeps_overlapping_ranges_live() {
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 0);
        m.factorization_level = FactorizationLevel::None;
        let cmp = build_comparison_gate(&mut m, &mut pool, GateOp::Lt, 0, 1);
        assert!(
            matches!(
                &m.nodes[cmp as usize],
                Node::Gate {
                    op: GateOp::Lt,
                    width: 1,
                    ..
                }
            ),
            "independent 4-bit inputs have overlapping ranges; comparison must stay live"
        );
    }

    #[test]
    fn comparison_range_fold_tracks_replicated_concat_correlation() {
        let (mut m, mut pool) = fixture_with_inputs(0, 1, 2);
        m.factorization_level = FactorizationLevel::None;
        let bit = 0;
        let sel = 1;
        let concat_deps = node_deps(&m, bit);
        let (replicated, concat_is_new) =
            m.intern_gate(GateOp::Concat, vec![bit; 5], 5, concat_deps.clone());
        if concat_is_new {
            pool.add(replicated, 5, concat_deps);
        }
        let lo = make_constant(&mut m, &mut pool, 5, 0x02);
        let hi = make_constant(&mut m, &mut pool, 5, 0x12);
        let mux_deps =
            DepSet::union(&[&node_deps(&m, sel), &node_deps(&m, lo), &node_deps(&m, hi)]);
        let (masked_mux, mux_is_new) =
            m.intern_gate(GateOp::Mux, vec![sel, hi, lo], 5, mux_deps.clone());
        if mux_is_new {
            pool.add(masked_mux, 5, mux_deps);
        }
        let c0d = make_constant(&mut m, &mut pool, 5, 0x0d);
        let c1a = make_constant(&mut m, &mut pool, 5, 0x1a);
        let and_deps = DepSet::union(&[
            &node_deps(&m, c0d),
            &node_deps(&m, replicated),
            &node_deps(&m, masked_mux),
            &node_deps(&m, c1a),
        ]);
        let (masked, and_is_new) = m.intern_gate(
            GateOp::And,
            vec![c0d, replicated, masked_mux, c1a],
            5,
            and_deps.clone(),
        );
        if and_is_new {
            pool.add(masked, 5, and_deps);
        }
        let zero = make_constant(&mut m, &mut pool, 5, 0);
        let cmp = build_comparison_gate(&mut m, &mut pool, GateOp::Gt, masked, zero);
        assert!(
            matches!(
                &m.nodes[cmp as usize],
                Node::Constant { width: 1, value: 0 }
            ),
            "replicated concat bits must stay correlated; the masked value is always zero"
        );
    }

    #[test]
    fn comparison_range_fold_proves_reflexive_slice_tautologies() {
        let (mut m, mut pool) = fixture_with_inputs(1, 16, 0);
        m.factorization_level = FactorizationLevel::None;
        let wide = 0;
        let deps = node_deps(&m, wide);
        let (slice, is_new) =
            m.intern_gate(GateOp::Slice { hi: 5, lo: 0 }, vec![wide], 6, deps.clone());
        if is_new {
            pool.add(slice, 6, deps);
        }

        let le = build_comparison_gate(&mut m, &mut pool, GateOp::Le, slice, slice);
        assert!(
            matches!(
                &m.nodes[le as usize],
                Node::Constant { width: 1, value: 1 }
            ),
            "x <= x must fold to 1'b1 even when the semantic support is too wide for exact enumeration"
        );

        let lt = build_comparison_gate(&mut m, &mut pool, GateOp::Lt, slice, slice);
        assert!(
            matches!(&m.nodes[lt as usize], Node::Constant { width: 1, value: 0 }),
            "x < x must fold to 1'b0 for the same wide correlated operand"
        );
    }

    #[test]
    fn small_value_set_tracks_duplicate_xor_parity() {
        let (mut m, mut pool) = fixture_with_inputs(1, 5, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let deps = node_deps(&m, x);
        let (dup_xor, is_new) = m.intern_gate(GateOp::Xor, vec![x, x], 5, deps.clone());
        if is_new {
            pool.add(dup_xor, 5, deps);
        }

        let mut memo = SmallValueSetContext::default();
        assert_eq!(
            node_small_value_set(&m, dup_xor, &mut memo),
            Some(vec![0]),
            "x ^ x must collapse to the singleton set {{0}} in exact finite-set reasoning"
        );
    }

    #[test]
    fn comparison_range_fold_proves_ge_against_duplicate_xor_zero() {
        let (mut m, mut pool) = fixture_with_inputs(1, 5, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let deps = node_deps(&m, x);
        let (dup_xor, is_new) = m.intern_gate(GateOp::Xor, vec![x, x], 5, deps.clone());
        if is_new {
            pool.add(dup_xor, 5, deps);
        }

        let ge = build_comparison_gate(&mut m, &mut pool, GateOp::Ge, x, dup_xor);
        assert!(
            matches!(&m.nodes[ge as usize], Node::Constant { width: 1, value: 1 }),
            "x >= (x ^ x) must fold to 1'b1 because the rhs is provably zero"
        );
    }

    #[test]
    fn comparison_range_fold_proves_lt_against_reflexive_sub_zero() {
        let (mut m, mut pool) = fixture_with_inputs(2, 6, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let y = 1;

        let mul_deps = DepSet::union(&[&node_deps(&m, x), &node_deps(&m, y), &node_deps(&m, x)]);
        let (mul, mul_is_new) = m.intern_gate(GateOp::Mul, vec![x, y, x], 6, mul_deps.clone());
        if mul_is_new {
            pool.add(mul, 6, mul_deps);
        }

        let sub_deps = node_deps(&m, mul);
        let (zero_rhs, sub_is_new) =
            m.intern_gate(GateOp::Sub, vec![mul, mul], 6, sub_deps.clone());
        if sub_is_new {
            pool.add(zero_rhs, 6, sub_deps);
        }

        let sum_deps = DepSet::union(&[&node_deps(&m, x), &node_deps(&m, y)]);
        let (lhs, add_is_new) = m.intern_gate(GateOp::Add, vec![x, y], 6, sum_deps.clone());
        if add_is_new {
            pool.add(lhs, 6, sum_deps);
        }

        let lt = build_comparison_gate(&mut m, &mut pool, GateOp::Lt, lhs, zero_rhs);
        assert!(
            matches!(&m.nodes[lt as usize], Node::Constant { width: 1, value: 0 }),
            "unsigned lhs < (x - x) must fold to 1'b0 even when x itself is not exact"
        );
    }

    #[test]
    fn small_value_set_short_circuits_or_all_ones_prefix_over_wide_tail() {
        let (mut m, mut pool) = fixture_with_inputs(1, 16, 0);
        m.factorization_level = FactorizationLevel::None;
        let wide = 0;
        let wide_deps = node_deps(&m, wide);
        let (tail, tail_is_new) = m.intern_gate(
            GateOp::Slice { hi: 5, lo: 0 },
            vec![wide],
            6,
            wide_deps.clone(),
        );
        if tail_is_new {
            pool.add(tail, 6, wide_deps);
        }

        let c16 = make_constant(&mut m, &mut pool, 6, 0x16);
        let c39 = make_constant(&mut m, &mut pool, 6, 0x39);
        let deps = DepSet::union(&[
            &node_deps(&m, c16),
            &node_deps(&m, c39),
            &node_deps(&m, tail),
        ]);
        let (or, is_new) = m.intern_gate(GateOp::Or, vec![c16, c39, tail], 6, deps.clone());
        if is_new {
            pool.add(or, 6, deps);
        }

        let mut memo = SmallValueSetContext::default();
        assert_eq!(
            node_small_value_set(&m, or, &mut memo),
            Some(vec![0x3f]),
            "22 | 57 already saturates all six bits, so the wide-dependent tail cannot change the result"
        );
        assert_eq!(prove_node_exact_value(&m, or), Some(0x3f));
    }

    #[test]
    fn small_value_set_short_circuits_mul_zero_prefix_over_wide_tail() {
        let (mut m, mut pool) = fixture_with_inputs(1, 16, 0);
        m.factorization_level = FactorizationLevel::None;
        let wide = 0;
        let wide_deps = node_deps(&m, wide);
        let (tail, tail_is_new) = m.intern_gate(
            GateOp::Slice { hi: 1, lo: 0 },
            vec![wide],
            2,
            wide_deps.clone(),
        );
        if tail_is_new {
            pool.add(tail, 2, wide_deps);
        }

        let one = make_constant(&mut m, &mut pool, 2, 0x1);
        let two_a = make_constant(&mut m, &mut pool, 2, 0x2);
        let two_b = make_constant(&mut m, &mut pool, 2, 0x2);
        let deps = DepSet::union(&[
            &node_deps(&m, one),
            &node_deps(&m, two_a),
            &node_deps(&m, two_b),
            &node_deps(&m, tail),
        ]);
        let (mul, is_new) =
            m.intern_gate(GateOp::Mul, vec![one, two_a, two_b, tail], 2, deps.clone());
        if is_new {
            pool.add(mul, 2, deps);
        }

        let mut memo = SmallValueSetContext::default();
        assert_eq!(
            node_small_value_set(&m, mul, &mut memo),
            Some(vec![0]),
            "at width 2, 1 * 2 * 2 already wraps to zero, so the wide-dependent tail cannot revive the product"
        );
        assert_eq!(prove_node_exact_value(&m, mul), Some(0));
    }

    #[test]
    fn small_value_set_bails_out_before_cartesian_blow_up() {
        let (mut m, mut pool) = fixture_with_inputs(5, 8, 0);
        m.factorization_level = FactorizationLevel::None;
        let deps = DepSet::union(&[
            &node_deps(&m, 0),
            &node_deps(&m, 1),
            &node_deps(&m, 2),
            &node_deps(&m, 3),
            &node_deps(&m, 4),
        ]);
        let (sum, is_new) = m.intern_gate(GateOp::Add, vec![0, 1, 2, 3, 4], 8, deps.clone());
        if is_new {
            pool.add(sum, 8, deps);
        }

        let mut memo = SmallValueSetContext::default();
        assert_eq!(
            node_small_value_set(&m, sum, &mut memo),
            None,
            "budgeted exact finite-set reasoning should bail out instead of enumerating an unbounded cartesian product"
        );
    }

    #[test]
    fn small_value_set_skips_wide_support_cones() {
        let (mut m, mut pool) = fixture_with_inputs(4, 1, 0);
        m.factorization_level = FactorizationLevel::None;
        let deps = DepSet::union(&[
            &node_deps(&m, 0),
            &node_deps(&m, 1),
            &node_deps(&m, 2),
            &node_deps(&m, 3),
        ]);
        let (or, is_new) = m.intern_gate(GateOp::Or, vec![0, 1, 2, 3], 1, deps.clone());
        if is_new {
            pool.add(or, 1, deps);
        }

        let mut memo = SmallValueSetContext::default();
        assert_eq!(
            node_small_value_set(&m, or, &mut memo),
            None,
            "exact finite-set reasoning should stay reserved for small-support cones"
        );
        assert_eq!(prove_node_exact_value(&m, or), None);
    }

    #[test]
    fn prove_node_exact_value_detects_dynamic_overshift_zero() {
        let (mut m, mut pool) = fixture_with_inputs(1, 8, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;
        let deps = node_deps(&m, x);

        let c26 = make_constant(&mut m, &mut pool, 8, 0x26);
        let ceb = make_constant(&mut m, &mut pool, 8, 0xeb);
        let or_deps = DepSet::union(&[
            &node_deps(&m, x),
            &node_deps(&m, c26),
            &node_deps(&m, x),
            &node_deps(&m, ceb),
        ]);
        let (or, or_is_new) = m.intern_gate(GateOp::Or, vec![x, c26, x, ceb], 8, or_deps.clone());
        if or_is_new {
            pool.add(or, 8, or_deps);
        }

        let one = make_constant(&mut m, &mut pool, 1, 1);
        let (shl, shl_is_new) = m.intern_gate(GateOp::Shl, vec![or, one], 8, deps.clone());
        if shl_is_new {
            pool.add(shl, 8, deps.clone());
        }

        let five = make_constant(&mut m, &mut pool, 8, 5);
        let (rhs, rhs_is_new) = m.intern_gate(GateOp::Sub, vec![shl, five], 8, deps.clone());
        if rhs_is_new {
            pool.add(rhs, 8, deps.clone());
        }

        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![shl, rhs], 8, deps.clone());
        if shr_is_new {
            pool.add(shr, 8, deps);
        }

        assert_eq!(
            prove_node_exact_value(&m, shr),
            Some(0),
            "when rhs is derived from a left-shifted value minus a small constant, the shift can still be provably overshifted"
        );
    }

    #[test]
    fn prove_node_exact_value_detects_reduction_zero_from_dynamic_single_bit_shr() {
        let (mut m, mut pool) = fixture_with_inputs(1, 8, 0);
        m.factorization_level = FactorizationLevel::None;
        let rhs = 0;

        let one = make_constant(&mut m, &mut pool, 4, 1);
        let shr_deps = DepSet::union(&[&node_deps(&m, one), &node_deps(&m, rhs)]);
        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![one, rhs], 4, shr_deps.clone());
        if shr_is_new {
            pool.add(shr, 4, shr_deps);
        }

        let red_and_deps = node_deps(&m, shr);
        let (red_and, red_and_is_new) =
            m.intern_gate(GateOp::RedAnd, vec![shr], 1, red_and_deps.clone());
        if red_and_is_new {
            pool.add(red_and, 1, red_and_deps);
        }

        assert_eq!(
            prove_node_exact_value(&m, red_and),
            Some(0),
            "reduction-AND of `1 >> dynamic_rhs` must fold to zero because the shifted source can never become all ones"
        );
    }

    #[test]
    fn prove_node_exact_value_detects_dynamic_overshift_zero_through_wide_slice() {
        let (mut m, mut pool) = fixture_with_inputs(1, 9, 0);
        m.factorization_level = FactorizationLevel::None;
        let wide = 0;
        let wide_deps = node_deps(&m, wide);
        let (slice, slice_is_new) = m.intern_gate(
            GateOp::Slice { hi: 7, lo: 0 },
            vec![wide],
            8,
            wide_deps.clone(),
        );
        if slice_is_new {
            pool.add(slice, 8, wide_deps.clone());
        }

        let c26 = make_constant(&mut m, &mut pool, 8, 0x26);
        let ceb = make_constant(&mut m, &mut pool, 8, 0xeb);
        let or_deps = DepSet::union(&[
            &node_deps(&m, slice),
            &node_deps(&m, c26),
            &node_deps(&m, slice),
            &node_deps(&m, ceb),
        ]);
        let (or, or_is_new) =
            m.intern_gate(GateOp::Or, vec![slice, c26, slice, ceb], 8, or_deps.clone());
        if or_is_new {
            pool.add(or, 8, or_deps);
        }

        let one = make_constant(&mut m, &mut pool, 1, 1);
        let (shl, shl_is_new) = m.intern_gate(GateOp::Shl, vec![or, one], 8, wide_deps.clone());
        if shl_is_new {
            pool.add(shl, 8, wide_deps.clone());
        }

        let five = make_constant(&mut m, &mut pool, 8, 5);
        let (rhs, rhs_is_new) = m.intern_gate(GateOp::Sub, vec![shl, five], 8, wide_deps.clone());
        if rhs_is_new {
            pool.add(rhs, 8, wide_deps.clone());
        }

        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![shl, rhs], 8, wide_deps);
        if shr_is_new {
            pool.add(shr, 8, node_deps(&m, shr));
        }

        assert_eq!(
            prove_node_exact_value(&m, shr),
            Some(0),
            "narrow slices of wider cones must still participate in the exact-value proof that detects dynamic overshifts"
        );
    }

    #[test]
    fn prove_node_exact_value_detects_overshift_from_wrapped_small_rhs_set() {
        let (mut m, mut pool) = fixture_with_inputs(4, 8, 0);
        m.factorization_level = FactorizationLevel::None;

        let a = 0;
        let b = 1;
        let c = 2;
        let d = 3;

        let lhs_deps = DepSet::union(&[
            &node_deps(&m, a),
            &node_deps(&m, b),
            &node_deps(&m, c),
            &node_deps(&m, d),
        ]);
        let (lhs, lhs_is_new) = m.intern_gate(GateOp::Or, vec![a, b, c, d], 8, lhs_deps.clone());
        if lhs_is_new {
            pool.add(lhs, 8, lhs_deps.clone());
        }

        let one = make_constant(&mut m, &mut pool, 8, 1);
        let (bit, bit_is_new) = m.intern_gate(GateOp::Eq, vec![lhs, one], 1, lhs_deps.clone());
        if bit_is_new {
            pool.add(bit, 1, lhs_deps.clone());
        }

        let rhs_deps = node_deps(&m, bit);
        let (replicate, replicate_is_new) = m.intern_gate(
            GateOp::Concat,
            vec![bit, bit, bit, bit, bit, bit, bit, bit],
            8,
            rhs_deps.clone(),
        );
        if replicate_is_new {
            pool.add(replicate, 8, rhs_deps.clone());
        }

        let cd4 = make_constant(&mut m, &mut pool, 8, 0xd4);
        let (rhs, rhs_is_new) =
            m.intern_gate(GateOp::Add, vec![replicate, cd4], 8, rhs_deps.clone());
        if rhs_is_new {
            pool.add(rhs, 8, rhs_deps);
        }

        let shr_deps = DepSet::union(&[&node_deps(&m, lhs), &node_deps(&m, rhs)]);
        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![lhs, rhs], 8, shr_deps.clone());
        if shr_is_new {
            pool.add(shr, 8, shr_deps);
        }

        assert!(
            !can_enumerate_small_value_set(&m, rhs),
            "the rhs itself should exceed the exact small-set support cap so this proof exercises the tiny-domain fallback",
        );
        assert!(
            !can_enumerate_small_value_set(&m, shr),
            "the whole shift node should exceed the small-set support cap so this proof exercises the rhs-only overshift path"
        );
        assert_eq!(
            prove_node_exact_value(&m, shr),
            Some(0),
            "a shift must still fold to zero when the rhs small-value set stays entirely above the source width, even if the whole node cannot use exact small-set enumeration"
        );
    }

    #[test]
    fn add_bounds_preserve_shifted_single_interval_without_small_set_help() {
        let (mut m, mut pool) = fixture_with_inputs(4, 2, 0);
        m.factorization_level = FactorizationLevel::None;

        let a = 0;
        let b = 1;
        let c = 2;
        let d = 3;
        let deps = DepSet::union(&[
            &node_deps(&m, a),
            &node_deps(&m, b),
            &node_deps(&m, c),
            &node_deps(&m, d),
        ]);

        let (concat, concat_is_new) =
            m.intern_gate(GateOp::Concat, vec![a, b, c, d], 8, deps.clone());
        if concat_is_new {
            pool.add(concat, 8, deps.clone());
        }

        let e7 = make_constant(&mut m, &mut pool, 8, 0xe7);
        let (rhs_base, rhs_base_is_new) =
            m.intern_gate(GateOp::Or, vec![e7, concat], 8, deps.clone());
        if rhs_base_is_new {
            pool.add(rhs_base, 8, deps.clone());
        }

        let c0 = make_constant(&mut m, &mut pool, 8, 0x0c);
        let c1 = make_constant(&mut m, &mut pool, 8, 0xc4);
        let (rhs, rhs_is_new) = m.intern_gate(GateOp::Add, vec![rhs_base, c0, c1], 8, deps.clone());
        if rhs_is_new {
            pool.add(rhs, 8, deps.clone());
        }

        let lhs = make_constant(&mut m, &mut pool, 3, 0b101);
        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![lhs, rhs], 3, deps.clone());
        if shr_is_new {
            pool.add(shr, 3, deps);
        }

        assert!(
            !can_enumerate_small_value_set(&m, rhs),
            "the rhs must exceed the exact small-set support cap so this regression exercises bounds, not enumeration"
        );
        assert!(
            !can_enumerate_small_value_set(&m, shr),
            "the shift node must also stay outside the exact small-set path"
        );

        let mut memo = HashMap::new();
        assert_eq!(
            node_unsigned_bounds(&m, rhs, &mut memo),
            (183, 207),
            "a single non-exact interval shifted by exact wrapped constants should keep its useful lower bound"
        );
        assert_eq!(
            prove_node_exact_value(&m, shr),
            Some(0),
            "bounds alone should prove the overshift even when no small-set path is available"
        );
    }

    #[test]
    fn case_mux_bounds_follow_exact_selector_for_dynamic_overshift() {
        let (mut m, mut pool) = fixture_with_inputs(1, 2, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;

        let selector = make_constant(&mut m, &mut pool, 1, 1);
        let unused_arm = make_constant(&mut m, &mut pool, 8, 0x73);
        let base = make_constant(&mut m, &mut pool, 8, 0x5d);
        let add_deps = DepSet::union(&[&node_deps(&m, base), &node_deps(&m, x)]);
        let (dynamic_arm, dynamic_arm_is_new) =
            m.intern_gate(GateOp::Add, vec![base, x], 8, add_deps.clone());
        if dynamic_arm_is_new {
            pool.add(dynamic_arm, 8, add_deps);
        }

        let rhs = make_case_mux(&mut m, &mut pool, selector, &[unused_arm, dynamic_arm], 8);
        let mut memo = HashMap::new();
        assert_eq!(
            node_unsigned_bounds(&m, rhs, &mut memo),
            (0x5d, 0x60),
            "an exact case selector should expose the selected arm's bounds"
        );

        let lhs = make_constant(&mut m, &mut pool, 5, 0x1c);
        let shr_deps = DepSet::union(&[&node_deps(&m, lhs), &node_deps(&m, rhs)]);
        let (shr, shr_is_new) = m.intern_gate(GateOp::Shr, vec![lhs, rhs], 5, shr_deps.clone());
        if shr_is_new {
            pool.add(shr, 5, shr_deps);
        }

        assert_eq!(
            prove_node_exact_value(&m, shr),
            Some(0),
            "case-selected shift amounts that are always >= source width should fold before Yosys warns"
        );
    }

    #[test]
    fn casez_mux_bounds_follow_exact_matching_pattern() {
        let (mut m, mut pool) = fixture_with_inputs(1, 3, 0);
        m.factorization_level = FactorizationLevel::None;
        let x = 0;

        let selector = make_constant(&mut m, &mut pool, 2, 3);
        let low_arm = make_constant(&mut m, &mut pool, 8, 0x11);
        let base = make_constant(&mut m, &mut pool, 8, 0xa0);
        let high_arm_deps = DepSet::union(&[&node_deps(&m, base), &node_deps(&m, x)]);
        let (high_arm, high_arm_is_new) =
            m.intern_gate(GateOp::Add, vec![base, x], 8, high_arm_deps.clone());
        if high_arm_is_new {
            pool.add(high_arm, 8, high_arm_deps);
        }

        let rhs = make_casez_mux(
            &mut m,
            &mut pool,
            selector,
            &[(0b00, 0b01), (0b10, 0b01)],
            &[low_arm, high_arm],
            8,
        );
        let mut memo = HashMap::new();
        assert_eq!(
            node_unsigned_bounds(&m, rhs, &mut memo),
            (0xa0, 0xa7),
            "an exact casez selector should use the first matching wildcard arm's bounds"
        );
    }
}
