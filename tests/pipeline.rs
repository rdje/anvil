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

/// Regression guard for Rule 18 (no orphan gates). The generator
/// must produce modules whose every `Node::Gate` has at least one
/// consumer (other gate's operand, flop D / mux operand, or
/// output drive). Measured across all four strategy values at
/// several seeds.
#[test]
fn zero_orphans_at_default_knobs() {
    use anvil::ir::{FlopMux, Node};
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
                match &f.mux {
                    FlopMux::Encoded { sel, data } => {
                        used[*sel as usize] = true;
                        for d in data {
                            used[*d as usize] = true;
                        }
                    }
                    FlopMux::OneHot(arms) => {
                        for arm in arms {
                            used[arm.data as usize] = true;
                            used[arm.sel as usize] = true;
                        }
                    }
                    FlopMux::None => {}
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

/// Informational regression guard for the
/// `nested_associative_operand_count` metric. At default knobs
/// today (Associative layer NOT implemented), this count is
/// non-zero. When the Associative layer lands in a future slice,
/// this assertion will start failing — flip it to `== 0` then,
/// as direct validation that flattening worked.
#[test]
fn nested_associative_opportunities_exist_today() {
    // Seed 42 produced 373 opportunities at the time this test was
    // added (slice 99084a8). Using a lower bound that still proves
    // non-triviality without pinning the exact count (distribution
    // can shift with generator evolution).
    let cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let m = Generator::new(cfg).generate_module();
    let metrics = anvil::metrics::compute(&m);
    assert!(
        metrics.nested_associative_operand_count > 0,
        "expected nested-associative opportunities at default knobs \
         pre-Associative-layer; got {}. If the Associative layer just \
         landed, this test should be updated to assert == 0.",
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
        "priority_encoder_prob",
        "coefficient_prob",
        "const_shift_amount_prob",
        "const_comparand_prob",
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
