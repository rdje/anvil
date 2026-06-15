//! `SV-VERSION-TARGETING.2b.1` — the down-gating byte-identity proof.
//!
//! The opt-in `--sv-version <2012|2017|2023>` gate is a construction-time
//! capability bound (decision `0009`). Its first increment threads the target
//! into the emitter *without* any version-distinctive construct yet: ANVIL's
//! entire current emitted subset (`logic` / `always_ff` / `always_comb` /
//! `case` / `casez` / `for` / packed `struct` / packed arrays / `typedef` /
//! `localparam`) is valid in IEEE 1800-2012, so emitting the **same IR** at any
//! of the three targets must produce byte-identical SystemVerilog.
//!
//! That is the concrete, testable statement of the **down-gating guarantee over
//! the current subset**: the bound removes nothing today because nothing newer
//! than 2012 is emitted (the first up-opted construct, where 2017/2023 would
//! diverge, lands at `SV-VERSION-TARGETING.3`). It also guards that the default
//! emission path (`Sv2012`) is byte-identical to the historical no-version
//! entry points (`to_sv` / `to_sv_design`), i.e. `tests/snapshots.rs` stays
//! valid.

use anvil::config::{ConstructionStrategy, SvVersion};
use anvil::{Config, Generator};

const VERSIONS: [SvVersion; 3] = [SvVersion::Sv2012, SvVersion::Sv2017, SvVersion::Sv2023];

/// Leaf-module configs that, between them, exercise the constructs the
/// single-module lane can emit: combinational operators, sequential
/// `always_ff`, the structured `case`/`casez`/`for`/priority surfaces,
/// the arithmetic/comparison/shift/reduction operators, the inferrable-memory
/// template, and the generated-encoding FSM template.
fn leaf_configs() -> Vec<Config> {
    vec![
        // Plain combinational.
        Config {
            seed: 1,
            ..Config::default()
        },
        // Sequential-heavy (`always_ff`, flop-D cones, feedback).
        Config {
            seed: 2,
            flop_prob: 0.6,
            max_flops_per_module: 16,
            ..Config::default()
        },
        // Structured surfaces: case / casez / for-fold / priority encoder.
        Config {
            seed: 3,
            case_mux_prob: 0.5,
            casez_mux_prob: 0.5,
            for_fold_prob: 0.5,
            priority_encoder_prob: 0.3,
            ..Config::default()
        },
        // Operator spread: arithmetic / comparison / shift / reduction,
        // plus the coefficient / const-comparand / const-shift motifs.
        Config {
            seed: 4,
            gate_arith_weight: 4,
            gate_compare_weight: 3,
            gate_reduce_weight: 3,
            gate_shift_weight: 3,
            coefficient_prob: 0.5,
            const_comparand_prob: 0.6,
            const_shift_amount_prob: 0.5,
            ..Config::default()
        },
        // Inferrable-memory leaf (packed memory array + synchronous template).
        Config {
            seed: 5,
            memory_prob: 1.0,
            ..Config::default()
        },
        // Generated-encoding FSM leaf (localparam state constants + Moore decode).
        Config {
            seed: 6,
            fsm_prob: 1.0,
            ..Config::default()
        },
    ]
}

/// Design configs that exercise the multi-module hierarchy emitter surface
/// (instances, instance outputs, multi-file), plus a packed-aggregate-enabled
/// design (the `typedef struct packed` / packed-array emitter projection).
fn design_configs() -> Vec<Config> {
    vec![
        // Bounded recursive hierarchy (library child sourcing).
        Config {
            seed: 11,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 2,
            ..Config::default()
        },
        // Legacy depth-1 wrapper with real child instances.
        Config {
            seed: 12,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            construction_strategy: ConstructionStrategy::Sequential,
            ..Config::default()
        },
        // Hierarchy with packed-aggregate projection forced on (covers the
        // aggregate emitter surface to whatever extent it fires for this seed;
        // either way the identity invariant must hold).
        Config {
            seed: 13,
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            aggregate_prob: 1.0,
            aggregate_array_prob: 1.0,
            construction_strategy: ConstructionStrategy::Sequential,
            ..Config::default()
        },
    ]
}

#[test]
fn leaf_emission_is_byte_identical_across_sv_versions() {
    for cfg in leaf_configs() {
        cfg.validate().expect("snapshot config must be valid");
        let m = Generator::new(cfg.clone()).generate_module();
        let baseline = anvil::emit::to_sv_versioned(&m, SvVersion::Sv2012);

        // The historical no-version entry point must equal the floor target,
        // so `tests/snapshots.rs` (which calls `to_sv*`) stays byte-identical.
        assert_eq!(
            anvil::emit::to_sv(&m),
            baseline,
            "seed {}: to_sv() diverged from the Sv2012 floor",
            cfg.seed
        );

        for v in VERSIONS {
            assert_eq!(
                anvil::emit::to_sv_versioned(&m, v),
                baseline,
                "seed {}: emission targeting {:?} diverged from the 2012 floor — \
                 the current subset must be a 2012/2017/2023 common floor",
                cfg.seed,
                v
            );
        }
    }
}

#[test]
fn design_emission_is_byte_identical_across_sv_versions() {
    for cfg in design_configs() {
        cfg.validate().expect("snapshot config must be valid");
        let design = Generator::new(cfg.clone()).generate_design();
        anvil::ir::validate::validate_design(&design).expect("design must validate");
        let baseline = anvil::emit::to_sv_design_versioned(&design, SvVersion::Sv2012);

        assert_eq!(
            anvil::emit::to_sv_design(&design),
            baseline,
            "seed {}: to_sv_design() diverged from the Sv2012 floor",
            cfg.seed
        );

        for v in VERSIONS {
            assert_eq!(
                anvil::emit::to_sv_design_versioned(&design, v),
                baseline,
                "seed {}: design emission targeting {:?} diverged from the 2012 floor",
                cfg.seed,
                v
            );
        }
    }
}
