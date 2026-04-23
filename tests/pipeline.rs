//! End-to-end: generate many modules across seeds and assert each
//! passes IR validation and produces non-empty SV output.

use anvil::config::ConstructionStrategy;
use anvil::ir::{GateOp, Node};
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
