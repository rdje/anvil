//! Generator entry points. See `book/src/algorithm.md`.

pub mod cone;
pub mod module;
pub mod pool;
// pub mod hierarchy; // Phase 5+

use crate::config::Config;
use crate::ir::{Design, Module};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

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

    pub fn generate_design(&mut self) -> Design {
        // Phase 5+: populate a library, generate a top with hierarchy.
        let m = self.generate_module();
        let name = m.name.clone();
        Design {
            top: name,
            modules: vec![m],
        }
    }
}
