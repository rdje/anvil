//! Phase 4 hierarchy generation.
//!
//! The first live slice is intentionally narrow: depth-1 wrapper
//! hierarchy only. We generate a library of leaf modules, then a real
//! top module that instantiates a planned selection of leaf
//! definitions and builds a real top module above them. The top uses
//! child instance outputs as real leaf variables for its own
//! combinational output cones, so this is genuine composition rather
//! than a fake multi-file bundle, but it still does not recurse beyond
//! that parent-side combinational layer.

use super::{
    cone::{self, FlopWorklist},
    module,
    pool::SignalPool,
    Generator,
};
use crate::config::ConstructionStrategy;
use crate::ir::{
    DepSet, Design, Direction, GateOp, Instance, InstanceId, Module, Node, NodeId, Port, PortId,
};
use rand::seq::SliceRandom;
use rand::Rng;

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
    let instance_plan = plan_child_instance_indices(g, modules.len());

    let top_index = g.next_module_index;
    g.next_module_index += 1;
    let top = generate_wrapper_top(g, top_index, &modules, &instance_plan);
    let top_name = top.name.clone();
    modules.push(top);

    Design {
        top: top_name,
        modules,
    }
}

fn plan_child_instance_indices(g: &mut Generator, library_len: usize) -> Vec<usize> {
    debug_assert!(
        library_len > 0,
        "hierarchy requires at least one leaf module"
    );
    let target = g.cfg.effective_num_child_instances() as usize;
    debug_assert!(
        target > 0,
        "effective child instance count should stay positive under validated hierarchy configs"
    );

    let mut plan = Vec::with_capacity(target);
    if target <= library_len {
        let mut indices: Vec<_> = (0..library_len).collect();
        indices.shuffle(&mut g.rng);
        indices.truncate(target);
        plan.extend(indices);
        return plan;
    }

    let mut indices: Vec<_> = (0..library_len).collect();
    indices.shuffle(&mut g.rng);
    plan.extend(indices);
    while plan.len() < target {
        plan.push(g.rng.gen_range(0..library_len));
    }
    plan
}

fn generate_wrapper_top(
    g: &mut Generator,
    index: u64,
    library: &[Module],
    instance_plan: &[usize],
) -> Module {
    let mut top = Module {
        name: format!("mod_{}_{:04}", g.cfg.seed, index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
        ..Module::default()
    };

    let any_sequential_child = instance_plan
        .iter()
        .any(|&child_idx| library[child_idx].has_local_flops());
    let mut next_port_id: PortId = 0;

    let shared_clock =
        any_sequential_child.then(|| add_top_input(&mut top, &mut next_port_id, "clk", 1));
    let shared_reset =
        any_sequential_child.then(|| add_top_input(&mut top, &mut next_port_id, "rst_n", 1));
    top.clock = shared_clock.map(|(port_id, _)| port_id);
    top.reset = shared_reset.map(|(port_id, _)| port_id);

    let mut instance_pool = SignalPool::new();
    let mut planned_outputs = Vec::new();

    for (instance_idx, child_idx) in instance_plan.iter().copied().enumerate() {
        let child = &library[child_idx];
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
            instance_pool.add(
                node_id,
                child_output.width,
                DepSet::from_instance_output_virtual(instance_id, child_output.id),
            );
            let top_output_id = next_port_id;
            next_port_id += 1;
            top.outputs.push(Port {
                id: top_output_id,
                name: format!("{}__{}", instance_name, child_output.name),
                width: child_output.width,
                dir: Direction::Out,
            });
            planned_outputs.push((top_output_id, child_output.width));
        }
    }

    let mut pool = instance_pool.clone();
    let mut worklist: FlopWorklist = Vec::new();
    let roots = build_parent_output_roots(g, &mut top, &mut pool, &mut worklist);
    debug_assert!(
        worklist.is_empty(),
        "Phase 4 top-level parent composition stays combinational in the current slice"
    );

    for ((port_id, width), root) in planned_outputs.into_iter().zip(roots) {
        let promoted =
            promote_parent_output_root(g, &mut top, &mut pool, &instance_pool, width, root);
        top.drives.push((port_id, promoted));
    }

    module::finalize_generated_module(g, &mut top, &mut pool);
    top
}

fn build_parent_output_roots(
    g: &mut Generator,
    top: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> Vec<NodeId> {
    let saved_flop_prob = g.cfg.flop_prob;
    g.cfg.flop_prob = 0.0;
    let roots = match g.cfg.construction_strategy {
        ConstructionStrategy::Sequential | ConstructionStrategy::Shuffled => {
            let mut build_order: Vec<usize> = (0..top.outputs.len()).collect();
            if matches!(g.cfg.construction_strategy, ConstructionStrategy::Shuffled) {
                build_order.shuffle(&mut g.rng);
            }
            let mut slots = vec![None; top.outputs.len()];
            for idx in build_order {
                let width = top.outputs[idx].width;
                let root = cone::build_cone_with_retry(g, top, pool, worklist, width, None);
                slots[idx] = Some(root);
            }
            slots
                .into_iter()
                .map(|slot| slot.expect("drive root"))
                .collect()
        }
        ConstructionStrategy::Interleaved | ConstructionStrategy::GraphFirst => {
            cone::build_outputs_interleaved(g, top, pool, worklist)
        }
    };
    g.cfg.flop_prob = saved_flop_prob;
    roots
}

fn promote_parent_output_root(
    g: &mut Generator,
    top: &mut Module,
    pool: &mut SignalPool,
    instance_pool: &SignalPool,
    width: u32,
    root: NodeId,
) -> NodeId {
    if matches!(top.nodes[root as usize], Node::Gate { .. }) {
        return root;
    }

    let Some(companion) = try_pick_parent_companion(g, top, instance_pool, width, root) else {
        return root;
    };

    let deps = DepSet::union(&[
        &cone::node_deps(top, root),
        &cone::node_deps(top, companion),
    ]);
    let (node_id, is_new) =
        top.intern_gate(GateOp::Add, vec![root, companion], width, deps.clone());
    if is_new {
        pool.add(node_id, width, deps);
    }
    node_id
}

fn try_pick_parent_companion(
    g: &mut Generator,
    top: &mut Module,
    instance_pool: &SignalPool,
    width: u32,
    exclude: NodeId,
) -> Option<NodeId> {
    if !instance_pool
        .iter()
        .any(|entry| entry.node != exclude && !entry.deps.is_empty())
    {
        return None;
    }

    let mut temp_pool = instance_pool.clone();
    Some(cone::pick_terminal_dep_bearing(
        g,
        top,
        &mut temp_pool,
        width,
        Some(exclude),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Flop, FlopKind, FlopMux, ResetKind};

    fn sequential_leaf(name: &str) -> Module {
        let mut module = Module {
            name: name.to_string(),
            ..Module::default()
        };
        module.clock = Some(0);
        module.reset = Some(1);
        module.inputs.push(Port {
            id: 0,
            name: "clk".into(),
            width: 1,
            dir: Direction::In,
        });
        module.inputs.push(Port {
            id: 1,
            name: "rst_n".into(),
            width: 1,
            dir: Direction::In,
        });
        module.inputs.push(Port {
            id: 2,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        module.outputs.push(Port {
            id: 3,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });
        module.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        module.nodes.push(Node::PrimaryInput { port: 1, width: 1 });
        module.nodes.push(Node::PrimaryInput { port: 2, width: 8 });
        module.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(2),
            q: 3,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        module.nodes.push(Node::FlopQ { flop: 0, width: 8 });
        module.drives.push((3, 3));
        module
    }

    #[test]
    fn wrapper_top_tags_shared_clock_and_reset_ports() {
        let mut g = Generator::new(crate::config::Config {
            seed: 7,
            hierarchy_depth: 1,
            num_leaf_modules: 1,
            ..crate::config::Config::default()
        });
        let child = sequential_leaf("leaf");
        let top = generate_wrapper_top(&mut g, 1, &[child], &[0]);

        let clock = top.clock.expect("wrapper top should tag shared clock");
        let reset = top.reset.expect("wrapper top should tag shared reset");
        assert_eq!(
            top.input_port(clock).map(|port| port.name.as_str()),
            Some("clk")
        );
        assert_eq!(
            top.input_port(reset).map(|port| port.name.as_str()),
            Some("rst_n")
        );
        assert_eq!(top.inputs.len(), 3, "clk + rst_n + one child data input");
    }
}
