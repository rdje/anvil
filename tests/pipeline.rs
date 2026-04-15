//! End-to-end: generate many modules across seeds and assert each
//! passes IR validation and produces non-empty SV output.

use anvil::config::ConstructionStrategy;
use anvil::{Config, Generator};

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
fn graph_first_is_default() {
    // The default strategy is now GraphFirst. Omitting the flag and
    // explicitly passing graph-first must produce byte-identical output.
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
        "default strategy must be GraphFirst (byte-identical output)"
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
fn graph_first_differs_from_sequential() {
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
        "graph-first must produce different output from sequential"
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
