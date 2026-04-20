use anvil::config::{ConstructionStrategy, FactorizationLevel, IdentityMode};
use anvil::metrics::Metrics;
use anvil::{Config, Generator};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

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

    /// Return non-zero if the matrix misses intended coverage.
    #[arg(long)]
    fail_on_coverage_gap: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Scenario {
    name: String,
    description: String,
    config: Config,
}

#[derive(Debug, Clone, Serialize)]
struct ToolInvocation {
    tool: String,
    argv: Vec<String>,
    success: bool,
    exit_code: Option<i32>,
    stdout_log: Option<String>,
    stderr_log: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ModuleReport {
    file: String,
    name: String,
    metrics: Metrics,
    verilator: Option<ToolInvocation>,
    yosys: Option<ToolInvocation>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct ToolSummary {
    verilator_passed: usize,
    verilator_failed: usize,
    yosys_passed: usize,
    yosys_failed: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
struct AggregateMetrics {
    modules: usize,
    total_nodes: usize,
    total_gates: usize,
    total_flops: usize,
    total_priority_encoder_blocks: u64,
    total_comb_muxes_one_hot: u64,
    total_comb_muxes_encoded: u64,
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
    gate_categories: BTreeSet<String>,
    gate_kinds: BTreeSet<String>,
    knob_attempts_seen: BTreeSet<String>,
    knob_fires_seen: BTreeSet<String>,
    saw_comb_only_module: bool,
    saw_sequential_module: bool,
    saw_priority_encoder: bool,
    saw_comb_mux_one_hot: bool,
    saw_comb_mux_encoded: bool,
    saw_flop_mux_one_hot: bool,
    saw_flop_mux_encoded: bool,
    saw_semantic_gate_merge: bool,
    saw_flop_merge: bool,
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
    coverage: CoverageSummary,
    coverage_gaps: Vec<String>,
    tool_summary: ToolSummary,
    scenarios: Vec<ScenarioReport>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.modules_per_scenario == 0 {
        bail!("--modules-per-scenario must be >= 1");
    }

    let scenarios = build_scenarios(cli.base_seed)?;
    if cli.list_scenarios {
        for scenario in &scenarios {
            println!("{}: {}", scenario.name, scenario.description);
        }
        return Ok(());
    }

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
        let report = run_scenario(&scenario, &cli, out_dir)?;
        merge_tool_summary(&mut global_tool_summary, &report.tool_summary);
        merge_coverage(&mut global_coverage, &report.coverage);
        scenario_reports.push(report);
    }

    let coverage_gaps = compute_coverage_gaps(&global_coverage);
    let report = MatrixReport {
        base_seed: cli.base_seed,
        modules_per_scenario: cli.modules_per_scenario,
        scenario_count: scenario_reports.len(),
        coverage: global_coverage,
        coverage_gaps,
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
    println!(
        "tool_matrix: Verilator pass/fail = {}/{}, Yosys pass/fail = {}/{}",
        report.tool_summary.verilator_passed,
        report.tool_summary.verilator_failed,
        report.tool_summary.yosys_passed,
        report.tool_summary.yosys_failed
    );
    if !report.coverage_gaps.is_empty() {
        println!(
            "tool_matrix: coverage gaps detected ({}): {}",
            report.coverage_gaps.len(),
            report.coverage_gaps.join("; ")
        );
    }

    if report.tool_summary.verilator_failed > 0 || report.tool_summary.yosys_failed > 0 {
        bail!(
            "tool_matrix detected downstream-tool failures; see {}",
            report_path.display()
        );
    }
    if cli.fail_on_coverage_gap && !report.coverage_gaps.is_empty() {
        bail!(
            "tool_matrix detected coverage gaps; see {}",
            report_path.display()
        );
    }

    Ok(())
}

fn build_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
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
            share_heavy_comb_only_config(strategy, next_seed),
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
            motif_heavy_sequential_config(strategy, next_seed),
        )?);
        next_seed += 1;
    }

    let mut seen = BTreeSet::new();
    for scenario in &scenarios {
        if !seen.insert(scenario.name.clone()) {
            bail!("duplicate scenario name {}", scenario.name);
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

fn share_heavy_comb_only_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        flop_prob: 0.0,
        share_prob: 0.9,
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

fn motif_heavy_sequential_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        flop_prob: 0.45,
        share_prob: 0.4,
        terminal_reuse_prob: 0.6,
        constant_prob: 0.15,
        coefficient_prob: 0.6,
        const_shift_amount_prob: 0.95,
        const_comparand_prob: 0.75,
        priority_encoder_prob: 0.25,
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

fn run_scenario(scenario: &Scenario, cli: &Cli, out_root: &Path) -> Result<ScenarioReport> {
    let scenario_dir = out_root.join(&scenario.name);
    std::fs::create_dir_all(&scenario_dir)
        .with_context(|| format!("create scenario directory {}", scenario_dir.display()))?;

    let mut generator = Generator::new(scenario.config.clone());
    let mut modules = Vec::with_capacity(cli.modules_per_scenario);

    for module_index in 0..cli.modules_per_scenario {
        let module = generator.generate_module();
        let metrics = anvil::metrics::compute(&module);
        let file = format!("mod_{}_{:04}.sv", scenario.config.seed, module_index);
        let sv_path = scenario_dir.join(&file);
        std::fs::write(&sv_path, anvil::emit::to_sv(&module))
            .with_context(|| format!("write {}", sv_path.display()))?;

        let stem = sv_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("scenario file stem not valid UTF-8")?;

        let verilator = if cli.skip_verilator {
            None
        } else {
            Some(run_verilator(
                &cli.verilator_bin,
                &scenario_dir,
                &sv_path,
                stem,
            )?)
        };

        let yosys = if cli.skip_yosys {
            None
        } else {
            Some(run_yosys(&cli.yosys_bin, &scenario_dir, &sv_path, stem)?)
        };

        modules.push(ModuleReport {
            file,
            name: module.name,
            metrics,
            verilator,
            yosys,
        });
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

fn run_verilator(bin: &str, out_dir: &Path, sv_path: &Path, stem: &str) -> Result<ToolInvocation> {
    run_tool(
        "verilator",
        bin,
        vec!["--lint-only".to_string(), sv_path.display().to_string()],
        out_dir,
        stem,
    )
}

fn run_yosys(bin: &str, out_dir: &Path, sv_path: &Path, stem: &str) -> Result<ToolInvocation> {
    let script = format!(
        "read_verilog -sv \"{}\"; synth; stat",
        escape_for_double_quotes(sv_path)
    );
    run_tool("yosys", bin, vec!["-p".to_string(), script], out_dir, stem)
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
            let stdout_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stdout",
                &output.stdout,
                !output.status.success(),
            )?;
            let stderr_log = write_tool_log_if_needed(
                out_dir,
                stem,
                tool_name,
                "stderr",
                &output.stderr,
                !output.status.success(),
            )?;
            Ok(ToolInvocation {
                tool: tool_name.to_string(),
                argv: std::iter::once(binary.to_string()).chain(argv).collect(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout_log,
                stderr_log,
                error: None,
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
        aggregate.total_priority_encoder_blocks +=
            u64::from(module.metrics.num_priority_encoder_blocks);
        aggregate.total_comb_muxes_one_hot += u64::from(module.metrics.num_comb_muxes_one_hot);
        aggregate.total_comb_muxes_encoded += u64::from(module.metrics.num_comb_muxes_encoded);
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
        if let Some(yosys) = &module.yosys {
            if yosys.success {
                summary.yosys_passed += 1;
            } else {
                summary.yosys_failed += 1;
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

    for module in modules {
        if module.metrics.num_flops == 0 {
            coverage.saw_comb_only_module = true;
        } else {
            coverage.saw_sequential_module = true;
        }

        coverage.saw_priority_encoder |= module.metrics.num_priority_encoder_blocks > 0;
        coverage.saw_comb_mux_one_hot |= module.metrics.num_comb_muxes_one_hot > 0;
        coverage.saw_comb_mux_encoded |= module.metrics.num_comb_muxes_encoded > 0;
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
    dst.yosys_passed += src.yosys_passed;
    dst.yosys_failed += src.yosys_failed;
}

fn merge_coverage(dst: &mut CoverageSummary, src: &CoverageSummary) {
    dst.construction_strategies
        .extend(src.construction_strategies.iter().cloned());
    dst.identity_modes
        .extend(src.identity_modes.iter().cloned());
    dst.factorization_levels
        .extend(src.factorization_levels.iter().cloned());
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
    dst.saw_flop_mux_one_hot |= src.saw_flop_mux_one_hot;
    dst.saw_flop_mux_encoded |= src.saw_flop_mux_encoded;
    dst.saw_semantic_gate_merge |= src.saw_semantic_gate_merge;
    dst.saw_flop_merge |= src.saw_flop_merge;
}

fn compute_coverage_gaps(coverage: &CoverageSummary) -> Vec<String> {
    let mut gaps = Vec::new();

    for strategy in ["sequential", "shuffled", "interleaved"] {
        if !coverage.construction_strategies.contains(strategy) {
            gaps.push(format!("missing construction strategy {strategy}"));
        }
    }
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
    if !coverage.saw_flop_mux_one_hot {
        gaps.push("matrix never emitted a one-hot flop mux".to_string());
    }
    if !coverage.saw_flop_mux_encoded {
        gaps.push("matrix never emitted an encoded flop mux".to_string());
    }

    for knob in [
        "comb_mux_prob",
        "coefficient_prob",
        "const_comparand_prob",
        "const_shift_amount_prob",
        "flop_prob",
        "priority_encoder_prob",
        "share_prob",
        "terminal_reuse_prob",
    ] {
        if !coverage.knob_attempts_seen.contains(knob) {
            gaps.push(format!("matrix never reached decision sites for {knob}"));
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
        "mux" | "slice" | "concat" => "structural",
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

fn escape_for_double_quotes(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_names_are_unique() {
        let scenarios = build_scenarios(7).expect("build scenarios");
        let mut names = BTreeSet::new();
        for scenario in scenarios {
            assert!(names.insert(scenario.name));
        }
    }

    #[test]
    fn matrix_covers_every_factorization_rung() {
        let scenarios = build_scenarios(0).expect("build scenarios");
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
        let scenarios = build_scenarios(0).expect("build scenarios");
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
        let gaps = compute_coverage_gaps(&coverage);
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("missing construction strategy sequential")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("missing gate category arithmetic")));
        assert!(gaps.iter().any(|gap| gap.contains("priority-encoder")));
    }
}
