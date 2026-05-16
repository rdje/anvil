//! Module-level dedup pass.
//!
//! Optional, opt-in. When `Config::hierarchy_module_dedup` is `true`,
//! `Generator::generate_design` calls `dedup_modules` after the
//! per-Module finalisation passes (compact, flop-merge, gate-merge)
//! have run. The pass collapses every group of `Module`s in
//! `design.modules` that share a canonical structural signature to a
//! single surviving entry and rewrites every `Instance.module`
//! reference in the surviving Modules so they point at the survivor.
//!
//! See `DEVELOPMENT_NOTES.md` "Module-dedup pass design sketch
//! (2026-05-15, HIERARCHY-AWARE-IDENTITY.3)" for the design
//! rationale, rejected alternatives, and proof shape.
//!
//! The canonical signature is the same FNV-1a 64-bit hash used by
//! `DesignMetrics.canonical_module_signatures` (`HIERARCHY-AWARE-IDENTITY.1`
//! / r85). It deliberately excludes `Instance.module` and
//! `Instance.name`, so two structurally-identical Modules whose
//! children have distinct names still share a signature.

use crate::ir::{Design, Module};
use crate::metrics::canonical_module_signature;
use std::collections::{BTreeMap, HashMap};

/// Iteratively collapse `Module` definitions that share a canonical
/// structural signature. Returns the number of Modules removed across
/// all fixed-point iterations.
///
/// **Top module is never merged away.** When the top appears in a
/// signature group, it is forced to be the survivor (regardless of
/// the lex-smallest-name tiebreaker that applies to all other
/// groups).
///
/// **Termination.** Each iteration that performs at least one merge
/// strictly decreases `design.modules.len()`. The minimum possible
/// length is 1 (the top alone). The loop therefore terminates after
/// at most `initial_len - 1` iterations.
pub fn dedup_modules(design: &mut Design) -> usize {
    let mut total_removed = 0usize;
    loop {
        let removed_this_pass = dedup_modules_once(design);
        if removed_this_pass == 0 {
            break;
        }
        total_removed += removed_this_pass;
    }
    total_removed
}

/// One sweep of dedup over the current `design.modules`. Returns the
/// number of Modules removed during this sweep.
fn dedup_modules_once(design: &mut Design) -> usize {
    if design.modules.len() <= 1 {
        return 0;
    }
    // Group Modules by canonical signature (excluding the top so it
    // cannot be merged away under any circumstance).
    let top_name = design.top.clone();
    let mut groups: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (idx, module) in design.modules.iter().enumerate() {
        if module.name == top_name {
            continue;
        }
        let sig = canonical_module_signature(module);
        groups.entry(sig).or_default().push(idx);
    }

    // Build the rename map from merged-away names to survivor names.
    // For each group with >1 members the survivor is the one with the
    // lexicographically-smallest `Module.name` for determinism.
    let mut name_remap: HashMap<String, String> = HashMap::new();
    let mut indices_to_remove: Vec<usize> = Vec::new();
    for indices in groups.values() {
        if indices.len() < 2 {
            continue;
        }
        let survivor_idx = *indices
            .iter()
            .min_by(|a, b| design.modules[**a].name.cmp(&design.modules[**b].name))
            .expect("non-empty group");
        let survivor_name = design.modules[survivor_idx].name.clone();
        for &idx in indices {
            if idx == survivor_idx {
                continue;
            }
            let merged_name = design.modules[idx].name.clone();
            name_remap.insert(merged_name, survivor_name.clone());
            indices_to_remove.push(idx);
        }
    }
    if indices_to_remove.is_empty() {
        return 0;
    }

    // Rewrite Instance.module references in every surviving Module
    // (including the top). Indices to remove are dropped afterwards.
    rewrite_instance_module_names(&mut design.modules, &name_remap);

    // Remove merged-away Modules. Drop in descending order so earlier
    // indices stay valid as we go.
    indices_to_remove.sort_unstable();
    indices_to_remove.dedup();
    let removed = indices_to_remove.len();
    for idx in indices_to_remove.into_iter().rev() {
        design.modules.remove(idx);
    }

    removed
}

fn rewrite_instance_module_names(modules: &mut [Module], name_remap: &HashMap<String, String>) {
    if name_remap.is_empty() {
        return;
    }
    for module in modules.iter_mut() {
        for instance in module.instances.iter_mut() {
            if let Some(new_name) = name_remap.get(&instance.module) {
                instance.module = new_name.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Direction, Instance, InstanceRole, Node, Port};

    fn make_port(id: u32, name: &str, width: u32, dir: Direction) -> Port {
        Port {
            id,
            name: name.into(),
            width,
            dir,
        }
    }

    fn trivial_leaf(name: &str) -> Module {
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "in", 1, Direction::In));
        m.outputs.push(make_port(1, "out", 1, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.drives.push((1, 0));
        m
    }

    #[test]
    fn dedup_collapses_structurally_identical_leaves_under_a_top() {
        let leaf_a = trivial_leaf("leaf_a");
        let leaf_b = trivial_leaf("leaf_b");
        let leaf_c = trivial_leaf("leaf_c");
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        // Top instantiates each leaf once. The Instance.module fields
        // are what dedup rewrites.
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "leaf_a".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u1".into(),
            module: "leaf_b".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 2,
            name: "u2".into(),
            module: "leaf_c".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, leaf_c, top],
        };

        assert_eq!(design.modules.len(), 4);
        let removed = dedup_modules(&mut design);
        assert_eq!(
            removed, 2,
            "expected two of three identical leaves to be removed"
        );
        assert_eq!(design.modules.len(), 2, "expected top + one surviving leaf");

        // Survivor is the lex-smallest name: "leaf_a".
        let names: Vec<_> = design.modules.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"top"));
        assert!(names.contains(&"leaf_a"));

        // Every Instance.module in the surviving top now points at the
        // surviving leaf.
        let top_after = design
            .modules
            .iter()
            .find(|m| m.name == "top")
            .expect("top survived");
        assert!(top_after.instances.iter().all(|i| i.module == "leaf_a"));
    }

    #[test]
    fn dedup_is_a_no_op_when_modules_are_structurally_distinct() {
        let leaf_a = trivial_leaf("leaf_a");
        let mut leaf_b = trivial_leaf("leaf_b");
        // Make leaf_b structurally different: width-2 port instead of 1.
        leaf_b.inputs[0].width = 2;
        leaf_b.outputs[0].width = 2;
        leaf_b.nodes[0] = Node::PrimaryInput { port: 0, width: 2 };

        let top = Module {
            name: "top".into(),
            ..Module::default()
        };
        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };
        let before = design.modules.len();
        let removed = dedup_modules(&mut design);
        assert_eq!(removed, 0);
        assert_eq!(design.modules.len(), before);
    }

    #[test]
    fn dedup_never_removes_the_top_module() {
        // Construct an unusual case where the top and a "leaf" have
        // identical signatures (both empty Module shells). Dedup must
        // still preserve the top by name.
        let top = Module {
            name: "top".into(),
            ..Module::default()
        };
        let aux = Module {
            name: "aux".into(),
            ..Module::default()
        };
        let mut design = Design {
            top: "top".into(),
            modules: vec![top, aux],
        };
        // The grouping excludes the top by name. With only one
        // non-top Module, no group has >1 members, so nothing is
        // removed.
        let removed = dedup_modules(&mut design);
        assert_eq!(removed, 0);
        assert!(design.modules.iter().any(|m| m.name == "top"));
    }

    fn param_leaf(name: &str, w: u32) -> Module {
        use crate::ir::ParamEnv;
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "i", w, Direction::In));
        m.outputs.push(make_port(1, "o", w, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: w });
        m.drives.push((1, 0));
        m.param_env = Some(ParamEnv {
            name: "W".into(),
            min: 2,
            max: 16,
            design_value: w,
        });
        m.parameterized_input_ports = vec![0];
        m.parameterized_output_ports = vec![1];
        m
    }

    #[test]
    fn parameter_aware_identity_collapses_templates_differing_only_in_design_width() {
        // PHASE-5-PARAMETERIZATION.2.3: two structurally-identical
        // parameterizable templates that differ ONLY in their concrete
        // design_value are the same template and must share a
        // canonical signature (instances override the width via
        // `#(.W(v))`). A genuinely concrete module must NOT alias a
        // parameterized one — the param-presence marker disambiguates.
        let a = param_leaf("pa", 8);
        let b = param_leaf("pb", 16);
        assert_eq!(
            canonical_module_signature(&a),
            canonical_module_signature(&b),
            "parameterizable templates differing only in design_value must share a signature"
        );

        // Same structure, but concrete (no param_env): distinct.
        let mut c = param_leaf("c", 8);
        c.param_env = None;
        c.parameterized_input_ports.clear();
        c.parameterized_output_ports.clear();
        assert_ne!(
            canonical_module_signature(&a),
            canonical_module_signature(&c),
            "a parameterized template must never alias a structurally-identical concrete module"
        );

        // Different structure (extra width): still distinct even when
        // both are parameterized.
        let d = param_leaf("d", 8);
        let mut e = param_leaf("e", 8);
        e.outputs.push(make_port(2, "o2", 8, Direction::Out));
        assert_ne!(
            canonical_module_signature(&d),
            canonical_module_signature(&e),
            "structurally-different parameterized templates must not collide"
        );

        // dedup collapses the equal-signature pair (a @8, b @16) under
        // a top; the top is preserved by name.
        let top = Module {
            name: "top".into(),
            ..Module::default()
        };
        let mut design = Design {
            top: "top".into(),
            modules: vec![param_leaf("pa", 8), param_leaf("pb", 16), top],
        };
        let removed = dedup_modules(&mut design);
        assert_eq!(
            removed, 1,
            "the two equal-signature templates collapse to one"
        );
        assert_eq!(design.modules.len(), 2, "survivor + top");
        assert!(design.modules.iter().any(|m| m.name == "top"));
    }
}
