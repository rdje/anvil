//! Leaf-module generation: N inputs, M outputs, internal cones with
//! optional flops. Single CLK (posedge), single RST_N (async, active-low).

use super::{
    cone::{self, FlopWorklist},
    pool::SignalPool,
    Generator,
};
use crate::config::ConstructionStrategy;
use crate::ir::{DepSet, Direction, Module, Node, NodeId, Port, PortId};
use rand::seq::SliceRandom;
use rand::Rng;
use tracing::{debug, info, instrument};

const CLK_NAME: &str = "clk";
const RST_N_NAME: &str = "rst_n";

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
    let orphans = count_orphan_gates(&m);
    if orphans > 0 {
        tracing::warn!(
            orphans,
            "⚠️ module has orphan gates — Rule 18 residual, please report"
        );
    }

    info!(
        nodes = m.nodes.len(),
        flops = m.flops.len(),
        drives = m.drives.len(),
        orphans,
        "✅ module done"
    );
    m
}

/// Count gate nodes with no consumer. A consumer is: another gate's
/// operand, a flop's D / sel / data / Q reference, or an output drive
/// root. Primary inputs and constants are allowed to be unused (they
/// don't count as orphans). Used as a Rule-18 safety-net audit at the
/// end of `generate_leaf_module`.
fn count_orphan_gates(m: &Module) -> usize {
    use crate::ir::{FlopMux, Node};
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
        match &f.mux {
            FlopMux::Encoded { sel, data } => {
                used[*sel as usize] = true;
                for d in data {
                    used[*d as usize] = true;
                }
            }
            FlopMux::OneHot(arms) => {
                for arm in arms {
                    used[arm.data as usize] = true;
                    used[arm.sel as usize] = true;
                }
            }
            FlopMux::None => {}
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
