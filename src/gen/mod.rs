//! Generator entry points. See `book/src/algorithm.md`.

pub mod cone;
pub mod hierarchy;
pub mod module;
pub mod pool;

use crate::config::Config;
use crate::ir::{Design, Module};
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
    pub(crate) next_module_index: u64,
}

impl Generator {
    pub fn new(cfg: Config) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(cfg.seed);
        Self {
            rng,
            cfg,
            next_module_index: 0,
        }
    }

    pub fn generate_module(&mut self) -> Module {
        let idx = self.next_module_index;
        self.next_module_index += 1;
        module::generate_leaf_module(self, idx)
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
        if self.cfg.hierarchy_depth == 0 {
            let m = self.generate_module();
            let name = m.name.clone();
            Design {
                top: name,
                modules: vec![m],
            }
        } else {
            hierarchy::generate_design(self)
        }
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
