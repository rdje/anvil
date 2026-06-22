use anvil::config::{
    ConstructionStrategy, CountRange, FactorizationLevel, HierarchyChildSourceMode, IdentityMode,
    SvVersion,
};
use anvil::downstream::{AcceptanceTool, ValidateOptions, YosysMode};
use anvil::umbrella::{ArtifactLane, FrontendLane, MicrodesignLane};
use anvil::{Config, Generator};
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::info;

/// Artifact lane selector (`PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`).
///
/// `dut` (the default) is the L1 DUT RTL lane; the entire historical
/// CLI surface + every book example + every CI gate depend on
/// `--artifact dut` being byte-identical to today. `microdesign`
/// (Phase 7) and `frontend` (Phase 8) are the oracle-backed lanes;
/// each emits its `.sv` to stdout (or to `<out>/<top>.sv`) and its
/// expected-facts JSON manifest to stderr (or to `<out>/<top>.json`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
enum ArtifactKind {
    /// L1 — DUT RTL lane (Phases 1–6). Default. Byte-identical to
    /// the historical no-flag invocation.
    #[default]
    Dut,
    /// L2 — oracle-backed micro-design lane (Phase 7).
    Microdesign,
    /// L3 — source-level frontend / elaboration accept lane
    /// (Phase 8).
    Frontend,
}

/// Trace verbosity (UVM-style). `none` disables tracing entirely.
///
/// Level ordering: `none` < `low` < `medium` < `high` < `debug`.
/// `high` and `debug` both map to `tracing::LevelFilter::TRACE`,
/// but `debug` additionally enables the `trace_verbose!` events
/// (every intern, every op pick, every branch) via the crate's
/// `set_trace_debug` flag. At `high`, those events are suppressed
/// to keep the output readable.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum TraceLevel {
    /// No tracing. `off` accepted as an alias.
    #[value(alias = "off")]
    None,
    Low,
    Medium,
    High,
    Debug,
}

impl TraceLevel {
    fn to_level_filter(self) -> tracing::level_filters::LevelFilter {
        use tracing::level_filters::LevelFilter;
        match self {
            TraceLevel::None => LevelFilter::OFF,
            TraceLevel::Low => LevelFilter::INFO,
            TraceLevel::Medium => LevelFilter::DEBUG,
            TraceLevel::High => LevelFilter::TRACE,
            TraceLevel::Debug => LevelFilter::TRACE,
        }
    }

    /// True at `debug` only — super-verbose `trace_verbose!` events.
    fn debug_verbose(self) -> bool {
        matches!(self, TraceLevel::Debug)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChildInstancesPerDepthArg {
    depth: u32,
    range: CountRange,
}

fn parse_child_instances_per_depth_arg(s: &str) -> Result<ChildInstancesPerDepthArg, String> {
    let (depth_text, range_text) = s
        .split_once('=')
        .ok_or_else(|| "expected DEPTH=MIN:MAX".to_string())?;
    let depth = depth_text
        .parse::<u32>()
        .map_err(|_| format!("invalid depth `{depth_text}`"))?;
    let (min_text, max_text) = range_text
        .split_once(':')
        .ok_or_else(|| "expected DEPTH=MIN:MAX".to_string())?;
    let min = min_text
        .parse::<u32>()
        .map_err(|_| format!("invalid min `{min_text}`"))?;
    let max = max_text
        .parse::<u32>()
        .map_err(|_| format!("invalid max `{max_text}`"))?;
    Ok(ChildInstancesPerDepthArg {
        depth,
        range: CountRange { min, max },
    })
}

/// Parse one `--steer <key>=<weight>` argument into a `(key, weight)` pair
/// (`COVERAGE-STEERED-GENERATION.2c.1`). The `key` is left as a string — whether
/// it is a knob name or a steering category is classified later by
/// `SteeringConfig::set_weight` (in `resolve_config`), which is where the
/// unknown-key error surfaces. Here we only enforce the `key=weight` shape and a
/// parseable weight; the finite/non-negative range check is the config
/// validator's job. Keys contain no `=`, so a plain `split_once('=')` is exact.
fn parse_steer_arg(s: &str) -> Result<(String, f64), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("expected <key>=<weight>, got `{s}`"))?;
    let key = key.trim();
    if key.is_empty() {
        return Err(format!("empty steer key in `{s}`"));
    }
    let weight = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid steer weight `{value}` in `{s}`"))?;
    Ok((key.to_string(), weight))
}

#[derive(Parser, Debug)]
#[command(name = "anvil", version, about = "Random synthesizable RTL generator")]
struct Cli {
    /// Optional subcommand (`BUG-HUNT-ORCHESTRATION.2d`). When omitted
    /// (`None`), the historical flat-flag generate path runs unchanged —
    /// `anvil --seed N …` is byte-identical to every prior invocation, so the
    /// `snapshots` / `book_examples` gates are untouched. The only subcommand is
    /// `hunt` (the turnkey downstream bug-hunt loop).
    #[command(subcommand)]
    command: Option<Commands>,

    /// Artifact lane to generate
    /// (`PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c`). The default `dut`
    /// preserves byte-identical behaviour with every historical
    /// invocation + the entire CI-gated book.
    #[arg(long, value_enum, default_value_t = ArtifactKind::Dut)]
    artifact: ArtifactKind,

    /// Number of parameter/localparam decls for the microdesign /
    /// frontend lanes. Ignored by `--artifact dut`.
    #[arg(long, default_value_t = 5)]
    lane_n_params: usize,

    /// Number of child instances in the frontend lane's top module.
    /// Ignored by `--artifact dut` and `--artifact microdesign`.
    #[arg(long, default_value_t = 2)]
    lane_n_children: usize,

    /// RNG seed (deterministic output in seed + knobs).
    #[arg(long, default_value_t = 0)]
    seed: u64,

    /// Number of modules to generate.
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Output directory. If omitted and count == 1, writes to stdout.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Load knobs from a JSON file; CLI flags override individual fields.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Print effective knobs as JSON and exit.
    #[arg(long)]
    dump_config: bool,

    /// Emit the agent-introspection JSON document
    /// (`AGENT-INTROSPECTION-MCP.3`) to stdout instead of the
    /// SystemVerilog, for a single-artifact run (no `--out`, `--count 1`).
    /// The document is derived strictly from existing metrics/config (see
    /// `docs/AGENT_INTROSPECTION_SCHEMA.md`); default off ⇒ byte-identical.
    #[arg(long)]
    introspect: bool,

    #[arg(long)]
    min_inputs: Option<u32>,
    #[arg(long)]
    max_inputs: Option<u32>,
    #[arg(long)]
    min_outputs: Option<u32>,
    #[arg(long)]
    max_outputs: Option<u32>,
    #[arg(long)]
    min_width: Option<u32>,
    #[arg(long)]
    max_width: Option<u32>,
    #[arg(long)]
    max_depth: Option<u32>,
    /// Per-forced-leaf probability of reusing an existing
    /// matching-width pool signal instead of emitting a fresh
    /// constant. Higher values bias leaf decisions toward sharing.
    #[arg(long)]
    terminal_reuse_prob: Option<f64>,
    /// Per-forced-leaf probability of emitting a fresh constant when
    /// no matching-width signal exists. When this misses, leaf
    /// construction falls back to a width-adapter from an existing
    /// dep-bearing source if one exists.
    #[arg(long)]
    constant_prob: Option<f64>,
    #[arg(long)]
    flop_prob: Option<f64>,
    #[arg(long)]
    share_prob: Option<f64>,
    #[arg(long)]
    max_flops_per_module: Option<u32>,
    #[arg(long)]
    min_mux_arms: Option<u32>,
    #[arg(long)]
    max_mux_arms: Option<u32>,
    #[arg(long)]
    flop_qfeedback_prob: Option<f64>,
    #[arg(long)]
    flop_mux_encoding_prob: Option<f64>,
    #[arg(long)]
    min_gate_arity: Option<u32>,
    #[arg(long)]
    max_gate_arity: Option<u32>,
    #[arg(long)]
    comb_mux_prob: Option<f64>,
    #[arg(long)]
    comb_mux_encoding_prob: Option<f64>,
    /// Construction strategy: sequential, shuffled, interleaved
    /// (default), or graph-first. `graph-first` is a deprecated alias
    /// for `interleaved`. See `book/src/construction-strategies.md`.
    #[arg(long, value_enum)]
    construction_strategy: Option<ConstructionStrategy>,
    /// Legacy knob retained for backward-compatible configs. The
    /// retired speculative `graph-first` builder used this as its
    /// pool-growth target; the current live interleaved/default path
    /// ignores it.
    #[arg(long)]
    graph_first_pool_size: Option<u32>,
    /// Per-op probability (when build_cone picks Add / Sub / Mul) of
    /// emitting the linear-combination compound motif instead of a
    /// standard operator. See `book/src/structural-rules.md` "Roles of
    /// constants in RTL".
    #[arg(long)]
    coefficient_prob: Option<f64>,
    /// Minimum coefficient value for the linear-combination motif.
    #[arg(long)]
    min_coefficient: Option<u32>,
    /// Maximum coefficient value for the linear-combination motif.
    #[arg(long)]
    max_coefficient: Option<u32>,
    /// Relative weight for bitwise ops (And/Or/Xor/Not) in `pick_gate`.
    #[arg(long)]
    gate_bitwise_weight: Option<u32>,
    /// Relative weight for arithmetic ops (Add/Sub/Mul) in `pick_gate`.
    #[arg(long)]
    gate_arith_weight: Option<u32>,
    /// Relative weight for structural ops (Mux) in `pick_gate`.
    #[arg(long)]
    gate_struct_weight: Option<u32>,
    /// Relative weight for comparison ops (Eq/Neq/Lt/Gt/Le/Ge) in
    /// `pick_gate` when the target width is 1.
    #[arg(long)]
    gate_compare_weight: Option<u32>,
    /// Relative weight for reduction ops (RedAnd/RedOr/RedXor) in
    /// `pick_gate` when the target width is 1.
    #[arg(long)]
    gate_reduce_weight: Option<u32>,
    /// Per-shift probability that the shift amount is a constant
    /// literal instead of a recursively-generated signal (barrel
    /// shifter). Real designs bias heavily toward constant.
    #[arg(long)]
    const_shift_amount_prob: Option<f64>,
    /// Minimum constant shift amount.
    #[arg(long)]
    min_shift_amount: Option<u32>,
    /// Maximum constant shift amount. Clamped to `W-1` for a W-bit value.
    #[arg(long)]
    max_shift_amount: Option<u32>,
    /// Relative weight for Shl/Shr in pick_gate.
    #[arg(long)]
    gate_shift_weight: Option<u32>,
    /// Per-comparison probability that the RHS is a constant
    /// comparand instead of a recursive signal cone. Additive to
    /// signal-vs-signal comparisons.
    #[arg(long)]
    const_comparand_prob: Option<f64>,
    /// Minimum constant comparand value.
    #[arg(long)]
    min_comparand: Option<u32>,
    /// Maximum constant comparand value (clamped to 2^K - 1 for the
    /// chosen internal operand width K).
    #[arg(long)]
    max_comparand: Option<u32>,
    /// Per-emission probability of a priority-encoder block at a
    /// compatible target width.
    #[arg(long)]
    priority_encoder_prob: Option<f64>,
    /// Per-emission probability of a combinational `always_comb case`
    /// block. The block uses one encoded select bus and an explicit
    /// default-to-zero arm.
    #[arg(long)]
    case_mux_prob: Option<f64>,
    /// Per-emission probability of a combinational `always_comb casez`
    /// block. The block uses wildcard patterns plus an explicit
    /// default-to-zero arm.
    #[arg(long)]
    casez_mux_prob: Option<f64>,
    /// Per-emission probability of a combinational statically bounded
    /// `always_comb for`-fold block over packed chunks.
    #[arg(long)]
    for_fold_prob: Option<f64>,

    /// Legacy exact hierarchy depth. `0` keeps the Phase 1/2/3
    /// leaf-module path. `1` enables the legacy exact depth-1 wrapper
    /// slice. New bounded recursive hierarchy should use
    /// `--min-hierarchy-depth` / `--max-hierarchy-depth` instead.
    #[arg(long)]
    hierarchy_depth: Option<u32>,

    /// Minimum hierarchy depth for bounded recursive hierarchy mode.
    /// Must be paired with `--max-hierarchy-depth`.
    #[arg(long)]
    min_hierarchy_depth: Option<u32>,

    /// Maximum hierarchy depth for bounded recursive hierarchy mode.
    /// Must be paired with `--min-hierarchy-depth`.
    #[arg(long)]
    max_hierarchy_depth: Option<u32>,

    /// Number of leaf modules in the pre-generated library when
    /// `--hierarchy-depth 1` is enabled.
    #[arg(long)]
    num_leaf_modules: Option<u32>,

    /// Number of child instances the current Phase 4 top wrapper
    /// should instantiate. `0` preserves the legacy wrapper behavior:
    /// instantiate every generated leaf definition exactly once.
    #[arg(long)]
    num_child_instances: Option<u32>,

    /// How Phase 4 parents source child module definitions: from a
    /// reusable library or as fresh per-instance modules.
    #[arg(long, value_enum)]
    hierarchy_child_source_mode: Option<HierarchyChildSourceMode>,

    /// Minimum child-instance count for each non-leaf module in
    /// bounded recursive hierarchy mode. Must be paired with
    /// `--max-child-instances-per-module`.
    #[arg(long)]
    min_child_instances_per_module: Option<u32>,

    /// Maximum child-instance count for each non-leaf module in
    /// bounded recursive hierarchy mode. Must be paired with
    /// `--min-child-instances-per-module`.
    #[arg(long)]
    max_child_instances_per_module: Option<u32>,

    /// Override the child-instance range at a specific parent depth in
    /// bounded recursive hierarchy mode. Repeat this flag as needed.
    /// Depth `0` is the top module, depth `1` its direct children, and
    /// so on. Format: `DEPTH=MIN:MAX`.
    #[arg(long, value_parser = parse_child_instances_per_depth_arg)]
    child_instances_per_depth: Vec<ChildInstancesPerDepthArg>,

    /// Probability that a parent binds a child data input from a
    /// previously-instantiated sibling output when one is available.
    /// The resulting parent-side sibling routing is always acyclic:
    /// only earlier child outputs may feed later child inputs.
    #[arg(long)]
    hierarchy_sibling_route_prob: Option<f64>,

    /// Probability that a parent binds a later child data input through
    /// a local parent flop whose D input is driven by an earlier
    /// sibling output.
    #[arg(long)]
    hierarchy_registered_sibling_route_prob: Option<f64>,

    /// Probability that a direct registered sibling route mixes parent
    /// data-port support into the flop D side before driving the later
    /// child input.
    #[arg(long)]
    hierarchy_registered_sibling_mixed_support_prob: Option<f64>,

    /// Probability that a parent binds a later child data input through
    /// parent-local combinational logic over already-available parent
    /// sources, then one local parent flop. The logic can mix parent
    /// data inputs with earlier sibling outputs and can chain through
    /// earlier parent flops when those are live.
    #[arg(long)]
    hierarchy_registered_child_input_cone_prob: Option<f64>,

    /// Probability that a parent binds a child data input through a
    /// local combinational cone over already-available parent sources
    /// (parent data inputs, earlier sibling outputs, and earlier
    /// parent-side route gates).
    #[arg(long)]
    hierarchy_child_input_cone_prob: Option<f64>,

    /// Probability that a parent-composed child-input cone or
    /// parent-output cone instantiates an extra child module as an
    /// internal parent-cone source.
    #[arg(long)]
    hierarchy_parent_cone_instance_prob: Option<f64>,

    /// Maximum number of parent-cone helper child instances one hierarchy
    /// parent may instantiate. Default 1 preserves the first helper
    /// slice; 0 disables helper insertion regardless of probability.
    #[arg(long)]
    max_parent_cone_instances_per_module: Option<u32>,

    /// Probability that parent-side hierarchy cones may emit local
    /// parent flops. Applies to parent output cones and
    /// parent-composed child-input cones.
    #[arg(long)]
    hierarchy_parent_flop_prob: Option<f64>,

    /// Maximum number of times a given AST (gate expression / constant)
    /// may be materialised as a named node in one module. Default 1 =
    /// strict uniqueness (CSE). Higher N permits N copies; `u32::MAX`
    /// effectively disables deduplication. See
    /// `book/src/structural-rules.md`.
    #[arg(long)]
    max_ast_instances: Option<u32>,

    /// Probability that arms of an N-to-1 mux are permitted to share
    /// the same data signal. `0.0` (default) = every arm distinct;
    /// `1.0` = no constraint.
    #[arg(long)]
    mux_arm_duplication_rate: Option<f64>,

    /// Probability that an operator gate's operand list may contain
    /// the same NodeId twice. `0.0` (default) = strict operand
    /// uniqueness for Add/Mul (And/Or/Xor are always strict
    /// regardless). `1.0` = duplicates unrestricted. Opt in when you
    /// want to exercise `x + x = 2x` / `x * x = x^2` shapes in
    /// downstream tools.
    #[arg(long)]
    operand_duplication_rate: Option<f64>,

    /// Factorization level along the sharing/dedup chain.
    /// Values: `none` / `cse` / `operand-unique` / `commutative` /
    /// `associative` / `constant-fold` / `peephole` / `e-graph`
    /// (default request). `e-graph` is the theoretical ceiling;
    /// `effective()` clamps it to the highest implemented layer
    /// today. See `book/src/structural-rules.md` Rule 21c.
    #[arg(long, value_enum)]
    factorization_level: Option<FactorizationLevel>,
    /// Coarse identity mode: `node-id` (default) means NodeId is
    /// expression identity and keeps the factorization ladder live;
    /// `relaxed` disables the ladder entirely and allocates fresh
    /// NodeIds for every AST.
    #[arg(long, value_enum)]
    identity_mode: Option<IdentityMode>,
    /// Target IEEE 1800 SystemVerilog standard (`2012` / `2017` /
    /// `2023`). Default `2012` is the honest floor: ANVIL's current
    /// emitted subset is 1800-2012-valid, so the default reproduces
    /// today's output byte-for-byte. Down-gating is a guarantee (never
    /// emit a construct newer than the target); up-opting newer
    /// standards' distinctive constructs lands in a later slice
    /// (`SV-VERSION-TARGETING.3`). See `book/src/knobs.md`.
    #[arg(long, value_enum)]
    sv_version: Option<SvVersion>,
    /// Convenience alias for `--identity-mode node-id
    /// --factorization-level e-graph`: request the strongest
    /// currently-available identity/factorization mode.
    #[arg(
        long,
        conflicts_with_all = ["no_full_factorization", "identity_mode", "factorization_level"],
        action = clap::ArgAction::SetTrue
    )]
    full_factorization: bool,
    /// Convenience alias for `--identity-mode relaxed
    /// --factorization-level none`: disable the factorization ladder
    /// and allocate fresh NodeIds for every AST.
    #[arg(
        long,
        conflicts_with_all = ["full_factorization", "identity_mode", "factorization_level"],
        action = clap::ArgAction::SetTrue
    )]
    no_full_factorization: bool,

    /// Trace verbosity: `none` / `low` / `medium` / `high` / `debug`.
    /// Output goes to stderr (or `--trace-file`). `none` (default)
    /// compiles to near-zero overhead. `debug` adds super-verbose
    /// per-intern / per-branch events beyond what `high` shows.
    #[arg(long, value_enum, default_value_t = TraceLevel::None)]
    trace: TraceLevel,

    /// Route trace output to a file instead of stderr.
    #[arg(long)]
    trace_file: Option<PathBuf>,

    /// Print per-module metrics (JSON) to stderr in addition to
    /// writing them into manifest.json for multi-file runs. Always
    /// recorded in the manifest; this flag only affects stderr
    /// visibility.
    #[arg(long)]
    metrics: bool,

    /// Opt-in memory governor (`WORKLOAD-MEMORY-SAFETY.4`): abort an
    /// `--out` run once this process's resident set (RSS) reaches this
    /// many MiB. `0` (default) = off / byte-identical. Sampled between
    /// modules/designs; aborts cleanly with exit code 99 and a stderr
    /// message naming the seed + effective knobs, before the host
    /// danger zone. Complements `scripts/ram_guard.sh` from the inside.
    #[arg(long)]
    max_rss_mb: Option<u64>,

    /// Opt-in memory governor (`WORKLOAD-MEMORY-SAFETY.4`): abort an
    /// `--out` run once host used RAM reaches this percentage
    /// (`1..=100`). `0` (default) = off. Mirrors `scripts/ram_guard.sh`
    /// (macOS `memory_pressure` / Linux `/proc/meminfo`).
    #[arg(long)]
    ram_abort_pct: Option<u32>,

    // KNOB-ERGONOMICS-AND-PRESETS.2b.1 — a curated `--profile` preset plus the
    // 16 previously-config-file-only knobs promoted to CLI flags (decision
    // `0021`). Explicit flags here override a `--profile` preset; not passing one
    // leaves the preset / `--config` / default value intact (all `Option`, or a
    // `SetTrue` bool mapped to `Some(true)` only when present).
    /// Apply a curated knob preset before explicit flags
    /// (`arithmetic-heavy` / `deep-hierarchy` / `structured-emission-max` /
    /// `sv2023-upopts`). Explicit flags override the preset. See
    /// `book/src/knobs.md` and the `anvil://catalog/presets` MCP resource.
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
    /// Bias construction-time coverage steering: repeatable
    /// `--steer <key>=<weight>` where key is a knob name (e.g. `flop_prob`) or a
    /// steering category (`state`/`selectors`/`datapath`/`terminals`/`sharing`/
    /// `hierarchy`) and weight is a non-negative multiplier (`>1` emphasizes,
    /// `<1` de-emphasizes, `1` neutral). Layers on top of `--config`/`--profile`
    /// (explicit wins per key); default none ⇒ DUT byte-identical. The ergonomic
    /// shim over `Config.steering` (`COVERAGE-STEERED-GENERATION.2c.1`,
    /// decision `0023`).
    #[arg(long = "steer", value_name = "KEY=WEIGHT", value_parser = parse_steer_arg)]
    steer: Vec<(String, f64)>,
    /// Per-qualifying-gate probability of the `function automatic` emit-projection.
    #[arg(long)]
    function_emit_prob: Option<f64>,
    /// Per-qualifying-replication probability of the `generate for` emit-projection.
    #[arg(long)]
    generate_loop_emit_prob: Option<f64>,
    /// Per-qualifying-gate probability of the `task automatic` emit-projection.
    #[arg(long)]
    task_emit_prob: Option<f64>,
    /// Per-qualifying-cone probability of the whole-cone `function automatic` emit-projection.
    #[arg(long)]
    cone_function_emit_prob: Option<f64>,
    /// Per-leader probability of the multi-output `task automatic` emit-projection (a co-supported gate pair).
    #[arg(long)]
    multi_output_task_emit_prob: Option<f64>,
    /// Per-qualifying-mux probability of the procedural `always_comb` if/else emit-projection.
    #[arg(long)]
    mux_if_emit_prob: Option<f64>,
    /// Per-qualifying-CaseMux probability of the procedural `always_comb` if/else-if priority-chain emit-projection.
    #[arg(long)]
    case_mux_if_emit_prob: Option<f64>,
    /// Per-qualifying-CasezMux probability of the procedural `always_comb` if/else-if MASKED priority-chain emit-projection.
    #[arg(long)]
    casez_mux_if_emit_prob: Option<f64>,
    /// Per-low-bits-slice probability of the IEEE 1800-2023 `union soft` up-opt (needs `--sv-version 2023`).
    #[arg(long)]
    soft_union_slice_prob: Option<f64>,
    /// Per-module probability of width parameterization (Phase 5).
    #[arg(long)]
    width_parameterization_prob: Option<f64>,
    /// Per-module probability of packed-struct aggregate emission (Phase 5b).
    #[arg(long)]
    aggregate_prob: Option<f64>,
    /// Per-module probability of packed-array aggregate emission.
    #[arg(long)]
    aggregate_array_prob: Option<f64>,
    /// Per-module probability of an inferrable memory block (Phase 6).
    #[arg(long)]
    memory_prob: Option<f64>,
    /// Per-module probability of a generated-encoding FSM block (Phase 6).
    #[arg(long)]
    fsm_prob: Option<f64>,
    /// Given a generated FSM, probability its output is Mealy (decode over
    /// the current state and input) rather than Moore (CAPABILITY-BREADTH-EXPANSION.2b).
    #[arg(long)]
    fsm_mealy_prob: Option<f64>,
    /// Per-module probability of multi-clock CDC promotion.
    #[arg(long)]
    multi_clock_prob: Option<f64>,
    /// Destination-domain flop count in a generated CDC synchronizer chain (>= 2).
    #[arg(long)]
    cdc_synchronizer_stages: Option<u32>,
    /// Enable the opt-in structural hierarchy module-dedup pass.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    hierarchy_module_dedup: bool,
    /// Enable the opt-in bounded-semantic hierarchy module-dedup pass.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    hierarchy_semantic_module_dedup: bool,
    /// Enable the opt-in bounded-sequential whole-module dedup pass.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    hierarchy_sequential_module_dedup: bool,
    /// Enable the opt-in bounded bisimulation flop-merge pass.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    bisimulation_flop_merge: bool,
}

/// ANVIL subcommands (`BUG-HUNT-ORCHESTRATION.2d`). ANVIL is flat-flag by
/// default; a subcommand is opt-in and never perturbs the default generate path.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Turnkey downstream bug-hunt: fuzz a deterministic seed sweep, run the
    /// vetted tools on each artifact, detect any reject/warning (and, with
    /// `--diff-sim`, a cross-simulator trace mismatch) on legal-by-construction
    /// RTL, auto-minimize each failure, and print a JSON `HuntReport`. A thin
    /// shim over the same `hunt::run` the MCP `hunt` tool uses (decision `0017`);
    /// `--out DIR` additionally drops a self-contained reproducer bundle per
    /// finding.
    Hunt(HuntCommand),
}

/// The `anvil hunt` arguments — the CLI projection of `hunt::HuntRequest`.
#[derive(Parser, Debug)]
struct HuntCommand {
    /// Base seed of the sweep (it fuzzes `seed .. seed + seeds`).
    #[arg(long, default_value_t = 0)]
    seed: u64,

    /// Number of consecutive seeds to fuzz.
    #[arg(long, default_value_t = 16, value_parser = clap::value_parser!(u32).range(1..))]
    seeds: u32,

    /// Knob profile: a full `Config` JSON (as emitted by `anvil --dump-config`).
    /// Omit for defaults. The sweep stamps each seed into it.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Vetted downstream tools to run (repeat the flag or comma-separate;
    /// default `verilator,yosys`). A fixed allow-list — no arbitrary binaries.
    #[arg(long, value_enum, value_delimiter = ',')]
    tools: Vec<AcceptanceTool>,

    /// Yosys synthesis mode when `yosys` is selected.
    #[arg(long, value_enum, default_value_t = YosysMode::WithoutAbc)]
    yosys_mode: YosysMode,

    /// Do not auto-minimize failures (minimize is on by default).
    #[arg(long)]
    no_minimize: bool,

    /// Per-failure ceiling on minimize oracle (`validate`) evaluations.
    #[arg(long, default_value_t = 200, value_parser = clap::value_parser!(u32).range(1..))]
    budget: u32,

    /// Also run the cross-simulator agreement check (iverilog vs verilator) on
    /// each downstream-clean artifact; a post-reset trace mismatch is a finding.
    #[arg(long)]
    diff_sim: bool,

    /// Also classify acceptance divergence on each finding: when the selected
    /// tools disagree on legality (one accepts while another warns/rejects), the
    /// finding is reported as `detection=acceptance_divergence` with the
    /// per-tool verdicts. Not minimized (the disagreement can't survive shrinking).
    #[arg(long)]
    divergence: bool,

    /// Write a self-contained reproducer bundle directory per finding under DIR
    /// (the human-CLI convenience). Omit to report findings without on-disk
    /// bundles — the MCP `hunt` tool always omits it and serves artifacts from
    /// its cache instead.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_tracing(&cli)?;

    // BUG-HUNT-ORCHESTRATION.2d — subcommand dispatch (ANVIL's first subcommand).
    // When no subcommand is given (`cli.command == None`), the historical
    // flat-flag generate path below runs entirely unchanged ⇒ byte-identical
    // default (`snapshots` / `book_examples` untouched).
    if let Some(Commands::Hunt(hunt)) = &cli.command {
        return run_hunt_command(hunt);
    }

    // PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c — lane dispatch.
    //
    // `--artifact dut` (the default) falls through to the historical
    // DUT lane path below, BYTE-IDENTICAL to today's behaviour
    // (`BOOK-EXAMPLES-RUNNABLE` + every CI gate depend on this).
    // `--artifact microdesign` and `--artifact frontend` short-circuit
    // here into the umbrella's trait-dispatched path.
    if cli.artifact != ArtifactKind::Dut {
        return run_non_dut_lane(&cli);
    }

    let base = if let Some(path) = &cli.config {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str::<Config>(&text)?
    } else {
        Config::default()
    };
    // KNOB-ERGONOMICS-AND-PRESETS.2b.1 — one shared resolver (decision `0021`):
    // base (default | --config) -> --profile preset -> explicit flags -> seed,
    // then validate. With no `--profile` and no promoted flags this is exactly
    // the historical default|--config -> apply_cli_overrides -> seed path, so the
    // default DUT output stays byte-identical (`tests/snapshots.rs` untouched).
    let cfg =
        anvil::config::resolve_config(base, cli.profile.as_deref(), &cli_overrides(&cli), cli.seed)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

    if cli.dump_config {
        println!("{}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
    }

    // AGENT-INTROSPECTION-MCP.3 — the introspection surface is a
    // single-artifact stdout view; reject it for multi-artifact / --out runs
    // so the contract stays unambiguous (and the default --out path stays
    // byte-identical, never reaching this surface).
    if cli.introspect && (cli.out.is_some() || cli.count != 1) {
        anyhow::bail!(
            "--introspect requires a single-artifact stdout run (omit --out and use --count 1)"
        );
    }

    info!(seed = cli.seed, count = cli.count, "🚀 anvil start");
    let mut gen = Generator::new(cfg.clone());
    let hierarchical = cfg.effective_hierarchy_depth_range().is_some();

    match (&cli.out, cli.count) {
        (None, 1) => {
            if hierarchical {
                let design = gen.generate_design();
                anvil::ir::validate::validate_design(&design)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if cli.introspect {
                    let doc = anvil::introspect::design_document(cli.seed, &cfg, &design);
                    println!("{}", doc.to_json_pretty()?);
                } else {
                    let design_metrics = anvil::metrics::compute_design(&design);
                    print!(
                        "{}",
                        anvil::emit::to_sv_design_versioned(&design, cfg.sv_version)
                    );
                    if cli.metrics {
                        eprintln!("{}", serde_json::to_string_pretty(&design_metrics)?);
                        for module in &design.modules {
                            let metrics = anvil::metrics::compute(module);
                            eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                        }
                    }
                }
            } else {
                let m = gen.generate_module();
                if cli.introspect {
                    let doc = anvil::introspect::module_document(cli.seed, &cfg, &m);
                    println!("{}", doc.to_json_pretty()?);
                } else {
                    let metrics = anvil::metrics::compute(&m);
                    print!("{}", anvil::emit::to_sv_versioned(&m, cfg.sv_version));
                    if cli.metrics {
                        eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                    }
                }
            }
        }
        (Some(dir), n) => {
            std::fs::create_dir_all(dir)?;
            // Stream the manifest array element-by-element so peak
            // metadata memory stays O(1) in `--count` instead of
            // O(`--count`) (`WORKLOAD-MEMORY-SAFETY.2`). The `.sv` files
            // were already streamed (generate → emit → write → drop);
            // the leak was the accumulate-then-`to_string_pretty`
            // metadata `Vec`. Output is byte-identical — proven by
            // `anvil::manifest`'s `streamed_matches_reference` test.
            let seed = cli.seed;
            let metrics_to_stderr = cli.metrics;
            // WORKLOAD-MEMORY-SAFETY.4 — opt-in internal RAM/RSS governor.
            // Disabled by default (`check()` short-circuits to `None`
            // before any OS read), so the default `--out` loop is
            // byte-identical and consumes RNG identically. When armed it
            // is sampled BETWEEN units (decline-to-start-more), never
            // mid-cone — it stops the run cleanly rather than mutilating
            // a built module.
            let guard = anvil::mem_guard::MemGuard::from_config(&cfg);
            let mut scalars = serde_json::Map::new();
            scalars.insert("seed".to_string(), serde_json::json!(seed));
            scalars.insert("config".to_string(), serde_json::to_value(&cfg)?);
            let manifest_file =
                std::io::BufWriter::new(std::fs::File::create(dir.join("manifest.json"))?);
            let write_result: std::io::Result<()> = if hierarchical {
                let mut design_index = 0usize;
                anvil::manifest::write_streamed_manifest(
                    manifest_file,
                    &scalars,
                    "designs",
                    std::iter::from_fn(|| {
                        if design_index >= n {
                            return None;
                        }
                        // Governor checkpoint: abort before starting the
                        // next design if a prior one ballooned us past the
                        // ceiling (WORKLOAD-MEMORY-SAFETY.4).
                        if let Some(reason) = guard.check() {
                            return Some(Err(std::io::Error::new(
                                std::io::ErrorKind::OutOfMemory,
                                anvil::mem_guard::abort_message(&reason, seed, &cfg),
                            )));
                        }
                        let idx = design_index;
                        design_index += 1;
                        Some((|| -> std::io::Result<serde_json::Value> {
                            let design = gen.generate_design();
                            anvil::ir::validate::validate_design(&design)
                                .map_err(|e| std::io::Error::other(e.to_string()))?;
                            let design_metrics = anvil::metrics::compute_design(&design);
                            let mut modules = Vec::new();
                            for module in &design.modules {
                                let metrics = anvil::metrics::compute(module);
                                let fname = format!("{}.sv", module.name);
                                std::fs::write(
                                    dir.join(&fname),
                                    anvil::emit::to_sv_in_design_versioned(
                                        module,
                                        &design,
                                        cfg.sv_version,
                                    ),
                                )?;
                                modules.push(serde_json::json!({
                                    "file": fname,
                                    "name": module.name,
                                    "metrics": metrics,
                                }));
                                if metrics_to_stderr {
                                    if let Ok(s) = serde_json::to_string_pretty(&metrics) {
                                        eprintln!("{s}");
                                    }
                                }
                            }
                            if metrics_to_stderr {
                                if let Ok(s) = serde_json::to_string_pretty(&design_metrics) {
                                    eprintln!("{s}");
                                }
                            }
                            Ok(serde_json::json!({
                                "index": idx,
                                "top": design.top,
                                "metrics": design_metrics,
                                "modules": modules,
                            }))
                        })())
                    }),
                )
            } else {
                let mut i = 0usize;
                anvil::manifest::write_streamed_manifest(
                    manifest_file,
                    &scalars,
                    "modules",
                    std::iter::from_fn(|| {
                        if i >= n {
                            return None;
                        }
                        // Governor checkpoint: abort before starting the
                        // next module if a prior one ballooned us past the
                        // ceiling (WORKLOAD-MEMORY-SAFETY.4).
                        if let Some(reason) = guard.check() {
                            return Some(Err(std::io::Error::new(
                                std::io::ErrorKind::OutOfMemory,
                                anvil::mem_guard::abort_message(&reason, seed, &cfg),
                            )));
                        }
                        let idx = i;
                        i += 1;
                        Some((|| -> std::io::Result<serde_json::Value> {
                            let m = gen.generate_module();
                            let metrics = anvil::metrics::compute(&m);
                            let fname = format!("mod_{}_{:04}.sv", seed, idx);
                            std::fs::write(
                                dir.join(&fname),
                                anvil::emit::to_sv_versioned(&m, cfg.sv_version),
                            )?;
                            if metrics_to_stderr {
                                if let Ok(s) = serde_json::to_string_pretty(&metrics) {
                                    eprintln!("{s}");
                                }
                            }
                            Ok(serde_json::json!({
                                "file": fname,
                                "name": m.name,
                                "metrics": metrics,
                            }))
                        })())
                    }),
                )
            };
            if let Err(e) = write_result {
                if e.kind() == std::io::ErrorKind::OutOfMemory {
                    // Clean governor abort (WORKLOAD-MEMORY-SAFETY.4):
                    // deterministic non-zero exit code 99 (matching
                    // scripts/ram_guard.sh's convention) plus the
                    // seed + effective-knobs message on stderr.
                    eprintln!("{e}");
                    std::process::exit(99);
                }
                return Err(e.into());
            }
        }
        (None, _) => {
            anyhow::bail!("--out is required when --count > 1");
        }
    }

    info!("✅ anvil done");
    Ok(())
}

/// Build a [`HuntRequest`](anvil::hunt::HuntRequest) from the `anvil hunt`
/// arguments — the CLI projection. Factored out of [`run_hunt_command`] so the
/// arg → request mapping is unit-testable without running any tool: the knob
/// profile comes from `--config` (else defaults) with `--seed` stamped in, the
/// validate sandbox stays at the OS-temp default (the same caller-set rule as
/// the MCP path), `--out` becomes the on-disk `bundle_root` (the human-CLI
/// convenience), and an empty `--tools` falls back to the `verilator` + `yosys`
/// default.
fn build_hunt_request(args: &HuntCommand) -> anyhow::Result<anvil::hunt::HuntRequest> {
    let mut cfg = match &args.config {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("read config {}", path.display()))?;
            serde_json::from_str::<Config>(&text)
                .with_context(|| format!("parse config {}", path.display()))?
        }
        None => Config::default(),
    };
    cfg.seed = args.seed;
    cfg.validate().map_err(|e| anyhow::anyhow!("{e}"))?;

    let tools = if args.tools.is_empty() {
        ValidateOptions::default().tools
    } else {
        args.tools.clone()
    };

    Ok(anvil::hunt::HuntRequest {
        base_seed: args.seed,
        seeds: args.seeds,
        config: cfg,
        validate: ValidateOptions {
            tools,
            yosys_mode: args.yosys_mode,
            ..ValidateOptions::default()
        },
        minimize: !args.no_minimize,
        max_oracle_calls: args.budget,
        diff_sim: args.diff_sim,
        // `ACCEPTANCE-DIVERGENCE-HUNTING.2d` — the acceptance-divergence axis
        // (default-off ⇒ this shim stays byte-identical unless `--divergence`).
        divergence: args.divergence,
        bundle_root: args.out.clone(),
    })
}

/// Run the `anvil hunt` subcommand (`BUG-HUNT-ORCHESTRATION.2d`): a thin shim
/// over [`anvil::hunt::run`]. It builds the request from the CLI args, runs the
/// deterministic sweep, and prints the `HuntReport` as pretty JSON to stdout.
/// The MCP `hunt` tool and this CLI are both shims over the *same* `hunt::run`
/// (decision `0017`: the CLI is never a superset of the API). `--out` directs
/// the on-disk reproducer bundle the MCP path leaves to the agent's resource
/// fetches.
fn run_hunt_command(args: &HuntCommand) -> anyhow::Result<()> {
    let req = build_hunt_request(args)?;
    let report = anvil::hunt::run(&req)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

/// Wire a `tracing` subscriber. Output is deterministic: no timestamps,
/// no thread IDs, no ANSI colours — just `LEVEL module::path message`
/// (plus any structured fields). This keeps trace output diffable
/// across runs with the same `(seed, knobs)`.
/// PHASE-9-MULTI-ARTIFACT-UMBRELLA.2c — dispatch to the
/// microdesign / frontend lanes via the `ArtifactLane` trait.
///
/// Called only when `cli.artifact != Dut`. The DUT path stays
/// byte-identical to today's invocation pattern (the load-bearing
/// `BOOK-EXAMPLES-RUNNABLE` + CI-gate contract).
///
/// Emits the lane's `.sv` to stdout (or to `<out>/<top>.sv` if
/// `--out` is set) and the lane's elaborated-facts JSON manifest to
/// stderr (or to `<out>/<top>.json` if `--out` is set). Lane-scoped
/// knobs come from `--lane-n-params` (default 5) and
/// `--lane-n-children` (default 2; frontend only).
fn run_non_dut_lane(cli: &Cli) -> anyhow::Result<()> {
    let lane: Box<dyn ArtifactLane> = match cli.artifact {
        ArtifactKind::Dut => unreachable!("dispatched only when !Dut"),
        ArtifactKind::Microdesign => Box::new(MicrodesignLane::new(cli.lane_n_params)),
        ArtifactKind::Frontend => {
            Box::new(FrontendLane::new(cli.lane_n_params, cli.lane_n_children))
        }
    };
    info!(
        seed = cli.seed,
        artifact = ?cli.artifact,
        lane = lane.name(),
        "🚀 anvil start (non-DUT lane)"
    );
    let artifact = lane
        .generate(cli.seed)
        .map_err(|e| anyhow::anyhow!("lane {} generate failed: {:?}", lane.name(), e))?;
    match &cli.out {
        None => {
            // SV → stdout (matches the historical DUT default
            // behaviour for `count == 1` and no `--out`).
            print!("{}", artifact.sv);
            // Manifest → stderr so it doesn't contaminate stdout
            // pipelines.
            if let Some(manifest) = &artifact.manifest {
                eprintln!("{}", manifest);
            }
        }
        Some(dir) => {
            std::fs::create_dir_all(dir)?;
            // Derive the artifact's top-level name from the
            // emitted SV's first non-empty non-comment line that
            // starts with `module ` (or `package ` for
            // microdesign's package-first SV). Fallback to
            // `<lane>_<seed>` if the parse fails.
            let top = parse_top_name(&artifact.sv)
                .unwrap_or_else(|| format!("{}_{}", artifact.lane, artifact.seed));
            std::fs::write(dir.join(format!("{top}.sv")), &artifact.sv)?;
            if let Some(manifest) = &artifact.manifest {
                std::fs::write(dir.join(format!("{top}.json")), manifest)?;
            }
        }
    }
    Ok(())
}

/// Best-effort extraction of the "top" name from an emitted lane SV
/// for `<out>/<top>.sv` filenames. Reads the first `module <name>`
/// declaration (skipping leading comments, package headers, etc.).
/// Returns `None` if no `module ` line is found — the caller falls
/// back to a `<lane>_<seed>` filename.
fn parse_top_name(sv: &str) -> Option<String> {
    for line in sv.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            // The name is the next whitespace- or `#`- or `(`- or
            // `;`-delimited token.
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '#' || c == '(' || c == ';')
                .unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn init_tracing(cli: &Cli) -> anyhow::Result<()> {
    use tracing_subscriber::fmt;
    // Enable super-verbose `trace_verbose!` events at --trace debug.
    anvil::set_trace_debug(cli.trace.debug_verbose());
    let filter = cli.trace.to_level_filter();
    let builder = fmt()
        .with_max_level(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(false)
        .without_time();
    if let Some(path) = &cli.trace_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        builder.with_writer(std::sync::Mutex::new(file)).init();
    } else {
        builder.with_writer(std::io::stderr).init();
    }
    Ok(())
}

fn cli_overrides(cli: &Cli) -> anvil::config::Overrides {
    let child_instances_per_module_by_depth =
        (!cli.child_instances_per_depth.is_empty()).then(|| {
            cli.child_instances_per_depth
                .iter()
                .map(|entry| (entry.depth, entry.range))
                .collect::<BTreeMap<_, _>>()
        });
    anvil::config::Overrides {
        min_inputs: cli.min_inputs,
        max_inputs: cli.max_inputs,
        min_outputs: cli.min_outputs,
        max_outputs: cli.max_outputs,
        min_width: cli.min_width,
        max_width: cli.max_width,
        max_depth: cli.max_depth,
        terminal_reuse_prob: cli.terminal_reuse_prob,
        constant_prob: cli.constant_prob,
        flop_prob: cli.flop_prob,
        share_prob: cli.share_prob,
        max_flops_per_module: cli.max_flops_per_module,
        min_mux_arms: cli.min_mux_arms,
        max_mux_arms: cli.max_mux_arms,
        flop_qfeedback_prob: cli.flop_qfeedback_prob,
        flop_mux_encoding_prob: cli.flop_mux_encoding_prob,
        min_gate_arity: cli.min_gate_arity,
        max_gate_arity: cli.max_gate_arity,
        comb_mux_prob: cli.comb_mux_prob,
        comb_mux_encoding_prob: cli.comb_mux_encoding_prob,
        construction_strategy: cli.construction_strategy,
        identity_mode: if cli.no_full_factorization {
            Some(IdentityMode::Relaxed)
        } else if cli.full_factorization {
            Some(IdentityMode::NodeId)
        } else {
            cli.identity_mode
        },
        sv_version: cli.sv_version,
        graph_first_pool_size: cli.graph_first_pool_size,
        coefficient_prob: cli.coefficient_prob,
        min_coefficient: cli.min_coefficient,
        max_coefficient: cli.max_coefficient,
        gate_bitwise_weight: cli.gate_bitwise_weight,
        gate_arith_weight: cli.gate_arith_weight,
        gate_struct_weight: cli.gate_struct_weight,
        gate_compare_weight: cli.gate_compare_weight,
        gate_reduce_weight: cli.gate_reduce_weight,
        const_shift_amount_prob: cli.const_shift_amount_prob,
        min_shift_amount: cli.min_shift_amount,
        max_shift_amount: cli.max_shift_amount,
        gate_shift_weight: cli.gate_shift_weight,
        const_comparand_prob: cli.const_comparand_prob,
        min_comparand: cli.min_comparand,
        max_comparand: cli.max_comparand,
        priority_encoder_prob: cli.priority_encoder_prob,
        case_mux_prob: cli.case_mux_prob,
        casez_mux_prob: cli.casez_mux_prob,
        for_fold_prob: cli.for_fold_prob,
        max_ast_instances: cli.max_ast_instances,
        mux_arm_duplication_rate: cli.mux_arm_duplication_rate,
        operand_duplication_rate: cli.operand_duplication_rate,
        factorization_level: if cli.no_full_factorization {
            Some(FactorizationLevel::None)
        } else if cli.full_factorization {
            Some(FactorizationLevel::EGraph)
        } else {
            cli.factorization_level
        },
        hierarchy_depth: cli.hierarchy_depth,
        num_leaf_modules: cli.num_leaf_modules,
        num_child_instances: cli.num_child_instances,
        hierarchy_child_source_mode: cli.hierarchy_child_source_mode,
        min_hierarchy_depth: cli.min_hierarchy_depth,
        max_hierarchy_depth: cli.max_hierarchy_depth,
        min_child_instances_per_module: cli.min_child_instances_per_module,
        max_child_instances_per_module: cli.max_child_instances_per_module,
        child_instances_per_module_by_depth,
        hierarchy_sibling_route_prob: cli.hierarchy_sibling_route_prob,
        hierarchy_registered_sibling_route_prob: cli.hierarchy_registered_sibling_route_prob,
        hierarchy_registered_sibling_mixed_support_prob: cli
            .hierarchy_registered_sibling_mixed_support_prob,
        hierarchy_registered_child_input_cone_prob: cli.hierarchy_registered_child_input_cone_prob,
        hierarchy_child_input_cone_prob: cli.hierarchy_child_input_cone_prob,
        hierarchy_parent_cone_instance_prob: cli.hierarchy_parent_cone_instance_prob,
        max_parent_cone_instances_per_module: cli.max_parent_cone_instances_per_module,
        hierarchy_parent_flop_prob: cli.hierarchy_parent_flop_prob,
        max_rss_mb: cli.max_rss_mb,
        ram_abort_pct: cli.ram_abort_pct,
        // KNOB-ERGONOMICS-AND-PRESETS.2b.1 — the 16 promoted knobs. The four
        // dedup/identity bools are `SetTrue` flags ⇒ map a present flag to
        // `Some(true)` and an absent flag to `None` (so it never clobbers a
        // preset / config value with a spurious `false`).
        function_emit_prob: cli.function_emit_prob,
        generate_loop_emit_prob: cli.generate_loop_emit_prob,
        task_emit_prob: cli.task_emit_prob,
        cone_function_emit_prob: cli.cone_function_emit_prob,
        multi_output_task_emit_prob: cli.multi_output_task_emit_prob,
        mux_if_emit_prob: cli.mux_if_emit_prob,
        case_mux_if_emit_prob: cli.case_mux_if_emit_prob,
        casez_mux_if_emit_prob: cli.casez_mux_if_emit_prob,
        soft_union_slice_prob: cli.soft_union_slice_prob,
        width_parameterization_prob: cli.width_parameterization_prob,
        aggregate_prob: cli.aggregate_prob,
        aggregate_array_prob: cli.aggregate_array_prob,
        memory_prob: cli.memory_prob,
        fsm_prob: cli.fsm_prob,
        fsm_mealy_prob: cli.fsm_mealy_prob,
        multi_clock_prob: cli.multi_clock_prob,
        cdc_synchronizer_stages: cli.cdc_synchronizer_stages,
        hierarchy_module_dedup: cli.hierarchy_module_dedup.then_some(true),
        hierarchy_semantic_module_dedup: cli.hierarchy_semantic_module_dedup.then_some(true),
        hierarchy_sequential_module_dedup: cli.hierarchy_sequential_module_dedup.then_some(true),
        bisimulation_flop_merge: cli.bisimulation_flop_merge.then_some(true),
        steer: cli.steer.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BUG-HUNT-ORCHESTRATION.2d: the `anvil hunt` subcommand --------------

    /// The historical flat-flag invocation parses with **no** subcommand, so the
    /// existing generate path runs unchanged — the byte-identical-default guard
    /// at the parse level.
    #[test]
    fn flat_default_invocation_has_no_subcommand() {
        let cli = Cli::parse_from(["anvil", "--seed", "42"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.seed, 42);
    }

    // --- KNOB-ERGONOMICS-AND-PRESETS.2b.1: promoted CLI flags + --profile -----

    /// A promoted prob flag flows into `Overrides` when passed, and stays `None`
    /// (never clobbers) when absent; the `--profile` value is captured.
    #[test]
    fn promoted_prob_flag_and_profile_parse_into_overrides() {
        let cli = Cli::parse_from([
            "anvil",
            "--profile",
            "structured-emission-max",
            "--function-emit-prob",
            "1.0",
        ]);
        assert_eq!(cli.profile.as_deref(), Some("structured-emission-max"));
        let o = cli_overrides(&cli);
        assert_eq!(o.function_emit_prob, Some(1.0));
        // an un-passed promoted knob stays None
        assert_eq!(o.memory_prob, None);
    }

    /// COVERAGE-STEERED-GENERATION.2c.1 — repeatable `--steer KEY=WEIGHT` parses
    /// into `Overrides.steer` and resolves into `Config.steering` (knob name →
    /// per_knob, category → per_category); no `--steer` ⇒ empty steering.
    #[test]
    fn steer_flag_parses_and_resolves_into_steering() {
        let cli = Cli::parse_from(["anvil", "--steer", "state=4.0", "--steer", "flop_prob=2.0"]);
        let o = cli_overrides(&cli);
        assert_eq!(
            o.steer,
            vec![("state".to_string(), 4.0), ("flop_prob".to_string(), 2.0)]
        );
        let cfg =
            anvil::config::resolve_config(anvil::config::Config::default(), None, &o, 1).unwrap();
        assert_eq!(cfg.steering.per_category.get("state"), Some(&4.0));
        assert_eq!(cfg.steering.per_knob.get("flop_prob"), Some(&2.0));

        // No --steer ⇒ empty steer vec ⇒ empty steering (DUT byte-identical).
        let plain = cli_overrides(&Cli::parse_from(["anvil", "--seed", "1"]));
        assert!(plain.steer.is_empty());
    }

    #[test]
    fn parse_steer_arg_accepts_pair_and_rejects_malformed() {
        assert_eq!(
            parse_steer_arg("state=4.0").unwrap(),
            ("state".to_string(), 4.0)
        );
        // trims whitespace.
        assert_eq!(
            parse_steer_arg(" flop_prob = 0.5 ").unwrap(),
            ("flop_prob".to_string(), 0.5)
        );
        // missing '=' and a non-numeric weight both error (CLI-time).
        assert!(parse_steer_arg("state").is_err());
        assert!(parse_steer_arg("state=high").is_err());
        assert!(parse_steer_arg("=4.0").is_err());
    }

    /// A `SetTrue` dedup bool maps to `Some(true)` only when present, else `None`
    /// (so it never overrides a preset/config value with a spurious `false`).
    #[test]
    fn promoted_dedup_bool_flag_maps_to_some_true_only_when_present() {
        let on = Cli::parse_from(["anvil", "--hierarchy-module-dedup"]);
        assert_eq!(cli_overrides(&on).hierarchy_module_dedup, Some(true));

        let off = Cli::parse_from(["anvil", "--seed", "1"]);
        assert_eq!(cli_overrides(&off).hierarchy_module_dedup, None);
        assert_eq!(cli_overrides(&off).bisimulation_flop_merge, None);
    }

    /// `anvil hunt` with every flag set parses into the expected `HuntCommand`.
    #[test]
    fn hunt_subcommand_parses_all_flags() {
        let cli = Cli::parse_from([
            "anvil",
            "hunt",
            "--seed",
            "5",
            "--seeds",
            "3",
            "--tools",
            "verilator,iverilog",
            "--yosys-mode",
            "both",
            "--no-minimize",
            "--budget",
            "10",
            "--diff-sim",
            "--divergence",
            "--out",
            "/tmp/anvil-hunt",
        ]);
        let Some(Commands::Hunt(h)) = cli.command else {
            panic!("expected a hunt subcommand");
        };
        assert_eq!(h.seed, 5);
        assert_eq!(h.seeds, 3);
        assert_eq!(
            h.tools,
            vec![AcceptanceTool::Verilator, AcceptanceTool::Iverilog]
        );
        assert_eq!(h.yosys_mode, YosysMode::Both);
        assert!(h.no_minimize);
        assert_eq!(h.budget, 10);
        assert!(h.diff_sim);
        assert!(h.divergence);
        assert_eq!(h.out, Some(PathBuf::from("/tmp/anvil-hunt")));
    }

    /// `anvil hunt` with no flags carries the documented defaults (seed 0,
    /// 16 seeds, minimize on, budget 200, no diff-sim, no bundle, default tools).
    #[test]
    fn hunt_subcommand_defaults() {
        let cli = Cli::parse_from(["anvil", "hunt"]);
        let Some(Commands::Hunt(h)) = cli.command else {
            panic!("expected a hunt subcommand");
        };
        assert_eq!(h.seed, 0);
        assert_eq!(h.seeds, 16);
        assert!(h.tools.is_empty()); // empty ⇒ the verilator+yosys default at build time
        assert_eq!(h.yosys_mode, YosysMode::WithoutAbc);
        assert!(!h.no_minimize);
        assert_eq!(h.budget, 200);
        assert!(!h.diff_sim);
        assert!(!h.divergence);
        assert!(h.out.is_none());
    }

    /// `--seeds 0` is rejected by the clap range (the sweep must fuzz ≥ 1 seed).
    #[test]
    fn hunt_rejects_zero_seeds() {
        assert!(Cli::try_parse_from(["anvil", "hunt", "--seeds", "0"]).is_err());
    }

    /// The arg → `HuntRequest` mapping (no tool run): the seed is stamped into
    /// the knob profile, an empty `--tools` becomes the `verilator`+`yosys`
    /// default, `--no-minimize`/`--budget`/`--diff-sim`/`--out` map through.
    #[test]
    fn build_hunt_request_maps_args_to_request() {
        let args = HuntCommand {
            seed: 9,
            seeds: 4,
            config: None,
            tools: vec![], // empty ⇒ the default tool set
            yosys_mode: YosysMode::Both,
            no_minimize: true,
            budget: 12,
            diff_sim: true,
            divergence: true,
            out: Some(PathBuf::from("/tmp/anvil-hunt-out")),
        };
        let req = build_hunt_request(&args).expect("build request");
        assert_eq!(req.base_seed, 9);
        assert_eq!(req.seeds, 4);
        assert_eq!(req.config.seed, 9); // the sweep seed is stamped into the profile
        assert_eq!(req.validate.tools, ValidateOptions::default().tools);
        assert_eq!(req.validate.yosys_mode, YosysMode::Both);
        assert!(!req.minimize); // --no-minimize
        assert_eq!(req.max_oracle_calls, 12);
        assert!(req.diff_sim);
        assert!(req.divergence); // --divergence
        assert_eq!(req.bundle_root, Some(PathBuf::from("/tmp/anvil-hunt-out")));
    }

    #[test]
    fn identity_mode_cli_parses_directly() {
        let cli = Cli::parse_from(["anvil", "--identity-mode", "relaxed"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.identity_mode, Some(IdentityMode::Relaxed));
        assert_eq!(overrides.factorization_level, None);
    }

    #[test]
    fn full_factorization_alias_sets_identity_mode_and_egraph_request() {
        let cli = Cli::parse_from(["anvil", "--full-factorization"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.identity_mode, Some(IdentityMode::NodeId));
        assert_eq!(
            overrides.factorization_level,
            Some(FactorizationLevel::EGraph)
        );
    }

    #[test]
    fn no_full_factorization_alias_sets_relaxed_and_none() {
        let cli = Cli::parse_from(["anvil", "--no-full-factorization"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.identity_mode, Some(IdentityMode::Relaxed));
        assert_eq!(
            overrides.factorization_level,
            Some(FactorizationLevel::None)
        );
    }

    #[test]
    fn explicit_factorization_level_still_parses_directly() {
        let cli = Cli::parse_from(["anvil", "--factorization-level", "peephole"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.identity_mode, None);
        assert_eq!(
            overrides.factorization_level,
            Some(FactorizationLevel::Peephole)
        );
    }

    #[test]
    fn newly_exposed_cli_knobs_round_trip_into_overrides() {
        let cli = Cli::parse_from([
            "anvil",
            "--terminal-reuse-prob",
            "0.25",
            "--constant-prob",
            "0.4",
            "--gate-bitwise-weight",
            "9",
            "--gate-arith-weight",
            "8",
            "--gate-struct-weight",
            "7",
            "--gate-compare-weight",
            "6",
            "--gate-reduce-weight",
            "5",
            "--case-mux-prob",
            "0.2",
            "--casez-mux-prob",
            "0.3",
            "--for-fold-prob",
            "0.4",
            "--hierarchy-depth",
            "1",
            "--num-leaf-modules",
            "4",
            "--num-child-instances",
            "7",
            "--hierarchy-child-source-mode",
            "on-demand",
            "--min-hierarchy-depth",
            "2",
            "--max-hierarchy-depth",
            "3",
            "--min-child-instances-per-module",
            "2",
            "--max-child-instances-per-module",
            "5",
            "--child-instances-per-depth",
            "0=4:4",
            "--child-instances-per-depth",
            "1=2:3",
            "--hierarchy-registered-sibling-route-prob",
            "0.8",
            "--hierarchy-registered-sibling-mixed-support-prob",
            "0.7",
            "--hierarchy-registered-child-input-cone-prob",
            "0.85",
            "--hierarchy-child-input-cone-prob",
            "0.75",
            "--hierarchy-parent-cone-instance-prob",
            "0.55",
            "--max-parent-cone-instances-per-module",
            "3",
            "--hierarchy-parent-flop-prob",
            "0.6",
        ]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.terminal_reuse_prob, Some(0.25));
        assert_eq!(overrides.constant_prob, Some(0.4));
        assert_eq!(overrides.gate_bitwise_weight, Some(9));
        assert_eq!(overrides.gate_arith_weight, Some(8));
        assert_eq!(overrides.gate_struct_weight, Some(7));
        assert_eq!(overrides.gate_compare_weight, Some(6));
        assert_eq!(overrides.gate_reduce_weight, Some(5));
        assert_eq!(overrides.case_mux_prob, Some(0.2));
        assert_eq!(overrides.casez_mux_prob, Some(0.3));
        assert_eq!(overrides.for_fold_prob, Some(0.4));
        assert_eq!(overrides.hierarchy_depth, Some(1));
        assert_eq!(overrides.num_leaf_modules, Some(4));
        assert_eq!(overrides.num_child_instances, Some(7));
        assert_eq!(
            overrides.hierarchy_child_source_mode,
            Some(HierarchyChildSourceMode::OnDemand)
        );
        assert_eq!(overrides.min_hierarchy_depth, Some(2));
        assert_eq!(overrides.max_hierarchy_depth, Some(3));
        assert_eq!(overrides.min_child_instances_per_module, Some(2));
        assert_eq!(overrides.max_child_instances_per_module, Some(5));
        assert_eq!(
            overrides.child_instances_per_module_by_depth,
            Some(BTreeMap::from([
                (0, CountRange { min: 4, max: 4 }),
                (1, CountRange { min: 2, max: 3 }),
            ]))
        );
        assert_eq!(overrides.hierarchy_registered_sibling_route_prob, Some(0.8));
        assert_eq!(
            overrides.hierarchy_registered_sibling_mixed_support_prob,
            Some(0.7)
        );
        assert_eq!(
            overrides.hierarchy_registered_child_input_cone_prob,
            Some(0.85)
        );
        assert_eq!(overrides.hierarchy_child_input_cone_prob, Some(0.75));
        assert_eq!(overrides.hierarchy_parent_cone_instance_prob, Some(0.55));
        assert_eq!(overrides.max_parent_cone_instances_per_module, Some(3));
        assert_eq!(overrides.hierarchy_parent_flop_prob, Some(0.6));
    }

    #[test]
    fn memory_governor_cli_knobs_round_trip_into_overrides() {
        let cli = Cli::parse_from(["anvil", "--max-rss-mb", "8192", "--ram-abort-pct", "90"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.max_rss_mb, Some(8192));
        assert_eq!(overrides.ram_abort_pct, Some(90));
    }

    #[test]
    fn memory_governor_defaults_to_off_when_flags_absent() {
        let cli = Cli::parse_from(["anvil", "--seed", "1"]);
        let overrides = cli_overrides(&cli);
        assert_eq!(overrides.max_rss_mb, None);
        assert_eq!(overrides.ram_abort_pct, None);
    }
}
