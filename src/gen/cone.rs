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
    DepSet, Flop, FlopId, FlopKind, FlopMux, GateOp, Module, MuxArm, Node, NodeId, ResetKind,
};
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
pub fn build_graph_first(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> Vec<NodeId> {
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
        iterations += 1;
        if grow_pool_one_unit(g, m, pool, worklist) {
            emitted += 1;
        }
    }

    // Phase 2 — resolve flop D-cones using pool-only picks. By this
    // point the pool is fully grown, so every flop has the full pool
    // to pick its D-mux operands from. Q-feedback is permitted freely
    // (Rule 2) — `exclude` is None throughout.
    drain_flop_worklist_pool_only(g, m, pool, worklist);

    // Phase 3 — pick a drive-root for each output from the pool.
    // `pick_terminal` handles the adapter fallback when no matching-
    // width entry exists.
    (0..m.outputs.len())
        .map(|i| pick_terminal(g, m, pool, m.outputs[i].width, None))
        .collect()
}

fn grow_pool_one_unit(
    g: &mut Generator,
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> bool {
    let width = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);

    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    if flop_allowed && g.rng.gen_bool(g.cfg.flop_prob.min(1.0)) {
        build_flop_leaf(g, m, pool, worklist, width);
        return true;
    }

    if g.rng.gen_bool(g.cfg.comb_mux_prob.min(1.0)) {
        build_comb_mux_pool_only(g, m, pool, width);
        return true;
    }

    let op = pick_gate(g, width);

    // Coefficient motif (pool-only signal picks). Same doctrine as the
    // recursive path: Add/Sub/Mul with coefficient_prob probability
    // becomes a linear-combination compound.
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && g.rng.gen_bool(g.cfg.coefficient_prob.min(1.0))
    {
        build_linear_combination_pool(g, m, pool, op, width);
        return true;
    }

    // Constant shift-amount motif (pool-only). Value operand is a
    // pool pick; shift amount is a literal constant.
    if matches!(op, GateOp::Shl | GateOp::Shr)
        && g.rng.gen_bool(g.cfg.const_shift_amount_prob.min(1.0))
    {
        let value = pick_terminal(g, m, pool, width, None);
        build_shift_const_amount(g, m, pool, op, value, width);
        return true;
    }

    // Constant comparand motif (pool-only). LHS is a pool pick of
    // internal operand width K; RHS is a literal constant. Output
    // is 1-bit.
    if is_comparison_op(op) && g.rng.gen_bool(g.cfg.const_comparand_prob.min(1.0)) {
        let k = pick_comparison_operand_width(g);
        let lhs = pick_terminal(g, m, pool, k, None);
        build_comparison_const_comparand(g, m, pool, op, lhs, k);
        return true;
    }

    let operand_widths = input_widths_for(op, width, &g.cfg, &mut g.rng);
    for _ in 0..4 {
        let operands: Vec<NodeId> = operand_widths
            .iter()
            .map(|w| pick_terminal(g, m, pool, *w, None))
            .collect();
        if !violates_anti_collapse(op, &operands, m) {
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
            return true;
        }
    }
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

    let encoded = g.rng.gen_bool(g.cfg.comb_mux_encoding_prob.min(1.0));
    if encoded {
        let sel_width = ceil_log2(n_arms);
        let sel = pick_terminal(g, m, pool, sel_width, None);
        let datas: Vec<NodeId> = (0..n_arms)
            .map(|_| pick_terminal(g, m, pool, width, None))
            .collect();
        let fall_through = make_constant(m, pool, width, 0);
        let mut tail = fall_through;
        for idx_rev in 0..n_arms {
            let idx = n_arms - 1 - idx_rev;
            let eq = make_eq_const(m, pool, sel, sel_width, idx as u128);
            tail = make_mux(m, pool, eq, datas[idx as usize], tail, width);
        }
        tail
    } else {
        let mut arms: Vec<MuxArm> = Vec::with_capacity(n_arms as usize);
        for _ in 0..n_arms {
            let data = pick_terminal(g, m, pool, width, None);
            let sel = pick_terminal(g, m, pool, 1, None);
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

        let encoded = g.rng.gen_bool(g.cfg.flop_mux_encoding_prob.min(1.0));
        if encoded {
            let sel_width = ceil_log2(m_arms);
            let sel = pick_terminal(g, m, pool, sel_width, exclude);
            let datas: Vec<NodeId> = match kind {
                FlopKind::ZeroDefault => (0..m_arms)
                    .map(|_| pick_terminal(g, m, pool, width, exclude))
                    .collect(),
                FlopKind::QFeedback => (1..m_arms)
                    .map(|_| pick_terminal(g, m, pool, width, exclude))
                    .collect(),
            };
            let d = assemble_flop_d_encoded(m, pool, width, sel, sel_width, &datas, kind, q_node);
            m.flops[flop_id as usize].d = Some(d);
            m.flops[flop_id as usize].mux = FlopMux::Encoded { sel, data: datas };
        } else {
            let mut arms: Vec<MuxArm> = Vec::with_capacity(m_arms as usize);
            for _ in 0..m_arms {
                let data = pick_terminal(g, m, pool, width, exclude);
                let sel = pick_terminal(g, m, pool, 1, exclude);
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
/// flop D-cones are built depth-first per flop. Full symmetry awaits
/// `graph-first`.
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
    let force_leaf = frame.depth >= g.cfg.max_depth || g.rng.gen_bool(leaf_prob.min(1.0));

    if force_leaf {
        let node = pick_terminal(g, m, pool, frame.width, frame.exclude);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Flop block: allocates a Flop and enqueues its D-cone on the worklist.
    // The FlopQ node is returned immediately and the frame resolves.
    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    if flop_allowed && g.rng.gen_bool(g.cfg.flop_prob.min(1.0)) {
        let node = build_flop_leaf(g, m, pool, worklist, frame.width);
        deliver(g, m, pool, node, frame.dest, gate_frames, per_output_drive);
        return;
    }

    // Comb-mux block: builds its internal sub-cones depth-first within
    // this frame step. Block placement interleaves with other cones;
    // block internals do not. This matches the "near-symmetric" scope.
    if g.rng.gen_bool(g.cfg.comb_mux_prob.min(1.0)) {
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

    // Operator gate: push a GateFrame into the in-flight table, enqueue
    // one SignalFrame per operand slot. The gate finalizes when its
    // last operand resolves (see `deliver`).
    let op = pick_gate(g, frame.width);

    // Coefficient motif: Add/Sub/Mul with coefficient_prob becomes a
    // compound linear-combination tree. Built synchronously within
    // this frame step (the tree itself is atomic; its signal leaves
    // come from recursive build_cone just like block internals).
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && g.rng.gen_bool(g.cfg.coefficient_prob.min(1.0))
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
        && g.rng.gen_bool(g.cfg.const_shift_amount_prob.min(1.0))
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
    if is_comparison_op(op) && g.rng.gen_bool(g.cfg.const_comparand_prob.min(1.0)) {
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
        let shared = if g.rng.gen_bool(g.cfg.share_prob.min(1.0)) {
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

                // Structural anti-collapse: same check as recursive path.
                if violates_anti_collapse(gf.op, &operands, m) {
                    let fallback = pick_terminal(g, m, pool, gf.width, None);
                    deliver(g, m, pool, fallback, gf.dest, gate_frames, per_output_drive);
                    return;
                }

                let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
                let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
                let node_id = m.nodes.len() as NodeId;
                m.nodes.push(Node::Gate {
                    op: gf.op,
                    operands,
                    width: gf.width,
                    deps: deps.clone(),
                });
                pool.add(node_id, gf.width, deps);
                deliver(g, m, pool, node_id, gf.dest, gate_frames, per_output_drive);
            }
        }
    }
}

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

        let encoded = g.rng.gen_bool(g.cfg.flop_mux_encoding_prob.min(1.0));
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

fn make_constant(m: &mut Module, pool: &mut SignalPool, width: u32, value: u128) -> NodeId {
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Constant { width, value });
    pool.add(node_id, width, DepSet::new());
    node_id
}

fn make_eq_const(
    m: &mut Module,
    pool: &mut SignalPool,
    operand: NodeId,
    operand_width: u32,
    value: u128,
) -> NodeId {
    let const_node = make_constant(m, pool, operand_width, value);
    let deps = node_deps(m, operand);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Eq,
        operands: vec![operand, const_node],
        width: 1,
        deps: deps.clone(),
    });
    pool.add(node_id, 1, deps);
    node_id
}

fn make_mux(
    m: &mut Module,
    pool: &mut SignalPool,
    sel: NodeId,
    a: NodeId,
    b: NodeId,
    width: u32,
) -> NodeId {
    let deps = DepSet::union(&[&node_deps(m, sel), &node_deps(m, a), &node_deps(m, b)]);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Mux,
        operands: vec![sel, a, b],
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
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

fn make_mul(m: &mut Module, pool: &mut SignalPool, a: NodeId, b: NodeId, width: u32) -> NodeId {
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Mul,
        operands: vec![a, b],
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

fn make_sub(m: &mut Module, pool: &mut SignalPool, a: NodeId, b: NodeId, width: u32) -> NodeId {
    let deps = DepSet::union(&[&node_deps(m, a), &node_deps(m, b)]);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Sub,
        operands: vec![a, b],
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

/// N-arity Add with all operands at `width`. N must be >= 2.
fn make_nary_add(m: &mut Module, pool: &mut SignalPool, operands: &[NodeId], width: u32) -> NodeId {
    debug_assert!(operands.len() >= 2);
    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Add,
        operands: operands.to_vec(),
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

/// N-arity Mul with all operands at `width`. N must be >= 2.
fn make_nary_mul(m: &mut Module, pool: &mut SignalPool, operands: &[NodeId], width: u32) -> NodeId {
    debug_assert!(operands.len() >= 2);
    let deps_vec: Vec<DepSet> = operands.iter().map(|id| node_deps(m, *id)).collect();
    let deps = DepSet::union(&deps_vec.iter().collect::<Vec<_>>());
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op: GateOp::Mul,
        operands: operands.to_vec(),
        width,
        deps: deps.clone(),
    });
    pool.add(node_id, width, deps);
    node_id
}

/// Draw a strictly positive coefficient from the configured range.
fn pick_coefficient(g: &mut Generator) -> u128 {
    let coef_min = g.cfg.min_coefficient.max(1);
    let coef_max = g.cfg.max_coefficient.max(coef_min);
    g.rng.gen_range(coef_min..=coef_max) as u128
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
fn pick_mul_coefficient_and_arity(g: &mut Generator) -> (u128, u32) {
    let coef = pick_coefficient(g);
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
        let coef = pick_coefficient(g);
        let const_node = make_constant(m, pool, width, coef);
        terms.push(make_mul(m, pool, s, const_node, width));
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
        let coef = pick_coefficient(g);
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
    let const_node = make_constant(m, pool, width, coef);
    let mut operands: Vec<NodeId> = Vec::with_capacity(signals.len() + 1);
    operands.push(const_node);
    operands.extend_from_slice(signals);
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
            let (coef, n) = pick_mul_coefficient_and_arity(g);
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
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op,
        operands: vec![value_node, const_node],
        width: value_width,
        deps: deps.clone(),
    });
    pool.add(node_id, value_width, deps);
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
    let deps = node_deps(m, lhs);
    let node_id = m.nodes.len() as NodeId;
    m.nodes.push(Node::Gate {
        op,
        operands: vec![lhs, const_node],
        width: 1,
        deps: deps.clone(),
    });
    pool.add(node_id, 1, deps);
    node_id
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
            let signals: Vec<NodeId> = (0..n)
                .map(|_| pick_terminal(g, m, pool, width, None))
                .collect();
            assemble_add_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Sub => {
            let n = pick_linear_combination_arity(g);
            let signals: Vec<NodeId> = (0..n)
                .map(|_| pick_terminal(g, m, pool, width, None))
                .collect();
            assemble_sub_linear_combination(g, m, pool, width, &signals)
        }
        GateOp::Mul => {
            let (coef, n) = pick_mul_coefficient_and_arity(g);
            let signals: Vec<NodeId> = (0..n)
                .map(|_| pick_terminal(g, m, pool, width, None))
                .collect();
            assemble_mul_linear_combination(m, pool, width, coef, &signals)
        }
        _ => unreachable!("build_linear_combination_pool: op must be Add/Sub/Mul"),
    }
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

    // Recursion fork: flop block, comb-mux block, or operator gate.
    // Blocks take priority over operator gates. Ordering between flop
    // and comb-mux is first-come by their independent probability rolls.
    let flop_allowed = (m.flops.len() as u32) < g.cfg.max_flops_per_module;
    let pick_flop = flop_allowed && g.rng.gen_bool(g.cfg.flop_prob.min(1.0));
    if pick_flop {
        return build_flop_leaf(g, m, pool, worklist, width);
    }

    let pick_comb_mux = g.rng.gen_bool(g.cfg.comb_mux_prob.min(1.0));
    if pick_comb_mux {
        return build_comb_mux(g, m, pool, worklist, width, depth, exclude);
    }

    let op = pick_gate(g, width);

    // Coefficient motif: when the picked op is Add / Sub / Mul and the
    // per-op probability fires, emit a linear-combination compound tree
    // (see `book/src/structural-rules.md` "Roles of constants in RTL").
    // Signals are picked via the usual recursive path.
    if matches!(op, GateOp::Add | GateOp::Sub | GateOp::Mul)
        && g.rng.gen_bool(g.cfg.coefficient_prob.min(1.0))
    {
        return build_linear_combination_recursive(g, m, pool, worklist, op, width, depth, exclude);
    }

    // Constant shift-amount motif: when the picked op is Shl/Shr and
    // the per-shift probability fires, emit `value OP const` with a
    // literal shift amount instead of a barrel shifter.
    if matches!(op, GateOp::Shl | GateOp::Shr)
        && g.rng.gen_bool(g.cfg.const_shift_amount_prob.min(1.0))
    {
        let value = build_cone(g, m, pool, worklist, width, depth + 1, exclude);
        return build_shift_const_amount(g, m, pool, op, value, width);
    }

    // Constant comparand motif: when the picked op is a comparison
    // and the per-comparison probability fires, emit `lhs OP const`
    // instead of recursing on both operands.
    if is_comparison_op(op) && g.rng.gen_bool(g.cfg.const_comparand_prob.min(1.0)) {
        let k = pick_comparison_operand_width(g);
        let lhs = build_cone(g, m, pool, worklist, k, depth + 1, exclude);
        return build_comparison_const_comparand(g, m, pool, op, lhs, k);
    }

    let operand_widths = input_widths_for(op, width, &g.cfg, &mut g.rng);
    let mut operands = Vec::with_capacity(operand_widths.len());
    for w in operand_widths {
        // DAG-sharing fork (Phase 2): with probability share_prob, terminate
        // this operand at an existing matching-width pool entry instead of
        // recursing to create fresh logic. Falls back to recursion if no
        // shareable candidate exists. Share/recurse is decided per-operand,
        // so a single gate's operands can mix shared and freshly-built sub-cones.
        let share = g.rng.gen_bool(g.cfg.share_prob.min(1.0));
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
/// Build an M-to-1 combinational mux block.
///
/// A *block* (not an operator — see `book/src/structural-rules.md`):
/// ports are M data inputs (width W) + 1 select (1-bit × M for
/// OneHot, ceil(log2(M))-bit for Encoded). No Q-feedback axis because
/// combinational muxes have no state.
///
/// When no select asserts (OneHot) or select is out of range
/// (Encoded, when M is not a power of 2), output is 0.
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

    let encoded = g.rng.gen_bool(g.cfg.comb_mux_encoding_prob.min(1.0));
    if encoded {
        build_comb_mux_encoded(g, m, pool, worklist, width, depth, exclude, n_arms)
    } else {
        build_comb_mux_one_hot(g, m, pool, worklist, width, depth, exclude, n_arms)
    }
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
    let arith: &[GateOp] = &[Add, Sub, Mul];
    let structured: &[GateOp] = &[Mux];
    let compare: &[GateOp] = if target_width == 1 {
        &[Eq, Neq, Lt]
    } else {
        &[]
    };
    // Shifts only make sense on multi-bit signals (shifting a 1-bit
    // value by >= 1 always yields 0 for unsigned; a shift-by-0 is a
    // wire). Keep them out of the pool at width 1.
    let shifts: &[GateOp] = if target_width > 1 { &[Shl, Shr] } else { &[] };

    let w = &g.cfg;
    let buckets: [(u32, &[GateOp]); 5] = [
        (w.gate_bitwise_weight, bitwise),
        (w.gate_arith_weight, arith),
        (w.gate_struct_weight, structured),
        (w.gate_compare_weight, compare),
        (w.gate_shift_weight, shifts),
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

fn node_deps(m: &Module, id: NodeId) -> DepSet {
    match &m.nodes[id as usize] {
        Node::PrimaryInput { port, .. } => DepSet::from_port(*port),
        Node::Constant { .. } => DepSet::new(),
        Node::FlopQ { flop, .. } => DepSet::from_flop_virtual(*flop),
        Node::Gate { deps, .. } => deps.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ir::{Direction, Flop, FlopKind, MuxArm, Port, ResetKind};

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
        let arms = vec![
            MuxArm { data: 0, sel: 2 },
            MuxArm { data: 1, sel: 3 },
        ];
        let d = assemble_flop_d_one_hot(&mut m, &mut pool, 4, &arms, FlopKind::ZeroDefault, q);
        match &m.nodes[d as usize] {
            Node::Gate { op, width, .. } => {
                assert_eq!(*op, GateOp::Or, "top-level of OneHot ZeroDefault should be Or");
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
        let arms = vec![
            MuxArm { data: 0, sel: 2 },
            MuxArm { data: 1, sel: 3 },
        ];
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
        assert!(has_not, "QFeedback OneHot should emit a Not for ~(OR of sels)");
    }

    #[test]
    fn assemble_flop_d_encoded_zero_default_top_is_mux() {
        // 2 data (width 4) + 1 sel bus (sel_width = ceil_log2(M=2) = 1) + 1 flop.
        let (mut m, mut pool) = fixture_with_inputs(2, 4, 1);
        let q = alloc_flop(&mut m, &mut pool, 4, FlopKind::ZeroDefault);
        // Nodes: 0=data0, 1=data1, 2=sel (1 bit), 3=Q.
        let datas = vec![0, 1];
        let d = assemble_flop_d_encoded(
            &mut m,
            &mut pool,
            4,
            2,
            1,
            &datas,
            FlopKind::ZeroDefault,
            q,
        );
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

    fn make_generator(flop_prob: f64) -> Generator {
        let cfg = Config {
            seed: 42,
            flop_prob,
            ..Config::default()
        };
        Generator::new(cfg)
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
    fn width_adapter_concat_then_slice_non_multiple() {
        let (mut m, mut pool, src, deps) = scaffold_module_with_input(3);
        // target = 8, copies = ceil(8/3) = 3, concat width = 9, then slice to 8.
        let out = make_width_adapter(&mut m, &mut pool, src, 3, deps, 8);
        // The outermost node should be a Slice of width 8.
        match &m.nodes[out as usize] {
            Node::Gate {
                op: GateOp::Slice { hi, lo },
                width,
                ..
            } => {
                assert_eq!(*hi, 7);
                assert_eq!(*lo, 0);
                assert_eq!(*width, 8);
            }
            other => panic!("expected outer Slice, got {other:?}"),
        }
        // And a Concat of width 9 should exist somewhere in the module.
        let has_concat_9 = m.nodes.iter().any(|n| {
            matches!(
                n,
                Node::Gate {
                    op: GateOp::Concat,
                    width: 9,
                    ..
                }
            )
        });
        assert!(has_concat_9, "expected a 9-bit Concat as the Slice source");
    }
}
