use anvil::config::{
    ConstructionStrategy, CountRange, FactorizationLevel, HierarchyChildSourceMode, IdentityMode,
};
use anvil::metrics::{DesignMetrics, Metrics};
use anvil::{Config, Design, Generator, GeneratorCheckpoint};
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
const PHASE3_STRUCTURED_MIN_TOTAL_MODULES: usize = 210;
const PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO: usize = 4;

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

    /// Elevate the run to the repo-owned Phase 3 structured-surface
    /// gate: run the representative structured-surface matrix and
    /// require its coverage.
    #[arg(long)]
    phase3_structured_gate: bool,

    /// Elevate the run to the repo-owned Phase 4 hierarchy gate:
    /// run the representative hierarchy matrix and require its
    /// coverage.
    #[arg(long)]
    phase4_hierarchy_gate: bool,

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
struct EmittedModuleReport {
    file: String,
    name: String,
    metrics: Metrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HierarchyFacts {
    library_modules: usize,
    top_instances: usize,
    unique_instantiated_modules: usize,
    reused_child_definition: bool,
    underinstantiated_library: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesignReport {
    index: usize,
    top: String,
    files: Vec<String>,
    modules: Vec<EmittedModuleReport>,
    #[serde(default)]
    hierarchy: HierarchyFacts,
    #[serde(default)]
    metrics: DesignMetrics,
    verilator: Option<ToolInvocation>,
    yosys: Vec<ToolInvocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesignFileHash {
    file: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesignCheckpoint {
    skip_verilator: bool,
    skip_yosys: bool,
    yosys_mode: String,
    runtime_fingerprint: Option<String>,
    files: Vec<DesignFileHash>,
    generator_checkpoint: Option<GeneratorCheckpoint>,
    report: DesignReport,
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
    total_for_fold_blocks: u64,
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
    hierarchy_depths: BTreeSet<String>,
    hierarchy_leaf_module_counts: BTreeSet<String>,
    hierarchy_child_instance_counts: BTreeSet<String>,
    hierarchy_child_source_modes: BTreeSet<String>,
    hierarchy_child_instance_override_profiles: BTreeSet<String>,
    gate_categories: BTreeSet<String>,
    gate_kinds: BTreeSet<String>,
    knob_attempts_seen: BTreeSet<String>,
    knob_fires_seen: BTreeSet<String>,
    saw_hierarchy_design: bool,
    saw_multifile_design: bool,
    saw_instance_module: bool,
    saw_instance_output_node: bool,
    saw_reused_child_definition: bool,
    saw_underinstantiated_library: bool,
    saw_on_demand_child_sourcing: bool,
    saw_profiled_child_interface_synthesis: bool,
    saw_hierarchy_sibling_routing: bool,
    saw_hierarchy_registered_sibling_routing: bool,
    saw_hierarchy_registered_sibling_mixed_support_routing: bool,
    saw_recursive_hierarchy_registered_sibling_mixed_support_routing: bool,
    saw_hierarchy_direct_sibling_parent_cone_instance_routing: bool,
    saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing: bool,
    saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing: bool,
    saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing: bool,
    saw_hierarchy_registered_parent_composed_routing: bool,
    saw_hierarchy_registered_mixed_support_routing: bool,
    saw_recursive_hierarchy_registered_mixed_support_routing: bool,
    saw_hierarchy_registered_multistage_routing: bool,
    saw_recursive_hierarchy_registered_multistage_routing: bool,
    saw_recursive_hierarchy_registered_multistage_mixed_support_routing: bool,
    saw_hierarchy_registered_multistage_sibling_routing: bool,
    saw_recursive_hierarchy_registered_multistage_sibling_routing: bool,
    saw_hierarchy_registered_multistage_parent_cone_instance_routing: bool,
    saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing: bool,
    saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing: bool,
    saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing:
        bool,
    saw_hierarchy_parent_composed_parent_cone_instance_flop_routing: bool,
    saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing: bool,
    saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing: bool,
    saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing: bool,
    saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing: bool,
    saw_hierarchy_registered_parent_cone_instance_routing: bool,
    saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing: bool,
    saw_hierarchy_parent_composed_child_inputs: bool,
    saw_hierarchy_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_mixed_support_child_inputs: bool,
    saw_hierarchy_parent_cone_instance_routing: bool,
    saw_hierarchy_parent_cone_instance_mixed_support_routing: bool,
    saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing: bool,
    saw_hierarchy_parent_cone_instance_outputs: bool,
    saw_recursive_hierarchy_parent_cone_instance_outputs: bool,
    saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs: bool,
    saw_hierarchy_parent_cone_instance_flop_outputs: bool,
    saw_recursive_hierarchy_parent_cone_instance_flop_outputs: bool,
    saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs: bool,
    saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs: bool,
    saw_multiple_parent_cone_instances_per_parent: bool,
    saw_recursive_multiple_parent_cone_instances_per_parent: bool,
    saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs: bool,
    saw_recursive_multiple_parent_cone_instances_per_parent_through_flops: bool,
    saw_hierarchy_parent_local_flops: bool,
    saw_recursive_hierarchy: bool,
    saw_per_depth_branching_metrics: bool,
    saw_mixed_leaf_depth_hierarchy: bool,
    saw_hierarchy_parent_composition: bool,
    saw_hierarchy_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_3_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_3_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_3_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_4_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_4_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_4_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_5_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_5_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_5_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_6_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_6_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_6_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_7_parent_local_flops: bool,
    saw_recursive_hierarchy_depth_7_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_depth_7_parent_port_composed_outputs: bool,
    saw_comb_only_module: bool,
    saw_sequential_module: bool,
    saw_priority_encoder: bool,
    saw_comb_mux_one_hot: bool,
    saw_comb_mux_encoded: bool,
    saw_case_mux: bool,
    saw_casez_mux: bool,
    saw_for_fold: bool,
    saw_variable_shift: bool,
    saw_flop_mux_one_hot: bool,
    saw_flop_mux_encoded: bool,
    saw_semantic_gate_merge: bool,
    saw_flop_merge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioSet {
    Default,
    Phase2Share,
    Phase3Structured,
    Phase4Hierarchy,
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
    artifact_kind: String,
    aggregate: AggregateMetrics,
    coverage: CoverageSummary,
    tool_summary: ToolSummary,
    modules: Vec<ModuleReport>,
    designs: Vec<DesignReport>,
}

#[derive(Debug, Clone, Serialize)]
struct MatrixReport {
    base_seed: u64,
    modules_per_scenario: usize,
    scenario_count: usize,
    total_modules: usize,
    scenario_set: String,
    artifact_kind: String,
    phase1_gate: bool,
    phase2_share_gate: bool,
    phase3_structured_gate: bool,
    phase4_hierarchy_gate: bool,
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

#[derive(Debug, Clone)]
struct DesignPaths {
    checkpoint_path: PathBuf,
}

#[derive(Debug, Clone)]
struct PreparedEmittedModule {
    file: String,
    name: String,
    metrics: Metrics,
    sv_path: PathBuf,
    sv_text: String,
    sv_hash: String,
}

#[derive(Debug, Clone)]
struct PreparedDesign {
    paths: DesignPaths,
    index: usize,
    top: String,
    hierarchy: HierarchyFacts,
    metrics: DesignMetrics,
    modules: Vec<PreparedEmittedModule>,
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
        artifact_kind: artifact_kind_slug(scenario_set).to_string(),
        phase1_gate: cli.phase1_gate,
        phase2_share_gate: cli.phase2_share_gate,
        phase3_structured_gate: cli.phase3_structured_gate,
        phase4_hierarchy_gate: cli.phase4_hierarchy_gate,
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
        "tool_matrix: {} scenarios, {} {}/scenario, report {}",
        report.scenario_count,
        report.modules_per_scenario,
        report.artifact_kind,
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
    } else if cli.phase3_structured_gate {
        PHASE3_STRUCTURED_MIN_TOTAL_MODULES.div_ceil(scenario_count)
    } else if cli.phase4_hierarchy_gate {
        PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO
    } else {
        1
    };
    let modules_per_scenario = cli.modules_per_scenario.max(gate_modules_per_scenario);
    let total_modules = modules_per_scenario * scenario_count;
    RunPlan {
        modules_per_scenario,
        fail_on_coverage_gap: cli.fail_on_coverage_gap
            || cli.phase1_gate
            || cli.phase2_share_gate
            || cli.phase3_structured_gate
            || cli.phase4_hierarchy_gate,
        total_modules,
    }
}

fn select_scenario_set(cli: &Cli) -> Result<ScenarioSet> {
    let enabled_gates = usize::from(cli.phase1_gate)
        + usize::from(cli.phase2_share_gate)
        + usize::from(cli.phase3_structured_gate)
        + usize::from(cli.phase4_hierarchy_gate);
    if enabled_gates > 1 {
        bail!(
            "--phase1-gate, --phase2-share-gate, --phase3-structured-gate, and --phase4-hierarchy-gate are mutually exclusive"
        );
    }
    if cli.phase2_share_gate {
        Ok(ScenarioSet::Phase2Share)
    } else if cli.phase3_structured_gate {
        Ok(ScenarioSet::Phase3Structured)
    } else if cli.phase4_hierarchy_gate {
        Ok(ScenarioSet::Phase4Hierarchy)
    } else {
        Ok(ScenarioSet::Default)
    }
}

fn build_scenarios(base_seed: u64, scenario_set: ScenarioSet) -> Result<Vec<Scenario>> {
    let scenarios = match scenario_set {
        ScenarioSet::Default => build_default_scenarios(base_seed)?,
        ScenarioSet::Phase2Share => build_phase2_share_scenarios(base_seed)?,
        ScenarioSet::Phase3Structured => build_phase3_structured_scenarios(base_seed)?,
        ScenarioSet::Phase4Hierarchy => build_phase4_hierarchy_scenarios(base_seed)?,
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

fn build_phase3_structured_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ] {
        let strategy_label = construction_strategy_name(strategy);
        let strategy_slug = strategy_slug(strategy);

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_comb_mux"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused combinational mux surface."
            ),
            phase3_comb_mux_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_case_mux"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused procedural case-mux surface."
            ),
            phase3_case_mux_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_casez_mux"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused procedural casez-mux surface."
            ),
            phase3_casez_mux_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_for_fold"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused bounded for-fold surface."
            ),
            phase3_for_fold_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_priority_encoder"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused priority-encoder surface."
            ),
            phase3_priority_encoder_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_flop_mix"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused sequential flop / flop-mux surface."
            ),
            phase3_flop_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;

        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_phase3_slice_concat_varshift"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, focused selectable Slice/Concat plus variable-shift surface."
            ),
            phase3_slice_concat_varshift_focus_config(strategy, next_seed),
        )?);
        next_seed += 1;
    }

    Ok(scenarios)
}

fn build_phase4_hierarchy_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ] {
        let strategy_label = construction_strategy_name(strategy);
        let strategy_slug = strategy_slug(strategy);

        for (name_suffix, description_suffix, config) in [
            (
                "phase4_hier2_inst2_comb",
                "depth-1 hierarchy with 2 leaf modules, 2 child instances, and exact child/library cardinality with combinational share-heavy children",
                phase4_hierarchy_comb_focus_config(strategy, next_seed, 2, 2),
            ),
            (
                "phase4_hier2_inst4_seq",
                "depth-1 hierarchy with 2 leaf modules, 4 child instances, and reused child definitions with sequential motif-heavy children",
                phase4_hierarchy_seq_focus_config(strategy, next_seed + 1, 2, 4),
            ),
            (
                "phase4_hier4_inst2_comb",
                "depth-1 hierarchy with 4 leaf modules, 2 child instances, and an under-instantiated library with combinational share-heavy children",
                phase4_hierarchy_comb_focus_config(strategy, next_seed + 2, 4, 2),
            ),
            (
                "phase4_recur_d2_b2to3_comb",
                "bounded recursive hierarchy at exact depth 2 with child-instance fallback range [2:3] and combinational share-heavy leaves",
                phase4_recursive_comb_focus_config(strategy, next_seed + 3),
            ),
            (
                "phase4_recur_profile_d2_top4_mid2_seq",
                "bounded recursive hierarchy at exact depth 2 with depth-specific branching override top=4 and depth1=2 plus sequential motif-heavy leaves",
                phase4_recursive_profile_seq_focus_config(strategy, next_seed + 4),
            ),
            (
                "phase4_recur_d2to3_b2_mixed_comb",
                "bounded recursive hierarchy with leaf depths inside [2:3], exact child-instance count 2, and combinational share-heavy leaves so the realized tree mixes shallow and deep branches",
                phase4_recursive_mixed_depth_comb_focus_config(strategy, next_seed + 5),
            ),
            (
                "phase4_recur_d2_b2_ondemand_comb",
                "bounded recursive hierarchy at exact depth 2 with exact child-instance count 2 and fresh on-demand child synthesis per instance slot",
                phase4_recursive_ondemand_comb_focus_config(strategy, next_seed + 6),
            ),
            (
                "phase4_hier2_inst4_parent_state",
                "depth-1 hierarchy with combinational children and explicit parent-local flop state in the hierarchy layer",
                phase4_hierarchy_parent_state_focus_config(strategy, next_seed + 7),
            ),
            (
                "phase4_hier2_inst4_registered_sibling_state",
                "depth-1 hierarchy with combinational children and registered child-to-child routing through parent-local state",
                phase4_hierarchy_registered_sibling_state_focus_config(strategy, next_seed + 8),
            ),
            (
                "phase4_hier2_inst4_registered_sibling_multistage_state",
                "depth-1 hierarchy with combinational children and registered child-to-child routing that chains through earlier parent-local state",
                phase4_hierarchy_registered_sibling_multistage_state_focus_config(
                    strategy,
                    next_seed + 9,
                ),
            ),
            (
                "phase4_hier2_inst4_registered_sibling_mixed_support_state",
                "depth-1 hierarchy with direct registered sibling-routed child inputs whose parent-local D paths mix parent data ports with sibling instance outputs",
                phase4_hierarchy_registered_sibling_mixed_support_focus_config(
                    strategy,
                    next_seed + 33,
                ),
            ),
            (
                "phase4_recur_d2_registered_sibling_mixed_support_state",
                "bounded recursive hierarchy at exact depth 2 where non-top direct registered sibling-routed child inputs mix parent data ports with sibling instance outputs",
                phase4_recursive_registered_sibling_mixed_support_focus_config(
                    strategy,
                    next_seed + 34,
                ),
            ),
            (
                "phase4_recur_d2_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 2 where non-top unregistered parent-composed child-input cones mix parent data ports with sibling instance outputs without helper instances",
                phase4_recursive_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 35,
                ),
            ),
            (
                "phase4_recur_d2_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-output cones mix parent data ports with child outputs without helper instances or parent-local state",
                phase4_recursive_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 36,
                ),
            ),
            (
                "phase4_recur_d2_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-output cones mix parent data ports, child outputs, and parent-local Qs without helper instances",
                phase4_recursive_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 37,
                ),
            ),
            (
                "phase4_recur_d2_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 2 where non-top unregistered parent-composed child-input cones mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 38,
                ),
            ),
            (
                "phase4_recur_d2_parent_state",
                "bounded recursive hierarchy at exact depth 2 where non-top parents own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_parent_state_focus_config(strategy, next_seed + 39),
            ),
            (
                "phase4_recur_d3_parent_state",
                "bounded recursive hierarchy at exact depth 3 where non-top parents at two intermediate layers own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_d3_parent_state_focus_config(strategy, next_seed + 40),
            ),
            (
                "phase4_recur_d3_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 3 where non-top unregistered parent-composed child-input cones at two intermediate layers mix parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d3_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 41,
                ),
            ),
            (
                "phase4_recur_d3_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 3 where non-top parent-output cones at two intermediate layers mix parent data ports with child outputs without helper instances or parent-local state",
                phase4_recursive_d3_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 42,
                ),
            ),
            (
                "phase4_recur_d3_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 3 where non-top parent-output cones at two intermediate layers mix parent data ports, child outputs, and parent-local Qs without helper instances",
                phase4_recursive_d3_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 43,
                ),
            ),
            (
                "phase4_recur_d3_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 3 where non-top unregistered parent-composed child-input cones at two intermediate layers mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d3_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 44,
                ),
            ),
            (
                "phase4_recur_d4_parent_state",
                "bounded recursive hierarchy at exact depth 4 where non-top parents at three intermediate layers own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_d4_parent_state_focus_config(strategy, next_seed + 45),
            ),
            (
                "phase4_recur_d4_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 4 where non-top unregistered parent-composed child-input cones at three intermediate layers mix parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d4_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 46,
                ),
            ),
            (
                "phase4_recur_d4_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 4 where non-top parent-output cones at three intermediate layers mix parent data ports with child outputs without helper instances or parent-local state",
                phase4_recursive_d4_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 47,
                ),
            ),
            (
                "phase4_recur_d4_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 4 where non-top parent-output cones at three intermediate layers mix parent data ports, child outputs, and parent-local Qs without helper instances",
                phase4_recursive_d4_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 48,
                ),
            ),
            (
                "phase4_recur_d4_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 4 where non-top unregistered parent-composed child-input cones at three intermediate layers mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d4_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 49,
                ),
            ),
            (
                "phase4_recur_d5_parent_state",
                "bounded recursive hierarchy at exact depth 5 where non-top parents at four intermediate layers own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_d5_parent_state_focus_config(strategy, next_seed + 50),
            ),
            (
                "phase4_recur_d5_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 5 where non-top unregistered parent-composed child-input cones at four intermediate layers mix parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d5_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 51,
                ),
            ),
            (
                "phase4_recur_d5_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 5 where non-top parent outputs at four intermediate layers compose parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d5_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 52,
                ),
            ),
            (
                "phase4_recur_d5_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 5 where non-top parent outputs at four intermediate layers compose parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d5_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 53,
                ),
            ),
            (
                "phase4_recur_d5_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 5 where non-top unregistered parent-composed child-input cones at four intermediate layers mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d5_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 54,
                ),
            ),
            (
                "phase4_recur_d6_parent_state",
                "bounded recursive hierarchy at exact depth 6 where non-top parents at five intermediate layers own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_d6_parent_state_focus_config(strategy, next_seed + 55),
            ),
            (
                "phase4_recur_d6_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 6 where non-top unregistered parent-composed child-input cones at five intermediate layers mix parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d6_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 56,
                ),
            ),
            (
                "phase4_recur_d6_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 6 where non-top parent outputs at five intermediate layers compose parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d6_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 57,
                ),
            ),
            (
                "phase4_recur_d6_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 6 where non-top parent outputs at five intermediate layers compose parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d6_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 58,
                ),
            ),
            (
                "phase4_recur_d6_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 6 where non-top unregistered parent-composed child-input cones at five intermediate layers mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances (2,2 calibrated)",
                phase4_recursive_d6_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 59,
                ),
            ),
            (
                "phase4_recur_d7_parent_state",
                "bounded recursive hierarchy at exact depth 7 where non-top parents at six intermediate layers own local flops without helper instances, sibling routing, registered routing, or parent-composed child-input cones",
                phase4_recursive_d7_parent_state_focus_config(strategy, next_seed + 60),
            ),
            (
                "phase4_recur_d7_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 7 where non-top unregistered parent-composed child-input cones at six intermediate layers mix parent data ports with sibling instance outputs without helper instances or parent-local state (2,2 calibrated)",
                phase4_recursive_d7_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 61,
                ),
            ),
            (
                "phase4_recur_d7_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 7 where non-top parent outputs at six intermediate layers compose parent data ports with sibling instance outputs without helper instances or parent-local state",
                phase4_recursive_d7_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 62,
                ),
            ),
            (
                "phase4_recur_d2_registered_sibling_multistage_state",
                "bounded recursive hierarchy at exact depth 2 where non-top direct registered sibling-routed child inputs chain through earlier parent-local state without helper instances",
                phase4_recursive_registered_sibling_multistage_state_focus_config(
                    strategy,
                    next_seed + 32,
                ),
            ),
            (
                "phase4_hier2_inst4_direct_sibling_parent_cone_instance",
                "depth-1 hierarchy with direct sibling-routed child inputs that instantiate helper children as internal parent-cone sources",
                phase4_hierarchy_direct_sibling_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 10,
                ),
            ),
            (
                "phase4_recur_d2_direct_sibling_parent_cone_instance",
                "bounded recursive hierarchy at exact depth 2 where non-top direct sibling-routed child inputs instantiate helper children as internal parent-cone sources",
                phase4_recursive_direct_sibling_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 22,
                ),
            ),
            (
                "phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state",
                "bounded recursive hierarchy at exact depth 2 where non-top direct registered sibling-routed child inputs instantiate helper children as internal parent-cone sources",
                phase4_recursive_direct_registered_sibling_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 23,
                ),
            ),
            (
                "phase4_hier2_inst4_direct_registered_sibling_parent_cone_instance_state",
                "depth-1 hierarchy with direct registered sibling-routed child inputs whose parent-local D paths instantiate helper children as internal parent-cone sources",
                phase4_hierarchy_direct_registered_sibling_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 11,
                ),
            ),
            (
                "phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state",
                "depth-1 hierarchy with direct registered sibling-routed child inputs that chain helper-sourced parent-local Qs through later parent-local state",
                phase4_hierarchy_registered_sibling_parent_cone_instance_multistage_focus_config(
                    strategy,
                    next_seed + 18,
                ),
            ),
            (
                "phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state",
                "bounded recursive hierarchy at exact depth 2 where non-top direct registered sibling-routed child inputs chain helper-sourced parent-local Qs through later parent-local state",
                phase4_recursive_registered_sibling_parent_cone_instance_multistage_focus_config(
                    strategy,
                    next_seed + 25,
                ),
            ),
            (
                "phase4_hier2_inst4_registered_child_input_cone_state",
                "depth-1 hierarchy with combinational children and registered child-input routing through parent-composed logic plus parent-local state",
                phase4_hierarchy_registered_child_input_cone_state_focus_config(
                    strategy,
                    next_seed + 12,
                ),
            ),
            (
                "phase4_recur_d2_registered_mixed_child_input_state",
                "bounded recursive hierarchy at exact depth 2 where non-top registered parent-composed child-input D cones mix parent data ports with child outputs",
                phase4_recursive_registered_mixed_child_input_state_focus_config(
                    strategy,
                    next_seed + 30,
                ),
            ),
            (
                "phase4_recur_d2_registered_multistage_child_input_state",
                "bounded recursive hierarchy at exact depth 2 where non-top registered parent-composed child-input D cones chain through earlier parent-local Qs without helper instances",
                phase4_recursive_registered_multistage_child_input_state_focus_config(
                    strategy,
                    next_seed + 31,
                ),
            ),
            (
                "phase4_recur_d2_registered_parent_cone_instance_state",
                "bounded recursive hierarchy at exact depth 2 where non-top registered parent-composed child-input D cones instantiate helper children as internal parent-cone sources",
                phase4_recursive_registered_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 24,
                ),
            ),
            (
                "phase4_hier2_inst4_parent_cone_instance",
                "depth-1 hierarchy with combinational children and parent-composed child-input cones that instantiate helper children as internal parent-cone sources",
                phase4_hierarchy_parent_cone_instance_focus_config(strategy, next_seed + 13),
            ),
            (
                "phase4_hier2_inst4_parent_output_cone_instance",
                "depth-1 hierarchy with combinational children and parent-output cones that instantiate helper children as internal parent-cone sources",
                phase4_hierarchy_parent_output_cone_instance_focus_config(
                    strategy,
                    next_seed + 14,
                ),
            ),
            (
                "phase4_recur_d2_parent_output_cone_instance",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-output cones instantiate helper children as internal parent-cone sources",
                phase4_recursive_parent_output_cone_instance_focus_config(
                    strategy,
                    next_seed + 27,
                ),
            ),
            (
                "phase4_hier2_inst4_parent_output_cone_instance_state",
                "depth-1 hierarchy with combinational children and parent-output cones that route helper children through parent-local state",
                phase4_hierarchy_parent_output_cone_instance_state_focus_config(
                    strategy,
                    next_seed + 15,
                ),
            ),
            (
                "phase4_recur_d2_parent_output_cone_instance_state",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-output cones route helper children through parent-local state",
                phase4_recursive_parent_output_cone_instance_state_focus_config(
                    strategy,
                    next_seed + 28,
                ),
            ),
            (
                "phase4_hier2_inst4_parent_cone_instance_budget3",
                "depth-1 hierarchy with combinational children and a three-helper parent-cone instance budget",
                phase4_hierarchy_parent_cone_instance_budget_focus_config(
                    strategy,
                    next_seed + 16,
                ),
            ),
            (
                "phase4_recur_d2_parent_cone_instance_budget3",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-composed child-input cones can spend a three-helper parent-cone budget",
                phase4_recursive_parent_cone_instance_budget_focus_config(
                    strategy,
                    next_seed + 29,
                ),
            ),
            (
                "phase4_hier2_inst4_registered_parent_cone_instance_state",
                "depth-1 hierarchy with combinational children and registered parent-composed child-input cones that instantiate helper children as internal parent-cone sources",
                phase4_hierarchy_registered_parent_cone_instance_focus_config(
                    strategy,
                    next_seed + 17,
                ),
            ),
            (
                "phase4_hier2_inst4_registered_parent_cone_instance_multistage_state",
                "depth-1 hierarchy with registered parent-composed child-input cones that chain helper-sourced parent-local Qs through later parent-composed logic",
                phase4_hierarchy_registered_parent_cone_instance_multistage_focus_config(
                    strategy,
                    next_seed + 19,
                ),
            ),
            (
                "phase4_recur_d2_registered_parent_cone_instance_multistage_state",
                "bounded recursive hierarchy at exact depth 2 where non-top registered parent-composed child-input D cones chain helper-sourced parent-local Qs through later parent-composed logic",
                phase4_recursive_registered_parent_cone_instance_multistage_focus_config(
                    strategy,
                    next_seed + 26,
                ),
            ),
            (
                "phase4_hier2_inst4_parent_cone_instance_state",
                "depth-1 hierarchy with parent-composed child-input helper routes through parent-local state",
                phase4_hierarchy_parent_cone_instance_state_focus_config(
                    strategy,
                    next_seed + 20,
                ),
            ),
            (
                "phase4_recur_d2_parent_cone_instance_state",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-composed child-input helper routes pass through parent-local state",
                phase4_recursive_parent_cone_instance_state_focus_config(
                    strategy,
                    next_seed + 21,
                ),
            ),
        ] {
            scenarios.push(make_scenario(
                &format!("{strategy_slug}_nodeid_egraph_{name_suffix}"),
                &format!(
                    "{strategy_label} strategy, node-id + e-graph, {description_suffix}."
                ),
                config,
            )?);
        }
        next_seed += 38;
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
        for_fold_prob: 0.25,
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

fn hierarchy_focused_sequential_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = motif_heavy_sequential_config(strategy, seed, 0.3);
    cfg.flop_prob = 0.3;
    cfg.terminal_reuse_prob = 0.5;
    cfg.constant_prob = 0.1;
    cfg.coefficient_prob = 0.25;
    cfg.const_shift_amount_prob = 0.8;
    cfg.const_comparand_prob = 0.6;
    cfg.priority_encoder_prob = 0.05;
    cfg.case_mux_prob = 0.1;
    cfg.casez_mux_prob = 0.1;
    cfg.for_fold_prob = 0.1;
    cfg.comb_mux_prob = 0.2;
    cfg.min_inputs = 2;
    cfg.max_inputs = 4;
    cfg.min_outputs = 1;
    cfg.max_outputs = 3;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 4;
    cfg.min_mux_arms = 2;
    cfg.max_mux_arms = 4;
    cfg
}

fn phase3_base_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        share_prob: 0.5,
        terminal_reuse_prob: 0.8,
        constant_prob: 0.05,
        min_inputs: 3,
        max_inputs: 8,
        min_outputs: 2,
        max_outputs: 4,
        min_width: 2,
        max_width: 16,
        max_depth: 6,
        min_mux_arms: 2,
        max_mux_arms: 5,
        ..Config::default()
    }
}

fn phase3_comb_mux_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 1.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 0.0,
        gate_bitwise_weight: 0,
        gate_arith_weight: 0,
        gate_struct_weight: 1,
        gate_compare_weight: 0,
        gate_reduce_weight: 0,
        gate_shift_weight: 0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_case_mux_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 1.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 0.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_casez_mux_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 1.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 0.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_for_fold_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 1.0,
        priority_encoder_prob: 0.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        min_width: 2,
        max_width: 8,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_priority_encoder_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 1.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        constant_prob: 0.0,
        gate_bitwise_weight: 0,
        gate_arith_weight: 0,
        gate_struct_weight: 1,
        gate_compare_weight: 0,
        gate_reduce_weight: 0,
        gate_shift_weight: 0,
        min_width: 2,
        max_width: 3,
        min_mux_arms: 3,
        max_mux_arms: 5,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_flop_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 1.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 0.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        share_prob: 0.4,
        terminal_reuse_prob: 0.6,
        min_width: 2,
        max_width: 16,
        max_depth: 5,
        ..phase3_base_config(strategy, seed)
    }
}

fn phase3_slice_concat_varshift_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        flop_prob: 0.0,
        comb_mux_prob: 0.0,
        case_mux_prob: 0.0,
        casez_mux_prob: 0.0,
        for_fold_prob: 0.0,
        priority_encoder_prob: 0.0,
        coefficient_prob: 0.0,
        const_shift_amount_prob: 0.0,
        const_comparand_prob: 0.0,
        share_prob: 0.0,
        terminal_reuse_prob: 1.0,
        constant_prob: 0.0,
        gate_bitwise_weight: 0,
        gate_arith_weight: 0,
        gate_struct_weight: 1,
        gate_compare_weight: 0,
        gate_reduce_weight: 0,
        gate_shift_weight: 2,
        min_inputs: 2,
        max_inputs: 4,
        min_outputs: 2,
        max_outputs: 2,
        min_width: 4,
        max_width: 8,
        max_depth: 4,
        ..phase3_base_config(strategy, seed)
    }
}

fn with_hierarchy_wrapper(
    mut cfg: Config,
    num_leaf_modules: u32,
    num_child_instances: u32,
) -> Config {
    cfg.hierarchy_depth = 1;
    cfg.num_leaf_modules = num_leaf_modules;
    cfg.num_child_instances = num_child_instances;
    cfg
}

fn with_recursive_hierarchy(
    mut cfg: Config,
    min_depth: u32,
    max_depth: u32,
    min_child_instances: u32,
    max_child_instances: u32,
) -> Config {
    cfg.min_hierarchy_depth = min_depth;
    cfg.max_hierarchy_depth = max_depth;
    cfg.min_child_instances_per_module = min_child_instances;
    cfg.max_child_instances_per_module = max_child_instances;
    cfg
}

fn with_recursive_hierarchy_profile(
    mut cfg: Config,
    min_depth: u32,
    max_depth: u32,
    min_child_instances: u32,
    max_child_instances: u32,
    child_instances_per_depth: BTreeMap<u32, CountRange>,
) -> Config {
    cfg = with_recursive_hierarchy(
        cfg,
        min_depth,
        max_depth,
        min_child_instances,
        max_child_instances,
    );
    cfg.child_instances_per_module_by_depth = child_instances_per_depth;
    cfg
}

fn with_hierarchy_child_source_mode(
    mut cfg: Config,
    source_mode: HierarchyChildSourceMode,
) -> Config {
    cfg.hierarchy_child_source_mode = source_mode;
    cfg
}

fn phase4_hierarchy_comb_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
    num_leaf_modules: u32,
    num_child_instances: u32,
) -> Config {
    let mut cfg = with_hierarchy_wrapper(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        num_leaf_modules,
        num_child_instances,
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_hierarchy_seq_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
    num_leaf_modules: u32,
    num_child_instances: u32,
) -> Config {
    let mut cfg = with_hierarchy_wrapper(
        hierarchy_focused_sequential_config(strategy, seed),
        num_leaf_modules,
        num_child_instances,
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_recursive_comb_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        3,
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_recursive_profile_seq_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = with_recursive_hierarchy_profile(
        hierarchy_focused_sequential_config(strategy, seed),
        2,
        2,
        1,
        3,
        BTreeMap::from([
            (0, CountRange { min: 4, max: 4 }),
            (1, CountRange { min: 2, max: 2 }),
        ]),
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_recursive_mixed_depth_comb_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        3,
        2,
        2,
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_recursive_ondemand_comb_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_hierarchy_child_source_mode(
        with_recursive_hierarchy(
            share_heavy_comb_only_config(strategy, seed, 0.9),
            2,
            2,
            2,
            2,
        ),
        HierarchyChildSourceMode::OnDemand,
    );
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg
}

fn phase4_recursive_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_recursive_parent_state_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d3_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        3,
        3,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d3_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        3,
        3,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d3_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d3_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d3_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d3_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_recursive_d4_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        4,
        4,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d4_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        4,
        4,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d4_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d4_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d4_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d4_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_recursive_d4_parent_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        4,
        4,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d5_parent_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        5,
        5,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d7_parent_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        7,
        7,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d7_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Calibration: depth-7 mixed-support cells use 2,2 child-instance bounds
    // (depths 3-5 used 4,4; depth 6 dropped to 2,2). At depth 7 the 4,4 tree
    // would grow to ~5461 internal occurrences, far beyond a safe-slice
    // budget for downstream-clean tools. 2,2/depth-7 yields 127 occurrences
    // and proves the mixed-support surface at exact depth 7 cleanly.
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        7,
        7,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d7_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        7,
        7,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d6_parent_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        6,
        6,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d6_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Calibration: depth-6 mixed-support cells use 2,2 child-instance bounds
    // (depths 3-5 used 4,4). At 4,4/depth-6 the design is ~1365 internal
    // module occurrences and the downstream-clean gate takes hours per
    // scenario. 2,2/depth-6 yields 63 occurrences and still proves the
    // mixed-support surface at exact depth 6.
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        6,
        6,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d6_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        6,
        6,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d6_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d6_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d6_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Calibration: built atop r74's 2,2 child-instance helper for the same
    // safe-slice reason. depths 3-5 used 4,4 for stateful mixed-support
    // cells; at 4,4/depth-6 the gate would take many hours per scenario.
    let mut cfg = phase4_recursive_d6_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_recursive_d5_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        5,
        5,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d5_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        5,
        5,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_d5_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d5_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d5_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d5_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_recursive_d3_parent_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        3,
        3,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}
fn phase4_recursive_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_hierarchy_parent_state_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 8;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_registered_sibling_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 1.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_registered_sibling_multistage_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_registered_sibling_state_focus_config(strategy, seed);
    cfg.flop_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn phase4_hierarchy_registered_sibling_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_registered_sibling_state_focus_config(strategy, seed);
    cfg.flop_prob = 0.0;
    cfg.hierarchy_registered_sibling_mixed_support_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}
fn phase4_recursive_registered_sibling_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 1.0;
    cfg.hierarchy_registered_sibling_mixed_support_prob = 1.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_registered_sibling_multistage_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 1.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_direct_sibling_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_direct_registered_sibling_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_direct_sibling_parent_cone_instance_focus_config(strategy, seed);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 1.0;
    cfg.max_flops_per_module = 8;
    cfg
}

fn phase4_recursive_direct_sibling_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 1.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_direct_registered_sibling_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_direct_sibling_parent_cone_instance_focus_config(strategy, seed);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 1.0;
    cfg.max_flops_per_module = 8;
    cfg
}

fn phase4_hierarchy_registered_sibling_parent_cone_instance_multistage_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_direct_registered_sibling_parent_cone_instance_focus_config(
        strategy, seed,
    );
    cfg.flop_prob = 0.0;
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn phase4_recursive_registered_sibling_parent_cone_instance_multistage_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_direct_registered_sibling_parent_cone_instance_focus_config(
        strategy, seed,
    );
    cfg.min_child_instances_per_module = 4;
    cfg.max_child_instances_per_module = 4;
    cfg.flop_prob = 0.0;
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn phase4_hierarchy_registered_child_input_cone_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_registered_mixed_child_input_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_registered_multistage_child_input_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        4,
        4,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_parent_cone_instance_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_parent_cone_instance_focus_config(strategy, seed);
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_parent_cone_instance_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_hierarchy_parent_output_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_comb_focus_config(strategy, seed, 2, 4);
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_parent_output_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_parent_output_cone_instance_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_parent_output_cone_instance_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_parent_output_cone_instance_state_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_parent_output_cone_instance_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_hierarchy_parent_cone_instance_budget_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_parent_cone_instance_focus_config(strategy, seed);
    cfg.max_parent_cone_instances_per_module = 3;
    cfg
}

fn phase4_recursive_parent_cone_instance_budget_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 0.0;
    cfg.hierarchy_child_input_cone_prob = 1.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_registered_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_registered_child_input_cone_state_focus_config(strategy, seed);
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn phase4_recursive_registered_parent_cone_instance_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = with_recursive_hierarchy(
        share_heavy_comb_only_config(strategy, seed, 0.9),
        2,
        2,
        2,
        2,
    );
    cfg.flop_prob = 0.0;
    cfg.hierarchy_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_sibling_route_prob = 0.0;
    cfg.hierarchy_registered_child_input_cone_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 3;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 8;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_hierarchy_registered_parent_cone_instance_multistage_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_hierarchy_registered_parent_cone_instance_focus_config(strategy, seed);
    cfg.flop_prob = 0.0;
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn phase4_recursive_registered_parent_cone_instance_multistage_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_registered_parent_cone_instance_focus_config(strategy, seed);
    cfg.min_child_instances_per_module = 4;
    cfg.max_child_instances_per_module = 4;
    cfg.flop_prob = 0.0;
    cfg.max_parent_cone_instances_per_module = 1;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg
}

fn run_scenario(
    scenario: &Scenario,
    cli: &Cli,
    plan: &RunPlan,
    out_root: &Path,
    runtime_fingerprint: Option<&str>,
) -> Result<ScenarioReport> {
    if scenario.config.effective_hierarchy_depth_range().is_some() {
        return run_design_scenario(scenario, cli, plan, out_root, runtime_fingerprint);
    }

    run_module_scenario(scenario, cli, plan, out_root, runtime_fingerprint)
}

fn run_module_scenario(
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
        artifact_kind: "module".to_string(),
        aggregate,
        coverage,
        tool_summary,
        modules,
        designs: Vec::new(),
    })
}

fn run_design_scenario(
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
    let mut designs = Vec::with_capacity(plan.modules_per_scenario);

    for design_index in 0..plan.modules_per_scenario {
        if let Some(report) = resume_existing_design(
            &mut generator,
            cli,
            &scenario_dir,
            design_index,
            runtime_fingerprint,
        )? {
            designs.push(report);
            continue;
        }

        let prepared = prepare_design(&mut generator, &scenario_dir, design_index)?;
        let generator_checkpoint = generator.checkpoint();
        designs.push(materialize_prepared_design(
            cli,
            &prepared,
            &generator_checkpoint,
            runtime_fingerprint,
            true,
        )?);
    }

    write_design_scenario_manifest(&scenario_dir, scenario, &designs)?;

    let aggregate = aggregate_design_metrics(&designs);
    let coverage = summarize_design_coverage(scenario, &designs);
    let tool_summary = summarize_design_tools(&designs);

    Ok(ScenarioReport {
        name: scenario.name.clone(),
        description: scenario.description.clone(),
        out_dir: scenario_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&scenario.name)
            .to_string(),
        config: scenario.config.clone(),
        artifact_kind: "design".to_string(),
        aggregate,
        coverage,
        tool_summary,
        modules: Vec::new(),
        designs,
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

fn resume_existing_design(
    generator: &mut Generator,
    cli: &Cli,
    scenario_dir: &Path,
    design_index: usize,
    runtime_fingerprint: Option<&str>,
) -> Result<Option<DesignReport>> {
    if !cli.resume {
        return Ok(None);
    }

    let paths = design_paths(scenario_dir, design_index);
    let Some(checkpoint) = load_design_checkpoint(&paths.checkpoint_path)? else {
        return Ok(None);
    };

    if let Some(report) = try_fast_resume_design_checkpoint(
        generator,
        cli,
        scenario_dir,
        &checkpoint,
        runtime_fingerprint,
    )? {
        return Ok(Some(report));
    }

    let prepared = prepare_design(generator, scenario_dir, design_index)?;
    validate_checkpoint_against_prepared_design(&checkpoint.report, &prepared)?;
    validate_design_files_against_prepared(&prepared)?;

    let generator_checkpoint = generator.checkpoint();
    let report = if checkpoint_matches_design_cli(&checkpoint, cli) {
        let mut report = checkpoint.report;
        report.hierarchy = prepared.hierarchy.clone();
        report.metrics = prepared.metrics.clone();
        report.modules = prepared
            .modules
            .iter()
            .map(|module| EmittedModuleReport {
                file: module.file.clone(),
                name: module.name.clone(),
                metrics: module.metrics.clone(),
            })
            .collect();
        report
    } else {
        run_design_tools(cli, &prepared)?
    };
    write_design_checkpoint(
        cli,
        &prepared.paths.checkpoint_path,
        &report,
        &generator_checkpoint,
        runtime_fingerprint,
        &prepared.modules,
    )?;
    Ok(Some(report))
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

fn write_design_scenario_manifest(
    scenario_dir: &Path,
    scenario: &Scenario,
    designs: &[DesignReport],
) -> Result<()> {
    let manifest_designs: Vec<_> = designs
        .iter()
        .map(|design| {
            let modules: Vec<_> = design
                .modules
                .iter()
                .map(|module| {
                    serde_json::json!({
                        "file": module.file,
                        "name": module.name,
                        "metrics": module.metrics,
                    })
                })
                .collect();
            serde_json::json!({
                "index": design.index,
                "top": design.top,
                "files": design.files,
                "hierarchy": design.hierarchy,
                "metrics": design.metrics,
                "modules": modules,
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
        "designs": manifest_designs,
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

fn design_paths(scenario_dir: &Path, design_index: usize) -> DesignPaths {
    DesignPaths {
        checkpoint_path: scenario_dir.join(format!("design_{design_index:04}.design-report.json")),
    }
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

fn prepare_design(
    generator: &mut Generator,
    scenario_dir: &Path,
    design_index: usize,
) -> Result<PreparedDesign> {
    let paths = design_paths(scenario_dir, design_index);
    let design = generator.generate_design();
    anvil::ir::validate::validate_design(&design).map_err(|err| anyhow::anyhow!("{err}"))?;
    prepared_design_from_design(paths, design_index, &design, scenario_dir)
}

fn prepared_design_from_design(
    paths: DesignPaths,
    design_index: usize,
    design: &Design,
    scenario_dir: &Path,
) -> Result<PreparedDesign> {
    let metrics = anvil::metrics::compute_design(design);
    let hierarchy = hierarchy_facts_from_design(design, design_index, &metrics)?;
    let mut modules = Vec::with_capacity(design.modules.len());
    for module in &design.modules {
        let metrics = anvil::metrics::compute(module);
        let file = format!("{}.sv", module.name);
        let sv_path = scenario_dir.join(&file);
        let sv_text = anvil::emit::to_sv_in_design(module, design);
        let sv_hash = hash_bytes(sv_text.as_bytes());
        modules.push(PreparedEmittedModule {
            file,
            name: module.name.clone(),
            metrics,
            sv_path,
            sv_text,
            sv_hash,
        });
    }
    if !modules.iter().any(|module| module.name == design.top) {
        bail!(
            "design {} missing top module {} in emitted module set",
            design_index,
            design.top
        );
    }
    Ok(PreparedDesign {
        paths,
        index: design_index,
        top: design.top.clone(),
        hierarchy,
        metrics,
        modules,
    })
}

fn hierarchy_facts_from_design(
    design: &Design,
    design_index: usize,
    metrics: &DesignMetrics,
) -> Result<HierarchyFacts> {
    let top = design
        .modules
        .iter()
        .find(|module| module.name == design.top)
        .with_context(|| format!("design {design_index} missing top module {}", design.top))?;
    let top_instances = top.instances.len();

    Ok(HierarchyFacts {
        library_modules: metrics.num_library_modules,
        top_instances,
        unique_instantiated_modules: metrics.num_unique_instantiated_modules,
        reused_child_definition: metrics
            .instantiated_module_histogram
            .values()
            .any(|&count| count > 1),
        underinstantiated_library: metrics.num_unused_module_definitions > 0,
    })
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

fn materialize_prepared_design(
    cli: &Cli,
    prepared: &PreparedDesign,
    generator_checkpoint: &GeneratorCheckpoint,
    runtime_fingerprint: Option<&str>,
    write_sv: bool,
) -> Result<DesignReport> {
    if write_sv {
        for module in &prepared.modules {
            std::fs::write(&module.sv_path, &module.sv_text)
                .with_context(|| format!("write {}", module.sv_path.display()))?;
        }
    }

    let report = run_design_tools(cli, prepared)?;
    write_design_checkpoint(
        cli,
        &prepared.paths.checkpoint_path,
        &report,
        generator_checkpoint,
        runtime_fingerprint,
        &prepared.modules,
    )?;
    Ok(report)
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

fn run_design_tools(cli: &Cli, prepared: &PreparedDesign) -> Result<DesignReport> {
    let sv_paths: Vec<_> = prepared
        .modules
        .iter()
        .map(|module| module.sv_path.clone())
        .collect();
    let files: Vec<_> = prepared
        .modules
        .iter()
        .map(|module| module.file.clone())
        .collect();
    let modules: Vec<_> = prepared
        .modules
        .iter()
        .map(|module| EmittedModuleReport {
            file: module.file.clone(),
            name: module.name.clone(),
            metrics: module.metrics.clone(),
        })
        .collect();
    let scenario_dir = prepared
        .modules
        .first()
        .and_then(|module| module.sv_path.parent())
        .context("prepared design missing scenario directory")?;

    let verilator = if cli.skip_verilator {
        None
    } else {
        Some(run_verilator_design(
            &cli.verilator_bin,
            scenario_dir,
            &sv_paths,
            &prepared.top,
        )?)
    };

    let yosys = if cli.skip_yosys {
        Vec::new()
    } else {
        run_yosys_design(
            cli.yosys_mode,
            &cli.yosys_bin,
            scenario_dir,
            &sv_paths,
            &prepared.top,
        )?
    };

    Ok(DesignReport {
        index: prepared.index,
        top: prepared.top.clone(),
        files,
        modules,
        hierarchy: prepared.hierarchy.clone(),
        metrics: prepared.metrics.clone(),
        verilator,
        yosys,
    })
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

fn write_design_checkpoint(
    cli: &Cli,
    path: &Path,
    report: &DesignReport,
    generator_checkpoint: &GeneratorCheckpoint,
    runtime_fingerprint: Option<&str>,
    modules: &[PreparedEmittedModule],
) -> Result<()> {
    let files = modules
        .iter()
        .map(|module| DesignFileHash {
            file: module.file.clone(),
            hash: module.sv_hash.clone(),
        })
        .collect();
    let checkpoint = DesignCheckpoint {
        skip_verilator: cli.skip_verilator,
        skip_yosys: cli.skip_yosys,
        yosys_mode: yosys_mode_slug(cli.yosys_mode).to_string(),
        runtime_fingerprint: runtime_fingerprint.map(str::to_owned),
        files,
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

fn load_design_checkpoint(path: &Path) -> Result<Option<DesignCheckpoint>> {
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    match serde_json::from_str::<DesignCheckpoint>(&text) {
        Ok(checkpoint) => Ok(Some(checkpoint)),
        Err(_) => Ok(None),
    }
}

fn checkpoint_matches_cli(checkpoint: &ModuleCheckpoint, cli: &Cli) -> bool {
    checkpoint.skip_verilator == cli.skip_verilator
        && checkpoint.skip_yosys == cli.skip_yosys
        && checkpoint.yosys_mode == yosys_mode_slug(cli.yosys_mode)
}

fn checkpoint_matches_design_cli(checkpoint: &DesignCheckpoint, cli: &Cli) -> bool {
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

fn try_fast_resume_design_checkpoint(
    generator: &mut Generator,
    cli: &Cli,
    scenario_dir: &Path,
    checkpoint: &DesignCheckpoint,
    runtime_fingerprint: Option<&str>,
) -> Result<Option<DesignReport>> {
    if !checkpoint_matches_design_cli(checkpoint, cli) {
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
    for file in &checkpoint.files {
        let path = scenario_dir.join(&file.file);
        if !path.exists() {
            return Ok(None);
        }
        let existing_hash = hash_file(&path)?;
        if existing_hash != file.hash {
            return Ok(None);
        }
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

fn validate_checkpoint_against_prepared_design(
    report: &DesignReport,
    prepared: &PreparedDesign,
) -> Result<()> {
    if report.index != prepared.index {
        bail!(
            "resume mismatch for design {}: checkpoint index {}, expected {}",
            prepared.top,
            report.index,
            prepared.index
        );
    }
    if report.top != prepared.top {
        bail!(
            "resume mismatch for design {}: checkpoint top {}, expected {}",
            prepared.index,
            report.top,
            prepared.top
        );
    }
    let expected_files: Vec<_> = prepared
        .modules
        .iter()
        .map(|module| module.file.clone())
        .collect();
    if report.files != expected_files {
        bail!(
            "resume mismatch for design {}: checkpoint file set differs from regenerated design",
            prepared.top
        );
    }
    if report.modules.len() != prepared.modules.len() {
        bail!(
            "resume mismatch for design {}: checkpoint module count {}, expected {}",
            prepared.top,
            report.modules.len(),
            prepared.modules.len()
        );
    }
    for (reported, expected) in report.modules.iter().zip(&prepared.modules) {
        if reported.file != expected.file || reported.name != expected.name {
            bail!(
                "resume mismatch for design {}: checkpoint module {} / {} differs from regenerated {} / {}",
                prepared.top,
                reported.file,
                reported.name,
                expected.file,
                expected.name
            );
        }
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

fn validate_design_files_against_prepared(prepared: &PreparedDesign) -> Result<()> {
    for module in &prepared.modules {
        let existing = std::fs::read_to_string(&module.sv_path)
            .with_context(|| format!("read {}", module.sv_path.display()))?;
        if existing != module.sv_text {
            bail!(
                "resume mismatch for design {}: existing SV differs from regenerated module {}",
                prepared.top,
                module.file
            );
        }
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

fn run_verilator_design(
    bin: &str,
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> Result<ToolInvocation> {
    let mut argv = vec![
        "--lint-only".to_string(),
        "--top-module".to_string(),
        top.to_string(),
    ];
    argv.extend(sv_paths.iter().map(|path| path.display().to_string()));
    run_tool("verilator", bin, argv, out_dir, top)
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

fn run_yosys_design(
    mode: YosysMode,
    bin: &str,
    out_dir: &Path,
    sv_paths: &[PathBuf],
    top: &str,
) -> Result<Vec<ToolInvocation>> {
    let mut invocations = Vec::new();
    for (tool_label, script) in yosys_design_invocations(mode, sv_paths, top) {
        invocations.push(run_tool(
            tool_label,
            bin,
            vec!["-p".to_string(), script],
            out_dir,
            top,
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

fn yosys_design_invocations(
    mode: YosysMode,
    sv_paths: &[PathBuf],
    top: &str,
) -> Vec<(&'static str, String)> {
    let escaped_files = escape_paths_for_double_quotes(sv_paths);
    match mode {
        YosysMode::WithoutAbc => vec![(
            "yosys-without-abc",
            format!(
                "read_verilog -sv {escaped_files}; synth -top {top} -noabc; stat; check"
            ),
        )],
        YosysMode::WithAbc => vec![(
            "yosys-with-abc",
            format!(
                "read_verilog -sv {escaped_files}; synth -top {top} -noabc; abc -fast; opt -fast; stat; check"
            ),
        )],
        YosysMode::Both => vec![
            (
                "yosys-without-abc",
                format!(
                    "read_verilog -sv {escaped_files}; synth -top {top} -noabc; stat; check"
                ),
            ),
            (
                "yosys-with-abc",
                format!(
                    "read_verilog -sv {escaped_files}; synth -top {top} -noabc; abc -fast; opt -fast; stat; check"
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
        accumulate_metrics(&mut aggregate, &module.metrics);
    }
    aggregate
}

fn aggregate_design_metrics(designs: &[DesignReport]) -> AggregateMetrics {
    let mut aggregate = AggregateMetrics::default();
    for design in designs {
        for module in &design.modules {
            accumulate_metrics(&mut aggregate, &module.metrics);
        }
    }
    aggregate
}

fn summarize_tools(modules: &[ModuleReport]) -> ToolSummary {
    let mut summary = ToolSummary::default();
    for module in modules {
        accumulate_tool_summary(&mut summary, module.verilator.as_ref(), &module.yosys);
    }
    summary
}

fn summarize_design_tools(designs: &[DesignReport]) -> ToolSummary {
    let mut summary = ToolSummary::default();
    for design in designs {
        accumulate_tool_summary(&mut summary, design.verilator.as_ref(), &design.yosys);
    }
    summary
}

fn summarize_coverage(scenario: &Scenario, modules: &[ModuleReport]) -> CoverageSummary {
    let mut coverage = CoverageSummary::default();
    seed_scenario_coverage(&mut coverage, scenario);

    for module in modules {
        accumulate_module_coverage(&mut coverage, &module.metrics);
    }

    coverage
}

fn summarize_design_coverage(scenario: &Scenario, designs: &[DesignReport]) -> CoverageSummary {
    let mut coverage = CoverageSummary::default();
    seed_scenario_coverage(&mut coverage, scenario);

    for design in designs {
        coverage.saw_hierarchy_design = true;
        coverage.saw_multifile_design |= design.files.len() > 1;
        coverage.saw_reused_child_definition |= design.hierarchy.reused_child_definition;
        coverage.saw_underinstantiated_library |= design.hierarchy.underinstantiated_library;
        coverage.saw_on_demand_child_sourcing |= scenario.config.uses_on_demand_child_sourcing()
            && design.metrics.num_reused_instance_slots == 0
            && design.metrics.num_unused_module_definitions == 0
            && design.metrics.num_single_use_instantiated_modules
                == design.metrics.num_unique_instantiated_modules;
        coverage.saw_profiled_child_interface_synthesis |=
            scenario.config.uses_on_demand_child_sourcing()
                && design.metrics.num_profiled_instance_slots == design.metrics.num_instances
                && design.metrics.profiled_instance_fraction == 1.0;
        coverage.saw_hierarchy_sibling_routing |=
            design.metrics.child_input_bindings_from_instance_outputs > 0
                || design.metrics.child_input_bindings_from_mixed_support > 0;
        coverage.saw_hierarchy_registered_sibling_routing |= design
            .metrics
            .child_input_bindings_from_registered_instance_outputs
            > 0;
        coverage.saw_hierarchy_registered_sibling_mixed_support_routing |=
            scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario
                    .config
                    .hierarchy_registered_sibling_mixed_support_prob
                    > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_sibling_mixed_support
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_mixed_support
                    == 0;
        coverage.saw_recursive_hierarchy_registered_sibling_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario
                    .config
                    .hierarchy_registered_sibling_mixed_support_prob
                    > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_sibling_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_sibling_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_mixed_support
                    == 0;
        coverage.saw_hierarchy_direct_sibling_parent_cone_instance_routing |=
            scenario.config.hierarchy_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.child_input_bindings_from_instance_outputs > 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.child_input_bindings_from_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing |=
            scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0;
        coverage.saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0;
        coverage.saw_hierarchy_registered_parent_composed_routing |=
            scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    > 0;
        coverage.saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_cone_instances;
        coverage.saw_hierarchy_registered_mixed_support_routing |=
            scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_mixed_support
                    > 0;
        coverage.saw_recursive_hierarchy_registered_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_registered_multistage_routing |=
            scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_logic
                    > 0;
        coverage.saw_recursive_hierarchy_registered_multistage_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_registered_multistage_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_registered_multistage_sibling_routing |=
            scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_instance_outputs
                    > 0;
        coverage.saw_recursive_hierarchy_registered_multistage_sibling_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_registered_multistage_parent_cone_instance_routing |=
            scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    > 0;
        coverage.saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob > 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_instance_outputs
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_instance_outputs
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_logic
                    == 0;
        coverage
            .saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing |=
            scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                    > 0;
        coverage
            .saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_multistage_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_parent_composed_parent_cone_instance_flop_routing |=
            scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_cone_instances_through_parent_flops
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing |=
            scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instance_flop_mixed_support
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0;
        coverage
            .saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instance_flop_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_cone_instance_flop_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0;
        coverage.saw_hierarchy_registered_parent_cone_instance_routing |= design
            .metrics
            .child_input_bindings_from_registered_parent_cone_instances
            > 0;
        coverage.saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instance_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_parent_cone_instance_mixed_support;
        coverage.saw_hierarchy_parent_composed_child_inputs |= design
            .metrics
            .child_input_bindings_from_parent_composed_logic
            > 0;
        coverage.saw_hierarchy_mixed_support_child_inputs |=
            scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > 0
                && design.metrics.child_input_bindings_from_mixed_support > 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0;
        coverage.saw_recursive_hierarchy_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_mixed_support
                    == 0;
        coverage.saw_hierarchy_parent_cone_instance_routing |= design
            .metrics
            .child_input_bindings_from_parent_cone_instances
            > 0;
        coverage.saw_hierarchy_parent_cone_instance_mixed_support_routing |=
            scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instance_mixed_support
                    > 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instance_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_cone_instance_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instance_mixed_support
                    == 0;
        coverage.saw_hierarchy_parent_cone_instance_outputs |= design
            .metrics
            .hierarchy_outputs_reaching_parent_cone_instances
            > 0;
        coverage.saw_recursive_hierarchy_parent_cone_instance_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances
                    > design.metrics.top_outputs_reaching_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                    == 0;
        coverage.saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instance_mixed_support
                    > design
                        .metrics
                        .top_outputs_reaching_parent_cone_instance_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                    == 0;
        coverage.saw_hierarchy_parent_cone_instance_flop_outputs |= design
            .metrics
            .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
            > 0;
        coverage.saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs |= design
            .metrics
            .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support
            > 0;
        coverage.saw_recursive_hierarchy_parent_cone_instance_flop_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                    > design
                        .metrics
                        .top_outputs_reaching_parent_cone_instances_through_parent_flops
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support
                    > design
                        .metrics
                        .top_outputs_reaching_parent_cone_instances_through_parent_flops_with_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_multiple_parent_cone_instances_per_parent |=
            design.metrics.max_parent_cone_instances_per_internal_module > 1;
        coverage.saw_recursive_multiple_parent_cone_instances_per_parent |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.max_parent_cone_instances_per_module > 1
                && design.metrics.max_parent_cone_instances_per_internal_module
                    >= scenario.config.max_parent_cone_instances_per_module as usize
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances
                    > design.metrics.top_outputs_reaching_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && scenario.config.max_parent_cone_instances_per_module > 1
                && design.metrics.max_parent_cone_instances_per_internal_module
                    >= scenario.config.max_parent_cone_instances_per_module as usize
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_cone_instances
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_recursive_multiple_parent_cone_instances_per_parent_through_flops |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && scenario.config.max_parent_cone_instances_per_module > 1
                && design.metrics.max_parent_cone_instances_per_internal_module
                    >= scenario.config.max_parent_cone_instances_per_module as usize
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops
                    > design
                        .metrics
                        .top_outputs_reaching_parent_cone_instances_through_parent_flops
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances_through_parent_flops
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_cone_instances
                    == 0;
        coverage.saw_hierarchy_parent_local_flops |=
            design.metrics.hierarchy_parent_local_flops > 0;
        coverage.saw_recursive_hierarchy_parent_local_flops |=
            design.metrics.realized_max_leaf_depth > 1
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_3_parent_local_flops |=
            design.metrics.realized_max_leaf_depth >= 3
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_3_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 3
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_3_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 3
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0;
        coverage.saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 3
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_outputs_through_parent_flops
                    > design
                        .metrics
                        .top_parent_port_composed_outputs_through_parent_flops
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage
            .saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 3
                && design
                    .metrics
                    .child_input_bindings_from_stateful_parent_composed_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_stateful_parent_composed_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_4_parent_local_flops |=
            design.metrics.realized_max_leaf_depth >= 4
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_4_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 4
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_4_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 4
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0;
        coverage.saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 4
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_outputs_through_parent_flops
                    > design
                        .metrics
                        .top_parent_port_composed_outputs_through_parent_flops
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage
            .saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 4
                && design
                    .metrics
                    .child_input_bindings_from_stateful_parent_composed_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_stateful_parent_composed_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_5_parent_local_flops |=
            design.metrics.realized_max_leaf_depth >= 5
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_5_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 5
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_5_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 5
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0;
        coverage.saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 5
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_outputs_through_parent_flops
                    > design
                        .metrics
                        .top_parent_port_composed_outputs_through_parent_flops
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage
            .saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 5
                && design
                    .metrics
                    .child_input_bindings_from_stateful_parent_composed_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_stateful_parent_composed_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_6_parent_local_flops |=
            design.metrics.realized_max_leaf_depth >= 6
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_6_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 6
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_6_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 6
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0;
        coverage.saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 6
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_outputs_through_parent_flops
                    > design
                        .metrics
                        .top_parent_port_composed_outputs_through_parent_flops
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage
            .saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 6
                && design
                    .metrics
                    .child_input_bindings_from_stateful_parent_composed_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_stateful_parent_composed_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_7_parent_local_flops |=
            design.metrics.realized_max_leaf_depth >= 7
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design.metrics.internal_module_occurrences_with_local_flops > 0;
        coverage.saw_recursive_hierarchy_depth_7_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 7
                && design.metrics.child_input_bindings_from_mixed_support
                    > design.metrics.top_child_input_bindings_from_mixed_support
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_hierarchy_depth_7_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 7
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0;
        coverage.saw_recursive_hierarchy |= design.metrics.realized_max_leaf_depth > 1;
        coverage.saw_per_depth_branching_metrics |=
            design.metrics.avg_child_instances_by_parent_depth.len() > 1;
        coverage.saw_mixed_leaf_depth_hierarchy |=
            design.metrics.realized_min_leaf_depth < design.metrics.realized_max_leaf_depth;
        coverage.saw_hierarchy_parent_composition |=
            design.metrics.hierarchy_parent_composed_outputs > 0;
        coverage.saw_hierarchy_parent_port_composed_outputs |=
            design.metrics.hierarchy_parent_port_composed_outputs > 0;
        coverage.saw_recursive_hierarchy_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && scenario.config.hierarchy_parent_flop_prob == 0.0
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_output_fraction
                    > 0.0
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops == 0
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_composed_outputs
                    > design.metrics.top_parent_composed_outputs
                && design.metrics.hierarchy_parent_port_composed_outputs
                    > design.metrics.top_parent_port_composed_outputs
                && design
                    .metrics
                    .hierarchy_parent_port_composed_outputs_through_parent_flops
                    > design
                        .metrics
                        .top_parent_port_composed_outputs_through_parent_flops
                && design
                    .metrics
                    .hierarchy_parent_port_composed_parent_flop_output_fraction
                    > 0.0
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .hierarchy_outputs_reaching_parent_cone_instances
                    == 0;
        coverage.saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_sibling_route_prob == 0.0
                && scenario.config.hierarchy_registered_child_input_cone_prob == 0.0
                && scenario.config.hierarchy_child_input_cone_prob > 0.0
                && scenario.config.hierarchy_parent_cone_instance_prob == 0.0
                && scenario.config.hierarchy_parent_flop_prob > 0.0
                && design.metrics.hierarchy_parent_cone_instances == 0
                && design
                    .metrics
                    .child_input_bindings_from_parent_cone_instances
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_instance_outputs
                    == 0
                && design
                    .metrics
                    .child_input_bindings_from_registered_parent_composed_logic
                    == 0
                && design.metrics.hierarchy_parent_local_flops > design.metrics.top_local_flops
                && design
                    .metrics
                    .child_input_bindings_from_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_parent_composed_logic
                && design
                    .metrics
                    .child_input_bindings_from_stateful_parent_composed_mixed_support
                    > design
                        .metrics
                        .top_child_input_bindings_from_stateful_parent_composed_mixed_support
                && design
                    .metrics
                    .stateful_parent_composed_mixed_support_child_input_binding_fraction
                    > 0.0;
        for module in &design.modules {
            accumulate_module_coverage(&mut coverage, &module.metrics);
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
    dst.hierarchy_depths
        .extend(src.hierarchy_depths.iter().cloned());
    dst.hierarchy_leaf_module_counts
        .extend(src.hierarchy_leaf_module_counts.iter().cloned());
    dst.hierarchy_child_instance_counts
        .extend(src.hierarchy_child_instance_counts.iter().cloned());
    dst.hierarchy_child_source_modes
        .extend(src.hierarchy_child_source_modes.iter().cloned());
    dst.hierarchy_child_instance_override_profiles.extend(
        src.hierarchy_child_instance_override_profiles
            .iter()
            .cloned(),
    );
    dst.gate_categories
        .extend(src.gate_categories.iter().cloned());
    dst.gate_kinds.extend(src.gate_kinds.iter().cloned());
    dst.knob_attempts_seen
        .extend(src.knob_attempts_seen.iter().cloned());
    dst.knob_fires_seen
        .extend(src.knob_fires_seen.iter().cloned());
    dst.saw_hierarchy_design |= src.saw_hierarchy_design;
    dst.saw_multifile_design |= src.saw_multifile_design;
    dst.saw_instance_module |= src.saw_instance_module;
    dst.saw_instance_output_node |= src.saw_instance_output_node;
    dst.saw_reused_child_definition |= src.saw_reused_child_definition;
    dst.saw_underinstantiated_library |= src.saw_underinstantiated_library;
    dst.saw_on_demand_child_sourcing |= src.saw_on_demand_child_sourcing;
    dst.saw_profiled_child_interface_synthesis |= src.saw_profiled_child_interface_synthesis;
    dst.saw_hierarchy_sibling_routing |= src.saw_hierarchy_sibling_routing;
    dst.saw_hierarchy_registered_sibling_routing |= src.saw_hierarchy_registered_sibling_routing;
    dst.saw_hierarchy_registered_sibling_mixed_support_routing |=
        src.saw_hierarchy_registered_sibling_mixed_support_routing;
    dst.saw_recursive_hierarchy_registered_sibling_mixed_support_routing |=
        src.saw_recursive_hierarchy_registered_sibling_mixed_support_routing;
    dst.saw_hierarchy_direct_sibling_parent_cone_instance_routing |=
        src.saw_hierarchy_direct_sibling_parent_cone_instance_routing;
    dst.saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing |=
        src.saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing;
    dst.saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing |=
        src.saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing;
    dst.saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing |=
        src.saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing;
    dst.saw_hierarchy_registered_parent_composed_routing |=
        src.saw_hierarchy_registered_parent_composed_routing;
    dst.saw_hierarchy_registered_mixed_support_routing |=
        src.saw_hierarchy_registered_mixed_support_routing;
    dst.saw_recursive_hierarchy_registered_mixed_support_routing |=
        src.saw_recursive_hierarchy_registered_mixed_support_routing;
    dst.saw_hierarchy_registered_multistage_routing |=
        src.saw_hierarchy_registered_multistage_routing;
    dst.saw_recursive_hierarchy_registered_multistage_routing |=
        src.saw_recursive_hierarchy_registered_multistage_routing;
    dst.saw_recursive_hierarchy_registered_multistage_mixed_support_routing |=
        src.saw_recursive_hierarchy_registered_multistage_mixed_support_routing;
    dst.saw_hierarchy_registered_multistage_sibling_routing |=
        src.saw_hierarchy_registered_multistage_sibling_routing;
    dst.saw_recursive_hierarchy_registered_multistage_sibling_routing |=
        src.saw_recursive_hierarchy_registered_multistage_sibling_routing;
    dst.saw_hierarchy_registered_multistage_parent_cone_instance_routing |=
        src.saw_hierarchy_registered_multistage_parent_cone_instance_routing;
    dst.saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing |=
        src.saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing;
    dst.saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing |=
        src.saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing;
    dst.saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing |=
        src.saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing;
    dst.saw_hierarchy_parent_composed_parent_cone_instance_flop_routing |=
        src.saw_hierarchy_parent_composed_parent_cone_instance_flop_routing;
    dst.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing |=
        src.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing;
    dst.saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing |=
        src.saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing;
    dst.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing |=
        src.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing;
    dst.saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing |=
        src.saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing;
    dst.saw_hierarchy_registered_parent_cone_instance_routing |=
        src.saw_hierarchy_registered_parent_cone_instance_routing;
    dst.saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing |=
        src.saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing;
    dst.saw_hierarchy_parent_composed_child_inputs |=
        src.saw_hierarchy_parent_composed_child_inputs;
    dst.saw_hierarchy_mixed_support_child_inputs |= src.saw_hierarchy_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_mixed_support_child_inputs;
    dst.saw_hierarchy_parent_cone_instance_routing |=
        src.saw_hierarchy_parent_cone_instance_routing;
    dst.saw_hierarchy_parent_cone_instance_mixed_support_routing |=
        src.saw_hierarchy_parent_cone_instance_mixed_support_routing;
    dst.saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing |=
        src.saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing;
    dst.saw_hierarchy_parent_cone_instance_outputs |=
        src.saw_hierarchy_parent_cone_instance_outputs;
    dst.saw_recursive_hierarchy_parent_cone_instance_outputs |=
        src.saw_recursive_hierarchy_parent_cone_instance_outputs;
    dst.saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs |=
        src.saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs;
    dst.saw_hierarchy_parent_cone_instance_flop_outputs |=
        src.saw_hierarchy_parent_cone_instance_flop_outputs;
    dst.saw_recursive_hierarchy_parent_cone_instance_flop_outputs |=
        src.saw_recursive_hierarchy_parent_cone_instance_flop_outputs;
    dst.saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs |=
        src.saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs;
    dst.saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs |=
        src.saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs;
    dst.saw_multiple_parent_cone_instances_per_parent |=
        src.saw_multiple_parent_cone_instances_per_parent;
    dst.saw_recursive_multiple_parent_cone_instances_per_parent |=
        src.saw_recursive_multiple_parent_cone_instances_per_parent;
    dst.saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs |=
        src.saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs;
    dst.saw_recursive_multiple_parent_cone_instances_per_parent_through_flops |=
        src.saw_recursive_multiple_parent_cone_instances_per_parent_through_flops;
    dst.saw_hierarchy_parent_local_flops |= src.saw_hierarchy_parent_local_flops;
    dst.saw_recursive_hierarchy |= src.saw_recursive_hierarchy;
    dst.saw_per_depth_branching_metrics |= src.saw_per_depth_branching_metrics;
    dst.saw_mixed_leaf_depth_hierarchy |= src.saw_mixed_leaf_depth_hierarchy;
    dst.saw_hierarchy_parent_composition |= src.saw_hierarchy_parent_composition;
    dst.saw_hierarchy_parent_port_composed_outputs |=
        src.saw_hierarchy_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_parent_local_flops |=
        src.saw_recursive_hierarchy_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_3_parent_local_flops |=
        src.saw_recursive_hierarchy_depth_3_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_3_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_3_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_3_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_3_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_4_parent_local_flops |=
        src.saw_recursive_hierarchy_depth_4_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_4_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_4_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_4_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_4_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_5_parent_local_flops |=
        src.saw_recursive_hierarchy_depth_5_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_5_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_5_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_5_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_5_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_6_parent_local_flops |=
        src.saw_recursive_hierarchy_depth_6_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_6_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_6_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_6_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_6_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_7_parent_local_flops |=
        src.saw_recursive_hierarchy_depth_7_parent_local_flops;
    dst.saw_recursive_hierarchy_depth_7_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_7_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_depth_7_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_7_parent_port_composed_outputs;
    dst.saw_comb_only_module |= src.saw_comb_only_module;
    dst.saw_sequential_module |= src.saw_sequential_module;
    dst.saw_priority_encoder |= src.saw_priority_encoder;
    dst.saw_comb_mux_one_hot |= src.saw_comb_mux_one_hot;
    dst.saw_comb_mux_encoded |= src.saw_comb_mux_encoded;
    dst.saw_case_mux |= src.saw_case_mux;
    dst.saw_casez_mux |= src.saw_casez_mux;
    dst.saw_for_fold |= src.saw_for_fold;
    dst.saw_variable_shift |= src.saw_variable_shift;
    dst.saw_flop_mux_one_hot |= src.saw_flop_mux_one_hot;
    dst.saw_flop_mux_encoded |= src.saw_flop_mux_encoded;
    dst.saw_semantic_gate_merge |= src.saw_semantic_gate_merge;
    dst.saw_flop_merge |= src.saw_flop_merge;
}

fn seed_scenario_coverage(coverage: &mut CoverageSummary, scenario: &Scenario) {
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
    if let Some((min_depth, max_depth)) = scenario.config.effective_hierarchy_depth_range() {
        let depth_label = if min_depth == max_depth {
            min_depth.to_string()
        } else {
            format!("{min_depth}:{max_depth}")
        };
        coverage.hierarchy_depths.insert(depth_label);
        coverage.hierarchy_child_source_modes.insert(
            hierarchy_child_source_mode_slug(scenario.config.hierarchy_child_source_mode)
                .to_string(),
        );
        coverage
            .hierarchy_leaf_module_counts
            .insert(scenario.config.num_leaf_modules.to_string());
        if let Some((min_instances, max_instances)) =
            scenario.config.effective_child_instance_range()
        {
            let child_label = if min_instances == max_instances {
                min_instances.to_string()
            } else {
                format!("{min_instances}:{max_instances}")
            };
            coverage.hierarchy_child_instance_counts.insert(child_label);
        }
        if let Some(profile) = child_instances_override_profile_label(
            &scenario.config.child_instances_per_module_by_depth,
        ) {
            coverage
                .hierarchy_child_instance_override_profiles
                .insert(profile);
        }
    }
}

fn hierarchy_child_source_mode_slug(mode: HierarchyChildSourceMode) -> &'static str {
    match mode {
        HierarchyChildSourceMode::Library => "library",
        HierarchyChildSourceMode::OnDemand => "on-demand",
    }
}

fn child_instances_override_profile_label(overrides: &BTreeMap<u32, CountRange>) -> Option<String> {
    if overrides.is_empty() {
        None
    } else {
        Some(
            overrides
                .iter()
                .map(|(depth, range)| format!("{depth}={}:{}", range.min, range.max))
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

fn accumulate_metrics(aggregate: &mut AggregateMetrics, metrics: &Metrics) {
    aggregate.modules += 1;
    aggregate.total_nodes += metrics.num_nodes;
    aggregate.total_gates += metrics.num_gates;
    aggregate.total_flops += metrics.num_flops;
    aggregate.total_shared_nodes += metrics.num_shared_nodes;
    aggregate.total_priority_encoder_blocks += u64::from(metrics.num_priority_encoder_blocks);
    aggregate.total_comb_muxes_one_hot += u64::from(metrics.num_comb_muxes_one_hot);
    aggregate.total_comb_muxes_encoded += u64::from(metrics.num_comb_muxes_encoded);
    aggregate.total_case_mux_blocks += u64::from(metrics.num_case_mux_blocks);
    aggregate.total_casez_mux_blocks += u64::from(metrics.num_casez_mux_blocks);
    aggregate.total_for_fold_blocks += u64::from(metrics.num_for_fold_blocks);
    aggregate.total_semantic_gates_merged += u64::from(metrics.semantic_gates_merged);
    aggregate.total_flops_merged += u64::from(metrics.flops_merged);

    merge_usize_count_map_into_u64(&mut aggregate.gates_by_kind, &metrics.gates_by_kind);
    merge_count_map(
        &mut aggregate.knob_roll_attempts,
        &metrics.knob_roll_attempts,
    );
    merge_count_map(&mut aggregate.knob_roll_fires, &metrics.knob_roll_fires);
}

fn accumulate_tool_summary(
    summary: &mut ToolSummary,
    verilator: Option<&ToolInvocation>,
    yosys: &[ToolInvocation],
) {
    if let Some(verilator) = verilator {
        if verilator.success {
            summary.verilator_passed += 1;
        } else {
            summary.verilator_failed += 1;
        }
    }
    for yosys in yosys {
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

fn accumulate_module_coverage(coverage: &mut CoverageSummary, metrics: &Metrics) {
    if metrics.num_flops == 0 {
        coverage.saw_comb_only_module = true;
    } else {
        coverage.saw_sequential_module = true;
    }

    coverage.saw_instance_module |= metrics.num_instances > 0;
    coverage.saw_instance_output_node |= metrics.num_instance_outputs > 0;
    coverage.saw_priority_encoder |= metrics.num_priority_encoder_blocks > 0;
    coverage.saw_comb_mux_one_hot |= metrics.num_comb_muxes_one_hot > 0;
    coverage.saw_comb_mux_encoded |= metrics.num_comb_muxes_encoded > 0;
    coverage.saw_case_mux |= metrics.num_case_mux_blocks > 0;
    coverage.saw_casez_mux |= metrics.num_casez_mux_blocks > 0;
    coverage.saw_for_fold |= metrics.num_for_fold_blocks > 0;
    coverage.saw_variable_shift |= metrics.num_variable_shift_gates > 0;
    coverage.saw_flop_mux_one_hot |= metrics.flops_mux_one_hot > 0;
    coverage.saw_flop_mux_encoded |= metrics.flops_mux_encoded > 0;
    coverage.saw_semantic_gate_merge |= metrics.semantic_gates_merged > 0;
    coverage.saw_flop_merge |= metrics.flops_merged > 0;

    for gate_kind in metrics.gates_by_kind.keys() {
        coverage.gate_kinds.insert(gate_kind.clone());
        coverage
            .gate_categories
            .insert(gate_kind_category(gate_kind).to_string());
    }
    for knob in metrics.knob_roll_attempts.keys() {
        coverage.knob_attempts_seen.insert(knob.clone());
    }
    for knob in metrics.knob_roll_fires.keys() {
        coverage.knob_fires_seen.insert(knob.clone());
    }
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
        ScenarioSet::Phase3Structured => {
            if !coverage.identity_modes.contains("node-id") {
                gaps.push("missing identity mode node-id".to_string());
            }
            if !coverage.factorization_levels.contains("e-graph") {
                gaps.push("missing factorization level e-graph".to_string());
            }
        }
        ScenarioSet::Phase4Hierarchy => {
            if !coverage.identity_modes.contains("node-id") {
                gaps.push("missing identity mode node-id".to_string());
            }
            if !coverage.factorization_levels.contains("e-graph") {
                gaps.push("missing factorization level e-graph".to_string());
            }
            if !coverage.hierarchy_depths.contains("1") {
                gaps.push("missing hierarchy depth 1".to_string());
            }
            if !coverage.hierarchy_depths.contains("2") {
                gaps.push("missing recursive hierarchy depth 2".to_string());
            }
            if !coverage.hierarchy_depths.contains("2:3") {
                gaps.push("missing mixed recursive hierarchy depth range 2:3".to_string());
            }
            for leaf_count in ["2", "4"] {
                if !coverage.hierarchy_leaf_module_counts.contains(leaf_count) {
                    gaps.push(format!("missing num_leaf_modules scenario {leaf_count}"));
                }
            }
            for child_count in ["2", "4", "2:3", "1:3"] {
                if !coverage
                    .hierarchy_child_instance_counts
                    .contains(child_count)
                {
                    gaps.push(format!("missing child-instance profile {child_count}"));
                }
            }
            for source_mode in ["library", "on-demand"] {
                if !coverage.hierarchy_child_source_modes.contains(source_mode) {
                    gaps.push(format!("missing hierarchy child-source mode {source_mode}"));
                }
            }
            if !coverage
                .hierarchy_child_instance_override_profiles
                .contains("0=4:4,1=2:2")
            {
                gaps.push(
                    "missing per-depth child-instance override profile 0=4:4,1=2:2".to_string(),
                );
            }
        }
    }

    let required_categories: &[&str] = match scenario_set {
        ScenarioSet::Default | ScenarioSet::Phase2Share => &[
            "arithmetic",
            "bitwise",
            "compare",
            "reduce",
            "shift",
            "structural",
        ],
        ScenarioSet::Phase3Structured => &["shift", "structural"],
        ScenarioSet::Phase4Hierarchy => &[
            "arithmetic",
            "bitwise",
            "compare",
            "reduce",
            "shift",
            "structural",
        ],
    };
    for &category in required_categories {
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
    if !coverage.saw_for_fold {
        gaps.push("matrix never emitted a combinational for-fold block".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_hierarchy_design {
        gaps.push("matrix never emitted a hierarchy design".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_multifile_design {
        gaps.push("matrix never emitted a multi-file hierarchy design".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_instance_module {
        gaps.push("matrix never emitted a module with child instances".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_instance_output_node {
        gaps.push("matrix never emitted an instance-output node".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_reused_child_definition {
        gaps.push(
            "matrix never reused a child module definition across multiple instances".to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_underinstantiated_library {
        gaps.push("matrix never left generated leaf definitions unused by the wrapper".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_on_demand_child_sourcing {
        gaps.push("matrix never proved on-demand child sourcing structurally".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_profiled_child_interface_synthesis
    {
        gaps.push("matrix never proved exact profiled child-interface synthesis".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_hierarchy_sibling_routing {
        gaps.push("matrix never proved sibling-routed hierarchy child inputs".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_sibling_routing
    {
        gaps.push(
            "matrix never proved registered sibling-routed hierarchy child inputs".to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_sibling_mixed_support_routing
    {
        gaps.push(
            "matrix never proved direct registered sibling-routed child input bindings mixing parent ports with sibling outputs"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_sibling_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top direct registered sibling-routed child input bindings mixing parent ports with sibling outputs"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_direct_sibling_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved direct sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved recursive non-top direct sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_direct_registered_sibling_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved direct registered sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved recursive non-top direct registered sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_parent_composed_routing
    {
        gaps.push(
            "matrix never proved registered parent-composed hierarchy child input bindings"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_mixed_support_routing
    {
        gaps.push(
            "matrix never proved registered hierarchy child input bindings mixing parent ports with child outputs"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top registered hierarchy child input bindings mixing parent ports with child outputs"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_multistage_routing
    {
        gaps.push(
            "matrix never proved multi-stage registered parent-composed hierarchy child input bindings"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_multistage_routing
    {
        gaps.push(
            "matrix never proved recursive non-top multi-stage registered parent-composed hierarchy child input bindings without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_multistage_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top multi-stage registered mixed-support hierarchy child input bindings without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_multistage_sibling_routing
    {
        gaps.push(
            "matrix never proved multi-stage registered sibling-routed hierarchy child input bindings"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_multistage_sibling_routing
    {
        gaps.push(
            "matrix never proved recursive non-top multi-stage registered sibling-routed hierarchy child input bindings without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_multistage_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved multi-stage registered sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved recursive non-top multi-stage registered sibling-routed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved multi-stage registered parent-composed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved recursive non-top multi-stage registered parent-composed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_composed_parent_cone_instance_flop_routing
    {
        gaps.push(
            "matrix never proved parent-composed child inputs sourced from parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing
    {
        gaps.push(
            "matrix never proved recursive non-top parent-composed child inputs sourced from parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing
    {
        gaps.push(
            "matrix never proved parent-composed child inputs mixed parent ports with parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top parent-composed child inputs mixed parent ports with parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_registered_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved registered parent-composed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved recursive non-top registered parent-composed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top registered parent-cone helper child input bindings mixed with parent ports"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_composed_child_inputs
    {
        gaps.push("matrix never proved parent-composed hierarchy child input bindings".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved parent-composed child input bindings mixing parent ports with sibling outputs without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive non-top parent-composed child input bindings mixing parent ports with sibling outputs without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_cone_instance_routing
    {
        gaps.push(
            "matrix never proved parent-composed child inputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_cone_instance_mixed_support_routing
    {
        gaps.push(
            "matrix never proved parent-composed child inputs mixed parent ports with parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing
    {
        gaps.push(
            "matrix never proved recursive non-top parent-composed child inputs mixed parent ports with parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_cone_instance_outputs
    {
        gaps.push(
            "matrix never proved parent outputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_cone_instance_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top parent outputs sourced from parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top parent outputs mixed parent ports with parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_cone_instance_flop_outputs
    {
        gaps.push(
            "matrix never proved parent outputs sourced from parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_cone_instance_flop_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top parent outputs sourced from parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs
    {
        gaps.push(
            "matrix never proved parent outputs mixed parent ports with parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top parent outputs mixed parent ports with parent-cone helper instances through parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_multiple_parent_cone_instances_per_parent
    {
        gaps.push(
            "matrix never proved multiple parent-cone helper instances in one hierarchy parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_multiple_parent_cone_instances_per_parent
    {
        gaps.push(
            "matrix never proved recursive non-top parents can spend multiple parent-cone helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs
    {
        gaps.push(
            "matrix never proved recursive non-top parents can spend multiple parent-cone helper instances on child-input bindings"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_multiple_parent_cone_instances_per_parent_through_flops
    {
        gaps.push(
            "matrix never proved recursive non-top parents can spend multiple parent-cone helper instances through parent-output flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_hierarchy_parent_local_flops {
        gaps.push("matrix never proved local parent flops in hierarchy modules".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_recursive_hierarchy {
        gaps.push("matrix never emitted a recursive hierarchy design".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_per_depth_branching_metrics {
        gaps.push("matrix never reported per-depth branching metrics".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_mixed_leaf_depth_hierarchy {
        gaps.push("matrix never realized mixed shallow/deep recursive leaf depths".to_string());
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_hierarchy_parent_composition {
        gaps.push(
            "matrix never emitted hierarchy outputs composed above instance outputs".to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_hierarchy_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never emitted hierarchy outputs mixing parent ports with child outputs"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top hierarchy outputs mixing parent ports with child outputs without helper instances or parent-local state"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive non-top hierarchy outputs mixing parent ports, child outputs, and parent-local Qs without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive non-top hierarchy unregistered parent-composed child inputs mixing parent ports, child outputs, and parent-local Qs without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive non-top hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_3_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive depth-3 hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_3_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-3 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_3_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-3 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local state"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-3 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-3 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_4_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive depth-4 hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_4_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-4 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_4_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-4 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local state"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-4 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-4 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_5_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive depth-5 hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_5_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-5 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_5_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-5 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-5 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-5 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_6_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive depth-6 hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_6_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-6 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_6_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-6 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-6 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-6 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_7_parent_local_flops
    {
        gaps.push(
            "matrix never proved recursive depth-7 hierarchy parent-local flops below the top parent"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_7_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-7 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_7_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-7 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local flops"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase3Structured && !coverage.gate_kinds.contains("slice") {
        gaps.push("matrix never emitted a selectable slice gate".to_string());
    }
    if scenario_set == ScenarioSet::Phase3Structured && !coverage.gate_kinds.contains("concat") {
        gaps.push("matrix never emitted a selectable concat gate".to_string());
    }
    if scenario_set == ScenarioSet::Phase3Structured && !coverage.saw_variable_shift {
        gaps.push("matrix never emitted a variable shift".to_string());
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
            "for_fold_prob",
            "coefficient_prob",
            "const_comparand_prob",
            "const_shift_amount_prob",
            "flop_prob",
            "priority_encoder_prob",
            "share_prob",
            "terminal_reuse_prob",
        ],
        ScenarioSet::Phase2Share => &["share_prob", "terminal_reuse_prob", "flop_prob"],
        ScenarioSet::Phase3Structured => &[
            "flop_prob",
            "flop_mux_encoding_prob",
            "comb_mux_prob",
            "comb_mux_encoding_prob",
            "case_mux_prob",
            "casez_mux_prob",
            "for_fold_prob",
            "priority_encoder_prob",
            "const_shift_amount_prob",
        ],
        ScenarioSet::Phase4Hierarchy => &[
            "flop_prob",
            "share_prob",
            "terminal_reuse_prob",
            "comb_mux_prob",
            "case_mux_prob",
            "casez_mux_prob",
            "for_fold_prob",
            "priority_encoder_prob",
            "hierarchy_sibling_route_prob",
            "hierarchy_registered_sibling_route_prob",
            "hierarchy_registered_sibling_mixed_support_prob",
            "hierarchy_registered_child_input_cone_prob",
            "hierarchy_child_input_cone_prob",
            "hierarchy_parent_cone_instance_prob",
            "hierarchy_parent_flop_prob",
        ],
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
        "mux" | "case_mux" | "casez_mux" | "for_fold_xor" | "for_fold_or" | "for_fold_and"
        | "for_fold_add" | "slice" | "concat" => "structural",
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
        ScenarioSet::Phase3Structured => "phase3-structured",
        ScenarioSet::Phase4Hierarchy => "phase4-hierarchy",
    }
}

fn artifact_kind_slug(scenario_set: ScenarioSet) -> &'static str {
    match scenario_set {
        ScenarioSet::Phase4Hierarchy => "design",
        ScenarioSet::Default | ScenarioSet::Phase2Share | ScenarioSet::Phase3Structured => "module",
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

fn escape_paths_for_double_quotes(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| format!("\"{}\"", escape_for_double_quotes(path)))
        .collect::<Vec<_>>()
        .join(" ")
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
            phase3_structured_gate: false,
            phase4_hierarchy_gate: false,
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
    fn phase3_structured_gate_raises_modules_per_scenario_for_surface_gate() {
        let mut cli = test_cli();
        cli.phase3_structured_gate = true;

        let plan = derive_run_plan(&cli, 21);
        assert_eq!(plan.modules_per_scenario, 10);
        assert_eq!(plan.total_modules, 210);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase3_structured_matrix_covers_requested_surface_profiles() {
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase3Structured).expect("build phase3 scenarios");
        let mut strategies = BTreeSet::new();
        let mut names = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            names.insert(scenario.name.clone());
            assert_eq!(scenario.config.identity_mode, IdentityMode::NodeId);
            assert_eq!(
                scenario.config.factorization_level,
                FactorizationLevel::EGraph
            );
        }
        assert_eq!(scenarios.len(), 21);
        assert_eq!(names.len(), 21);
        assert_eq!(
            strategies,
            BTreeSet::from(["interleaved", "sequential", "shuffled"])
        );
        for suffix in [
            "phase3_comb_mux",
            "phase3_case_mux",
            "phase3_casez_mux",
            "phase3_for_fold",
            "phase3_priority_encoder",
            "phase3_flop_mix",
            "phase3_slice_concat_varshift",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "expected at least one phase3 scenario ending with {suffix}"
            );
        }
    }

    #[test]
    fn phase3_structured_coverage_requires_slice_concat_and_variable_shift() {
        let coverage = CoverageSummary {
            construction_strategies: BTreeSet::from([
                "interleaved".to_string(),
                "sequential".to_string(),
                "shuffled".to_string(),
            ]),
            identity_modes: BTreeSet::from(["node-id".to_string()]),
            factorization_levels: BTreeSet::from(["e-graph".to_string()]),
            gate_categories: BTreeSet::from(["shift".to_string(), "structural".to_string()]),
            gate_kinds: BTreeSet::from(["mux".to_string()]),
            knob_attempts_seen: BTreeSet::from([
                "flop_prob".to_string(),
                "flop_mux_encoding_prob".to_string(),
                "comb_mux_prob".to_string(),
                "comb_mux_encoding_prob".to_string(),
                "case_mux_prob".to_string(),
                "casez_mux_prob".to_string(),
                "for_fold_prob".to_string(),
                "priority_encoder_prob".to_string(),
                "const_shift_amount_prob".to_string(),
            ]),
            saw_comb_only_module: true,
            saw_sequential_module: true,
            saw_priority_encoder: true,
            saw_comb_mux_one_hot: true,
            saw_comb_mux_encoded: true,
            saw_case_mux: true,
            saw_casez_mux: true,
            saw_for_fold: true,
            saw_flop_mux_one_hot: true,
            saw_flop_mux_encoded: true,
            ..CoverageSummary::default()
        };

        let gaps = compute_coverage_gaps(ScenarioSet::Phase3Structured, &coverage, None);
        assert!(gaps.iter().any(|gap| gap.contains("selectable slice")));
        assert!(gaps.iter().any(|gap| gap.contains("selectable concat")));
        assert!(gaps.iter().any(|gap| gap.contains("variable shift")));
    }

    #[test]
    fn phase4_hierarchy_gate_raises_designs_per_scenario_for_matrix() {
        let mut cli = test_cli();
        cli.phase4_hierarchy_gate = true;
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase4Hierarchy).expect("build phase4 scenarios");

        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(plan.modules_per_scenario, 4);
        assert_eq!(plan.total_modules, 756);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles() {
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase4Hierarchy).expect("build phase4 scenarios");
        let mut strategies = BTreeSet::new();
        let mut leaf_counts = BTreeSet::new();
        let mut child_counts = BTreeSet::new();
        let mut child_source_modes = BTreeSet::new();
        let mut override_profiles = BTreeSet::new();
        let mut range_depths = BTreeSet::new();
        let mut names = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            leaf_counts.insert(scenario.config.num_leaf_modules);
            child_counts.insert(
                scenario
                    .config
                    .effective_child_instance_range()
                    .expect("phase4 scenarios should be hierarchical"),
            );
            child_source_modes.insert(hierarchy_child_source_mode_slug(
                scenario.config.hierarchy_child_source_mode,
            ));
            if let Some(profile) = child_instances_override_profile_label(
                &scenario.config.child_instances_per_module_by_depth,
            ) {
                override_profiles.insert(profile);
            }
            range_depths.insert(
                scenario
                    .config
                    .effective_hierarchy_depth_range()
                    .expect("phase4 scenarios should be hierarchical"),
            );
            names.insert(scenario.name.clone());
            assert_eq!(scenario.config.identity_mode, IdentityMode::NodeId);
            assert_eq!(
                scenario.config.factorization_level,
                FactorizationLevel::EGraph
            );
            assert!(
                scenario.config.hierarchy_child_input_cone_prob == 1.0
                    || scenario.config.hierarchy_registered_sibling_route_prob == 1.0
                    || scenario.config.hierarchy_registered_child_input_cone_prob == 1.0
                    || scenario.config.hierarchy_parent_cone_instance_prob == 1.0
                    || scenario
                        .name
                        .ends_with("phase4_recur_d2_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d2_stateful_parent_port_composed_output")
                    || scenario.name.ends_with("phase4_recur_d2_parent_state")
                    || scenario.name.ends_with("phase4_recur_d3_parent_state")
                    || scenario.name.ends_with("phase4_recur_d4_parent_state")
                    || scenario.name.ends_with("phase4_recur_d5_parent_state")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d3_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d3_stateful_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d4_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d4_stateful_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d5_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d5_stateful_parent_port_composed_output")
                    || scenario.name.ends_with("phase4_recur_d6_parent_state")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d6_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d6_stateful_parent_port_composed_output")
                    || scenario.name.ends_with("phase4_recur_d7_parent_state")
                    || scenario
                        .name
                        .ends_with("phase4_recur_d7_parent_port_composed_output")
            );
        }
        assert_eq!(scenarios.len(), 189);
        assert_eq!(names.len(), 189);
        assert_eq!(leaf_counts, BTreeSet::from([0, 2, 4]));
        assert_eq!(
            child_counts,
            BTreeSet::from([(1, 3), (2, 2), (2, 3), (4, 4)])
        );
        assert_eq!(child_source_modes, BTreeSet::from(["library", "on-demand"]));
        assert_eq!(
            range_depths,
            BTreeSet::from([
                (1, 1),
                (2, 2),
                (2, 3),
                (3, 3),
                (4, 4),
                (5, 5),
                (6, 6),
                (7, 7)
            ])
        );
        assert_eq!(
            override_profiles,
            BTreeSet::from(["0=4:4,1=2:2".to_string()])
        );
        assert_eq!(
            strategies,
            BTreeSet::from(["interleaved", "sequential", "shuffled"])
        );
        for suffix in [
            "phase4_hier2_inst2_comb",
            "phase4_hier2_inst4_seq",
            "phase4_hier4_inst2_comb",
            "phase4_recur_d2_b2to3_comb",
            "phase4_recur_profile_d2_top4_mid2_seq",
            "phase4_recur_d2to3_b2_mixed_comb",
            "phase4_recur_d2_b2_ondemand_comb",
            "phase4_hier2_inst4_parent_state",
            "phase4_hier2_inst4_registered_sibling_state",
            "phase4_hier2_inst4_registered_sibling_multistage_state",
            "phase4_hier2_inst4_registered_sibling_mixed_support_state",
            "phase4_recur_d2_registered_sibling_mixed_support_state",
            "phase4_recur_d2_parent_composed_mixed_support_child_input",
            "phase4_recur_d2_parent_port_composed_output",
            "phase4_recur_d2_stateful_parent_port_composed_output",
            "phase4_recur_d2_stateful_parent_composed_mixed_support_child_input",
            "phase4_recur_d2_parent_state",
            "phase4_recur_d3_parent_state",
            "phase4_recur_d3_parent_composed_mixed_support_child_input",
            "phase4_recur_d3_parent_port_composed_output",
            "phase4_recur_d3_stateful_parent_port_composed_output",
            "phase4_recur_d3_stateful_parent_composed_mixed_support_child_input",
            "phase4_recur_d4_parent_state",
            "phase4_recur_d4_parent_composed_mixed_support_child_input",
            "phase4_recur_d4_parent_port_composed_output",
            "phase4_recur_d4_stateful_parent_port_composed_output",
            "phase4_recur_d4_stateful_parent_composed_mixed_support_child_input",
            "phase4_recur_d5_parent_state",
            "phase4_recur_d5_parent_composed_mixed_support_child_input",
            "phase4_recur_d5_parent_port_composed_output",
            "phase4_recur_d5_stateful_parent_port_composed_output",
            "phase4_recur_d5_stateful_parent_composed_mixed_support_child_input",
            "phase4_recur_d6_parent_state",
            "phase4_recur_d6_parent_composed_mixed_support_child_input",
            "phase4_recur_d6_parent_port_composed_output",
            "phase4_recur_d6_stateful_parent_port_composed_output",
            "phase4_recur_d6_stateful_parent_composed_mixed_support_child_input",
            "phase4_recur_d7_parent_state",
            "phase4_recur_d7_parent_composed_mixed_support_child_input",
            "phase4_recur_d7_parent_port_composed_output",
            "phase4_recur_d2_registered_sibling_multistage_state",
            "phase4_hier2_inst4_direct_sibling_parent_cone_instance",
            "phase4_recur_d2_direct_sibling_parent_cone_instance",
            "phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state",
            "phase4_hier2_inst4_direct_registered_sibling_parent_cone_instance_state",
            "phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state",
            "phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state",
            "phase4_hier2_inst4_registered_child_input_cone_state",
            "phase4_recur_d2_registered_mixed_child_input_state",
            "phase4_recur_d2_registered_multistage_child_input_state",
            "phase4_recur_d2_registered_parent_cone_instance_state",
            "phase4_hier2_inst4_parent_cone_instance",
            "phase4_hier2_inst4_parent_output_cone_instance",
            "phase4_recur_d2_parent_output_cone_instance",
            "phase4_hier2_inst4_parent_output_cone_instance_state",
            "phase4_recur_d2_parent_output_cone_instance_state",
            "phase4_hier2_inst4_parent_cone_instance_budget3",
            "phase4_recur_d2_parent_cone_instance_budget3",
            "phase4_hier2_inst4_registered_parent_cone_instance_state",
            "phase4_hier2_inst4_registered_parent_cone_instance_multistage_state",
            "phase4_recur_d2_registered_parent_cone_instance_multistage_state",
            "phase4_hier2_inst4_parent_cone_instance_state",
            "phase4_recur_d2_parent_cone_instance_state",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "expected at least one phase4 scenario ending with {suffix}"
            );
        }
    }

    #[test]
    fn phase4_hierarchy_coverage_requires_design_facts() {
        let coverage = CoverageSummary {
            construction_strategies: BTreeSet::from([
                "interleaved".to_string(),
                "sequential".to_string(),
                "shuffled".to_string(),
            ]),
            identity_modes: BTreeSet::from(["node-id".to_string()]),
            factorization_levels: BTreeSet::from(["e-graph".to_string()]),
            hierarchy_depths: BTreeSet::from(["1".to_string()]),
            hierarchy_leaf_module_counts: BTreeSet::from(["2".to_string()]),
            hierarchy_child_instance_counts: BTreeSet::from(["2".to_string()]),
            hierarchy_child_source_modes: BTreeSet::from(["library".to_string()]),
            gate_categories: BTreeSet::from([
                "arithmetic".to_string(),
                "bitwise".to_string(),
                "compare".to_string(),
                "reduce".to_string(),
                "shift".to_string(),
                "structural".to_string(),
            ]),
            knob_attempts_seen: BTreeSet::from([
                "flop_prob".to_string(),
                "share_prob".to_string(),
                "terminal_reuse_prob".to_string(),
                "comb_mux_prob".to_string(),
                "case_mux_prob".to_string(),
                "casez_mux_prob".to_string(),
                "for_fold_prob".to_string(),
                "priority_encoder_prob".to_string(),
            ]),
            saw_comb_only_module: true,
            saw_sequential_module: true,
            saw_priority_encoder: true,
            saw_comb_mux_one_hot: true,
            saw_comb_mux_encoded: true,
            saw_case_mux: true,
            saw_casez_mux: true,
            saw_for_fold: true,
            saw_flop_mux_one_hot: true,
            saw_flop_mux_encoded: true,
            ..CoverageSummary::default()
        };

        let gaps = compute_coverage_gaps(ScenarioSet::Phase4Hierarchy, &coverage, None);
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("recursive hierarchy depth 2")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("mixed recursive hierarchy depth range 2:3")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("num_leaf_modules scenario 4")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("child-instance profile 4")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("child-source mode on-demand")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("per-depth child-instance override profile")));
        assert!(gaps.iter().any(|gap| gap.contains("hierarchy design")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("multi-file hierarchy design")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("module with child instances")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("on-demand child sourcing")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("exact profiled child-interface synthesis")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("sibling-routed hierarchy child inputs")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("registered sibling-routed hierarchy child inputs")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "direct registered sibling-routed child input bindings mixing parent ports with sibling outputs",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top direct registered sibling-routed child input bindings mixing parent ports with sibling outputs",
            )
        }));
        assert!(gaps.iter().any(|gap| gap.contains(
            "direct sibling-routed child inputs sourced from parent-cone helper instances"
        )));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top direct sibling-routed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps.iter().any(|gap| gap.contains(
            "direct registered sibling-routed child inputs sourced from parent-cone helper instances"
        )));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top direct registered sibling-routed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("registered parent-composed hierarchy child input bindings")));
        assert!(gaps.iter().any(
            |gap| gap.contains("registered hierarchy child input bindings mixing parent ports")
        ));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top registered hierarchy child input bindings mixing parent ports",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("multi-stage registered parent-composed hierarchy child input bindings")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top multi-stage registered parent-composed hierarchy child input bindings without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top multi-stage registered mixed-support hierarchy child input bindings without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("multi-stage registered sibling-routed hierarchy child input bindings")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top multi-stage registered sibling-routed hierarchy child input bindings without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "multi-stage registered sibling-routed child inputs sourced from parent-cone helper"
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top multi-stage registered sibling-routed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "multi-stage registered parent-composed child inputs sourced from parent-cone helper"
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top multi-stage registered parent-composed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "parent-composed child inputs sourced from parent-cone helper instances through parent-local flops"
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent-composed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "parent-composed child inputs mixed parent ports with parent-cone helper instances through parent-local flops"
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent-composed child inputs mixed parent ports with parent-cone helper instances through parent-local flops",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("registered parent-composed child inputs sourced from parent-cone helper")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top registered parent-composed child inputs sourced from parent-cone helper",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top registered parent-cone helper child input bindings mixed with parent ports",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("parent-composed hierarchy child input bindings")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "parent-composed child input bindings mixing parent ports with sibling outputs without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent-composed child input bindings mixing parent ports with sibling outputs without helper instances",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("parent-cone helper instances")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "parent-composed child inputs mixed parent ports with parent-cone helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent-composed child inputs mixed parent ports with parent-cone helper instances",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("parent outputs sourced from parent-cone helper instances")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent outputs sourced from parent-cone helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent outputs mixed parent ports with parent-cone helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("parent outputs sourced from parent-cone helper instances through parent-local flops")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent outputs sourced from parent-cone helper instances through parent-local flops",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "parent outputs mixed parent ports with parent-cone helper instances through parent-local flops"
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parent outputs mixed parent ports with parent-cone helper instances through parent-local flops",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("multiple parent-cone helper instances")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parents can spend multiple parent-cone helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parents can spend multiple parent-cone helper instances on child-input bindings",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top parents can spend multiple parent-cone helper instances through parent-output flops",
            )
        }));
        assert!(gaps.iter().any(|gap| gap.contains("local parent flops")));
        assert!(gaps.iter().any(|gap| gap.contains("instance-output node")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("reused a child module definition")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("left generated leaf definitions unused")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("recursive hierarchy design")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("per-depth branching metrics")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("mixed shallow/deep recursive leaf depths")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("composed above instance outputs")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("mixing parent ports with child outputs")));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top hierarchy outputs mixing parent ports with child outputs without helper instances or parent-local state",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top hierarchy outputs mixing parent ports, child outputs, and parent-local Qs without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive non-top hierarchy unregistered parent-composed child inputs mixing parent ports, child outputs, and parent-local Qs without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("recursive non-top hierarchy parent-local flops below the top parent")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("recursive depth-3 hierarchy parent-local flops below the top parent")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-3 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-3 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local state",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-3 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-3 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("recursive depth-4 hierarchy parent-local flops below the top parent")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-4 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-4 hierarchy parent outputs mixing parent ports with child outputs below the top parent without helper instances or parent-local state",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-4 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-4 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances",
            )
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains("recursive depth-5 hierarchy parent-local flops below the top parent")
        }));
        assert!(gaps.iter().any(|gap| {
            gap.contains(
                "recursive depth-5 hierarchy unregistered parent-composed child-input bindings mixing parent ports with child outputs below the top parent without helper instances",
            )
        }));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_sibling_route_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_registered_sibling_route_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_registered_sibling_mixed_support_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_registered_child_input_cone_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_child_input_cone_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_parent_cone_instance_prob")));
        assert!(gaps
            .iter()
            .any(|gap| gap.contains("hierarchy_parent_flop_prob")));
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
            saw_for_fold: true,
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
    fn hierarchy_yosys_mode_expands_to_expected_invocations() {
        let paths = vec![PathBuf::from("/tmp/a.sv"), PathBuf::from("/tmp/b.sv")];

        let without = yosys_design_invocations(YosysMode::WithoutAbc, &paths, "top_mod");
        assert_eq!(without.len(), 1);
        assert!(without[0].1.contains("read_verilog -sv"));
        assert!(without[0].1.contains("\"/tmp/a.sv\" \"/tmp/b.sv\""));
        assert!(without[0]
            .1
            .contains("synth -top top_mod -noabc; stat; check"));

        let with = yosys_design_invocations(YosysMode::WithAbc, &paths, "top_mod");
        assert_eq!(with.len(), 1);
        assert!(with[0]
            .1
            .contains("synth -top top_mod -noabc; abc -fast; opt -fast; stat; check"));
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

    #[test]
    fn fast_resume_restores_generator_state_for_next_design() {
        let out_root = temp_test_dir("resume-fast-design");
        let scenario = make_scenario(
            "resume_fast_design_case",
            "resume fast hierarchy path test",
            with_hierarchy_wrapper(
                share_heavy_comb_only_config(ConstructionStrategy::Interleaved, 23, 0.9),
                1,
                1,
            ),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let cli = test_cli_resume();
        let mut baseline = Generator::new(scenario.config.clone());
        let prepared0 = prepare_design(&mut baseline, &scenario_dir, 0).unwrap();
        for module in &prepared0.modules {
            fs::write(&module.sv_path, &module.sv_text).unwrap();
        }
        let report0 = run_design_tools(&cli, &prepared0).unwrap();
        let checkpoint0 = baseline.checkpoint();
        write_design_checkpoint(
            &cli,
            &prepared0.paths.checkpoint_path,
            &report0,
            &checkpoint0,
            Some(TEST_RUNTIME_FINGERPRINT),
            &prepared0.modules,
        )
        .unwrap();

        let expected1 = prepare_design(&mut baseline, &scenario_dir, 1).unwrap();

        let checkpoint = load_design_checkpoint(&prepared0.paths.checkpoint_path)
            .unwrap()
            .expect("checkpoint");
        let mut resumed = Generator::new(scenario.config.clone());
        let report = try_fast_resume_design_checkpoint(
            &mut resumed,
            &cli,
            &scenario_dir,
            &checkpoint,
            Some(TEST_RUNTIME_FINGERPRINT),
        )
        .unwrap();
        assert!(report.is_some());

        let actual1 = prepare_design(&mut resumed, &scenario_dir, 1).unwrap();
        assert_eq!(actual1.top, expected1.top);
        let actual_files: Vec<_> = actual1
            .modules
            .iter()
            .map(|module| module.file.clone())
            .collect();
        let expected_files: Vec<_> = expected1
            .modules
            .iter()
            .map(|module| module.file.clone())
            .collect();
        assert_eq!(actual_files, expected_files);

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn run_design_tools_reports_design_metrics() {
        let out_root = temp_test_dir("design-metrics-report");
        let scenario = make_scenario(
            "design_metrics_case",
            "design metrics report test",
            with_hierarchy_wrapper(
                share_heavy_comb_only_config(ConstructionStrategy::Interleaved, 29, 0.9),
                2,
                4,
            ),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let mut cli = test_cli();
        cli.skip_verilator = true;
        cli.skip_yosys = true;

        let mut generator = Generator::new(scenario.config.clone());
        let prepared = prepare_design(&mut generator, &scenario_dir, 0).unwrap();
        let report = run_design_tools(&cli, &prepared).unwrap();

        assert_eq!(report.metrics, prepared.metrics);
        assert_eq!(report.metrics.design, report.top);
        assert_eq!(report.metrics.num_instances, report.hierarchy.top_instances);
        assert_eq!(
            report.metrics.num_unique_instantiated_modules,
            report.hierarchy.unique_instantiated_modules
        );

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn recursive_hierarchy_facts_follow_design_metrics() {
        let out_root = temp_test_dir("recursive-hierarchy-facts");
        let mut cfg = share_heavy_comb_only_config(ConstructionStrategy::Interleaved, 31, 0.4);
        cfg.min_inputs = 2;
        cfg.max_inputs = 3;
        cfg.min_outputs = 1;
        cfg.max_outputs = 2;
        cfg.min_width = 1;
        cfg.max_width = 4;
        cfg.max_depth = 3;
        let scenario = make_scenario(
            "recursive_hierarchy_case",
            "recursive hierarchy facts test",
            with_recursive_hierarchy_profile(
                cfg,
                2,
                2,
                1,
                2,
                BTreeMap::from([
                    (0, CountRange { min: 2, max: 2 }),
                    (1, CountRange { min: 1, max: 1 }),
                ]),
            ),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let mut generator = Generator::new(scenario.config.clone());
        let prepared = prepare_design(&mut generator, &scenario_dir, 0).unwrap();

        assert_eq!(
            prepared.hierarchy.library_modules,
            prepared.metrics.num_library_modules
        );
        assert_eq!(
            prepared.hierarchy.unique_instantiated_modules,
            prepared.metrics.num_unique_instantiated_modules
        );
        assert_eq!(
            prepared.hierarchy.underinstantiated_library,
            prepared.metrics.num_unused_module_definitions > 0
        );
        assert_eq!(
            prepared.hierarchy.reused_child_definition,
            prepared
                .metrics
                .instantiated_module_histogram
                .values()
                .any(|&count| count > 1)
        );

        let _ = fs::remove_dir_all(out_root);
    }

    #[test]
    fn design_manifest_embeds_design_metrics() {
        let out_root = temp_test_dir("design-metrics-manifest");
        let scenario = make_scenario(
            "design_manifest_case",
            "design metrics manifest test",
            with_hierarchy_wrapper(
                share_heavy_comb_only_config(ConstructionStrategy::Interleaved, 31, 0.9),
                4,
                2,
            ),
        )
        .expect("scenario");
        let scenario_dir = out_root.join(&scenario.name);
        fs::create_dir_all(&scenario_dir).expect("create scenario dir");

        let mut cli = test_cli();
        cli.skip_verilator = true;
        cli.skip_yosys = true;

        let mut generator = Generator::new(scenario.config.clone());
        let prepared = prepare_design(&mut generator, &scenario_dir, 0).unwrap();
        let report = run_design_tools(&cli, &prepared).unwrap();

        write_design_scenario_manifest(&scenario_dir, &scenario, std::slice::from_ref(&report))
            .unwrap();

        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(scenario_dir.join("manifest.json")).unwrap())
                .unwrap();
        let design = &manifest["designs"][0];
        assert_eq!(
            design["metrics"]["num_instances"].as_u64(),
            Some(report.metrics.num_instances as u64)
        );
        assert_eq!(
            design["metrics"]["num_unused_leaf_modules"].as_u64(),
            Some(report.metrics.num_unused_leaf_modules as u64)
        );
        assert_eq!(
            design["hierarchy"]["top_instances"].as_u64(),
            Some(report.hierarchy.top_instances as u64)
        );
        assert_eq!(design["top"].as_str(), Some(report.top.as_str()));

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
