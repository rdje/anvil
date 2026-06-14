//! Flop D-cone assembly + worklist draining (`CONE-DECOMPOSITION.6`).
//!
//! Draining the deferred flop worklist (`drain_flop_worklist`, the public
//! entry used by `src/gen/module.rs` and `src/gen/hierarchy.rs`), the
//! one-hot / encoded flop-mux D-cone assemblers, the `build_flop_leaf`
//! allocator, and `pick_reset_value`. Also hosts the two small shared
//! helpers `ceil_log2` and `pick_mux_arm_count` that sit inline with the
//! flop-mux encoders (re-exported, so other callers keep their paths).
//! Extracted verbatim from `cone.rs`; behaviour is unchanged.

use super::{
    build_cone_with_retry, make_and, make_constant, make_eq_const, make_mux, make_none_selected,
    or_reduce_terms, replicate_to_width, roll_knob, FlopWorklist,
};
use crate::gen::pool::SignalPool;
use crate::gen::Generator;
use crate::ir::{
    DepSet, Flop, FlopId, FlopKind, FlopMux, KnobId, Module, MuxArm, Node, NodeId, ResetKind,
};
use rand::Rng;
use tracing::{instrument, warn};

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

pub(crate) fn drain_flop_one_hot(
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

pub(crate) fn drain_flop_encoded(
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
pub(crate) fn ceil_log2(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        32 - (n - 1).leading_zeros()
    }
}

/// Pick M from {0, 2, 3, ..., max_mux_arms}. M = 1 is excluded by
/// design — a 1-arm mux is just a wire.
pub(crate) fn pick_mux_arm_count(g: &mut Generator) -> u32 {
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
pub(crate) fn assemble_flop_d_one_hot(
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
pub(crate) fn assemble_flop_d_encoded(
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

/// Allocate a flop and a `FlopQ` node. The Q is returned (and added to
/// the pool) as the leaf for the current cone. The flop's D-cone is
/// queued for later construction by `drain_flop_worklist`.
#[instrument(level = "trace", skip(g, m, pool, worklist))]
pub(crate) fn build_flop_leaf(
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

pub(crate) fn pick_reset_value(g: &mut Generator, width: u32) -> u128 {
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
