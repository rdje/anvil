//! End-to-end: generate many modules across seeds and assert each
//! passes IR validation and produces non-empty SV output.

use anvil::config::{ConstructionStrategy, HierarchyChildSourceMode};
use anvil::ir::{GateOp, Node};
use anvil::{Config, Generator};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn generates_valid_modules_across_seeds() {
    for seed in 0..20u64 {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        cfg.validate().expect("default config should be valid");

        let mut g = Generator::new(cfg);
        let m = g.generate_module();
        anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
            panic!("seed {}: IR validation failed: {}", seed, e);
        });

        let sv = anvil::emit::to_sv(&m);
        assert!(sv.contains("module "));
        assert!(sv.contains("endmodule"));
    }
}

#[test]
fn generates_valid_depth1_wrapper_designs() {
    for seed in 0..5u64 {
        let cfg = Config {
            seed,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            ..Config::default()
        };
        cfg.validate()
            .expect("depth-1 hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!("hierarchy seed {}: design validation failed: {}", seed, e);
        });

        assert_eq!(design.modules.len(), 3, "2 leaves + 1 top wrapper expected");
        let top = design
            .modules
            .iter()
            .find(|module| module.name == design.top)
            .expect("top module must exist");
        assert_eq!(top.instances.len(), 2, "top must instantiate every leaf");

        let sv = anvil::emit::to_sv_design(&design);
        assert!(
            sv.matches("\nmodule ").count() >= 2 || sv.starts_with("module "),
            "hierarchical emission should contain multiple module declarations"
        );
        assert!(
            sv.contains(" u_0 (") || sv.contains(" u_1 ("),
            "top wrapper should emit real child instances:\n{sv}"
        );
    }
}

#[test]
fn generates_valid_depth1_ondemand_wrapper_designs() {
    for seed in 0..5u64 {
        let cfg = Config {
            seed,
            hierarchy_depth: 1,
            num_child_instances: 3,
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        cfg.validate()
            .expect("depth-1 on-demand hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "on-demand hierarchy seed {}: design validation failed: {}",
                seed, e
            );
        });

        assert_eq!(
            design.modules.len(),
            4,
            "3 fresh child definitions + 1 top wrapper expected"
        );
        let top = design
            .modules
            .iter()
            .find(|module| module.name == design.top)
            .expect("top module must exist");
        let used_children: BTreeSet<_> = top
            .instances
            .iter()
            .map(|instance| instance.module.as_str())
            .collect();
        assert_eq!(top.instances.len(), 3);
        assert_eq!(
            used_children.len(),
            3,
            "on-demand wrapper mode should synthesize one fresh child definition per instance slot"
        );

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.num_reused_instance_slots, 0);
        assert_eq!(metrics.num_single_use_instantiated_modules, 3);
        assert_eq!(metrics.single_use_instantiated_module_fraction, 1.0);
        assert_eq!(metrics.num_profiled_instance_slots, 3);
        assert_eq!(metrics.profiled_instance_fraction, 1.0);
    }
}

#[test]
fn generates_valid_recursive_hierarchy_designs_with_bounded_shape() {
    for seed in 0..5u64 {
        let cfg = Config {
            seed,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 3,
            ..Config::default()
        };
        cfg.validate()
            .expect("bounded recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive hierarchy seed {}: design validation failed: {}",
                seed, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            (2..=3).contains(&metrics.realized_max_leaf_depth),
            "realized depth must stay within requested bound"
        );
        assert_eq!(
            metrics.realized_min_leaf_depth, 2,
            "recursive hierarchy should preserve the requested minimum depth"
        );
        assert!(
            (2..=3).contains(&metrics.min_child_instances_per_internal_module),
            "internal branching floor must stay inside requested range"
        );
        assert!(
            (2..=3).contains(&metrics.max_child_instances_per_internal_module),
            "internal branching ceiling must stay inside requested range"
        );
    }
}

#[test]
fn generates_valid_recursive_hierarchy_designs_with_ondemand_child_sourcing() {
    for seed in 0..4u64 {
        let cfg = Config {
            seed,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        cfg.validate()
            .expect("on-demand recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "on-demand recursive hierarchy seed {}: design validation failed: {}",
                seed, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert_eq!(metrics.num_reused_instance_slots, 0);
        assert_eq!(
            metrics.num_single_use_instantiated_modules, metrics.num_unique_instantiated_modules,
            "on-demand recursive sourcing should keep every instantiated definition single-use"
        );
        assert_eq!(metrics.single_use_instantiated_module_fraction, 1.0);
        assert_eq!(
            metrics.num_profiled_instance_slots, metrics.num_instances,
            "on-demand recursive sourcing should synthesize every instantiated child from a parent-planned exact profile"
        );
        assert_eq!(metrics.profiled_instance_fraction, 1.0);
    }
}

#[test]
fn on_demand_recursive_hierarchy_exactly_realizes_profiled_child_interfaces() {
    let cfg = Config {
        seed: 31,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 2,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 2,
        hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
        ..Config::default()
    };
    cfg.validate()
        .expect("profiled on-demand recursive hierarchy config should be valid");

    let mut g = Generator::new(cfg);
    let design = g.generate_design();
    anvil::ir::validate::validate_design(&design)
        .expect("profiled on-demand recursive hierarchy should validate");

    let modules_view = design
        .modules
        .iter()
        .map(|module| (module.name.as_str(), module))
        .collect::<BTreeMap<_, _>>();

    let profiled_modules: Vec<_> = design
        .modules
        .iter()
        .filter(|module| module.planned_interface_profile.is_some())
        .collect();
    assert!(
        !profiled_modules.is_empty(),
        "on-demand recursive hierarchy should carry exact planned child-interface profiles"
    );

    for module in profiled_modules {
        let profile = module
            .planned_interface_profile
            .as_ref()
            .expect("profiled module should carry its planned profile");
        let got_data_inputs: Vec<_> = module
            .emitted_data_input_ports_in(Some(&modules_view))
            .map(|port| port.width)
            .collect();
        let got_outputs: Vec<_> = module.outputs.iter().map(|port| port.width).collect();
        assert_eq!(got_data_inputs, profile.data_input_widths);
        assert_eq!(got_outputs, profile.output_widths);
    }
}

#[test]
fn generates_valid_recursive_hierarchy_designs_with_mixed_leaf_depths() {
    let cfg = Config {
        seed: 19,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 3,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 2,
        ..Config::default()
    };
    cfg.validate()
        .expect("mixed-depth recursive hierarchy config should be valid");

    let mut g = Generator::new(cfg);
    let design = g.generate_design();
    anvil::ir::validate::validate_design(&design)
        .expect("mixed recursive hierarchy should validate");

    let metrics = anvil::metrics::compute_design(&design);
    assert_eq!(metrics.realized_min_leaf_depth, 2);
    assert_eq!(metrics.realized_max_leaf_depth, 3);
    assert_eq!(metrics.leaf_module_occurrences_by_depth.get(&2), Some(&2));
    assert_eq!(metrics.leaf_module_occurrences_by_depth.get(&3), Some(&4));
}

#[test]
fn generates_valid_recursive_hierarchy_designs_with_per_depth_branching_controls() {
    for seed in 0..4u64 {
        let cfg = Config {
            seed,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 1,
            max_child_instances_per_module: 3,
            child_instances_per_module_by_depth: BTreeMap::from([
                (0, anvil::config::CountRange { min: 4, max: 4 }),
                (1, anvil::config::CountRange { min: 2, max: 2 }),
            ]),
            ..Config::default()
        };
        cfg.validate()
            .expect("per-depth recursive hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "per-depth recursive hierarchy seed {}: design validation failed: {}",
                seed, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert_eq!(
            metrics.min_child_instances_by_parent_depth.get(&0),
            Some(&4)
        );
        assert_eq!(
            metrics.max_child_instances_by_parent_depth.get(&0),
            Some(&4)
        );
        assert_eq!(
            metrics.avg_child_instances_by_parent_depth.get(&0),
            Some(&4.0)
        );
        assert_eq!(
            metrics.min_child_instances_by_parent_depth.get(&1),
            Some(&2)
        );
        assert_eq!(
            metrics.max_child_instances_by_parent_depth.get(&1),
            Some(&2)
        );
        assert_eq!(
            metrics.avg_child_instances_by_parent_depth.get(&1),
            Some(&2.0)
        );
    }
}

#[test]
fn depth1_wrapper_can_reuse_leaf_definitions_across_more_instances_than_library_entries() {
    let cfg = Config {
        seed: 11,
        hierarchy_depth: 1,
        num_leaf_modules: 2,
        num_child_instances: 5,
        ..Config::default()
    };
    cfg.validate()
        .expect("depth-1 hierarchy reuse config should be valid");

    let mut g = Generator::new(cfg);
    let design = g.generate_design();
    anvil::ir::validate::validate_design(&design)
        .expect("reused-child depth-1 design should validate");

    assert_eq!(design.modules.len(), 3, "2 leaves + 1 top wrapper expected");
    let top = design
        .modules
        .iter()
        .find(|module| module.name == design.top)
        .expect("top module must exist");
    let used_children: BTreeSet<_> = top
        .instances
        .iter()
        .map(|instance| instance.module.as_str())
        .collect();
    assert_eq!(
        top.instances.len(),
        5,
        "top must honor explicit child instance count"
    );
    assert_eq!(
        used_children.len(),
        2,
        "the two library modules should both stay usable"
    );
    assert!(
        top.instances.len() > used_children.len(),
        "at least one leaf definition should be reused when instances exceed library size"
    );
}

#[test]
fn depth1_wrapper_can_under_instantiate_the_leaf_library() {
    let cfg = Config {
        seed: 17,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 2,
        ..Config::default()
    };
    cfg.validate()
        .expect("depth-1 hierarchy under-instantiation config should be valid");

    let mut g = Generator::new(cfg);
    let design = g.generate_design();
    anvil::ir::validate::validate_design(&design)
        .expect("under-instantiated depth-1 design should validate");

    assert_eq!(design.modules.len(), 5, "4 leaves + 1 top wrapper expected");
    let top = design
        .modules
        .iter()
        .find(|module| module.name == design.top)
        .expect("top module must exist");
    let library_children: BTreeSet<_> = design
        .modules
        .iter()
        .filter(|module| module.name != design.top)
        .map(|module| module.name.as_str())
        .collect();
    let used_children: BTreeSet<_> = top
        .instances
        .iter()
        .map(|instance| instance.module.as_str())
        .collect();

    assert_eq!(
        top.instances.len(),
        2,
        "top must honor explicit child instance count"
    );
    assert_eq!(
        library_children.len(),
        4,
        "the leaf library should still contain all definitions"
    );
    assert_eq!(
        used_children.len(),
        2,
        "only two leaf definitions should be instantiated"
    );
    assert!(
        used_children.len() < library_children.len(),
        "under-instantiation should leave some generated leaf definitions unused by the wrapper"
    );
}

#[test]
fn depth1_parent_outputs_depend_on_child_instance_outputs() {
    for seed in 0..5u64 {
        let cfg = Config {
            seed,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            ..Config::default()
        };
        cfg.validate()
            .expect("depth-1 hierarchy composition config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!("hierarchy seed {}: design validation failed: {}", seed, e);
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(
            metrics.top_outputs_reaching_instance_outputs, metrics.top_outputs,
            "top outputs should stay functions of child instance outputs: {metrics:#?}"
        );
        assert_eq!(
            metrics.top_outputs_without_instance_outputs,
            0,
            "the current parent-composition slice should not emit top outputs that bypass child outputs"
        );
        assert!(
            metrics.top_parent_composed_outputs > 0,
            "expected at least one genuine parent-composed output for seed {seed}: {metrics:#?}"
        );
    }
}

#[test]
fn hierarchy_parent_outputs_can_mix_parent_ports_with_child_outputs() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 7,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            min_inputs: 2,
            max_inputs: 2,
            min_outputs: 2,
            max_outputs: 2,
            flop_prob: 0.0,
            max_flops_per_module: 0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("mixed parent-output hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "mixed parent-output hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(
            metrics.top_outputs_reaching_instance_outputs, metrics.top_outputs,
            "mixed parent outputs must still retain child-output support: {metrics:#?}"
        );
        assert!(
            metrics.top_parent_port_composed_outputs > 0,
            "expected parent outputs to mix parent data ports with child outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_parent_port_composed_output_fraction > 0.0,
            "mixed parent-output fraction should be measurable for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_child_inputs_can_be_routed_from_sibling_instance_outputs() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_sibling_routing = false;
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                hierarchy_depth: 1,
                num_leaf_modules: 2,
                num_child_instances: 4,
                hierarchy_sibling_route_prob: 1.0,
                hierarchy_child_input_cone_prob: 0.0,
                construction_strategy: strategy,
                ..Config::default()
            };
            cfg.validate()
                .expect("sibling-routed hierarchy config should be valid");

            let mut g = Generator::new(cfg);
            let design = g.generate_design();
            anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
                panic!(
                    "sibling-routed hierarchy strategy {:?} seed {}: design validation failed: {}",
                    strategy, seed, e
                );
            });

            let metrics = anvil::metrics::compute_design(&design);
            if metrics.child_input_bindings_from_instance_outputs > 0 {
                assert!(
                    metrics.top_child_input_bindings_from_instance_outputs > 0,
                    "strategy {:?} seed {} should expose sibling-routed child inputs at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.instance_output_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero hierarchy-wide sibling-routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_instance_output_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level sibling-routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                saw_sibling_routing = true;
                break;
            }
        }
        assert!(
            saw_sibling_routing,
            "expected at least one sibling-routed hierarchy design across the 32-seed sweep for strategy {:?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_child_inputs_can_be_bound_through_parent_composed_logic() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_parent_composed_binding = false;
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                hierarchy_depth: 1,
                num_leaf_modules: 2,
                num_child_instances: 4,
                hierarchy_sibling_route_prob: 0.0,
                hierarchy_child_input_cone_prob: 1.0,
                construction_strategy: strategy,
                ..Config::default()
            };
            cfg.validate()
                .expect("parent-composed child-input hierarchy config should be valid");

            let mut g = Generator::new(cfg);
            let design = g.generate_design();
            anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
                panic!(
                    "parent-composed hierarchy strategy {:?} seed {}: design validation failed: {}",
                    strategy, seed, e
                );
            });

            let metrics = anvil::metrics::compute_design(&design);
            if metrics.child_input_bindings_from_parent_composed_logic > 0 {
                assert!(
                    metrics.top_child_input_bindings_from_parent_composed_logic > 0,
                    "strategy {:?} seed {} should expose parent-composed child input bindings at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.parent_composed_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero hierarchy-wide parent-composed binding fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_parent_composed_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level parent-composed binding fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                saw_parent_composed_binding = true;
                break;
            }
        }
        assert!(
            saw_parent_composed_binding,
            "expected at least one parent-composed child-input hierarchy design across the 32-seed sweep for strategy {:?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_child_input_cones_can_instantiate_helper_children() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("parent-cone helper instance hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "parent-cone helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected at least one top-level parent-cone helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances > 0,
            "expected child input bindings to depend on helper instance outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected helper-instance binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instance should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_parent_cone_helper_budget_allows_multiple_helpers() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("budgeted parent-cone helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;
        let helper_budget = cfg.max_parent_cone_instances_per_module as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "budgeted parent-cone helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(
            metrics.top_parent_cone_instances, helper_budget,
            "expected the top module to spend the configured helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.max_parent_cone_instances_per_internal_module, helper_budget,
            "expected per-parent helper metric to record the configured budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances > 0,
            "expected child input bindings to depend on budgeted helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances >= planned_child_instances + helper_budget,
            "helper instances should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive budgeted parent-composed helper hierarchy config should be valid");

        let helper_budget = cfg.max_parent_cone_instances_per_module as usize;
        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive budgeted parent-composed helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.max_parent_cone_instances_per_internal_module, helper_budget,
            "expected at least one recursive parent to spend the configured child-input helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.top_parent_cone_instances, helper_budget,
            "expected top parent to keep the configured helper budget baseline for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances >= helper_budget * 2,
            "expected child-input helper budget placement below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected recursive helper instances beyond the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_composed_logic
                > metrics.top_child_input_bindings_from_parent_composed_logic,
            "expected non-top parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances
                > metrics.top_child_input_bindings_from_parent_cone_instances,
            "expected non-top child-input bindings sourced from budgeted helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances_through_parent_flops, 0,
            "this focused config should prove combinational child-input helper budget use, not helper-through-flop routes, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_sibling_routes_can_use_helper_instances() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("sibling helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "sibling helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_instance_outputs > 0,
            "expected direct sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_instance_outputs, 0,
            "this focused config should prove unregistered sibling helper use, not registered sibling routing, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not count registered helper D paths for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances > 0,
            "expected direct sibling bindings to depend on helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero direct helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero top direct helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instance should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive sibling helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive sibling helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_instance_outputs
                > metrics.top_child_input_bindings_from_instance_outputs,
            "expected non-top sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances
                > metrics.top_child_input_bindings_from_parent_cone_instances,
            "expected non-top direct sibling helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_instance_outputs,
            0,
            "this focused config should prove unregistered sibling helper use, not registered sibling routing, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances,
            0,
            "this focused config should not count registered helper D paths for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_sibling_routes_can_mix_parent_port_support() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_mixed_support_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("registered sibling mixed-support hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "registered sibling mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs > 0,
            "expected direct registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_sibling_mixed_support > 0,
            "expected direct registered sibling D paths to mix parent ports with child outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic,
            0,
            "direct registered sibling mixed-support routes should not be counted as registered parent-composed D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_mixed_support,
            0,
            "direct registered sibling mixed-support routes should stay separate from registered parent-composed mixed support for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_sibling_mixed_support_child_input_binding_fraction > 0.0,
            "expected non-zero registered sibling mixed-support binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_sibling_mixed_support_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive registered sibling mixed-support hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered sibling mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_parent_cone_instances, 0,
            "this focused config should not instantiate helper children for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected direct registered sibling routes below top to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs
                > metrics.top_child_input_bindings_from_registered_instance_outputs,
            "expected non-top direct registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_sibling_mixed_support
                > metrics.top_child_input_bindings_from_registered_sibling_mixed_support,
            "expected non-top direct registered sibling D paths to mix parent ports with child outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic,
            0,
            "direct registered sibling mixed-support routes should not count as registered parent-composed D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_mixed_support,
            0,
            "direct registered sibling mixed-support routes should stay separate from registered parent-composed mixed support for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances,
            0,
            "this focused config should not count registered helper D paths for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_sibling_mixed_support_child_input_binding_fraction > 0.0,
            "expected non-zero registered sibling mixed-support binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive registered sibling helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered sibling helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected registered non-top sibling routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs
                > metrics.top_child_input_bindings_from_registered_instance_outputs,
            "expected non-top registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances
                > metrics.top_child_input_bindings_from_registered_parent_cone_instances,
            "expected non-top registered sibling helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic,
            0,
            "this focused config should prove direct registered sibling helper use, not registered parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero registered helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive multi-stage registered helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive multi-stage registered helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected registered non-top sibling routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_instance_outputs
                > metrics.top_child_input_bindings_from_registered_multistage_instance_outputs,
            "expected non-top multi-stage registered sibling bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances
                > metrics.top_child_input_bindings_from_registered_multistage_parent_cone_instances,
            "expected non-top multi-stage registered sibling helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic,
            0,
            "this focused config should prove direct registered sibling helper chaining, not registered parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic,
            0,
            "this focused config should prove direct registered sibling helper chaining, not multi-stage parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_child_input_cones_can_use_helper_instances() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("registered parent-cone helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "registered parent-cone helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic > 0,
            "expected registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected registered child-input bindings to depend on helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero registered helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive registered parent-composed helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered parent-composed helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected registered non-top parent-composed routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_parent_composed_logic,
            "expected non-top registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances
                > metrics.top_child_input_bindings_from_registered_parent_cone_instances,
            "expected non-top registered parent-composed helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_composed_child_input_binding_fraction > 0.0,
            "expected non-zero registered parent-composed binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero registered helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive registered helper mixed-support hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered helper mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected non-top helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected registered non-top helper routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_parent_composed_logic,
            "expected non-top registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances
                > metrics.top_child_input_bindings_from_registered_parent_cone_instances,
            "expected non-top registered helper-sourced bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instance_mixed_support
                > metrics
                    .top_child_input_bindings_from_registered_parent_cone_instance_mixed_support,
            "expected non-top registered helper-sourced D cones to mix in parent ports for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_cone_instance_mixed_support_child_input_binding_fraction
                > 0.0,
            "expected non-zero registered helper mixed-support binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_mixed_support_routes_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive registered mixed-support hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_parent_cone_instances, 0,
            "this focused config should prove registered mixed-support routing without helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected non-top registered parent-composed routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_parent_composed_logic,
            "expected non-top registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs
                > metrics.top_child_input_bindings_from_registered_instance_outputs,
            "expected non-top registered routes to use child outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_mixed_support
                > metrics.top_child_input_bindings_from_registered_mixed_support,
            "expected non-top registered routes to mix parent ports with child outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_mixed_support_child_input_binding_fraction > 0.0,
            "expected non-zero registered mixed-support fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not depend on registered helper-sourced D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "recursive registered parent-composed multistage hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered parent-composed multistage hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_parent_cone_instances, 0,
            "this focused config should prove no-helper registered parent-composed routing for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected non-top registered parent-composed routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_parent_composed_logic,
            "expected non-top registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_multistage_parent_composed_logic,
            "expected non-top multi-stage registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_parent_composed_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage registered parent-composed binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not depend on registered helper-sourced D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances, 0,
            "this focused config should not depend on direct registered sibling helper chains for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics
                .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances,
            0,
            "this focused config should not depend on parent-composed helper chains for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_multistage_mixed_support_routes_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "recursive registered multistage mixed-support hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive registered multistage mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_parent_cone_instances, 0,
            "this focused config should prove no-helper registered multistage mixed-support routing for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected non-top registered parent-composed routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_parent_composed_logic,
            "expected non-top registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_mixed_support
                > metrics.top_child_input_bindings_from_registered_mixed_support,
            "expected non-top registered mixed-support child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_multistage_parent_composed_logic,
            "expected non-top multi-stage registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_mixed_support
                > metrics.top_child_input_bindings_from_registered_multistage_mixed_support,
            "expected non-top multi-stage registered mixed-support child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_mixed_support_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage registered mixed-support binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not depend on registered helper-sourced D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances, 0,
            "this focused config should not depend on direct registered sibling helper chains for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics
                .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances,
            0,
            "this focused config should not depend on parent-composed helper chains for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_parent_outputs_can_depend_on_helper_instance_outputs() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("parent-output helper instance hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected at least one top-level parent-cone helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove helper use through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_outputs_reaching_parent_cone_instances > 0,
            "expected top outputs to depend on helper instance outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances > 0,
            "expected hierarchy outputs to record helper instance support for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_parent_cone_instance_output_fraction > 0.0,
            "expected non-zero top helper-output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instance should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive parent-output helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected helper instances below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove recursive helper use through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances
                > metrics.top_outputs_reaching_parent_cone_instances,
            "expected non-top parent outputs to depend on helper instance outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops,
            0,
            "this focused config should prove direct recursive parent-output helpers, not helper-through-flop outputs, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instance_output_fraction > 0.0,
            "expected non-zero hierarchy helper-output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "recursive parent-output helper mixed-support hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive parent-output helper mixed-support hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected helper instances below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove helper use through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances
                > metrics.top_outputs_reaching_parent_cone_instances,
            "expected non-top parent outputs to depend on helper instance outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instance_mixed_support
                > metrics.top_outputs_reaching_parent_cone_instance_mixed_support,
            "expected non-top parent outputs to mix helper outputs with parent ports for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops,
            0,
            "this focused config should prove direct recursive parent-output helpers, not helper-through-flop outputs, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instance_mixed_support_output_fraction > 0.0,
            "expected non-zero hierarchy helper mixed-support output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive budgeted parent-output helper hierarchy config should be valid");

        let helper_budget = cfg.max_parent_cone_instances_per_module as usize;
        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive budgeted parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.max_parent_cone_instances_per_internal_module, helper_budget,
            "expected at least one recursive parent to spend the configured helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.top_parent_cone_instances, helper_budget,
            "expected top parent to keep the configured helper budget baseline for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances >= helper_budget * 2,
            "expected helper budget placement below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected recursive helper instances beyond the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove recursive budgeted helpers through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances
                > metrics.top_outputs_reaching_parent_cone_instances,
            "expected non-top parent outputs to depend on budgeted helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("stateful parent-output helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "stateful parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected at least one top-level parent-cone helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_local_flops > 0,
            "expected local parent flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove helper use through stateful parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_outputs_reaching_parent_cone_instances_through_parent_flops > 0,
            "expected top outputs to reach helper instance outputs through parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > 0,
            "expected hierarchy outputs to record helper-through-flop support for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_parent_cone_instance_flop_output_fraction > 0.0,
            "expected non-zero top helper-through-flop output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instance_flop_output_fraction > 0.0,
            "expected non-zero hierarchy helper-through-flop output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instance should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive stateful parent-output helper hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive stateful parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected helper instances below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected parent-local flops below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove recursive stateful helper use through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                > metrics.top_outputs_reaching_parent_cone_instances_through_parent_flops,
            "expected non-top parent outputs to reach helper instance outputs through parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instance_flop_output_fraction > 0.0,
            "expected non-zero hierarchy helper-through-flop output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive stateful budgeted parent-output helper config should be valid");

        let helper_budget = cfg.max_parent_cone_instances_per_module as usize;
        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive stateful budgeted parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.max_parent_cone_instances_per_internal_module, helper_budget,
            "expected at least one recursive parent to spend the configured stateful helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.top_parent_cone_instances, helper_budget,
            "expected top parent to keep the configured helper budget baseline for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances >= helper_budget * 2,
            "expected stateful helper budget placement below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected recursive helper instances beyond the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected parent-local flops below the top parent for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove stateful budgeted helpers through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances_through_parent_flops, 0,
            "this focused config should not spend helper-state routes on child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not create registered child-input helper D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                > metrics.top_outputs_reaching_parent_cone_instances_through_parent_flops,
            "expected non-top parent outputs to reach budgeted helper outputs through parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instance_flop_output_fraction > 0.0,
            "expected non-zero hierarchy helper-through-flop output fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_parent_outputs_can_spend_helper_budget() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("budgeted parent-output helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;
        let helper_budget = cfg.max_parent_cone_instances_per_module as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "budgeted parent-output helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(
            metrics.top_parent_cone_instances, helper_budget,
            "expected parent-output helper placement to spend the configured helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.max_parent_cone_instances_per_internal_module, helper_budget,
            "expected per-parent helper metric to record the output-helper budget for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_parent_cone_instances, 0,
            "this focused config should prove budgeted helpers through parent outputs, not child-input bindings, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_outputs_reaching_parent_cone_instances >= helper_budget,
            "expected parent outputs to depend on budgeted helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances >= planned_child_instances + helper_budget,
            "helper instances should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_module_names_are_unique_across_batch_generation() {
    let cfg = Config {
        seed: 123,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 3,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 3,
        hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
        hierarchy_child_input_cone_prob: 1.0,
        hierarchy_parent_cone_instance_prob: 1.0,
        terminal_reuse_prob: 1.0,
        constant_prob: 0.0,
        ..Config::default()
    };
    cfg.validate()
        .expect("recursive hierarchy config should be valid");

    let mut g = Generator::new(cfg);
    let mut all_names = BTreeSet::new();
    let mut total_modules = 0usize;
    for design_idx in 0..3 {
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!("recursive hierarchy design {design_idx} should validate: {e}");
        });

        let mut design_names = BTreeSet::new();
        for module in &design.modules {
            assert!(
                design_names.insert(module.name.clone()),
                "duplicate module name within design {design_idx}: {}",
                module.name,
            );
            assert!(
                all_names.insert(module.name.clone()),
                "module name reused across generated hierarchy designs: {}",
                module.name,
            );
        }
        assert!(
            design_names.contains(&design.top),
            "design {design_idx} top should name an emitted module"
        );
        total_modules += design.modules.len();
    }

    assert_eq!(all_names.len(), total_modules);
}

#[test]
fn hierarchy_parents_can_emit_local_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_parent_flops = false;
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                hierarchy_depth: 1,
                num_leaf_modules: 2,
                num_child_instances: 4,
                flop_prob: 0.0,
                hierarchy_sibling_route_prob: 1.0,
                hierarchy_child_input_cone_prob: 1.0,
                hierarchy_parent_flop_prob: 1.0,
                max_flops_per_module: 8,
                max_depth: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            cfg.validate()
                .expect("parent-local-flop hierarchy config should be valid");

            let mut g = Generator::new(cfg);
            let design = g.generate_design();
            anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
                panic!(
                    "parent-local-flop hierarchy strategy {:?} seed {}: design validation failed: {}",
                    strategy, seed, e
                );
            });

            let metrics = anvil::metrics::compute_design(&design);
            if metrics.hierarchy_parent_local_flops > 0 {
                assert!(
                    metrics.internal_module_occurrences_with_local_flops > 0,
                    "strategy {:?} seed {} should report parent modules with local flops: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_local_flops > 0,
                    "strategy {:?} seed {} should expose local parent flops at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_clock_inputs, 1,
                    "strategy {:?} seed {} should emit a top clock for local parent flops: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_reset_inputs, 1,
                    "strategy {:?} seed {} should emit a top reset for local parent flops: {metrics:#?}",
                    strategy,
                    seed,
                );
                saw_parent_flops = true;
                break;
            }
        }
        assert!(
            saw_parent_flops,
            "expected at least one local-parent-flop hierarchy design across the 32-seed sweep for strategy {:?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_child_inputs_can_be_registered_from_sibling_instance_outputs() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_registered_sibling_route = false;
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                hierarchy_depth: 1,
                num_leaf_modules: 2,
                num_child_instances: 4,
                flop_prob: 0.0,
                hierarchy_sibling_route_prob: 0.0,
                hierarchy_registered_sibling_route_prob: 1.0,
                hierarchy_child_input_cone_prob: 0.0,
                hierarchy_parent_flop_prob: 0.0,
                max_flops_per_module: 8,
                construction_strategy: strategy,
                ..Config::default()
            };
            cfg.validate()
                .expect("registered sibling-routed hierarchy config should be valid");

            let mut g = Generator::new(cfg);
            let design = g.generate_design();
            anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
                panic!(
                    "registered sibling-route hierarchy strategy {:?} seed {}: design validation failed: {}",
                    strategy, seed, e
                );
            });

            let metrics = anvil::metrics::compute_design(&design);
            if metrics.child_input_bindings_from_registered_instance_outputs > 0 {
                assert!(
                    metrics.top_child_input_bindings_from_registered_instance_outputs > 0,
                    "strategy {:?} seed {} should expose registered sibling-routed child inputs at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.child_input_bindings_from_parent_flops > 0,
                    "strategy {:?} seed {} should count the parent-local flop leg: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.registered_instance_output_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero hierarchy-wide registered sibling-routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_registered_instance_output_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level registered sibling-routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.hierarchy_parent_local_flops > 0,
                    "strategy {:?} seed {} should report local parent state: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_clock_inputs, 1,
                    "strategy {:?} seed {} should emit top clk for registered sibling routing: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_reset_inputs, 1,
                    "strategy {:?} seed {} should emit top rst_n for registered sibling routing: {metrics:#?}",
                    strategy,
                    seed,
                );
                saw_registered_sibling_route = true;
                break;
            }
        }
        assert!(
            saw_registered_sibling_route,
            "expected at least one registered sibling-routed hierarchy design across the 32-seed sweep for strategy {:?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_sibling_routes_can_use_helper_instances() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 3,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("registered sibling helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "registered sibling helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs > 0,
            "expected registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic, 0,
            "this focused config should prove direct registered sibling helper use, not registered parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected registered sibling D flops to depend on helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero registered helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instance should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_sibling_routes_can_chain_through_parent_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("multi-stage registered sibling hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "multi-stage registered sibling hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs > 0,
            "expected first-stage registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_instance_outputs > 0,
            "expected registered sibling routes to chain through earlier parent-local Qs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_child_input_bindings_from_registered_multistage_instance_outputs > 0,
            "expected top-level multi-stage registered sibling bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic, 0,
            "this focused config should prove direct registered sibling chaining, not registered parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic, 0,
            "this focused config should prove direct registered sibling chaining, not multi-stage parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_instance_output_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage registered sibling binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_registered_multistage_instance_output_child_input_binding_fraction > 0.0,
            "expected non-zero top-level multi-stage registered sibling binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 0.0,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "recursive multi-stage registered sibling no-helper hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive multi-stage registered sibling no-helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.hierarchy_parent_cone_instances, 0,
            "this focused config should prove no-helper registered sibling routing for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected non-top registered sibling routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_instance_outputs
                > metrics.top_child_input_bindings_from_registered_instance_outputs,
            "expected non-top registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_instance_outputs
                > metrics.top_child_input_bindings_from_registered_multistage_instance_outputs,
            "expected non-top multi-stage registered sibling child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_instance_output_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage registered sibling binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic, 0,
            "this focused config should not depend on registered parent-composed D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic, 0,
            "this focused config should not depend on multi-stage registered parent-composed D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances, 0,
            "this focused config should not depend on registered helper-sourced D cones for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances, 0,
            "this focused config should not depend on direct registered sibling helper chains for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 1.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("multi-stage registered helper hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "multi-stage registered helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected first-stage registered sibling D paths to depend on helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_instance_outputs > 0,
            "expected registered sibling routes to chain through parent-local Qs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances > 0,
            "expected a later registered sibling route to chain from a helper-sourced parent Q for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_child_input_bindings_from_registered_multistage_parent_cone_instances > 0,
            "expected top-level multi-stage helper-sourced registered sibling bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_composed_logic, 0,
            "this focused config should prove direct registered sibling helper chaining, not registered parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic, 0,
            "this focused config should prove direct registered sibling helper chaining, not multi-stage parent-composed D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0,
            "expected non-zero multi-stage helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_registered_multistage_parent_cone_instance_child_input_binding_fraction
                > 0.0,
            "expected non-zero top-level multi-stage helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instances should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "multi-stage registered parent-composed helper hierarchy config should be valid",
        );
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "multi-stage registered parent-composed helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_composed_logic > 0,
            "expected first-stage registered parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic > 0,
            "expected registered parent-composed routes to chain through parent-local Qs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_parent_cone_instances > 0,
            "expected registered parent-composed D paths to depend on helper outputs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances,
            0,
            "this focused config should prove parent-composed helper chaining, not direct registered sibling helper chaining, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                > 0,
            "expected a later registered parent-composed route to chain from a helper-sourced parent Q for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                > 0,
            "expected top-level multi-stage parent-composed helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction
                > 0.0,
            "expected non-zero multi-stage parent-composed helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction
                > 0.0,
            "expected non-zero top-level multi-stage parent-composed helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instances should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 4,
            max_child_instances_per_module: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 1.0,
            hierarchy_child_input_cone_prob: 0.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 0.0,
            max_flops_per_module: 8,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            max_depth: 4,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate().expect(
            "recursive multi-stage registered parent-composed helper hierarchy config should be valid",
        );

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive multi-stage registered parent-composed helper hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected registered non-top parent-composed routes to create non-top parent-local flops for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_registered_multistage_parent_composed_logic
                > metrics.top_child_input_bindings_from_registered_multistage_parent_composed_logic,
            "expected non-top multi-stage registered parent-composed bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_multistage_parent_cone_instances,
            0,
            "this focused config should prove parent-composed helper chaining, not direct registered sibling helper chaining, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                > metrics
                    .top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances,
            "expected non-top multi-stage parent-composed helper bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics
                .registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction
                > 0.0,
            "expected non-zero multi-stage parent-composed helper binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_parent_composed_helper_routes_can_use_parent_flops() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("parent-composed helper-through-parent-flop hierarchy config should be valid");
        let planned_child_instances = cfg.num_child_instances as usize;

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "parent-composed helper-through-parent-flop hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert!(
            metrics.top_parent_cone_instances > 0,
            "expected top-level helper instances for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_composed_logic > 0,
            "expected parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances > 0,
            "expected helper-sourced parent-composed child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances_through_parent_flops > 0,
            "expected parent-composed helper child inputs to read helper outputs through parent-local Qs for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_child_input_bindings_from_parent_cone_instances_through_parent_flops > 0,
            "expected top-level helper-through-parent-flop child-input bindings for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances,
            0,
            "this focused config should prove stateful parent-composed helper inputs, not registered child-input helper D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.parent_cone_instance_flop_child_input_binding_fraction > 0.0,
            "expected non-zero helper-through-parent-flop binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.top_parent_cone_instance_flop_child_input_binding_fraction > 0.0,
            "expected non-zero top-level helper-through-parent-flop binding fraction for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.num_instances > planned_child_instances,
            "helper instances should be additional to planned child slots for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let cfg = Config {
            seed: 42,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            flop_prob: 0.0,
            hierarchy_sibling_route_prob: 0.0,
            hierarchy_registered_sibling_route_prob: 0.0,
            hierarchy_registered_child_input_cone_prob: 0.0,
            hierarchy_child_input_cone_prob: 1.0,
            hierarchy_parent_cone_instance_prob: 1.0,
            max_parent_cone_instances_per_module: 1,
            hierarchy_parent_flop_prob: 1.0,
            max_flops_per_module: 64,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            min_width: 1,
            max_width: 8,
            max_depth: 1,
            construction_strategy: strategy,
            ..Config::default()
        };
        cfg.validate()
            .expect("recursive helper-through-parent-flop hierarchy config should be valid");

        let mut g = Generator::new(cfg);
        let design = g.generate_design();
        anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
            panic!(
                "recursive helper-through-parent-flop hierarchy strategy {:?}: design validation failed: {}",
                strategy, e
            );
        });

        let metrics = anvil::metrics::compute_design(&design);
        assert_eq!(metrics.realized_min_leaf_depth, 2);
        assert_eq!(metrics.realized_max_leaf_depth, 2);
        assert!(
            metrics.num_internal_module_occurrences > 1,
            "expected a recursive hierarchy with non-top internal parents for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_cone_instances > metrics.top_parent_cone_instances,
            "expected at least one non-top helper instance for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.hierarchy_parent_local_flops > metrics.top_local_flops,
            "expected at least one non-top parent-local flop for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert!(
            metrics.child_input_bindings_from_parent_cone_instances_through_parent_flops
                > metrics.top_child_input_bindings_from_parent_cone_instances_through_parent_flops,
            "expected a non-top parent-composed helper child-input route through parent-local state for strategy {:?}: {metrics:#?}",
            strategy,
        );
        assert_eq!(
            metrics.child_input_bindings_from_registered_parent_cone_instances,
            0,
            "this focused config should prove recursive stateful parent-composed helper inputs, not registered child-input helper D cones, for strategy {:?}: {metrics:#?}",
            strategy,
        );
    }
}

#[test]
fn hierarchy_child_inputs_can_be_registered_from_parent_composed_logic() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_registered_parent_composed_route = false;
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                hierarchy_depth: 1,
                num_leaf_modules: 2,
                num_child_instances: 4,
                flop_prob: 0.0,
                hierarchy_sibling_route_prob: 0.0,
                hierarchy_registered_sibling_route_prob: 0.0,
                hierarchy_registered_child_input_cone_prob: 1.0,
                hierarchy_child_input_cone_prob: 0.0,
                hierarchy_parent_flop_prob: 0.0,
                max_flops_per_module: 8,
                max_depth: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            cfg.validate()
                .expect("registered parent-composed hierarchy config should be valid");

            let mut g = Generator::new(cfg);
            let design = g.generate_design();
            anvil::ir::validate::validate_design(&design).unwrap_or_else(|e| {
                panic!(
                    "registered parent-composed hierarchy strategy {:?} seed {}: design validation failed: {}",
                    strategy, seed, e
                );
            });

            let metrics = anvil::metrics::compute_design(&design);
            if metrics.child_input_bindings_from_registered_parent_composed_logic > 0 {
                assert!(
                    metrics.top_child_input_bindings_from_registered_parent_composed_logic > 0,
                    "strategy {:?} seed {} should expose registered parent-composed child inputs at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.child_input_bindings_from_registered_instance_outputs > 0,
                    "strategy {:?} seed {} should also prove that registered route reaches sibling outputs: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.child_input_bindings_from_registered_mixed_support > 0,
                    "strategy {:?} seed {} should prove the registered route can mix parent ports with sibling outputs: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_child_input_bindings_from_registered_mixed_support > 0,
                    "strategy {:?} seed {} should expose registered mixed-support routing at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics
                        .child_input_bindings_from_registered_multistage_parent_composed_logic
                        > 0,
                    "strategy {:?} seed {} should prove multi-stage registered parent-composed routing: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics
                        .top_child_input_bindings_from_registered_multistage_parent_composed_logic
                        > 0,
                    "strategy {:?} seed {} should expose multi-stage registered parent-composed routing at the top: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.child_input_bindings_from_parent_flops > 0,
                    "strategy {:?} seed {} should count the parent-local flop leg: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.registered_parent_composed_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero registered parent-composed routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_registered_parent_composed_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level registered parent-composed routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.registered_mixed_support_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero registered mixed-support routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.top_registered_mixed_support_child_input_binding_fraction > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level registered mixed-support routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics
                        .registered_multistage_parent_composed_child_input_binding_fraction
                        > 0.0,
                    "strategy {:?} seed {} should report a non-zero multi-stage registered parent-composed routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics
                        .top_registered_multistage_parent_composed_child_input_binding_fraction
                        > 0.0,
                    "strategy {:?} seed {} should report a non-zero top-level multi-stage registered parent-composed routing fraction: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert!(
                    metrics.hierarchy_parent_local_flops > 0,
                    "strategy {:?} seed {} should report local parent state: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_clock_inputs, 1,
                    "strategy {:?} seed {} should emit top clk for registered parent-composed routing: {metrics:#?}",
                    strategy,
                    seed,
                );
                assert_eq!(
                    metrics.top_reset_inputs, 1,
                    "strategy {:?} seed {} should emit top rst_n for registered parent-composed routing: {metrics:#?}",
                    strategy,
                    seed,
                );
                saw_registered_parent_composed_route = true;
                break;
            }
        }
        assert!(
            saw_registered_parent_composed_route,
            "expected at least one registered parent-composed hierarchy design across the 32-seed sweep for strategy {:?}",
            strategy,
        );
    }
}

#[test]
fn reproducibility() {
    let cfg = Config {
        seed: 12345,
        ..Config::default()
    };
    let a = anvil::emit::to_sv(&Generator::new(cfg.clone()).generate_module());
    let b = anvil::emit::to_sv(&Generator::new(cfg).generate_module());
    assert_eq!(a, b, "same seed must produce byte-identical output");
}

#[test]
fn shuffled_reproducibility() {
    // Shuffled must also be deterministic in (seed, knobs).
    let cfg = Config {
        seed: 42,
        min_outputs: 4,
        max_outputs: 4,
        construction_strategy: ConstructionStrategy::Shuffled,
        ..Config::default()
    };
    let a = anvil::emit::to_sv(&Generator::new(cfg.clone()).generate_module());
    let b = anvil::emit::to_sv(&Generator::new(cfg).generate_module());
    assert_eq!(
        a, b,
        "shuffled strategy must still be byte-identical for same seed"
    );
}

#[test]
fn shuffled_differs_from_sequential() {
    // With 4 outputs the shuffle of the build order is overwhelmingly
    // likely to pick a non-identity permutation, which reorders RNG
    // consumption and produces different emitted SV. If this test
    // ever flakes, widen `max_outputs` or try multiple seeds — but on
    // seed 42 with 4 outputs it is deterministic by design.
    let base = Config {
        seed: 42,
        min_outputs: 4,
        max_outputs: 4,
        ..Config::default()
    };
    let seq_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::Sequential,
            ..base.clone()
        })
        .generate_module(),
    );
    let shuf_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::Shuffled,
            ..base
        })
        .generate_module(),
    );
    assert_ne!(
        seq_sv, shuf_sv,
        "shuffled must produce different output from sequential on a multi-output seed"
    );
}

#[test]
fn all_strategies_produce_valid_modules() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..10u64 {
            let cfg = Config {
                seed,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "strategy {:?} seed {}: IR validation failed: {}",
                    strategy, seed, e
                )
            });
        }
    }
}

#[test]
fn graph_first_alias_matches_default_interleaved() {
    // `GraphFirst` is a deprecated alias for the current default
    // `Interleaved` strategy. Omitting the flag and explicitly passing
    // graph-first must therefore produce byte-identical output.
    let default_cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let explicit_cfg = Config {
        seed: 42,
        construction_strategy: ConstructionStrategy::GraphFirst,
        ..Config::default()
    };
    let a = anvil::emit::to_sv(&Generator::new(default_cfg).generate_module());
    let b = anvil::emit::to_sv(&Generator::new(explicit_cfg).generate_module());
    assert_eq!(
        a, b,
        "graph-first alias must match the default interleaved strategy"
    );
}

#[test]
fn graph_first_reproducibility() {
    let cfg = Config {
        seed: 42,
        construction_strategy: ConstructionStrategy::GraphFirst,
        ..Config::default()
    };
    let a = anvil::emit::to_sv(&Generator::new(cfg.clone()).generate_module());
    let b = anvil::emit::to_sv(&Generator::new(cfg).generate_module());
    assert_eq!(
        a, b,
        "graph-first strategy must be byte-identical for same seed"
    );
}

#[test]
fn coefficient_motif_emits_compound_shapes() {
    // With coefficient_prob = 1.0 every Add/Sub/Mul emission takes the
    // linear-combination compound form. On a non-trivial seed sweep
    // we expect to see:
    //   - signal*const 2-arity Mul patterns (feeding Add/Sub roots)
    //   - N-arity Add of product terms (top-level Add compound)
    //   - chained 2-arity Sub of product terms (top-level Sub compound)
    //   - N+1-arity Mul with a front constant (top-level Mul compound)
    // Over a multi-seed sweep at least one seed produces a Mul with a
    // leading constant operand like `<width>'h<hex> * ...`. This
    // confirms the motif dispatches on Mul as well as Add/Sub.
    let mut saw_front_const_mul = false;
    for seed in 0..16u64 {
        let cfg = Config {
            seed,
            coefficient_prob: 1.0,
            min_outputs: 2,
            max_outputs: 2,
            graph_first_pool_size: 48,
            construction_strategy: ConstructionStrategy::GraphFirst,
            ..Config::default()
        };
        let m = Generator::new(cfg).generate_module();
        let sv = anvil::emit::to_sv(&m);
        // Look for `<width>'h... * w_` — a constant operand at the start
        // of a multi-operand Mul expression.
        for line in sv.lines() {
            if let Some(assign_rhs) = line.trim().strip_prefix("assign ") {
                // Very loose pattern: "N'h<hex> * w_" or "N'h<hex> * i_"
                // early in an expression suggests a front-coefficient Mul.
                if assign_rhs.contains("'h")
                    && assign_rhs.contains(" * ")
                    && assign_rhs.matches(" * ").count() >= 2
                {
                    // Heuristic: if the first operand after '=' is a
                    // constant literal and there are >= 2 '*' operators,
                    // this is a front-coef Mul.
                    if let Some(eq_rhs) = assign_rhs.split_once('=').map(|(_, r)| r.trim_start()) {
                        if eq_rhs.starts_with(|c: char| c.is_ascii_digit()) && eq_rhs.contains("'h")
                        {
                            saw_front_const_mul = true;
                            break;
                        }
                    }
                }
            }
        }
        if saw_front_const_mul {
            break;
        }
    }
    assert!(
        saw_front_const_mul,
        "expected at least one Mul compound (c * s1 * s2 ...) across the seed sweep"
    );
}

#[test]
fn const_shift_amount_appears_in_output() {
    // With const_shift_amount_prob = 1.0, every Shl/Shr picked by
    // pick_gate emits `value << const` / `value >> const`. Verify at
    // least one seed produces such a pattern. We bias gate_shift_weight
    // so shifts are frequently picked.
    let mut saw_shift_const = false;
    for seed in 0..32u64 {
        let cfg = Config {
            seed,
            const_shift_amount_prob: 1.0,
            gate_shift_weight: 10,
            min_outputs: 2,
            max_outputs: 2,
            min_width: 4,
            max_width: 8,
            graph_first_pool_size: 48,
            construction_strategy: ConstructionStrategy::GraphFirst,
            ..Config::default()
        };
        let m = Generator::new(cfg).generate_module();
        let sv = anvil::emit::to_sv(&m);
        for line in sv.lines() {
            // "<< N'hX" or ">> N'hX" immediately after the shift operator
            if line.contains(" << ") && line.contains("'h") {
                saw_shift_const = true;
                break;
            }
            if line.contains(" >> ") && line.contains("'h") {
                saw_shift_const = true;
                break;
            }
        }
        if saw_shift_const {
            break;
        }
    }
    assert!(
        saw_shift_const,
        "expected at least one constant-shift-amount emission across the 32-seed sweep"
    );
}

#[test]
fn variable_shift_amount_appears_in_output() {
    // We want proof of the variable-shift surface, not reliance on one
    // lucky seed. Sweep a small shift-only corpus and demand that at
    // least one final module still contains a non-constant shift rhs in
    // both IR and emitted SV.
    let mut saw_variable_shift = false;
    for seed in 0..32u64 {
        let cfg = Config {
            seed,
            min_inputs: 2,
            max_inputs: 2,
            min_outputs: 2,
            max_outputs: 2,
            min_width: 4,
            max_width: 8,
            max_depth: 2,
            flop_prob: 0.0,
            share_prob: 0.0,
            terminal_reuse_prob: 1.0,
            constant_prob: 0.0,
            gate_bitwise_weight: 0,
            gate_arith_weight: 0,
            gate_struct_weight: 0,
            gate_compare_weight: 0,
            gate_reduce_weight: 0,
            coefficient_prob: 0.0,
            const_shift_amount_prob: 0.0,
            gate_shift_weight: 10,
            const_comparand_prob: 0.0,
            priority_encoder_prob: 0.0,
            case_mux_prob: 0.0,
            casez_mux_prob: 0.0,
            for_fold_prob: 0.0,
            max_flops_per_module: 0,
            comb_mux_prob: 0.0,
            construction_strategy: ConstructionStrategy::GraphFirst,
            graph_first_pool_size: 48,
            ..Config::default()
        };

        let m = Generator::new(cfg).generate_module();
        let saw_variable_shift_ir = m.nodes.iter().any(|node| match node {
            Node::Gate {
                op: GateOp::Shl | GateOp::Shr,
                operands,
                ..
            } => !matches!(m.nodes[operands[1] as usize], Node::Constant { .. }),
            _ => false,
        });
        if !saw_variable_shift_ir {
            continue;
        }

        let sv = anvil::emit::to_sv(&m);
        let saw_variable_shift_sv = sv.lines().any(|line| {
            [" << ", " >> "].iter().any(|op| {
                line.split_once(op)
                    .map(|(_, rhs)| !rhs.trim_end_matches(';').trim().contains("'h"))
                    .unwrap_or(false)
            })
        });
        if saw_variable_shift_sv {
            saw_variable_shift = true;
            break;
        }
    }

    assert!(
        saw_variable_shift,
        "expected at least one emitted variable shift across the 32-seed sweep"
    );
}

#[test]
fn priority_encoder_block_across_all_strategies_is_valid() {
    // priority_encoder_prob = 1.0 with a reasonable arm range. All four
    // strategies must produce IR-valid modules; the PE's dispatch
    // helper gracefully falls through when target width isn't
    // compatible with any N in the arity range.
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                priority_encoder_prob: 1.0,
                min_mux_arms: 3,
                max_mux_arms: 5,
                max_depth: 3, // keep test runtime bounded under PE recursion
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "priority_encoder_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
        }
    }
}

#[test]
fn case_mux_block_across_all_strategies_emits_always_comb_case() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                case_mux_prob: 1.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                comb_mux_prob: 0.0,
                priority_encoder_prob: 0.0,
                flop_prob: 0.0,
                max_depth: 3,
                min_mux_arms: 2,
                max_mux_arms: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "case_mux_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
            let sv = anvil::emit::to_sv(&m);
            assert!(
                sv.contains("always_comb begin") && sv.contains("case ("),
                "case_mux_prob=1.0 strategy {:?} seed {} should emit always_comb case",
                strategy,
                seed
            );
        }
    }
}

#[test]
fn casez_mux_block_across_all_strategies_emits_always_comb_casez() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                case_mux_prob: 0.0,
                casez_mux_prob: 1.0,
                for_fold_prob: 0.0,
                comb_mux_prob: 0.0,
                priority_encoder_prob: 0.0,
                flop_prob: 0.0,
                max_depth: 3,
                min_mux_arms: 2,
                max_mux_arms: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "casez_mux_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
            let sv = anvil::emit::to_sv(&m);
            assert!(
                sv.contains("always_comb begin") && sv.contains("casez ("),
                "casez_mux_prob=1.0 strategy {:?} seed {} should emit always_comb casez",
                strategy,
                seed
            );
        }
    }
}

#[test]
fn for_fold_block_across_all_strategies_emits_bounded_always_comb_for() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 1.0,
                constant_prob: 0.0,
                coefficient_prob: 0.0,
                const_shift_amount_prob: 0.0,
                const_comparand_prob: 0.0,
                comb_mux_prob: 0.0,
                priority_encoder_prob: 0.0,
                flop_prob: 0.0,
                max_depth: 3,
                min_width: 2,
                max_width: 8,
                min_gate_arity: 2,
                max_gate_arity: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "for_fold_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
            let sv = anvil::emit::to_sv(&m);
            assert!(
                sv.contains("always_comb begin") && sv.contains("for (int i = 0; i < "),
                "for_fold_prob=1.0 strategy {:?} seed {} should emit always_comb for-loop",
                strategy,
                seed
            );
        }
    }
}

#[test]
fn slice_and_concat_are_selectable_surfaces_across_all_strategies() {
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        let mut saw_slice = false;
        let mut saw_concat = false;
        for seed in 0..64u64 {
            let cfg = Config {
                seed,
                gate_bitwise_weight: 0,
                gate_arith_weight: 0,
                gate_struct_weight: 1,
                gate_compare_weight: 0,
                gate_reduce_weight: 0,
                gate_shift_weight: 0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                coefficient_prob: 0.0,
                const_shift_amount_prob: 0.0,
                const_comparand_prob: 0.0,
                constant_prob: 0.0,
                comb_mux_prob: 0.0,
                priority_encoder_prob: 0.0,
                flop_prob: 0.0,
                min_width: 4,
                max_width: 8,
                min_outputs: 2,
                max_outputs: 2,
                max_depth: 4,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!("slice/concat strategy {:?} seed {}: {e}", strategy, seed)
            });
            for node in &m.nodes {
                if let Node::Gate { op, .. } = node {
                    saw_slice |= matches!(op, GateOp::Slice { .. });
                    saw_concat |= matches!(op, GateOp::Concat);
                }
            }
            if saw_slice && saw_concat {
                break;
            }
        }
        assert!(
            saw_slice,
            "strategy {:?} should emit a live selectable Slice across the seed sweep",
            strategy
        );
        assert!(
            saw_concat,
            "strategy {:?} should emit a live selectable Concat across the seed sweep",
            strategy
        );
    }
}

#[test]
fn const_comparand_across_all_strategies_is_valid() {
    // const_comparand_prob = 1.0: every comparison picks a constant
    // RHS. Verify all four strategies still produce IR-valid modules.
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                const_comparand_prob: 1.0,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "const_comparand_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
        }
    }
}

#[test]
fn coefficient_motif_across_all_strategies() {
    // Every strategy must produce valid modules with coefficient_prob=1.0.
    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in 0..5u64 {
            let cfg = Config {
                seed,
                coefficient_prob: 1.0,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m).unwrap_or_else(|e| {
                panic!(
                    "coefficient_prob=1.0 strategy {:?} seed {}: {e}",
                    strategy, seed
                )
            });
        }
    }
}

#[test]
fn graph_first_alias_differs_from_sequential() {
    let base = Config {
        seed: 42,
        min_outputs: 3,
        max_outputs: 3,
        ..Config::default()
    };
    let seq_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::Sequential,
            ..base.clone()
        })
        .generate_module(),
    );
    let gf_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::GraphFirst,
            ..base
        })
        .generate_module(),
    );
    assert_ne!(
        seq_sv, gf_sv,
        "the graph-first alias (interleaved) must differ from sequential"
    );
}

#[test]
fn interleaved_reproducibility() {
    let cfg = Config {
        seed: 42,
        min_outputs: 3,
        max_outputs: 3,
        construction_strategy: ConstructionStrategy::Interleaved,
        ..Config::default()
    };
    let a = anvil::emit::to_sv(&Generator::new(cfg.clone()).generate_module());
    let b = anvil::emit::to_sv(&Generator::new(cfg).generate_module());
    assert_eq!(
        a, b,
        "interleaved strategy must still be byte-identical for same seed"
    );
}

#[test]
fn interleaved_differs_from_sequential() {
    // Same construction knobs, same seed; different strategy should
    // produce different emitted SV on a multi-output seed because the
    // order in which gates are created is fundamentally different
    // (global frame-queue pops vs declaration-order depth-first).
    let base = Config {
        seed: 42,
        min_outputs: 3,
        max_outputs: 3,
        ..Config::default()
    };
    let seq_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::Sequential,
            ..base.clone()
        })
        .generate_module(),
    );
    let ileaved_sv = anvil::emit::to_sv(
        &Generator::new(Config {
            construction_strategy: ConstructionStrategy::Interleaved,
            ..base
        })
        .generate_module(),
    );
    assert_ne!(
        seq_sv, ileaved_sv,
        "interleaved must produce different output from sequential"
    );
}

/// Regression guard for Rule 18 (no orphan gates). The generator
/// must produce modules whose every `Node::Gate` has at least one
/// consumer in the emitted design (other gate's operand, flop D, or
/// output drive). Measured across all four strategy values at several
/// seeds.
#[test]
fn zero_orphans_at_default_knobs() {
    use anvil::ir::Node;
    let strategies = [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ];
    for strat in strategies {
        for seed in [1u64, 42, 100, 777, 9999, 12345] {
            let cfg = Config {
                seed,
                construction_strategy: strat,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();

            // Mark every NodeId referenced by any gate operand,
            // flop field, or output drive.
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
            let orphans: Vec<usize> = m
                .nodes
                .iter()
                .enumerate()
                .filter(|(i, n)| matches!(n, Node::Gate { .. }) && !used[*i])
                .map(|(i, _)| i)
                .collect();
            assert!(
                orphans.is_empty(),
                "strategy={:?} seed={}: {} orphan gate(s) at NodeIds {:?}",
                strat,
                seed,
                orphans.len(),
                orphans
            );
        }
    }
}

#[test]
fn no_unused_primary_data_inputs_remain_after_finalisation() {
    use std::collections::BTreeSet;

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
        ConstructionStrategy::GraphFirst,
    ] {
        for seed in [1u64, 42, 100, 777, 9999, 12345] {
            let cfg = Config {
                seed,
                construction_strategy: strategy,
                ..Config::default()
            };
            let m = Generator::new(cfg).generate_module();
            let live_inputs: BTreeSet<_> = m
                .nodes
                .iter()
                .filter_map(|node| match node {
                    anvil::ir::Node::PrimaryInput { port, .. } => Some(*port),
                    _ => None,
                })
                .collect();
            for port in &m.inputs {
                let is_clock = m.clock == Some(port.id);
                let is_reset = m.reset == Some(port.id);
                if is_clock || is_reset {
                    continue;
                }
                assert!(
                    live_inputs.contains(&port.id),
                    "strategy={:?} seed={}: input {} ({}) survived finalisation without any live PrimaryInput node",
                    strategy,
                    seed,
                    port.id,
                    port.name
                );
            }
        }
    }
}

/// Regression guard for the factorization chain at its
/// currently-implemented ceiling (CSE + operand uniqueness +
/// commutative). At the default `operand_duplication_rate = 0.0`,
/// no gate of `And`/`Or`/`Xor`/`Add`/`Mul` may have a duplicate
/// `NodeId` in its operand list.
#[test]
fn zero_duplicate_operands_at_default_knobs() {
    use anvil::ir::{GateOp, Node};
    for seed in [1u64, 42, 100, 777, 9999] {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        let m = Generator::new(cfg).generate_module();
        for (idx, node) in m.nodes.iter().enumerate() {
            if let Node::Gate { op, operands, .. } = node {
                if !matches!(
                    op,
                    GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
                ) {
                    continue;
                }
                let mut seen = std::collections::HashSet::new();
                for &o in operands {
                    assert!(
                        seen.insert(o),
                        "seed={} node={} op={:?}: duplicate operand NodeId {} in {:?}",
                        seed,
                        idx,
                        op,
                        o,
                        operands
                    );
                }
            }
        }
    }
}

/// Doctrine guard for the `nested_associative_operand_count`
/// metric. **After** the Associative factorization layer landed
/// (slice 2026-04-17-0070), this count must be zero at default
/// knobs: every same-op same-width inner gate operand that is
/// flattenable under the current duplicate policy is spliced in at
/// intern time, so the final IR contains no remaining *legal*
/// flattening opportunities. The count can only become non-zero if
/// the Associative layer regresses OR the generator introduces a
/// construction path that materialises a nested associative shape
/// after intern (e.g. a post-hoc transform, not present today).
///
/// Residual nested `Add`/`Mul` shapes whose flattening would create
/// duplicate operands do not count here; the live Associative layer
/// intentionally preserves them at strict `operand_duplication_rate`
/// to avoid changing semantics (`x + (x + y)` is not `x + y`).
#[test]
fn nested_associative_opportunities_flatten_to_zero() {
    let cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let m = Generator::new(cfg).generate_module();
    let metrics = anvil::metrics::compute(&m);
    assert_eq!(
        metrics.nested_associative_operand_count, 0,
        "expected zero nested-associative opportunities at default \
         knobs with Associative layer live; got {}. The Associative \
         factorization layer may have regressed.",
        metrics.nested_associative_operand_count
    );
}

/// Doctrine guard: the `ConstantFold` factorization layer is live at
/// default knobs. Zero-valued constants fed into additive/XOR/shift
/// positions, one-valued constants into multiplicative positions,
/// and all-ones constants into AND positions must fold at intern
/// time — the counter surfaces each fire. A seed sweep at default
/// knobs should produce at least one fire over a modest range;
/// otherwise either the constant_prob knob stopped producing
/// identity-value constants or the fold layer regressed.
#[test]
fn constant_fold_layer_fires_at_default_knobs() {
    let mut total_fires: u64 = 0;
    for seed in 0..40u64 {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        let m = anvil::Generator::new(cfg).generate_module();
        let metrics = anvil::metrics::compute(&m);
        total_fires += metrics.fold_identities_applied;
    }
    assert!(
        total_fires > 0,
        "expected at least one ConstantFold fire across 40 seeds at \
         default knobs; got 0. Either the ConstantFold layer regressed \
         or constant_prob no longer produces identity-value constants."
    );
}

/// Doctrine guard: the `Peephole` factorization layer is live at
/// default knobs. A seed sweep should produce at least one local
/// rewrite — most commonly a fully-constant comparison evaluated
/// at intern time (the `const-comparand` motif lands both LHS and
/// RHS as constants after CSE), or a full-width `Slice` / single-
/// operand `Concat` identity. A zero count across 40 seeds means
/// the layer regressed or no peephole-reachable shape is being
/// generated.
#[test]
fn peephole_layer_fires_at_default_knobs() {
    let mut total_fires: u64 = 0;
    for seed in 0..40u64 {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        let m = anvil::Generator::new(cfg).generate_module();
        let metrics = anvil::metrics::compute(&m);
        total_fires += metrics.peephole_rewrites_applied;
    }
    assert!(
        total_fires > 0,
        "expected at least one Peephole fire across 40 seeds at \
         default knobs; got 0. Either the Peephole layer regressed \
         or no peephole-reachable shape (Not(Not(x)), const-const \
         comparison, full-width Slice, single-operand Concat) is \
         being produced."
    );
}

/// Doctrine guard: the `compact_node_ids` pass keeps Rule 18
/// (zero orphan gates) holding across all strategies and seeds,
/// and records a non-zero `nodes_compacted` count whenever the
/// Not(Not(x)) peephole actually fires (we'd expect this at least
/// once across a 40-seed sweep given how common Not chains are
/// through CSE). If `nodes_compacted` is always zero, either the
/// peephole regressed or compaction itself became a no-op in all
/// paths.
#[test]
fn compaction_preserves_rule_18_and_records_removals() {
    let mut total_compacted: u32 = 0;
    for seed in 0..40u64 {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        let m = anvil::Generator::new(cfg).generate_module();
        let metrics = anvil::metrics::compute(&m);
        total_compacted += metrics.nodes_compacted;

        // Rule 18 holds post-compaction for the emitted design.
        use anvil::ir::Node;
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
            used[f.q as usize] = true;
        }
        for (_, root) in &m.drives {
            used[*root as usize] = true;
        }
        let orphans: Vec<usize> = m
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, n)| matches!(n, Node::Gate { .. }) && !used[*i])
            .map(|(i, _)| i)
            .collect();
        assert!(
            orphans.is_empty(),
            "seed={}: {} orphan gate(s) after compaction: {:?}",
            seed,
            orphans.len(),
            orphans
        );

        // Validator must still accept post-compaction IR.
        anvil::ir::validate::validate(&m)
            .unwrap_or_else(|e| panic!("seed={} validator rejects post-compaction IR: {e}", seed));
    }
    // Across 40 seeds at default knobs, Not(Not) should fire at
    // least once and compaction should register it.
    assert!(
        total_compacted > 0,
        "expected compaction to remove at least one node across \
         40 seeds at default knobs; got 0. Either the Not(Not(x)) \
         peephole regressed or compact_node_ids became a no-op."
    );
}

/// Doctrine guard: every probability knob that the generator
/// actually consults must show up in `knob_roll_attempts`, and the
/// empirical fire-rate should be bounded by the configured
/// probability (with some slack to allow for sampling noise). This
/// is the measurability doctrine in test form — if a knob stops
/// firing, or stops being rolled, the generator has regressed.
#[test]
fn knob_rolls_recorded_across_seeds() {
    // Aggregate attempts+fires over a sweep so we get enough
    // samples to see every probability knob at least once.
    let mut total_attempts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut total_fires: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for seed in 0..20u64 {
        let cfg = Config {
            seed,
            ..Config::default()
        };
        let m = anvil::Generator::new(cfg).generate_module();
        let metrics = anvil::metrics::compute(&m);
        for (k, v) in &metrics.knob_roll_attempts {
            *total_attempts.entry(k.clone()).or_insert(0) += v;
        }
        for (k, v) in &metrics.knob_roll_fires {
            *total_fires.entry(k.clone()).or_insert(0) += v;
        }
    }

    // Every probability knob whose default is > 0 should log
    // attempts. `priority_encoder_prob` default is 0.05 so even
    // with seed variation we expect attempts across 20 seeds.
    let expected_knobs = [
        "flop_prob",
        "comb_mux_prob",
        "case_mux_prob",
        "casez_mux_prob",
        "for_fold_prob",
        "priority_encoder_prob",
        "coefficient_prob",
        "const_shift_amount_prob",
        "const_comparand_prob",
        "constant_prob",
        "terminal_reuse_prob",
        "comb_mux_encoding_prob",
        "flop_mux_encoding_prob",
        "share_prob",
        "flop_qfeedback_prob",
    ];
    for knob in expected_knobs {
        let attempts = total_attempts.get(knob).copied().unwrap_or(0);
        assert!(
            attempts > 0,
            "expected knob {knob} to be rolled at least once across 20 seeds; \
             got 0 attempts. Either the knob is no longer consulted or its \
             roll site is unreachable at default knobs."
        );
        // Fires must never exceed attempts.
        let fires = total_fires.get(knob).copied().unwrap_or(0);
        assert!(
            fires <= attempts,
            "knob {knob}: fires ({fires}) > attempts ({attempts}) — bookkeeping bug"
        );
    }
}

#[test]
fn gate_categories_are_exercisable_end_to_end() {
    use std::collections::BTreeSet;

    let category_runs = [
        (
            "bitwise",
            Config {
                gate_bitwise_weight: 1,
                gate_arith_weight: 0,
                gate_struct_weight: 0,
                gate_compare_weight: 0,
                gate_reduce_weight: 0,
                gate_shift_weight: 0,
                min_width: 4,
                max_width: 4,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["and", "or", "xor", "not"].map(str::to_string)),
        ),
        (
            "arith",
            Config {
                gate_bitwise_weight: 0,
                gate_arith_weight: 1,
                gate_struct_weight: 0,
                gate_compare_weight: 0,
                gate_reduce_weight: 0,
                gate_shift_weight: 0,
                min_width: 4,
                max_width: 4,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["add", "sub", "mul"].map(str::to_string)),
        ),
        (
            "struct",
            Config {
                gate_bitwise_weight: 0,
                gate_arith_weight: 0,
                gate_struct_weight: 1,
                gate_compare_weight: 0,
                gate_reduce_weight: 0,
                gate_shift_weight: 0,
                min_width: 4,
                max_width: 4,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["mux"].map(str::to_string)),
        ),
        (
            "compare",
            Config {
                gate_bitwise_weight: 0,
                gate_arith_weight: 0,
                gate_struct_weight: 0,
                gate_compare_weight: 1,
                gate_reduce_weight: 0,
                gate_shift_weight: 0,
                min_width: 1,
                max_width: 1,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["eq", "neq", "lt", "gt", "le", "ge"].map(str::to_string)),
        ),
        (
            "reduce",
            Config {
                gate_bitwise_weight: 0,
                gate_arith_weight: 0,
                gate_struct_weight: 0,
                gate_compare_weight: 0,
                gate_reduce_weight: 1,
                gate_shift_weight: 0,
                min_width: 1,
                max_width: 1,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["red_and", "red_or", "red_xor"].map(str::to_string)),
        ),
        (
            "shift",
            Config {
                gate_bitwise_weight: 0,
                gate_arith_weight: 0,
                gate_struct_weight: 0,
                gate_compare_weight: 0,
                gate_reduce_weight: 0,
                gate_shift_weight: 1,
                min_width: 4,
                max_width: 4,
                flop_prob: 0.0,
                comb_mux_prob: 0.0,
                case_mux_prob: 0.0,
                casez_mux_prob: 0.0,
                for_fold_prob: 0.0,
                priority_encoder_prob: 0.0,
                coefficient_prob: 0.0,
                ..Config::default()
            },
            BTreeSet::from(["shl", "shr"].map(str::to_string)),
        ),
    ];

    for (name, base_cfg, expected) in category_runs {
        let mut seen = BTreeSet::new();
        for seed in 0..32u64 {
            let cfg = Config {
                seed,
                ..base_cfg.clone()
            };
            let m = Generator::new(cfg).generate_module();
            anvil::ir::validate::validate(&m)
                .unwrap_or_else(|e| panic!("category {name} seed {seed}: {e}"));
            let metrics = anvil::metrics::compute(&m);
            for kind in metrics.gates_by_kind.keys() {
                if expected.contains(kind) {
                    seen.insert(kind.clone());
                }
            }
            if seen == expected {
                break;
            }
        }
        assert_eq!(
            seen, expected,
            "category {name}: expected to exercise {expected:?}, saw {seen:?}"
        );
    }
}
