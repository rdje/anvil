//! Generator entry points. See `book/src/algorithm.md`.

pub mod cone;
pub mod hierarchy;
pub mod module;
pub mod pool;

use crate::config::Config;
use crate::ir::{Design, KnobId, Module, ModuleInterfaceProfile};
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
        // Phase 5: opt-in post-construction width parameterization.
        // Default-off (prob 0.0) leaves every module byte-identical.
        // Runs after dedup so it annotates the surviving modules.
        if self.cfg.width_parameterization_prob > 0.0 {
            for module in &mut design.modules {
                crate::ir::param::parameterize_module(module, &mut self.rng, &self.cfg);
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
