//! Leaf-module generation: N inputs, M outputs, internal cones.

use super::{cone, pool::SignalPool, Generator};
use crate::ir::{DepSet, Direction, Module, Node, Port, PortId};
use rand::Rng;

pub fn generate_leaf_module(g: &mut Generator, index: u64) -> Module {
    let n_in = g.rng.gen_range(g.cfg.min_inputs..=g.cfg.max_inputs);
    let n_out = g.rng.gen_range(g.cfg.min_outputs..=g.cfg.max_outputs);

    let mut m = Module {
        name: format!("mod_{}_{:04}", g.cfg.seed, index),
        ..Module::default()
    };

    // Primary inputs
    for i in 0..n_in {
        let w = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);
        let port_id = i as PortId;
        m.inputs.push(Port {
            id: port_id,
            name: format!("i_{}", i),
            width: w,
            dir: Direction::In,
        });
    }

    // Primary outputs
    for i in 0..n_out {
        let w = g.rng.gen_range(g.cfg.min_width..=g.cfg.max_width);
        let port_id = (n_in + i) as PortId;
        m.outputs.push(Port {
            id: port_id,
            name: format!("o_{}", i),
            width: w,
            dir: Direction::Out,
        });
    }

    // Seed signal pool with primary inputs as Node::PrimaryInput entries.
    let mut pool = SignalPool::new();
    for (idx, p) in m.inputs.clone().iter().enumerate() {
        let node_id = m.nodes.len() as u32;
        m.nodes.push(Node::PrimaryInput {
            port: p.id,
            width: p.width,
        });
        pool.add(node_id, p.width, DepSet::from_port(idx as u32));
    }

    // Build an output cone per primary output. Regenerate on trivial.
    for out in m.outputs.clone() {
        let cone_root = cone::build_cone_with_retry(g, &mut m, &mut pool, out.width);
        m.drives.push((out.id, cone_root));
    }

    // Phase 2+: drain flop worklist here.

    m
}
