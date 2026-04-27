//! Phase 4 hierarchy generation.
//!
//! The current Phase 4 slice has two hierarchy planning modes:
//!
//! - the legacy exact depth-1 wrapper mode (`hierarchy_depth = 1`)
//! - the newer bounded recursive mode
//!   (`min_hierarchy_depth..=max_hierarchy_depth`,
//!   `min_child_instances_per_module..=max_child_instances_per_module`,
//!   plus optional per-parent-depth branching overrides)
//!
//! In both cases, parent modules use child instance outputs as real
//! leaf variables for their own parent-side cones, so this is genuine
//! composition rather than a fake multi-file bundle. Parent-local
//! flops are controlled by `hierarchy_parent_flop_prob`.

use super::{
    cone::{self, FlopWorklist},
    module,
    pool::SignalPool,
    Generator,
};
use crate::config::{ConstructionStrategy, HierarchyChildSourceMode};
use crate::ir::{
    DepSet, Design, Direction, Flop, FlopId, FlopKind, FlopMux, GateOp, Instance, InstanceId,
    InstanceRole, KnobId, Module, ModuleInterfaceProfile, Node, NodeId, Port, PortId, ResetKind,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::BTreeMap;

struct BuiltSubtree {
    modules: Vec<Module>,
}

pub fn generate_design(g: &mut Generator) -> Design {
    if g.cfg.uses_hierarchy_range_mode() {
        generate_recursive_design(g)
    } else {
        generate_legacy_exact_design(g)
    }
}

fn generate_legacy_exact_design(g: &mut Generator) -> Design {
    debug_assert!(
        g.cfg.hierarchy_depth == 1,
        "legacy hierarchy mode expects exact depth-1 wrapper planning"
    );
    let exact_instances = g.cfg.effective_num_child_instances() as usize;
    let (mut modules, instance_plan) = match g.cfg.hierarchy_child_source_mode {
        HierarchyChildSourceMode::Library => {
            let mut modules = Vec::with_capacity(g.cfg.num_leaf_modules as usize + 1);
            for _ in 0..g.cfg.num_leaf_modules {
                modules.push(g.generate_module());
            }
            let instance_plan = plan_child_instance_indices(g, modules.len(), exact_instances);
            (modules, instance_plan)
        }
        HierarchyChildSourceMode::OnDemand => {
            let mut modules = Vec::with_capacity(exact_instances + 1);
            for _ in 0..exact_instances {
                let profile = module::sample_leaf_interface_profile(g);
                modules.push(g.generate_module_with_interface_profile(Some(&profile)));
            }
            let instance_plan = (0..exact_instances).collect();
            (modules, instance_plan)
        }
    };

    let top_index = g.reserve_module_index();
    let top = generate_parent_module(g, top_index, &modules, &[], &instance_plan, None);
    let top_name = top.name.clone();
    modules.push(top);

    Design {
        top: top_name,
        modules,
    }
}

fn generate_recursive_design(g: &mut Generator) -> Design {
    let (min_depth, max_depth) = g
        .cfg
        .effective_hierarchy_depth_range()
        .expect("hierarchy range mode should have an effective depth range");
    let built = build_recursive_subtree(g, 0, min_depth, max_depth, None);
    let top_name = built
        .modules
        .last()
        .expect("hierarchy subtree must produce a root")
        .name
        .clone();
    Design {
        top: top_name,
        modules: built.modules,
    }
}

fn build_recursive_subtree(
    g: &mut Generator,
    parent_depth: u32,
    min_remaining_depth: u32,
    max_remaining_depth: u32,
    demanded_profile: Option<ModuleInterfaceProfile>,
) -> BuiltSubtree {
    debug_assert!(
        min_remaining_depth <= max_remaining_depth,
        "recursive hierarchy ranges must stay ordered"
    );

    if max_remaining_depth == 0 {
        debug_assert_eq!(
            min_remaining_depth, 0,
            "a zero max remaining depth implies an exact leaf"
        );
        return BuiltSubtree {
            modules: vec![g.generate_module_with_interface_profile(demanded_profile.as_ref())],
        };
    }

    let (min_instances, max_instances) = g
        .cfg
        .effective_child_instance_range_for_parent_depth(parent_depth)
        .expect("recursive hierarchy requires child instance bounds");
    let target_instances = if min_instances == max_instances {
        min_instances as usize
    } else {
        g.rng
            .gen_range(min_instances as usize..=max_instances as usize)
    };

    if min_remaining_depth == 0 && target_instances == 1 {
        let chosen_depth = g.rng.gen_range(0..=max_remaining_depth);
        if chosen_depth == 0 {
            return BuiltSubtree {
                modules: vec![g.generate_module_with_interface_profile(demanded_profile.as_ref())],
            };
        }
        return build_recursive_subtree(
            g,
            parent_depth,
            chosen_depth,
            chosen_depth,
            demanded_profile,
        );
    }

    let child_min_depth = min_remaining_depth.saturating_sub(1);
    let child_max_depth = max_remaining_depth - 1;
    let force_mixed_children = child_min_depth < child_max_depth && target_instances >= 2;
    let (child_definition_count, instance_plan) = match g.cfg.hierarchy_child_source_mode {
        HierarchyChildSourceMode::Library => {
            let library_len = plan_child_library_len(g, target_instances, force_mixed_children);
            let instance_plan = plan_child_instance_indices(g, library_len, target_instances);
            (library_len, instance_plan)
        }
        HierarchyChildSourceMode::OnDemand => {
            let instance_plan = (0..target_instances).collect();
            (target_instances, instance_plan)
        }
    };
    let child_depth_ranges =
        plan_child_depth_ranges(g, child_definition_count, child_min_depth, child_max_depth);

    let child_profiles = if g.cfg.hierarchy_child_source_mode == HierarchyChildSourceMode::OnDemand
    {
        Some(
            (0..child_definition_count)
                .map(|_| module::sample_leaf_interface_profile(g))
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let mut descendant_modules = Vec::new();
    let mut direct_children = Vec::with_capacity(child_definition_count);
    for (idx, (child_min, child_max)) in child_depth_ranges.into_iter().enumerate() {
        let demanded_child_profile = child_profiles
            .as_ref()
            .map(|profiles| profiles[idx].clone());
        let mut child = build_recursive_subtree(
            g,
            parent_depth + 1,
            child_min,
            child_max,
            demanded_child_profile,
        )
        .modules;
        let child_root = child
            .pop()
            .expect("recursive child subtree should end with its root");
        descendant_modules.extend(child);
        direct_children.push(child_root);
    }

    let parent_index = g.reserve_module_index();
    let parent = generate_parent_module(
        g,
        parent_index,
        &direct_children,
        &descendant_modules,
        &instance_plan,
        demanded_profile.as_ref(),
    );

    descendant_modules.extend(direct_children);
    descendant_modules.push(parent);
    BuiltSubtree {
        modules: descendant_modules,
    }
}

fn plan_child_library_len(
    g: &mut Generator,
    target_instances: usize,
    force_mixed_children: bool,
) -> usize {
    debug_assert!(target_instances >= 1);
    let min_library_len = if force_mixed_children { 2 } else { 1 };
    debug_assert!(
        min_library_len <= target_instances,
        "mixed child planning requires at least two instance slots"
    );
    if min_library_len == target_instances {
        target_instances
    } else {
        g.rng.gen_range(min_library_len..=target_instances)
    }
}

fn plan_child_depth_ranges(
    g: &mut Generator,
    library_len: usize,
    child_min_depth: u32,
    child_max_depth: u32,
) -> Vec<(u32, u32)> {
    debug_assert!(library_len >= 1);
    debug_assert!(child_min_depth <= child_max_depth);

    if library_len == 1 {
        return vec![(child_min_depth, child_max_depth)];
    }

    let mut ranges = Vec::with_capacity(library_len);
    ranges.push((child_min_depth, child_min_depth));
    ranges.push((child_max_depth, child_max_depth));

    for _ in 2..library_len {
        if child_min_depth == child_max_depth {
            ranges.push((child_min_depth, child_max_depth));
            continue;
        }

        if g.rng.gen_bool(0.5) {
            let exact_depth = g.rng.gen_range(child_min_depth..=child_max_depth);
            ranges.push((exact_depth, exact_depth));
        } else {
            ranges.push((child_min_depth, child_max_depth));
        }
    }

    ranges
}

fn plan_child_instance_indices(
    g: &mut Generator,
    library_len: usize,
    target_instances: usize,
) -> Vec<usize> {
    debug_assert!(
        library_len > 0,
        "hierarchy requires at least one child module"
    );
    debug_assert!(
        target_instances > 0,
        "child instance count should stay positive under validated hierarchy configs"
    );

    let mut plan = Vec::with_capacity(target_instances);
    if target_instances <= library_len {
        let mut indices: Vec<_> = (0..library_len).collect();
        indices.shuffle(&mut g.rng);
        indices.truncate(target_instances);
        plan.extend(indices);
        return plan;
    }

    let mut indices: Vec<_> = (0..library_len).collect();
    indices.shuffle(&mut g.rng);
    plan.extend(indices);
    while plan.len() < target_instances {
        plan.push(g.rng.gen_range(0..library_len));
    }
    plan
}

fn generate_parent_module(
    g: &mut Generator,
    index: u64,
    library: &[Module],
    descendants: &[Module],
    instance_plan: &[usize],
    external_profile: Option<&ModuleInterfaceProfile>,
) -> Module {
    let mut modules_by_name = BTreeMap::new();
    for module in descendants {
        modules_by_name.insert(module.name.as_str(), module);
    }
    for module in library {
        modules_by_name.insert(module.name.as_str(), module);
    }

    let mut top = Module {
        name: g.module_name(index),
        max_ast_instances: g.cfg.max_ast_instances.max(1),
        mux_arm_duplication_rate: g.cfg.mux_arm_duplication_rate.clamp(0.0, 1.0),
        operand_duplication_rate: g.cfg.operand_duplication_rate.clamp(0.0, 1.0),
        identity_mode: g.cfg.identity_mode,
        factorization_level: g.cfg.factorization_level,
        planned_interface_profile: external_profile.cloned(),
        ..Module::default()
    };

    let any_sequential_child = instance_plan
        .iter()
        .any(|&child_idx| library[child_idx].carries_sequential_state_in(Some(&modules_by_name)));
    let parent_state_possible = g.cfg.max_flops_per_module > 0
        && (g.cfg.hierarchy_parent_flop_prob > 0.0
            || g.cfg.hierarchy_registered_sibling_route_prob > 0.0
            || g.cfg.hierarchy_registered_child_input_cone_prob > 0.0);
    let needs_control_ports = any_sequential_child || parent_state_possible;
    let mut next_port_id: PortId = 0;

    let shared_clock =
        needs_control_ports.then(|| add_top_input(&mut top, &mut next_port_id, "clk", 1));
    let shared_reset =
        needs_control_ports.then(|| add_top_input(&mut top, &mut next_port_id, "rst_n", 1));
    top.clock = shared_clock.map(|(port_id, _)| port_id);
    top.reset = shared_reset.map(|(port_id, _)| port_id);

    let mut external_input_pool = SignalPool::new();
    let mut parent_source_pool = SignalPool::new();
    let mut planned_outputs = Vec::new();
    if let Some(profile) = external_profile {
        for (idx, width) in profile.data_input_widths.iter().copied().enumerate() {
            let (port_id, node_id) =
                add_top_input(&mut top, &mut next_port_id, &format!("i_{}", idx), width);
            let deps = DepSet::from_port(port_id);
            external_input_pool.add(node_id, width, deps.clone());
            parent_source_pool.add(node_id, width, deps);
        }
        for (idx, width) in profile.output_widths.iter().copied().enumerate() {
            let port_id = next_port_id;
            next_port_id += 1;
            top.outputs.push(Port {
                id: port_id,
                name: format!("o_{}", idx),
                width,
                dir: Direction::Out,
            });
            planned_outputs.push((port_id, width));
        }
    }

    let mut instance_pool = SignalPool::new();
    let mut parent_cone_instances_inserted = 0u32;

    for (instance_idx, child_idx) in instance_plan.iter().copied().enumerate() {
        let child = &library[child_idx];
        let instance_name = format!("u_{}", instance_idx);
        let mut input_bindings = Vec::new();

        if child.carries_sequential_state_in(Some(&modules_by_name)) {
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

        let mut binding_ctx = ChildInputBindingContext {
            top: &mut top,
            instance_pool: &mut instance_pool,
            external_input_pool: &mut external_input_pool,
            parent_source_pool: &mut parent_source_pool,
            next_port_id: &mut next_port_id,
            library,
            modules_by_name: &modules_by_name,
            helper_child_indices: instance_plan,
            shared_clock,
            shared_reset,
            allow_external_pool_reuse: external_profile.is_some(),
            allow_parent_cone_instance_input_sources: true,
            parent_cone_instances_inserted: &mut parent_cone_instances_inserted,
        };
        for child_input in child.emitted_data_input_ports_in(Some(&modules_by_name)) {
            let node_id = bind_child_input_from_parent_sources(
                g,
                &mut binding_ctx,
                &instance_name,
                &child_input.name,
                child_input.width,
                external_profile.is_some(),
            );
            input_bindings.push((child_input.id, node_id));
        }
        let ChildInputBindingContext {
            top,
            instance_pool,
            external_input_pool: _,
            parent_source_pool,
            next_port_id,
            library: _,
            modules_by_name: _,
            helper_child_indices: _,
            shared_clock: _,
            shared_reset: _,
            allow_external_pool_reuse: _,
            allow_parent_cone_instance_input_sources: _,
            parent_cone_instances_inserted: _,
        } = binding_ctx;

        let instance_id = top.instances.len() as InstanceId;
        top.instances.push(Instance {
            id: instance_id,
            name: instance_name.clone(),
            module: child.name.clone(),
            role: InstanceRole::PlannedChild,
            inputs: input_bindings,
        });

        for child_output in &child.outputs {
            let node_id = top.nodes.len() as NodeId;
            top.nodes.push(Node::InstanceOutput {
                instance: instance_id,
                port: child_output.id,
                width: child_output.width,
            });
            let deps = DepSet::from_instance_output_virtual(instance_id, child_output.id);
            instance_pool.add(node_id, child_output.width, deps.clone());
            parent_source_pool.add(node_id, child_output.width, deps);
            if external_profile.is_none() {
                let top_output_id = *next_port_id;
                *next_port_id += 1;
                top.outputs.push(Port {
                    id: top_output_id,
                    name: format!("{}__{}", instance_name, child_output.name),
                    width: child_output.width,
                    dir: Direction::Out,
                });
                planned_outputs.push((top_output_id, child_output.width));
            }
        }
    }

    let parent_output_helper_sources = collect_parent_output_helper_sources(
        g,
        &mut top,
        &mut instance_pool,
        &mut external_input_pool,
        &mut parent_source_pool,
        &mut next_port_id,
        library,
        &modules_by_name,
        instance_plan,
        shared_clock,
        shared_reset,
        external_profile.is_some(),
        &mut parent_cone_instances_inserted,
        &planned_outputs,
    );

    let mut pool = parent_source_pool.clone();
    let mut worklist: FlopWorklist = Vec::new();
    let roots = build_parent_output_roots(g, &mut top, &mut pool, &mut worklist);
    debug_assert!(worklist.is_empty(), "parent output flop worklist drained");
    for (output_idx, ((port_id, width), root)) in planned_outputs.into_iter().zip(roots).enumerate()
    {
        let mut promotion_ctx = ParentOutputPromotionContext {
            pool: &mut pool,
            instance_pool: &instance_pool,
            parent_source_pool: &parent_source_pool,
            required_parent_cone_instance_source: pick_parent_output_helper_source(
                &parent_output_helper_sources,
                output_idx,
            ),
        };
        let promoted = promote_parent_output_root(g, &mut top, &mut promotion_ctx, width, root);
        top.drives.push((port_id, promoted));
    }

    let mut finalize_pool = hierarchy_parent_finalize_pool(&pool);
    module::finalize_generated_module(g, &mut top, &mut finalize_pool);
    repair_parent_output_roots_after_finalize(g, &mut top);
    top
}

fn hierarchy_parent_finalize_pool(pool: &SignalPool) -> SignalPool {
    let mut out = SignalPool::new();
    for entry in pool
        .iter()
        .filter(|entry| entry.deps.has_instance_output_virtuals())
    {
        out.add(entry.node, entry.width, entry.deps.clone());
    }

    if out.is_empty() {
        pool.clone()
    } else {
        out
    }
}

fn repair_parent_output_roots_after_finalize(g: &mut Generator, top: &mut Module) {
    let instance_pool = current_instance_output_pool(top);
    let parent_port_pool = current_parent_port_pool(top);
    if instance_pool.is_empty() {
        return;
    }

    let mut scratch_pool = SignalPool::new();
    for drive_idx in 0..top.drives.len() {
        let (port_id, root) = top.drives[drive_idx];
        let width = top
            .outputs
            .iter()
            .find(|output| output.id == port_id)
            .expect("drive output must exist")
            .width;

        let with_child_support = if output_root_reaches_instance_output(top, root) {
            root
        } else {
            let Some(companion) = try_pick_parent_companion(g, top, &instance_pool, width, root)
            else {
                continue;
            };
            add_parent_companion_gate(top, &mut scratch_pool, width, root, companion)
        };

        let with_parent_port_support = if cone::node_deps(top, with_child_support).has_ports()
            || parent_port_pool.is_empty()
        {
            with_child_support
        } else {
            match try_pick_parent_port_companion(
                g,
                top,
                &parent_port_pool,
                width,
                with_child_support,
            ) {
                Some(companion) => add_parent_companion_gate(
                    top,
                    &mut scratch_pool,
                    width,
                    with_child_support,
                    companion,
                ),
                None => with_child_support,
            }
        };

        top.drives[drive_idx].1 = with_parent_port_support;
    }
}

fn current_instance_output_pool(top: &Module) -> SignalPool {
    let mut pool = SignalPool::new();
    for (node_id, node) in top.nodes.iter().enumerate() {
        let Node::InstanceOutput {
            instance,
            port,
            width,
        } = node
        else {
            continue;
        };
        pool.add(
            node_id as NodeId,
            *width,
            DepSet::from_instance_output_virtual(*instance, *port),
        );
    }
    pool
}

fn current_parent_port_pool(top: &Module) -> SignalPool {
    let mut pool = SignalPool::new();
    for (node_id, node) in top.nodes.iter().enumerate() {
        let Node::PrimaryInput { port, width } = node else {
            continue;
        };
        if top.clock == Some(*port) || top.reset == Some(*port) {
            continue;
        }
        pool.add(node_id as NodeId, *width, DepSet::from_port(*port));
    }
    pool
}

fn pick_parent_cone_instance_source(
    top: &Module,
    parent_source_pool: &SignalPool,
    target_width: u32,
) -> Option<NodeId> {
    parent_source_pool
        .iter()
        .filter(|entry| node_is_parent_cone_instance_output(top, entry.node))
        .find(|entry| entry.width == target_width)
        .map(|entry| entry.node)
        .or_else(|| {
            parent_source_pool
                .iter()
                .find(|entry| node_is_parent_cone_instance_output(top, entry.node))
                .map(|entry| entry.node)
        })
}

#[allow(clippy::too_many_arguments)]
fn collect_parent_output_helper_sources(
    g: &mut Generator,
    top: &mut Module,
    instance_pool: &mut SignalPool,
    external_input_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    next_port_id: &mut PortId,
    library: &[Module],
    modules_by_name: &BTreeMap<&str, &Module>,
    instance_plan: &[usize],
    shared_clock: Option<(PortId, NodeId)>,
    shared_reset: Option<(PortId, NodeId)>,
    allow_external_pool_reuse: bool,
    parent_cone_instances_inserted: &mut u32,
    planned_outputs: &[(PortId, u32)],
) -> Vec<NodeId> {
    let mut sources = Vec::new();
    if planned_outputs.is_empty() || g.cfg.hierarchy_parent_cone_instance_prob <= 0.0 {
        return sources;
    }

    for (_, width) in planned_outputs {
        if *parent_cone_instances_inserted < g.cfg.max_parent_cone_instances_per_module {
            let source = {
                let mut helper_ctx = ChildInputBindingContext {
                    top,
                    instance_pool,
                    external_input_pool,
                    parent_source_pool,
                    next_port_id,
                    library,
                    modules_by_name,
                    helper_child_indices: instance_plan,
                    shared_clock,
                    shared_reset,
                    allow_external_pool_reuse,
                    allow_parent_cone_instance_input_sources: false,
                    parent_cone_instances_inserted,
                };
                maybe_add_parent_cone_instance_source(g, &mut helper_ctx, *width)
            };
            if let Some(source) = source {
                sources.push(source);
                continue;
            }
        }

        if sources.is_empty() {
            if let Some(source) = pick_parent_cone_instance_source(top, parent_source_pool, *width)
            {
                sources.push(source);
            }
        }
    }

    sources
}

fn pick_parent_output_helper_source(sources: &[NodeId], output_idx: usize) -> Option<NodeId> {
    if sources.is_empty() {
        None
    } else {
        Some(sources[output_idx % sources.len()])
    }
}

fn node_is_parent_cone_instance_output(top: &Module, node_id: NodeId) -> bool {
    let Node::InstanceOutput { instance, .. } = top.nodes[node_id as usize] else {
        return false;
    };
    top.instances
        .get(instance as usize)
        .is_some_and(|inst| inst.role == InstanceRole::ParentCone)
}

fn output_root_reaches_instance_output(top: &Module, root: NodeId) -> bool {
    let mut memo = vec![None; top.nodes.len()];
    node_reaches_instance_output(top, root, &mut memo)
}

fn node_reaches_instance_output(top: &Module, node_id: NodeId, memo: &mut [Option<bool>]) -> bool {
    if let Some(reaches) = memo[node_id as usize] {
        return reaches;
    }

    let reaches = match &top.nodes[node_id as usize] {
        Node::InstanceOutput { .. } => true,
        Node::Gate { operands, .. } => operands
            .iter()
            .any(|operand| node_reaches_instance_output(top, *operand, memo)),
        _ => false,
    };
    memo[node_id as usize] = Some(reaches);
    reaches
}

struct ChildInputBindingContext<'a> {
    top: &'a mut Module,
    instance_pool: &'a mut SignalPool,
    external_input_pool: &'a mut SignalPool,
    parent_source_pool: &'a mut SignalPool,
    next_port_id: &'a mut PortId,
    library: &'a [Module],
    modules_by_name: &'a BTreeMap<&'a str, &'a Module>,
    helper_child_indices: &'a [usize],
    shared_clock: Option<(PortId, NodeId)>,
    shared_reset: Option<(PortId, NodeId)>,
    allow_external_pool_reuse: bool,
    allow_parent_cone_instance_input_sources: bool,
    parent_cone_instances_inserted: &'a mut u32,
}

fn bind_child_input_from_parent_sources(
    g: &mut Generator,
    ctx: &mut ChildInputBindingContext<'_>,
    instance_name: &str,
    child_input_name: &str,
    width: u32,
    allow_external_pool_reuse: bool,
) -> NodeId {
    if ctx
        .parent_source_pool
        .iter()
        .any(|entry| entry.deps.has_instance_output_virtuals())
        && (ctx.top.flops.len() as u32) < g.cfg.max_flops_per_module
        && roll_hierarchy_registered_child_input_cone(
            ctx.top,
            &mut g.rng,
            g.cfg.hierarchy_registered_child_input_cone_prob,
        )
    {
        let parent_cone_instance_source = maybe_add_parent_cone_instance_source(g, ctx, width);
        return build_registered_child_input_cone_route(
            g,
            ctx.top,
            ctx.parent_source_pool,
            width,
            parent_cone_instance_source,
        );
    }

    if ctx.instance_pool.iter().any(|entry| !entry.deps.is_empty())
        && (ctx.top.flops.len() as u32) < g.cfg.max_flops_per_module
        && roll_hierarchy_registered_sibling_route(
            ctx.top,
            &mut g.rng,
            g.cfg.hierarchy_registered_sibling_route_prob,
        )
    {
        return build_registered_sibling_route(
            g,
            ctx.top,
            ctx.instance_pool,
            ctx.parent_source_pool,
            width,
        );
    }

    if ctx
        .parent_source_pool
        .iter()
        .any(|entry| !entry.deps.is_empty())
        && roll_hierarchy_child_input_cone(
            ctx.top,
            &mut g.rng,
            g.cfg.hierarchy_child_input_cone_prob,
        )
    {
        let parent_cone_instance_source = maybe_add_parent_cone_instance_source(g, ctx, width);
        return build_child_input_parent_cone(
            g,
            ctx.top,
            ctx.parent_source_pool,
            width,
            parent_cone_instance_source,
        );
    }

    if ctx.instance_pool.iter().any(|entry| !entry.deps.is_empty())
        && roll_hierarchy_sibling_route(ctx.top, &mut g.rng, g.cfg.hierarchy_sibling_route_prob)
    {
        return cone::pick_terminal_dep_bearing(g, ctx.top, ctx.instance_pool, width, None);
    }

    if allow_external_pool_reuse
        && ctx
            .external_input_pool
            .iter()
            .any(|entry| !entry.deps.is_empty())
    {
        return cone::pick_terminal_dep_bearing(g, ctx.top, ctx.external_input_pool, width, None);
    }

    let input_name = format!("{instance_name}__{child_input_name}");
    let (port_id, node_id) = add_top_input(ctx.top, ctx.next_port_id, &input_name, width);
    let deps = DepSet::from_port(port_id);
    ctx.external_input_pool.add(node_id, width, deps.clone());
    ctx.parent_source_pool.add(node_id, width, deps);
    node_id
}

fn roll_hierarchy_sibling_route(m: &mut Module, rng: &mut impl Rng, prob: f64) -> bool {
    let fired = rng.gen_bool(prob);
    m.knob_rolls
        .record(KnobId::HierarchySiblingRouteProb, fired);
    fired
}

fn roll_hierarchy_registered_sibling_route(m: &mut Module, rng: &mut impl Rng, prob: f64) -> bool {
    let fired = rng.gen_bool(prob);
    m.knob_rolls
        .record(KnobId::HierarchyRegisteredSiblingRouteProb, fired);
    fired
}

fn roll_hierarchy_registered_child_input_cone(
    m: &mut Module,
    rng: &mut impl Rng,
    prob: f64,
) -> bool {
    let fired = rng.gen_bool(prob);
    m.knob_rolls
        .record(KnobId::HierarchyRegisteredChildInputConeProb, fired);
    fired
}

fn roll_hierarchy_child_input_cone(m: &mut Module, rng: &mut impl Rng, prob: f64) -> bool {
    let fired = rng.gen_bool(prob);
    m.knob_rolls
        .record(KnobId::HierarchyChildInputConeProb, fired);
    fired
}

fn roll_hierarchy_parent_cone_instance(m: &mut Module, rng: &mut impl Rng, prob: f64) -> bool {
    let fired = rng.gen_bool(prob);
    m.knob_rolls
        .record(KnobId::HierarchyParentConeInstanceProb, fired);
    fired
}

fn maybe_add_parent_cone_instance_source(
    g: &mut Generator,
    ctx: &mut ChildInputBindingContext<'_>,
    target_width: u32,
) -> Option<NodeId> {
    if *ctx.parent_cone_instances_inserted >= g.cfg.max_parent_cone_instances_per_module
        || !roll_hierarchy_parent_cone_instance(
            ctx.top,
            &mut g.rng,
            g.cfg.hierarchy_parent_cone_instance_prob,
        )
    {
        return None;
    }

    let candidates: Vec<usize> = ctx
        .helper_child_indices
        .iter()
        .copied()
        .filter(|&idx| ctx.library[idx].outputs.iter().any(|port| port.width > 0))
        .collect();
    if candidates.is_empty() {
        return None;
    }

    let child_idx = candidates[g.rng.gen_range(0..candidates.len())];
    let child = &ctx.library[child_idx];
    let child_name = child.name.clone();
    let child_outputs = child.outputs.clone();
    let child_data_inputs: Vec<_> = child
        .emitted_data_input_ports_in(Some(ctx.modules_by_name))
        .cloned()
        .collect();
    let child_clock = child.clock;
    let child_reset = child.reset;
    let child_is_sequential = child.carries_sequential_state_in(Some(ctx.modules_by_name));
    let instance_id = ctx.top.instances.len() as InstanceId;
    let instance_name = format!("pc_{}", instance_id);
    let mut input_bindings = Vec::new();

    if child_is_sequential {
        let (_, clk_node) = ctx
            .shared_clock
            .expect("parent-cone instance of sequential child requires shared clk");
        let (_, rst_node) = ctx
            .shared_reset
            .expect("parent-cone instance of sequential child requires shared rst_n");
        input_bindings.push((child_clock.expect("child clock id"), clk_node));
        input_bindings.push((child_reset.expect("child reset id"), rst_node));
    }

    for child_input in child_data_inputs {
        let node_id = bind_parent_cone_instance_input(
            g,
            ctx,
            &instance_name,
            &child_input.name,
            child_input.width,
        );
        input_bindings.push((child_input.id, node_id));
    }

    ctx.top.instances.push(Instance {
        id: instance_id,
        name: instance_name,
        module: child_name,
        role: InstanceRole::ParentCone,
        inputs: input_bindings,
    });
    *ctx.parent_cone_instances_inserted += 1;

    let mut helper_outputs = Vec::new();
    for child_output in &child_outputs {
        let node_id = ctx.top.nodes.len() as NodeId;
        ctx.top.nodes.push(Node::InstanceOutput {
            instance: instance_id,
            port: child_output.id,
            width: child_output.width,
        });
        let deps = DepSet::from_instance_output_virtual(instance_id, child_output.id);
        ctx.instance_pool
            .add(node_id, child_output.width, deps.clone());
        ctx.parent_source_pool
            .add(node_id, child_output.width, deps.clone());
        helper_outputs.push((node_id, child_output.width));
    }

    helper_outputs
        .iter()
        .copied()
        .find(|(_, width)| *width == target_width)
        .map(|(node, _)| node)
        .or_else(|| helper_outputs.first().map(|(node, _)| *node))
}

fn bind_parent_cone_instance_input(
    g: &mut Generator,
    ctx: &mut ChildInputBindingContext<'_>,
    instance_name: &str,
    child_input_name: &str,
    width: u32,
) -> NodeId {
    if ctx.allow_parent_cone_instance_input_sources {
        if ctx
            .parent_source_pool
            .iter()
            .any(|entry| !entry.deps.is_empty())
        {
            return cone::pick_terminal_dep_bearing(
                g,
                ctx.top,
                ctx.parent_source_pool,
                width,
                None,
            );
        }
    } else {
        let mut non_helper_pool = SignalPool::new();
        for entry in ctx
            .parent_source_pool
            .iter()
            .filter(|entry| !node_is_parent_cone_instance_output(ctx.top, entry.node))
        {
            non_helper_pool.add(entry.node, entry.width, entry.deps.clone());
        }
        if non_helper_pool.iter().any(|entry| !entry.deps.is_empty()) {
            return cone::pick_terminal_dep_bearing(g, ctx.top, &mut non_helper_pool, width, None);
        }
    }

    if ctx.allow_external_pool_reuse
        && ctx
            .external_input_pool
            .iter()
            .any(|entry| !entry.deps.is_empty())
    {
        return cone::pick_terminal_dep_bearing(g, ctx.top, ctx.external_input_pool, width, None);
    }

    if ctx.allow_external_pool_reuse {
        let (node, is_new) = ctx.top.intern_constant(width, 0);
        if is_new {
            ctx.parent_source_pool.add(node, width, DepSet::new());
        }
        return node;
    }

    let input_name = format!("{instance_name}__{child_input_name}");
    let (port_id, node_id) = add_top_input(ctx.top, ctx.next_port_id, &input_name, width);
    let deps = DepSet::from_port(port_id);
    ctx.external_input_pool.add(node_id, width, deps.clone());
    ctx.parent_source_pool.add(node_id, width, deps);
    node_id
}

fn build_registered_sibling_route(
    g: &mut Generator,
    top: &mut Module,
    instance_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    width: u32,
) -> NodeId {
    let d_node = cone::pick_terminal_dep_bearing(g, top, instance_pool, width, None);
    let flop_id = top.flops.len() as FlopId;
    let q_node_id = top.nodes.len() as NodeId;
    top.nodes.push(Node::FlopQ {
        flop: flop_id,
        width,
    });
    top.flops.push(Flop {
        id: flop_id,
        width,
        d: Some(d_node),
        q: q_node_id,
        reset_val: 0,
        reset_kind: ResetKind::Async,
        kind: FlopKind::ZeroDefault,
        mux: FlopMux::None,
    });
    let deps = DepSet::from_flop_virtual(flop_id);
    parent_source_pool.add(q_node_id, width, deps);
    q_node_id
}

fn build_registered_child_input_cone_route(
    g: &mut Generator,
    top: &mut Module,
    parent_source_pool: &mut SignalPool,
    width: u32,
    required_parent_cone_instance_source: Option<NodeId>,
) -> NodeId {
    let mut route_pool = parent_source_pool.clone();

    let saved_flop_prob = g.cfg.flop_prob;
    let saved_flop_knob = g.active_flop_knob;
    g.cfg.flop_prob = 0.0;
    g.active_flop_knob = KnobId::HierarchyParentFlopProb;
    let mut worklist: FlopWorklist = Vec::new();
    let d_root = cone::build_cone_with_retry(g, top, &mut route_pool, &mut worklist, width, None);
    cone::drain_flop_worklist(g, top, &mut route_pool, &mut worklist);
    debug_assert!(
        worklist.is_empty(),
        "registered child-input cone worklist drained"
    );
    g.cfg.flop_prob = saved_flop_prob;
    g.active_flop_knob = saved_flop_knob;

    let d_logic = promote_registered_child_input_cone_root(
        g,
        top,
        &mut route_pool,
        parent_source_pool,
        d_root,
        width,
        required_parent_cone_instance_source,
    );

    register_parent_child_input_route(top, parent_source_pool, d_logic, width)
}

fn promote_registered_child_input_cone_root(
    g: &mut Generator,
    top: &mut Module,
    local_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    root: NodeId,
    width: u32,
    required_parent_cone_instance_source: Option<NodeId>,
) -> NodeId {
    let with_child_support = if cone::node_deps(top, root).has_instance_output_virtuals() {
        root
    } else {
        let Some(companion) =
            try_pick_instance_output_companion(g, top, parent_source_pool, width, root)
        else {
            return root;
        };
        add_registered_child_input_companion_gate(
            top,
            local_pool,
            parent_source_pool,
            width,
            root,
            companion,
        )
    };

    let with_helper_support = if let Some(required_source) = required_parent_cone_instance_source {
        ensure_registered_parent_cone_instance_support(
            top,
            local_pool,
            parent_source_pool,
            with_child_support,
            required_source,
            width,
        )
    } else {
        with_child_support
    };

    let with_parent_port_support = if cone::node_deps(top, with_helper_support).has_ports() {
        with_helper_support
    } else {
        let Some(companion) =
            try_pick_parent_port_companion(g, top, parent_source_pool, width, with_helper_support)
        else {
            return with_helper_support;
        };
        add_registered_child_input_companion_gate(
            top,
            local_pool,
            parent_source_pool,
            width,
            with_helper_support,
            companion,
        )
    };

    let with_parent_flop_support =
        if cone::node_deps(top, with_parent_port_support).has_flop_virtuals() {
            with_parent_port_support
        } else {
            let Some(companion) = try_pick_parent_flop_companion(
                g,
                top,
                parent_source_pool,
                width,
                with_parent_port_support,
            ) else {
                return ensure_parent_logic_above_instance_source(
                    top,
                    local_pool,
                    parent_source_pool,
                    with_parent_port_support,
                    width,
                );
            };
            add_registered_child_input_companion_gate(
                top,
                local_pool,
                parent_source_pool,
                width,
                with_parent_port_support,
                companion,
            )
        };

    ensure_parent_logic_above_instance_source(
        top,
        local_pool,
        parent_source_pool,
        with_parent_flop_support,
        width,
    )
}

fn ensure_registered_parent_cone_instance_support(
    top: &mut Module,
    local_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    root: NodeId,
    required_source: NodeId,
    width: u32,
) -> NodeId {
    let Node::InstanceOutput {
        instance,
        port,
        width: source_width,
    } = &top.nodes[required_source as usize]
    else {
        return root;
    };
    let (instance, port, source_width) = (*instance, *port, *source_width);
    if cone::node_deps(top, root).contains_instance_output_virtual(instance, port) {
        return root;
    }

    let source_deps = cone::node_deps(top, required_source);
    let adapted = cone::make_width_adapter(
        top,
        local_pool,
        required_source,
        source_width,
        source_deps,
        width,
    );
    add_registered_child_input_companion_gate(
        top,
        local_pool,
        parent_source_pool,
        width,
        root,
        adapted,
    )
}

fn ensure_parent_logic_above_instance_source(
    top: &mut Module,
    local_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    root: NodeId,
    width: u32,
) -> NodeId {
    let root_deps = cone::node_deps(top, root);
    debug_assert!(
        root_deps.has_instance_output_virtuals(),
        "registered parent-composed child input must retain sibling-output support"
    );

    if is_registered_parent_composed_logic_node(top, root) {
        add_pool_entry_once(parent_source_pool, root, width, root_deps);
        return root;
    }

    let all_ones = if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    };
    let (ones, ones_is_new) = top.intern_constant(width, all_ones);
    if ones_is_new {
        local_pool.add(ones, width, DepSet::new());
    }

    let (logic, logic_is_new) =
        top.intern_gate(GateOp::Xor, vec![root, ones], width, root_deps.clone());
    if logic_is_new {
        local_pool.add(logic, width, root_deps.clone());
    }
    add_pool_entry_once(parent_source_pool, logic, width, root_deps);
    logic
}

fn add_registered_child_input_companion_gate(
    top: &mut Module,
    local_pool: &mut SignalPool,
    parent_source_pool: &mut SignalPool,
    width: u32,
    root: NodeId,
    companion: NodeId,
) -> NodeId {
    let node = add_parent_companion_gate(top, local_pool, width, root, companion);
    add_pool_entry_once(parent_source_pool, node, width, cone::node_deps(top, node));
    node
}

fn is_registered_parent_composed_logic_node(top: &Module, node: NodeId) -> bool {
    matches!(
        top.nodes[node as usize],
        Node::Gate { op, .. }
            if !matches!(op, GateOp::Slice { .. } | GateOp::Concat)
    )
}

fn register_parent_child_input_route(
    top: &mut Module,
    parent_source_pool: &mut SignalPool,
    d_node: NodeId,
    width: u32,
) -> NodeId {
    let flop_id = top.flops.len() as FlopId;
    let q_node_id = top.nodes.len() as NodeId;
    top.nodes.push(Node::FlopQ {
        flop: flop_id,
        width,
    });
    top.flops.push(Flop {
        id: flop_id,
        width,
        d: Some(d_node),
        q: q_node_id,
        reset_val: 0,
        reset_kind: ResetKind::Async,
        kind: FlopKind::ZeroDefault,
        mux: FlopMux::None,
    });
    let deps = DepSet::from_flop_virtual(flop_id);
    parent_source_pool.add(q_node_id, width, deps);
    q_node_id
}

fn add_pool_entry_once(pool: &mut SignalPool, node: NodeId, width: u32, deps: DepSet) {
    if !pool.iter().any(|entry| entry.node == node) {
        pool.add(node, width, deps);
    }
}

fn build_child_input_parent_cone(
    g: &mut Generator,
    top: &mut Module,
    parent_source_pool: &mut SignalPool,
    width: u32,
    required_parent_cone_instance_source: Option<NodeId>,
) -> NodeId {
    let saved_flop_prob = g.cfg.flop_prob;
    let saved_flop_knob = g.active_flop_knob;
    g.cfg.flop_prob = g.cfg.hierarchy_parent_flop_prob;
    g.active_flop_knob = KnobId::HierarchyParentFlopProb;
    let mut worklist: FlopWorklist = Vec::new();
    let root = cone::build_cone_with_retry(g, top, parent_source_pool, &mut worklist, width, None);
    cone::drain_flop_worklist(g, top, parent_source_pool, &mut worklist);
    debug_assert!(
        worklist.is_empty(),
        "parent child-input flop worklist drained"
    );
    g.cfg.flop_prob = saved_flop_prob;
    g.active_flop_knob = saved_flop_knob;
    if let Some(required_source) = required_parent_cone_instance_source {
        ensure_parent_cone_instance_support(top, parent_source_pool, root, required_source, width)
    } else {
        root
    }
}

fn ensure_parent_cone_instance_support(
    top: &mut Module,
    parent_source_pool: &mut SignalPool,
    root: NodeId,
    required_source: NodeId,
    width: u32,
) -> NodeId {
    let Node::InstanceOutput {
        instance,
        port,
        width: source_width,
    } = &top.nodes[required_source as usize]
    else {
        return root;
    };
    let (instance, port, source_width) = (*instance, *port, *source_width);
    if cone::node_deps(top, root).contains_instance_output_virtual(instance, port) {
        return root;
    }

    let source_deps = cone::node_deps(top, required_source);
    let adapted = cone::make_width_adapter(
        top,
        parent_source_pool,
        required_source,
        source_width,
        source_deps,
        width,
    );
    add_parent_companion_gate(top, parent_source_pool, width, root, adapted)
}

fn build_parent_output_roots(
    g: &mut Generator,
    top: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
) -> Vec<NodeId> {
    let saved_flop_prob = g.cfg.flop_prob;
    let saved_flop_knob = g.active_flop_knob;
    g.cfg.flop_prob = g.cfg.hierarchy_parent_flop_prob;
    g.active_flop_knob = KnobId::HierarchyParentFlopProb;
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
    cone::drain_flop_worklist(g, top, pool, worklist);
    g.cfg.flop_prob = saved_flop_prob;
    g.active_flop_knob = saved_flop_knob;
    roots
}

struct ParentOutputPromotionContext<'a> {
    pool: &'a mut SignalPool,
    instance_pool: &'a SignalPool,
    parent_source_pool: &'a SignalPool,
    required_parent_cone_instance_source: Option<NodeId>,
}

fn promote_parent_output_root(
    g: &mut Generator,
    top: &mut Module,
    ctx: &mut ParentOutputPromotionContext<'_>,
    width: u32,
    root: NodeId,
) -> NodeId {
    let with_child_support = if cone::node_deps(top, root).has_instance_output_virtuals() {
        root
    } else {
        let Some(companion) = try_pick_parent_companion(g, top, ctx.instance_pool, width, root)
        else {
            return root;
        };

        add_parent_companion_gate(top, ctx.pool, width, root, companion)
    };

    let with_helper_support =
        if let Some(required_source) = ctx.required_parent_cone_instance_source {
            ensure_parent_cone_instance_support(
                top,
                ctx.pool,
                with_child_support,
                required_source,
                width,
            )
        } else {
            with_child_support
        };

    if cone::node_deps(top, with_helper_support).has_ports() {
        return with_helper_support;
    }

    let Some(parent_companion) =
        try_pick_parent_port_companion(g, top, ctx.parent_source_pool, width, with_helper_support)
    else {
        return with_helper_support;
    };

    add_parent_companion_gate(top, ctx.pool, width, with_helper_support, parent_companion)
}

fn add_parent_companion_gate(
    top: &mut Module,
    pool: &mut SignalPool,
    width: u32,
    root: NodeId,
    companion: NodeId,
) -> NodeId {
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

fn try_pick_instance_output_companion(
    g: &mut Generator,
    top: &mut Module,
    parent_source_pool: &SignalPool,
    width: u32,
    exclude: NodeId,
) -> Option<NodeId> {
    let mut temp_pool = SignalPool::new();
    for entry in parent_source_pool
        .iter()
        .filter(|entry| entry.node != exclude && entry.deps.has_instance_output_virtuals())
    {
        temp_pool.add(entry.node, entry.width, entry.deps.clone());
    }

    if temp_pool.is_empty() {
        return None;
    }

    Some(cone::pick_terminal_dep_bearing(
        g,
        top,
        &mut temp_pool,
        width,
        Some(exclude),
    ))
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

fn try_pick_parent_port_companion(
    g: &mut Generator,
    top: &mut Module,
    parent_source_pool: &SignalPool,
    width: u32,
    exclude: NodeId,
) -> Option<NodeId> {
    let mut temp_pool = SignalPool::new();
    for entry in parent_source_pool
        .iter()
        .filter(|entry| entry.node != exclude && entry.deps.has_ports())
    {
        temp_pool.add(entry.node, entry.width, entry.deps.clone());
    }

    if temp_pool.is_empty() {
        return None;
    }

    Some(cone::pick_terminal_dep_bearing(
        g,
        top,
        &mut temp_pool,
        width,
        Some(exclude),
    ))
}

fn try_pick_parent_flop_companion(
    g: &mut Generator,
    top: &mut Module,
    parent_source_pool: &SignalPool,
    width: u32,
    exclude: NodeId,
) -> Option<NodeId> {
    let mut temp_pool = SignalPool::new();
    for entry in parent_source_pool
        .iter()
        .filter(|entry| entry.node != exclude && entry.deps.has_flop_virtuals())
    {
        temp_pool.add(entry.node, entry.width, entry.deps.clone());
    }

    if temp_pool.is_empty() {
        return None;
    }

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
    use crate::config::Config;
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
        let top = generate_parent_module(&mut g, 1, &[child], &[], &[0], None);

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

    #[test]
    fn legacy_wrapper_on_demand_synthesizes_one_child_definition_per_instance() {
        let mut g = Generator::new(Config {
            seed: 23,
            hierarchy_depth: 1,
            num_child_instances: 3,
            hierarchy_child_source_mode: crate::config::HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        });

        let design = generate_design(&mut g);
        let top = design
            .modules
            .iter()
            .find(|m| m.name == design.top)
            .expect("top exists");
        let used_children: std::collections::BTreeSet<_> = top
            .instances
            .iter()
            .map(|instance| instance.module.as_str())
            .collect();

        assert_eq!(top.instances.len(), 3);
        assert_eq!(used_children.len(), 3);
        assert_eq!(design.modules.len(), 4, "3 fresh children + top");
    }

    #[test]
    fn profiled_parent_module_honors_exact_data_interface_shape() {
        let mut g = Generator::new(Config {
            seed: 29,
            hierarchy_depth: 1,
            ..Config::default()
        });
        let child = Module {
            name: "leaf".into(),
            outputs: vec![Port {
                id: 0,
                name: "y".into(),
                width: 8,
                dir: Direction::Out,
            }],
            ..Module::default()
        };
        let profile = ModuleInterfaceProfile {
            data_input_widths: vec![5, 9],
            output_widths: vec![7, 11],
        };

        let top = generate_parent_module(&mut g, 1, &[child], &[], &[0], Some(&profile));
        let got_inputs: Vec<_> = top
            .emitted_data_input_ports()
            .map(|port| port.width)
            .collect();
        let got_outputs: Vec<_> = top.outputs.iter().map(|port| port.width).collect();

        assert_eq!(top.planned_interface_profile, Some(profile.clone()));
        assert_eq!(got_inputs, profile.data_input_widths);
        assert_eq!(got_outputs, profile.output_widths);
    }

    #[test]
    fn recursive_range_generation_builds_requested_exact_depth() {
        let mut g = Generator::new(Config {
            seed: 3,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 3,
            ..Config::default()
        });

        let design = generate_design(&mut g);
        let top = design
            .modules
            .iter()
            .find(|m| m.name == design.top)
            .expect("top exists");
        assert!(
            !top.instances.is_empty(),
            "depth-2 top should instantiate children"
        );
        assert!(
            design
                .modules
                .iter()
                .any(|m| !m.instances.is_empty() && m.name != design.top),
            "depth-2 design should contain at least one nested non-leaf child"
        );
    }

    #[test]
    fn recursive_range_generation_can_mix_shallow_and_deep_branches() {
        let mut g = Generator::new(Config {
            seed: 19,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            ..Config::default()
        });

        let design = generate_design(&mut g);
        let metrics = crate::metrics::compute_design(&design);
        assert_eq!(
            metrics.realized_min_leaf_depth, 2,
            "mixed recursive planning should preserve the requested minimum depth"
        );
        assert_eq!(
            metrics.realized_max_leaf_depth, 3,
            "mixed recursive planning should preserve the requested maximum depth"
        );
        assert_eq!(
            metrics.leaf_module_occurrences_by_depth.get(&2),
            Some(&2),
            "depth-2 leaves should be present in the mixed tree"
        );
        assert_eq!(
            metrics.leaf_module_occurrences_by_depth.get(&3),
            Some(&4),
            "depth-3 leaves should be present in the mixed tree"
        );
    }

    #[test]
    fn recursive_range_generation_respects_per_depth_branching_overrides() {
        let mut g = Generator::new(Config {
            seed: 11,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 1,
            max_child_instances_per_module: 3,
            child_instances_per_module_by_depth: BTreeMap::from([
                (0, crate::config::CountRange { min: 4, max: 4 }),
                (1, crate::config::CountRange { min: 2, max: 2 }),
            ]),
            ..Config::default()
        });

        let design = generate_design(&mut g);
        let modules_by_name = design
            .modules
            .iter()
            .map(|module| (module.name.as_str(), module))
            .collect::<BTreeMap<_, _>>();
        let top = modules_by_name
            .get(design.top.as_str())
            .expect("top exists");
        assert_eq!(
            top.instances.len(),
            4,
            "top-level branching should follow depth-0 override"
        );

        for instance in &top.instances {
            let child = modules_by_name
                .get(instance.module.as_str())
                .expect("child exists");
            assert_eq!(
                child.instances.len(),
                2,
                "nested branching should follow depth-1 override"
            );
        }
    }
}
