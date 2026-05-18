//! INSTA-SNAPSHOTS.1 — byte-identical reproducibility guard-rails.
//!
//! ANVIL's core contract is that `(seed, config)` produces
//! byte-identical SystemVerilog forever, on any platform
//! (`README.md`, `book/src/knobs.md`). These `insta` snapshots are the
//! cheapest *direct* enforcement of that: any accidental output drift
//! — HashMap iteration order, RNG re-seeding, planner reorder, emit
//! ordering — breaks a snapshot and requires a deliberate
//! `cargo insta accept` (a paired diff review), never a silent pass.
//!
//! `.1` landed the baseline for two canonical modes (one leaf, one
//! bounded recursive library design). `.2` expands to ≥5 distinct
//! `(seed, config)` shapes spanning the reachable generation axes:
//! library *and* on-demand child sourcing, helper-instance/sibling
//! routes, registered/parent-composed routes, and a design that
//! exercises `canonical_module_signatures` under
//! `hierarchy_module_dedup` (so dedup follow-up work in
//! `HIERARCHY-AWARE-IDENTITY` cannot silently drift the emitted
//! text). `.3` wires `cargo insta test` into the pre-commit
//! checklist + documents the acceptance protocol. Snapshots only
//! catch *drift*, not correctness bugs (that is the matrix gate /
//! focused proofs). Every config below is fully deterministic
//! (fixed seed, exact `min == max` bounds where applicable, fixed
//! `construction_strategy`) so the emitted text is byte-stable.

use anvil::config::{ConstructionStrategy, HierarchyChildSourceMode};
use anvil::{Config, Generator};

/// Fixed, fully-deterministic minimal combinational leaf.
fn canonical_leaf_config() -> Config {
    Config {
        seed: 1,
        max_depth: 2,
        min_inputs: 2,
        max_inputs: 2,
        max_outputs: 1,
        flop_prob: 0.0,
        share_prob: 0.0,
        ..Config::default()
    }
}

/// Fixed, fully-deterministic bounded recursive design in library
/// child-sourcing mode. Exact depth + exact branching (min == max)
/// so the shape — hence the emitted text — is stable across runs and
/// platforms (proven config shape, see
/// `tests/pipeline.rs::generates_valid_recursive_hierarchy_designs_with_bounded_shape`).
fn bounded_recursive_library_config() -> Config {
    Config {
        seed: 11,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 2,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 2,
        ..Config::default()
    }
}

/// `.2` — bounded recursive design in **on-demand** child-sourcing
/// mode (the other child-sourcing axis vs the library snapshot).
/// Proven shape:
/// `tests/pipeline.rs::generates_valid_recursive_hierarchy_designs_with_ondemand_child_sourcing`.
fn bounded_recursive_ondemand_config() -> Config {
    Config {
        seed: 11,
        min_hierarchy_depth: 2,
        max_hierarchy_depth: 2,
        min_child_instances_per_module: 2,
        max_child_instances_per_module: 2,
        hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
        ..Config::default()
    }
}

/// `.2` — depth-1 wrapper with **sibling-routed** child inputs
/// (`hierarchy_sibling_route_prob = 1.0`): exercises the
/// helper-instance / sibling-route path. Fixed strategy for
/// determinism. Proven shape: the sibling-routing pipeline test.
fn sibling_route_config() -> Config {
    Config {
        seed: 42,
        hierarchy_depth: 1,
        num_leaf_modules: 2,
        num_child_instances: 4,
        hierarchy_sibling_route_prob: 1.0,
        hierarchy_child_input_cone_prob: 0.0,
        construction_strategy: ConstructionStrategy::Sequential,
        ..Config::default()
    }
}

/// `.2` — depth-1 wrapper with **parent-composed** child inputs via
/// a parent-cone helper instance (`hierarchy_child_input_cone_prob =
/// 1.0` + `hierarchy_parent_cone_instance_prob = 1.0`). Proven shape:
/// the parent-cone-instance pipeline test.
fn parent_composed_route_config() -> Config {
    Config {
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
        construction_strategy: ConstructionStrategy::Sequential,
        ..Config::default()
    }
}

/// `.2` — the dedup proof base (4 structurally-duplicate library
/// leaves, 1-bit) **with `hierarchy_module_dedup = true`**: exercises
/// `canonical_module_signatures` + the post-finalisation
/// instance-rewrite, so any dedup change in `HIERARCHY-AWARE-IDENTITY`
/// that perturbs emitted text breaks this snapshot. Fixed strategy.
/// Proven shape: the `hierarchy_module_dedup` pipeline proof's base.
fn dedup_canonical_signatures_config() -> Config {
    Config {
        seed: 42,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        min_inputs: 1,
        max_inputs: 1,
        min_outputs: 1,
        max_outputs: 1,
        min_width: 1,
        max_width: 1,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        terminal_reuse_prob: 1.0,
        constant_prob: 0.0,
        max_depth: 1,
        hierarchy_module_dedup: true,
        construction_strategy: ConstructionStrategy::Sequential,
        ..Config::default()
    }
}

fn emit(cfg: Config) -> String {
    cfg.validate().expect("snapshot config must be valid");
    let design = Generator::new(cfg).generate_design();
    anvil::ir::validate::validate_design(&design).expect("snapshot design must validate");
    anvil::emit::to_sv_design(&design)
}

#[test]
fn snapshot_canonical_leaf() {
    insta::assert_snapshot!("canonical_leaf", emit(canonical_leaf_config()));
}

#[test]
fn snapshot_bounded_recursive_library() {
    insta::assert_snapshot!(
        "bounded_recursive_library",
        emit(bounded_recursive_library_config())
    );
}

#[test]
fn snapshot_bounded_recursive_ondemand() {
    insta::assert_snapshot!(
        "bounded_recursive_ondemand",
        emit(bounded_recursive_ondemand_config())
    );
}

#[test]
fn snapshot_sibling_route() {
    insta::assert_snapshot!("sibling_route", emit(sibling_route_config()));
}

#[test]
fn snapshot_parent_composed_route() {
    insta::assert_snapshot!(
        "parent_composed_route",
        emit(parent_composed_route_config())
    );
}

#[test]
fn snapshot_dedup_canonical_signatures() {
    insta::assert_snapshot!(
        "dedup_canonical_signatures",
        emit(dedup_canonical_signatures_config())
    );
}
