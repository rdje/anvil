//! Generator entry points. See `book/src/algorithm.md`.

pub mod cone;
pub mod hierarchy;
pub mod module;
/// `MULTI-CLOCK-CDC.3a` / `SIGNOFF-SURFACE-EXPANSION.1` —
/// synchronizer construction primitive for cross-clock-domain
/// signals. Per `MULTI-CLOCK-CDC.1`'s design + the rules-first
/// generation doctrine (`feedback_rules_first_generation.md`), the
/// synchronizer is constructed in place when the generator makes a
/// domain-crossing decision; never via a post-pass filter. The
/// default chain is 2 stages; `Config::cdc_synchronizer_stages >= 3`
/// opts into the N-flop variant.
pub mod multi_clock;
pub mod pool;

use crate::config::{Config, FactorizationLevel, IdentityMode};
use crate::ir::{Design, KnobId, Module, ModuleInterfaceProfile};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratorCheckpoint {
    pub next_module_index: u64,
    pub stream: u64,
    pub word_pos_lo: u64,
    pub word_pos_hi: u64,
}

impl GeneratorCheckpoint {
    #[inline]
    fn from_word_pos(word_pos: u128, stream: u64, next_module_index: u64) -> Self {
        Self {
            next_module_index,
            stream,
            word_pos_lo: word_pos as u64,
            word_pos_hi: (word_pos >> 64) as u64,
        }
    }

    #[inline]
    fn word_pos(&self) -> u128 {
        (u128::from(self.word_pos_hi) << 64) | u128::from(self.word_pos_lo)
    }
}

pub struct Generator {
    pub(crate) rng: ChaCha8Rng,
    pub(crate) cfg: Config,
    next_module_index: u64,
    pub(crate) active_flop_knob: KnobId,
}

impl Generator {
    pub fn new(cfg: Config) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(cfg.seed);
        Self {
            rng,
            cfg,
            next_module_index: 0,
            active_flop_knob: KnobId::FlopProb,
        }
    }

    pub fn generate_module(&mut self) -> Module {
        let idx = self.reserve_module_index();
        let mut m = module::generate_leaf_module(self, idx);
        // `MULTI-CLOCK-CDC.3b.2` — the single-module path
        // (`tool_matrix`'s per-scenario flow) also needs the
        // multi-clock promotion pass. `generate_design` applies
        // it after dedup + the parameterization / aggregate
        // passes; here we apply it inline since the single-module
        // flow has no design-level passes to interleave with.
        // Default `multi_clock_prob = 0.0` ⇒ skip ⇒ byte-identical.
        if self.cfg.multi_clock_prob > 0.0 {
            let p = self.cfg.multi_clock_prob.clamp(0.0, 1.0);
            if self.rng.gen_bool(p) {
                let _ = multi_clock::promote_to_multi_clock_with_stages(
                    &mut m,
                    self.cfg.cdc_synchronizer_stages,
                );
            }
        }
        // `SV-VERSION-TARGETING.3b.2` — opt-in: mark proper low-bits `Slice`
        // gates for the IEEE-1800-2023 `union soft` overlay (the emitter only
        // realizes it when the target also permits 2023). Default
        // `soft_union_slice_prob = 0.0` ⇒ no roll ⇒ byte-identical stream +
        // output. Mirrors the `aggregate_prob` call-site roll.
        if self.cfg.soft_union_slice_prob > 0.0 {
            let p = self.cfg.soft_union_slice_prob;
            crate::ir::soft_union::annotate_soft_union_slices(&mut m, &mut self.rng, p);
        }
        // `STRUCTURED-EMISSION-EXPANSION.2b.1` — opt-in combinational
        // `function automatic` emit-projection marker (decision `0012`).
        // Runs AFTER soft_union so the `union soft` marks are visible and
        // excluded (the two emit-projections are mutually exclusive on a
        // gate). Default `function_emit_prob = 0.0` ⇒ no roll ⇒
        // byte-identical stream + output. Mirrors the soft_union call-site
        // roll.
        if self.cfg.function_emit_prob > 0.0 {
            let p = self.cfg.function_emit_prob;
            crate::ir::function_emit::annotate_function_emit_gates(&mut m, &mut self.rng, p);
        }
        // `STRUCTURED-EMISSION-EXPANSION.4b` — opt-in `generate for` loop
        // emit-projection marker for `{N{x}}` 1-bit-lane replications
        // (decision `0013`). Runs AFTER function_emit so an already
        // function-emit-marked replication is excluded (the two
        // emit-projections are mutually exclusive on a gate). Default
        // `generate_loop_emit_prob = 0.0` ⇒ no roll ⇒ byte-identical stream
        // + output. Mirrors the function_emit call-site roll.
        if self.cfg.generate_loop_emit_prob > 0.0 {
            let p = self.cfg.generate_loop_emit_prob;
            crate::ir::generate_loop::annotate_generate_loop_gates(&mut m, &mut self.rng, p);
        }
        // `STRUCTURED-EMISSION-EXPANSION.6b.1` — opt-in combinational
        // `task automatic` emit-projection marker (decision `0014`). Runs
        // AFTER function_emit and generate_loop so an already-marked gate is
        // excluded (the emit-projections are mutually exclusive on a gate).
        // Default `task_emit_prob = 0.0` ⇒ no roll ⇒ byte-identical stream +
        // output. Mirrors the generate_loop call-site roll.
        if self.cfg.task_emit_prob > 0.0 {
            let p = self.cfg.task_emit_prob;
            crate::ir::task_emit::annotate_task_emit_gates(&mut m, &mut self.rng, p);
        }
        m
    }

    pub fn generate_module_with_interface_profile(
        &mut self,
        interface_profile: Option<&ModuleInterfaceProfile>,
    ) -> Module {
        let idx = self.reserve_module_index();
        let mut m =
            module::generate_leaf_module_with_interface_profile(self, idx, interface_profile);
        // Same single-module-path promotion as `generate_module`
        // — keep parity per `MULTI-CLOCK-CDC.3b.2`.
        if self.cfg.multi_clock_prob > 0.0 {
            let p = self.cfg.multi_clock_prob.clamp(0.0, 1.0);
            if self.rng.gen_bool(p) {
                let _ = multi_clock::promote_to_multi_clock_with_stages(
                    &mut m,
                    self.cfg.cdc_synchronizer_stages,
                );
            }
        }
        m
    }

    pub(crate) fn reserve_module_index(&mut self) -> u64 {
        let idx = self.next_module_index;
        self.next_module_index += 1;
        idx
    }

    pub(crate) fn module_name(&self, index: u64) -> String {
        format!("mod_{}_{:04}", self.cfg.seed, index)
    }

    pub fn checkpoint(&self) -> GeneratorCheckpoint {
        GeneratorCheckpoint::from_word_pos(
            self.rng.get_word_pos(),
            self.rng.get_stream(),
            self.next_module_index,
        )
    }

    pub fn restore_checkpoint(&mut self, checkpoint: &GeneratorCheckpoint) {
        self.rng = ChaCha8Rng::seed_from_u64(self.cfg.seed);
        self.rng.set_stream(checkpoint.stream);
        self.rng.set_word_pos(checkpoint.word_pos());
        self.next_module_index = checkpoint.next_module_index;
    }

    pub fn generate_design(&mut self) -> Design {
        let mut design = if self.cfg.effective_hierarchy_depth_range().is_none() {
            let m = self.generate_module();
            let name = m.name.clone();
            Design {
                top: name,
                modules: vec![m],
            }
        } else {
            hierarchy::generate_design(self)
        };
        if self.cfg.hierarchy_module_dedup {
            crate::ir::dedup::dedup_modules(&mut design);
        }
        if self.cfg.hierarchy_semantic_module_dedup
            && self.cfg.identity_mode == IdentityMode::NodeId
            && self.cfg.effective_factorization_level() >= FactorizationLevel::EGraph
        {
            crate::ir::dedup::dedup_semantic_modules(&mut design);
        }
        // `IDENTITY-DEEPENING.3b.2b.1` — opt-in bounded whole-leaf-module
        // sequential-equivalence dedup (decision `0008`). The sequential
        // generalization of the combinational pass above; runs after it, gated
        // identically (node-id / e-graph). Default-off (`default = false`) ⇒
        // every existing design byte-identical.
        if self.cfg.hierarchy_sequential_module_dedup
            && self.cfg.identity_mode == IdentityMode::NodeId
            && self.cfg.effective_factorization_level() >= FactorizationLevel::EGraph
        {
            crate::ir::dedup::dedup_sequential_modules(&mut design);
        }
        // Phase 5: opt-in post-construction width-parameterization
        // annotation. The opt-in *decision* is taken once at
        // construction time by the rules-first
        // `build_parameterizable_leaf` lane (no RNG draw here, so no
        // double-roll). Default-off (prob 0.0) skips the pass entirely
        // → every module byte-identical. Runs after dedup so it
        // annotates surviving modules; the idempotent guard skips
        // modules the constructor lane already annotated, and
        // organically-generated modules are ~never width-generic so
        // they are left untouched.
        if self.cfg.width_parameterization_prob > 0.0 {
            for module in &mut design.modules {
                crate::ir::param::annotate_parameterized(module, &self.cfg);
            }
        }
        // Phase 5b: opt-in post-construction packed-aggregate emitter
        // projection (`PHASE-5B-AGGREGATES.2.1`). The per-module
        // *decision* is rolled here via the seeded generator RNG
        // (reproducible; never `thread_rng`) so `0 < p < 1` is a real
        // Bernoulli choice; `annotate_aggregate` itself is non-rolling.
        // Runs after dedup + the param pass; the pass skips
        // parameterized modules (`.2.1` scoping) and modules with no
        // eligible same-direction data-port group. Default-off (prob
        // 0.0) skips entirely → every module byte-identical.
        // `MULTI-CLOCK-CDC.3b` — opt-in post-construction
        // multi-clock promotion pass. Per `.1`'s design + the
        // rules-first-generation doctrine
        // (`feedback_rules_first_generation.md`), the
        // synchronizer is constructed in place by
        // `multi_clock::promote_to_multi_clock` (which calls the
        // `.3a` primitive); there is no post-pass filter.
        // Default-off (`multi_clock_prob == 0.0`) skips entirely
        // ⇒ every module byte-identical to pre-`.3b` ANVIL —
        // verified by the load-bearing book/snapshot/lib tests.
        // The per-module Bernoulli roll uses the seeded
        // generator RNG (reproducible; never `thread_rng`),
        // mirroring the `aggregate_prob` pattern below.
        if self.cfg.multi_clock_prob > 0.0 {
            let p = self.cfg.multi_clock_prob.clamp(0.0, 1.0);
            for module in &mut design.modules {
                if self.rng.gen_bool(p) {
                    let _ = multi_clock::promote_to_multi_clock_with_stages(
                        module,
                        self.cfg.cdc_synchronizer_stages,
                    );
                }
            }
        }
        if self.cfg.aggregate_prob > 0.0 {
            let p = self.cfg.aggregate_prob.clamp(0.0, 1.0);
            // AGGREGATE-ARRAY-PACKING.3: a second, conditional seeded
            // roll selects a packed *array* over a packed `struct` when
            // the projected group is uniform-width. Guarded by `> 0.0`
            // so the default (`aggregate_array_prob == 0.0`) draws
            // nothing extra from the RNG ⇒ byte-identical stream + output.
            let array_p = self.cfg.aggregate_array_prob.clamp(0.0, 1.0);
            // `.2.1` scaffold scoping: only project modules that are
            // **not instantiated** by any other module. A projected
            // child would change its emitted port surface while the
            // parent-side instance connection still uses the flat port
            // names; rewriting parent-side aggregate connections is
            // deferred to a later `.2.x` sub-slice. Single-module
            // designs and the (never-instantiated) top are eligible;
            // hierarchy children are left flat. Soundness-scoped in
            // the same spirit as Phase 5's planned-child loop.
            let instantiated: std::collections::BTreeSet<String> = design
                .modules
                .iter()
                .flat_map(|m| m.instances.iter().map(|i| i.module.clone()))
                .collect();
            for module in &mut design.modules {
                if instantiated.contains(&module.name) {
                    continue;
                }
                if self.rng.gen_bool(p) {
                    let prefer_array = array_p > 0.0 && self.rng.gen_bool(array_p);
                    crate::ir::aggregate::annotate_aggregate_with_kind(module, prefer_array);
                }
            }
        }
        // `SV-VERSION-TARGETING.3b.2` — opt-in `union soft` low-bits-`Slice`
        // overlay marker (design path), mirroring the single-module roll in
        // `generate_module`. Default `soft_union_slice_prob = 0.0` ⇒ no roll ⇒
        // every module byte-identical. The emitter only realizes the overlay
        // when the target permits 2023; below 2023 a marked gate down-gates to
        // the plain slice.
        if self.cfg.soft_union_slice_prob > 0.0 {
            let p = self.cfg.soft_union_slice_prob;
            for module in &mut design.modules {
                crate::ir::soft_union::annotate_soft_union_slices(module, &mut self.rng, p);
            }
        }
        // `STRUCTURED-EMISSION-EXPANSION.2b.1` — opt-in combinational
        // `function automatic` emit-projection marker (design path),
        // mirroring the single-module roll in `generate_module`. Runs AFTER
        // soft_union so the `union soft` marks are excluded. Default
        // `function_emit_prob = 0.0` ⇒ no roll ⇒ every module
        // byte-identical.
        if self.cfg.function_emit_prob > 0.0 {
            let p = self.cfg.function_emit_prob;
            for module in &mut design.modules {
                crate::ir::function_emit::annotate_function_emit_gates(module, &mut self.rng, p);
            }
        }
        // `STRUCTURED-EMISSION-EXPANSION.4b` — opt-in `generate for` loop
        // emit-projection marker (design path), mirroring the single-module
        // roll in `generate_module`. Runs AFTER function_emit so an already
        // function-emit-marked replication is excluded. Default
        // `generate_loop_emit_prob = 0.0` ⇒ no roll ⇒ every module
        // byte-identical.
        if self.cfg.generate_loop_emit_prob > 0.0 {
            let p = self.cfg.generate_loop_emit_prob;
            for module in &mut design.modules {
                crate::ir::generate_loop::annotate_generate_loop_gates(module, &mut self.rng, p);
            }
        }
        // `STRUCTURED-EMISSION-EXPANSION.6b.1` — opt-in combinational
        // `task automatic` emit-projection marker (design path), mirroring the
        // single-module roll in `generate_module`. Runs AFTER function_emit
        // and generate_loop so an already-marked gate is excluded. Default
        // `task_emit_prob = 0.0` ⇒ no roll ⇒ every module byte-identical.
        if self.cfg.task_emit_prob > 0.0 {
            let p = self.cfg.task_emit_prob;
            for module in &mut design.modules {
                crate::ir::task_emit::annotate_task_emit_gates(module, &mut self.rng, p);
            }
        }
        design
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit;

    #[test]
    fn checkpoint_round_trip_reproduces_next_module() {
        let cfg = Config::default();
        let mut baseline = Generator::new(cfg.clone());
        let _ = baseline.generate_module();
        let checkpoint = baseline.checkpoint();
        let expected = emit::to_sv(&baseline.generate_module());

        let mut restored = Generator::new(cfg);
        restored.restore_checkpoint(&checkpoint);
        let actual = emit::to_sv(&restored.generate_module());

        assert_eq!(actual, expected);
    }

    /// `MULTI-CLOCK-CDC.3b` — end-to-end: `multi_clock_prob = 1.0`
    /// plus a sequential-favoring config produces a design whose
    /// modules (at least one) carry K=2 clock domains and a 2-flop
    /// synchronizer, emitting two `always_ff` blocks in SV.
    /// Defaults (`multi_clock_prob = 0.0`) skip the pass; this
    /// test explicitly opts in.
    #[test]
    fn generate_design_with_multi_clock_prob_one_promotes_at_least_one_module() {
        // Sequential-favoring config so the generated module
        // has at least one flop driving a 1-bit output (the
        // promotion pass's eligibility predicate).
        let cfg = Config {
            seed: 42,
            multi_clock_prob: 1.0,
            flop_prob: 1.0,
            // Force narrow outputs so the 1-bit-only first-cut
            // promotion has a target.
            min_width: 1,
            max_width: 1,
            ..Config::default()
        };
        let mut gen = Generator::new(cfg);
        let design = gen.generate_design();
        // At least one module should have been promoted: K=2
        // clock domains + at least one flop in domain 1.
        let promoted_count = design
            .modules
            .iter()
            .filter(|m| {
                m.clock_domains.len() >= 2 && m.flops.iter().any(|f| m.flop_domain(f.id) == 1)
            })
            .count();
        assert!(
            promoted_count >= 1,
            "expected at least one promoted module; got {promoted_count} \
             (modules={})",
            design.modules.len()
        );
        // Emit must produce two `always_ff` blocks for the
        // promoted module.
        for m in &design.modules {
            if m.clock_domains.len() < 2 {
                continue;
            }
            let sv = emit::to_sv(m);
            let n_blocks = sv.matches("always_ff @(").count();
            assert!(
                n_blocks >= 2,
                "promoted module emit should have ≥2 always_ff blocks; got {n_blocks}:\n{sv}"
            );
            assert!(
                sv.contains("always_ff @(posedge clk_b or negedge rst_n_b)"),
                "promoted module emit missing domain B always_ff:\n{sv}"
            );
        }
    }

    /// `MULTI-CLOCK-CDC.3b` — backward-compat: default
    /// `multi_clock_prob = 0.0` produces zero promotions, and
    /// the generated design is byte-identical to a separate
    /// generator run with the same seed (a regression guard on
    /// the promotion pass's idempotent skip).
    #[test]
    fn default_multi_clock_prob_zero_skips_promotion_entirely() {
        let cfg = Config {
            seed: 7,
            ..Config::default()
        };
        assert_eq!(cfg.multi_clock_prob, 0.0);
        let mut gen = Generator::new(cfg.clone());
        let design = gen.generate_design();
        for m in &design.modules {
            assert!(
                m.clock_domains.is_empty(),
                "default multi_clock_prob should produce no clock_domains entries; \
                 got {} for module {}",
                m.clock_domains.len(),
                m.name
            );
            assert!(
                m.flop_domains.is_empty(),
                "default multi_clock_prob should produce no flop_domains entries"
            );
        }
        // Same seed, same config → byte-identical SV.
        let mut gen2 = Generator::new(cfg);
        let design2 = gen2.generate_design();
        assert_eq!(design.modules.len(), design2.modules.len());
        for (a, b) in design.modules.iter().zip(design2.modules.iter()) {
            assert_eq!(emit::to_sv(a), emit::to_sv(b));
        }
    }
}
