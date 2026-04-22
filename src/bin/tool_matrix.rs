use anvil::config::{ConstructionStrategy, FactorizationLevel, IdentityMode};
use anvil::metrics::Metrics;
use anvil::{Config, Generator, GeneratorCheckpoint};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const PHASE1_MIN_TOTAL_MODULES: usize = 1000;
const PHASE2_SHARE_MIN_TOTAL_MODULES: usize = 216;

#[derive(Parser, Debug)]
#[command(
    name = "tool_matrix",
    version,
    about = "Generate a reproducible ANVIL scenario matrix and run Verilator/Yosys on it"
)]
struct Cli {
    /// Output directory. Each scenario gets its own subdirectory.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Base seed used to derive deterministic per-scenario seeds.
    #[arg(long, default_value_t = 0)]
    base_seed: u64,

    /// Number of modules to generate per scenario.
    #[arg(long, default_value_t = 1)]
    modules_per_scenario: usize,

    /// Elevate the run to the repo-owned Phase 1 gate:
    /// require full coverage and at least 1000 generated modules total.
    #[arg(long)]
    phase1_gate: bool,

    /// Elevate the run to the repo-owned Phase 2 sharing gate:
    /// run the representative share_prob sweep and require its coverage.
    #[arg(long)]
    phase2_share_gate: bool,

    /// Print the built-in scenario list and exit.
    #[arg(long)]
    list_scenarios: bool,

    /// Skip Verilator.
    #[arg(long)]
    skip_verilator: bool,

    /// Skip Yosys.
    #[arg(long)]
    skip_yosys: bool,

    /// Verilator executable to run.
    #[arg(long, default_value = "verilator")]
    verilator_bin: String,

    /// Yosys executable to run.
    #[arg(long, default_value = "yosys")]
    yosys_bin: String,

    /// Yosys synthesis mode: keep the current no-ABC path, run the
    /// warning-clean ABC-enabled harness path, or run both.
    #[arg(long, value_enum, default_value_t = YosysMode::WithoutAbc)]
    yosys_mode: YosysMode,

    /// Return non-zero if the matrix misses intended coverage.
    #[arg(long)]
    fail_on_coverage_gap: bool,

    /// Resume from per-module checkpoints in --out when present.
    #[arg(long)]
    resume: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum YosysMode {
    WithoutAbc,
    WithAbc,
    Both,
}

#[derive(Debug, Clone, Serialize)]
struct Scenario {
    name: String,
    description: String,
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolInvocation {
    tool: String,
    argv: Vec<String>,
    success: bool,
    exit_code: Option<i32>,
    stdout_log: Option<String>,
    stderr_log: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModuleReport {
    file: String,
    name: String,
    metrics: Metrics,
    verilator: Option<ToolInvocation>,
    yosys: Vec<ToolInvocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModuleCheckpoint {
    skip_verilator: bool,
    skip_yosys: bool,
    yosys_mode: String,
    runtime_fingerprint: Option<String>,
    sv_hash: Option<String>,
    generator_checkpoint: Option<GeneratorCheckpoint>,
    report: ModuleReport,
}

#[derive(Debug, Clone, Serialize, Default)]
struct ToolSummary {
    verilator_passed: usize,
    verilator_failed: usize,
    yosys_without_abc_passed: usize,
    yosys_without_abc_failed: usize,
    yosys_with_abc_passed: usize,
    yosys_with_abc_failed: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
struct AggregateMetrics {
    modules: usize,
    total_nodes: usize,
    total_gates: usize,
    total_flops: usize,
    total_shared_nodes: usize,
    total_priority_encoder_blocks: u64,
    total_comb_muxes_one_hot: u64,
    total_comb_muxes_encoded: u64,
    total_case_mux_blocks: u64,
    total_casez_mux_blocks: u64,
    total_semantic_gates_merged: u64,
    total_flops_merged: u64,
    gates_by_kind: BTreeMap<String, u64>,
    knob_roll_attempts: BTreeMap<String, u64>,
    knob_roll_fires: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct CoverageSummary {
    construction_strategies: BTreeSet<String>,
    identity_modes: BTreeSet<String>,
    factorization_levels: BTreeSet<String>,
    share_prob_values: BTreeSet<String>,
    gate_categories: BTreeSet<String>,
    gate_kinds: BTreeSet<String>,
    knob_attempts_seen: BTreeSet<String>,
    knob_fires_seen: BTreeSet<String>,
    saw_comb_only_module: bool,
    saw_sequential_module: bool,
    saw_priority_encoder: bool,
    saw_comb_mux_one_hot: bool,
    saw_comb_mux_encoded: bool,
    saw_case_mux: bool,
    saw_casez_mux: bool,
    saw_flop_mux_one_hot: bool,
    saw_flop_mux_encoded: bool,
    saw_semantic_gate_merge: bool,
    saw_flop_merge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioSet {
    Default,
    Phase2Share,
}

#[derive(Debug, Clone, Serialize, Default)]
struct ShareSweepBucket {
    scenarios: usize,
    modules: usize,
    total_nodes: usize,
    total_shared_nodes: usize,
    avg_nodes_per_module: f64,
    shared_node_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct ShareSweepSummary {
    buckets: BTreeMap<String, ShareSweepBucket>,
}

#[derive(Debug, Clone, Serialize)]
struct ScenarioReport {
    name: String,
    description: String,
    out_dir: String,
    config: Config,
    aggregate: AggregateMetrics,
    coverage: CoverageSummary,
    tool_summary: ToolSummary,
    modules: Vec<ModuleReport>,
}

#[derive(Debug, Clone, Serialize)]
struct MatrixReport {
    base_seed: u64,
    modules_per_scenario: usize,
    scenario_count: usize,
    total_modules: usize,
    scenario_set: String,
    phase1_gate: bool,
    phase2_share_gate: bool,
    yosys_mode: String,
    coverage: CoverageSummary,
    coverage_gaps: Vec<String>,
    share_sweep: Option<ShareSweepSummary>,
    tool_summary: ToolSummary,
    scenarios: Vec<ScenarioReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RunPlan {
    modules_per_scenario: usize,
    fail_on_coverage_gap: bool,
    total_modules: usize,
}

#[derive(Debug, Clone)]
struct ModulePaths {
    file: String,
    stem: String,
    sv_path: PathBuf,
    checkpoint_path: PathBuf,
}

#[derive(Debug, Clone)]
struct PreparedModule {
    paths: ModulePaths,
    name: String,
    metrics: Metrics,
    sv_text: String,
    sv_hash: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.modules_per_scenario == 0 {
        bail!("--modules-per-scenario must be >= 1");
    }
    let runtime_fingerprint = current_runtime_fingerprint().ok();
    let scenario_set = select_scenario_set(&cli)?;

    let scenarios = build_scenarios(cli.base_seed, scenario_set)?;
    if cli.list_scenarios {
        for scenario in &scenarios {
            println!("{}: {}", scenario.name, scenario.description);
        }
        return Ok(());
    }

    let plan = derive_run_plan(&cli, scenarios.len());

    let out_dir = cli
        .out
        .as_ref()
        .context("--out is required unless --list-scenarios is used")?;

    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create output directory {}", out_dir.display()))?;

    let mut scenario_reports = Vec::with_capacity(scenarios.len());
    let mut global_tool_summary = ToolSummary::default();
    let mut global_coverage = CoverageSummary::default();

    for scenario in scenarios {
        let report = run_scenario(
            &scenario,
            &cli,
            &plan,
            out_dir,
            runtime_fingerprint.as_deref(),
        )?;
        merge_tool_summary(&mut global_tool_summary, &report.tool_summary);
        merge_coverage(&mut global_coverage, &report.coverage);
        scenario_reports.push(report);
    }

    let share_sweep = (scenario_set == ScenarioSet::Phase2Share)
        .then(|| summarize_share_sweep(&scenario_reports));
    let coverage_gaps = compute_coverage_gaps(scenario_set, &global_coverage, share_sweep.as_ref());
    let report = MatrixReport {
        base_seed: cli.base_seed,
        modules_per_scenario: plan.modules_per_scenario,
        scenario_count: scenario_reports.len(),
        total_modules: plan.total_modules,
        scenario_set: scenario_set_slug(scenario_set).to_string(),
        phase1_gate: cli.phase1_gate,
        phase2_share_gate: cli.phase2_share_gate,
        yosys_mode: yosys_mode_slug(cli.yosys_mode).to_string(),
        coverage: global_coverage,
        coverage_gaps,
        share_sweep,
        tool_summary: global_tool_summary,
        scenarios: scenario_reports,
    };

    let report_path = out_dir.join("tool_matrix_report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("write {}", report_path.display()))?;

    println!(
        "tool_matrix: {} scenarios, {} modules/scenario, report {}",
        report.scenario_count,
        report.modules_per_scenario,
        report_path.display()
    );
    println!("tool_matrix: total modules = {}", report.total_modules);
    println!(
        "tool_matrix: Verilator pass/fail = {}/{}, Yosys without-abc pass/fail = {}/{}, Yosys with-abc pass/fail = {}/{}",
        report.tool_summary.verilator_passed,
        report.tool_summary.verilator_failed,
        report.tool_summary.yosys_without_abc_passed,
        report.tool_summary.yosys_without_abc_failed,
        report.tool_summary.yosys_with_abc_passed,
        report.tool_summary.yosys_with_abc_failed
    );
    if let Some(share_sweep) = &report.share_sweep {
        for (share_prob, bucket) in &share_sweep.buckets {
            println!(
                "tool_matrix: share_prob={} -> scenarios={}, modules={}, total_nodes={}, total_shared_nodes={}, avg_nodes/module={:.2}, shared_node_fraction={:.4}",
                share_prob,
                bucket.scenarios,
                bucket.modules,
                bucket.total_nodes,
                bucket.total_shared_nodes,
                bucket.avg_nodes_per_module,
                bucket.shared_node_fraction
            );
        }
    }
    if !report.coverage_gaps.is_empty() {
        println!(
            "tool_matrix: coverage gaps detected ({}): {}",
            report.coverage_gaps.len(),
            report.coverage_gaps.join("; ")
        );
    }

    if report.tool_summary.verilator_failed > 0 || report.tool_summary.yosys_failed() > 0 {
        bail!(
            "tool_matrix detected downstream-tool failures; see {}",
            report_path.display()
        );
    }
    if plan.fail_on_coverage_gap && !report.coverage_gaps.is_empty() {
        bail!(
            "tool_matrix detected coverage gaps; see {}",
            report_path.display()
        );
    }

    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(hash_bytes(&buf))
}

fn current_runtime_fingerprint() -> Result<String> {
    let exe = std::env::current_exe().context("resolve current tool_matrix executable")?;
    hash_file(&exe)
}

fn derive_run_plan(cli: &Cli, scenario_count: usize) -> RunPlan {
    let gate_modules_per_scenario = if cli.phase1_gate {
        PHASE1_MIN_TOTAL_MODULES.div_ceil(scenario_count)
    } else if cli.phase2_share_gate {
        PHASE2_SHARE_MIN_TOTAL_MODULES.div_ceil(scenario_count)
    } else {
        1
    };
    let modules_per_scenario = cli.modules_per_scenario.max(gate_modules_per_scenario);
    let total_modules = modules_per_scenario * scenario_count;
    RunPlan {
        modules_per_scenario,
        fail_on_coverage_gap: cli.fail_on_coverage_gap || cli.phase1_gate || cli.phase2_share_gate,
        total_modules,
    }
}

fn select_scenario_set(cli: &Cli) -> Result<ScenarioSet> {
    if cli.phase1_gate && cli.phase2_share_gate {
        bail!("--phase1-gate and --phase2-share-gate are mutually exclusive");
    }
    if cli.phase2_share_gate {
        Ok(ScenarioSet::Phase2Share)
    } else {
        Ok(ScenarioSet::Default)
    }
}

fn build_scenarios(base_seed: u64, scenario_set: ScenarioSet) -> Result<Vec<Scenario>> {
    let scenarios = match scenario_set {
        ScenarioSet::Default => build_default_scenarios(base_seed)?,
        ScenarioSet::Phase2Share => build_phase2_share_scenarios(base_seed)?,
    };

    let mut seen = BTreeSet::new();
    for scenario in &scenarios {
        if !seen.insert(scenario.name.clone()) {
            bail!("duplicate scenario name {}", scenario.name);
        }
    }

    Ok(scenarios)
}

fn build_default_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;

    scenarios.push(make_scenario(
        "int_relaxed_none_default",
        "Interleaved default knobs with relaxed identity mode and no factorization.",
        relaxed_default_config(ConstructionStrategy::Interleaved, next_seed),
    )?);
    next_seed += 1;

    for level in [
        FactorizationLevel::None,
        FactorizationLevel::Cse,
        FactorizationLevel::OperandUnique,
        FactorizationLevel::Commutative,
        FactorizationLevel::Associative,
        FactorizationLevel::ConstantFold,
        FactorizationLevel::Peephole,
        FactorizationLevel::EGraph,
    ] {
        let name = format!("int_nodeid_{}_default", factorization_level_slug(level));
        let description = format!(
            "Interleaved default knobs with node-id identity mode at {}.",
            factorization_level_name(level)
        );
        scenarios.push(make_scenario(
            &name,
            &description,
            nodeid_default_config(ConstructionStrategy::Interleaved, level, next_seed),
        )?);
        next_seed += 1;
    }

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ] {
        let share_name = format!(
            "{}_nodeid_egraph_share_heavy_comb_only",
            strategy_slug(strategy)
        );
        let share_desc = format!(
            "{} strategy, node-id + e-graph, combinational share-heavy profile.",
            construction_strategy_name(strategy)
        );
        scenarios.push(make_scenario(
            &share_name,
            &share_desc,
            share_heavy_comb_only_config(strategy, next_seed, 0.9),
        )?);
        next_seed += 1;

        let motif_name = format!("{}_nodeid_egraph_motif_heavy_seq", strategy_slug(strategy));
        let motif_desc = format!(
            "{} strategy, node-id + e-graph, sequential motif-heavy profile.",
            construction_strategy_name(strategy)
        );
        scenarios.push(make_scenario(
            &motif_name,
            &motif_desc,
            motif_heavy_sequential_config(strategy, next_seed, 0.4),
        )?);
        next_seed += 1;
    }

    Ok(scenarios)
}

fn build_phase2_share_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ] {
        for share_prob in [0.0, 0.3, 0.9] {
            let share_slug = share_prob_slug(share_prob);
            let share_label = share_prob_label(share_prob);

            let comb_name = format!(
                "{}_nodeid_egraph_comb_share{}",
                strategy_slug(strategy),
                share_slug
            );
            let comb_desc = format!(
                "{} strategy, node-id + e-graph, combinational sharing sweep at share_prob={}.",
                construction_strategy_name(strategy),
                share_label
            );
            scenarios.push(make_scenario(
                &comb_name,
                &comb_desc,
                share_heavy_comb_only_config(strategy, next_seed, share_prob),
            )?);
            next_seed += 1;

            let seq_name = format!(
                "{}_nodeid_egraph_seq_share{}",
                strategy_slug(strategy),
                share_slug
            );
            let seq_desc = format!(
                "{} strategy, node-id + e-graph, sequential sharing sweep at share_prob={}.",
                construction_strategy_name(strategy),
                share_label
            );
            scenarios.push(make_scenario(
                &seq_name,
                &seq_desc,
                motif_heavy_sequential_config(strategy, next_seed, share_prob),
            )?);
            next_seed += 1;
        }
    }

    Ok(scenarios)
}

fn make_scenario(name: &str, description: &str, config: Config) -> Result<Scenario> {
    config.validate().map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(Scenario {
        name: name.to_string(),
        description: description.to_string(),
        config,
    })
}

fn relaxed_default_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::Relaxed,
        factorization_level: FactorizationLevel::None,
        ..Config::default()
    }
}

fn nodeid_default_config(
    strategy: ConstructionStrategy,
    level: FactorizationLevel,
    seed: u64,
) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: level,
        ..Config::default()
    }
}

fn share_heavy_comb_only_config(
    strategy: ConstructionStrategy,
    seed: u64,
    share_prob: f64,
) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        flop_prob: 0.0,
        share_prob,
        terminal_reuse_prob: 0.9,
        constant_prob: 0.05,
        max_depth: 8,
        min_inputs: 4,
        max_inputs: 8,
        min_outputs: 2,
        max_outputs: 4,
        ..Config::default()
    }
}

fn motif_heavy_sequential_config(
    strategy: ConstructionStrategy,
    seed: u64,
    share_prob: f64,
) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        flop_prob: 0.45,
        share_prob,
        terminal_reuse_prob: 0.6,
        constant_prob: 0.15,
        coefficient_prob: 0.6,
        const_shift_amount_prob: 0.95,
        const_comparand_prob: 0.75,
        priority_encoder_prob: 0.25,
        case_mux_prob: 0.25,
        casez_mux_prob: 0.25,
        comb_mux_prob: 0.35,
        gate_shift_weight: 3,
        gate_compare_weight: 3,
        gate_reduce_weight: 2,
        min_inputs: 3,
        max_inputs: 8,
        min_outputs: 2,
        max_outputs: 4,
        min_width: 1,
        max_width: 16,
        max_depth: 7,
        ..Config::default()
    }
}

fn run_scenario(
    scenario: &Scenario,
    cli: &Cli,
    plan: &RunPlan,
    out_root: &Path,
    runtime_fingerprint: Option<&str>,
) -> Result<ScenarioReport> {
    let scenario_dir = out_root.join(&scenario.name);
    std::fs::create_dir_all(&scenario_dir)
        .with_context(|| format!("create scenario directory {}", scenario_dir.display()))?;

    let mut generator = Generator::new(scenario.config.clone());
    let mut modules = Vec::with_capacity(plan.modules_per_scenario);

    for module_index in 0..plan.modules_per_scenario {
        if let Some(report) = resume_existing_module(
            &mut generator,
            scenario,
            cli,
            &scenario_dir,
            module_index,
            runtime_fingerprint,
        )? {
            modules.push(report);
            continue;
        }

        let prepared = prepare_module(&mut generator, scenario, &scenario_dir, module_index)?;
        let generator_checkpoint = generator.checkpoint();
        modules.push(materialize_prepared_module(
            cli,
            &scenario_dir,
            prepared,
            &generator_checkpoint,
            runtime_fingerprint,
            true,
        )?);
    }

    write_scenario_manifest(&scenario_dir, scenario, &modules)?;

    let aggregate = aggregate_metrics(&modules);
    let coverage = summarize_coverage(scenario, &modules);
    let tool_summary = summarize_tools(&modules);

    Ok(ScenarioReport {
        name: scenario.name.clone(),
        description: scenario.description.clone(),
        out_dir: scenario_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&scenario.name)
            .to_string(),
        config: scenario.config.clone(),
        aggregate,
        coverage,
        tool_summary,
        modules,
    })
}

fn resume_existing_module(
    generator: &mut Generator,
    scenario: &Scenario,
    cli: &Cli,
    scenario_dir: &Path,
    module_index: usize,
    runtime_fingerprint: Option<&str>,
) -> Result<Option<ModuleReport>> {
    if !cli.resume {
        return Ok(None);
    }

    let paths = module_paths(scenario_dir, scenario.config.seed, module_index)?;
    let checkpoint = load_module_checkpoint(&paths.checkpoint_path)?;
    if !paths.sv_path.exists() && checkpoint.is_none() {
        return Ok(None);
    }

    if let Some(ref checkpoint) = checkpoint {
        if let Some(report) =
            try_fast_resume_checkpoint(generator, cli, &paths, checkpoint, runtime_fingerprint)?
        {
            return Ok(Some(report));
        }
    }

    let prepared = prepare_module_with_paths(generator, scenario, paths)?;
    if let Some(checkpoint) = checkpoint {
        if checkpoint_matches_cli(&checkpoint, cli) {
            let mut report = checkpoint.report;
            validate_checkpoint_against_prepared(&report, &prepared)?;
            report.metrics = prepared.metrics.clone();
            let generator_checkpoint = generator.checkpoint();
            write_module_checkpoint(
                cli,
                &prepared.paths.checkpoint_path,
                &report,
                &generator_checkpoint,
                runtime_fingerprint,
                &prepared.sv_hash,
            )?;
            return Ok(Some(report));
        }
    }

    if !prepared.paths.sv_path.exists() {
        bail!(
            "cannot resume {}: checkpoint exists but {} is missing",
            prepared.paths.file,
            prepared.paths.sv_path.display()
        );
    }

    validate_legacy_sv_against_prepared(&prepared)?;
    let generator_checkpoint = generator.checkpoint();
    materialize_prepared_module(
        cli,
        scenario_dir,
        prepared,
        &generator_checkpoint,
        runtime_fingerprint,
        false,
    )
    .map(Some)
}

fn write_scenario_manifest(
    scenario_dir: &Path,
    scenario: &Scenario,
    modules: &[ModuleReport],
) -> Result<()> {
    let manifest_modules: Vec<_> = modules
        .iter()
        .map(|module| {
            serde_json::json!({
                "file": module.file,
                "name": module.name,
                "metrics": module.metrics,
            })
        })
        .collect();

    let manifest = serde_json::json!({
        "scenario": {
            "name": scenario.name,
            "description": scenario.description,
        },
        "seed": scenario.config.seed,
        "config": scenario.config,
        "modules": manifest_modules,
    });

    let manifest_path = scenario_dir.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(())
}

fn module_paths(scenario_dir: &Path, seed: u64, module_index: usize) -> Result<ModulePaths> {
    let file = format!("mod_{}_{:04}.sv", seed, module_index);
    let sv_path = scenario_dir.join(&file);
    let stem = sv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("scenario file stem not valid UTF-8")?
        .to_string();
    let checkpoint_path = scenario_dir.join(format!("{stem}.module-report.json"));
    Ok(ModulePaths {
        file,
        stem,
        sv_path,
        checkpoint_path,
    })
}

fn prepare_module(
    generator: &mut Generator,
    scenario: &Scenario,
    scenario_dir: &Path,
    module_index: usize,
) -> Result<PreparedModule> {
    let paths = module_paths(scenario_dir, scenario.config.seed, module_index)?;
    prepare_module_with_paths(generator, scenario, paths)
}

fn prepare_module_with_paths(
    generator: &mut Generator,
    _scenario: &Scenario,
    paths: ModulePaths,
) -> Result<PreparedModule> {
    let module = generator.generate_module();
    let metrics = anvil::metrics::compute(&module);
    let sv_text = anvil::emit::to_sv(&module);
    let sv_hash = hash_bytes(sv_text.as_bytes());
    Ok(PreparedModule {
        paths,
        name: module.name,
        metrics,
        sv_text,
        sv_hash,
    })
}

fn materialize_prepared_module(
    cli: &Cli,
    scenario_dir: &Path,
    prepared: PreparedModule,
    generator_checkpoint: &GeneratorCheckpoint,
    runtime_fingerprint: Option<&str>,
    write_sv: bool,
) -> Result<ModuleReport> {
    if write_sv {
        std::fs::write(&prepared.paths.sv_path, &prepared.sv_text)
            .with_context(|| format!("write {}", prepared.paths.sv_path.display()))?;
    }

    let (verilator, yosys) = run_module_tools(
        cli,
        scenario_dir,
        &prepared.paths.sv_path,
        &prepared.paths.stem,
    )?;

    let report = ModuleReport {
        file: prepared.paths.file.clone(),
        name: prepared.name,
        metrics: prepared.metrics,
        verilator,
        yosys,
    };
    write_module_checkpoint(
        cli,
        &prepared.paths.checkpoint_path,
        &report,
        generator_checkpoint,
        runtime_fingerprint,
        &prepared.sv_hash,
    )?;
    Ok(report)
}

fn run_module_tools(
    cli: &Cli,
    scenario_dir: &Path,
    sv_path: &Path,
    stem: &str,
) -> Result<(Option<ToolInvocation>, Vec<ToolInvocation>)> {
    let verilator = if cli.skip_verilator {
        None
    } else {
        Some(run_verilator(
            &cli.verilator_bin,
            scenario_dir,
            sv_path,
            stem,
        )?)
    };

    let yosys = if cli.skip_yosys {
        Vec::new()
    } else {
        run_yosys(cli.yosys_mode, &cli.yosys_bin, scenario_dir, sv_path, stem)?
    };

    Ok((verilator, yosys))
}

fn write_module_checkpoint(
    cli: &Cli,
    path: &Path,
    report: &ModuleReport,
    generator_checkpoint: &GeneratorCheckpoint,
    runtime_fingerprint: Option<&str>,
    sv_hash: &str,
) -> Result<()> {
    let checkpoint = ModuleCheckpoint {
        skip_verilator: cli.skip_verilator,
        skip_yosys: cli.skip_yosys,
        yosys_mode: yosys_mode_slug(cli.yosys_mode).to_string(),
        runtime_fingerprint: runtime_fingerprint.map(str::to_owned),
        sv_hash: Some(sv_hash.to_string()),
        generator_checkpoint: Some(generator_checkpoint.clone()),
        report: report.clone(),
    };
    std::fs::write(path, serde_json::to_string_pretty(&checkpoint)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn load_module_checkpoint(path: &Path) -> Result<Option<ModuleCheckpoint>> {
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    match serde_json::from_str::<ModuleCheckpoint>(&text) {
        Ok(checkpoint) => Ok(Some(checkpoint)),
        Err(_) => Ok(None),
    }
}

fn checkpoint_matches_cli(checkpoint: &ModuleCheckpoint, cli: &Cli) -> bool {
    checkpoint.skip_verilator == cli.skip_verilator
        && checkpoint.skip_yosys == cli.skip_yosys
        && checkpoint.yosys_mode == yosys_mode_slug(cli.yosys_mode)
}

fn try_fast_resume_checkpoint(
    generator: &mut Generator,
    cli: &Cli,
    paths: &ModulePaths,
    checkpoint: &ModuleCheckpoint,
    runtime_fingerprint: Option<&str>,
) -> Result<Option<ModuleReport>> {
    if !checkpoint_matches_cli(checkpoint, cli) {
        return Ok(None);
    }
    let expected_fingerprint = match runtime_fingerprint {
        Some(fingerprint) => fingerprint,
        None => return Ok(None),
    };
    if checkpoint.runtime_fingerprint.as_deref() != Some(expected_fingerprint) {
        return Ok(None);
    }
    let generator_checkpoint = match checkpoint.generator_checkpoint.as_ref() {
        Some(state) => state,
        None => return Ok(None),
    };
    let expected_sv_hash = match checkpoint.sv_hash.as_deref() {
        Some(hash) => hash,
        None => return Ok(None),
    };
    if !paths.sv_path.exists() {
        return Ok(None);
    }
    if checkpoint.report.file != paths.file {
        return Ok(None);
    }
    let existing_sv_hash = hash_file(&paths.sv_path)?;
    if existing_sv_hash != expected_sv_hash {
        return Ok(None);
    }

    generator.restore_checkpoint(generator_checkpoint);
    Ok(Some(checkpoint.report.clone()))
}

fn validate_checkpoint_against_prepared(
    report: &ModuleReport,
    prepared: &PreparedModule,
) -> Result<()> {
    validate_legacy_sv_against_prepared(prepared)?;
    if report.file != prepared.paths.file {
        bail!(
            "resume mismatch for {}: checkpoint file {}, expected {}",
            prepared.paths.file,
            report.file,
            prepared.paths.file
        );
    }
    if report.name != prepared.name {
        bail!(
            "resume mismatch for {}: checkpoint module {}, expected {}",
            prepared.paths.file,
            report.name,
            prepared.name
        );
    }
    Ok(())
}

fn validate_legacy_sv_against_prepared(prepared: &PreparedModule) -> Result<()> {
    let existing = std::fs::read_to_string(&prepared.paths.sv_path)
        .with_context(|| format!("read {}", prepared.paths.sv_path.display()))?;
    if existing != prepared.sv_text {
        bail!(
            "resume mismatch for {}: existing SV differs from regenerated module",
            prepared.paths.file
        );
    }
    Ok(())
}

fn run_verilator(bin: &str, out_dir: &Path, sv_path: &Path, stem: &str) -> Result<ToolInvocation> {
    run_tool(
        "verilator",
        bin,
        vec!["--lint-only".to_string(), sv_path.display().to_string()],
        out_dir,
        stem,
    )
}

fn run_yosys(
    mode: YosysMode,
    bin: &str,
    out_dir: &Path,
    sv_path: &Path,
    stem: &str,
) -> Result<Vec<ToolInvocation>> {
    let mut invocations = Vec::new();
    for (tool_label, script) in yosys_invocations(mode, sv_path) {
        invocations.push(run_tool(
            tool_label,
            bin,
            vec!["-p".to_string(), script],
            out_dir,
            stem,
        )?);
    }
    Ok(invocations)
}

fn yosys_invocations(mode: YosysMode, sv_path: &Path) -> Vec<(&'static str, String)> {
    let escaped = escape_for_double_quotes(sv_path);
    match mode {
        YosysMode::WithoutAbc => vec![(
            "yosys-without-abc",
            format!("read_verilog -sv \"{escaped}\"; synth -noabc; stat"),
        )],
        YosysMode::WithAbc => vec![(
            "yosys-with-abc",
            format!(
                "read_verilog -sv \"{escaped}\"; synth -noabc; abc -fast; opt -fast; stat; check"
            ),
        )],
        YosysMode::Both => vec![
            (
                "yosys-without-abc",
                format!("read_verilog -sv \"{escaped}\"; synth -noabc; stat"),
            ),
            (
                "yosys-with-abc",
                format!(
                    "read_verilog -sv \"{escaped}\"; synth -noabc; abc -fast; opt -fast; stat; check"
                ),
            ),
        ],
    }
}

fn run_tool(
    tool_name: &str,
    binary: &str,
    argv: Vec<String>,
    out_dir: &Path,
    stem: &str,
) -> Result<ToolInvocation> {
    let output = Command::new(binary).args(&argv).output();
    match output {
        Ok(output) => {
            let warning = first_tool_warning(
                tool_name,
                String::from_utf8_lossy(&output.stdout).as_ref(),
                String::from_utf8_lossy(&output.stderr).as_ref(),
            );
            let success = output.status.success() && warning.is_none();
            let stdout_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stdout",
                &output.stdout,
                !success,
            )?;
            let stderr_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stderr",
                &output.stderr,
                !success,
            )?;
            Ok(ToolInvocation {
                tool: tool_name.to_string(),
                argv: std::iter::once(binary.to_string()).chain(argv).collect(),
                success,
                exit_code: output.status.code(),
                stdout_log,
                stderr_log,
                error: warning,
            })
        }
        Err(err) => Ok(ToolInvocation {
            tool: tool_name.to_string(),
            argv: std::iter::once(binary.to_string()).chain(argv).collect(),
            success: false,
            exit_code: None,
            stdout_log: None,
            stderr_log: None,
            error: Some(err.to_string()),
        }),
    }
}

fn first_tool_warning(tool_name: &str, stdout: &str, stderr: &str) -> Option<String> {
    match tool_name {
        "verilator" => stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim_start)
            .find(|line| line.starts_with("%Warning-"))
            .map(ToOwned::to_owned),
        tool_name if tool_name.starts_with("yosys") => stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim_start)
            .find(|line| line.starts_with("Warning:") || line.contains(": Warning:"))
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn write_tool_log_if_needed(
    out_dir: &Path,
    stem: &str,
    tool_name: &str,
    stream: &str,
    bytes: &[u8],
    always_write_on_failure: bool,
) -> Result<Option<String>> {
    if bytes.is_empty() && !always_write_on_failure {
        return Ok(None);
    }
    let file_name = format!("{stem}.{tool_name}.{stream}.log");
    let path = out_dir.join(&file_name);
    std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(Some(file_name))
}

fn aggregate_metrics(modules: &[ModuleReport]) -> AggregateMetrics {
    let mut aggregate = AggregateMetrics::default();
    for module in modules {
        aggregate.modules += 1;
        aggregate.total_nodes += module.metrics.num_nodes;
        aggregate.total_gates += module.metrics.num_gates;
        aggregate.total_flops += module.metrics.num_flops;
        aggregate.total_shared_nodes += module.metrics.num_shared_nodes;
        aggregate.total_priority_encoder_blocks +=
            u64::from(module.metrics.num_priority_encoder_blocks);
        aggregate.total_comb_muxes_one_hot += u64::from(module.metrics.num_comb_muxes_one_hot);
        aggregate.total_comb_muxes_encoded += u64::from(module.metrics.num_comb_muxes_encoded);
        aggregate.total_case_mux_blocks += u64::from(module.metrics.num_case_mux_blocks);
        aggregate.total_casez_mux_blocks += u64::from(module.metrics.num_casez_mux_blocks);
        aggregate.total_semantic_gates_merged += u64::from(module.metrics.semantic_gates_merged);
        aggregate.total_flops_merged += u64::from(module.metrics.flops_merged);

        merge_usize_count_map_into_u64(&mut aggregate.gates_by_kind, &module.metrics.gates_by_kind);
        merge_count_map(
            &mut aggregate.knob_roll_attempts,
            &module.metrics.knob_roll_attempts,
        );
        merge_count_map(
            &mut aggregate.knob_roll_fires,
            &module.metrics.knob_roll_fires,
        );
    }
    aggregate
}

fn summarize_tools(modules: &[ModuleReport]) -> ToolSummary {
    let mut summary = ToolSummary::default();
    for module in modules {
        if let Some(verilator) = &module.verilator {
            if verilator.success {
                summary.verilator_passed += 1;
            } else {
                summary.verilator_failed += 1;
            }
        }
        for yosys in &module.yosys {
            match yosys.tool.as_str() {
                "yosys-without-abc" => {
                    if yosys.success {
                        summary.yosys_without_abc_passed += 1;
                    } else {
                        summary.yosys_without_abc_failed += 1;
                    }
                }
                "yosys-with-abc" => {
                    if yosys.success {
                        summary.yosys_with_abc_passed += 1;
                    } else {
                        summary.yosys_with_abc_failed += 1;
                    }
                }
                _ => {}
            }
        }
    }
    summary
}

fn summarize_coverage(scenario: &Scenario, modules: &[ModuleReport]) -> CoverageSummary {
    let mut coverage = CoverageSummary::default();
    coverage
        .construction_strategies
        .insert(construction_strategy_slug(scenario.config.construction_strategy).to_string());
    coverage
        .identity_modes
        .insert(identity_mode_slug(scenario.config.identity_mode).to_string());
    coverage
        .factorization_levels
        .insert(factorization_level_slug(scenario.config.factorization_level).to_string());
    coverage
        .share_prob_values
        .insert(share_prob_label(scenario.config.share_prob));

    for module in modules {
        if module.metrics.num_flops == 0 {
            coverage.saw_comb_only_module = true;
        } else {
            coverage.saw_sequential_module = true;
        }

        coverage.saw_priority_encoder |= module.metrics.num_priority_encoder_blocks > 0;
        coverage.saw_comb_mux_one_hot |= module.metrics.num_comb_muxes_one_hot > 0;
        coverage.saw_comb_mux_encoded |= module.metrics.num_comb_muxes_encoded > 0;
        coverage.saw_case_mux |= module.metrics.num_case_mux_blocks > 0;
        coverage.saw_casez_mux |= module.metrics.num_casez_mux_blocks > 0;
        coverage.saw_flop_mux_one_hot |= module.metrics.flops_mux_one_hot > 0;
        coverage.saw_flop_mux_encoded |= module.metrics.flops_mux_encoded > 0;
        coverage.saw_semantic_gate_merge |= module.metrics.semantic_gates_merged > 0;
        coverage.saw_flop_merge |= module.metrics.flops_merged > 0;

        for gate_kind in module.metrics.gates_by_kind.keys() {
            coverage.gate_kinds.insert(gate_kind.clone());
            coverage
                .gate_categories
                .insert(gate_kind_category(gate_kind).to_string());
        }
        for knob in module.metrics.knob_roll_attempts.keys() {
            coverage.knob_attempts_seen.insert(knob.clone());
        }
        for knob in module.metrics.knob_roll_fires.keys() {
            coverage.knob_fires_seen.insert(knob.clone());
        }
    }

    coverage
}

fn merge_tool_summary(dst: &mut ToolSummary, src: &ToolSummary) {
    dst.verilator_passed += src.verilator_passed;
    dst.verilator_failed += src.verilator_failed;
    dst.yosys_without_abc_passed += src.yosys_without_abc_passed;
    dst.yosys_without_abc_failed += src.yosys_without_abc_failed;
    dst.yosys_with_abc_passed += src.yosys_with_abc_passed;
    dst.yosys_with_abc_failed += src.yosys_with_abc_failed;
}

fn merge_coverage(dst: &mut CoverageSummary, src: &CoverageSummary) {
    dst.construction_strategies
        .extend(src.construction_strategies.iter().cloned());
    dst.identity_modes
        .extend(src.identity_modes.iter().cloned());
    dst.factorization_levels
        .extend(src.factorization_levels.iter().cloned());
    dst.share_prob_values
        .extend(src.share_prob_values.iter().cloned());
    dst.gate_categories
        .extend(src.gate_categories.iter().cloned());
    dst.gate_kinds.extend(src.gate_kinds.iter().cloned());
    dst.knob_attempts_seen
        .extend(src.knob_attempts_seen.iter().cloned());
    dst.knob_fires_seen
        .extend(src.knob_fires_seen.iter().cloned());
    dst.saw_comb_only_module |= src.saw_comb_only_module;
    dst.saw_sequential_module |= src.saw_sequential_module;
    dst.saw_priority_encoder |= src.saw_priority_encoder;
    dst.saw_comb_mux_one_hot |= src.saw_comb_mux_one_hot;
    dst.saw_comb_mux_encoded |= src.saw_comb_mux_encoded;
    dst.saw_case_mux |= src.saw_case_mux;
    dst.saw_casez_mux |= src.saw_casez_mux;
    dst.saw_flop_mux_one_hot |= src.saw_flop_mux_one_hot;
    dst.saw_flop_mux_encoded |= src.saw_flop_mux_encoded;
    dst.saw_semantic_gate_merge |= src.saw_semantic_gate_merge;
    dst.saw_flop_merge |= src.saw_flop_merge;
}

fn summarize_share_sweep(scenarios: &[ScenarioReport]) -> ShareSweepSummary {
    let mut summary = ShareSweepSummary::default();
    for scenario in scenarios {
        let share_prob = share_prob_label(scenario.config.share_prob);
        let bucket = summary.buckets.entry(share_prob).or_default();
        bucket.scenarios += 1;
        bucket.modules += scenario.aggregate.modules;
        bucket.total_nodes += scenario.aggregate.total_nodes;
        bucket.total_shared_nodes += scenario.aggregate.total_shared_nodes;
    }
    for bucket in summary.buckets.values_mut() {
        if bucket.modules > 0 {
            bucket.avg_nodes_per_module = bucket.total_nodes as f64 / bucket.modules as f64;
        }
        if bucket.total_nodes > 0 {
            bucket.shared_node_fraction =
                bucket.total_shared_nodes as f64 / bucket.total_nodes as f64;
        }
    }
    summary
}

fn compute_coverage_gaps(
    scenario_set: ScenarioSet,
    coverage: &CoverageSummary,
    share_sweep: Option<&ShareSweepSummary>,
) -> Vec<String> {
    let mut gaps = Vec::new();

    for strategy in ["sequential", "shuffled", "interleaved"] {
        if !coverage.construction_strategies.contains(strategy) {
            gaps.push(format!("missing construction strategy {strategy}"));
        }
    }

    match scenario_set {
        ScenarioSet::Default => {
            for mode in ["relaxed", "node-id"] {
                if !coverage.identity_modes.contains(mode) {
                    gaps.push(format!("missing identity mode {mode}"));
                }
            }
            for level in [
                "none",
                "cse",
                "operand-unique",
                "commutative",
                "associative",
                "constant-fold",
                "peephole",
                "e-graph",
            ] {
                if !coverage.factorization_levels.contains(level) {
                    gaps.push(format!("missing factorization level {level}"));
                }
            }
        }
        ScenarioSet::Phase2Share => {
            if !coverage.identity_modes.contains("node-id") {
                gaps.push("missing identity mode node-id".to_string());
            }
            if !coverage.factorization_levels.contains("e-graph") {
                gaps.push("missing factorization level e-graph".to_string());
            }
            for share_prob in ["0.0", "0.3", "0.9"] {
                if !coverage.share_prob_values.contains(share_prob) {
                    gaps.push(format!("missing share_prob scenario {share_prob}"));
                }
            }
        }
    }

    for category in [
        "arithmetic",
        "bitwise",
        "compare",
        "reduce",
        "shift",
        "structural",
    ] {
        if !coverage.gate_categories.contains(category) {
            gaps.push(format!("missing gate category {category}"));
        }
    }

    if !coverage.saw_comb_only_module {
        gaps.push("matrix never produced a comb-only module".to_string());
    }
    if !coverage.saw_sequential_module {
        gaps.push("matrix never produced a sequential module".to_string());
    }
    if !coverage.saw_priority_encoder {
        gaps.push("matrix never emitted a priority-encoder block".to_string());
    }
    if !coverage.saw_comb_mux_one_hot {
        gaps.push("matrix never emitted a combinational one-hot mux block".to_string());
    }
    if !coverage.saw_comb_mux_encoded {
        gaps.push("matrix never emitted a combinational encoded mux block".to_string());
    }
    if !coverage.saw_case_mux {
        gaps.push("matrix never emitted a combinational case mux block".to_string());
    }
    if !coverage.saw_casez_mux {
        gaps.push("matrix never emitted a combinational casez mux block".to_string());
    }
    if !coverage.saw_flop_mux_one_hot {
        gaps.push("matrix never emitted a one-hot flop mux".to_string());
    }
    if !coverage.saw_flop_mux_encoded {
        gaps.push("matrix never emitted an encoded flop mux".to_string());
    }

    let required_knobs: &[&str] = match scenario_set {
        ScenarioSet::Default => &[
            "comb_mux_prob",
            "case_mux_prob",
            "casez_mux_prob",
            "coefficient_prob",
            "const_comparand_prob",
            "const_shift_amount_prob",
            "flop_prob",
            "priority_encoder_prob",
            "share_prob",
            "terminal_reuse_prob",
        ],
        ScenarioSet::Phase2Share => &["share_prob", "terminal_reuse_prob", "flop_prob"],
    };
    for &knob in required_knobs {
        if !coverage.knob_attempts_seen.contains(knob) {
            gaps.push(format!("matrix never reached decision sites for {knob}"));
        }
    }

    if scenario_set == ScenarioSet::Phase2Share {
        let Some(summary) = share_sweep else {
            gaps.push("phase2-share coverage missing share sweep summary".to_string());
            return gaps;
        };
        let low = summary
            .buckets
            .get("0.0")
            .map(|bucket| bucket.shared_node_fraction);
        let mid = summary
            .buckets
            .get("0.3")
            .map(|bucket| bucket.shared_node_fraction);
        let high = summary
            .buckets
            .get("0.9")
            .map(|bucket| bucket.shared_node_fraction);
        match (low, mid, high) {
            (Some(low), Some(mid), Some(high)) => {
                if !(low < mid && mid < high) {
                    gaps.push(format!(
                        "share sweep did not increase shared-node fraction monotonically: 0.0={low:.4}, 0.3={mid:.4}, 0.9={high:.4}"
                    ));
                }
            }
            _ => gaps
                .push("phase2-share coverage missing one or more share sweep buckets".to_string()),
        }
    }

    gaps
}

fn merge_count_map<T>(dst: &mut BTreeMap<String, T>, src: &BTreeMap<String, T>)
where
    T: Copy + Default + std::ops::AddAssign<T>,
{
    for (key, value) in src {
        let entry = dst.entry(key.clone()).or_default();
        *entry += *value;
    }
}

fn merge_usize_count_map_into_u64(dst: &mut BTreeMap<String, u64>, src: &BTreeMap<String, usize>) {
    for (key, value) in src {
        let entry = dst.entry(key.clone()).or_default();
        *entry += *value as u64;
    }
}

fn gate_kind_category(gate_kind: &str) -> &'static str {
    match gate_kind {
        "and" | "or" | "xor" | "not" => "bitwise",
        "add" | "sub" | "mul" => "arithmetic",
        "eq" | "neq" | "lt" | "gt" | "le" | "ge" => "compare",
        "red_and" | "red_or" | "red_xor" => "reduce",
        "shl" | "shr" => "shift",
        "mux" | "case_mux" | "casez_mux" | "slice" | "concat" => "structural",
        _ => "other",
    }
}

fn construction_strategy_name(strategy: ConstructionStrategy) -> &'static str {
    match strategy {
        ConstructionStrategy::Sequential => "Sequential",
        ConstructionStrategy::Shuffled => "Shuffled",
        ConstructionStrategy::Interleaved | ConstructionStrategy::GraphFirst => "Interleaved",
    }
}

fn construction_strategy_slug(strategy: ConstructionStrategy) -> &'static str {
    match strategy {
        ConstructionStrategy::Sequential => "sequential",
        ConstructionStrategy::Shuffled => "shuffled",
        ConstructionStrategy::Interleaved | ConstructionStrategy::GraphFirst => "interleaved",
    }
}

fn strategy_slug(strategy: ConstructionStrategy) -> &'static str {
    match strategy {
        ConstructionStrategy::Sequential => "seq",
        ConstructionStrategy::Shuffled => "shuf",
        ConstructionStrategy::Interleaved | ConstructionStrategy::GraphFirst => "int",
    }
}

fn identity_mode_slug(mode: IdentityMode) -> &'static str {
    match mode {
        IdentityMode::Relaxed => "relaxed",
        IdentityMode::NodeId => "node-id",
    }
}

fn factorization_level_name(level: FactorizationLevel) -> &'static str {
    match level {
        FactorizationLevel::None => "none",
        FactorizationLevel::Cse => "cse",
        FactorizationLevel::OperandUnique => "operand-unique",
        FactorizationLevel::Commutative => "commutative",
        FactorizationLevel::Associative => "associative",
        FactorizationLevel::ConstantFold => "constant-fold",
        FactorizationLevel::Peephole => "peephole",
        FactorizationLevel::EGraph => "e-graph",
    }
}

fn factorization_level_slug(level: FactorizationLevel) -> &'static str {
    factorization_level_name(level)
}

fn share_prob_label(share_prob: f64) -> String {
    format!("{share_prob:.1}")
}

fn share_prob_slug(share_prob: f64) -> String {
    share_prob_label(share_prob).replace('.', "p")
}

fn scenario_set_slug(scenario_set: ScenarioSet) -> &'static str {
    match scenario_set {
        ScenarioSet::Default => "default",
        ScenarioSet::Phase2Share => "phase2-share",
    }
}

fn yosys_mode_slug(mode: YosysMode) -> &'static str {
    match mode {
        YosysMode::WithoutAbc => "without-abc",
        YosysMode::WithAbc => "with-abc",
        YosysMode::Both => "both",
    }
}

impl ToolSummary {
    fn yosys_failed(&self) -> usize {
        self.yosys_without_abc_failed + self.yosys_with_abc_failed
    }
}

fn escape_for_double_quotes(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_RUNTIME_FINGERPRINT: &str = "test-runtime";

    fn test_cli() -> Cli {
        Cli {
            out: None,
            base_seed: 0,
            modules_per_scenario: 1,
            phase1_gate: false,
            phase2_share_gate: false,
            list_scenarios: false,
            skip_verilator: false,
            skip_yosys: false,
            verilator_bin: "verilator".to_string(),
            yosys_bin: "yosys".to_string(),
            yosys_mode: YosysMode::WithoutAbc,
            fail_on_coverage_gap: false,
            resume: false,
        }
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "anvil-tool-matrix-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn scenario_names_are_unique() {
        let scenarios = build_scenarios(7, ScenarioSet::Default).expect("build scenarios");
        let mut names = BTreeSet::new();
        for scenario in scenarios {
            assert!(names.insert(scenario.name));
        }
    }

    #[test]
    fn matrix_covers_every_factorization_rung() {
        let scenarios = build_scenarios(0, ScenarioSet::Default).expect("build scenarios");
        let mut levels = BTreeSet::new();
        let mut saw_relaxed = false;
        for scenario in scenarios {
            if scenario.config.identity_mode == IdentityMode::Relaxed {
                saw_relaxed = true;
            }
            levels.insert(factorization_level_slug(
                scenario.config.factorization_level,
            ));
        }
        assert!(saw_relaxed);
        assert_eq!(
            levels,
            BTreeSet::from([
                "none",
                "cse",
                "operand-unique",
                "commutative",
                "associative",
                "constant-fold",
                "peephole",
                "e-graph",
            ])
        );
    }

    #[test]
    fn matrix_covers_all_construction_strategies() {
        let scenarios = build_scenarios(0, ScenarioSet::Default).expect("build scenarios");
        let mut strategies = BTreeSet::new();
        for scenario in scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
        }
        assert_eq!(
            strategies,
            BTreeSet::from(["interleaved", "sequential", "shuffled"])
        );
    }

    #[test]
    fn coverage_gaps_detect_missing_categories() {
        let coverage = CoverageSummary::default();
        let gaps = compute_coverage_gaps(ScenarioSet::Default, &coverage, None);
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("missing construction strategy sequential")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("missing gate category arithmetic")));
        assert!(gaps.iter().any(|gap| gap.contains("priority-encoder")));
    }

    #[test]
    fn phase1_gate_raises_modules_per_scenario_to_cover_1000_modules() {
        let mut cli = test_cli();
        cli.phase1_gate = true;

        let plan = derive_run_plan(&cli, 15);
        assert_eq!(plan.modules_per_scenario, 67);
        assert_eq!(plan.total_modules, 1005);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase1_gate_preserves_larger_explicit_module_count() {
        let mut cli = test_cli();
        cli.phase1_gate = true;
        cli.modules_per_scenario = 100;

        let plan = derive_run_plan(&cli, 15);
        assert_eq!(plan.modules_per_scenario, 100);
        assert_eq!(plan.total_modules, 1500);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase2_share_gate_raises_modules_per_scenario_for_share_sweep() {
        let mut cli = test_cli();
        cli.phase2_share_gate = true;

        let plan = derive_run_plan(&cli, 18);
        assert_eq!(plan.modules_per_scenario, 12);
        assert_eq!(plan.total_modules, 216);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase2_share_matrix_covers_requested_share_prob_levels() {
        let scenarios = build_scenarios(0, ScenarioSet::Phase2Share).expect("build scenarios");
        let mut share_probs = BTreeSet::new();
        let mut strategies = BTreeSet::new();
        for scenario in &scenarios {
            share_probs.insert(share_prob_label(scenario.config.share_prob));
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            assert_eq!(scenario.config.identity_mode, IdentityMode::NodeId);
            assert_eq!(
                scenario.config.factorization_level,
                FactorizationLevel::EGraph
            );
        }
        assert_eq!(scenarios.len(), 18);
        assert_eq!(
            share_probs,
            BTreeSet::from(["0.0".to_string(), "0.3".to_string(), "0.9".to_string()])
        );
        assert_eq!(
            strategies,
            BTreeSet::from(["interleaved", "sequential", "shuffled"])
        );
    }

    #[test]
    fn phase2_share_coverage_requires_monotonic_shared_node_fraction() {
        let coverage = CoverageSummary {
            construction_strategies: BTreeSet::from([
                "interleaved".to_string(),
                "sequential".to_string(),
                "shuffled".to_string(),
            ]),
            identity_modes: BTreeSet::from(["node-id".to_string()]),
            factorization_levels: BTreeSet::from(["e-graph".to_string()]),
            share_prob_values: BTreeSet::from([
                "0.0".to_string(),
                "0.3".to_string(),
                "0.9".to_string(),
            ]),
            gate_categories: BTreeSet::from([
                "arithmetic".to_string(),
                "bitwise".to_string(),
                "compare".to_string(),
                "reduce".to_string(),
                "shift".to_string(),
                "structural".to_string(),
            ]),
            knob_attempts_seen: BTreeSet::from([
                "share_prob".to_string(),
                "terminal_reuse_prob".to_string(),
                "flop_prob".to_string(),
            ]),
            saw_comb_only_module: true,
            saw_sequential_module: true,
            saw_priority_encoder: true,
            saw_comb_mux_one_hot: true,
            saw_comb_mux_encoded: true,
            saw_case_mux: true,
            saw_casez_mux: true,
            saw_flop_mux_one_hot: true,
            saw_flop_mux_encoded: true,
            ..CoverageSummary::default()
        };
        let summary = ShareSweepSummary {
            buckets: BTreeMap::from([
                (
                    "0.0".to_string(),
                    ShareSweepBucket {
                        scenarios: 6,
                        modules: 72,
                        total_nodes: 7200,
                        total_shared_nodes: 720,
                        avg_nodes_per_module: 100.0,
                        shared_node_fraction: 0.1000,
                    },
                ),
                (
                    "0.3".to_string(),
                    ShareSweepBucket {
                        scenarios: 6,
                        modules: 72,
                        total_nodes: 7200,
                        total_shared_nodes: 648,
                        avg_nodes_per_module: 100.0,
                        shared_node_fraction: 0.0900,
                    },
                ),
                (
                    "0.9".to_string(),
                    ShareSweepBucket {
                        scenarios: 6,
                        modules: 72,
                        total_nodes: 7200,
                        total_shared_nodes: 1008,
                        avg_nodes_per_module: 100.0,
                        shared_node_fraction: 0.1400,
                    },
                ),
            ]),
        };

        let gaps = compute_coverage_gaps(ScenarioSet::Phase2Share, &coverage, Some(&summary));
        assert!(gaps
            .iter()
            .any(|gap| gap
                .contains("share sweep did not increase shared-node fraction monotonically")));
    }

    #[test]
    fn yosys_mode_expands_to_expected_invocations() {
        let path = Path::new("/tmp/example.sv");

        let without = yosys_invocations(YosysMode::WithoutAbc, path);
        assert_eq!(without.len(), 1);
        assert_eq!(without[0].0, "yosys-without-abc");
        assert!(without[0].1.contains("synth -noabc; stat"));

        let with = yosys_invocations(YosysMode::WithAbc, path);
        assert_eq!(with.len(), 1);
        assert_eq!(with[0].0, "yosys-with-abc");
        assert!(with[0]
            .1
            .contains("synth -noabc; abc -fast; opt -fast; stat; check"));
        assert!(with[0].1.contains("abc -fast"));

        let both = yosys_invocations(YosysMode::Both, path);
        assert_eq!(both.len(), 2);
        assert_eq!(both[0].0, "yosys-without-abc");
        assert_eq!(both[1].0, "yosys-with-abc");
    }

    #[test]
    fn summarize_tools_counts_yosys_modes_separately() {
        let modules = vec![ModuleReport {
            file: "mod.sv".to_string(),
            name: "mod_0_0000".to_string(),
            metrics: Metrics::default(),
            verilator: Some(ToolInvocation {
                tool: "verilator".to_string(),
                argv: vec![],
                success: true,
                exit_code: Some(0),
                stdout_log: None,
                stderr_log: None,
                error: None,
            }),
            yosys: vec![
                ToolInvocation {
                    tool: "yosys-without-abc".to_string(),
                    argv: vec![],
                    success: true,
                    exit_code: Some(0),
                    stdout_log: None,
                    stderr_log: None,
                    error: None,
                },
                ToolInvocation {
                    tool: "yosys-with-abc".to_string(),
                    argv: vec![],
                    success: false,
                    exit_code: Some(1),
                    stdout_log: None,
                    stderr_log: Some("stderr.log".to_string()),
                    error: Some("ABC: Warning: example".to_string()),
                },
            ],
        }];

        let summary = summarize_tools(&modules);
        assert_eq!(summary.verilator_passed, 1);
        assert_eq!(summary.yosys_without_abc_passed, 1);
        assert_eq!(summary.yosys_with_abc_failed, 1);
        assert_eq!(summary.yosys_failed(), 1);
    }

    #[test]
    fn fast_resume_restores_generator_state_for_next_module() {
        let out_root = temp_test_dir("resume-fast");
        let scenario = make_scenario(
            "resume_fast_case",
            "resume fast path test",
            relaxed_default_config(ConstructionStrategy::Interleaved, 17),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let cli = test_cli_resume();
        let mut baseline = Generator::new(scenario.config.clone());
        let prepared0 = prepare_module(&mut baseline, &scenario, &scenario_dir, 0).unwrap();
        fs::write(&prepared0.paths.sv_path, &prepared0.sv_text).unwrap();
        let report0 = ModuleReport {
            file: prepared0.paths.file.clone(),
            name: prepared0.name.clone(),
            metrics: prepared0.metrics.clone(),
            verilator: None,
            yosys: vec![],
        };
        let checkpoint0 = baseline.checkpoint();
        write_module_checkpoint(
            &cli,
            &prepared0.paths.checkpoint_path,
            &report0,
            &checkpoint0,
            Some(TEST_RUNTIME_FINGERPRINT),
            &prepared0.sv_hash,
        )
        .unwrap();

        let expected1 = prepare_module(&mut baseline, &scenario, &scenario_dir, 1).unwrap();

        let paths0 = module_paths(&scenario_dir, scenario.config.seed, 0).unwrap();
        let checkpoint = load_module_checkpoint(&paths0.checkpoint_path)
            .unwrap()
            .expect("checkpoint");
        let mut resumed = Generator::new(scenario.config.clone());
        let report = try_fast_resume_checkpoint(
            &mut resumed,
            &cli,
            &paths0,
            &checkpoint,
            Some(TEST_RUNTIME_FINGERPRINT),
        )
        .unwrap();
        assert!(report.is_some());

        let actual1 = prepare_module(&mut resumed, &scenario, &scenario_dir, 1).unwrap();
        assert_eq!(actual1.sv_text, expected1.sv_text);

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn fast_resume_rejects_sv_hash_mismatch() {
        let out_root = temp_test_dir("resume-fast-mismatch");
        let scenario = make_scenario(
            "resume_fast_mismatch_case",
            "resume fast path mismatch test",
            relaxed_default_config(ConstructionStrategy::Interleaved, 19),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let cli = test_cli_resume();
        let mut generator = Generator::new(scenario.config.clone());
        let prepared = prepare_module(&mut generator, &scenario, &scenario_dir, 0).unwrap();
        fs::write(&prepared.paths.sv_path, "// tampered\n").unwrap();
        let report = ModuleReport {
            file: prepared.paths.file.clone(),
            name: prepared.name.clone(),
            metrics: prepared.metrics.clone(),
            verilator: None,
            yosys: vec![],
        };
        let checkpoint = generator.checkpoint();
        write_module_checkpoint(
            &cli,
            &prepared.paths.checkpoint_path,
            &report,
            &checkpoint,
            Some(TEST_RUNTIME_FINGERPRINT),
            &prepared.sv_hash,
        )
        .unwrap();

        let paths = module_paths(&scenario_dir, scenario.config.seed, 0).unwrap();
        let checkpoint = load_module_checkpoint(&paths.checkpoint_path)
            .unwrap()
            .expect("checkpoint");
        let mut resumed = Generator::new(scenario.config.clone());
        let fast_path = try_fast_resume_checkpoint(
            &mut resumed,
            &cli,
            &paths,
            &checkpoint,
            Some(TEST_RUNTIME_FINGERPRINT),
        )
        .unwrap();
        assert!(fast_path.is_none());

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn resume_uses_checkpointed_modules_and_generates_the_rest() {
        let out_root = temp_test_dir("resume-checkpoint");
        let scenario = make_scenario(
            "resume_case",
            "resume test",
            relaxed_default_config(ConstructionStrategy::Interleaved, 11),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let mut generator = Generator::new(scenario.config.clone());
        for module_index in 0..2 {
            let prepared =
                prepare_module(&mut generator, &scenario, &scenario_dir, module_index).unwrap();
            fs::write(&prepared.paths.sv_path, &prepared.sv_text).unwrap();
            let report = ModuleReport {
                file: prepared.paths.file.clone(),
                name: prepared.name,
                metrics: prepared.metrics,
                verilator: None,
                yosys: vec![],
            };
            let legacy_checkpoint = serde_json::json!({
                "skip_verilator": true,
                "skip_yosys": true,
                "yosys_mode": yosys_mode_slug(YosysMode::WithoutAbc),
                "report": report,
            });
            fs::write(
                &prepared.paths.checkpoint_path,
                serde_json::to_string_pretty(&legacy_checkpoint).unwrap(),
            )
            .unwrap();
        }

        let cli = test_cli_resume();
        let plan = RunPlan {
            modules_per_scenario: 3,
            fail_on_coverage_gap: false,
            total_modules: 3,
        };
        let report = run_scenario(
            &scenario,
            &cli,
            &plan,
            &out_root,
            Some(TEST_RUNTIME_FINGERPRINT),
        )
        .expect("run scenario");

        assert_eq!(report.modules.len(), 3);
        assert!(scenario_dir.join("mod_11_0000.module-report.json").exists());
        assert!(scenario_dir.join("mod_11_0001.module-report.json").exists());
        assert!(scenario_dir.join("mod_11_0002.module-report.json").exists());
        let upgraded = load_module_checkpoint(&scenario_dir.join("mod_11_0000.module-report.json"))
            .unwrap()
            .expect("upgraded checkpoint");
        assert!(upgraded.generator_checkpoint.is_some());
        assert_eq!(
            upgraded.runtime_fingerprint.as_deref(),
            Some(TEST_RUNTIME_FINGERPRINT)
        );
        assert!(upgraded.sv_hash.is_some());

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn resume_bootstraps_legacy_sv_without_checkpoint() {
        let out_root = temp_test_dir("resume-legacy");
        let scenario = make_scenario(
            "legacy_case",
            "legacy resume test",
            relaxed_default_config(ConstructionStrategy::Interleaved, 13),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let mut generator = Generator::new(scenario.config.clone());
        let prepared = prepare_module(&mut generator, &scenario, &scenario_dir, 0).unwrap();
        fs::write(&prepared.paths.sv_path, &prepared.sv_text).unwrap();

        let cli = test_cli_resume();
        let plan = RunPlan {
            modules_per_scenario: 2,
            fail_on_coverage_gap: false,
            total_modules: 2,
        };
        let report = run_scenario(
            &scenario,
            &cli,
            &plan,
            &out_root,
            Some(TEST_RUNTIME_FINGERPRINT),
        )
        .expect("run scenario");

        assert_eq!(report.modules.len(), 2);
        assert!(scenario_dir.join("mod_13_0000.module-report.json").exists());
        assert!(scenario_dir.join("mod_13_0001.module-report.json").exists());

        let _ = fs::remove_dir_all(out_root);
    }

    fn test_cli_resume() -> Cli {
        let mut cli = test_cli();
        cli.skip_verilator = true;
        cli.skip_yosys = true;
        cli.resume = true;
        cli
    }
}
