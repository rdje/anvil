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
    }
}
