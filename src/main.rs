use anvil::config::{
    ConstructionStrategy, CountRange, FactorizationLevel, HierarchyChildSourceMode, IdentityMode,
};
use anvil::{Config, Generator};
use clap::{Parser, ValueEnum};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::info;

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

#[derive(Parser, Debug)]
#[command(name = "anvil", version, about = "Random synthesizable RTL generator")]
struct Cli {
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_tracing(&cli)?;

    let mut cfg = if let Some(path) = &cli.config {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str::<Config>(&text)?
    } else {
        Config::default()
    };
    cfg.apply_cli_overrides(&cli_overrides(&cli));
    cfg.seed = cli.seed;
    cfg.validate().map_err(|e| anyhow::anyhow!("{}", e))?;

    if cli.dump_config {
        println!("{}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
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
                let design_metrics = anvil::metrics::compute_design(&design);
                print!("{}", anvil::emit::to_sv_design(&design));
                if cli.metrics {
                    eprintln!("{}", serde_json::to_string_pretty(&design_metrics)?);
                    for module in &design.modules {
                        let metrics = anvil::metrics::compute(module);
                        eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                    }
                }
            } else {
                let m = gen.generate_module();
                let metrics = anvil::metrics::compute(&m);
                print!("{}", anvil::emit::to_sv(&m));
                if cli.metrics {
                    eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                }
            }
        }
        (Some(dir), n) => {
            std::fs::create_dir_all(dir)?;
            if hierarchical {
                let mut designs = Vec::new();
                for design_index in 0..n {
                    let design = gen.generate_design();
                    anvil::ir::validate::validate_design(&design)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    let design_metrics = anvil::metrics::compute_design(&design);
                    let mut modules = Vec::new();
                    for module in &design.modules {
                        let metrics = anvil::metrics::compute(module);
                        let fname = format!("{}.sv", module.name);
                        std::fs::write(
                            dir.join(&fname),
                            anvil::emit::to_sv_in_design(module, &design),
                        )?;
                        modules.push(serde_json::json!({
                            "file": fname,
                            "name": module.name,
                            "metrics": metrics,
                        }));
                        if cli.metrics {
                            eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                        }
                    }
                    designs.push(serde_json::json!({
                        "index": design_index,
                        "top": design.top,
                        "metrics": design_metrics,
                        "modules": modules,
                    }));
                    if cli.metrics {
                        eprintln!("{}", serde_json::to_string_pretty(&design_metrics)?);
                    }
                }
                std::fs::write(
                    dir.join("manifest.json"),
                    serde_json::to_string_pretty(&serde_json::json!({
                        "seed": cli.seed,
                        "config": cfg,
                        "designs": designs,
                    }))?,
                )?;
            } else {
                let mut manifest = Vec::new();
                for i in 0..n {
                    let m = gen.generate_module();
                    let metrics = anvil::metrics::compute(&m);
                    let fname = format!("mod_{}_{:04}.sv", cli.seed, i);
                    std::fs::write(dir.join(&fname), anvil::emit::to_sv(&m))?;
                    manifest.push(serde_json::json!({
                        "file": fname,
                        "name": m.name,
                        "metrics": metrics,
                    }));
                    if cli.metrics {
                        eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
                    }
                }
                std::fs::write(
                    dir.join("manifest.json"),
                    serde_json::to_string_pretty(&serde_json::json!({
                        "seed": cli.seed,
                        "config": cfg,
                        "modules": manifest,
                    }))?,
                )?;
            }
        }
        (None, _) => {
            anyhow::bail!("--out is required when --count > 1");
        }
    }

    info!("✅ anvil done");
    Ok(())
}

/// Wire a `tracing` subscriber. Output is deterministic: no timestamps,
/// no thread IDs, no ANSI colours — just `LEVEL module::path message`
/// (plus any structured fields). This keeps trace output diffable
/// across runs with the same `(seed, knobs)`.
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
        hierarchy_registered_child_input_cone_prob: cli.hierarchy_registered_child_input_cone_prob,
        hierarchy_child_input_cone_prob: cli.hierarchy_child_input_cone_prob,
        hierarchy_parent_cone_instance_prob: cli.hierarchy_parent_cone_instance_prob,
        max_parent_cone_instances_per_module: cli.max_parent_cone_instances_per_module,
        hierarchy_parent_flop_prob: cli.hierarchy_parent_flop_prob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            overrides.hierarchy_registered_child_input_cone_prob,
            Some(0.85)
        );
        assert_eq!(overrides.hierarchy_child_input_cone_prob, Some(0.75));
        assert_eq!(overrides.hierarchy_parent_cone_instance_prob, Some(0.55));
        assert_eq!(overrides.max_parent_cone_instances_per_module, Some(3));
        assert_eq!(overrides.hierarchy_parent_flop_prob, Some(0.6));
    }
}
