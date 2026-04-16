//! Knobs: shape, mix, and termination parameters for the generator.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Strategy for constructing a module's internal logic.
///
/// See `book/src/construction-strategies.md` for the full comparison.
/// Only `Sequential` and `Shuffled` are implemented today; `Interleaved`
/// and `GraphFirst` will land in later slices. When `GraphFirst` lands
/// it becomes the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ConstructionStrategy {
    /// Build cones per-output in declaration order.
    Sequential,
    /// Build cones per-output in a random permutation of declaration order.
    Shuffled,
    /// Build signal-level frames across all cones from one global work
    /// queue, popping a random frame each step. Cones grow in lockstep
    /// so each cone's leaves see gates built by other cones' earlier
    /// frames. Near-symmetric within-module sharing. Blocks (flop,
    /// comb-mux) still build synchronously within one frame step; flop
    /// D-cones are drained synchronously at the end (as today).
    Interleaved,
    /// Deprecated alias for `Interleaved`. The original `GraphFirst`
    /// implementation grew a gate pool speculatively before any
    /// drive-roots were picked, producing 10–30 % orphan gates per
    /// module (Rule 18 violation). Retained for CLI / config-file
    /// backward compatibility only; silently routes to `Interleaved`.
    /// See `book/src/construction-strategies.md`.
    #[serde(alias = "graph-first", alias = "graph_first")]
    GraphFirst,
}

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

    // Operator arity for the associative operators (And/Or/Xor/Add/Mul).
    // N = rand(min_gate_arity, max_gate_arity), inclusive.
    // Arity only applies to operators — blocks (mux, flop) have ports,
    // not arity. Sub is strictly 2-arity (not associative).
    pub min_gate_arity: u32,
    pub max_gate_arity: u32,

    // Coefficient motif: when `build_cone` picks Add or Sub, with
    // probability `coefficient_prob` replace the standard operand
    // recursion with a linear-combination compound:
    //   Add: y = s1*c1 + s2*c2 + ... + sN*cN
    //   Sub: y = s1*c1 - s2*c2 - ... - sN*cN  (left-associative)
    // Each ck is a strictly positive integer drawn from
    // [min_coefficient, max_coefficient]. N is drawn from
    // [min_gate_arity, max_gate_arity]. See `book/src/structural-rules.md`
    // "Roles of constants in RTL".
    pub coefficient_prob: f64,
    pub min_coefficient: u32,
    pub max_coefficient: u32,

    // Shift-amount motif: when `build_cone` picks `Shl` or `Shr`, the
    // shift-amount operand is either a recursive signal sub-cone
    // (variable-amount shift — barrel shifter in hardware) or a
    // constant literal drawn from [min_shift_amount, max_shift_amount]
    // clamped to [0, W-1] for a W-bit value. Real designs
    // overwhelmingly use constant shift amounts, so the default
    // biases strongly toward constant. See
    // `book/src/structural-rules.md` "Roles of constants in RTL".
    pub const_shift_amount_prob: f64,
    pub min_shift_amount: u32,
    pub max_shift_amount: u32,

    // Relative weight for the shifts (Shl/Shr) bucket in `pick_gate`.
    pub gate_shift_weight: u32,

    // Comparand motif: when `build_cone` picks a comparison op
    // (Eq/Neq/Lt/Gt/Le/Ge), with probability `const_comparand_prob`
    // the RHS operand is a constant literal drawn from
    // [min_comparand, max_comparand] (clamped to fit the chosen
    // internal operand width K). Additive to signal-vs-signal
    // comparisons — the LHS is still a signal. No zero-exclusion.
    // See `book/src/structural-rules.md` "Roles of constants in RTL".
    pub const_comparand_prob: f64,
    pub min_comparand: u32,
    pub max_comparand: u32,

    // Priority-encoder block: takes N 1-bit request signals and emits
    // a ceil(log2(N))-bit index of the highest-priority asserted bit
    // (lowest-indexed). Emitted as a chained ternary. N is drawn from
    // `[min_mux_arms, max_mux_arms]` constrained to have
    // `ceil(log2(N))` == the caller's target width. See
    // `book/src/structural-rules.md`.
    pub priority_encoder_prob: f64,

    // Sequential bounds
    pub max_flops_per_module: u32,
    pub min_mux_arms: u32,
    pub max_mux_arms: u32,
    pub flop_qfeedback_prob: f64,
    pub flop_mux_encoding_prob: f64,
    pub comb_mux_prob: f64,
    pub comb_mux_encoding_prob: f64,

    // Hierarchy (Phase 5+)
    pub hierarchy_depth: u32,
    pub num_leaf_modules: u32,

    // Clocking (Phase 2+)
    pub use_async_reset: bool,

    // How to schedule cone construction across outputs. See
    // `book/src/construction-strategies.md`.
    pub construction_strategy: ConstructionStrategy,

    /// Target number of top-level units (operator gate / flop /
    /// comb-mux block) grown in the pool by the `GraphFirst`
    /// strategy. Only consulted when `construction_strategy ==
    /// GraphFirst`. Does not count the internal primitive gates
    /// generated inside comb-mux assembly or flop-mux assembly.
    pub graph_first_pool_size: u32,

    /// Rate at which arms of an N-to-1 mux are permitted to share
    /// the same data signal. `0.0` (default) = every arm must be
    /// a distinct signal; `1.0` = no constraint (all arms may be
    /// connected to the same data); intermediate values permit
    /// duplication probabilistically.
    ///
    /// At each arm pick, if the candidate signal would duplicate
    /// a signal already picked for this mux, it is kept with
    /// probability `mux_arm_duplication_rate` and rejected
    /// (pick again) otherwise. Bounded retries — after an 8-try
    /// budget the candidate is accepted regardless to avoid
    /// pathological re-pick loops when the pool is too small.
    ///
    /// Applies uniformly to 2-to-1 `Mux` gates, N-to-1 one-hot
    /// muxes, and N-to-1 encoded chained-ternary muxes (comb and
    /// flop-D variants).
    pub mux_arm_duplication_rate: f64,

    /// Maximum number of times a given AST (gate expression /
    /// constant) may be materialised as a named node in one module.
    /// Default 1 → strict uniqueness (CSE): an expression is named
    /// exactly once and every consumer references that single node.
    /// N > 1 → bounded duplication: up to N named copies before
    /// callers are routed to the most-recent existing instance.
    /// `u32::MAX` → effectively no deduplication.
    ///
    /// When debugging it can be useful to raise this knob to see how
    /// much duplication the construction strategies would naturally
    /// produce; for production seed sweeps, leave it at 1.
    pub max_ast_instances: u32,
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
            share_prob: 0.3,
            min_gate_arity: 2,
            max_gate_arity: 4,
            coefficient_prob: 0.2,
            min_coefficient: 1,
            max_coefficient: 15,
            const_shift_amount_prob: 0.8,
            min_shift_amount: 0,
            max_shift_amount: 7,
            gate_shift_weight: 1,
            const_comparand_prob: 0.3,
            min_comparand: 0,
            max_comparand: 255,
            priority_encoder_prob: 0.05,
            max_flops_per_module: 32,
            min_mux_arms: 1,
            max_mux_arms: 4,
            flop_qfeedback_prob: 0.5,
            flop_mux_encoding_prob: 0.5,
            comb_mux_prob: 0.1,
            comb_mux_encoding_prob: 0.5,
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
            construction_strategy: ConstructionStrategy::Interleaved,
            graph_first_pool_size: 32,
            mux_arm_duplication_rate: 0.0,
            max_ast_instances: 1,
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
    #[error("invalid gate arity range: min={0}, max={1} (need 2 <= min <= max)")]
    GateArityRange(u32, u32),
    #[error("invalid coefficient range: min={0}, max={1} (need 1 <= min <= max)")]
    CoefficientRange(u32, u32),
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
        if self.min_gate_arity < 2 || self.max_gate_arity < self.min_gate_arity {
            return Err(ConfigError::GateArityRange(
                self.min_gate_arity,
                self.max_gate_arity,
            ));
        }
        if self.min_coefficient < 1 || self.max_coefficient < self.min_coefficient {
            return Err(ConfigError::CoefficientRange(
                self.min_coefficient,
                self.max_coefficient,
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
            ("comb_mux_prob", self.comb_mux_prob),
            ("comb_mux_encoding_prob", self.comb_mux_encoding_prob),
            ("coefficient_prob", self.coefficient_prob),
            ("const_shift_amount_prob", self.const_shift_amount_prob),
            ("const_comparand_prob", self.const_comparand_prob),
            ("priority_encoder_prob", self.priority_encoder_prob),
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
        if let Some(v) = o.max_flops_per_module {
            self.max_flops_per_module = v;
        }
        if let Some(v) = o.min_mux_arms {
            self.min_mux_arms = v;
        }
        if let Some(v) = o.max_mux_arms {
            self.max_mux_arms = v;
        }
        if let Some(v) = o.flop_qfeedback_prob {
            self.flop_qfeedback_prob = v;
        }
        if let Some(v) = o.flop_mux_encoding_prob {
            self.flop_mux_encoding_prob = v;
        }
        if let Some(v) = o.min_gate_arity {
            self.min_gate_arity = v;
        }
        if let Some(v) = o.max_gate_arity {
            self.max_gate_arity = v;
        }
        if let Some(v) = o.comb_mux_prob {
            self.comb_mux_prob = v;
        }
        if let Some(v) = o.comb_mux_encoding_prob {
            self.comb_mux_encoding_prob = v;
        }
        if let Some(v) = o.construction_strategy {
            self.construction_strategy = v;
        }
        if let Some(v) = o.graph_first_pool_size {
            self.graph_first_pool_size = v;
        }
        if let Some(v) = o.coefficient_prob {
            self.coefficient_prob = v;
        }
        if let Some(v) = o.min_coefficient {
            self.min_coefficient = v;
        }
        if let Some(v) = o.max_coefficient {
            self.max_coefficient = v;
        }
        if let Some(v) = o.const_shift_amount_prob {
            self.const_shift_amount_prob = v;
        }
        if let Some(v) = o.min_shift_amount {
            self.min_shift_amount = v;
        }
        if let Some(v) = o.max_shift_amount {
            self.max_shift_amount = v;
        }
        if let Some(v) = o.gate_shift_weight {
            self.gate_shift_weight = v;
        }
        if let Some(v) = o.const_comparand_prob {
            self.const_comparand_prob = v;
        }
        if let Some(v) = o.min_comparand {
            self.min_comparand = v;
        }
        if let Some(v) = o.max_comparand {
            self.max_comparand = v;
        }
        if let Some(v) = o.priority_encoder_prob {
            self.priority_encoder_prob = v;
        }
        if let Some(v) = o.max_ast_instances {
            self.max_ast_instances = v;
        }
        if let Some(v) = o.mux_arm_duplication_rate {
            self.mux_arm_duplication_rate = v;
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
    pub max_flops_per_module: Option<u32>,
    pub min_mux_arms: Option<u32>,
    pub max_mux_arms: Option<u32>,
    pub flop_qfeedback_prob: Option<f64>,
    pub flop_mux_encoding_prob: Option<f64>,
    pub min_gate_arity: Option<u32>,
    pub max_gate_arity: Option<u32>,
    pub comb_mux_prob: Option<f64>,
    pub comb_mux_encoding_prob: Option<f64>,
    pub construction_strategy: Option<ConstructionStrategy>,
    pub graph_first_pool_size: Option<u32>,
    pub coefficient_prob: Option<f64>,
    pub min_coefficient: Option<u32>,
    pub max_coefficient: Option<u32>,
    pub const_shift_amount_prob: Option<f64>,
    pub min_shift_amount: Option<u32>,
    pub max_shift_amount: Option<u32>,
    pub gate_shift_weight: Option<u32>,
    pub const_comparand_prob: Option<f64>,
    pub min_comparand: Option<u32>,
    pub max_comparand: Option<u32>,
    pub priority_encoder_prob: Option<f64>,
    pub max_ast_instances: Option<u32>,
    pub mux_arm_duplication_rate: Option<f64>,
}
