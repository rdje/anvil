//! Knobs: shape, mix, and termination parameters for the generator.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Strategy for constructing a module's internal logic.
///
/// See `book/src/construction-strategies.md` for the full comparison.
/// `Sequential`, `Shuffled`, and `Interleaved` are live today.
/// `GraphFirst` is retained as a deprecated alias for `Interleaved`
/// so older configs and CLI invocations keep working.
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

/// Inclusive integer range used for hierarchy-planning bounds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountRange {
    pub min: u32,
    pub max: u32,
}

/// How Phase 4 parents obtain child module definitions.
///
/// `Library` keeps the current reusable-definition story live:
/// parent planning first builds a child library, then instance slots
/// pick from that pool. `OnDemand` instead synthesizes a fresh child
/// definition for each planned instance slot. In the current stronger
/// Phase 4 slice, those fresh children are synthesized against
/// parent-planned exact data-interface profiles rather than choosing
/// their own data boundary locally. This is intentionally orthogonal
/// to hierarchy depth and branching: both the legacy depth-1 wrapper
/// lane and the recursive lane can now choose whether children come
/// from a reusable library or from fresh per-instance synthesis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum HierarchyChildSourceMode {
    /// Pre-generate a reusable child-definition pool and instantiate
    /// from it.
    #[default]
    Library,
    /// Synthesize a fresh child definition for each planned instance
    /// slot against a parent-planned exact data-interface profile.
    /// Reuse then has to be asked for explicitly in future
    /// hierarchy-aware identity work rather than appearing from the
    /// child planner.
    OnDemand,
}

fn default_hierarchy_child_input_cone_prob() -> f64 {
    0.35
}

fn default_hierarchy_parent_flop_prob() -> f64 {
    0.0
}

fn default_width_parameterization_prob() -> f64 {
    0.0
}

fn default_aggregate_prob() -> f64 {
    0.0
}

fn default_aggregate_array_prob() -> f64 {
    0.0
}

fn default_soft_union_slice_prob() -> f64 {
    0.0
}

fn default_memory_prob() -> f64 {
    0.0
}

fn default_fsm_prob() -> f64 {
    0.0
}

/// `MULTI-CLOCK-CDC.3b` — per-module roll for the multi-clock
/// promotion pass. Defaults to `0.0` so every existing run is
/// byte-identical to pre-`.3b` ANVIL.
fn default_multi_clock_prob() -> f64 {
    0.0
}

fn default_cdc_synchronizer_stages() -> u32 {
    2
}

fn default_hierarchy_registered_sibling_route_prob() -> f64 {
    0.0
}
fn default_hierarchy_registered_sibling_mixed_support_prob() -> f64 {
    0.0
}

fn default_hierarchy_registered_child_input_cone_prob() -> f64 {
    0.0
}

fn default_hierarchy_parent_cone_instance_prob() -> f64 {
    0.0
}

fn default_max_parent_cone_instances_per_module() -> u32 {
    1
}

/// Identity mode — the coarse answer to "what does a `NodeId`
/// mean?".
///
/// This is intentionally orthogonal to `ConstructionStrategy`:
/// construction strategy decides *how* fanin cones are walked and
/// built, while identity mode decides *when* two built expressions
/// must share one `NodeId`.
///
/// `NodeId` (default) means `NodeId` is the identity of an
/// expression — the full-factorization doctrine. The
/// `factorization_level` ladder does not redefine that meaning; it
/// selects how much of that doctrine the current build can
/// currently enforce/prove.
///
/// `Relaxed` disables the identity/factorization ladder entirely:
/// every `intern_gate` / `intern_constant` call allocates a fresh
/// `NodeId` regardless of the requested `factorization_level`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum IdentityMode {
    /// Relaxed identity: disable the factorization ladder and let
    /// every AST materialise as a fresh node. Useful for debugging
    /// downstream CSE-sensitive tooling or measuring the raw
    /// duplication the construction strategy would otherwise
    /// produce.
    #[value(alias = "off")]
    Relaxed,
    /// NodeId = expression identity, i.e. the full-factorization
    /// doctrine. The requested `factorization_level` stays live as
    /// the current-build enforcement/proof-depth dial and is clamped
    /// to the strongest implemented rung by `effective()`.
    #[default]
    #[serde(alias = "nodeid", alias = "node_id")]
    #[value(alias = "nodeid", alias = "node_id")]
    NodeId,
}

/// Progressive factorization dial along the full chain:
/// `none → cse → operand-unique → commutative → associative →
/// constant-fold → peephole → e-graph`. Each level implies all
/// lower ones. Default `e-graph` (theoretical ceiling — the
/// generator activates every layer it knows how to implement;
/// future slices add more without a config change).
///
/// This dial is subordinate to `IdentityMode`: in
/// `IdentityMode::Relaxed` the effective level is forced to `none`
/// regardless of the requested rung.
///
/// See `book/src/structural-rules.md` Rule 21b for the chain,
/// motivation, and "NodeId = identity of an expression" doctrine.
#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum FactorizationLevel {
    /// Weakest implementation rung inside `identity_mode = node-id`.
    /// No dedup of any kind. Every `intern_gate` call creates a
    /// fresh `NodeId`, even for identical ASTs. Useful for
    /// debugging CSE-sensitive downstream tools and for matrix
    /// coverage; not the doctrinal meaning of `NodeId` identity.
    None,
    /// Syntactic CSE: `(op, operands, width)` identifies a node.
    /// Same-key calls share `NodeId` (up to `max_ast_instances`).
    Cse,
    /// CSE + operand uniqueness. No same `NodeId` appears twice
    /// in one operator gate's operand list (per Rule 8 extended).
    OperandUnique,
    /// Commutative normalization on top of operand uniqueness.
    /// Operand lists of `And`/`Or`/`Xor`/`Add`/`Mul` are sorted
    /// ascending before interning, so `a + b` and `b + a` share
    /// identity (Rule 21b).
    Commutative,
    /// Associative flattening on top of commutative normalization.
    /// **Implemented** — at intern time, any operand of an
    /// `And`/`Or`/`Xor`/`Add`/`Mul` gate that is itself a same-op
    /// same-width gate is spliced into the outer operand list, so
    /// `Add(a, Add(b, c))` becomes `Add(a, b, c)` and shares
    /// identity with `Add(a + b + c)` built any other way. Per-op
    /// semantic normalisation: `And`/`Or` dedup (idempotent),
    /// `Xor` pair-cancel, `Add`/`Mul` conservative (skip when
    /// duplicates would result at strict `operand_duplication_rate`
    /// to preserve `x + x = 2x` / `x * x = x²` semantics). Inner
    /// gates orphaned by the splice are cleaned up by
    /// `compact_node_ids` at module finalisation. Fires counted
    /// in `Metrics::flatten_associative_applied`.
    Associative,
    /// Constant folding on top of associative flattening.
    /// **Implemented** as of the ConstantFold slice. Algebraic
    /// identities fire at intern time: `x + 0 → x`, `x * 1 → x`,
    /// `x & 0 → 0`, `x | all_ones → all_ones`, `x ^ 0 → x`,
    /// `x * 0 → 0`, `x & all_ones → x`, `x - 0 → x`,
    /// `x << 0 → x`, `x >> 0 → x`. Fires counted in
    /// `Metrics::fold_identities_applied`.
    ConstantFold,
    /// Peephole rewrite rules on top of constant folding.
    /// **Implemented** as a curated set of local, unambiguous
    /// rewrites: `Not(Not(x)) → x`, fully-constant comparisons
    /// evaluated at intern time, full-width `Slice(hi, 0)` with
    /// `hi + 1 == src_width` returning the source, and single-
    /// operand `Concat → that operand`. Cross-gate algebraic
    /// rewrites like `(a + b) - b = a` are still deferred to the
    /// future e-graph layer. Fires counted in
    /// `Metrics::peephole_rewrites_applied`.
    Peephole,
    /// Theoretical ceiling — semantic equivalence via e-graph.
    /// **Default**, and still the aspiration: every mathematically-
    /// equivalent expression shares one `NodeId`. The full engine is
    /// not here yet, but this rung is now partially live: under
    /// `identity_mode = node-id`, ANVIL runs the full intern-time
    /// ladder plus a bounded post-construction semantic-sharing
    /// fragment for small-support combinational cones. Future slices
    /// can strengthen this rung without requiring users to change
    /// their config — they progressively get tighter factorization
    /// "for free."
    #[default]
    EGraph,
}

impl FactorizationLevel {
    /// Whether this specific layer is implemented today. Every
    /// current rung is live; `effective()` keeps this helper so a
    /// future aspirational rung can be added without accidentally
    /// enabling a layer the generator does not yet implement.
    pub fn is_implemented(self) -> bool {
        matches!(
            self,
            FactorizationLevel::None
                | FactorizationLevel::Cse
                | FactorizationLevel::OperandUnique
                | FactorizationLevel::Commutative
                | FactorizationLevel::Associative
                | FactorizationLevel::ConstantFold
                | FactorizationLevel::Peephole
                | FactorizationLevel::EGraph
        )
    }

    /// Highest layer that is actually implemented in the current
    /// build. Today this is `EGraph`'s bounded semantic fragment; the
    /// walk remains defensive for future ladder extensions.
    pub fn highest_implemented() -> Self {
        // Walk down from EGraph until we find an implemented layer.
        for lvl in [
            FactorizationLevel::EGraph,
            FactorizationLevel::Peephole,
            FactorizationLevel::ConstantFold,
            FactorizationLevel::Associative,
            FactorizationLevel::Commutative,
            FactorizationLevel::OperandUnique,
            FactorizationLevel::Cse,
            FactorizationLevel::None,
        ] {
            if lvl.is_implemented() {
                return lvl;
            }
        }
        FactorizationLevel::None
    }

    /// Effective level: returns the highest *implemented* layer at
    /// or below `self`. Use this at every gating site instead of
    /// comparing `self` directly, so a user request like `EGraph`
    /// activates every live layer today while preserving a clean
    /// fallback path for future aspirational rungs.
    pub fn effective(self) -> Self {
        for lvl in [
            FactorizationLevel::EGraph,
            FactorizationLevel::Peephole,
            FactorizationLevel::ConstantFold,
            FactorizationLevel::Associative,
            FactorizationLevel::Commutative,
            FactorizationLevel::OperandUnique,
            FactorizationLevel::Cse,
            FactorizationLevel::None,
        ] {
            if lvl <= self && lvl.is_implemented() {
                return lvl;
            }
        }
        FactorizationLevel::None
    }
}

/// Target IEEE 1800 SystemVerilog standard for emission
/// (`SV-VERSION-TARGETING`, decision `0009`).
///
/// An opt-in capability gate with two construction-time effects, both
/// rules-first (no generate-then-filter):
/// - **down-gating** (a guarantee): the emitter never emits a construct
///   newer than the target, so output stays valid for a tool/flow pinned
///   to that standard;
/// - **up-opting** (future, `SV-VERSION-TARGETING.3`): a higher target may
///   deliberately emit that standard's distinctive synthesizable
///   constructs, each gated on `target.permits(that_standard)` and proven
///   downstream-clean in the matching tool standard mode.
///
/// `Ord` follows declaration order (`Sv2012 < Sv2017 < Sv2023`) so a
/// capability check reads `target.permits(SvVersion::Sv2017)`.
///
/// **Default `Sv2012`** is the honest floor: ANVIL's entire current emitted
/// subset (`logic` / `always_ff` / `always_comb` / packed `struct` / packed
/// arrays / `typedef` / `localparam`) is valid in IEEE 1800-2012, so the
/// default reproduces today's emission byte-for-byte and down-gating *to the
/// floor* removes nothing. CLI / serde value spelling is the bare year
/// (`--sv-version 2017`; `"sv_version": "2012"`).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
pub enum SvVersion {
    /// IEEE 1800-2012 — the current emitted floor subset (default).
    #[default]
    #[serde(rename = "2012")]
    #[value(name = "2012")]
    Sv2012,
    /// IEEE 1800-2017.
    #[serde(rename = "2017")]
    #[value(name = "2017")]
    Sv2017,
    /// IEEE 1800-2023.
    #[serde(rename = "2023")]
    #[value(name = "2023")]
    Sv2023,
}

impl SvVersion {
    /// Whether emission targeting `self` permits a construct introduced by
    /// `introduced` — the down-gating capability bound: a construct is legal
    /// only when the target is at least the standard that introduced it.
    /// Today every emitted construct's introducing standard is `Sv2012`, so
    /// this is `true` for every target; the bound is real but vacuous until
    /// the first up-opted construct (`SV-VERSION-TARGETING.3`).
    pub fn permits(self, introduced: SvVersion) -> bool {
        self >= introduced
    }

    /// The IEEE 1800 standard label (`"1800-2012"` / `"1800-2017"` /
    /// `"1800-2023"`), e.g. for a downstream tool's language selector
    /// (`SV-VERSION-TARGETING.2b.2`).
    pub fn ieee_standard(self) -> &'static str {
        match self {
            SvVersion::Sv2012 => "1800-2012",
            SvVersion::Sv2017 => "1800-2017",
            SvVersion::Sv2023 => "1800-2023",
        }
    }
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
    /// Per-module construction-time node budget (`WORKLOAD-MEMORY-SAFETY.3`).
    /// Sentinel `0` = unlimited (the default; byte-identical to the
    /// historical unbounded behaviour). When non-zero, cone construction
    /// stops opening new sub-cones once the module's node arena
    /// (`Module::nodes`) reaches this many nodes — steering to existing
    /// terminals (rules-first; it never truncates a finished cone), so a
    /// pathological `(seed, knobs)` cannot grow one module's `Vec<Node>`
    /// without bound. A *soft* ceiling: a bounded number of
    /// terminal/adapter nodes may still be appended to legally close
    /// already-open frames. Its effect is measured by `Metrics::num_nodes`.
    pub max_nodes_per_module: u32,

    /// Opt-in process-RSS abort ceiling in MiB (`WORKLOAD-MEMORY-SAFETY.4`).
    /// Sentinel `0` = off (the default; no sampling ⇒ byte-identical). When
    /// `> 0`, between-unit checkpoints in the `--out` loop abort the run
    /// cleanly (deterministic non-zero exit, a stderr message naming the
    /// seed + effective knobs) once this process's resident set reaches the
    /// ceiling. A process-safety governor, **not** a generation knob: it
    /// never alters emitted RTL (it declines to start more units; it never
    /// truncates a built cone). See `src/mem_guard.rs`.
    #[serde(default)]
    pub max_rss_mb: u64,
    /// Opt-in host used-RAM abort percentage (`WORKLOAD-MEMORY-SAFETY.4`).
    /// Sentinel `0` = off (the default). When in `1..=100`, between-unit
    /// `--out` checkpoints abort once host used RAM reaches it, mirroring
    /// `scripts/ram_guard.sh` (macOS `memory_pressure` / Linux
    /// `/proc/meminfo`). Default-off ⇒ byte-identical. See `src/mem_guard.rs`.
    #[serde(default)]
    pub ram_abort_pct: u32,

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

    // Case-mux block: takes one encoded select bus plus M data inputs
    // and emits a procedural `always_comb case (sel)` block with an
    // explicit default to zero. M is drawn from
    // `[max(2, min_mux_arms), max_mux_arms]`; sel width is
    // `ceil(log2(M))`. This is a syntax-surface motif distinct from
    // the expression-level mux tree.
    pub case_mux_prob: f64,

    // Casez-mux block: takes one encoded select bus plus M data inputs
    // and emits a procedural `always_comb casez (sel)` block with
    // wildcard patterns and an explicit default to zero. The emitted
    // patterns are generated non-overlapping by construction so the
    // block remains a pure wildcarded mux surface rather than an
    // accidental priority encoder.
    pub casez_mux_prob: f64,

    // For-fold block: takes one packed source bus whose width is
    // `trip_count * chunk_width`, then emits an `always_comb` block
    // with a statically bounded `for (int i = 0; i < N; i++)`
    // accumulator over fixed-size chunks. This is a syntax-surface
    // motif distinct from the expression-level N-ary operators.
    pub for_fold_prob: f64,

    // Sequential bounds
    pub max_flops_per_module: u32,
    pub min_mux_arms: u32,
    pub max_mux_arms: u32,
    pub flop_qfeedback_prob: f64,
    pub flop_mux_encoding_prob: f64,
    pub comb_mux_prob: f64,
    pub comb_mux_encoding_prob: f64,

    // Hierarchy (Phase 4+)
    pub hierarchy_depth: u32,
    pub num_leaf_modules: u32,
    #[serde(default)]
    pub num_child_instances: u32,
    #[serde(default)]
    pub hierarchy_child_source_mode: HierarchyChildSourceMode,
    #[serde(default)]
    pub min_hierarchy_depth: u32,
    #[serde(default)]
    pub max_hierarchy_depth: u32,
    #[serde(default)]
    pub min_child_instances_per_module: u32,
    #[serde(default)]
    pub max_child_instances_per_module: u32,
    /// Optional per-parent-depth override for recursive hierarchy
    /// branching. Keys are internal parent depths (`0` = top).
    #[serde(default)]
    pub child_instances_per_module_by_depth: BTreeMap<u32, CountRange>,
    /// Probability that a parent binds a child data input from a
    /// previously-instantiated sibling output when one is available.
    /// The sibling-routing graph is always acyclic by construction:
    /// only earlier sibling outputs may feed later sibling inputs.
    /// When the roll misses, the current parent falls back to its
    /// external input boundary.
    pub hierarchy_sibling_route_prob: f64,
    /// Probability that a parent binds a later child data input through
    /// a local parent flop. The first route sources the flop D side
    /// from an earlier sibling output; later routes may also reuse an
    /// earlier parent-local Q as the D source, which creates a
    /// multi-stage registered sibling chain. This is the registered
    /// sibling-routing counterpart to `hierarchy_sibling_route_prob`;
    /// default 0.0 preserves the current combinational direct route
    /// unless explicitly requested.
    #[serde(default = "default_hierarchy_registered_sibling_route_prob")]
    pub hierarchy_registered_sibling_route_prob: f64,
    /// Probability that a direct registered sibling route also mixes a
    /// parent data-port companion into the flop D side before driving
    /// the later child input. This keeps the direct registered
    /// child-to-child route live while exercising parent-port support
    /// in the same D cone. Default 0.0 preserves the original direct
    /// registered sibling structure unless explicitly requested.
    #[serde(default = "default_hierarchy_registered_sibling_mixed_support_prob")]
    pub hierarchy_registered_sibling_mixed_support_prob: f64,
    /// Probability that a parent binds a later child data input through
    /// local parent combinational logic over already-available parent
    /// sources and then one local parent flop. When parent data inputs
    /// and sibling outputs are both live, the route can mix both
    /// supports; when earlier parent flops are available, the route can
    /// also build a multi-stage registered chain. This is the registered
    /// counterpart to `hierarchy_child_input_cone_prob` and proves the
    /// structure: parent source(s) -> parent logic -> parent flop ->
    /// later child input.
    #[serde(default = "default_hierarchy_registered_child_input_cone_prob")]
    pub hierarchy_registered_child_input_cone_prob: f64,
    /// Probability that a parent binds a child data input through a
    /// local combinational cone over already-available parent sources:
    /// current parent data inputs, earlier sibling instance outputs,
    /// and parent-side route gates already built for previous child
    /// bindings. Local parent flops are controlled separately by
    /// `hierarchy_parent_flop_prob`.
    #[serde(default = "default_hierarchy_child_input_cone_prob")]
    pub hierarchy_child_input_cone_prob: f64,
    /// Probability that a parent-composed child-input cone or parent-output
    /// cone may instantiate an extra child module as an internal parent-cone
    /// source before building the cone. Default 0.0 preserves the current
    /// planned child instance set unless this axis is explicitly requested.
    #[serde(default = "default_hierarchy_parent_cone_instance_prob")]
    pub hierarchy_parent_cone_instance_prob: f64,
    /// Maximum number of parent-cone helper child instances one hierarchy
    /// parent may instantiate. Default 1 preserves the first landed helper
    /// slice; set to 0 to disable helper insertion even when
    /// `hierarchy_parent_cone_instance_prob` is non-zero.
    #[serde(default = "default_max_parent_cone_instances_per_module")]
    pub max_parent_cone_instances_per_module: u32,
    /// Probability that parent-side hierarchy cones may emit local
    /// parent flops. This applies to parent output cones and
    /// parent-composed child-input cones. Default 0.0 preserves the
    /// current combinational hierarchy unless explicitly requested.
    #[serde(default = "default_hierarchy_parent_flop_prob")]
    pub hierarchy_parent_flop_prob: f64,

    /// When `true`, the generator runs `crate::ir::dedup::dedup_modules`
    /// after `generate_design` finishes assembling the design's
    /// modules. The pass collapses every group of `Module`s in
    /// `design.modules` that share a canonical structural signature
    /// (the same FNV-1a 64-bit hash recorded in
    /// `DesignMetrics.canonical_module_signatures`) to a single
    /// surviving entry and rewrites every `Instance.module`
    /// reference in the surviving Modules so they point at the
    /// survivor. The top module is never merged away. The pass is
    /// opt-in: `default = false` preserves existing behaviour.
    /// First slice: `HIERARCHY-AWARE-IDENTITY.4` — see
    /// `docs/tasks/HIERARCHY-AWARE-IDENTITY.md`.
    #[serde(default)]
    pub hierarchy_module_dedup: bool,

    /// When `true`, the generator runs the bounded semantic module
    /// dedup pass after structural module dedup. The pass is narrower
    /// than an arbitrary module-equivalence engine: it only admits
    /// pure combinational, instance-free, concrete non-top modules
    /// with identical emitted data port IDs/widths and <= 12 bits of
    /// total input support. It is effective only under
    /// `identity_mode = node-id` with effective `factorization_level`
    /// `e-graph`; `identity_mode = relaxed` remains the semantic
    /// off-switch. Default `false` preserves existing hierarchy output.
    /// First slice: `HIERARCHY-SEMANTIC-IDENTITY.1`.
    #[serde(default)]
    pub hierarchy_semantic_module_dedup: bool,

    /// When `true`, finalization runs the opt-in bounded bisimulation
    /// flop-merge pass (`crate::ir::compact::merge_bisimilar_flops`,
    /// `IDENTITY-DEEPENING`, decision `0007`) after the exact
    /// `merge_equivalent_flops` pass. It is a greatest-fixpoint partition
    /// refinement that merges flops proven sequentially equivalent *up to
    /// a state correspondence* — for example mutually-recursive registers
    /// whose D-cones reference each other's `Q` — a class the exact
    /// reset-defined self-hold rule provably cannot prove. It is effective
    /// only under `identity_mode = node-id` with effective
    /// `factorization_level` `e-graph`; `identity_mode = relaxed` remains
    /// the semantic off-switch. Resetless flops are excluded (no reset
    /// base case). Proofs reuse the same 12-bit / 128-node / 131072-work
    /// budget as the semantic gate merge; over-budget cones fall back to
    /// structural identity. `default = false` keeps every existing output
    /// byte-identical. Parallel to `hierarchy_module_dedup` /
    /// `hierarchy_semantic_module_dedup`.
    #[serde(default)]
    pub bisimulation_flop_merge: bool,

    /// Phase 5 (parameterization). Probability that a finalized module
    /// is given a single width `parameter` by the post-construction
    /// `crate::ir::param::parameterize_module` pass. The module body
    /// stays concrete at the chosen design width; the `parameter`
    /// declaration defaults to that design width, so a default
    /// instantiation is byte-identical to the pre-Phase-5 module
    /// (valid by construction). `default = 0.0` keeps every existing
    /// output byte-identical. First slice:
    /// `PHASE-5-PARAMETERIZATION.2.1` — see
    /// `docs/tasks/PHASE-5-PARAMETERIZATION.md` and
    /// `DEVELOPMENT_NOTES.md` "Phase 5 parameterization design".
    #[serde(default = "default_width_parameterization_prob")]
    pub width_parameterization_prob: f64,

    /// Phase 5b (synthesizable aggregates). Probability that a
    /// finalized, non-parameterized module with an eligible
    /// same-direction data-port group is given a packed-aggregate
    /// emitter projection by the post-construction
    /// `crate::ir::aggregate::annotate_aggregate` pass. Purely an
    /// emitter-surface regrouping; the flat IR body, validators, CSE
    /// keys and `canonical_module_signature` are all unaffected.
    /// `default = 0.0` keeps every existing output byte-identical.
    /// First slice: `PHASE-5B-AGGREGATES.2.1` — see
    /// `docs/tasks/PHASE-5B-AGGREGATES.md` and `DEVELOPMENT_NOTES.md`
    /// "Phase 5b packed-aggregate emitter projection design".
    #[serde(default = "default_aggregate_prob")]
    pub aggregate_prob: f64,

    /// AGGREGATE-ARRAY-PACKING. Conditional on a module being
    /// aggregate-projected (the `aggregate_prob` roll fired), the
    /// probability that a **uniform-width** projected group is rendered
    /// as a packed *array* (`typedef logic [N-1:0][W-1:0] …`) instead
    /// of a packed `struct`. Only takes effect when every projected
    /// group is internally same-width; otherwise the layout falls back
    /// to `StructPacked`. A packed array is LRM-bit-equivalent to the
    /// field concatenation, so this is a faithful, semantically-empty
    /// regrouping — the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected. `default = 0.0`
    /// keeps every existing output byte-identical (always
    /// `StructPacked`). See `docs/tasks/AGGREGATE-ARRAY-PACKING.md`.
    #[serde(default = "default_aggregate_array_prob")]
    pub aggregate_array_prob: f64,

    /// `SV-VERSION-TARGETING.3b.2` — the first version-distinctive *up-opt*.
    /// Probability, per *proper low-bits* `Slice` gate
    /// (`GateOp::Slice { hi, lo: 0 }` over a non-constant source narrower
    /// than the source), that the emitter renders it via an internal IEEE
    /// 1800-2023 `union soft` overlay (`u.w = src; gate = u.n`) **iff** the
    /// emission target also permits 2023 (`sv_version >= 2023`); below 2023 a
    /// marked gate down-gates to the plain `src[hi:0]` slice. The overlay is
    /// behaviour-preserving (packed-union members are LSB-aligned, so
    /// `u.n == src[hi:0]`) and genuinely 2023 (heterogeneous-width packed-union
    /// members are legal only as `union soft`, §7.3.1). `default = 0.0` keeps
    /// every existing output byte-identical; the marker is an emitter-surface
    /// annotation only, so the flat IR body / validators / CSE keys /
    /// `canonical_module_signature` are unaffected. Orthogonal to
    /// `--sv-version`: needs *both* `> 0.0` *and* a 2023 target to change
    /// output. See decision `0010` + `docs/tasks/SV-VERSION-TARGETING.md`.
    #[serde(default = "default_soft_union_slice_prob")]
    pub soft_union_slice_prob: f64,

    /// Phase 6 (advanced motifs). Probability that the free-standing
    /// single-module lane builds a rules-first inferrable-memory leaf
    /// (`crate::gen::module::build_memory_leaf`) instead of an
    /// ordinary leaf. The leaf emits the `.1`-validated Yosys-
    /// `$mem_v2` synchronous template. `default = 0.0` keeps every
    /// existing output byte-identical; mutually exclusive with the
    /// Phase 5 width-parameterization lane. First slice:
    /// `PHASE-6-ADVANCED-MOTIFS.2.1b`.
    #[serde(default = "default_memory_prob")]
    pub memory_prob: f64,

    /// Phase 6 (advanced motifs). Probability that the free-standing
    /// single-module lane builds a rules-first generated-encoding FSM
    /// block (`crate::gen::module::build_fsm_block`) instead of an
    /// ordinary leaf. The block emits the `.3.1`-probed-clean Moore
    /// FSM template (encoding-derived `localparam` state constants +
    /// an async-reset state register + `always_comb` next-state /
    /// Moore-output `case`s). `default = 0.0` keeps every existing
    /// output byte-identical; mutually exclusive with the Phase 5
    /// width-parameterization and Phase 6 memory lanes. Slice:
    /// `PHASE-6-ADVANCED-MOTIFS.3.2b`.
    #[serde(default = "default_fsm_prob")]
    pub fsm_prob: f64,

    /// `MULTI-CLOCK-CDC.3b` — per-module roll for the multi-clock
    /// promotion pass. When fired, a second clock domain is added
    /// to the generated module + one flop-driven output is wrapped
    /// in a by-construction synchronizer chain in the new domain.
    /// `default = 0.0` keeps every existing output byte-identical
    /// to pre-`.3b` ANVIL — the load-bearing default-`dut`
    /// book-runnable contract from Phase 9 is preserved.
    #[serde(default = "default_multi_clock_prob")]
    pub multi_clock_prob: f64,

    /// `SIGNOFF-SURFACE-EXPANSION.1` — number of destination-domain
    /// flops in the generated 1-bit CDC synchronizer chain. Default
    /// `2` preserves the existing `MULTI-CLOCK-CDC` behavior
    /// byte-for-byte. Values `>= 3` opt into the N-flop synchronizer
    /// primitive for higher-MTBF CDC stress.
    #[serde(default = "default_cdc_synchronizer_stages")]
    pub cdc_synchronizer_stages: u32,

    // Clocking (Phase 2+)
    pub use_async_reset: bool,

    // How to schedule cone construction across outputs. See
    // `book/src/construction-strategies.md`.
    pub construction_strategy: ConstructionStrategy,

    /// Identity mode — the coarse answer to "what does a NodeId
    /// mean?". Default `node-id` keeps the factorization ladder
    /// live; `relaxed` disables it entirely and forces fresh
    /// NodeIds even when `factorization_level` requests stronger
    /// sharing. Orthogonal to `construction_strategy`.
    #[serde(default)]
    pub identity_mode: IdentityMode,

    /// Target IEEE 1800 SystemVerilog standard for emission
    /// (`SV-VERSION-TARGETING`, decision `0009`). An opt-in capability
    /// gate: **down-gating** guarantees the emitter never emits a
    /// construct newer than the target; **up-opting** (future `.3`) lets a
    /// higher target deliberately emit that standard's distinctive
    /// synthesizable constructs. Default `Sv2012` is the honest floor — the
    /// current emitted subset is 1800-2012-valid, so the default reproduces
    /// today's emission byte-for-byte (`tests/snapshots.rs` untouched) and
    /// down-gating to the floor removes nothing. Orthogonal to
    /// `identity_mode` / `factorization_level`.
    #[serde(default)]
    pub sv_version: SvVersion,

    /// Target number of top-level units (operator gate / flop /
    /// comb-mux block) grown in the pool by the `GraphFirst`
    /// strategy. Only consulted when `construction_strategy ==
    /// GraphFirst`. Does not count the internal primitive gates
    /// generated inside comb-mux assembly or flop-mux assembly.
    pub graph_first_pool_size: u32,

    /// Rate at which an operator gate's operand list may contain
    /// duplicates (same `NodeId` appearing twice or more across the
    /// N slots). Range `[0.0, 1.0]`. Default `0.0` — operand lists
    /// are strictly distinct. `1.0` — duplicates unrestricted.
    ///
    /// Covers `Add` and `Mul` only: duplicates in `And` / `Or` / `Xor`
    /// remain *always forbidden* (they collapse to `x` / `0`
    /// algebraically regardless of the knob), and comparisons / `Sub`
    /// / `Mux` keep their 2-operand degenerate-shape rejection. The
    /// knob is about stylistic freedom for the algebraically-
    /// meaningful dups: `x + x = 2x`, `x * x = x²`. Opt in to exercise
    /// those shapes in downstream tools.
    pub operand_duplication_rate: f64,

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

    /// Factorization level — the rung requested within
    /// `identity_mode == node-id`. Default `e-graph` requests the
    /// strongest semantics the build knows how to provide;
    /// `effective()` maps that request to the strongest implemented
    /// layer at or below it (today the bounded `e-graph` fragment).
    /// Lower settings disable individual layers in order. See
    /// `book/src/structural-rules.md` Rule 21b / 21c.
    ///
    /// Fine-grained knobs (`max_ast_instances`,
    /// `operand_duplication_rate`, `mux_arm_duplication_rate`)
    /// remain in effect at their active level; the factorization
    /// level gates whether a layer contributes at all. When
    /// `identity_mode == relaxed`, the effective level is forced
    /// to `none` regardless of this requested rung.
    pub factorization_level: FactorizationLevel,

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
            // Sentinel 0 = unlimited (byte-identical to the historical
            // unbounded behaviour). Opt in to a real cap via --config.
            // (WORKLOAD-MEMORY-SAFETY.3 — previously 1000 but never
            // enforced; enforcing the old default would have changed
            // output for any module exceeding it.)
            max_nodes_per_module: 0,
            // Sentinel 0 = off on both axes: the internal RAM/RSS
            // governor never samples ⇒ byte-identical default path
            // (WORKLOAD-MEMORY-SAFETY.4).
            max_rss_mb: 0,
            ram_abort_pct: 0,
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
            case_mux_prob: 0.05,
            casez_mux_prob: 0.05,
            for_fold_prob: 0.05,
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
            num_child_instances: 0,
            hierarchy_child_source_mode: HierarchyChildSourceMode::Library,
            min_hierarchy_depth: 0,
            max_hierarchy_depth: 0,
            min_child_instances_per_module: 0,
            max_child_instances_per_module: 0,
            child_instances_per_module_by_depth: BTreeMap::new(),
            hierarchy_sibling_route_prob: 0.35,
            hierarchy_registered_sibling_route_prob:
                default_hierarchy_registered_sibling_route_prob(),
            hierarchy_registered_sibling_mixed_support_prob:
                default_hierarchy_registered_sibling_mixed_support_prob(),
            hierarchy_registered_child_input_cone_prob:
                default_hierarchy_registered_child_input_cone_prob(),
            hierarchy_child_input_cone_prob: default_hierarchy_child_input_cone_prob(),
            hierarchy_parent_cone_instance_prob: default_hierarchy_parent_cone_instance_prob(),
            max_parent_cone_instances_per_module: default_max_parent_cone_instances_per_module(),
            hierarchy_parent_flop_prob: default_hierarchy_parent_flop_prob(),
            hierarchy_module_dedup: false,
            hierarchy_semantic_module_dedup: false,
            bisimulation_flop_merge: false,
            width_parameterization_prob: default_width_parameterization_prob(),
            aggregate_prob: default_aggregate_prob(),
            aggregate_array_prob: default_aggregate_array_prob(),
            soft_union_slice_prob: default_soft_union_slice_prob(),
            memory_prob: default_memory_prob(),
            fsm_prob: default_fsm_prob(),
            multi_clock_prob: default_multi_clock_prob(),
            cdc_synchronizer_stages: default_cdc_synchronizer_stages(),
            use_async_reset: true,
            construction_strategy: ConstructionStrategy::Interleaved,
            identity_mode: IdentityMode::NodeId,
            sv_version: SvVersion::Sv2012,
            graph_first_pool_size: 32,
            mux_arm_duplication_rate: 0.0,
            operand_duplication_rate: 0.0,
            factorization_level: FactorizationLevel::EGraph,
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
    #[error(
        "hierarchy_depth ({0}) is not supported yet; current Phase 4 slice supports only 0 or 1"
    )]
    HierarchyDepthUnsupported(u32),
    #[error(
        "hierarchy depth range is invalid: min={min}, max={max} (use both zero for leaf-only mode, or 1 <= min <= max)"
    )]
    HierarchyDepthRange { min: u32, max: u32 },
    #[error("hierarchy_depth > 0 requires num_leaf_modules >= 1 (got {0})")]
    HierarchyRequiresLeafModules(u32),
    #[error(
        "hierarchy_child_source_mode={0:?} requires hierarchy mode; the knob is ignored in leaf-only mode"
    )]
    HierarchyChildSourceRequiresHierarchy(HierarchyChildSourceMode),
    #[error(
        "num_child_instances ({0}) requires hierarchy_depth > 0; the knob is ignored in leaf-only mode"
    )]
    ChildInstancesRequireHierarchy(u32),
    #[error(
        "hierarchy_child_source_mode=on-demand in legacy depth-1 wrapper mode requires num_child_instances >= 1"
    )]
    OnDemandWrapperRequiresChildInstances,
    #[error(
        "child instance range is invalid: min={min}, max={max} (use both zero to keep legacy exact-child-count mode, or 1 <= min <= max)"
    )]
    ChildInstancesRange { min: u32, max: u32 },
    #[error(
        "child instance range override for parent depth {depth} is invalid: min={min}, max={max} (need 1 <= min <= max)"
    )]
    ChildInstancesRangeForDepth { depth: u32, min: u32, max: u32 },
    #[error(
        "child instance per-depth overrides require a global child instance range fallback via --min-child-instances-per-module / --max-child-instances-per-module"
    )]
    ChildInstancesByDepthRequireGlobalRange,
    #[error(
        "child instance per-depth override at parent depth {depth} is outside the valid realized internal-depth range [0:{max_parent_depth}] for max_hierarchy_depth {max_hierarchy_depth}"
    )]
    ChildInstancesByDepthOutOfRange {
        depth: u32,
        max_parent_depth: u32,
        max_hierarchy_depth: u32,
    },
    #[error(
        "hierarchy_depth exact knob ({exact}) conflicts with hierarchy depth range [{min}:{max}]"
    )]
    HierarchyDepthConflict { exact: u32, min: u32, max: u32 },
    #[error(
        "num_child_instances exact knob ({exact}) conflicts with child instance range [{min}:{max}]"
    )]
    ChildInstancesConflict { exact: u32, min: u32, max: u32 },
    #[error(
        "num_leaf_modules ({0}) is only valid in legacy exact depth-1 wrapper mode; recursive range mode plans child libraries on demand"
    )]
    LeafLibraryRequiresLegacyHierarchy(u32),
    #[error(
        "num_leaf_modules ({0}) is only valid when hierarchy_child_source_mode=library in legacy exact depth-1 wrapper mode"
    )]
    LeafLibraryRequiresLibrarySourcing(u32),
    #[error("cdc_synchronizer_stages ({0}) must be >= 2")]
    CdcSynchronizerStagesTooSmall(u32),
    #[error("ram_abort_pct ({0}) must be in 0..=100 (0 = off)")]
    RamAbortPctRange(u32),
}

impl Config {
    /// Effective factorization level after the coarse identity mode
    /// has been applied.
    pub fn effective_factorization_level(&self) -> FactorizationLevel {
        match self.identity_mode {
            IdentityMode::Relaxed => FactorizationLevel::None,
            IdentityMode::NodeId => self.factorization_level.effective(),
        }
    }

    /// Effective number of child instances in the current Phase 4
    /// wrapper slice. `0` preserves the legacy behavior: instantiate
    /// every generated leaf definition exactly once.
    pub fn effective_num_child_instances(&self) -> u32 {
        if self.num_child_instances == 0 {
            self.num_leaf_modules
        } else {
            self.num_child_instances
        }
    }

    pub fn uses_on_demand_child_sourcing(&self) -> bool {
        self.hierarchy_child_source_mode == HierarchyChildSourceMode::OnDemand
    }

    pub fn uses_hierarchy_range_mode(&self) -> bool {
        self.min_hierarchy_depth > 0
            || self.max_hierarchy_depth > 0
            || self.min_child_instances_per_module > 0
            || self.max_child_instances_per_module > 0
            || !self.child_instances_per_module_by_depth.is_empty()
    }

    pub fn effective_hierarchy_depth_range(&self) -> Option<(u32, u32)> {
        if self.uses_hierarchy_range_mode() {
            Some((self.min_hierarchy_depth, self.max_hierarchy_depth))
        } else if self.hierarchy_depth > 0 {
            Some((self.hierarchy_depth, self.hierarchy_depth))
        } else {
            None
        }
    }

    pub fn effective_child_instance_range(&self) -> Option<(u32, u32)> {
        self.effective_hierarchy_depth_range().map(|_| {
            if self.uses_hierarchy_range_mode() {
                (
                    self.min_child_instances_per_module,
                    self.max_child_instances_per_module,
                )
            } else {
                let exact = self.effective_num_child_instances();
                (exact, exact)
            }
        })
    }

    pub fn effective_child_instance_range_for_parent_depth(
        &self,
        parent_depth: u32,
    ) -> Option<(u32, u32)> {
        if self.uses_hierarchy_range_mode() {
            if let Some(range) = self.child_instances_per_module_by_depth.get(&parent_depth) {
                Some((range.min, range.max))
            } else {
                self.effective_child_instance_range()
            }
        } else {
            self.effective_child_instance_range()
        }
    }

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
        if self.cdc_synchronizer_stages < 2 {
            return Err(ConfigError::CdcSynchronizerStagesTooSmall(
                self.cdc_synchronizer_stages,
            ));
        }
        if self.ram_abort_pct > 100 {
            return Err(ConfigError::RamAbortPctRange(self.ram_abort_pct));
        }
        if self.uses_hierarchy_range_mode() {
            if self.min_hierarchy_depth == 0 || self.max_hierarchy_depth < self.min_hierarchy_depth
            {
                return Err(ConfigError::HierarchyDepthRange {
                    min: self.min_hierarchy_depth,
                    max: self.max_hierarchy_depth,
                });
            }
            if self.min_child_instances_per_module == 0
                || self.max_child_instances_per_module < self.min_child_instances_per_module
            {
                return Err(ConfigError::ChildInstancesRange {
                    min: self.min_child_instances_per_module,
                    max: self.max_child_instances_per_module,
                });
            }
            if !self.child_instances_per_module_by_depth.is_empty()
                && (self.min_child_instances_per_module == 0
                    || self.max_child_instances_per_module == 0)
            {
                return Err(ConfigError::ChildInstancesByDepthRequireGlobalRange);
            }
            let max_parent_depth = self.max_hierarchy_depth - 1;
            for (&depth, range) in &self.child_instances_per_module_by_depth {
                if range.min == 0 || range.max < range.min {
                    return Err(ConfigError::ChildInstancesRangeForDepth {
                        depth,
                        min: range.min,
                        max: range.max,
                    });
                }
                if depth > max_parent_depth {
                    return Err(ConfigError::ChildInstancesByDepthOutOfRange {
                        depth,
                        max_parent_depth,
                        max_hierarchy_depth: self.max_hierarchy_depth,
                    });
                }
            }
            if self.hierarchy_depth > 0 {
                return Err(ConfigError::HierarchyDepthConflict {
                    exact: self.hierarchy_depth,
                    min: self.min_hierarchy_depth,
                    max: self.max_hierarchy_depth,
                });
            }
            if self.num_child_instances > 0 {
                return Err(ConfigError::ChildInstancesConflict {
                    exact: self.num_child_instances,
                    min: self.min_child_instances_per_module,
                    max: self.max_child_instances_per_module,
                });
            }
            if self.num_leaf_modules > 0 {
                return Err(ConfigError::LeafLibraryRequiresLegacyHierarchy(
                    self.num_leaf_modules,
                ));
            }
        } else {
            if self.hierarchy_depth == 0 && self.uses_on_demand_child_sourcing() {
                return Err(ConfigError::HierarchyChildSourceRequiresHierarchy(
                    self.hierarchy_child_source_mode,
                ));
            }
            if self.hierarchy_depth > 1 {
                return Err(ConfigError::HierarchyDepthUnsupported(self.hierarchy_depth));
            }
            if self.hierarchy_depth > 0 {
                if self.uses_on_demand_child_sourcing() {
                    if self.num_leaf_modules > 0 {
                        return Err(ConfigError::LeafLibraryRequiresLibrarySourcing(
                            self.num_leaf_modules,
                        ));
                    }
                    if self.num_child_instances == 0 {
                        return Err(ConfigError::OnDemandWrapperRequiresChildInstances);
                    }
                } else if self.num_leaf_modules < 1 {
                    return Err(ConfigError::HierarchyRequiresLeafModules(
                        self.num_leaf_modules,
                    ));
                }
            }
            if self.hierarchy_depth == 0 && self.num_child_instances > 0 {
                return Err(ConfigError::ChildInstancesRequireHierarchy(
                    self.num_child_instances,
                ));
            }
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
            ("case_mux_prob", self.case_mux_prob),
            ("casez_mux_prob", self.casez_mux_prob),
            ("for_fold_prob", self.for_fold_prob),
            (
                "hierarchy_sibling_route_prob",
                self.hierarchy_sibling_route_prob,
            ),
            (
                "hierarchy_registered_sibling_route_prob",
                self.hierarchy_registered_sibling_route_prob,
            ),
            (
                "hierarchy_registered_sibling_mixed_support_prob",
                self.hierarchy_registered_sibling_mixed_support_prob,
            ),
            (
                "hierarchy_registered_child_input_cone_prob",
                self.hierarchy_registered_child_input_cone_prob,
            ),
            (
                "hierarchy_child_input_cone_prob",
                self.hierarchy_child_input_cone_prob,
            ),
            (
                "hierarchy_parent_cone_instance_prob",
                self.hierarchy_parent_cone_instance_prob,
            ),
            (
                "hierarchy_parent_flop_prob",
                self.hierarchy_parent_flop_prob,
            ),
            ("mux_arm_duplication_rate", self.mux_arm_duplication_rate),
            ("operand_duplication_rate", self.operand_duplication_rate),
            (
                "width_parameterization_prob",
                self.width_parameterization_prob,
            ),
            ("aggregate_prob", self.aggregate_prob),
            ("aggregate_array_prob", self.aggregate_array_prob),
            ("soft_union_slice_prob", self.soft_union_slice_prob),
            ("memory_prob", self.memory_prob),
            ("fsm_prob", self.fsm_prob),
            ("multi_clock_prob", self.multi_clock_prob),
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
        if let Some(v) = o.terminal_reuse_prob {
            self.terminal_reuse_prob = v;
        }
        if let Some(v) = o.constant_prob {
            self.constant_prob = v;
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
        if let Some(v) = o.identity_mode {
            self.identity_mode = v;
        }
        if let Some(v) = o.sv_version {
            self.sv_version = v;
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
        if let Some(v) = o.gate_bitwise_weight {
            self.gate_bitwise_weight = v;
        }
        if let Some(v) = o.gate_arith_weight {
            self.gate_arith_weight = v;
        }
        if let Some(v) = o.gate_struct_weight {
            self.gate_struct_weight = v;
        }
        if let Some(v) = o.gate_compare_weight {
            self.gate_compare_weight = v;
        }
        if let Some(v) = o.gate_reduce_weight {
            self.gate_reduce_weight = v;
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
        if let Some(v) = o.case_mux_prob {
            self.case_mux_prob = v;
        }
        if let Some(v) = o.casez_mux_prob {
            self.casez_mux_prob = v;
        }
        if let Some(v) = o.for_fold_prob {
            self.for_fold_prob = v;
        }
        if let Some(v) = o.max_ast_instances {
            self.max_ast_instances = v;
        }
        if let Some(v) = o.mux_arm_duplication_rate {
            self.mux_arm_duplication_rate = v;
        }
        if let Some(v) = o.operand_duplication_rate {
            self.operand_duplication_rate = v;
        }
        if let Some(v) = o.factorization_level {
            self.factorization_level = v;
        }
        if let Some(v) = o.hierarchy_depth {
            self.hierarchy_depth = v;
        }
        if let Some(v) = o.num_leaf_modules {
            self.num_leaf_modules = v;
        }
        if let Some(v) = o.num_child_instances {
            self.num_child_instances = v;
        }
        if let Some(v) = o.hierarchy_child_source_mode {
            self.hierarchy_child_source_mode = v;
        }
        if let Some(v) = o.min_hierarchy_depth {
            self.min_hierarchy_depth = v;
        }
        if let Some(v) = o.max_hierarchy_depth {
            self.max_hierarchy_depth = v;
        }
        if let Some(v) = o.min_child_instances_per_module {
            self.min_child_instances_per_module = v;
        }
        if let Some(v) = o.max_child_instances_per_module {
            self.max_child_instances_per_module = v;
        }
        if let Some(v) = &o.child_instances_per_module_by_depth {
            self.child_instances_per_module_by_depth = v.clone();
        }
        if let Some(v) = o.hierarchy_sibling_route_prob {
            self.hierarchy_sibling_route_prob = v;
        }
        if let Some(v) = o.hierarchy_registered_sibling_route_prob {
            self.hierarchy_registered_sibling_route_prob = v;
        }
        if let Some(v) = o.hierarchy_registered_sibling_mixed_support_prob {
            self.hierarchy_registered_sibling_mixed_support_prob = v;
        }
        if let Some(v) = o.hierarchy_registered_child_input_cone_prob {
            self.hierarchy_registered_child_input_cone_prob = v;
        }
        if let Some(v) = o.hierarchy_child_input_cone_prob {
            self.hierarchy_child_input_cone_prob = v;
        }
        if let Some(v) = o.hierarchy_parent_cone_instance_prob {
            self.hierarchy_parent_cone_instance_prob = v;
        }
        if let Some(v) = o.max_parent_cone_instances_per_module {
            self.max_parent_cone_instances_per_module = v;
        }
        if let Some(v) = o.hierarchy_parent_flop_prob {
            self.hierarchy_parent_flop_prob = v;
        }
        if let Some(v) = o.max_rss_mb {
            self.max_rss_mb = v;
        }
        if let Some(v) = o.ram_abort_pct {
            self.ram_abort_pct = v;
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
    pub terminal_reuse_prob: Option<f64>,
    pub constant_prob: Option<f64>,
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
    pub identity_mode: Option<IdentityMode>,
    pub sv_version: Option<SvVersion>,
    pub graph_first_pool_size: Option<u32>,
    pub coefficient_prob: Option<f64>,
    pub min_coefficient: Option<u32>,
    pub max_coefficient: Option<u32>,
    pub const_shift_amount_prob: Option<f64>,
    pub gate_bitwise_weight: Option<u32>,
    pub gate_arith_weight: Option<u32>,
    pub gate_struct_weight: Option<u32>,
    pub gate_compare_weight: Option<u32>,
    pub gate_reduce_weight: Option<u32>,
    pub min_shift_amount: Option<u32>,
    pub max_shift_amount: Option<u32>,
    pub gate_shift_weight: Option<u32>,
    pub const_comparand_prob: Option<f64>,
    pub min_comparand: Option<u32>,
    pub max_comparand: Option<u32>,
    pub priority_encoder_prob: Option<f64>,
    pub case_mux_prob: Option<f64>,
    pub casez_mux_prob: Option<f64>,
    pub for_fold_prob: Option<f64>,
    pub max_ast_instances: Option<u32>,
    pub mux_arm_duplication_rate: Option<f64>,
    pub operand_duplication_rate: Option<f64>,
    pub factorization_level: Option<FactorizationLevel>,
    pub hierarchy_depth: Option<u32>,
    pub num_leaf_modules: Option<u32>,
    pub num_child_instances: Option<u32>,
    pub hierarchy_child_source_mode: Option<HierarchyChildSourceMode>,
    pub min_hierarchy_depth: Option<u32>,
    pub max_hierarchy_depth: Option<u32>,
    pub min_child_instances_per_module: Option<u32>,
    pub max_child_instances_per_module: Option<u32>,
    pub child_instances_per_module_by_depth: Option<BTreeMap<u32, CountRange>>,
    pub hierarchy_sibling_route_prob: Option<f64>,
    pub hierarchy_registered_sibling_route_prob: Option<f64>,
    pub hierarchy_registered_sibling_mixed_support_prob: Option<f64>,
    pub hierarchy_registered_child_input_cone_prob: Option<f64>,
    pub hierarchy_child_input_cone_prob: Option<f64>,
    pub hierarchy_parent_cone_instance_prob: Option<f64>,
    pub max_parent_cone_instances_per_module: Option<u32>,
    pub hierarchy_parent_flop_prob: Option<f64>,
    pub max_rss_mb: Option<u64>,
    pub ram_abort_pct: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_duplicate_and_mux_rate_probabilities_outside_unit_interval() {
        let mut cfg = Config {
            mux_arm_duplication_rate: 1.5,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::Probability { name, value }) => {
                assert_eq!(name, "mux_arm_duplication_rate");
                assert_eq!(value, 1.5);
            }
            other => panic!("expected mux_arm_duplication_rate probability error, got {other:?}"),
        }

        cfg = Config {
            operand_duplication_rate: -0.1,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::Probability { name, value }) => {
                assert_eq!(name, "operand_duplication_rate");
                assert_eq!(value, -0.1);
            }
            other => {
                panic!("expected operand_duplication_rate probability error, got {other:?}")
            }
        }
    }

    #[test]
    fn validate_rejects_cdc_synchronizer_stage_counts_below_two() {
        let cfg = Config {
            cdc_synchronizer_stages: 1,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::CdcSynchronizerStagesTooSmall(stages)) => assert_eq!(stages, 1),
            other => panic!("expected cdc stage-count rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_ram_abort_pct_above_100() {
        let cfg = Config {
            ram_abort_pct: 101,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::RamAbortPctRange(pct)) => assert_eq!(pct, 101),
            other => panic!("expected ram_abort_pct range rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_accepts_ram_abort_pct_boundaries_and_off_sentinel() {
        // 0 = off, 100 = full, both valid; the governor knobs default off.
        for pct in [0u32, 1, 88, 100] {
            let cfg = Config {
                ram_abort_pct: pct,
                max_rss_mb: 4096,
                ..Config::default()
            };
            assert!(cfg.validate().is_ok(), "pct {pct} should validate");
        }
        assert_eq!(Config::default().max_rss_mb, 0);
        assert_eq!(Config::default().ram_abort_pct, 0);
    }

    #[test]
    fn effective_factorization_level_respects_identity_mode() {
        let mut cfg = Config {
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Config::default()
        };
        assert_eq!(
            cfg.effective_factorization_level(),
            FactorizationLevel::EGraph
        );

        cfg.identity_mode = IdentityMode::Relaxed;
        assert_eq!(
            cfg.effective_factorization_level(),
            FactorizationLevel::None
        );
    }

    #[test]
    fn sv_version_defaults_to_floor_and_orders_and_permits_correctly() {
        // Default is the honest floor — the byte-identical contract.
        assert_eq!(SvVersion::default(), SvVersion::Sv2012);
        assert_eq!(Config::default().sv_version, SvVersion::Sv2012);

        // Declaration order is the standard order, so `permits` is "target
        // is at least the introducing standard".
        assert!(SvVersion::Sv2012 < SvVersion::Sv2017);
        assert!(SvVersion::Sv2017 < SvVersion::Sv2023);

        // Down-gating bound: a 2012 target permits only 2012 constructs;
        // a 2023 target permits everything; a construct's own standard is
        // always permitted by itself.
        assert!(SvVersion::Sv2012.permits(SvVersion::Sv2012));
        assert!(!SvVersion::Sv2012.permits(SvVersion::Sv2017));
        assert!(!SvVersion::Sv2012.permits(SvVersion::Sv2023));
        assert!(SvVersion::Sv2017.permits(SvVersion::Sv2012));
        assert!(SvVersion::Sv2017.permits(SvVersion::Sv2017));
        assert!(!SvVersion::Sv2017.permits(SvVersion::Sv2023));
        assert!(SvVersion::Sv2023.permits(SvVersion::Sv2012));
        assert!(SvVersion::Sv2023.permits(SvVersion::Sv2023));

        assert_eq!(SvVersion::Sv2012.ieee_standard(), "1800-2012");
        assert_eq!(SvVersion::Sv2017.ieee_standard(), "1800-2017");
        assert_eq!(SvVersion::Sv2023.ieee_standard(), "1800-2023");
    }

    #[test]
    fn sv_version_serde_uses_bare_year_spelling_and_defaults_when_absent() {
        // Bare-year spelling on the wire (dump-config / introspection).
        assert_eq!(
            serde_json::to_string(&SvVersion::Sv2017).unwrap(),
            "\"2017\""
        );
        assert_eq!(
            serde_json::from_str::<SvVersion>("\"2023\"").unwrap(),
            SvVersion::Sv2023
        );

        // Backward-compat (`#[serde(default)]`): an old full config JSON
        // that predates the field — every other key present, `sv_version`
        // absent — still deserializes, falling back to the floor.
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v.as_object_mut().unwrap().remove("sv_version");
        assert!(
            v.get("sv_version").is_none(),
            "sv_version must be absent for this test to be meaningful"
        );
        let cfg: Config = serde_json::from_value(v).unwrap();
        assert_eq!(cfg.sv_version, SvVersion::Sv2012);
    }

    #[test]
    fn validate_rejects_unsupported_hierarchy_depth() {
        let cfg = Config {
            hierarchy_depth: 2,
            num_leaf_modules: 2,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::HierarchyDepthUnsupported(depth)) => assert_eq!(depth, 2),
            other => panic!("expected hierarchy depth rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_zero_leaf_count_when_hierarchy_enabled() {
        let cfg = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 0,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::HierarchyRequiresLeafModules(count)) => assert_eq!(count, 0),
            other => panic!("expected hierarchy leaf-count rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_child_instance_count_without_hierarchy() {
        let cfg = Config {
            hierarchy_depth: 0,
            num_child_instances: 3,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::ChildInstancesRequireHierarchy(count)) => assert_eq!(count, 3),
            other => panic!("expected child-instance-count rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_on_demand_hierarchy_knob_without_hierarchy() {
        let cfg = Config {
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::HierarchyChildSourceRequiresHierarchy(mode)) => {
                assert_eq!(mode, HierarchyChildSourceMode::OnDemand);
            }
            other => panic!("expected hierarchy child-source rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_on_demand_wrapper_without_explicit_child_instances() {
        let cfg = Config {
            hierarchy_depth: 1,
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::OnDemandWrapperRequiresChildInstances) => {}
            other => panic!("expected on-demand wrapper child-count rejection, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_leaf_library_knob_in_on_demand_wrapper_mode() {
        let cfg = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 2,
            num_child_instances: 4,
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::LeafLibraryRequiresLibrarySourcing(count)) => {
                assert_eq!(count, 2);
            }
            other => panic!("expected on-demand leaf-library rejection, got {other:?}"),
        }
    }

    #[test]
    fn effective_num_child_instances_preserves_legacy_zero_as_leaf_count() {
        let cfg = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 4,
            num_child_instances: 0,
            ..Config::default()
        };
        assert_eq!(cfg.effective_num_child_instances(), 4);

        let cfg = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 4,
            num_child_instances: 7,
            ..Config::default()
        };
        assert_eq!(cfg.effective_num_child_instances(), 7);

        let cfg = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 0,
            num_child_instances: 5,
            hierarchy_child_source_mode: HierarchyChildSourceMode::OnDemand,
            ..Config::default()
        };
        assert_eq!(cfg.effective_num_child_instances(), 5);
    }

    #[test]
    fn effective_hierarchy_ranges_support_legacy_exact_and_new_bounded_modes() {
        let legacy = Config {
            hierarchy_depth: 1,
            num_leaf_modules: 4,
            num_child_instances: 7,
            ..Config::default()
        };
        assert_eq!(legacy.effective_hierarchy_depth_range(), Some((1, 1)));
        assert_eq!(legacy.effective_child_instance_range(), Some((7, 7)));

        let ranged = Config {
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 4,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 5,
            child_instances_per_module_by_depth: BTreeMap::from([(
                1,
                CountRange { min: 4, max: 4 },
            )]),
            ..Config::default()
        };
        assert_eq!(ranged.effective_hierarchy_depth_range(), Some((2, 4)));
        assert_eq!(ranged.effective_child_instance_range(), Some((2, 5)));
        assert_eq!(
            ranged.effective_child_instance_range_for_parent_depth(0),
            Some((2, 5))
        );
        assert_eq!(
            ranged.effective_child_instance_range_for_parent_depth(1),
            Some((4, 4))
        );
    }

    #[test]
    fn validate_accepts_bounded_recursive_hierarchy_ranges() {
        let cfg = Config {
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 4,
            child_instances_per_module_by_depth: BTreeMap::from([
                (0, CountRange { min: 4, max: 4 }),
                (1, CountRange { min: 2, max: 2 }),
            ]),
            ..Config::default()
        };
        cfg.validate()
            .expect("bounded recursive hierarchy range should be valid");
    }

    #[test]
    fn validate_rejects_conflicting_exact_and_range_hierarchy_knobs() {
        let cfg = Config {
            hierarchy_depth: 1,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 4,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::HierarchyDepthConflict { exact, min, max }) => {
                assert_eq!(exact, 1);
                assert_eq!(min, 2);
                assert_eq!(max, 3);
            }
            other => panic!("expected hierarchy depth conflict, got {other:?}"),
        }

        let cfg = Config {
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            num_child_instances: 5,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 4,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::ChildInstancesConflict { exact, min, max }) => {
                assert_eq!(exact, 5);
                assert_eq!(min, 2);
                assert_eq!(max, 4);
            }
            other => panic!("expected child instance conflict, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_invalid_hierarchy_ranges() {
        let cfg = Config {
            min_hierarchy_depth: 0,
            max_hierarchy_depth: 2,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 4,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::HierarchyDepthRange { min, max }) => {
                assert_eq!(min, 0);
                assert_eq!(max, 2);
            }
            other => panic!("expected hierarchy depth range rejection, got {other:?}"),
        }

        let cfg = Config {
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 0,
            max_child_instances_per_module: 4,
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::ChildInstancesRange { min, max }) => {
                assert_eq!(min, 0);
                assert_eq!(max, 4);
            }
            other => panic!("expected child instance range rejection, got {other:?}"),
        }

        let cfg = Config {
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
            min_child_instances_per_module: 2,
            max_child_instances_per_module: 4,
            child_instances_per_module_by_depth: BTreeMap::from([(
                3,
                CountRange { min: 2, max: 2 },
            )]),
            ..Config::default()
        };
        match cfg.validate() {
            Err(ConfigError::ChildInstancesByDepthOutOfRange {
                depth,
                max_parent_depth,
                max_hierarchy_depth,
            }) => {
                assert_eq!(depth, 3);
                assert_eq!(max_parent_depth, 2);
                assert_eq!(max_hierarchy_depth, 3);
            }
            other => panic!("expected per-depth range rejection, got {other:?}"),
        }
    }
}
