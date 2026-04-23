//! Phase 4 hierarchy generation.
//!
//! The first live slice is intentionally narrow: depth-1 wrapper
//! hierarchy only. We generate a library of leaf modules, then a real
//! top module that instantiates every leaf and exposes every child
//! output as a top-level output. This is genuine module composition,
//! not a fake multi-file bundle, but it does not yet recurse through
//! parent-side cone construction.

use super::Generator;
use crate::ir::{Design, Direction, Instance, InstanceId, Module, Node, NodeId, Port, PortId};

pub fn generate_design(g: &mut Generator) -> Design {
    debug_assert!(
        g.cfg.hierarchy_depth == 1,
        "config validation should reject hierarchy_depth > 1 for the current Phase 4 slice"
    );
    debug_assert!(
        g.cfg.num_leaf_modules >= 1,
        "config validation should reject hierarchy with zero leaf modules"
    );

    let mut modules = Vec::with_capacity(g.cfg.num_leaf_modules as usize + 1);
    for _ in 0..g.cfg.num_leaf_modules {
        modules.push(g.generate_module());
    }

    let top_index = g.next_module_index;
    g.next_module_index += 1;
    let top = generate_wrapper_top(g.cfg.seed, top_index, &modules);
    let top_name = top.name.clone();
    modules.push(top);

    Design {
        top: top_name,
        modules,
    }
}

fn generate_wrapper_top(seed: u64, index: u64, library: &[Module]) -> Module {
    let mut top = Module {
        name: format!("mod_{}_{:04}", seed, index),
        ..Module::default()
    };

    let any_sequential_child = library.iter().any(Module::has_local_flops);
    let mut next_port_id: PortId = 0;

    let shared_clock =
        any_sequential_child.then(|| add_top_input(&mut top, &mut next_port_id, "clk", 1));
    let shared_reset =
        any_sequential_child.then(|| add_top_input(&mut top, &mut next_port_id, "rst_n", 1));

    for (instance_idx, child) in library.iter().enumerate() {
        let instance_id = top.instances.len() as InstanceId;
        let instance_name = format!("u_{}", instance_idx);
        let mut input_bindings = Vec::new();

        if child.has_local_flops() {
            let (clk_port, clk_node) =
                shared_clock.expect("sequential children require shared clk");
            let (rst_port, rst_node) =
                shared_reset.expect("sequential children require shared rst_n");
            debug_assert_eq!(
                top.input_port(clk_port).map(|port| port.name.as_str()),
                Some("clk")
            );
            debug_assert_eq!(
                top.input_port(rst_port).map(|port| port.name.as_str()),
                Some("rst_n")
            );
            input_bindings.push((child.clock.expect("leaf clock id"), clk_node));
            input_bindings.push((child.reset.expect("leaf reset id"), rst_node));
        }

        for child_input in child.emitted_input_ports() {
            if child.clock == Some(child_input.id) || child.reset == Some(child_input.id) {
                continue;
            }
            let input_name = format!("{}__{}", instance_name, child_input.name);
            let (_, node_id) =
                add_top_input(&mut top, &mut next_port_id, &input_name, child_input.width);
            input_bindings.push((child_input.id, node_id));
        }

        top.instances.push(Instance {
            id: instance_id,
            name: instance_name.clone(),
            module: child.name.clone(),
            inputs: input_bindings,
        });

        for child_output in &child.outputs {
            let node_id = top.nodes.len() as NodeId;
            top.nodes.push(Node::InstanceOutput {
                instance: instance_id,
                port: child_output.id,
                width: child_output.width,
            });
            let top_output_id = next_port_id;
            next_port_id += 1;
            top.outputs.push(Port {
                id: top_output_id,
                name: format!("{}__{}", instance_name, child_output.name),
                width: child_output.width,
                dir: Direction::Out,
            });
            top.drives.push((top_output_id, node_id));
        }
    }

    top
}

fn add_top_input(
    top: &mut Module,
    next_port_id: &mut PortId,
    name: &str,
    width: u32,
) -> (PortId, NodeId) {
    let port_id = *next_port_id;
    *next_port_id += 1;
    top.inputs.push(Port {
        id: port_id,
        name: name.into(),
        width,
        dir: Direction::In,
    });
    let node_id = top.nodes.len() as NodeId;
    top.nodes.push(Node::PrimaryInput {
        port: port_id,
        width,
    });
    (port_id, node_id)
}
