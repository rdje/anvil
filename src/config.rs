//! Knobs: shape, mix, and termination parameters for the generator.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub seed: u64,

    // Structural knobs
    pub min_inputs: u32,
    pub max_inputs: u32,
    pub min_outputs: u32,
    pub max_outputs: u32,
    pub min_width: u32,
    pub max_width: u32,
    pub max_depth: u32,
    pub max_nodes_per_module: u32,

    // Probability knobs
    pub flop_prob: f64,
    pub share_prob: f64,
    pub terminal_reuse_prob: f64,
    pub constant_prob: f64,
    pub library_prob: f64,

    // Gate mix (relative weights, not probabilities)
    pub gate_bitwise_weight: u32,
    pub gate_arith_weight: u32,
    pub gate_struct_weight: u32,
    pub gate_compare_weight: u32,
    pub gate_reduce_weight: u32,

    // Sequential bounds
    pub max_flops_per_module: u32,
    pub min_mux_arms: u32,
    pub max_mux_arms: u32,
    pub flop_qfeedback_prob: f64,
    pub flop_mux_encoding_prob: f64,

    // Hierarchy (Phase 5+)
    pub hierarchy_depth: u32,
    pub num_leaf_modules: u32,

    // Clocking (Phase 2+)
    pub use_async_reset: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 0,
            min_inputs: 2,
            max_inputs: 8,
            min_outputs: 1,
            max_outputs: 4,
            min_width: 1,
            max_width: 32,
            max_depth: 6,
            max_nodes_per_module: 1000,
            flop_prob: 0.15,
            share_prob: 0.0,
            max_flops_per_module: 32,
            min_mux_arms: 1,
            max_mux_arms: 4,
            flop_qfeedback_prob: 0.5,
            flop_mux_encoding_prob: 0.5,
            terminal_reuse_prob: 0.3,
            constant_prob: 0.1,
            library_prob: 0.5,
            gate_bitwise_weight: 3,
            gate_arith_weight: 2,
            gate_struct_weight: 1,
            gate_compare_weight: 1,
            gate_reduce_weight: 1,
            hierarchy_depth: 0,
            num_leaf_modules: 0,
            use_async_reset: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("min_inputs ({0}) > max_inputs ({1})")]
    InputRange(u32, u32),
    #[error("min_outputs ({0}) > max_outputs ({1})")]
    OutputRange(u32, u32),
    #[error("min_width ({0}) > max_width ({1})")]
    WidthRange(u32, u32),
    #[error("probability {name} ({value}) outside [0.0, 1.0]")]
    Probability { name: &'static str, value: f64 },
    #[error("max_depth must be >= 1")]
    DepthTooSmall,
    #[error("min_width must be >= 1")]
    WidthTooSmall,
    #[error("invalid mux arms range: min={0}, max={1} (need 1 <= min <= max)")]
    MuxArmsRange(u32, u32),
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_inputs > self.max_inputs {
            return Err(ConfigError::InputRange(self.min_inputs, self.max_inputs));
        }
        if self.min_outputs > self.max_outputs {
            return Err(ConfigError::OutputRange(self.min_outputs, self.max_outputs));
        }
        if self.min_width > self.max_width {
            return Err(ConfigError::WidthRange(self.min_width, self.max_width));
        }
        if self.min_width < 1 {
            return Err(ConfigError::WidthTooSmall);
        }
        if self.max_depth < 1 {
            return Err(ConfigError::DepthTooSmall);
        }
        if self.min_mux_arms < 1 || self.max_mux_arms < self.min_mux_arms {
            return Err(ConfigError::MuxArmsRange(
                self.min_mux_arms,
                self.max_mux_arms,
            ));
        }
        for (name, value) in [
            ("flop_prob", self.flop_prob),
            ("share_prob", self.share_prob),
            ("terminal_reuse_prob", self.terminal_reuse_prob),
            ("constant_prob", self.constant_prob),
            ("library_prob", self.library_prob),
            ("flop_qfeedback_prob", self.flop_qfeedback_prob),
            ("flop_mux_encoding_prob", self.flop_mux_encoding_prob),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(ConfigError::Probability { name, value });
            }
        }
        Ok(())
    }

    pub fn apply_cli_overrides(&mut self, o: &Overrides) {
        if let Some(v) = o.min_inputs {
            self.min_inputs = v;
        }
        if let Some(v) = o.max_inputs {
            self.max_inputs = v;
        }
        if let Some(v) = o.min_outputs {
            self.min_outputs = v;
        }
        if let Some(v) = o.max_outputs {
            self.max_outputs = v;
        }
        if let Some(v) = o.min_width {
            self.min_width = v;
        }
        if let Some(v) = o.max_width {
            self.max_width = v;
        }
        if let Some(v) = o.max_depth {
            self.max_depth = v;
        }
        if let Some(v) = o.flop_prob {
            self.flop_prob = v;
        }
        if let Some(v) = o.share_prob {
            self.share_prob = v;
        }
    }
}

#[derive(Debug, Default)]
pub struct Overrides {
    pub min_inputs: Option<u32>,
    pub max_inputs: Option<u32>,
    pub min_outputs: Option<u32>,
    pub max_outputs: Option<u32>,
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
    pub max_depth: Option<u32>,
    pub flop_prob: Option<f64>,
    pub share_prob: Option<f64>,
}
