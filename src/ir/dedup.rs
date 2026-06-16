//! Module-level dedup pass.
//!
//! Optional, opt-in. When `Config::hierarchy_module_dedup` is `true`,
//! `Generator::generate_design` calls `dedup_modules` after the
//! per-Module finalisation passes (compact, flop-merge, gate-merge)
//! have run. The pass collapses every group of `Module`s in
//! `design.modules` that share a canonical structural signature to a
//! single surviving entry and rewrites every `Instance.module`
//! reference in the surviving Modules so they point at the survivor.
//! If at least one merge occurred, it then prunes module definitions
//! that were reachable before dedup but are no longer reachable from
//! `Design::top`. Pre-existing under-instantiated library definitions
//! are preserved.
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
//!
//! `dedup_semantic_modules` is separate and default-off. It uses a
//! bounded whole-module truth-table proof for pure-combinational
//! non-top Modules, including bounded wrappers whose children are
//! recursively proven. It keeps leaf and wrapper proof classes
//! separate and skips ancestor/descendant merge groups so an
//! `Instance.module` rewrite cannot introduce a hierarchy cycle.

use crate::ir::{Design, Module};
use crate::metrics::{
    canonical_module_signature, semantic_module_proof_with_modules, SemanticModuleProof,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Iteratively collapse `Module` definitions that share a canonical
/// structural signature. Returns the number of Modules removed across
/// all fixed-point iterations.
///
/// **Top module is never merged away.** The top is excluded from
/// grouping, so non-top modules may still merge with each other even
/// when the top happens to share their structural signature.
///
/// **Termination.** Each iteration that performs at least one merge
/// strictly decreases `design.modules.len()`. The minimum possible
/// length is 1 (the top alone). The loop therefore terminates after
/// at most `initial_len - 1` iterations.
pub fn dedup_modules(design: &mut Design) -> usize {
    let mut total_removed = 0usize;
    let reachable_before = reachable_module_names(design);
    loop {
        let removed_this_pass = dedup_modules_once(design);
        if removed_this_pass == 0 {
            break;
        }
        total_removed += removed_this_pass;
    }
    if total_removed > 0 {
        total_removed += prune_modules_made_unreachable(design, &reachable_before);
    }
    total_removed
}

/// Collapse pure-combinational, instance-free non-top `Module`
/// definitions whose bounded whole-module semantic proof is identical.
/// Returns the number of Modules removed across all fixed-point
/// iterations plus any modules made unreachable by a real merge.
///
/// This is a separate opt-in pass from [`dedup_modules`]. Structural
/// dedup remains structural-only; this function admits only the
/// `SemanticModuleProof` boundary from `metrics.rs`.
pub fn dedup_semantic_modules(design: &mut Design) -> usize {
    let mut total_removed = 0usize;
    let reachable_before = reachable_module_names(design);
    loop {
        let removed_this_pass = dedup_semantic_modules_once(design);
        if removed_this_pass == 0 {
            break;
        }
        total_removed += removed_this_pass;
    }
    if total_removed > 0 {
        total_removed += prune_modules_made_unreachable(design, &reachable_before);
    }
    total_removed
}

/// Collapse stateful flops-only **leaf** `Module` definitions proven
/// **sequentially (observationally) equivalent** by a bounded cross-module
/// bisimulation. Returns the number of Modules removed across all fixed-point
/// iterations plus any modules made unreachable by a real merge.
///
/// This is a separate opt-in pass from [`dedup_semantic_modules`] — the
/// sequential generalization of its pure-combinational (zero-flop) truth-table
/// proof (`IDENTITY-DEEPENING.3b`, decision `0008`). The equivalence verdict is
/// `crate::ir::compact::modules_sequentially_equivalent`; this function only
/// owns the candidate grouping (a cheap structural pre-filter + greedy-by-
/// representative grouping inside each bucket) and the survivor / instance-
/// rewrite / unreachable-prune tail it shares with the other dedup passes.
///
/// Sequential equivalence is a true equivalence relation, so grouping each
/// module against an existing group's representative is sound: if `X ≡ rep` and
/// `Y ≡ rep` then `X ≡ Y`, so rewriting both to the group's lex-smallest
/// survivor is sound even when the prover only checked each against `rep`.
pub fn dedup_sequential_modules(design: &mut Design) -> usize {
    let mut total_removed = 0usize;
    let reachable_before = reachable_module_names(design);
    loop {
        let removed_this_pass = dedup_sequential_modules_once(design);
        if removed_this_pass == 0 {
            break;
        }
        total_removed += removed_this_pass;
    }
    if total_removed > 0 {
        total_removed += prune_modules_made_unreachable(design, &reachable_before);
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

/// One semantic sweep over the current `design.modules`. Returns the
/// number of Modules removed during this sweep.
fn dedup_semantic_modules_once(design: &mut Design) -> usize {
    if design.modules.len() <= 1 {
        return 0;
    }

    let groups = {
        let modules_by_name: BTreeMap<_, _> = design
            .modules
            .iter()
            .map(|module| (module.name.as_str(), module))
            .collect();
        let top_name = design.top.clone();
        let mut groups: BTreeMap<SemanticModuleProof, Vec<usize>> = BTreeMap::new();
        for (idx, module) in design.modules.iter().enumerate() {
            if module.name == top_name {
                continue;
            }
            if let Some(proof) = semantic_module_proof_with_modules(module, &modules_by_name) {
                groups.entry(proof).or_default().push(idx);
            }
        }
        groups
    };

    let mut name_remap: HashMap<String, String> = HashMap::new();
    let mut indices_to_remove: Vec<usize> = Vec::new();
    for indices in groups.values() {
        if indices.len() < 2 {
            continue;
        }
        if semantic_group_has_ancestor_relation(design, indices) {
            continue;
        }
        let survivor_idx = *indices
            .iter()
            .min_by(|a, b| design.modules[**a].name.cmp(&design.modules[**b].name))
            .expect("non-empty semantic group");
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

    rewrite_instance_module_names(&mut design.modules, &name_remap);

    indices_to_remove.sort_unstable();
    indices_to_remove.dedup();
    let removed = indices_to_remove.len();
    for idx in indices_to_remove.into_iter().rev() {
        design.modules.remove(idx);
    }

    removed
}

/// Cheap structural pre-filter key for sequential-equivalence candidates: only
/// modules sharing this key can be sequentially equivalent, so the `O(n²)`
/// cross-module proof runs inside a bucket, not across the whole design. Keys by
/// `(sorted inputs, sorted outputs)` (`(PortId, width)`) and the sorted flop
/// multiset `(width, reset-discriminant, reset_val)` — the necessary conditions
/// the real proof later re-checks. Domain is omitted because eligible modules
/// are single-clock (`clock_domains.is_empty()`), so every flop is in domain 0.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SequentialPrefilterKey {
    inputs: Vec<(crate::ir::PortId, u32)>,
    outputs: Vec<(crate::ir::PortId, u32)>,
    flops: Vec<(u32, u8, u128)>,
}

fn sequential_prefilter_key(module: &Module) -> SequentialPrefilterKey {
    use crate::ir::ResetKind;
    let mut inputs: Vec<(crate::ir::PortId, u32)> =
        module.inputs.iter().map(|p| (p.id, p.width)).collect();
    let mut outputs: Vec<(crate::ir::PortId, u32)> =
        module.outputs.iter().map(|p| (p.id, p.width)).collect();
    inputs.sort_unstable();
    outputs.sort_unstable();
    let reset_disc = |kind: ResetKind| -> u8 {
        match kind {
            ResetKind::None => 0,
            ResetKind::Sync => 1,
            ResetKind::Async => 2,
        }
    };
    let mut flops: Vec<(u32, u8, u128)> = module
        .flops
        .iter()
        .map(|f| (f.width, reset_disc(f.reset_kind), f.reset_val))
        .collect();
    flops.sort_unstable();
    SequentialPrefilterKey {
        inputs,
        outputs,
        flops,
    }
}

/// Group the design's eligible stateful flops-only **leaf** modules into proven
/// sequential-equivalence classes (module indices into `design.modules`). The
/// top is excluded (never a merge candidate); ineligible modules are omitted
/// entirely. Each returned class is non-empty (singletons included).
///
/// Non-mutating, deterministic, and the single source of truth shared by the
/// [`dedup_sequential_modules`] pass and the `DesignMetrics` sequential
/// proof-signature metric, so the metric's "duplicate pairs" can never disagree
/// with what the pass would actually collapse (`IDENTITY-DEEPENING.3b.2b.2a`).
///
/// Grouping = a cheap structural pre-filter (interface + flop multiset) bucket
/// followed by greedy-by-representative grouping inside each bucket (sound
/// because sequential equivalence is a true equivalence relation: matching any
/// group representative implies equivalence to every member). Leaf modules never
/// instantiate anything, so no ancestor/descendant rewrite-cycle guard is needed
/// (unlike the wrapper-bearing combinational pass).
pub(crate) fn group_sequentially_equivalent_modules(design: &Design) -> Vec<Vec<usize>> {
    let top_name = design.top.as_str();
    let mut buckets: BTreeMap<SequentialPrefilterKey, Vec<usize>> = BTreeMap::new();
    for (idx, module) in design.modules.iter().enumerate() {
        if module.name == top_name {
            continue;
        }
        if !crate::ir::compact::sequential_leaf_eligible(module) {
            continue;
        }
        buckets
            .entry(sequential_prefilter_key(module))
            .or_default()
            .push(idx);
    }

    let mut classes: Vec<Vec<usize>> = Vec::new();
    for indices in buckets.values() {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for &idx in indices {
            let mut placed = false;
            for group in &mut groups {
                let rep = group[0];
                if crate::ir::compact::modules_sequentially_equivalent(
                    &design.modules[rep],
                    &design.modules[idx],
                ) {
                    group.push(idx);
                    placed = true;
                    break;
                }
            }
            if !placed {
                groups.push(vec![idx]);
            }
        }
        classes.extend(groups);
    }
    classes
}

/// One sequential sweep over the current `design.modules`. Returns the number
/// of Modules removed during this sweep.
fn dedup_sequential_modules_once(design: &mut Design) -> usize {
    if design.modules.len() <= 1 {
        return 0;
    }

    let classes = group_sequentially_equivalent_modules(design);
    let mut name_remap: HashMap<String, String> = HashMap::new();
    let mut indices_to_remove: Vec<usize> = Vec::new();
    for group in &classes {
        if group.len() < 2 {
            continue;
        }
        let survivor_idx = *group
            .iter()
            .min_by(|a, b| design.modules[**a].name.cmp(&design.modules[**b].name))
            .expect("non-empty sequential group");
        let survivor_name = design.modules[survivor_idx].name.clone();
        for &idx in group {
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

    rewrite_instance_module_names(&mut design.modules, &name_remap);

    indices_to_remove.sort_unstable();
    indices_to_remove.dedup();
    let removed = indices_to_remove.len();
    for idx in indices_to_remove.into_iter().rev() {
        design.modules.remove(idx);
    }

    removed
}

fn semantic_group_has_ancestor_relation(design: &Design, indices: &[usize]) -> bool {
    for (pos, &a_idx) in indices.iter().enumerate() {
        let a_name = design.modules[a_idx].name.as_str();
        for &b_idx in &indices[pos + 1..] {
            let b_name = design.modules[b_idx].name.as_str();
            if module_reaches_module(design, a_name, b_name)
                || module_reaches_module(design, b_name, a_name)
            {
                return true;
            }
        }
    }
    false
}

fn module_reaches_module(design: &Design, from: &str, target: &str) -> bool {
    if from == target {
        return false;
    }
    let modules_by_name: HashMap<&str, &Module> = design
        .modules
        .iter()
        .map(|module| (module.name.as_str(), module))
        .collect();
    let mut seen = BTreeSet::new();
    let mut stack = vec![from];
    while let Some(name) = stack.pop() {
        if !seen.insert(name) {
            continue;
        }
        let Some(module) = modules_by_name.get(name) else {
            continue;
        };
        for instance in &module.instances {
            if instance.module == target {
                return true;
            }
            stack.push(instance.module.as_str());
        }
    }
    false
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

fn prune_modules_made_unreachable(
    design: &mut Design,
    reachable_before: &BTreeSet<String>,
) -> usize {
    if design.modules.len() <= 1 {
        return 0;
    }

    let reachable_after = reachable_module_names(design);
    if reachable_after.is_empty() {
        return 0;
    }

    let before = design.modules.len();
    let top_name = design.top.clone();
    design.modules.retain(|module| {
        module.name == top_name
            || reachable_after.contains(&module.name)
            || !reachable_before.contains(&module.name)
    });
    before - design.modules.len()
}

fn reachable_module_names(design: &Design) -> BTreeSet<String> {
    let modules_by_name: HashMap<&str, &Module> = design
        .modules
        .iter()
        .map(|module| (module.name.as_str(), module))
        .collect();
    if !modules_by_name.contains_key(design.top.as_str()) {
        return BTreeSet::new();
    }

    let mut reachable: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![design.top.clone()];
    while let Some(name) = stack.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        let Some(module) = modules_by_name.get(name.as_str()) else {
            continue;
        };
        for instance in &module.instances {
            if !reachable.contains(&instance.module) {
                stack.push(instance.module.clone());
            }
        }
    }
    reachable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, GateOp, Instance, InstanceRole, Node, Port};

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

    fn double_not_leaf(name: &str) -> Module {
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "in", 1, Direction::In));
        m.outputs.push(make_port(1, "out", 1, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![0],
            width: 1,
            deps: DepSet::from_port(0),
        });
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![1],
            width: 1,
            deps: DepSet::from_port(0),
        });
        m.drives.push((1, 2));
        m
    }

    fn stateful_leaf(name: &str) -> Module {
        use crate::ir::{Flop, FlopKind, FlopMux, ResetKind};
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "clk", 1, Direction::In));
        m.inputs.push(make_port(1, "rst_n", 1, Direction::In));
        m.inputs.push(make_port(2, "in", 1, Direction::In));
        m.outputs.push(make_port(3, "out", 1, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 });
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 });
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((3, 1));
        m
    }

    fn shifted_port_leaf(name: &str) -> Module {
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(2, "in", 1, Direction::In));
        m.outputs.push(make_port(3, "out", 1, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 });
        m.drives.push((3, 0));
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
    fn dedup_keeps_semantic_equivalent_structurally_distinct_modules_separate() {
        let leaf_a = trivial_leaf("leaf_a");
        let leaf_b = double_not_leaf("leaf_b");
        assert_ne!(
            canonical_module_signature(&leaf_a),
            canonical_module_signature(&leaf_b),
            "module dedup is structural-only, even when two modules are semantically equivalent"
        );

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u_a".into(),
            module: "leaf_a".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u_b".into(),
            module: "leaf_b".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };

        let removed = dedup_modules(&mut design);
        assert_eq!(
            removed, 0,
            "structurally distinct modules must not merge without a module-level semantic proof"
        );
        assert_eq!(design.modules.len(), 3);
        let top_after = design
            .modules
            .iter()
            .find(|module| module.name == "top")
            .expect("top survived");
        let instance_modules: Vec<_> = top_after
            .instances
            .iter()
            .map(|instance| instance.module.as_str())
            .collect();
        assert_eq!(instance_modules, vec!["leaf_a", "leaf_b"]);
    }

    #[test]
    fn semantic_dedup_collapses_bounded_equivalent_pure_comb_leaves() {
        let leaf_a = trivial_leaf("leaf_a");
        let leaf_b = double_not_leaf("leaf_b");

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u_a".into(),
            module: "leaf_a".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u_b".into(),
            module: "leaf_b".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };

        let removed = dedup_semantic_modules(&mut design);
        assert_eq!(
            removed, 1,
            "bounded semantic module dedup should merge out = in and out = ~~in"
        );
        let names: Vec<_> = design
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        assert_eq!(names, vec!["leaf_a", "top"]);
        let top_after = design
            .modules
            .iter()
            .find(|module| module.name == "top")
            .expect("top survived");
        assert!(top_after.instances.iter().all(|i| i.module == "leaf_a"));
    }

    #[test]
    fn semantic_dedup_keeps_stateful_modules_outside_proof_boundary() {
        let mut design = Design {
            top: "top".into(),
            modules: vec![
                stateful_leaf("state_a"),
                stateful_leaf("state_b"),
                Module {
                    name: "top".into(),
                    ..Module::default()
                },
            ],
        };

        assert_eq!(
            dedup_semantic_modules(&mut design),
            0,
            "stateful modules need sequential proof inputs, so semantic module dedup must skip them"
        );
        assert_eq!(design.modules.len(), 3);
    }

    /// A two-cycle delay-line stateful leaf (`out` = `in` delayed two cycles).
    /// With `double_not`, stage 0's D-cone is `~~in` (semantically `in`) so the
    /// module is structurally distinct but sequentially equivalent — the
    /// cross-module merge target (`IDENTITY-DEEPENING.3b.2b.1`).
    fn delay2_leaf(name: &str, double_not: bool) -> Module {
        use crate::ir::{Flop, FlopKind, FlopMux, NodeId, ResetKind};
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "clk", 1, Direction::In));
        m.inputs.push(make_port(1, "rst_n", 1, Direction::In));
        m.inputs.push(make_port(2, "in", 1, Direction::In));
        m.outputs.push(make_port(3, "out", 1, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 0 = in
        let d0: NodeId = if double_not {
            m.nodes.push(Node::Gate {
                op: GateOp::Not,
                operands: vec![0],
                width: 1,
                deps: DepSet::from_port(2),
            });
            m.nodes.push(Node::Gate {
                op: GateOp::Not,
                operands: vec![1],
                width: 1,
                deps: DepSet::from_port(2),
            });
            2
        } else {
            0
        };
        let q0 = m.nodes.len() as NodeId;
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 });
        let q1 = m.nodes.len() as NodeId;
        m.nodes.push(Node::FlopQ { flop: 1, width: 1 });
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(d0),
            q: q0,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 1,
            d: Some(q0),
            q: q1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((3, q1));
        m
    }

    fn stateful_delay_top(a_module: &str, b_module: &str) -> Module {
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "clk", 1, Direction::In));
        top.inputs.push(make_port(1, "rst_n", 1, Direction::In));
        top.inputs.push(make_port(2, "in", 1, Direction::In));
        top.outputs.push(make_port(4, "oa", 1, Direction::Out));
        top.outputs.push(make_port(5, "ob", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 clk
        top.nodes.push(Node::PrimaryInput { port: 1, width: 1 }); // 1 rst_n
        top.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 2 in
        top.instances.push(Instance {
            id: 0,
            name: "u_a".into(),
            module: a_module.into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0), (1, 1), (2, 2)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u_b".into(),
            module: b_module.into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0), (1, 1), (2, 2)],
            param_bindings: Vec::new(),
        });
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 3,
            width: 1,
        }); // 3
        top.nodes.push(Node::InstanceOutput {
            instance: 1,
            port: 3,
            width: 1,
        }); // 4
        top.drives.push((4, 3));
        top.drives.push((5, 4));
        top
    }

    #[test]
    fn sequential_dedup_collapses_bounded_equivalent_stateful_leaves() {
        let leaf_a = delay2_leaf("leaf_a", false);
        let leaf_b = delay2_leaf("leaf_b", true);
        // Structurally distinct (B carries two extra `Not` gates), so the
        // structural and combinational module-dedup passes must leave them apart.
        assert_ne!(
            canonical_module_signature(&leaf_a),
            canonical_module_signature(&leaf_b),
            "the delay lines are structurally distinct"
        );

        let top = stateful_delay_top("leaf_a", "leaf_b");
        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };

        assert_eq!(
            dedup_modules(&mut design),
            0,
            "structural dedup must not merge structurally-distinct leaves"
        );
        assert_eq!(
            dedup_semantic_modules(&mut design),
            0,
            "combinational module dedup skips stateful modules"
        );
        assert_eq!(
            dedup_sequential_modules(&mut design),
            1,
            "the sequentially-equivalent stateful delay lines collapse to one"
        );
        let names: Vec<_> = design
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        assert_eq!(names, vec!["leaf_a", "top"]);
        let top_after = design
            .modules
            .iter()
            .find(|module| module.name == "top")
            .expect("top survived");
        assert!(top_after.instances.iter().all(|i| i.module == "leaf_a"));
    }

    #[test]
    fn sequential_dedup_keeps_non_equivalent_stateful_leaves_separate() {
        // Two delay lines that share interface + flop shape but observe a
        // different stage (two-cycle vs one-cycle delay): genuinely not
        // sequentially equivalent, so the pass must not merge them.
        let leaf_a = delay2_leaf("leaf_a", false);
        let mut leaf_b = delay2_leaf("leaf_b", false);
        // Rewire B's output from Q_1 (node 2) to Q_0 (node 1): one-cycle delay.
        leaf_b.drives.clear();
        leaf_b.drives.push((3, 1));

        let top = stateful_delay_top("leaf_a", "leaf_b");
        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };

        assert_eq!(
            dedup_sequential_modules(&mut design),
            0,
            "one-cycle and two-cycle delays are not sequentially equivalent"
        );
        assert_eq!(design.modules.len(), 3);
    }

    #[test]
    fn sequential_proof_metric_counts_then_collapses_pair() {
        use crate::metrics::compute_design;
        let leaf_a = delay2_leaf("leaf_a", false);
        let leaf_b = delay2_leaf("leaf_b", true);
        let top = stateful_delay_top("leaf_a", "leaf_b");
        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };

        let before = compute_design(&design);
        assert_eq!(
            before.num_sequentially_duplicate_module_pairs, 1,
            "the two equivalent stateful leaves form one sequential duplicate pair"
        );
        let sigs = &before.sequential_module_proof_signatures;
        assert!(
            sigs[0].is_some() && sigs[0] == sigs[1],
            "sequentially-equivalent leaves share one proof-class signature"
        );
        assert!(
            sigs[2].is_none(),
            "the top is outside the sequential proof boundary"
        );

        assert_eq!(dedup_sequential_modules(&mut design), 1);
        let after = compute_design(&design);
        assert_eq!(
            after.num_sequentially_duplicate_module_pairs, 0,
            "the merge reduces the duplicate-pair count to zero"
        );
    }

    /// Downstream-clean bank hook (`IDENTITY-DEEPENING.3b.2b.2a`): the merged
    /// multi-module stateful design emits valid, downstream-clean SV. Default
    /// no-op; re-bank with
    ///   ANVIL_DUMP_SEQ_MODULE_SV=1 cargo test --lib \
    ///     sequential_dedup_merged_design_is_downstream_clean
    /// then lint /tmp/anvil-seq-module-merged.sv with verilator --lint-only -Wall
    /// + yosys (both modes) + iverilog -g2012.
    #[test]
    fn sequential_dedup_merged_design_is_downstream_clean() {
        let leaf_a = delay2_leaf("leaf_a", false);
        let leaf_b = delay2_leaf("leaf_b", true);
        let top = stateful_delay_top("leaf_a", "leaf_b");
        let mut design = Design {
            top: "top".into(),
            modules: vec![leaf_a, leaf_b, top],
        };
        assert_eq!(dedup_sequential_modules(&mut design), 1);
        crate::ir::validate::validate_design(&design)
            .expect("merged sequential-equivalence design should validate");
        if std::env::var("ANVIL_DUMP_SEQ_MODULE_SV").is_ok() {
            std::fs::write(
                "/tmp/anvil-seq-module-merged.sv",
                crate::emit::to_sv_design(&design),
            )
            .unwrap();
        }
    }

    #[test]
    fn semantic_dedup_collapses_bounded_equivalent_comb_wrappers() {
        let child_a = trivial_leaf("child_a");
        let child_b = double_not_leaf("child_b");
        let parent_a = parent_instantiating_child("parent_a", "child_a");
        let parent_b = parent_instantiating_child("parent_b", "child_b");
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u_a".into(),
            module: "parent_a".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u_b".into(),
            module: "parent_b".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![child_a, child_b, parent_a, parent_b, top],
        };

        assert_eq!(
            dedup_semantic_modules(&mut design),
            2,
            "semantically equivalent pure-combinational wrappers and their equivalent children should merge"
        );
        let names: Vec<_> = design
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        assert_eq!(names, vec!["child_a", "parent_a", "top"]);
        let top_after = design
            .modules
            .iter()
            .find(|module| module.name == "top")
            .expect("top survived");
        assert!(top_after.instances.iter().all(|i| i.module == "parent_a"));
    }

    #[test]
    fn semantic_dedup_keeps_leaf_and_wrapper_in_separate_proof_classes() {
        let child = trivial_leaf("child");
        let parent = parent_instantiating_child("parent", "child");
        let mut design = Design {
            top: "top".into(),
            modules: vec![
                child,
                parent,
                Module {
                    name: "top".into(),
                    ..Module::default()
                },
            ],
        };

        assert_eq!(
            dedup_semantic_modules(&mut design),
            0,
            "leaf and wrapper proofs stay in separate classes to avoid flattening-by-dedup surprises"
        );
        assert_eq!(design.modules.len(), 3);
    }

    #[test]
    fn semantic_dedup_skips_ancestor_descendant_wrapper_groups() {
        let child = trivial_leaf("leaf");
        let mid = parent_instantiating_child("mid", "leaf");
        let parent = parent_instantiating_child("parent", "mid");
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u_parent".into(),
            module: "parent".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![child, mid, parent, top],
        };

        assert_eq!(
            dedup_semantic_modules(&mut design),
            0,
            "equivalent wrappers that instantiate each other must not merge because a rewrite could create a hierarchy cycle"
        );
        assert!(design.modules.iter().any(|module| module.name == "mid"));
        assert!(design.modules.iter().any(|module| module.name == "parent"));
    }

    #[test]
    fn semantic_dedup_requires_matching_port_ids() {
        let mut design = Design {
            top: "top".into(),
            modules: vec![
                trivial_leaf("leaf_a"),
                shifted_port_leaf("leaf_b"),
                Module {
                    name: "top".into(),
                    ..Module::default()
                },
            ],
        };

        assert_eq!(
            dedup_semantic_modules(&mut design),
            0,
            "rewriting an instance keeps port-id bindings, so different public port IDs must not merge"
        );
        assert_eq!(design.modules.len(), 3);
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

    fn parent_instantiating_child(name: &str, child_module: &str) -> Module {
        let mut m = Module {
            name: name.into(),
            ..Module::default()
        };
        m.inputs.push(make_port(0, "i", 1, Direction::In));
        m.outputs.push(make_port(1, "o", 1, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.instances.push(Instance {
            id: 0,
            name: "u_child".into(),
            module: child_module.into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        m.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1,
            width: 1,
        });
        m.drives.push((1, 1));
        m
    }

    #[test]
    fn dedup_prunes_modules_made_unreachable_by_a_merge() {
        let child_a = trivial_leaf("child_a");
        let mut child_b = trivial_leaf("child_b");
        child_b.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Not,
            operands: vec![0],
            width: 1,
            deps: crate::ir::DepSet::from_port(0),
        });
        child_b.drives[0] = (1, 1);
        let parent_a = parent_instantiating_child("parent_a", "child_a");
        let parent_b = parent_instantiating_child("parent_b", "child_b");

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "parent_a".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.instances.push(Instance {
            id: 1,
            name: "u1".into(),
            module: "parent_b".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![child_a, child_b, parent_a, parent_b, top],
        };

        let removed = dedup_modules(&mut design);
        assert_eq!(
            removed, 2,
            "one duplicate parent and its now-unreachable child should be removed"
        );
        let names: Vec<_> = design.modules.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["child_a", "parent_a", "top"]);
        let top_after = design
            .modules
            .iter()
            .find(|module| module.name == "top")
            .expect("top survived");
        assert!(top_after.instances.iter().all(|i| i.module == "parent_a"));
    }

    #[test]
    fn dedup_preserves_unreachable_modules_when_no_merge_occurs() {
        let used = trivial_leaf("used");
        let mut unused_distinct = trivial_leaf("unused_distinct");
        unused_distinct.inputs[0].width = 2;
        unused_distinct.outputs[0].width = 2;
        unused_distinct.nodes[0] = Node::PrimaryInput { port: 0, width: 2 };

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(make_port(0, "i", 1, Direction::In));
        top.outputs.push(make_port(1, "o", 1, Direction::Out));
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "used".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: Vec::new(),
        });
        top.drives.push((1, 0));

        let mut design = Design {
            top: "top".into(),
            modules: vec![used, unused_distinct, top],
        };

        let removed = dedup_modules(&mut design);
        assert_eq!(removed, 0);
        let names: Vec<_> = design.modules.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["used", "unused_distinct", "top"],
            "no-merge calls preserve existing under-instantiated library definitions"
        );
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
