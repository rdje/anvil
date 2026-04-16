use anvil::config::ConstructionStrategy;
use anvil::{Config, Generator};
use clap::{Parser, ValueEnum};
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
    /// Construction strategy: sequential, shuffled, interleaved, or
    /// graph-first (default). See `book/src/construction-strategies.md`.
    #[arg(long, value_enum)]
    construction_strategy: Option<ConstructionStrategy>,
    /// Target number of top-level units (gate / flop / comb-mux block)
    /// grown in the pool by the `graph-first` strategy.
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

    match (&cli.out, cli.count) {
        (None, 1) => {
            let m = gen.generate_module();
            let metrics = anvil::metrics::compute(&m);
            print!("{}", anvil::emit::to_sv(&m));
            if cli.metrics {
                eprintln!("{}", serde_json::to_string_pretty(&metrics)?);
            }
        }
        (Some(dir), n) => {
            std::fs::create_dir_all(dir)?;
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
    anvil::config::Overrides {
        min_inputs: cli.min_inputs,
        max_inputs: cli.max_inputs,
        min_outputs: cli.min_outputs,
        max_outputs: cli.max_outputs,
        min_width: cli.min_width,
        max_width: cli.max_width,
        max_depth: cli.max_depth,
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
        graph_first_pool_size: cli.graph_first_pool_size,
        coefficient_prob: cli.coefficient_prob,
        min_coefficient: cli.min_coefficient,
        max_coefficient: cli.max_coefficient,
        const_shift_amount_prob: cli.const_shift_amount_prob,
        min_shift_amount: cli.min_shift_amount,
        max_shift_amount: cli.max_shift_amount,
        gate_shift_weight: cli.gate_shift_weight,
        const_comparand_prob: cli.const_comparand_prob,
        min_comparand: cli.min_comparand,
        max_comparand: cli.max_comparand,
        priority_encoder_prob: cli.priority_encoder_prob,
        max_ast_instances: cli.max_ast_instances,
        mux_arm_duplication_rate: cli.mux_arm_duplication_rate,
    }
}
