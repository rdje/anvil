//! Leaf-module generation: N inputs, M outputs, internal cones with
//! optional flops. Single CLK (posedge), single RST_N (async, active-low).

use super::{
    cone::{self, FlopWorklist},
    pool::SignalPool,
    Generator,
};
use crate::config::ConstructionStrategy;
use crate::ir::{
    DepSet, Direction, GateOp, MemKind, Memory, Module, ModuleInterfaceProfile, Node, NodeId, Port,
    PortId,
};
use rand::seq::SliceRandom;
use rand::Rng;
use tracing::{debug, info, instrument};

const CLK_NAME: &str = "clk";
const RST_N_NAME: &str = "rst_n";

/// Generate one leaf module: ports, internal cones, and optional state.
///
/// This remains the Phase 1/2/3 kernel. Future hierarchy should compose
/// leaf modules above this function rather than silently folding
/// inter-module generation into it.
#[instrument(level = "info", skip(g), fields(seed = g.cfg.seed))]
pub fn generate_leaf_module(g: &mut Generator, index: u64) -> Module {
    generate_leaf_module_with_interface_profile(g, index, None)
}

/// Rules-first parameterizable-leaf constructor
/// (PHASE-5-PARAMETERIZATION.2.2.2).
///
/// Builds a **width-homogeneous combinational leaf** by construction:
/// a single design width `W >= 2`; `n_in` data inputs and `n_out`
/// outputs all width `W`; no `clk`/`rst_n` (purely combinational, no
/// flops, no instances); each output driven by one N-arity
/// width-preserving gate (`Xor`/`And`/`Or`/`Add`) over *all* input
/// nodes, so every port and every node width equals `W`. No
/// `Constant`/`Slice`/`Concat`/`ForFold`/`Mux`/compare. The
/// `is_width_generic` soundness gate therefore always accepts the
/// result, and the single emitted SystemVerilog body text is correct
/// for every `W` — valid by construction, not by post-hoc filtering.
///
/// All random choices go through `g.rng` (reproducible). The op
/// palette is indexed by output position so the first outputs use
/// distinct ops (distinct expressions); identical expressions beyond
/// the palette legitimately share a node (fanout).
fn build_parameterizable_leaf(g: &mut Generator, index: u64) -> Module {
    let mut m = Module {
        name: g.module_name(index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
        ..Module::default()
    };

    // Design width: >= 2 (so `[W-1:0]` is well-formed) and within the
    // configured width band.
    let w_lo = g.cfg.min_width.max(2);
    let w_hi = g.cfg.max_width.max(w_lo);
    let width: u32 = g.rng.gen_range(w_lo..=w_hi);

    // N-arity width-preserving operators need >= 2 operands; keep the
    // interface small and bounded.
    let n_in: usize = g.rng.gen_range(2..=4);
    let n_out: usize = g.rng.gen_range(1..=3);

    // Data input ports (ids 0..n_in) + their PrimaryInput nodes. No
    // control ports: a width-homogeneous module is purely
    // combinational, so width-1 clk/rst_n must not appear.
    let mut input_nodes: Vec<NodeId> = Vec::with_capacity(n_in);
    let mut input_deps: Vec<DepSet> = Vec::with_capacity(n_in);
    for i in 0..n_in {
        let port_id = i as PortId;
        m.inputs.push(Port {
            id: port_id,
            name: format!("i_{}", i),
            width,
            dir: Direction::In,
        });
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput {
            port: port_id,
            width,
        });
        input_nodes.push(node_id);
        input_deps.push(DepSet::from_port(port_id));
    }

    // Output ports (ids n_in..n_in+n_out), each driven by one
    // width-preserving N-arity gate over all input nodes.
    let palette = [GateOp::Xor, GateOp::And, GateOp::Or, GateOp::Add];
    let deps = DepSet::union(&input_deps.iter().collect::<Vec<_>>());
    for o in 0..n_out {
        let port_id = (n_in + o) as PortId;
        m.outputs.push(Port {
            id: port_id,
            name: format!("o_{}", o),
            width,
            dir: Direction::Out,
        });
        let op = palette[o % palette.len()];
        let (root, _) = m.intern_gate(op, input_nodes.clone(), width, deps.clone());
        m.drives.push((port_id, root));
    }

    m
}

/// Rules-first inferrable-memory-leaf constructor
/// (`PHASE-6-ADVANCED-MOTIFS.2.1b`).
///
/// Builds, by rule, a single-`Memory` leaf whose emitted SV is the
/// `.1`-validated synchronous-write / synchronous-read template Yosys
/// infers as `$mem_v2`: a shared `clk`, `we`/`waddr`/`wdata` write
/// inputs (+ an independent `raddr` for `SimpleDualPort`), and one
/// `rdata` output driven by the opaque `Node::MemRead` leaf. Valid by
/// construction — not a post-hoc filter. All random choices go through
/// `g.rng` (reproducible). No combinational gates, no flops; the
/// memory is the only state.
fn build_memory_leaf(g: &mut Generator, index: u64) -> Module {
    let mut m = Module {
        name: g.module_name(index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
        clock: Some(0),
        reset: Some(1),
        ..Module::default()
    };
    // Bounded, well-formed dimensions.
    let addr_width: u32 = g.rng.gen_range(2..=4);
    let dw_lo = g.cfg.min_width.max(2);
    let dw_hi = g.cfg.max_width.max(dw_lo);
    let data_width: u32 = g.rng.gen_range(dw_lo..=dw_hi);
    let dual = g.rng.gen_bool(0.5);
    let kind = if dual {
        MemKind::SimpleDualPort
    } else {
        MemKind::SinglePort
    };

    // Control ports (clk/rst_n) are shared; rst_n is unused by the
    // reset-less memory block but kept for the standard control-port
    // surface (downstream-clean under bare `verilator --lint-only`).
    m.inputs.push(Port {
        id: 0,
        name: CLK_NAME.into(),
        width: 1,
        dir: Direction::In,
    });
    m.inputs.push(Port {
        id: 1,
        name: RST_N_NAME.into(),
        width: 1,
        dir: Direction::In,
    });
    m.inputs.push(Port {
        id: 2,
        name: "we".into(),
        width: 1,
        dir: Direction::In,
    });
    m.inputs.push(Port {
        id: 3,
        name: "waddr".into(),
        width: addr_width,
        dir: Direction::In,
    });
    m.inputs.push(Port {
        id: 4,
        name: "wdata".into(),
        width: data_width,
        dir: Direction::In,
    });
    let we = m.nodes.len() as NodeId;
    m.nodes.push(Node::PrimaryInput { port: 2, width: 1 });
    let waddr = m.nodes.len() as NodeId;
    m.nodes.push(Node::PrimaryInput {
        port: 3,
        width: addr_width,
    });
    let wdata = m.nodes.len() as NodeId;
    m.nodes.push(Node::PrimaryInput {
        port: 4,
        width: data_width,
    });
    let raddr = if dual {
        m.inputs.push(Port {
            id: 5,
            name: "raddr".into(),
            width: addr_width,
            dir: Direction::In,
        });
        let r = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput {
            port: 5,
            width: addr_width,
        });
        r
    } else {
        // SinglePort: one shared address (raddr == waddr).
        waddr
    };
    let out_id: PortId = if dual { 6 } else { 5 };
    m.outputs.push(Port {
        id: out_id,
        name: "rdata".into(),
        width: data_width,
        dir: Direction::Out,
    });
    m.memories.push(Memory {
        id: 0,
        addr_width,
        data_width,
        kind,
        we,
        waddr,
        wdata,
        raddr,
    });
    let rd = m.nodes.len() as NodeId;
    m.nodes.push(Node::MemRead {
        mem: 0,
        width: data_width,
    });
    m.drives.push((out_id, rd));
    m
}

pub(super) fn generate_leaf_module_with_interface_profile(
    g: &mut Generator,
    index: u64,
    interface_profile: Option<&ModuleInterfaceProfile>,
) -> Module {
    // Phase 5 (PHASE-5-PARAMETERIZATION.2.2.2): rules-first
    // parameterizable lane. When the opt-in knob rolls true and this
    // is the free-standing single-module lane (no parent-demanded
    // interface profile), *construct* a width-homogeneous
    // combinational leaf by rule so the feature actually fires —
    // rather than generating a normal module and hoping it happens to
    // be width-homogeneous (inert + generate-then-filter). This is the
    // single opt-in roll; `param::annotate_parameterized` downstream
    // is non-rolling. Default-off (`prob == 0.0`) never enters here,
    // so emission stays byte-identical.
    if interface_profile.is_none()
        && g.cfg.width_parameterization_prob > 0.0
        && g.rng
            .gen_bool(g.cfg.width_parameterization_prob.clamp(0.0, 1.0))
    {
        let mut m = build_parameterizable_leaf(g, index);
        crate::ir::param::annotate_parameterized(&mut m, &g.cfg);
        return m;
    }

    // Phase 6 (PHASE-6-ADVANCED-MOTIFS.2.1b): rules-first
    // inferrable-memory lane. Mutually exclusive with the Phase 5
    // parameterization lane above (only reached when that did not
    // fire). Single opt-in roll; default-off (`memory_prob == 0.0`)
    // never enters here, so emission stays byte-identical.
    if interface_profile.is_none()
        && g.cfg.memory_prob > 0.0
        && g.rng.gen_bool(g.cfg.memory_prob.clamp(0.0, 1.0))
    {
        return build_memory_leaf(g, index);
    }

    let planned_profile = interface_profile
        .cloned()
        .unwrap_or_else(|| sample_leaf_interface_profile(g));
    let n_in = planned_profile.data_input_widths.len();
    let n_out = planned_profile.output_widths.len();
    info!(
        n_in,
        n_out,
        strategy = ?g.cfg.construction_strategy,
        "🚀 build module"
    );

    let mut m = Module {
        name: g.module_name(index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
        planned_interface_profile: interface_profile.cloned(),
        ..Module::default()
    };

    // Reserve port id 0 for clk and 1 for rst_n. They are shared by every
    // flop in the module. Whether they appear in the emitted SV depends on
    // whether any flops are generated (decided post-hoc by the emitter).
    let clk_id: PortId = 0;
    let rst_n_id: PortId = 1;
    m.inputs.push(Port {
        id: clk_id,
        name: CLK_NAME.into(),
        width: 1,
        dir: Direction::In,
    });
    m.inputs.push(Port {
        id: rst_n_id,
        name: RST_N_NAME.into(),
        width: 1,
        dir: Direction::In,
    });
    m.clock = Some(clk_id);
    m.reset = Some(rst_n_id);

    // Primary data inputs: port ids 2..2+n_in.
    for (i, w) in planned_profile
        .data_input_widths
        .iter()
        .copied()
        .enumerate()
    {
        let port_id = (2 + i) as PortId;
        m.inputs.push(Port {
            id: port_id,
            name: format!("i_{}", i),
            width: w,
            dir: Direction::In,
        });
    }

    // Primary outputs: port ids start after all inputs.
    let out_id_base = 2 + n_in;
    for (i, w) in planned_profile.output_widths.iter().copied().enumerate() {
        let port_id = (out_id_base + i) as PortId;
        m.outputs.push(Port {
            id: port_id,
            name: format!("o_{}", i),
            width: w,
            dir: Direction::Out,
        });
    }

    // Seed the signal pool with primary DATA inputs only — clk and rst_n
    // must never appear as cone leaves.
    let mut pool = SignalPool::new();
    let data_inputs: Vec<Port> = m
        .inputs
        .iter()
        .filter(|p| p.id != clk_id && p.id != rst_n_id)
        .cloned()
        .collect();
    for p in &data_inputs {
        let node_id = m.nodes.len() as u32;
        m.nodes.push(Node::PrimaryInput {
            port: p.id,
            width: p.width,
        });
        pool.add(node_id, p.width, DepSet::from_port(p.id));
    }

    let mut worklist: FlopWorklist = Vec::new();

    // Build an output cone per primary output. The iteration order / the
    // overall scheduling is governed by `cfg.construction_strategy`:
    //
    // - `Sequential`: declaration order (0, 1, ..., n_out-1), depth-first.
    // - `Shuffled`: random permutation of output order, depth-first per cone.
    // - `Interleaved`: one global frame queue; cones grow in lockstep.
    //
    // Cones are recorded in `m.drives` in declaration order regardless —
    // this affects only which output's cone sees the richest pool at
    // leaf-selection time, not the SV emission order. See
    // `book/src/construction-strategies.md`.
    let per_output_drive: Vec<NodeId> = match g.cfg.construction_strategy {
        ConstructionStrategy::Sequential | ConstructionStrategy::Shuffled => {
            let build_order: Vec<usize> = match g.cfg.construction_strategy {
                ConstructionStrategy::Sequential => (0..m.outputs.len()).collect(),
                ConstructionStrategy::Shuffled => {
                    let mut idxs: Vec<usize> = (0..m.outputs.len()).collect();
                    idxs.shuffle(&mut g.rng);
                    idxs
                }
                _ => unreachable!(),
            };
            let mut slots: Vec<Option<NodeId>> = vec![None; m.outputs.len()];
            for idx in build_order {
                let out = m.outputs[idx].clone();
                let cone_root = cone::build_cone_with_retry(
                    g,
                    &mut m,
                    &mut pool,
                    &mut worklist,
                    out.width,
                    None,
                );
                slots[idx] = Some(cone_root);
            }
            slots.into_iter().map(|s| s.expect("drive root")).collect()
        }
        ConstructionStrategy::Interleaved | ConstructionStrategy::GraphFirst => {
            // GraphFirst routes to Interleaved — the original speculative
            // pool-growth implementation produced Rule-18-violating
            // orphan gates and has been retired. The `GraphFirst` CLI /
            // config value is kept as a silent alias for backward compat.
            cone::build_outputs_interleaved(g, &mut m, &mut pool, &mut worklist)
        }
    };

    for (idx, root) in per_output_drive.into_iter().enumerate() {
        m.drives.push((m.outputs[idx].id, root));
    }

    // Drain the flop worklist: each pending flop's D-cone is built with
    // the same recursion, possibly enqueuing more flops.
    debug!(
        pending_flops = worklist.len(),
        "drain flop worklist (recursive path)"
    );
    cone::drain_flop_worklist(g, &mut m, &mut pool, &mut worklist);

    finalize_generated_module(g, &mut m, &mut pool);
    m
}

pub(super) fn sample_leaf_interface_profile(g: &mut Generator) -> ModuleInterfaceProfile {
    let n_in = g.rng.gen_range(g.cfg.min_inputs..=g.cfg.max_inputs) as usize;
    let n_out = g.rng.gen_range(g.cfg.min_outputs..=g.cfg.max_outputs) as usize;
    ModuleInterfaceProfile {
        data_input_widths: (0..n_in)
            .map(|_| g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width))
            .collect(),
        output_widths: (0..n_out)
            .map(|_| g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width))
            .collect(),
    }
}

pub(super) fn finalize_generated_module(g: &mut Generator, m: &mut Module, pool: &mut SignalPool) {
    // Flop-mux operand NodeIds are construction-time metadata only:
    // once D has been assembled, emission and validation care about
    // `flop.d`, not the intermediate select/data leaves that happened
    // to build it. Keep the variant shape for metrics/debugging, but
    // discard those operand references before liveness/compaction so
    // metadata-only cones do not survive into emitted SV.
    summarize_flop_mux_metadata(m);

    // Downstream-clean proof pass: revisit already-built cones using
    // the current graph so exact constants and constant-selector muxes
    // do not survive purely because the proof became visible late.
    crate::ir::compact::fold_proven_gates(m);
    crate::ir::compact::flatten_posthoc_associative_gates(m);
    crate::ir::compact::fold_mixed_associative_constants(m);

    // Bounded semantic gate-sharing pass: once every output and flop D
    // cone exists, `identity_mode = node-id` at the live `EGraph`
    // fragment can collapse small-support combinational cones that are
    // proven functionally equal over the same canonical leaf
    // endpoints. Construction strategy is irrelevant here: this is a
    // post-construction identity pass, not a builder.
    let semantic_gates_merged = crate::ir::compact::merge_equivalent_gates(m);
    m.semantic_gates_merged = semantic_gates_merged;
    crate::ir::compact::flatten_posthoc_associative_gates(m);
    crate::ir::compact::fold_mixed_associative_constants(m);

    // Endpoint-preserving sequential sharing pass: once every flop has
    // a concrete D-cone, `identity_mode = node-id` can conservatively
    // merge duplicate state elements whose emitted semantics are the
    // same over the same canonical leaf variables. Today that proof is
    // the same bounded subset as the live `EGraph` fragment:
    // normalized structural proof first, plus a bounded semantic check
    // for small-support cones. Duplicates become dead Q nodes that the
    // compaction pass below removes.
    let flops_merged = crate::ir::compact::merge_equivalent_flops(m);
    m.flops_merged = flops_merged;

    // Sharing/remap can expose new exact cones, so rerun the
    // downstream-clean proof pass once on the settled graph.
    crate::ir::compact::fold_proven_gates(m);
    crate::ir::compact::flatten_posthoc_associative_gates(m);
    crate::ir::compact::fold_mixed_associative_constants(m);

    // Safety-net claimed-set audit (Rule 18): demand-driven
    // construction should leave zero orphan gates, but if the snapshot/
    // rollback in build_cone or the frame machine in build_outputs_
    // interleaved misses a case, this check surfaces it instead of
    // silently emitting the orphan. In release builds the audit is a
    // cheap fanout walk; on violation it logs a warning with the
    // orphan count, then leaves the IR untouched (the emitter would
    // otherwise produce valid SV that validator accepts — the orphan
    // just wastes a wire). Future work may promote this to a hard
    // assertion once the anti-collapse rollback is provably complete
    // for every strategy.
    let orphans_before_compact = count_orphan_gates(m);

    // Output roots must remain functions of primary inputs and/or leaf
    // endpoints rather than collapsing to trivial constants after the
    // late proof-cleanup passes.
    let repaired_constant_drives = repair_constant_output_roots(g, m, pool);

    // Parent-planned exact data interfaces are load-bearing in Phase 4:
    // if a profiled input would otherwise shrink or prune away, make it
    // genuinely live at full width by threading a reduction of that
    // full-width input into an output cone before compaction.
    let repaired_profiled_inputs = repair_profiled_input_coverage(m, pool);

    // NodeId compaction pass: remove any nodes that are unreachable
    // from roots (drives, flop fields). Idempotent — a no-op when
    // the IR is already Rule-18-clean. Exists primarily to let
    // construction-time rewrites (e.g. the `Not(Not(x)) → x`
    // peephole) orphan intermediate gates safely; this pass cleans
    // them up. The count is surfaced via `Metrics::nodes_compacted`
    // for empirical measurement.
    let compacted = crate::ir::compact::compact_node_ids(m);
    m.nodes_compacted = compacted;

    // Post-compaction safety net. Should always be 0 — if compaction
    // leaves an orphan, it's a BFS or holder-enumeration bug in
    // `compact_node_ids`. Keep the warning (not an assertion) so a
    // release build degrades gracefully.
    let orphans = count_orphan_gates(m);
    if orphans > 0 {
        tracing::warn!(
            orphans,
            compacted,
            orphans_before_compact,
            "⚠️ module has orphan gates after compaction — compact_node_ids bug, please report"
        );
    }

    shrink_primary_inputs_to_live_width(m);
    prune_unused_input_ports(m);
    let enforced_profiled_interface = enforce_planned_interface_profile(m, pool);

    info!(
        module = %m.name,
        nodes = m.nodes.len(),
        flops = m.flops.len(),
        semantic_gates_merged,
        flops_merged,
        drives = m.drives.len(),
        orphans,
        compacted,
        repaired_constant_drives,
        repaired_profiled_inputs,
        enforced_profiled_interface,
        "✅ module finalized"
    );
}

/// Flop-mux operand NodeIds are only needed while D is being assembled.
/// After that, keep the variant shape but clear the operand lists so
/// later liveness reasoning matches the emitted hardware rather than
/// construction-time bookkeeping.
fn summarize_flop_mux_metadata(m: &mut Module) {
    use crate::ir::FlopMux;

    for flop in &mut m.flops {
        match &mut flop.mux {
            FlopMux::None => {}
            FlopMux::OneHot(arms) => arms.clear(),
            FlopMux::Encoded { sel, data } => {
                *sel = flop.q;
                data.clear();
            }
        }
    }
}

fn repair_constant_output_roots(g: &mut Generator, m: &mut Module, pool: &mut SignalPool) -> u32 {
    let repairs: Vec<(usize, u32)> = m
        .drives
        .iter()
        .enumerate()
        .filter(|(_, (_, root))| output_root_has_empty_deps(m, *root))
        .map(|(idx, (port, _root))| {
            let width = m
                .outputs
                .iter()
                .find(|out| out.id == *port)
                .expect("drive port must exist in outputs")
                .width;
            (idx, width)
        })
        .collect();

    for (idx, width) in &repairs {
        let replacement = cone::pick_terminal_dep_bearing(g, m, pool, *width, None);
        m.drives[*idx].1 = replacement;
    }

    repairs.len() as u32
}

fn output_root_has_empty_deps(m: &Module, root: NodeId) -> bool {
    match &m.nodes[root as usize] {
        Node::PrimaryInput { .. }
        | Node::FlopQ { .. }
        | Node::MemRead { .. }
        | Node::FsmOut { .. }
        | Node::InstanceOutput { .. } => false,
        Node::Constant { .. } => true,
        Node::Gate { deps, .. } => deps.is_empty(),
    }
}

/// Shrink each surviving primary data input down to the highest bit
/// that any live consumer actually touches. This trims warnings like
/// "bits of signal are not used" on ports that only ever feed low-bit
/// slices. The analysis is conservative: any non-Slice consumer
/// demands the full current width.
fn compute_required_primary_input_widths(m: &Module) -> std::collections::HashMap<PortId, u32> {
    use std::collections::HashMap;

    let mut required: HashMap<PortId, u32> = HashMap::new();
    let mut note_use = |port: PortId, width: u32| {
        required
            .entry(port)
            .and_modify(|w| *w = (*w).max(width))
            .or_insert(width);
    };

    for node in &m.nodes {
        if let Node::Gate { op, operands, .. } = node {
            for (operand_idx, operand) in operands.iter().enumerate() {
                let Node::PrimaryInput { port, width } = &m.nodes[*operand as usize] else {
                    continue;
                };
                let needed = match (op, operand_idx) {
                    (GateOp::Slice { hi, .. }, 0) => hi + 1,
                    _ => *width,
                };
                note_use(*port, needed);
            }
        }
    }

    for (_, root) in &m.drives {
        if let Node::PrimaryInput { port, width } = &m.nodes[*root as usize] {
            note_use(*port, *width);
        }
    }
    for instance in &m.instances {
        for (_, node_id) in &instance.inputs {
            if let Node::PrimaryInput { port, width } = &m.nodes[*node_id as usize] {
                note_use(*port, *width);
            }
        }
    }
    for flop in &m.flops {
        if let Some(d) = flop.d {
            if let Node::PrimaryInput { port, width } = &m.nodes[d as usize] {
                note_use(*port, *width);
            }
        }
    }

    required
}

fn shrink_primary_inputs_to_live_width(m: &mut Module) {
    let required = compute_required_primary_input_widths(m);

    for node in &mut m.nodes {
        if let Node::PrimaryInput { port, width } = node {
            if let Some(new_width) = required.get(port).copied() {
                *width = new_width;
            }
        }
    }
    for input in &mut m.inputs {
        let is_clock = m.clock == Some(input.id);
        let is_reset = m.reset == Some(input.id);
        if is_clock || is_reset {
            continue;
        }
        if let Some(new_width) = required.get(&input.id).copied() {
            input.width = new_width;
        }
    }
}

/// Drop primary data-input ports that no surviving node references.
/// Clock/reset stay declared unconditionally; the emitter already hides
/// them when the module contains no flops.
fn prune_unused_input_ports(m: &mut Module) {
    use std::collections::BTreeSet;

    let live_ports: BTreeSet<_> = m
        .nodes
        .iter()
        .filter_map(|node| match node {
            Node::PrimaryInput { port, .. } => Some(*port),
            _ => None,
        })
        .collect();

    m.inputs.retain(|p| {
        let is_clock = m.clock == Some(p.id);
        let is_reset = m.reset == Some(p.id);
        is_clock || is_reset || live_ports.contains(&p.id)
    });
}

fn repair_profiled_input_coverage(m: &mut Module, pool: &mut SignalPool) -> u32 {
    if m.planned_interface_profile.is_none() || m.drives.is_empty() {
        return 0;
    }

    let required = compute_required_primary_input_widths(m);
    let primary_inputs = m
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| match node {
            Node::PrimaryInput { port, width } => Some((*port, (idx as NodeId, *width))),
            _ => None,
        })
        .collect::<std::collections::HashMap<_, _>>();
    let profiled_ports: Vec<_> = m
        .inputs
        .iter()
        .filter(|input| m.clock != Some(input.id) && m.reset != Some(input.id))
        .map(|input| (input.id, input.width))
        .collect();

    let mut repairs = 0;
    for (repair_idx, (port_id, width)) in profiled_ports.into_iter().enumerate() {
        if required.get(&port_id).copied().unwrap_or(0) >= width {
            continue;
        }

        let Some((src_node, _)) = primary_inputs.get(&port_id).copied() else {
            continue;
        };
        thread_profiled_input_into_output(m, pool, repair_idx, port_id, src_node);
        repairs += 1;
    }

    repairs
}

fn enforce_planned_interface_profile(m: &mut Module, pool: &mut SignalPool) -> u32 {
    let Some(profile) = m.planned_interface_profile.clone() else {
        return 0;
    };
    if m.drives.is_empty() {
        return 0;
    }

    let mut repairs = 0;
    let control_inputs = m
        .inputs
        .iter()
        .take_while(|input| m.clock == Some(input.id) || m.reset == Some(input.id))
        .count();

    for (idx, expected_width) in profile.data_input_widths.iter().copied().enumerate() {
        let expected_name = format!("i_{}", idx);
        let port_id = if let Some(existing_port) = m
            .inputs
            .iter()
            .find(|input| {
                m.clock != Some(input.id)
                    && m.reset != Some(input.id)
                    && input.name == expected_name
            })
            .map(|input| input.id)
        {
            let port = m
                .inputs
                .iter_mut()
                .find(|input| input.id == existing_port)
                .expect("existing profiled data port must still be present");
            if port.width != expected_width {
                port.width = expected_width;
                repairs += 1;
            }
            port.name = expected_name.clone();
            existing_port
        } else {
            let new_port_id = m
                .inputs
                .iter()
                .chain(m.outputs.iter())
                .map(|port| port.id)
                .max()
                .map_or(0, |max_id| max_id + 1);
            m.inputs.insert(
                control_inputs + idx,
                Port {
                    id: new_port_id,
                    name: expected_name,
                    width: expected_width,
                    dir: Direction::In,
                },
            );
            repairs += 1;
            new_port_id
        };

        let src_node = if let Some((node_id, _)) =
            m.nodes
                .iter_mut()
                .enumerate()
                .find_map(|(node_id, node)| match node {
                    Node::PrimaryInput { port, width } if *port == port_id => {
                        if *width != expected_width {
                            *width = expected_width;
                        }
                        Some((node_id as NodeId, *width))
                    }
                    _ => None,
                }) {
            node_id
        } else {
            let node_id = m.nodes.len() as NodeId;
            m.nodes.push(Node::PrimaryInput {
                port: port_id,
                width: expected_width,
            });
            repairs += 1;
            node_id
        };

        if compute_required_primary_input_widths(m)
            .get(&port_id)
            .copied()
            .unwrap_or(0)
            < expected_width
        {
            thread_profiled_input_into_output(m, pool, idx, port_id, src_node);
            repairs += 1;
        }
    }

    let mut control_ports = Vec::new();
    let mut data_ports = Vec::new();
    for input in std::mem::take(&mut m.inputs) {
        if m.clock == Some(input.id) || m.reset == Some(input.id) {
            control_ports.push(input);
        } else {
            data_ports.push(input);
        }
    }
    data_ports.sort_by_key(|input| {
        input
            .name
            .strip_prefix("i_")
            .and_then(|slot| slot.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    control_ports.extend(data_ports);
    m.inputs = control_ports;

    repairs
}

fn thread_profiled_input_into_output(
    m: &mut Module,
    pool: &mut SignalPool,
    repair_idx: usize,
    port_id: PortId,
    src_node: NodeId,
) {
    let src_deps = DepSet::from_port(port_id);
    let (reduced, is_new) = m.intern_gate(GateOp::RedXor, vec![src_node], 1, src_deps.clone());
    if is_new {
        pool.add(reduced, 1, src_deps);
    }

    let drive_idx = repair_idx % m.drives.len();
    let (out_port, root) = m.drives[drive_idx];
    let out_width = m
        .outputs
        .iter()
        .find(|output| output.id == out_port)
        .expect("drive output must exist")
        .width;
    let reduced_deps = cone::node_deps(m, reduced);
    let widened = cone::make_width_adapter(m, pool, reduced, 1, reduced_deps, out_width);
    let deps = DepSet::union(&[&cone::node_deps(m, root), &cone::node_deps(m, widened)]);
    let (mixed, is_new) = m.intern_gate(GateOp::Xor, vec![root, widened], out_width, deps.clone());
    if is_new {
        pool.add(mixed, out_width, deps);
    }
    m.drives[drive_idx].1 = mixed;
}

/// Count gate nodes with no consumer. A consumer is: another gate's
/// operand, a flop's D input, or an output drive root. Primary inputs
/// and constants are allowed to be unused (they don't count as
/// orphans). Used as a Rule-18 safety-net audit at the end of
/// `generate_leaf_module`.
fn count_orphan_gates(m: &Module) -> usize {
    use crate::ir::Node;

    let mut used = vec![false; m.nodes.len()];
    for node in &m.nodes {
        if let Node::Gate { operands, .. } = node {
            for &op in operands {
                used[op as usize] = true;
            }
        }
    }
    for f in &m.flops {
        if let Some(d) = f.d {
            used[d as usize] = true;
        }
    }
    for (_, root) in &m.drives {
        used[*root as usize] = true;
    }
    for instance in &m.instances {
        for (_, node_id) in &instance.inputs {
            used[*node_id as usize] = true;
        }
    }
    m.nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| matches!(n, Node::Gate { .. }) && !used[*i])
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Instance;

    #[test]
    fn shrink_primary_input_trims_unused_high_bits() {
        let mut m = Module::default();
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 29,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 20,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 29 });
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 19, lo: 0 },
            operands: vec![0],
            width: 20,
            deps: DepSet::from_port(0),
        });
        m.drives.push((1, 1));

        shrink_primary_inputs_to_live_width(&mut m);

        assert_eq!(m.inputs[0].width, 20);
        match &m.nodes[0] {
            Node::PrimaryInput { width, .. } => assert_eq!(*width, 20),
            other => panic!("expected primary input, got {other:?}"),
        }
    }

    #[test]
    fn shrink_primary_input_keeps_full_width_for_non_slice_use() {
        let mut m = Module::default();
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 12,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 12,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 2,
            name: "y".into(),
            width: 12,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 12 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 12 });
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![0, 1],
            width: 12,
            deps: DepSet::from_port(0),
        });
        m.drives.push((2, 2));

        shrink_primary_inputs_to_live_width(&mut m);

        assert_eq!(m.inputs[0].width, 12);
        match &m.nodes[0] {
            Node::PrimaryInput { width, .. } => assert_eq!(*width, 12),
            other => panic!("expected primary input, got {other:?}"),
        }
    }

    #[test]
    fn shrink_primary_input_keeps_full_width_for_instance_input_binding() {
        let mut m = Module::default();
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 25,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 3,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 25 });
        m.nodes.push(Node::Gate {
            op: GateOp::Slice { hi: 2, lo: 0 },
            operands: vec![0],
            width: 3,
            deps: DepSet::from_port(0),
        });
        m.drives.push((1, 1));
        m.instances.push(Instance {
            id: 0,
            name: "u_0".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(3, 0)],
            param_bindings: Vec::new(),
        });

        shrink_primary_inputs_to_live_width(&mut m);

        assert_eq!(m.inputs[0].width, 25);
        match &m.nodes[0] {
            Node::PrimaryInput { width, .. } => assert_eq!(*width, 25),
            other => panic!("expected primary input, got {other:?}"),
        }
    }

    #[test]
    fn orphan_gate_count_treats_instance_input_binding_as_consumer() {
        let mut m = Module::default();
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![0],
            width: 8,
            deps: DepSet::from_port(0),
        });
        m.instances.push(Instance {
            id: 0,
            name: "u_0".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(0, 1)],
            param_bindings: Vec::new(),
        });

        assert_eq!(count_orphan_gates(&m), 0);
    }
}
