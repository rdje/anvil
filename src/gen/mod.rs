//! Generator entry points. See `book/src/algorithm.md`.

pub mod cone;
pub mod hierarchy;
pub mod module;
/// `MULTI-CLOCK-CDC.3a` — 2-flop synchronizer construction
/// primitive for cross-clock-domain signals. Per
/// `MULTI-CLOCK-CDC.1`'s design + the rules-first generation
/// doctrine (`feedback_rules_first_generation.md`), the
/// synchronizer is constructed in place when the generator
/// makes a domain-crossing decision; never via a post-pass
/// filter. `.3b` wires this primitive into the per-module
/// generator's domain-crossing decision path.
pub mod multi_clock;
pub mod pool;

use crate::config::Config;
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
        module::generate_leaf_module(self, idx)
    }

    pub fn generate_module_with_interface_profile(
        &mut self,
        interface_profile: Option<&ModuleInterfaceProfile>,
    ) -> Module {
        let idx = self.reserve_module_index();
        module::generate_leaf_module_with_interface_profile(self, idx, interface_profile)
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
        if self.cfg.aggregate_prob > 0.0 {
            let p = self.cfg.aggregate_prob.clamp(0.0, 1.0);
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
                    crate::ir::aggregate::annotate_aggregate(module);
                }
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
}
