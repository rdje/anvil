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
//! `.1` lands the baseline for two canonical modes (one leaf, one
//! bounded recursive library design). `.2` expands the axes; `.3`
//! wires `cargo insta test` into the pre-commit checklist + documents
//! the acceptance protocol. Snapshots only catch *drift*, not
//! correctness bugs (that is the matrix gate / focused proofs).

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
