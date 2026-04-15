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

const CLK_NAME: &str = "clk";
const RST_N_NAME: &str = "rst_n";

pub fn generate_leaf_module(g: &mut Generator, index: u64) -> Module {
    let n_in = g.rng.gen_range(g.cfg.min_inputs..=g.cfg.max_inputs);
    let n_out = g.rng.gen_range(g.cfg.min_outputs..=g.cfg.max_outputs);

    let mut m = Module {
        name: format!("mod_{}_{:04}", g.cfg.seed, index),
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

    // Build an output cone per primary output. The iteration order over
    // outputs is governed by `cfg.construction_strategy`:
    //
    // - `Sequential`: declaration order (0, 1, ..., n_out-1).
    // - `Shuffled`: a random permutation drawn from the seeded RNG.
    //
    // Cones are recorded in `m.drives` in declaration order regardless —
    // this affects only which output's cone sees the richest pool at
    // leaf-selection time, not the emission order. See
    // `book/src/construction-strategies.md`.
    let build_order: Vec<usize> = match g.cfg.construction_strategy {
        ConstructionStrategy::Sequential => (0..m.outputs.len()).collect(),
        ConstructionStrategy::Shuffled => {
            let mut idxs: Vec<usize> = (0..m.outputs.len()).collect();
            idxs.shuffle(&mut g.rng);
            idxs
        }
    };

    let mut per_output_drive: Vec<Option<NodeId>> = vec![None; m.outputs.len()];
    for idx in build_order {
        let out = m.outputs[idx].clone();
        let cone_root =
            cone::build_cone_with_retry(g, &mut m, &mut pool, &mut worklist, out.width, None);
        per_output_drive[idx] = Some(cone_root);
    }
    for (idx, root) in per_output_drive.into_iter().enumerate() {
        m.drives.push((
            m.outputs[idx].id,
            root.expect("every output must have a drive root"),
        ));
    }

    // Drain the flop worklist: each pending flop's D-cone is built with
    // the same recursion, possibly enqueuing more flops.
    cone::drain_flop_worklist(g, &mut m, &mut pool, &mut worklist);

    m
}
