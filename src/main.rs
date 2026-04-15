use anvil::config::ConstructionStrategy;
use anvil::{Config, Generator};
use clap::Parser;
use std::path::PathBuf;

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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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

    let mut gen = Generator::new(cfg.clone());

    match (&cli.out, cli.count) {
        (None, 1) => {
            let m = gen.generate_module();
            print!("{}", anvil::emit::to_sv(&m));
        }
        (Some(dir), n) => {
            std::fs::create_dir_all(dir)?;
            let mut manifest = Vec::new();
            for i in 0..n {
                let m = gen.generate_module();
                let fname = format!("mod_{}_{:04}.sv", cli.seed, i);
                std::fs::write(dir.join(&fname), anvil::emit::to_sv(&m))?;
                manifest.push(serde_json::json!({
                    "file": fname,
                    "name": m.name,
                    "inputs": m.inputs.len(),
                    "outputs": m.outputs.len(),
                    "nodes": m.nodes.len(),
                }));
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
    }
}
