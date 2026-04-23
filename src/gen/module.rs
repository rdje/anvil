//! Leaf-module generation: N inputs, M outputs, internal cones with
//! optional flops. Single CLK (posedge), single RST_N (async, active-low).

use super::{
    cone::{self, FlopWorklist},
    pool::SignalPool,
    Generator,
};
use crate::config::ConstructionStrategy;
use crate::ir::{DepSet, Direction, GateOp, Module, Node, NodeId, Port, PortId};
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
    let n_in = g.rng.gen_range(g.cfg.min_inputs..=g.cfg.max_inputs);
    let n_out = g.rng.gen_range(g.cfg.min_outputs..=g.cfg.max_outputs);
    info!(
        n_in,
        n_out,
        strategy = ?g.cfg.construction_strategy,
        "🚀 build module"
    );

    let mut m = Module {
        name: format!("mod_{}_{:04}", g.cfg.seed, index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
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
    for i in 0..n_in {
        let w = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);
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
    for i in 0..n_out {
        let w = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);
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

    // Flop-mux operand NodeIds are construction-time metadata only:
    // once D has been assembled, emission and validation care about
    // `flop.d`, not the intermediate select/data leaves that happened
    // to build it. Keep the variant shape for metrics/debugging, but
    // discard those operand references before liveness/compaction so
    // metadata-only cones do not survive into emitted SV.
    summarize_flop_mux_metadata(&mut m);

    // Downstream-clean proof pass: revisit already-built cones using
    // the current graph so exact constants and constant-selector muxes
    // do not survive purely because the proof became visible late.
    crate::ir::compact::fold_proven_gates(&mut m);
    crate::ir::compact::flatten_posthoc_associative_gates(&mut m);
    crate::ir::compact::fold_mixed_associative_constants(&mut m);

    // Bounded semantic gate-sharing pass: once every output and flop D
    // cone exists, `identity_mode = node-id` at the live `EGraph`
    // fragment can collapse small-support combinational cones that are
    // proven functionally equal over the same canonical leaf
    // endpoints. Construction strategy is irrelevant here: this is a
    // post-construction identity pass, not a builder.
    let semantic_gates_merged = crate::ir::compact::merge_equivalent_gates(&mut m);
    m.semantic_gates_merged = semantic_gates_merged;
    crate::ir::compact::flatten_posthoc_associative_gates(&mut m);
    crate::ir::compact::fold_mixed_associative_constants(&mut m);

    // Endpoint-preserving sequential sharing pass: once every flop has
    // a concrete D-cone, `identity_mode = node-id` can conservatively
    // merge duplicate state elements whose emitted semantics are the
    // same over the same canonical leaf variables. Today that proof is
    // the same bounded subset as the live `EGraph` fragment:
    // normalized structural proof first, plus a bounded semantic check
    // for small-support cones. Duplicates become dead Q nodes that the
    // compaction pass below removes.
    let flops_merged = crate::ir::compact::merge_equivalent_flops(&mut m);
    m.flops_merged = flops_merged;

    // Sharing/remap can expose new exact cones, so rerun the
    // downstream-clean proof pass once on the settled graph.
    crate::ir::compact::fold_proven_gates(&mut m);
    crate::ir::compact::flatten_posthoc_associative_gates(&mut m);
    crate::ir::compact::fold_mixed_associative_constants(&mut m);

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
    let orphans_before_compact = count_orphan_gates(&m);

    // Output roots must remain functions of primary inputs and/or flop-Q
    // leaves rather than collapsing to trivial constants after the late
    // proof-cleanup passes.
    let repaired_constant_drives = repair_constant_output_roots(g, &mut m, &mut pool);

    // NodeId compaction pass: remove any nodes that are unreachable
    // from roots (drives, flop fields). Idempotent — a no-op when
    // the IR is already Rule-18-clean. Exists primarily to let
    // construction-time rewrites (e.g. the `Not(Not(x)) → x`
    // peephole) orphan intermediate gates safely; this pass cleans
    // them up. The count is surfaced via `Metrics::nodes_compacted`
    // for empirical measurement.
    let compacted = crate::ir::compact::compact_node_ids(&mut m);
    m.nodes_compacted = compacted;

    // Post-compaction safety net. Should always be 0 — if compaction
    // leaves an orphan, it's a BFS or holder-enumeration bug in
    // `compact_node_ids`. Keep the warning (not an assertion) so a
    // release build degrades gracefully.
    let orphans = count_orphan_gates(&m);
    if orphans > 0 {
        tracing::warn!(
            orphans,
            compacted,
            orphans_before_compact,
            "⚠️ module has orphan gates after compaction — compact_node_ids bug, please report"
        );
    }

    shrink_primary_inputs_to_live_width(&mut m);
    prune_unused_input_ports(&mut m);

    info!(
        nodes = m.nodes.len(),
        flops = m.flops.len(),
        semantic_gates_merged,
        flops_merged,
        drives = m.drives.len(),
        orphans,
        compacted,
        repaired_constant_drives,
        "✅ module done"
    );
    m
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
        Node::PrimaryInput { .. } | Node::FlopQ { .. } | Node::InstanceOutput { .. } => false,
        Node::Constant { .. } => true,
        Node::Gate { deps, .. } => deps.is_empty(),
    }
}

/// Shrink each surviving primary data input down to the highest bit
/// that any live consumer actually touches. This trims warnings like
/// "bits of signal are not used" on ports that only ever feed low-bit
/// slices. The analysis is conservative: any non-Slice consumer
/// demands the full current width.
fn shrink_primary_inputs_to_live_width(m: &mut Module) {
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
    for flop in &m.flops {
        if let Some(d) = flop.d {
            if let Node::PrimaryInput { port, width } = &m.nodes[d as usize] {
                note_use(*port, *width);
            }
        }
    }

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
    m.nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| matches!(n, Node::Gate { .. }) && !used[*i])
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
