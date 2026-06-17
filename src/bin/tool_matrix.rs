use anvil::config::{
    ConstructionStrategy, CountRange, FactorizationLevel, HierarchyChildSourceMode, IdentityMode,
    SvVersion,
};
// AGENT-INTROSPECTION-MCP.5.1 — the hardened downstream-tool invocations live in
// the library (`anvil::downstream`) so the agent `validate`/`minimize` tools
// reuse the same vetted command lines. `DOWNSTREAM-ADAPTER-EXPANSION.2a.3` routes
// this binary's per-unit acceptance columns through the closed `Adapter` registry
// too (`AcceptanceTool::adapter().run(&AdapterRunCx{..})`) rather than calling the
// `run_*` primitives directly — byte-identical, because each built-in adapter
// delegates verbatim to those primitives, so the serialized `ToolInvocation`
// shape (and banked reports + `--resume`) stay unchanged.
use anvil::downstream::{
    tool_version, yosys_mode_slug, AcceptanceTool, AdapterRunCx, AdapterTarget, ToolInvocation,
    ValidateReport, YosysMode,
};
// ACCEPTANCE-DIVERGENCE-HUNTING.2c.2 — the opt-in acceptance-divergence column
// reuses the one shared detector in `anvil::divergence`: `classify_report`
// projects the per-tool invocations this binary already ran into accept/warn/
// reject verdicts and classifies any disagreement. There is one classifier (the
// hunt loop shares it); this binary adds no second copy.
use anvil::divergence::{self, DivergenceReport};
// BUG-HUNT-ORCHESTRATION.2a — the per-module diff-sim run+compare pipeline
// (the `DiffSimReport` + the SV-text-driven testbench + the dual-simulator
// run+compare) now lives in `anvil::diff_sim` so the bug-hunt loop (decision
// `0018`) and the acceptance-divergence lane reuse the same hardened surface.
// This binary `use`s the report type; the serialized shape is unchanged.
use anvil::diff_sim::DiffSimReport;
use anvil::metrics::{DesignMetrics, Metrics};
use anvil::{Config, Design, Generator, GeneratorCheckpoint};
use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const PHASE1_MIN_TOTAL_MODULES: usize = 1000;
const PHASE2_SHARE_MIN_TOTAL_MODULES: usize = 216;
const PHASE3_STRUCTURED_MIN_TOTAL_MODULES: usize = 210;
const PHASE4_HIERARCHY_MIN_DESIGNS_PER_SCENARIO: usize = 4;
const SIGNOFF_KNOB_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 4;
const SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 2;
const FUNCTION_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 4;
const GENERATE_LOOP_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 4;
const TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 4;
const CONE_FUNCTION_SWEEP_MIN_UNITS_PER_SCENARIO: usize = 4;

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

    /// Elevate the run to the repo-owned signoff knob-sweep gate
    /// (`SIGNOFF-AUTOMATION-EXPANSION.2b`): run the focused
    /// richer-knob-sweep matrix (operand/mux-arm duplication,
    /// array-packed aggregate, memory×fsm interplay) and require its
    /// four coverage facts.
    #[arg(long)]
    signoff_knob_sweep_gate: bool,

    /// Elevate the run to the repo-owned per-version SystemVerilog gate
    /// (`SV-VERSION-TARGETING.2b.2b`): sweep the three IEEE 1800 targets
    /// (2012/2017/2023) over a focused corpus, run Verilator in the
    /// matching `--language 1800-20xx` standard mode, and require a
    /// per-version `saw_sv_version_*_targeted_acceptance` coverage fact.
    #[arg(long)]
    sv_version_gate: bool,

    /// Elevate the run to the repo-owned combinational `function
    /// automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.2b.2b`):
    /// force `function_emit_prob = 1.0` over comb-only DUTs across the
    /// three construction strategies and require the
    /// `saw_combinational_function_emit` coverage fact, proving the
    /// emitted functions are accepted warning-clean by Verilator + both
    /// Yosys modes (+ Icarus when `--iverilog-compile` is also set).
    #[arg(long)]
    function_emit_gate: bool,

    /// Elevate the run to the repo-owned `generate for` loop emit gate
    /// (`STRUCTURED-EMISSION-EXPANSION.4b.2b`): force
    /// `generate_loop_emit_prob = 1.0` over comb-only DUTs across the three
    /// construction strategies and require the `saw_generate_loop_emit`
    /// coverage fact, proving the emitted loops are accepted warning-clean by
    /// Verilator + both Yosys modes (+ Icarus when `--iverilog-compile` is
    /// also set).
    #[arg(long)]
    generate_loop_gate: bool,

    /// Elevate the run to the repo-owned combinational `task automatic`
    /// emit gate (`STRUCTURED-EMISSION-EXPANSION.6b.2b`): force
    /// `task_emit_prob = 1.0` over comb-only DUTs across the three
    /// construction strategies and require the `saw_combinational_task_emit`
    /// coverage fact, proving the emitted tasks are accepted warning-clean by
    /// Verilator + both Yosys modes (+ Icarus when `--iverilog-compile` is
    /// also set).
    #[arg(long)]
    task_emit_gate: bool,

    /// Elevate the run to the repo-owned multi-gate-cone `function automatic`
    /// emit gate (`STRUCTURED-EMISSION-EXPANSION.10b.2`): force
    /// `cone_function_emit_prob = 1.0` over comb-only DUTs across the three
    /// construction strategies and require the `saw_cone_function_emit`
    /// coverage fact, proving the emitted cone functions are accepted
    /// warning-clean by Verilator + both Yosys modes (+ Icarus when
    /// `--iverilog-compile` is also set).
    #[arg(long)]
    cone_function_gate: bool,

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

    /// Opt-in Icarus Verilog compile/elaboration acceptance column.
    /// When enabled, each generated artifact is compiled with
    /// `iverilog -g2012` after the normal Verilator/Yosys checks.
    /// This is lighter than `--diff-sim`: it proves an additional
    /// open-source simulator/frontend can compile the emitted SV, but
    /// it does not run a testbench or compare traces.
    #[arg(long)]
    iverilog_compile: bool,

    /// Icarus Verilog executable to run when `--iverilog-compile` is set.
    #[arg(long, default_value = "iverilog")]
    iverilog_bin: String,

    /// Opt-in `sv2v` SystemVerilog→Verilog-2005 transpile acceptance
    /// column (`DOWNSTREAM-ADAPTER-EXPANSION.2b.2`, decision `0020`).
    /// When enabled, each generated artifact is transpiled with `sv2v`
    /// after the normal Verilator/Yosys checks: a clean transpile
    /// accepts, a non-zero exit or a warning is a finding. Like
    /// `--iverilog-compile`, this is an acceptance gate, not a
    /// behavioural testbench — the transpiled Verilog is discarded.
    /// `sv2v` is absent on most hosts; when so this column is a
    /// friendly no-op (the run records a spawn failure, never a panic).
    #[arg(long)]
    sv2v: bool,

    /// `sv2v` executable to run when `--sv2v` is set.
    #[arg(long, default_value = "sv2v")]
    sv2v_bin: String,

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

    /// Opt-in cross-simulator differential gate
    /// (`DIFFERENTIAL-SIMULATION.3b.2`). When set, every scenario
    /// selected by the per-axis subset selector
    /// (combinational/sequential-flop/hierarchy/memory/fsm; capped
    /// K=5) gets an iverilog↔verilator byte-equal-trace check after
    /// Verilator and Yosys are both clean. Triggers the
    /// `saw_design_with_cross_simulator_agreement` coverage fact
    /// when at least one DUT in the subset passes. Friendly no-op
    /// when either simulator is absent (`tools_present()` probe).
    #[arg(long)]
    diff_sim: bool,

    /// Opt-in acceptance-divergence column
    /// (`ACCEPTANCE-DIVERGENCE-HUNTING.2c.2`, decision `0019`). When
    /// set, every unit in the per-axis subset (the same
    /// `select_diff_sim_subset` / `classify_diff_sim_axis` selector,
    /// capped K=5) gets a `DivergenceReport`: each tool the matrix
    /// **already ran** is projected to an accept/warn/reject verdict and
    /// any disagreement is classified (`accept_reject` / `accept_warn` /
    /// `warn_reject`). Unlike `--diff-sim` it spawns **no** extra tool —
    /// it is a pure projection of the existing per-unit invocations — so
    /// it does **not** require the tools to be clean first (a divergence
    /// is most interesting exactly when one tool rejects what another
    /// accepts). Lights the **opportunistic** `saw_acceptance_divergence`
    /// fact — never a required coverage gate, because all-agree is the
    /// valid-by-construction steady state.
    #[arg(long)]
    divergence: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Scenario {
    name: String,
    description: String,
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModuleReport {
    file: String,
    name: String,
    metrics: Metrics,
    verilator: Option<ToolInvocation>,
    yosys: Vec<ToolInvocation>,
    /// `SIGNOFF-SURFACE-EXPANSION.3` — opt-in Icarus Verilog
    /// compile/elaboration acceptance column. `None` unless
    /// `--iverilog-compile` was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    iverilog_compile: Option<ToolInvocation>,
    /// `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — opt-in `sv2v`
    /// SystemVerilog→Verilog-2005 transpile acceptance column. `None`
    /// unless `--sv2v` was set. Like `iverilog_compile`, the field is
    /// off the wire when `None`, so default runs stay byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sv2v: Option<ToolInvocation>,
    /// `DIFFERENTIAL-SIMULATION.3b.2` — opt-in cross-simulator
    /// byte-equal-trace report. `None` when `--diff-sim` was not
    /// set OR this scenario was not in the per-axis subset OR
    /// Verilator/Yosys were not both clean. `Some(DiffSimReport)`
    /// records the gate outcome and (on mismatch) a retained
    /// counterexample per the Phase-7 doctrine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diff_sim: Option<DiffSimReport>,
    /// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — opt-in acceptance-divergence
    /// column (decision `0019`). `None` unless `--divergence` was set AND this
    /// scenario was in the per-axis subset. `Some(DivergenceReport)` records the
    /// accept/warn/reject verdict of each tool the matrix already ran on this
    /// module and any classified disagreement — a pure projection of the
    /// existing per-tool invocations (no extra tool spawn), so it is populated
    /// regardless of whether the tools accepted. Mirrors the `diff_sim` column.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    divergence: Option<DivergenceReport>,
    /// `SV-VERSION-TARGETING.3b.2b` — `true` iff this module's emitted SV
    /// carries the IEEE 1800-2023 `union soft` up-opt overlay (the
    /// `soft_union_slice_prob` low-bits-slice rendering). Lights the
    /// `saw_sv_version_2023_soft_union_upopt` coverage fact. Such a module
    /// runs Verilator-only — Yosys/Icarus reject the `union soft` syntax and
    /// are a recorded no-op (decision `0010`).
    #[serde(default)]
    emitted_soft_union_overlay: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.2b.2b` — `true` iff this module's
    /// emitted SV carries at least one combinational `function automatic`
    /// emit-projection (the `function_emit_prob` rendering of a marked
    /// combinational gate; decision `0012`). Detected from the emitted SV
    /// text, mirroring the `emitted_soft_union_overlay` precedent. Lights
    /// the `saw_combinational_function_emit` coverage fact when the module
    /// is also accepted by the downstream tools. Unlike the `union soft`
    /// overlay, a synthesizable function is accepted by every tool, so
    /// such a module runs the full Verilator + Yosys (+ Icarus) plan.
    #[serde(default)]
    emitted_combinational_function: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.4b.2b` — `true` iff this module's
    /// emitted SV carries at least one `generate for` loop emit-projection
    /// (the `generate_loop_emit_prob` rendering of a marked `{N{x}}`
    /// replication; decision `0013`). Detected from the emitted SV text
    /// (`generate`/`genvar`), mirroring the `emitted_combinational_function`
    /// precedent. Lights the `saw_generate_loop_emit` coverage fact when the
    /// module is also accepted by the downstream tools. Like a function (and
    /// unlike the `union soft` overlay), a `generate for` is universally
    /// synthesizable, so such a module runs the full Verilator + Yosys (+
    /// Icarus) plan.
    #[serde(default)]
    emitted_generate_loop: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.6b.2b` — `true` iff this module's
    /// emitted SV carries at least one combinational `task automatic`
    /// emit-projection (the `task_emit_prob` rendering of a marked
    /// combinational gate; decision `0014`). Detected from the emitted SV
    /// text (`task automatic`), mirroring the `emitted_combinational_function`
    /// precedent. Lights the `saw_combinational_task_emit` coverage fact when
    /// the module is also accepted by the downstream tools. Like a function
    /// (and unlike the `union soft` overlay), a combinational `task` is
    /// universally synthesizable, so such a module runs the full Verilator +
    /// Yosys (+ Icarus) plan.
    #[serde(default)]
    emitted_combinational_task: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.10b.2` — `true` iff this module's
    /// emitted SV carries at least one multi-gate-cone `function automatic`
    /// emit-projection (the `cone_function_emit_prob` rendering of a marked
    /// combinational cone; decision `0016`). Detected from the emitted SV text
    /// (the `<root>__cf(` call/decl token, distinct from the single-gate
    /// `function_emit` `<wire>__f(` surface), mirroring the
    /// `emitted_combinational_task` precedent. Lights the
    /// `saw_cone_function_emit` coverage fact when the module is also accepted
    /// by the downstream tools. Like a single-gate function, a cone function is
    /// universally synthesizable, so such a module runs the full Verilator +
    /// Yosys (+ Icarus) plan.
    #[serde(default)]
    emitted_cone_function: bool,
}

// `DiffSimReport` (the per-module diff-sim outcome) now lives in
// `anvil::diff_sim` (imported above) — moved in `BUG-HUNT-ORCHESTRATION.2a`
// alongside the run+compare pipeline so the bug-hunt loop reuses it. The serde
// shape is unchanged, so `tool_matrix_report.json` stays byte-identical.

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
    #[serde(default)]
    skip_verilator: bool,
    #[serde(default)]
    skip_yosys: bool,
    #[serde(default)]
    iverilog_compile: bool,
    #[serde(default)]
    sv2v: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    iverilog_compile: Option<ToolInvocation>,
    /// `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — opt-in `sv2v` transpile
    /// column (the design-level counterpart of `ModuleReport.sv2v`).
    /// `None` unless `--sv2v` was set; off the wire when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sv2v: Option<ToolInvocation>,
    /// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — opt-in acceptance-divergence
    /// column (decision `0019`); the design-level counterpart of
    /// `ModuleReport.divergence`. `None` unless `--divergence` was set AND this
    /// scenario was in the per-axis subset; otherwise `Some(DivergenceReport)`
    /// projecting the verdict of each tool the matrix already ran on the design.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    divergence: Option<DivergenceReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesignFileHash {
    file: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesignCheckpoint {
    #[serde(default)]
    skip_verilator: bool,
    #[serde(default)]
    skip_yosys: bool,
    #[serde(default)]
    iverilog_compile: bool,
    #[serde(default)]
    sv2v: bool,
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
    iverilog_compile_passed: usize,
    iverilog_compile_failed: usize,
    sv2v_passed: usize,
    sv2v_failed: usize,
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
    total_fsms_merged: u64,
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
    saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs: bool,
    saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs: bool,
    saw_recursive_hierarchy_three_stage_registered_parent_composed_chain: bool,
    saw_recursive_parent_cone_helper_budget_5: bool,
    saw_recursive_hierarchy_canonical_module_signature_diversity: bool,
    saw_design_with_structurally_duplicate_modules: bool,
    saw_recursive_hierarchy_module_dedup_active: bool,
    saw_width_parameterized_design: bool,
    saw_packed_aggregate_design: bool,
    saw_inferrable_memory_design: bool,
    saw_fsm_design: bool,
    /// `SIGNOFF-AUTOMATION-EXPANSION.2b` — at least one module carried
    /// an `Add`/`Mul` operator gate with a duplicated operand slot
    /// (`num_operator_gates_with_duplicate_operands > 0`). Proves the
    /// `operand_duplication_rate` knob fired by construction.
    saw_operand_duplication: bool,
    /// `SIGNOFF-AUTOMATION-EXPANSION.2b` — at least one module carried
    /// a degenerate 2-to-1 mux whose two data arms are the same
    /// `NodeId` (`num_muxes_degenerate > 0`). Proves the
    /// `mux_arm_duplication_rate` knob fired by construction.
    saw_mux_arm_duplication: bool,
    /// `SIGNOFF-AUTOMATION-EXPANSION.2b` — at least one design carried a
    /// packed-array aggregate module
    /// (`num_array_packed_aggregate_modules` positive). Proves the
    /// `aggregate_array_prob` knob selected the `ArrayPacked` projection
    /// (the deferred `AGGREGATE-ARRAY-PACKING.4b` matrix instrumentation).
    saw_array_packed_aggregate_design: bool,
    /// `SIGNOFF-AUTOMATION-EXPANSION.2b` — at least one design carried a
    /// memory module **and** an FSM module in the same design
    /// (`num_memory_modules > 0 && num_fsm_modules > 0`). Proves the
    /// memory×fsm interplay that the single-knob `phase6_*` axes cannot:
    /// per-leaf memory-vs-FSM selection is mutually exclusive, so this
    /// needs `memory_prob ∈ (0,1)` + `fsm_prob = 1.0`.
    saw_memory_fsm_interplay_design: bool,
    /// `SV-VERSION-TARGETING.2b.2b` — at least one version-targeted
    /// artifact was accepted by the downstream tools in the matching
    /// standard mode (Verilator `--language 1800-20xx` + Yosys `-sv`).
    /// The umbrella fact: true iff any of the three per-version
    /// sub-facts below is true. Only lit under the `--sv-version-gate`
    /// run (the only run that sets the Verilator `--language` selector).
    saw_sv_version_targeted_acceptance: bool,
    /// `SV-VERSION-TARGETING.2b.2b` — an IEEE 1800-2012-targeted artifact
    /// was accepted by Verilator `--language 1800-2012` + Yosys `-sv`.
    saw_sv_version_2012_targeted_acceptance: bool,
    /// `SV-VERSION-TARGETING.2b.2b` — an IEEE 1800-2017-targeted artifact
    /// was accepted by Verilator `--language 1800-2017` + Yosys `-sv`.
    saw_sv_version_2017_targeted_acceptance: bool,
    /// `SV-VERSION-TARGETING.2b.2b` — an IEEE 1800-2023-targeted artifact
    /// was accepted by Verilator `--language 1800-2023` + Yosys `-sv`.
    saw_sv_version_2023_targeted_acceptance: bool,
    /// `SV-VERSION-TARGETING.3b.2b` — at least one IEEE 1800-2023-targeted
    /// module emitted the `union soft` low-bits-slice up-opt overlay
    /// (`soft_union_slice_prob`) and it was accepted by Verilator
    /// `--language 1800-2023`. Distinct from
    /// `saw_sv_version_2023_targeted_acceptance` (which requires Yosys-clean):
    /// Yosys/Icarus reject the `union soft` syntax and are a recorded no-op
    /// (decision `0010`), so this fact requires only Verilator matching-mode
    /// acceptance of a *genuinely emitted* overlay.
    saw_sv_version_2023_soft_union_upopt: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.2b.2b` — at least one module emitted
    /// a combinational `function automatic` emit-projection
    /// (`function_emit_prob`; decision `0012`) **and** that module was
    /// accepted by the downstream tools (Verilator success + Yosys clean;
    /// Icarus when enabled is enforced via the tool-summary bail). Proves
    /// the first richer-structured surface fires by construction and is
    /// downstream-clean, not just that the knob was requested.
    saw_combinational_function_emit: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.4b.2b` — at least one module emitted a
    /// `generate for` loop emit-projection (`generate_loop_emit_prob`;
    /// decision `0013`) **and** that module was accepted by the downstream
    /// tools (Verilator success + Yosys clean; Icarus when enabled is enforced
    /// via the tool-summary bail). Proves the second richer-structured surface
    /// fires by construction and is downstream-clean, not just that the knob
    /// was requested.
    saw_generate_loop_emit: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.6b.2b` — at least one module emitted a
    /// combinational `task automatic` emit-projection (`task_emit_prob`;
    /// decision `0014`) **and** that module was accepted by the downstream
    /// tools (Verilator success + Yosys clean; Icarus when enabled is enforced
    /// via the tool-summary bail). Proves the third richer-structured surface
    /// fires by construction and is downstream-clean, not just that the knob
    /// was requested.
    saw_combinational_task_emit: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.10b.2` — at least one module emitted a
    /// multi-gate-cone `function automatic` emit-projection
    /// (`cone_function_emit_prob`; decision `0016`) **and** that module was
    /// accepted by the downstream tools (Verilator success + Yosys clean;
    /// Icarus when enabled is enforced via the tool-summary bail). Proves the
    /// fifth richer-structured surface fires by construction and is
    /// downstream-clean, not just that the knob was requested.
    saw_cone_function_emit: bool,
    /// `DIFFERENTIAL-SIMULATION.3b.2` — at least one DUT in the
    /// `--diff-sim` per-axis subset achieved byte-equal post-reset
    /// traces across iverilog 13.0 and verilator 5.046. The
    /// first gate to assert downstream-tool *semantic* agreement
    /// on ANVIL output, complementing the existing
    /// parse/synth/lint columns.
    saw_design_with_cross_simulator_agreement: bool,
    /// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — at least one unit in the
    /// `--divergence` subset had two enabled tools disagree on its
    /// acceptance (one accepted while another warned or rejected). This is
    /// an **opportunistic** fact: on valid-by-construction RTL the steady
    /// state is that all tools agree, so a divergence is a genuine
    /// downstream-tool bug — the thing the lane exists to *surface*. It is
    /// therefore **never** a required coverage gate (`compute_coverage_gaps`
    /// never demands it); a gate requiring it would fail on clean output,
    /// which is the normal case (decision `0019`).
    saw_acceptance_divergence: bool,
    /// `MULTI-CLOCK-CDC.3b.2` — at least one DUT carried more
    /// than one declared clock domain
    /// (`Module.clock_domains.len() >= 2`). Lit when the
    /// `multi_clock_prob` scenario fires and the
    /// `promote_to_multi_clock` pass successfully adds a second
    /// domain.
    saw_multi_clock_design: bool,
    /// `MULTI-CLOCK-CDC.3b.2` — at least one DUT carried a
    /// exact 2-flop synchronizer chain. Distinct from
    /// `saw_multi_clock_design`: a module could declare K=2
    /// domains without any synchronizer if the promotion-pass
    /// decline path fired. Both facts together prove the
    /// by-construction synchronizer rule actually executed.
    saw_cdc_2_flop_synchronizer: bool,
    /// `SIGNOFF-SURFACE-EXPANSION.1` — at least one DUT carried a
    /// CDC synchronizer chain with three or more destination-domain
    /// stages. This proves the N-flop synchronizer primitive beyond
    /// the default 2-flop chain.
    saw_cdc_nflop_synchronizer: bool,
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
    /// `SIGNOFF-AUTOMATION-EXPANSION.2b` — the focused richer-knob-sweep
    /// gate. Promotes the four genuinely-unswept generator knobs
    /// (`operand_duplication_rate`, `mux_arm_duplication_rate`,
    /// `aggregate_array_prob`, and the memory×fsm interplay) into
    /// explicit first-class scenario axes so they fire by construction,
    /// not by chance (ROADMAP steering gap 3). Each is a depth-1 wrapper
    /// design across all three construction strategies.
    SignoffKnobSweep,
    /// `SV-VERSION-TARGETING.2b.2b` — the repo-owned per-version
    /// acceptance gate. Sweeps the three IEEE 1800 emission targets
    /// (2012/2017/2023) over a focused comb-leaf / seq-leaf / recursive
    /// hierarchy-design corpus and runs Verilator in the matching
    /// `--language 1800-20xx` standard mode (via the `.2b.2a` selector),
    /// proving each version-targeted corpus is accepted in the matching
    /// tool standard mode. Default `Sv2012` emission is byte-identical to
    /// today; the gate's value is the per-version downstream acceptance
    /// axis, not output divergence (divergence arrives with the future
    /// up-opting leaf `.3`).
    SvVersionSweep,
    /// `STRUCTURED-EMISSION-EXPANSION.2b.2b` — the repo-owned combinational
    /// `function automatic` emit gate. Forces `function_emit_prob = 1.0`
    /// over a comb-only single-module DUT across all three construction
    /// strategies, so every qualifying combinational gate is rendered as a
    /// behaviour-preserving `function automatic` over its direct operands
    /// (decision `0012`). Proves the first richer-structured emission
    /// surface is accepted warning-clean by Verilator + both Yosys modes
    /// (+ Icarus when `--iverilog-compile` is set), gated on the
    /// `saw_combinational_function_emit` coverage fact. Default
    /// `function_emit_prob = 0.0` emission stays byte-identical; the gate
    /// is the opt-in proof axis for the non-default surface.
    FunctionEmitSweep,
    /// `STRUCTURED-EMISSION-EXPANSION.4b.2b` — the repo-owned `generate for`
    /// loop emit gate. Forces `generate_loop_emit_prob = 1.0` over a comb-only
    /// single-module DUT across all three construction strategies, so every
    /// qualifying `{N{x}}` 1-bit-lane replication is rendered as a
    /// behaviour-preserving single-level `generate for` loop (decision
    /// `0013`). Proves the second richer-structured emission surface is
    /// accepted warning-clean by Verilator + both Yosys modes (+ Icarus when
    /// `--iverilog-compile` is set), gated on the `saw_generate_loop_emit`
    /// coverage fact. Default `generate_loop_emit_prob = 0.0` emission stays
    /// byte-identical; the gate is the opt-in proof axis for the non-default
    /// surface.
    GenerateLoopSweep,
    /// `STRUCTURED-EMISSION-EXPANSION.6b.2b` — the repo-owned combinational
    /// `task automatic` emit gate. Forces `task_emit_prob = 1.0` over a
    /// comb-only single-module DUT across all three construction strategies,
    /// so every qualifying combinational gate is rendered as a
    /// behaviour-preserving `task automatic` over its direct operands, called
    /// from `always_comb` into a `<wire>__tv` output var (decision `0014`).
    /// Proves the third richer-structured emission surface is accepted
    /// warning-clean by Verilator + both Yosys modes (+ Icarus when
    /// `--iverilog-compile` is set), gated on the `saw_combinational_task_emit`
    /// coverage fact. Default `task_emit_prob = 0.0` emission stays
    /// byte-identical; the gate is the opt-in proof axis for the non-default
    /// surface.
    TaskEmitSweep,
    /// `STRUCTURED-EMISSION-EXPANSION.10b.2` — the repo-owned multi-gate-cone
    /// `function automatic` emit gate. Forces `cone_function_emit_prob = 1.0`
    /// over a comb-only single-module DUT across all three construction
    /// strategies, so every qualifying combinational cone (a root gate plus its
    /// single-use interior gates) is rendered as one behaviour-preserving
    /// `function automatic` over the cone's boundary leaves (decision `0016`).
    /// Proves the fifth richer-structured emission surface is accepted
    /// warning-clean by Verilator + both Yosys modes (+ Icarus when
    /// `--iverilog-compile` is set), gated on the `saw_cone_function_emit`
    /// coverage fact. Default `cone_function_emit_prob = 0.0` emission stays
    /// byte-identical; the gate is the opt-in proof axis for the non-default
    /// surface.
    ConeFunctionSweep,
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
    #[serde(default)]
    signoff_knob_sweep_gate: bool,
    /// `SV-VERSION-TARGETING.2b.2b` — whether `--sv-version-gate` drove
    /// this run. When `true`, every scenario carries a non-default
    /// `sv_version` only via the sweep, Verilator ran in the matching
    /// `--language` mode, and the `saw_sv_version_*_targeted_acceptance`
    /// facts are enforced under `coverage_gaps`.
    #[serde(default)]
    sv_version_gate: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.2b.2b` — whether `--function-emit-gate`
    /// drove this run. When `true`, every scenario forced
    /// `function_emit_prob = 1.0` over comb-only DUTs and the
    /// `saw_combinational_function_emit` fact is enforced under
    /// `coverage_gaps`.
    #[serde(default)]
    function_emit_gate: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.4b.2b` — whether `--generate-loop-gate`
    /// drove this run. When `true`, every scenario forced
    /// `generate_loop_emit_prob = 1.0` over comb-only DUTs and the
    /// `saw_generate_loop_emit` fact is enforced under `coverage_gaps`.
    #[serde(default)]
    generate_loop_gate: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.6b.2b` — whether `--task-emit-gate`
    /// drove this run. When `true`, every scenario forced
    /// `task_emit_prob = 1.0` over comb-only DUTs and the
    /// `saw_combinational_task_emit` fact is enforced under `coverage_gaps`.
    #[serde(default)]
    task_emit_gate: bool,
    /// `STRUCTURED-EMISSION-EXPANSION.10b.2` — whether `--cone-function-gate`
    /// drove this run. When `true`, every scenario forced
    /// `cone_function_emit_prob = 1.0` over comb-only DUTs and the
    /// `saw_cone_function_emit` fact is enforced under `coverage_gaps`.
    #[serde(default)]
    cone_function_gate: bool,
    yosys_mode: String,
    coverage: CoverageSummary,
    coverage_gaps: Vec<String>,
    share_sweep: Option<ShareSweepSummary>,
    tool_summary: ToolSummary,
    scenarios: Vec<ScenarioReport>,
    /// `SIGNOFF-SURFACE-EXPANSION.3` — whether
    /// `--iverilog-compile` was set for this run. When `false`, no
    /// module/design report carries an `iverilog_compile` invocation.
    #[serde(default)]
    iverilog_compile_enabled: bool,
    /// `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — whether `--sv2v` was set
    /// for this run. When `false`, no module/design report carries an
    /// `sv2v` invocation.
    #[serde(default)]
    sv2v_enabled: bool,
    /// `DIFFERENTIAL-SIMULATION.3b.2` — whether `--diff-sim` was
    /// set for this run. When `false`, `diff_sim_subset` is empty
    /// and no `ModuleReport.diff_sim` is populated.
    #[serde(default)]
    diff_sim_enabled: bool,
    /// `DIFFERENTIAL-SIMULATION.3b.2` — the per-axis subset of
    /// scenario names selected by `select_diff_sim_subset`. The
    /// report is self-describing: a reader can see which scenarios
    /// were actually gated by the diff-sim column.
    #[serde(default)]
    diff_sim_subset: Vec<String>,
    /// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — whether `--divergence` was
    /// set for this run. When `false`, `divergence_subset` is empty and no
    /// `ModuleReport`/`DesignReport.divergence` is populated.
    #[serde(default)]
    divergence_enabled: bool,
    /// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — the per-axis subset of
    /// scenario names selected by `select_diff_sim_subset` (shared with the
    /// diff-sim column) for the acceptance-divergence column. Self-describing:
    /// a reader can see which scenarios carried a `divergence` report.
    #[serde(default)]
    divergence_subset: Vec<String>,
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

    // `DIFFERENTIAL-SIMULATION.3b.2` — compute the per-axis
    // subset once and persist it as a sentinel file
    // (`<out>/.diff-sim-subset`). `materialize_prepared_module`
    // reads this sentinel to decide whether to run the diff-sim
    // column for the scenario it is processing. The sentinel
    // pattern keeps the existing per-scenario API stable.
    let diff_sim_subset: Vec<String> = if cli.diff_sim {
        select_diff_sim_subset(&scenarios)
    } else {
        Vec::new()
    };
    if cli.diff_sim {
        std::fs::write(out_dir.join(".diff-sim-subset"), diff_sim_subset.join("\n"))
            .with_context(|| format!("write diff-sim subset sentinel in {}", out_dir.display()))?;
    }

    // `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — the acceptance-divergence column
    // reuses the *same* per-axis subset selector (`classify_diff_sim_axis`) as
    // diff-sim, but is gated by its own `--divergence` flag and its own
    // `.divergence-subset` sentinel (the two columns are independent). The
    // sentinel keeps the per-scenario materialization API stable, exactly as
    // the diff-sim sentinel does.
    let divergence_subset: Vec<String> = if cli.divergence {
        select_diff_sim_subset(&scenarios)
    } else {
        Vec::new()
    };
    if cli.divergence {
        std::fs::write(
            out_dir.join(".divergence-subset"),
            divergence_subset.join("\n"),
        )
        .with_context(|| format!("write divergence subset sentinel in {}", out_dir.display()))?;
    }

    // `SV-VERSION-TARGETING.2b.2b` — only the per-version gate runs the
    // downstream tools in the matching `--language 1800-20xx` standard
    // mode and lights the per-version acceptance facts. Every other run
    // leaves `version_targeted` false, so Verilator keeps today's
    // byte-identical argv (`language: None`).
    let version_targeted = scenario_set == ScenarioSet::SvVersionSweep;

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
            version_targeted,
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
        signoff_knob_sweep_gate: cli.signoff_knob_sweep_gate,
        sv_version_gate: cli.sv_version_gate,
        function_emit_gate: cli.function_emit_gate,
        generate_loop_gate: cli.generate_loop_gate,
        task_emit_gate: cli.task_emit_gate,
        cone_function_gate: cli.cone_function_gate,
        yosys_mode: yosys_mode_slug(cli.yosys_mode).to_string(),
        coverage: global_coverage,
        coverage_gaps,
        share_sweep,
        tool_summary: global_tool_summary,
        scenarios: scenario_reports,
        iverilog_compile_enabled: cli.iverilog_compile,
        sv2v_enabled: cli.sv2v,
        diff_sim_enabled: cli.diff_sim,
        diff_sim_subset,
        divergence_enabled: cli.divergence,
        divergence_subset,
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
        "tool_matrix: Verilator pass/fail = {}/{}, Yosys without-abc pass/fail = {}/{}, Yosys with-abc pass/fail = {}/{}, Icarus compile pass/fail = {}/{}, sv2v pass/fail = {}/{}",
        report.tool_summary.verilator_passed,
        report.tool_summary.verilator_failed,
        report.tool_summary.yosys_without_abc_passed,
        report.tool_summary.yosys_without_abc_failed,
        report.tool_summary.yosys_with_abc_passed,
        report.tool_summary.yosys_with_abc_failed,
        report.tool_summary.iverilog_compile_passed,
        report.tool_summary.iverilog_compile_failed,
        report.tool_summary.sv2v_passed,
        report.tool_summary.sv2v_failed
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

    if report.tool_summary.any_failed() {
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
    } else if cli.signoff_knob_sweep_gate {
        SIGNOFF_KNOB_SWEEP_MIN_UNITS_PER_SCENARIO
    } else if cli.sv_version_gate {
        SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO
    } else if cli.function_emit_gate {
        FUNCTION_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
    } else if cli.generate_loop_gate {
        GENERATE_LOOP_SWEEP_MIN_UNITS_PER_SCENARIO
    } else if cli.task_emit_gate {
        TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
    } else if cli.cone_function_gate {
        CONE_FUNCTION_SWEEP_MIN_UNITS_PER_SCENARIO
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
            || cli.phase4_hierarchy_gate
            || cli.signoff_knob_sweep_gate
            || cli.sv_version_gate
            || cli.function_emit_gate
            || cli.generate_loop_gate
            || cli.task_emit_gate
            || cli.cone_function_gate,
        total_modules,
    }
}

fn select_scenario_set(cli: &Cli) -> Result<ScenarioSet> {
    let enabled_gates = usize::from(cli.phase1_gate)
        + usize::from(cli.phase2_share_gate)
        + usize::from(cli.phase3_structured_gate)
        + usize::from(cli.phase4_hierarchy_gate)
        + usize::from(cli.signoff_knob_sweep_gate)
        + usize::from(cli.sv_version_gate)
        + usize::from(cli.function_emit_gate)
        + usize::from(cli.generate_loop_gate)
        + usize::from(cli.task_emit_gate)
        + usize::from(cli.cone_function_gate);
    if enabled_gates > 1 {
        bail!(
            "--phase1-gate, --phase2-share-gate, --phase3-structured-gate, --phase4-hierarchy-gate, --signoff-knob-sweep-gate, --sv-version-gate, --function-emit-gate, --generate-loop-gate, --task-emit-gate, and --cone-function-gate are mutually exclusive"
        );
    }
    if cli.phase2_share_gate {
        Ok(ScenarioSet::Phase2Share)
    } else if cli.phase3_structured_gate {
        Ok(ScenarioSet::Phase3Structured)
    } else if cli.phase4_hierarchy_gate {
        Ok(ScenarioSet::Phase4Hierarchy)
    } else if cli.signoff_knob_sweep_gate {
        Ok(ScenarioSet::SignoffKnobSweep)
    } else if cli.sv_version_gate {
        Ok(ScenarioSet::SvVersionSweep)
    } else if cli.function_emit_gate {
        Ok(ScenarioSet::FunctionEmitSweep)
    } else if cli.generate_loop_gate {
        Ok(ScenarioSet::GenerateLoopSweep)
    } else if cli.task_emit_gate {
        Ok(ScenarioSet::TaskEmitSweep)
    } else if cli.cone_function_gate {
        Ok(ScenarioSet::ConeFunctionSweep)
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
        ScenarioSet::SignoffKnobSweep => build_signoff_knob_sweep_scenarios(base_seed)?,
        ScenarioSet::SvVersionSweep => build_sv_version_sweep_scenarios(base_seed)?,
        ScenarioSet::FunctionEmitSweep => build_function_emit_sweep_scenarios(base_seed)?,
        ScenarioSet::GenerateLoopSweep => build_generate_loop_sweep_scenarios(base_seed)?,
        ScenarioSet::TaskEmitSweep => build_task_emit_sweep_scenarios(base_seed)?,
        ScenarioSet::ConeFunctionSweep => build_cone_function_sweep_scenarios(base_seed)?,
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

    // `MULTI-CLOCK-CDC.3b.2` / `SIGNOFF-SURFACE-EXPANSION.1` —
    // multi-clock scenarios. Force `multi_clock_prob = 1.0` + a
    // sequential + narrow-output profile so the promotion pass has an
    // eligible target. The first scenario preserves the exact 2-stage
    // path; the second proves the N-stage path.
    scenarios.push(make_scenario(
        "int_multi_clock_2flop_sync",
        "Interleaved with multi_clock_prob=1.0 + flop_prob=1.0 + min/max-width=1 — exercises the MULTI-CLOCK-CDC.3b promote_to_multi_clock pass; lights saw_multi_clock_design + saw_cdc_2_flop_synchronizer.",
        multi_clock_focus_config(ConstructionStrategy::Interleaved, next_seed),
    )?);
    next_seed += 1;
    scenarios.push(make_scenario(
        "int_multi_clock_3flop_sync",
        "Interleaved with multi_clock_prob=1.0 + cdc_synchronizer_stages=3 + flop_prob=1.0 + min/max-width=1 — exercises the SIGNOFF-SURFACE-EXPANSION.1 N-flop synchronizer path; lights saw_cdc_nflop_synchronizer.",
        multi_clock_nflop_focus_config(ConstructionStrategy::Interleaved, next_seed, 3),
    )?);

    Ok(scenarios)
}

/// `MULTI-CLOCK-CDC.3b.2` — config for the multi-clock scenario
/// in the default sweep. Sequential-favoring (flop_prob=1.0) +
/// narrow outputs (min/max width = 1) so the promotion pass's
/// 1-bit-flop-driven-output eligibility predicate matches; the
/// new ports the pass allocates are bog-standard inputs and
/// don't disturb Verilator/Yosys cleanliness on the existing
/// columns.
fn multi_clock_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    let mut cfg = relaxed_default_config(strategy, seed);
    cfg.multi_clock_prob = 1.0;
    cfg.flop_prob = 1.0;
    cfg.min_width = 1;
    cfg.max_width = 1;
    cfg
}

fn multi_clock_nflop_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
    stages: u32,
) -> Config {
    let mut cfg = multi_clock_focus_config(strategy, seed);
    cfg.cdc_synchronizer_stages = stages;
    cfg
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
                "phase4_recur_d7_stateful_parent_port_composed_output",
                "bounded recursive hierarchy at exact depth 7 where non-top parent outputs at six intermediate layers compose parent data ports, sibling instance outputs, and parent-local Qs without helper instances",
                phase4_recursive_d7_stateful_parent_port_composed_output_focus_config(
                    strategy,
                    next_seed + 63,
                ),
            ),
            (
                "phase4_recur_d7_stateful_parent_composed_mixed_support_child_input",
                "bounded recursive hierarchy at exact depth 7 where non-top unregistered parent-composed child-input cones at six intermediate layers mix parent data ports, sibling instance outputs, and parent-local Qs without helper instances (2,2 calibrated)",
                phase4_recursive_d7_stateful_parent_composed_mixed_support_focus_config(
                    strategy,
                    next_seed + 64,
                ),
            ),
            (
                "phase4_recur_d3_registered_three_stage_parent_composed_chain",
                "bounded recursive hierarchy at exact depth 3 where non-top registered parent-composed child-input bindings chain through at least three parent-local flop stages without helper instances",
                phase4_recursive_registered_three_stage_parent_composed_chain_focus_config(
                    strategy,
                    next_seed + 65,
                ),
            ),
            (
                "phase4_recur_d2_parent_cone_instance_budget5",
                "bounded recursive hierarchy at exact depth 2 where non-top parent-composed child-input cones can spend a five-helper parent-cone budget below the top parent",
                phase4_recursive_parent_cone_instance_budget_5_focus_config(
                    strategy,
                    next_seed + 66,
                ),
            ),
            (
                "phase4_recur_d2_canonical_module_signatures",
                "bounded recursive hierarchy at exact depth 2 used to anchor canonical-module-signature instrumentation in the matrix (first slice of hierarchy-aware identity)",
                phase4_recursive_canonical_module_signature_focus_config(
                    strategy,
                    next_seed + 67,
                ),
            ),
            (
                "phase4_hier1_structurally_duplicate_modules",
                "depth-1 wrapper-lane scenario with 4 tightly-constrained 1-in/1-out/width-1 leaf modules that collapse to a single canonical signature — proves the planner can emit structurally-duplicate Module definitions (HIERARCHY-AWARE-IDENTITY.2)",
                phase4_hierarchy_structurally_duplicate_modules_focus_config(
                    strategy,
                    next_seed + 68,
                ),
            ),
            (
                "phase4_hier1_module_dedup_active",
                "depth-1 wrapper-lane scenario identical to phase4_hier1_structurally_duplicate_modules but with hierarchy_module_dedup = true; proves the post-finalisation dedup pass collapses duplicates downstream-clean (HIERARCHY-AWARE-IDENTITY.4)",
                phase4_hierarchy_module_dedup_active_focus_config(strategy, next_seed + 69),
            ),
            (
                "phase5_width_parameterized",
                "depth-1 wrapper, library mode, width_parameterization_prob = 1.0: the library leaves are built by the rules-first parameterizable constructor and instantiated with per-instance #(.W(v)) overrides. Proves Phase 5 parameterized designs are downstream-clean (PHASE-5-PARAMETERIZATION.2.4)",
                phase5_width_parameterization_focus_config(strategy, next_seed + 70),
            ),
            (
                "phase5b_packed_aggregate",
                "depth-1 wrapper, library mode, aggregate_prob = 1.0: the never-instantiated top wrapper is given a packed-struct emitter projection (data ports folded into one aggregate port + boundary alias wires); leaves stay flat (scaffold scope). Proves Phase 5b packed-aggregate designs are downstream-clean (PHASE-5B-AGGREGATES.2.3)",
                phase5b_packed_aggregate_focus_config(strategy, next_seed + 71),
            ),
            (
                "phase6_inferrable_memory",
                "depth-1 wrapper, library mode, memory_prob = 1.0: the rules-first library leaves are inferrable-memory blocks (synchronous write + registered read) instantiated by the wrapper. Proves Phase 6 inferrable-memory designs are downstream-clean (PHASE-6-ADVANCED-MOTIFS.2.3)",
                phase6_inferrable_memory_focus_config(strategy, next_seed + 72),
            ),
            (
                "phase6_fsm",
                "depth-1 wrapper, library mode, fsm_prob = 1.0: the rules-first library leaves are generated-encoding Moore FSM blocks (encoding-derived state constants + async-reset state register + next-state/Moore case decode) instantiated by the wrapper. Proves Phase 6 generated-encoding FSM designs are downstream-clean (PHASE-6-ADVANCED-MOTIFS.3.4)",
                phase6_fsm_focus_config(strategy, next_seed + 73),
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

fn phase4_recursive_d7_stateful_parent_port_composed_output_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    let mut cfg = phase4_recursive_d7_parent_port_composed_output_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg.min_width = 1;
    cfg.max_width = 8;
    cfg.max_depth = 1;
    cfg
}

fn phase4_recursive_d7_stateful_parent_composed_mixed_support_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Calibration: built atop r79's 2,2 child-instance helper for the same
    // safe-slice reason as r77 at depth 6. depths 3-5 used 4,4 for stateful
    // mixed-support cells; at 4,4/depth-7 the gate would explode beyond a
    // safe slice. Closes the depth-7 sweep.
    let mut cfg = phase4_recursive_d7_parent_composed_mixed_support_focus_config(strategy, seed);
    cfg.hierarchy_parent_flop_prob = 1.0;
    cfg.max_flops_per_module = 64;
    cfg
}

fn phase4_hierarchy_module_dedup_active_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // HIERARCHY-AWARE-IDENTITY.4 anchor scenario. Identical to
    // phase4_hierarchy_structurally_duplicate_modules_focus_config but
    // with hierarchy_module_dedup enabled. The dedup pass collapses
    // the 4 library leaves to 1 surviving leaf, leaving the top + 1
    // leaf = 2 modules total, all structurally distinct
    // (num_distinct == num_modules). Both scenarios stay in the bank
    // so the before/after comparison is visible.
    let mut cfg = phase4_hierarchy_structurally_duplicate_modules_focus_config(strategy, seed);
    cfg.hierarchy_module_dedup = true;
    cfg
}

fn phase5_width_parameterization_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // PHASE-5-PARAMETERIZATION.2.4 anchor scenario. Legacy depth-1
    // wrapper, library child-sourcing (default), shaped exactly like
    // the dedup anchor (4 leaves / 4 instances) so the matrix's
    // leaf/child shape-coverage sets are unperturbed. With
    // `width_parameterization_prob = 1.0` each library leaf is built
    // by the rules-first `build_parameterizable_leaf` constructor (a
    // width-homogeneous combinational leaf), and the parent
    // instantiates them with per-instance in-range `#(.W(v))`
    // overrides (`min_width`/`max_width` span a real range). All
    // hierarchy-routing probabilities are 0.0; the leaves are purely
    // combinational. Suffix `phase5_width_parameterized` is in the
    // matrix unit-test exception list at the bottom of this file.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        width_parameterization_prob: 1.0,
        min_width: 2,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn phase5b_packed_aggregate_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // PHASE-5B-AGGREGATES.2.3 anchor scenario. Shaped EXACTLY like the
    // phase5 / dedup anchor (depth-1 wrapper, library child-sourcing,
    // 4 leaves / 4 instances, all hierarchy-routing probabilities 0.0,
    // purely combinational) so the matrix's leaf/child/range/source
    // shape-coverage sets are unperturbed — only the scenario and
    // module counts grow. The single difference from
    // `phase5_width_parameterization_focus_config` is
    // `aggregate_prob = 1.0` instead of width parameterization: the
    // never-instantiated top wrapper is given a packed-struct emitter
    // projection (the library leaves are instantiated, so the `.2.1`
    // scaffold scope correctly leaves them flat). Suffix
    // `phase5b_packed_aggregate` is in the matrix unit-test exception
    // list at the bottom of this file.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        aggregate_prob: 1.0,
        min_width: 2,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn phase6_inferrable_memory_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // PHASE-6-ADVANCED-MOTIFS.2.3 anchor scenario. Shaped EXACTLY like the
    // phase5 / dedup anchor (depth-1 wrapper, library child-sourcing,
    // 4 leaves / 4 instances, all hierarchy-routing probabilities 0.0,
    // purely combinational) so the matrix's leaf/child/range/source
    // shape-coverage sets are unperturbed — only the scenario and
    // module counts grow. The single difference from
    // `phase5_width_parameterization_focus_config` is
    // `memory_prob = 1.0` instead of width parameterization: the
    // rules-first library leaves are inferrable-memory blocks
    // (synchronous write + registered read) instantiated by the
    // wrapper. Suffix `phase6_inferrable_memory` is in the matrix
    // unit-test exception list at the bottom of this file.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        memory_prob: 1.0,
        min_width: 2,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn phase6_fsm_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // PHASE-6-ADVANCED-MOTIFS.3.4a anchor scenario. Shaped EXACTLY
    // like the phase6_inferrable_memory / phase5 / dedup anchor
    // (depth-1 wrapper, library child-sourcing, 4 leaves / 4
    // instances, all hierarchy-routing probabilities 0.0, purely
    // combinational) so the matrix's leaf/child/range/source
    // shape-coverage sets are unperturbed — only the scenario and
    // module counts grow. The single difference from
    // `phase6_inferrable_memory_focus_config` is `fsm_prob = 1.0`
    // instead of `memory_prob`: the rules-first library leaves are
    // generated-encoding Moore FSM blocks instantiated by the
    // wrapper. Suffix `phase6_fsm` is in the matrix unit-test
    // exception list at the bottom of this file.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        fsm_prob: 1.0,
        min_width: 2,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn signoff_operand_duplication_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // SIGNOFF-AUTOMATION-EXPANSION.2b anchor scenario for the
    // `operand_duplication_rate` knob (ROADMAP steering gap 3 — make a
    // previously-implicit knob fire by construction). Single-module DUT
    // (no hierarchy): arithmetic-only gate weights so every operator
    // gate is `Add`/`Mul`, a tiny terminal pool (1-2 inputs, no
    // constants, reuse-only) and a 3-4 operand arity so the operand
    // picker frequently re-draws the same `NodeId`, and
    // `operand_duplication_rate = 1.0` so those duplicates are kept
    // instead of re-rolled. Lights `saw_operand_duplication` via the
    // post-hoc `num_operator_gates_with_duplicate_operands` metric;
    // `flop_prob` stays at its default so the cone is large enough to
    // hit the duplication path. Downstream-clean (`a + a` / `a * a`).
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        operand_duplication_rate: 1.0,
        gate_arith_weight: 8,
        gate_bitwise_weight: 0,
        gate_struct_weight: 0,
        gate_compare_weight: 0,
        gate_reduce_weight: 0,
        gate_shift_weight: 0,
        min_inputs: 1,
        max_inputs: 2,
        constant_prob: 0.0,
        terminal_reuse_prob: 1.0,
        min_width: 2,
        max_width: 4,
        min_gate_arity: 3,
        max_gate_arity: 4,
        coefficient_prob: 0.0,
        ..Config::default()
    }
}

fn signoff_mux_arm_duplication_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // SIGNOFF-AUTOMATION-EXPANSION.2b anchor scenario for the
    // `mux_arm_duplication_rate` knob. Single-module DUT: comb-mux-only
    // structured gate weights, forced 2-arm encoded comb muxes
    // (`comb_mux_prob = comb_mux_encoding_prob = 1.0`,
    // `min_mux_arms = max_mux_arms = 2`), and a tiny 1-bit/2-bit
    // terminal pool so the chained-ternary arm and its running tail
    // collapse to the same `NodeId`, producing the degenerate
    // `(sel)?(x):(x)` form once `mux_arm_duplication_rate = 1.0` permits
    // it. Lights `saw_mux_arm_duplication` via `num_muxes_degenerate`;
    // `flop_prob` stays at its default so the cone is rich enough.
    // Downstream-clean (a redundant select Verilator/Yosys fold away).
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        mux_arm_duplication_rate: 1.0,
        comb_mux_prob: 1.0,
        comb_mux_encoding_prob: 1.0,
        min_mux_arms: 2,
        max_mux_arms: 2,
        gate_struct_weight: 8,
        gate_bitwise_weight: 0,
        gate_arith_weight: 0,
        gate_compare_weight: 0,
        gate_reduce_weight: 0,
        gate_shift_weight: 0,
        min_inputs: 1,
        max_inputs: 2,
        constant_prob: 0.0,
        terminal_reuse_prob: 1.0,
        min_width: 1,
        max_width: 2,
        ..Config::default()
    }
}

fn signoff_array_packed_aggregate_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // SIGNOFF-AUTOMATION-EXPANSION.2b anchor for `aggregate_array_prob`
    // — the deferred `AGGREGATE-ARRAY-PACKING.4b` matrix instrumentation.
    // Shaped like the `phase5b_packed_aggregate` anchor (depth-1
    // wrapper, library child-sourcing, 4 leaves / 4 instances, all
    // hierarchy-routing probabilities 0.0, combinational) but with
    // `aggregate_array_prob = 1.0` on top of `aggregate_prob = 1.0` and
    // a UNIFORM data-port width (`min_width == max_width`): `ArrayPacked`
    // is a faithful projection only over a uniform-width group
    // (`src/ir/aggregate.rs`), so a non-uniform group would fall back to
    // `StructPacked`. Lights `saw_array_packed_aggregate_design` via
    // `num_array_packed_aggregate_modules`.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        aggregate_prob: 1.0,
        aggregate_array_prob: 1.0,
        min_width: 8,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn signoff_memory_fsm_interplay_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    // SIGNOFF-AUTOMATION-EXPANSION.2b anchor for the memory×fsm
    // interplay. Shaped like the `phase6_*` anchors (depth-1 wrapper,
    // library child-sourcing, hierarchy-routing probabilities 0.0) but
    // proves a memory module AND an FSM module coexist in ONE design —
    // which the single-knob `phase6_inferrable_memory` / `phase6_fsm`
    // axes cannot. Per-leaf memory-vs-FSM selection in
    // `src/gen/module.rs` is mutually exclusive (memory is rolled first
    // and returns early), so `memory_prob = 1.0` would yield no FSM
    // leaf: this uses `memory_prob = 0.5` + `fsm_prob = 1.0` over 6
    // leaves so roughly half roll memory and the rest fall through to
    // the always-firing FSM roll. Lights
    // `saw_memory_fsm_interplay_design` via `num_memory_modules > 0 &&
    // num_fsm_modules > 0`.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 6,
        num_child_instances: 6,
        memory_prob: 0.5,
        fsm_prob: 1.0,
        min_width: 2,
        max_width: 8,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

/// `SIGNOFF-AUTOMATION-EXPANSION.2b` — the focused richer-knob-sweep
/// scenario set. Promotes the four genuinely-unswept generator knobs
/// into explicit first-class axes, one focused scenario per knob across
/// all three construction strategies (so the universal
/// construction-strategy coverage check is satisfied). Two are
/// single-module DUTs (`operand_duplication_rate`,
/// `mux_arm_duplication_rate`) and two are depth-1 wrapper designs
/// (`aggregate_array_prob`, memory×fsm interplay); the matrix routes
/// each per its config.
fn build_signoff_knob_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;

    for strategy in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ] {
        let strategy_slug = strategy_slug(strategy);
        let strategy_label = construction_strategy_name(strategy);
        for (name_suffix, description_suffix, config) in [
            (
                "signoff_operand_duplication",
                "operand_duplication_rate = 1.0 over an arithmetic-only tiny-pool DUT — promotes the previously-unswept Add/Mul operand-duplication knob to an explicit axis (lights saw_operand_duplication)",
                signoff_operand_duplication_focus_config(strategy, next_seed),
            ),
            (
                "signoff_mux_arm_duplication",
                "mux_arm_duplication_rate = 1.0 over a 2-arm comb-mux tiny-pool DUT — promotes the previously-unswept degenerate-mux knob to an explicit axis (lights saw_mux_arm_duplication)",
                signoff_mux_arm_duplication_focus_config(strategy, next_seed + 1),
            ),
            (
                "signoff_array_packed_aggregate",
                "depth-1 wrapper, aggregate_prob = aggregate_array_prob = 1.0 over uniform-width data ports — promotes the deferred array-packed-aggregate knob to an explicit axis (lights saw_array_packed_aggregate_design)",
                signoff_array_packed_aggregate_focus_config(strategy, next_seed + 2),
            ),
            (
                "signoff_memory_fsm_interplay",
                "depth-1 wrapper, memory_prob = 0.5 + fsm_prob = 1.0 over 6 leaves — proves a memory module and an FSM module coexist in one design (lights saw_memory_fsm_interplay_design)",
                signoff_memory_fsm_interplay_focus_config(strategy, next_seed + 3),
            ),
        ] {
            scenarios.push(make_scenario(
                &format!("{strategy_slug}_nodeid_egraph_{name_suffix}"),
                &format!("{strategy_label} strategy, node-id + e-graph, {description_suffix}."),
                config,
            )?);
        }
        next_seed += 4;
    }

    Ok(scenarios)
}

/// `STRUCTURED-EMISSION-EXPANSION.2b.2b` — the repo-owned combinational
/// `function automatic` emit gate. For each of the three construction
/// strategies it emits one comb-only single-module DUT with
/// `function_emit_prob = 1.0`, so every qualifying combinational gate
/// (non-structured, non-`Slice`, non-soft_union, >= 1 operand) is
/// rendered as a behaviour-preserving `function automatic` over its
/// direct operands (decision `0012`). The caller (`--function-emit-gate`)
/// runs the full Verilator + both Yosys modes (+ Icarus when
/// `--iverilog-compile` is set) plan and requires the
/// `saw_combinational_function_emit` coverage fact, proving the first
/// richer-structured emission surface is accepted warning-clean.
///
/// Default `function_emit_prob = 0.0` emission is byte-identical to
/// today; the gate forces the non-default knob, exactly like the
/// `--signoff-knob-sweep-gate` template forces its previously-unswept
/// knobs.
fn build_function_emit_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();

    for (index, strategy) in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ]
    .into_iter()
    .enumerate()
    {
        let seed = base_seed + index as u64;
        let strategy_slug = strategy_slug(strategy);
        let strategy_label = construction_strategy_name(strategy);
        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_function_emit"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, comb-only DUT with function_emit_prob = 1.0 — projects every qualifying combinational gate to a behaviour-preserving `function automatic` over its direct operands (decision 0012; lights saw_combinational_function_emit)."
            ),
            function_emit_focus_config(strategy, seed),
        )?);
    }

    Ok(scenarios)
}

/// `STRUCTURED-EMISSION-EXPANSION.2b.2b` anchor config for the
/// combinational `function automatic` emit-projection. A comb-only
/// (`flop_prob = 0.0`) single-module DUT shaped like
/// `share_heavy_comb_only_config` (node-id + e-graph, rich combinational
/// cone) with `function_emit_prob = 1.0` so the gen-time
/// `annotate_function_emit_gates` pass marks every qualifying gate and the
/// emitter renders each as a `function automatic`. node-id + e-graph keeps
/// the cone shapes canonical. Downstream-clean across Verilator + both
/// Yosys modes + Icarus (the live surface was banked clean at
/// `/tmp/anvil-fe-r2/`).
fn function_emit_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        function_emit_prob: 1.0,
        flop_prob: 0.0,
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

/// `STRUCTURED-EMISSION-EXPANSION.4b.2b` — the repo-owned `generate for`
/// loop emit gate. For each of the three construction strategies it emits
/// one comb-only single-module DUT with `generate_loop_emit_prob = 1.0`, so
/// every qualifying `{N{x}}` 1-bit-lane replication (the common one-hot
/// `{W{sel}}` mux-mask idiom) is rendered as a behaviour-preserving
/// single-level `generate for` loop (decision `0013`). The caller
/// (`--generate-loop-gate`) runs the full Verilator + both Yosys modes (+
/// Icarus when `--iverilog-compile` is set) plan and requires the
/// `saw_generate_loop_emit` coverage fact, proving the second
/// richer-structured emission surface is accepted warning-clean.
///
/// Default `generate_loop_emit_prob = 0.0` emission is byte-identical to
/// today; the gate forces the non-default knob, exactly like the
/// `--function-emit-gate` template.
fn build_generate_loop_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();

    for (index, strategy) in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ]
    .into_iter()
    .enumerate()
    {
        let seed = base_seed + index as u64;
        let strategy_slug = strategy_slug(strategy);
        let strategy_label = construction_strategy_name(strategy);
        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_generate_loop"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, comb-only DUT with generate_loop_emit_prob = 1.0 — projects every qualifying {{N{{x}}}} 1-bit-lane replication to a behaviour-preserving single-level `generate for` loop (decision 0013; lights saw_generate_loop_emit)."
            ),
            generate_loop_focus_config(strategy, seed),
        )?);
    }

    Ok(scenarios)
}

/// `STRUCTURED-EMISSION-EXPANSION.4b.2b` anchor config for the `generate for`
/// loop emit-projection. A comb-only (`flop_prob = 0.0`) single-module DUT
/// shaped like `function_emit_focus_config` (node-id + e-graph, rich
/// combinational cone) with `generate_loop_emit_prob = 1.0` so the gen-time
/// `annotate_generate_loop_gates` pass marks every qualifying `{N{x}}`
/// 1-bit-lane replication and the emitter renders each as a `generate for`
/// loop. The rich combinational cone produces the one-hot `{W{sel}}`
/// mux-mask broadcasts that are the index-regular source; an empirical probe
/// lit a loop on every interleaved seed and the great majority of
/// shuffled/sequential seeds, and with 4 modules/scenario × 3 strategies the
/// `saw_generate_loop_emit` fact lights robustly. Downstream-clean across
/// Verilator + both Yosys modes + Icarus (the live surface was banked clean
/// at `/tmp/anvil-gl-r1/`).
fn generate_loop_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        generate_loop_emit_prob: 1.0,
        flop_prob: 0.0,
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

/// `STRUCTURED-EMISSION-EXPANSION.6b.2b` — the repo-owned combinational
/// `task automatic` emit gate. For each of the three construction strategies
/// it emits one comb-only single-module DUT with `task_emit_prob = 1.0`, so
/// every qualifying combinational gate is rendered as a behaviour-preserving
/// `task automatic` over its direct operands, called from `always_comb` into a
/// `<wire>__tv` output var (decision `0014`). The caller (`--task-emit-gate`)
/// runs the full Verilator + both Yosys modes (+ Icarus when
/// `--iverilog-compile` is set) plan and requires the
/// `saw_combinational_task_emit` coverage fact, proving the third
/// richer-structured emission surface is accepted warning-clean.
///
/// Default `task_emit_prob = 0.0` emission is byte-identical to today; the gate
/// forces the non-default knob, exactly like the `--function-emit-gate` /
/// `--generate-loop-gate` templates.
fn build_task_emit_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();

    for (index, strategy) in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ]
    .into_iter()
    .enumerate()
    {
        let seed = base_seed + index as u64;
        let strategy_slug = strategy_slug(strategy);
        let strategy_label = construction_strategy_name(strategy);
        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_task_emit"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, comb-only DUT with task_emit_prob = 1.0 — projects every qualifying combinational gate to a behaviour-preserving `task automatic` over its direct operands, called from always_comb (decision 0014; lights saw_combinational_task_emit)."
            ),
            task_emit_focus_config(strategy, seed),
        )?);
    }

    Ok(scenarios)
}

/// `STRUCTURED-EMISSION-EXPANSION.6b.2b` anchor config for the combinational
/// `task automatic` emit-projection. The task surface shares the
/// `function_emit` candidate set (any non-structured, non-`Slice` combinational
/// gate), so this is the `function_emit_focus_config` shape (a comb-only,
/// `flop_prob = 0.0`, node-id + e-graph rich combinational cone) with
/// `task_emit_prob = 1.0` instead — so the gen-time `annotate_task_emit_gates`
/// pass marks every qualifying gate and the emitter renders each as a
/// `task automatic` + `always_comb` call + passthrough `assign`. Downstream-clean
/// across Verilator + both Yosys modes + Icarus (the live surface was banked
/// clean at `/tmp/anvil-te-r1/`).
fn task_emit_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        task_emit_prob: 1.0,
        flop_prob: 0.0,
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

/// `STRUCTURED-EMISSION-EXPANSION.10b.2` — the repo-owned multi-gate-cone
/// `function automatic` emit gate. For each of the three construction
/// strategies it emits one comb-only single-module DUT with
/// `cone_function_emit_prob = 1.0`, so every qualifying combinational cone (a
/// root gate plus its single-use interior gates) is rendered as one
/// behaviour-preserving `function automatic` over the cone's boundary leaves,
/// with one function-local per absorbed interior gate (decision `0016`). The
/// caller (`--cone-function-gate`) runs the full Verilator + both Yosys modes
/// (+ Icarus when `--iverilog-compile` is set) plan and requires the
/// `saw_cone_function_emit` coverage fact, proving the fifth richer-structured
/// emission surface is accepted warning-clean.
///
/// Default `cone_function_emit_prob = 0.0` emission is byte-identical to today;
/// the gate forces the non-default knob, exactly like the `--function-emit-gate`
/// / `--task-emit-gate` templates.
fn build_cone_function_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();

    for (index, strategy) in [
        ConstructionStrategy::Sequential,
        ConstructionStrategy::Shuffled,
        ConstructionStrategy::Interleaved,
    ]
    .into_iter()
    .enumerate()
    {
        let seed = base_seed + index as u64;
        let strategy_slug = strategy_slug(strategy);
        let strategy_label = construction_strategy_name(strategy);
        scenarios.push(make_scenario(
            &format!("{strategy_slug}_nodeid_egraph_cone_function"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph, comb-only DUT with cone_function_emit_prob = 1.0 — projects every qualifying combinational cone (root + single-use interior gates) to one behaviour-preserving `function automatic` over the cone's boundary leaves (decision 0016; lights saw_cone_function_emit)."
            ),
            cone_function_focus_config(strategy, seed),
        )?);
    }

    Ok(scenarios)
}

/// `STRUCTURED-EMISSION-EXPANSION.10b.2` anchor config for the multi-gate-cone
/// `function automatic` emit-projection. Like the `function_emit` /
/// `task_emit` focus configs this is a comb-only (`flop_prob = 0.0`), node-id +
/// e-graph rich combinational cone with `cone_function_emit_prob = 1.0`, so the
/// gen-time `annotate_cone_function_gates` pass marks every qualifying cone and
/// the emitter renders each as a multi-statement `function automatic` + call.
/// One deliberate deviation from the `task_emit_focus_config` shape: a lower
/// `terminal_reuse_prob` (the default `0.3` instead of `0.9`). The cone surface
/// absorbs only **single-use** interior gates, and heavier terminal reuse drives
/// more CSE-induced sharing (multi-use interiors that stay boundary params), so
/// the lower reuse keeps single-use interior gates plentiful and the surface
/// reliably fires. Downstream-clean across Verilator + both Yosys modes + Icarus
/// (the live surface was banked clean at `/tmp/anvil-cf-sweep/`).
fn cone_function_focus_config(strategy: ConstructionStrategy, seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        cone_function_emit_prob: 1.0,
        flop_prob: 0.0,
        terminal_reuse_prob: 0.3,
        constant_prob: 0.05,
        max_depth: 8,
        min_inputs: 4,
        max_inputs: 8,
        min_outputs: 2,
        max_outputs: 4,
        ..Config::default()
    }
}

/// `SV-VERSION-TARGETING.2b.2b` — bare-year slug for an `SvVersion`,
/// used in scenario names (`sv2017_comb_egraph`).
fn sv_version_year_slug(version: SvVersion) -> &'static str {
    match version {
        SvVersion::Sv2012 => "2012",
        SvVersion::Sv2017 => "2017",
        SvVersion::Sv2023 => "2023",
    }
}

/// `SV-VERSION-TARGETING.2b.2b` — the repo-owned per-version acceptance
/// matrix. For each of the three IEEE 1800 targets (2012/2017/2023) it
/// emits a focused, deterministic corpus — a combinational e-graph leaf,
/// a sequential motif leaf, and a recursive depth-2 hierarchy design —
/// each carrying `Config::sv_version` set to that target. The caller
/// (`--sv-version-gate`) runs Verilator in the matching
/// `--language 1800-20xx` standard mode (via the `.2b.2a` selector), so
/// the gate proves each version-targeted corpus is *accepted in the
/// matching tool standard mode*, lighting the per-version
/// `saw_sv_version_*_targeted_acceptance` facts.
///
/// All artifact kinds use the `Interleaved` strategy: the gate's
/// contract is per-version acceptance, not construction-strategy breadth
/// (the other gates own that), so `compute_coverage_gaps` returns early
/// for this set without the strategy/motif/category checks.
///
/// The first nine scenarios (3 targets × {comb leaf, seq leaf, recursive
/// hierarchy design}) are byte-identical across the three targets — the
/// current subset is a 2012/2017/2023 common floor — so their value is
/// the per-version downstream acceptance axis, not output divergence.
/// `SV-VERSION-TARGETING.3b.2b` adds a tenth scenario, the **up-opt**:
/// a slice-heavy 2023-targeted leaf with `soft_union_slice_prob = 1.0`
/// that genuinely diverges — every qualifying proper low-bits `Slice`
/// renders the IEEE 1800-2023 `union soft` overlay. That scenario runs
/// Verilator-only (Yosys/Icarus reject the `union soft` syntax → a
/// recorded no-op per decision `0010`) and lights the dedicated
/// `saw_sv_version_2023_soft_union_upopt` fact.
fn build_sv_version_sweep_scenarios(base_seed: u64) -> Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    let mut next_seed = base_seed;
    let strategy = ConstructionStrategy::Interleaved;
    let strategy_label = construction_strategy_name(strategy);

    for version in [SvVersion::Sv2012, SvVersion::Sv2017, SvVersion::Sv2023] {
        let year = sv_version_year_slug(version);
        let std = version.ieee_standard();

        let mut comb = share_heavy_comb_only_config(strategy, next_seed, 0.9);
        comb.sv_version = version;
        scenarios.push(make_scenario(
            &format!("sv{year}_comb_egraph"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph combinational leaf targeting IEEE {std}; proves the version-targeted combinational corpus is accepted by Verilator --language {std} + Yosys -sv."
            ),
            comb,
        )?);

        let mut seq = motif_heavy_sequential_config(strategy, next_seed + 1, 0.4);
        seq.sv_version = version;
        scenarios.push(make_scenario(
            &format!("sv{year}_seq_motif"),
            &format!(
                "{strategy_label} strategy, node-id + e-graph sequential motif leaf targeting IEEE {std}; proves the version-targeted sequential corpus is accepted in the matching tool standard mode."
            ),
            seq,
        )?);

        let mut hier =
            phase4_recursive_canonical_module_signature_focus_config(strategy, next_seed + 2);
        hier.sv_version = version;
        scenarios.push(make_scenario(
            &format!("sv{year}_hier_recursive"),
            &format!(
                "{strategy_label} strategy, recursive depth-2 hierarchy design targeting IEEE {std}; proves the version-targeted multi-file design path emits and is accepted in the matching tool standard mode."
            ),
            hier,
        )?);

        next_seed += 3;
    }

    // `SV-VERSION-TARGETING.3b.2b` — the up-opt scenario. A single
    // slice-heavy 2023-targeted leaf with `soft_union_slice_prob = 1.0`
    // genuinely diverges from the common floor: every qualifying proper
    // low-bits `Slice` renders the IEEE 1800-2023 `union soft` overlay.
    // Verilator-only (Yosys/Icarus reject the syntax → recorded no-op,
    // decision `0010`); lights `saw_sv_version_2023_soft_union_upopt`.
    scenarios.push(make_scenario(
        "sv2023_soft_union_upopt",
        &format!(
            "{strategy_label} strategy, slice-heavy combinational leaf targeting IEEE 1800-2023 with soft_union_slice_prob=1.0; proves the live `union soft` low-bits-slice up-opt is emitted and accepted by Verilator --language 1800-2023. Yosys/Icarus reject the syntax and are a recorded no-op (decision 0010)."
        ),
        soft_union_upopt_config(next_seed),
    )?);

    Ok(scenarios)
}

/// `SV-VERSION-TARGETING.3b.2b` — the up-opt scenario config. The proven
/// `.3b.2a` slice-heavy recipe: a high structured-gate weight + wide
/// widths make proper low-bits `Slice` gates plentiful, and
/// `soft_union_slice_prob = 1.0` marks every qualifying one, so the
/// `Sv2023` target reliably emits the `union soft` overlay (banked clean
/// at 159 overlays / 7 seeds in `tests/sv_version_downstream.rs`).
/// `Interleaved` matches the rest of the sweep.
fn soft_union_upopt_config(seed: u64) -> Config {
    Config {
        seed,
        construction_strategy: ConstructionStrategy::Interleaved,
        sv_version: SvVersion::Sv2023,
        soft_union_slice_prob: 1.0,
        gate_struct_weight: 10,
        gate_bitwise_weight: 1,
        gate_arith_weight: 1,
        min_width: 4,
        max_width: 16,
        max_depth: 5,
        min_inputs: 3,
        max_inputs: 6,
        min_outputs: 2,
        max_outputs: 4,
        ..Config::default()
    }
}

fn phase4_hierarchy_structurally_duplicate_modules_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // HIERARCHY-AWARE-IDENTITY.2 anchor scenario. Tight 1-in / 1-out /
    // width-1 / max_depth-1 leaves collapse to a single canonical
    // structure, so all four library leaves share a canonical signature
    // and `num_structurally_duplicate_module_pairs > 0`. This gives the
    // future dedup pass (H-A-I.4) a live example to exercise.
    //
    // Intentionally does NOT inherit from `share_heavy_comb_only_config`
    // — that helper sets `min_inputs = 4`, which would make leaves
    // structurally diverse and defeat the test. This scenario is the
    // one Phase 4 hierarchy scenario whose hierarchy-routing
    // probabilities are all 0.0 by design; its suffix
    // `phase4_hier1_structurally_duplicate_modules` is in the matrix
    // unit-test exception list at the bottom of this file.
    Config {
        seed,
        construction_strategy: strategy,
        identity_mode: IdentityMode::NodeId,
        factorization_level: FactorizationLevel::EGraph,
        hierarchy_depth: 1,
        num_leaf_modules: 4,
        num_child_instances: 4,
        min_inputs: 1,
        max_inputs: 1,
        min_outputs: 1,
        max_outputs: 1,
        min_width: 1,
        max_width: 1,
        flop_prob: 0.0,
        hierarchy_sibling_route_prob: 0.0,
        hierarchy_registered_sibling_route_prob: 0.0,
        hierarchy_registered_child_input_cone_prob: 0.0,
        hierarchy_child_input_cone_prob: 0.0,
        hierarchy_parent_cone_instance_prob: 0.0,
        hierarchy_parent_flop_prob: 0.0,
        max_flops_per_module: 0,
        terminal_reuse_prob: 1.0,
        constant_prob: 0.0,
        max_depth: 1,
        ..Config::default()
    }
}

fn phase4_recursive_canonical_module_signature_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // First slice of hierarchy-aware identity (PNT-3): a vanilla
    // recursive hierarchy at depth 2 with 4,4 children, exercised purely
    // to anchor canonical_module_signatures instrumentation in the
    // matrix. The metric is computed for every design, so every
    // scenario contributes; this scenario is just an explicit gate-time
    // anchor for the new fact. Future slices will use the same
    // instrumentation to dedupe Design::modules when
    // IdentityMode::NodeId is active.
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

fn phase4_recursive_parent_cone_instance_budget_5_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Extends the budget-3 helper config to budget 5. Uses 4,4 child
    // instances at depth 2 so that each parent has ~4 children x ~2
    // inputs = 8 child-input decision sites where helper allocation can
    // fire; that demand comfortably saturates a budget of 5 helpers per
    // parent. Mirrors r83 in style: a single-axis extension (helper
    // budget instead of chain depth) above the closed depth-3..7 sweeps.
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
    cfg.hierarchy_parent_cone_instance_prob = 1.0;
    cfg.max_parent_cone_instances_per_module = 5;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 4;
    cfg
}

fn phase4_recursive_registered_three_stage_parent_composed_chain_focus_config(
    strategy: ConstructionStrategy,
    seed: u64,
) -> Config {
    // Pushes the existing 2-stage registered parent-composed chain
    // subcase to chain length >= 3. The planner does not have a knob
    // that forces a particular chain length; instead this config gives
    // the planner enough flop budget and cone depth that chain-length-3
    // structures emerge naturally below the top across all
    // ConstructionStrategy values. Depth 3 with 4,4 children gives
    // multiple non-top internal parents per design, max_flops=128 lets
    // each parent allocate enough parent-local Qs, and max_depth=8
    // widens the registered child-input D-cones so they can reach back
    // through two prior Qs.
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
    cfg.hierarchy_registered_child_input_cone_prob = 1.0;
    cfg.hierarchy_child_input_cone_prob = 0.0;
    cfg.hierarchy_parent_cone_instance_prob = 0.0;
    cfg.hierarchy_parent_flop_prob = 0.0;
    cfg.max_flops_per_module = 128;
    cfg.terminal_reuse_prob = 1.0;
    cfg.constant_prob = 0.0;
    cfg.max_depth = 8;
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

/// `SV-VERSION-TARGETING.2b.2b` — the Verilator `--language 1800-20xx`
/// selector for a scenario. `Some(std)` only under the per-version gate
/// (`version_targeted`), where downstream tools run in the matching
/// standard mode; `None` for every other run, preserving today's
/// byte-identical Verilator argv.
fn verilator_language_for(scenario: &Scenario, version_targeted: bool) -> Option<&'static str> {
    version_targeted.then(|| scenario.config.sv_version.ieee_standard())
}

/// `SV-VERSION-TARGETING.3b.2b` — true iff this scenario's emission carries
/// the IEEE 1800-2023 `union soft` up-opt overlay: `soft_union_slice_prob`
/// is on **and** the target permits 2023 (below 2023 the overlay down-gates
/// to a plain slice every tool accepts). Yosys/Icarus reject the `union
/// soft` syntax (no 1800 selector, fixed subset), so such a scenario runs
/// Verilator-only — Yosys/Icarus are a *recorded no-op* (decision `0010`).
/// The tool plan is therefore a pure function of the scenario config, not a
/// separate flag.
fn scenario_emits_soft_union_overlay(scenario: &Scenario) -> bool {
    scenario.config.soft_union_slice_prob > 0.0
        && scenario.config.sv_version.permits(SvVersion::Sv2023)
}

fn run_scenario(
    scenario: &Scenario,
    cli: &Cli,
    plan: &RunPlan,
    out_root: &Path,
    runtime_fingerprint: Option<&str>,
    version_targeted: bool,
) -> Result<ScenarioReport> {
    if scenario.config.effective_hierarchy_depth_range().is_some() {
        return run_design_scenario(
            scenario,
            cli,
            plan,
            out_root,
            runtime_fingerprint,
            version_targeted,
        );
    }

    run_module_scenario(
        scenario,
        cli,
        plan,
        out_root,
        runtime_fingerprint,
        version_targeted,
    )
}

fn run_module_scenario(
    scenario: &Scenario,
    cli: &Cli,
    plan: &RunPlan,
    out_root: &Path,
    runtime_fingerprint: Option<&str>,
    version_targeted: bool,
) -> Result<ScenarioReport> {
    let scenario_dir = out_root.join(&scenario.name);
    std::fs::create_dir_all(&scenario_dir)
        .with_context(|| format!("create scenario directory {}", scenario_dir.display()))?;

    // `SV-VERSION-TARGETING.2b.2b` — under the per-version gate, run
    // Verilator in the scenario's matching `--language 1800-20xx` mode;
    // otherwise keep today's byte-identical argv (`None`).
    let verilator_language = verilator_language_for(scenario, version_targeted);
    // `SV-VERSION-TARGETING.3b.2b` — a `union soft` up-opt scenario runs
    // Verilator-only; Yosys/Icarus are a recorded no-op for it.
    let verilator_only = scenario_emits_soft_union_overlay(scenario);

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
            verilator_language,
            verilator_only,
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
            verilator_language,
            verilator_only,
        )?);
    }

    write_scenario_manifest(&scenario_dir, scenario, &modules)?;

    let aggregate = aggregate_metrics(&modules);
    let coverage = summarize_coverage(scenario, &modules, version_targeted);
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
    version_targeted: bool,
) -> Result<ScenarioReport> {
    let scenario_dir = out_root.join(&scenario.name);
    std::fs::create_dir_all(&scenario_dir)
        .with_context(|| format!("create scenario directory {}", scenario_dir.display()))?;

    // `SV-VERSION-TARGETING.2b.2b` — the scenario's emission target plus
    // the matching-mode Verilator selector (active only under the gate).
    let sv_version = scenario.config.sv_version;
    let verilator_language = verilator_language_for(scenario, version_targeted);

    let mut generator = Generator::new(scenario.config.clone());
    let mut designs = Vec::with_capacity(plan.modules_per_scenario);

    for design_index in 0..plan.modules_per_scenario {
        if let Some(report) = resume_existing_design(
            &mut generator,
            cli,
            &scenario_dir,
            design_index,
            runtime_fingerprint,
            sv_version,
            verilator_language,
        )? {
            designs.push(report);
            continue;
        }

        let prepared = prepare_design(&mut generator, &scenario_dir, design_index, sv_version)?;
        let generator_checkpoint = generator.checkpoint();
        designs.push(materialize_prepared_design(
            cli,
            &prepared,
            &generator_checkpoint,
            runtime_fingerprint,
            true,
            verilator_language,
        )?);
    }

    write_design_scenario_manifest(&scenario_dir, scenario, &designs)?;

    let aggregate = aggregate_design_metrics(&designs);
    let coverage = summarize_design_coverage(scenario, &designs, version_targeted);
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

// `verilator_only` (`SV-VERSION-TARGETING.3b.2b`) joins the existing
// downstream-tool plumbing; mirrors the wide-plumbing `#[allow]` already
// used across the generator (e.g. `gen/hierarchy.rs`).
#[allow(clippy::too_many_arguments)]
fn resume_existing_module(
    generator: &mut Generator,
    scenario: &Scenario,
    cli: &Cli,
    scenario_dir: &Path,
    module_index: usize,
    runtime_fingerprint: Option<&str>,
    verilator_language: Option<&str>,
    verilator_only: bool,
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
        verilator_language,
        verilator_only,
    )
    .map(Some)
}

fn resume_existing_design(
    generator: &mut Generator,
    cli: &Cli,
    scenario_dir: &Path,
    design_index: usize,
    runtime_fingerprint: Option<&str>,
    sv_version: SvVersion,
    verilator_language: Option<&str>,
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

    let prepared = prepare_design(generator, scenario_dir, design_index, sv_version)?;
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
        run_design_tools(cli, &prepared, verilator_language)?
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
    sv_version: SvVersion,
) -> Result<PreparedDesign> {
    let paths = design_paths(scenario_dir, design_index);
    let design = generator.generate_design();
    anvil::ir::validate::validate_design(&design).map_err(|err| anyhow::anyhow!("{err}"))?;
    prepared_design_from_design(paths, design_index, &design, scenario_dir, sv_version)
}

fn prepared_design_from_design(
    paths: DesignPaths,
    design_index: usize,
    design: &Design,
    scenario_dir: &Path,
    sv_version: SvVersion,
) -> Result<PreparedDesign> {
    let metrics = anvil::metrics::compute_design(design);
    let hierarchy = hierarchy_facts_from_design(design, design_index, &metrics)?;
    let mut modules = Vec::with_capacity(design.modules.len());
    for module in &design.modules {
        let metrics = anvil::metrics::compute(module);
        let file = format!("{}.sv", module.name);
        let sv_path = scenario_dir.join(&file);
        // `SV-VERSION-TARGETING.2b.2b` — emit at the scenario's target.
        // Byte-identical to `to_sv_in_design` at the `Sv2012` floor every
        // non-gate scenario uses (the current subset is a common floor).
        let sv_text = anvil::emit::to_sv_in_design_versioned(module, design, sv_version);
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
    scenario: &Scenario,
    paths: ModulePaths,
) -> Result<PreparedModule> {
    let module = generator.generate_module();
    let metrics = anvil::metrics::compute(&module);
    // `SV-VERSION-TARGETING.2b.2b` — emit at the scenario's target.
    // Byte-identical to `to_sv` at the `Sv2012` floor every non-gate
    // scenario uses (the current subset is a 2012/2017/2023 common floor).
    let sv_text = anvil::emit::to_sv_versioned(&module, scenario.config.sv_version);
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
    verilator_language: Option<&str>,
) -> Result<DesignReport> {
    if write_sv {
        for module in &prepared.modules {
            std::fs::write(&module.sv_path, &module.sv_text)
                .with_context(|| format!("write {}", module.sv_path.display()))?;
        }
    }

    let report = run_design_tools(cli, prepared, verilator_language)?;
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

#[allow(clippy::too_many_arguments)]
fn materialize_prepared_module(
    cli: &Cli,
    scenario_dir: &Path,
    prepared: PreparedModule,
    generator_checkpoint: &GeneratorCheckpoint,
    runtime_fingerprint: Option<&str>,
    write_sv: bool,
    verilator_language: Option<&str>,
    verilator_only: bool,
) -> Result<ModuleReport> {
    if write_sv {
        std::fs::write(&prepared.paths.sv_path, &prepared.sv_text)
            .with_context(|| format!("write {}", prepared.paths.sv_path.display()))?;
    }

    // `SV-VERSION-TARGETING.3b.2b` — real evidence the IEEE 1800-2023
    // `union soft` overlay was actually emitted (not just requested by the
    // knob): the emitted SV text carries it. Drives the up-opt coverage
    // fact, and is honest about emission even if a seed produced no
    // qualifying slice.
    let emitted_soft_union_overlay = prepared.sv_text.contains("union soft");

    // `STRUCTURED-EMISSION-EXPANSION.2b.2b` — real evidence the
    // combinational `function automatic` emit-projection was actually
    // emitted (not just requested by `function_emit_prob`): the emitted
    // SV text carries the declaration. Mirrors the
    // `emitted_soft_union_overlay` precedent and stays honest even if a
    // seed produced no qualifying gate.
    let emitted_combinational_function = prepared.sv_text.contains("function automatic");

    // `STRUCTURED-EMISSION-EXPANSION.4b.2b` — real evidence the `generate
    // for` loop emit-projection was actually emitted (not just requested by
    // `generate_loop_emit_prob`): the emitted SV text carries a `generate`
    // region. Mirrors the `emitted_combinational_function` precedent and
    // stays honest even if a seed produced no qualifying replication.
    let emitted_generate_loop = prepared.sv_text.contains("generate");

    // `STRUCTURED-EMISSION-EXPANSION.6b.2b` — real evidence the combinational
    // `task automatic` emit-projection was actually emitted (not just requested
    // by `task_emit_prob`): the emitted SV text carries the declaration.
    // Mirrors the `emitted_combinational_function` precedent and stays honest
    // even if a seed produced no qualifying gate.
    let emitted_combinational_task = prepared.sv_text.contains("task automatic");

    // `STRUCTURED-EMISSION-EXPANSION.10b.2` — real evidence the multi-gate-cone
    // `function automatic` emit-projection was actually emitted (not just
    // requested by `cone_function_emit_prob`): the emitted SV text carries a
    // `<root>__cf(` token (the cone-function decl + call). This is distinct from
    // the single-gate `function_emit` `<wire>__f(` surface, so it stays honest
    // even when both knobs are off or a seed produced no qualifying cone.
    let emitted_cone_function = prepared.sv_text.contains("__cf(");

    let (verilator, yosys, iverilog_compile, sv2v) = run_module_tools(
        cli,
        scenario_dir,
        &prepared.paths.sv_path,
        &prepared.paths.stem,
        verilator_language,
        verilator_only,
    )?;

    // `DIFFERENTIAL-SIMULATION.3b.2` — opt-in diff-sim column.
    // Runs only when `--diff-sim` is set AND Verilator+Yosys are
    // both clean on this module (the existing "downstream tools
    // already accepted the SV" precondition from `.3a`). The
    // per-axis subset selector is applied at scenario-level by the
    // caller via `diff_sim_runs_for_scenario`; here we trust
    // `cli.diff_sim` AND a precondition check.
    let diff_sim = if cli.diff_sim
        && tool_invocation_ok(verilator.as_ref())
        && all_yosys_invocations_ok(&yosys)
        && scenario_in_diff_sim_subset(scenario_dir)
    {
        Some(run_diff_sim_for_module(
            scenario_dir,
            &prepared.paths.stem,
            &prepared.name,
            &prepared.sv_text,
        ))
    } else {
        None
    };

    // `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — classify the tools just run on this
    // module for acceptance divergence (a pure projection; no extra spawn).
    let divergence = unit_divergence(
        cli,
        scenario_dir,
        "module",
        &prepared.name,
        verilator.as_ref(),
        &yosys,
        iverilog_compile.as_ref(),
        sv2v.as_ref(),
    );

    let report = ModuleReport {
        file: prepared.paths.file.clone(),
        name: prepared.name,
        metrics: prepared.metrics,
        verilator,
        yosys,
        iverilog_compile,
        sv2v,
        diff_sim,
        divergence,
        emitted_soft_union_overlay,
        emitted_combinational_function,
        emitted_generate_loop,
        emitted_combinational_task,
        emitted_cone_function,
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

/// `DIFFERENTIAL-SIMULATION.3b.2` — true when the prior tool
/// invocation succeeded. Helper around the `success` bit so the
/// precondition reads cleanly. `None` means the tool was skipped
/// (`--skip-verilator`), which still satisfies the precondition —
/// there is no Verilator failure to gate on.
fn tool_invocation_ok(inv: Option<&ToolInvocation>) -> bool {
    match inv {
        Some(t) => t.success,
        None => true,
    }
}

/// `DIFFERENTIAL-SIMULATION.3b.2` — every recorded Yosys
/// invocation must succeed (the `WithoutAbc`/`WithAbc`/`Both`
/// modes produce 1 or 2 invocations). Empty Vec satisfies the
/// precondition (`--skip-yosys`).
fn all_yosys_invocations_ok(invocations: &[ToolInvocation]) -> bool {
    invocations.iter().all(|t| t.success)
}

/// `DIFFERENTIAL-SIMULATION.3b.2` — read the diff-sim subset
/// sentinel file written by `run_matrix`. The matrix computes the
/// per-axis subset once at top level and persists the chosen names
/// to `<scenario_dir>/../.diff-sim-subset`; this helper checks
/// whether the current scenario's directory is in it. The sentinel
/// pattern keeps `materialize_prepared_module`'s signature stable
/// (it already takes `scenario_dir` and doesn't see the broader
/// scenario list).
fn scenario_in_diff_sim_subset(scenario_dir: &Path) -> bool {
    scenario_in_named_subset(scenario_dir, ".diff-sim-subset")
}

/// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — the divergence-column counterpart of
/// `scenario_in_diff_sim_subset`, reading the parallel `.divergence-subset`
/// sentinel written by `main` when `--divergence` is set. Shares the
/// membership logic via `scenario_in_named_subset`.
fn scenario_in_divergence_subset(scenario_dir: &Path) -> bool {
    scenario_in_named_subset(scenario_dir, ".divergence-subset")
}

/// Shared per-axis-subset membership check used by both the diff-sim column
/// (`.diff-sim-subset`) and the acceptance-divergence column
/// (`.divergence-subset`). The matrix computes each per-axis subset once at top
/// level and persists the chosen scenario names to `<out>/<sentinel_name>`;
/// this checks whether `scenario_dir`'s own directory name is in it. The
/// sentinel pattern keeps the per-scenario materialization signatures stable
/// (they already take `scenario_dir` and don't see the broader scenario list).
fn scenario_in_named_subset(scenario_dir: &Path, sentinel_name: &str) -> bool {
    let Some(parent) = scenario_dir.parent() else {
        return false;
    };
    let sentinel = parent.join(sentinel_name);
    let Ok(contents) = std::fs::read_to_string(&sentinel) else {
        // Defensive: if the sentinel is missing, evaluate the column for
        // EVERY scenario rather than silently skipping (the user explicitly
        // opted in with the column's flag). This also makes the column path
        // testable from focused unit/integration tests that don't go through
        // `run_matrix`.
        return true;
    };
    let Some(name) = scenario_dir.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == name)
}

/// `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — the per-unit acceptance-divergence
/// column. Unlike `--diff-sim`, this spawns **no** extra tool: it is a pure
/// projection of the tool invocations the matrix **already ran**, so it does
/// **not** require Verilator/Yosys to be clean first — a divergence is most
/// interesting exactly when one tool rejects what another accepts. Gated by
/// `--divergence` + the shared per-axis subset (`.divergence-subset`). It
/// assembles the already-run invocations into a [`ValidateReport`] and
/// classifies it through the **one** shared detector
/// `divergence::classify_report` (the same classifier the hunt loop uses — no
/// second copy, the full-factorization doctrine). Returns `None` when the
/// column is off / the scenario is out of subset / no tool ran on the unit.
///
/// The `run_id` is the unit's `top` name (a stable per-report identifier);
/// the matrix retains the actual `.sv` on disk per the reproducer policy, so it
/// does not content-address here.
// One collector parameter per acceptance column (Verilator / Yosys /
// iverilog-compile / sv2v) on top of cli + scenario_dir + kind + top. The
// per-column arguments are intentional — it checks the off/out-of-subset cases
// *before* assembling, so the default path clones nothing.
#[allow(clippy::too_many_arguments)]
fn unit_divergence(
    cli: &Cli,
    scenario_dir: &Path,
    kind: &str,
    top: &str,
    verilator: Option<&ToolInvocation>,
    yosys: &[ToolInvocation],
    iverilog_compile: Option<&ToolInvocation>,
    sv2v: Option<&ToolInvocation>,
) -> Option<DivergenceReport> {
    if !cli.divergence || !scenario_in_divergence_subset(scenario_dir) {
        return None;
    }
    let mut tools: Vec<ToolInvocation> = Vec::new();
    if let Some(v) = verilator {
        tools.push(v.clone());
    }
    tools.extend(yosys.iter().cloned());
    if let Some(i) = iverilog_compile {
        tools.push(i.clone());
    }
    if let Some(s) = sv2v {
        tools.push(s.clone());
    }
    if tools.is_empty() {
        return None;
    }
    let ok = tools.iter().all(|inv| inv.success);
    let report = ValidateReport {
        run_id: top.to_string(),
        lane: "dut".to_string(),
        kind: kind.to_string(),
        top: top.to_string(),
        sandbox: scenario_dir.display().to_string(),
        tools,
        ok,
        declined: None,
    };
    Some(divergence::classify_report(&report))
}

/// `BUG-HUNT-ORCHESTRATION.2a` — per-module diff-sim wrapper. The
/// run+compare pipeline (port parse → testbench → dual-simulator run
/// → trace compare) now lives in `anvil::diff_sim::run_agreement`; this
/// computes the per-module work dir and delegates. The behaviour (and the
/// emitted `tb.sv` / `DiffSimReport`) is byte-identical to the prior in-binary
/// implementation.
fn run_diff_sim_for_module(
    scenario_dir: &Path,
    stem: &str,
    top_name: &str,
    sv_text: &str,
) -> DiffSimReport {
    let dir = scenario_dir.join(format!("{stem}-diff-sim"));
    anvil::diff_sim::run_agreement(&dir, top_name, sv_text, 8)
}

/// `DIFFERENTIAL-SIMULATION.3b.2` — per-axis subset selector per
/// `.3a`'s design. Picks the first scenario per major axis
/// (memory → fsm → hierarchy → sequential-flop → combinational),
/// capped at K=5, deterministic. The diff-sim column runs only
/// on the returned scenario names. Per-axis is preferred over
/// random-N because it preserves curated coverage shape; rejected
/// hand-curated lists because they'd require updating per new
/// scenario set.
fn select_diff_sim_subset(scenarios: &[Scenario]) -> Vec<String> {
    let mut picked: Vec<String> = Vec::new();
    let mut axes_seen: BTreeSet<&'static str> = BTreeSet::new();
    for scenario in scenarios {
        if picked.len() >= 5 {
            break;
        }
        let axis = classify_diff_sim_axis(&scenario.config);
        if axes_seen.insert(axis) {
            picked.push(scenario.name.clone());
        }
    }
    picked
}

/// `DIFFERENTIAL-SIMULATION.3b.2` — bucket a scenario into one of
/// the five major axes per `.3a`'s design. Most-specific first:
/// a memory scenario also has flops, so the `memory` axis takes
/// precedence over `sequential-flop`.
fn classify_diff_sim_axis(cfg: &Config) -> &'static str {
    if cfg.memory_prob > 0.0 {
        "memory"
    } else if cfg.fsm_prob > 0.0 {
        "fsm"
    } else if cfg.effective_hierarchy_depth_range().is_some() {
        "hierarchy"
    } else if cfg.flop_prob > 0.0 {
        "sequential-flop"
    } else {
        "combinational"
    }
}

/// The per-module acceptance columns [`run_module_tools`] produces, in report
/// order: Verilator (one invocation), Yosys (1–2 per [`YosysMode`]), the opt-in
/// Icarus compile column, and the opt-in `sv2v` transpile column
/// (`DOWNSTREAM-ADAPTER-EXPANSION.2b.2`). Factored into an alias to keep the
/// signature readable (clippy `type_complexity`).
type ModuleToolColumns = (
    Option<ToolInvocation>,
    Vec<ToolInvocation>,
    Option<ToolInvocation>,
    Option<ToolInvocation>,
);

fn run_module_tools(
    cli: &Cli,
    scenario_dir: &Path,
    sv_path: &Path,
    stem: &str,
    verilator_language: Option<&str>,
    // `SV-VERSION-TARGETING.3b.2b` — a `union soft` up-opt module runs
    // Verilator-only: Yosys/Icarus reject the syntax and are a recorded
    // no-op (empty Yosys vec / `None` Icarus), decision `0010`.
    verilator_only: bool,
) -> Result<ModuleToolColumns> {
    // Dispatch each fixed column through the closed adapter registry
    // (`DOWNSTREAM-ADAPTER-EXPANSION.2a.3`, decision `0020`) instead of calling
    // the `run_*` primitives directly. Byte-identical: each built-in adapter's
    // `run` delegates verbatim to the same primitive with the same binary /
    // out_dir / Yosys mode / `--language` selector, so the fixed
    // verilator/yosys/iverilog_compile columns — and banked reports + `--resume`
    // — are unchanged. Routing through the registry is the bridge that makes a
    // new acceptance column (`sv2v`, `.2b`) a near-one-line registry add.
    let target = AdapterTarget::Module { sv_path, stem };
    let run_column = |tool: AcceptanceTool,
                      binary: &str,
                      language: Option<&str>|
     -> Result<Vec<ToolInvocation>> {
        let cx = AdapterRunCx {
            binary,
            out_dir: scenario_dir,
            target,
            yosys_mode: cli.yosys_mode,
            language,
        };
        tool.adapter().run(&cx)
    };

    let verilator = if cli.skip_verilator {
        None
    } else {
        run_column(
            AcceptanceTool::Verilator,
            &cli.verilator_bin,
            verilator_language,
        )?
        .into_iter()
        .next()
    };

    let yosys = if cli.skip_yosys || verilator_only {
        Vec::new()
    } else {
        run_column(AcceptanceTool::Yosys, &cli.yosys_bin, None)?
    };

    let iverilog_compile = if cli.iverilog_compile && !verilator_only {
        run_column(AcceptanceTool::Iverilog, &cli.iverilog_bin, None)?
            .into_iter()
            .next()
    } else {
        None
    };

    // `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — the opt-in sv2v transpile column.
    // Skipped for `union soft` up-opt modules alongside Yosys/Icarus (sv2v
    // targets Verilog and does not accept the SV-2023 `union soft` syntax), and
    // a **friendly no-op** when sv2v is absent: a presence probe (decision
    // `0020`, the diff-sim `tools_present()` precedent) means a requested-but-
    // missing sv2v records no column and never bails the run.
    let sv2v = if cli.sv2v && !verilator_only && tool_version(&cli.sv2v_bin).is_some() {
        run_column(AcceptanceTool::Sv2v, &cli.sv2v_bin, None)?
            .into_iter()
            .next()
    } else {
        None
    };

    Ok((verilator, yosys, iverilog_compile, sv2v))
}

fn run_design_tools(
    cli: &Cli,
    prepared: &PreparedDesign,
    verilator_language: Option<&str>,
) -> Result<DesignReport> {
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

    // Dispatch each fixed column through the closed adapter registry
    // (`DOWNSTREAM-ADAPTER-EXPANSION.2a.3`, decision `0020`), exactly as
    // `run_module_tools` does for the leaf path. Byte-identical: each built-in
    // adapter delegates verbatim to the same `run_*_design` primitive, so the
    // fixed verilator/yosys/iverilog_compile design columns — and banked reports
    // + `--resume` — are unchanged.
    let target = AdapterTarget::Design {
        sv_paths: &sv_paths,
        top: &prepared.top,
    };
    let run_column = |tool: AcceptanceTool,
                      binary: &str,
                      language: Option<&str>|
     -> Result<Vec<ToolInvocation>> {
        let cx = AdapterRunCx {
            binary,
            out_dir: scenario_dir,
            target,
            yosys_mode: cli.yosys_mode,
            language,
        };
        tool.adapter().run(&cx)
    };

    let verilator = if cli.skip_verilator {
        None
    } else {
        run_column(
            AcceptanceTool::Verilator,
            &cli.verilator_bin,
            verilator_language,
        )?
        .into_iter()
        .next()
    };

    let yosys = if cli.skip_yosys {
        Vec::new()
    } else {
        run_column(AcceptanceTool::Yosys, &cli.yosys_bin, None)?
    };

    let iverilog_compile = if cli.iverilog_compile {
        run_column(AcceptanceTool::Iverilog, &cli.iverilog_bin, None)?
            .into_iter()
            .next()
    } else {
        None
    };

    // `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — the opt-in sv2v transpile column;
    // a friendly no-op when sv2v is absent (presence probe, decision `0020`).
    let sv2v = if cli.sv2v && tool_version(&cli.sv2v_bin).is_some() {
        run_column(AcceptanceTool::Sv2v, &cli.sv2v_bin, None)?
            .into_iter()
            .next()
    } else {
        None
    };

    // `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — classify the tools just run on this
    // design for acceptance divergence (a pure projection; no extra spawn).
    let divergence = unit_divergence(
        cli,
        scenario_dir,
        "design",
        &prepared.top,
        verilator.as_ref(),
        &yosys,
        iverilog_compile.as_ref(),
        sv2v.as_ref(),
    );

    Ok(DesignReport {
        index: prepared.index,
        top: prepared.top.clone(),
        files,
        modules,
        hierarchy: prepared.hierarchy.clone(),
        metrics: prepared.metrics.clone(),
        verilator,
        yosys,
        iverilog_compile,
        sv2v,
        divergence,
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
        iverilog_compile: cli.iverilog_compile,
        sv2v: cli.sv2v,
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
        iverilog_compile: cli.iverilog_compile,
        sv2v: cli.sv2v,
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
        && checkpoint.iverilog_compile == cli.iverilog_compile
        && checkpoint.sv2v == cli.sv2v
        && checkpoint.yosys_mode == yosys_mode_slug(cli.yosys_mode)
}

fn checkpoint_matches_design_cli(checkpoint: &DesignCheckpoint, cli: &Cli) -> bool {
    checkpoint.skip_verilator == cli.skip_verilator
        && checkpoint.skip_yosys == cli.skip_yosys
        && checkpoint.iverilog_compile == cli.iverilog_compile
        && checkpoint.sv2v == cli.sv2v
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
        accumulate_tool_summary(
            &mut summary,
            module.verilator.as_ref(),
            &module.yosys,
            module.iverilog_compile.as_ref(),
            module.sv2v.as_ref(),
        );
    }
    summary
}

fn summarize_design_tools(designs: &[DesignReport]) -> ToolSummary {
    let mut summary = ToolSummary::default();
    for design in designs {
        accumulate_tool_summary(
            &mut summary,
            design.verilator.as_ref(),
            &design.yosys,
            design.iverilog_compile.as_ref(),
            design.sv2v.as_ref(),
        );
    }
    summary
}

/// `SV-VERSION-TARGETING.2b.2b` — record that a `sv_version`-targeted
/// artifact was accepted in the matching tool standard mode. Lights the
/// per-version sub-fact plus the umbrella fact.
fn light_sv_version_acceptance(coverage: &mut CoverageSummary, sv_version: SvVersion) {
    coverage.saw_sv_version_targeted_acceptance = true;
    match sv_version {
        SvVersion::Sv2012 => coverage.saw_sv_version_2012_targeted_acceptance = true,
        SvVersion::Sv2017 => coverage.saw_sv_version_2017_targeted_acceptance = true,
        SvVersion::Sv2023 => coverage.saw_sv_version_2023_targeted_acceptance = true,
    }
}

fn summarize_coverage(
    scenario: &Scenario,
    modules: &[ModuleReport],
    version_targeted: bool,
) -> CoverageSummary {
    let mut coverage = CoverageSummary::default();
    seed_scenario_coverage(&mut coverage, scenario);

    for module in modules {
        accumulate_module_coverage(&mut coverage, &module.metrics);
        // `DIFFERENTIAL-SIMULATION.3b.2` — the cross-simulator
        // agreement fact fires when at least one DUT actually ran
        // the diff-sim gate AND its traces matched byte-for-byte.
        // Modules outside the `--diff-sim` subset have
        // `diff_sim = None` and contribute nothing.
        if let Some(diff) = &module.diff_sim {
            if diff.ran && diff.success {
                coverage.saw_design_with_cross_simulator_agreement = true;
            }
        }
        // `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — the opportunistic
        // acceptance-divergence fact fires when a unit in the `--divergence`
        // subset had two enabled tools disagree. Never a required gate
        // (all-agree is the valid-by-construction steady state, decision
        // `0019`); units outside the subset have `divergence = None` and
        // contribute nothing.
        if let Some(div) = &module.divergence {
            if div.diverged {
                coverage.saw_acceptance_divergence = true;
            }
        }
        // `MULTI-CLOCK-CDC.3b.2` / `SIGNOFF-SURFACE-EXPANSION.1`
        // — multi-clock facts surface via the per-module Metrics
        // fields populated by `anvil::metrics::compute`.
        // Module-level only — the design-level path uses
        // `summarize_design_coverage`.
        if module.metrics.num_clock_domains >= 2 {
            coverage.saw_multi_clock_design = true;
        }
        if module.metrics.num_cdc_2_flop_synchronizers >= 1 {
            coverage.saw_cdc_2_flop_synchronizer = true;
        }
        if module.metrics.max_cdc_synchronizer_stages >= 3 {
            coverage.saw_cdc_nflop_synchronizer = true;
        }
        // `SV-VERSION-TARGETING.2b.2b` — only the per-version gate runs
        // Verilator in the matching `--language` mode, so the acceptance
        // fact is honest only when `version_targeted`. Require Verilator
        // to have actually run and succeeded plus Yosys to have actually
        // run and stayed clean — `!yosys.is_empty()` guards against a
        // Verilator-only `union soft` up-opt module (empty Yosys vec)
        // vacuously lighting this Yosys-requiring fact.
        if version_targeted
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && !module.yosys.is_empty()
            && all_yosys_invocations_ok(&module.yosys)
        {
            light_sv_version_acceptance(&mut coverage, scenario.config.sv_version);
        }
        // `SV-VERSION-TARGETING.3b.2b` — the 2023 `union soft` up-opt fact:
        // a genuinely-emitted overlay (proven from the SV text) accepted by
        // Verilator `--language 1800-2023`. Yosys/Icarus reject the syntax
        // and are a recorded no-op (decision `0010`), so this fact requires
        // only matching-mode Verilator acceptance — never Yosys.
        if version_targeted
            && scenario.config.sv_version == SvVersion::Sv2023
            && module.emitted_soft_union_overlay
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
        {
            coverage.saw_sv_version_2023_soft_union_upopt = true;
        }
        // `STRUCTURED-EMISSION-EXPANSION.2b.2b` — the combinational
        // `function automatic` emit fact: a genuinely-emitted function
        // (proven from the SV text) accepted by the downstream tools.
        // Unlike the `union soft` overlay, a synthesizable function is
        // accepted by every tool, so this fact requires Verilator success
        // **and** Yosys clean (`!yosys.is_empty()` guards the vacuous
        // empty-vec case). Icarus acceptance, when `--iverilog-compile` is
        // set, is enforced separately via the tool-summary `any_failed`
        // bail.
        if module.emitted_combinational_function
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && !module.yosys.is_empty()
            && all_yosys_invocations_ok(&module.yosys)
        {
            coverage.saw_combinational_function_emit = true;
        }

        // `STRUCTURED-EMISSION-EXPANSION.4b.2b` — a genuinely-emitted
        // `generate for` loop (proven from the SV text) accepted by the
        // downstream tools. Like a function (and unlike the `union soft`
        // overlay), a `generate for` is universally synthesizable, so this
        // fact requires Verilator success **and** Yosys clean
        // (`!yosys.is_empty()` guards the vacuous empty-vec case). Icarus
        // acceptance, when `--iverilog-compile` is set, is enforced separately
        // via the tool-summary `any_failed` bail.
        if module.emitted_generate_loop
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && !module.yosys.is_empty()
            && all_yosys_invocations_ok(&module.yosys)
        {
            coverage.saw_generate_loop_emit = true;
        }

        // `STRUCTURED-EMISSION-EXPANSION.6b.2b` — a genuinely-emitted
        // combinational `task automatic` (proven from the SV text) accepted by
        // the downstream tools. Like a function (and unlike the `union soft`
        // overlay), a combinational `task` is universally synthesizable, so this
        // fact requires Verilator success **and** Yosys clean
        // (`!yosys.is_empty()` guards the vacuous empty-vec case). Icarus
        // acceptance, when `--iverilog-compile` is set, is enforced separately
        // via the tool-summary `any_failed` bail.
        if module.emitted_combinational_task
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && !module.yosys.is_empty()
            && all_yosys_invocations_ok(&module.yosys)
        {
            coverage.saw_combinational_task_emit = true;
        }

        // `STRUCTURED-EMISSION-EXPANSION.10b.2` — a genuinely-emitted
        // multi-gate-cone `function automatic` (proven from the SV text)
        // accepted by the downstream tools. Like a single-gate function (and
        // unlike the `union soft` overlay), a cone function is universally
        // synthesizable, so this fact requires Verilator success **and** Yosys
        // clean (`!yosys.is_empty()` guards the vacuous empty-vec case). Icarus
        // acceptance, when `--iverilog-compile` is set, is enforced separately
        // via the tool-summary `any_failed` bail.
        if module.emitted_cone_function
            && module
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && !module.yosys.is_empty()
            && all_yosys_invocations_ok(&module.yosys)
        {
            coverage.saw_cone_function_emit = true;
        }
    }

    coverage
}

fn summarize_design_coverage(
    scenario: &Scenario,
    designs: &[DesignReport],
    version_targeted: bool,
) -> CoverageSummary {
    let mut coverage = CoverageSummary::default();
    seed_scenario_coverage(&mut coverage, scenario);

    for design in designs {
        coverage.saw_hierarchy_design = true;
        // `ACCEPTANCE-DIVERGENCE-HUNTING.2c.2` — opportunistic
        // acceptance-divergence fact (design-level; see `summarize_coverage`).
        // Never a required gate (decision `0019`).
        if let Some(div) = &design.divergence {
            if div.diverged {
                coverage.saw_acceptance_divergence = true;
            }
        }
        // `SV-VERSION-TARGETING.2b.2b` — version-targeted design accepted
        // in the matching tool standard mode (see `summarize_coverage`).
        if version_targeted
            && design
                .verilator
                .as_ref()
                .map(|t| t.success)
                .unwrap_or(false)
            && all_yosys_invocations_ok(&design.yosys)
        {
            light_sv_version_acceptance(&mut coverage, scenario.config.sv_version);
        }
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
        coverage.saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs |=
            design.metrics.realized_max_leaf_depth >= 7
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
            .saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs |=
            design.metrics.realized_max_leaf_depth >= 7
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
        coverage.saw_recursive_hierarchy_three_stage_registered_parent_composed_chain |=
            design.metrics.realized_max_leaf_depth > 1
                && design
                    .metrics
                    .child_input_bindings_from_registered_three_stage_parent_composed_logic
                    > design
                        .metrics
                        .top_child_input_bindings_from_registered_three_stage_parent_composed_logic
                && design.metrics.hierarchy_parent_cone_instances == 0;
        coverage.saw_recursive_parent_cone_helper_budget_5 |=
            design.metrics.realized_max_leaf_depth > 1
                && scenario.config.hierarchy_parent_cone_instance_prob > 0.0
                && scenario.config.max_parent_cone_instances_per_module >= 5
                && design.metrics.max_parent_cone_instances_per_internal_module >= 5
                && design.metrics.hierarchy_parent_cone_instances
                    > design.metrics.top_parent_cone_instances;
        coverage.saw_recursive_hierarchy_canonical_module_signature_diversity |=
            design.metrics.realized_max_leaf_depth > 1
                && design.metrics.canonical_module_signatures.len() == design.metrics.num_modules
                && design
                    .metrics
                    .canonical_module_signatures
                    .iter()
                    .all(|sig| *sig != 0)
                && design.metrics.num_distinct_module_signatures >= 2;
        coverage.saw_design_with_structurally_duplicate_modules |=
            design.metrics.num_structurally_duplicate_module_pairs > 0
                && design.metrics.num_distinct_module_signatures < design.metrics.num_modules;
        coverage.saw_recursive_hierarchy_module_dedup_active |=
            scenario.config.hierarchy_module_dedup
                && design.metrics.num_modules >= 2
                && design.metrics.num_structurally_duplicate_module_pairs == 0
                && design.metrics.num_distinct_module_signatures == design.metrics.num_modules;
        coverage.saw_width_parameterized_design |= scenario.config.width_parameterization_prob
            > 0.0
            && design.metrics.num_width_parameterized_modules > 0
            && design.metrics.num_param_override_instances > 0;
        coverage.saw_packed_aggregate_design |=
            scenario.config.aggregate_prob > 0.0 && design.metrics.num_packed_aggregate_modules > 0;
        coverage.saw_inferrable_memory_design |=
            scenario.config.memory_prob > 0.0 && design.metrics.num_memory_modules > 0;
        coverage.saw_fsm_design |=
            scenario.config.fsm_prob > 0.0 && design.metrics.num_fsm_modules > 0;
        coverage.saw_array_packed_aggregate_design |= scenario.config.aggregate_array_prob > 0.0
            && design.metrics.num_array_packed_aggregate_modules > 0;
        coverage.saw_memory_fsm_interplay_design |= scenario.config.memory_prob > 0.0
            && scenario.config.fsm_prob > 0.0
            && design.metrics.num_memory_modules > 0
            && design.metrics.num_fsm_modules > 0;
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
    dst.iverilog_compile_passed += src.iverilog_compile_passed;
    dst.iverilog_compile_failed += src.iverilog_compile_failed;
    dst.sv2v_passed += src.sv2v_passed;
    dst.sv2v_failed += src.sv2v_failed;
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
    dst.saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs |=
        src.saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs;
    dst.saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs |=
        src.saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs;
    dst.saw_recursive_hierarchy_three_stage_registered_parent_composed_chain |=
        src.saw_recursive_hierarchy_three_stage_registered_parent_composed_chain;
    dst.saw_recursive_parent_cone_helper_budget_5 |= src.saw_recursive_parent_cone_helper_budget_5;
    dst.saw_recursive_hierarchy_canonical_module_signature_diversity |=
        src.saw_recursive_hierarchy_canonical_module_signature_diversity;
    dst.saw_design_with_structurally_duplicate_modules |=
        src.saw_design_with_structurally_duplicate_modules;
    dst.saw_recursive_hierarchy_module_dedup_active |=
        src.saw_recursive_hierarchy_module_dedup_active;
    dst.saw_width_parameterized_design |= src.saw_width_parameterized_design;
    dst.saw_packed_aggregate_design |= src.saw_packed_aggregate_design;
    dst.saw_inferrable_memory_design |= src.saw_inferrable_memory_design;
    dst.saw_fsm_design |= src.saw_fsm_design;
    dst.saw_operand_duplication |= src.saw_operand_duplication;
    dst.saw_mux_arm_duplication |= src.saw_mux_arm_duplication;
    dst.saw_array_packed_aggregate_design |= src.saw_array_packed_aggregate_design;
    dst.saw_memory_fsm_interplay_design |= src.saw_memory_fsm_interplay_design;
    dst.saw_sv_version_targeted_acceptance |= src.saw_sv_version_targeted_acceptance;
    dst.saw_sv_version_2012_targeted_acceptance |= src.saw_sv_version_2012_targeted_acceptance;
    dst.saw_sv_version_2017_targeted_acceptance |= src.saw_sv_version_2017_targeted_acceptance;
    dst.saw_sv_version_2023_targeted_acceptance |= src.saw_sv_version_2023_targeted_acceptance;
    dst.saw_sv_version_2023_soft_union_upopt |= src.saw_sv_version_2023_soft_union_upopt;
    dst.saw_combinational_function_emit |= src.saw_combinational_function_emit;
    dst.saw_generate_loop_emit |= src.saw_generate_loop_emit;
    dst.saw_combinational_task_emit |= src.saw_combinational_task_emit;
    dst.saw_cone_function_emit |= src.saw_cone_function_emit;
    dst.saw_design_with_cross_simulator_agreement |= src.saw_design_with_cross_simulator_agreement;
    dst.saw_acceptance_divergence |= src.saw_acceptance_divergence;
    dst.saw_multi_clock_design |= src.saw_multi_clock_design;
    dst.saw_cdc_2_flop_synchronizer |= src.saw_cdc_2_flop_synchronizer;
    dst.saw_cdc_nflop_synchronizer |= src.saw_cdc_nflop_synchronizer;
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
    aggregate.total_fsms_merged += u64::from(metrics.fsms_merged);

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
    iverilog_compile: Option<&ToolInvocation>,
    sv2v: Option<&ToolInvocation>,
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
    if let Some(iverilog_compile) = iverilog_compile {
        if iverilog_compile.success {
            summary.iverilog_compile_passed += 1;
        } else {
            summary.iverilog_compile_failed += 1;
        }
    }
    if let Some(sv2v) = sv2v {
        if sv2v.success {
            summary.sv2v_passed += 1;
        } else {
            summary.sv2v_failed += 1;
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
    coverage.saw_operand_duplication |= metrics.num_operator_gates_with_duplicate_operands > 0;
    coverage.saw_mux_arm_duplication |= metrics.num_muxes_degenerate > 0;
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

    // `SV-VERSION-TARGETING.2b.2b` — the per-version acceptance gate's
    // sole contract is to prove each targeted IEEE 1800 standard's corpus
    // is accepted in the matching tool standard mode (Verilator
    // `--language 1800-20xx` + Yosys `-sv`). Construction-strategy / motif
    // / category breadth is intentionally out of scope (the other gates
    // own it), so return before the strategy loop and check exactly the
    // per-version acceptance facts.
    if scenario_set == ScenarioSet::SvVersionSweep {
        for (lit, std) in [
            (
                coverage.saw_sv_version_2012_targeted_acceptance,
                "1800-2012",
            ),
            (
                coverage.saw_sv_version_2017_targeted_acceptance,
                "1800-2017",
            ),
            (
                coverage.saw_sv_version_2023_targeted_acceptance,
                "1800-2023",
            ),
        ] {
            if !lit {
                gaps.push(format!(
                    "matrix never proved sv_version {std} targeted acceptance (Verilator --language {std} + Yosys -sv clean on a {std}-targeted artifact)"
                ));
            }
        }
        if !coverage.saw_sv_version_targeted_acceptance {
            gaps.push("matrix never proved any sv_version targeted acceptance".to_string());
        }
        // `SV-VERSION-TARGETING.3b.2b` — the up-opt fact. Requires a
        // genuinely-emitted IEEE 1800-2023 `union soft` overlay accepted by
        // Verilator `--language 1800-2023` (Yosys/Icarus recorded no-op).
        if !coverage.saw_sv_version_2023_soft_union_upopt {
            gaps.push(
                "matrix never proved the 2023 `union soft` up-opt (Verilator --language 1800-2023 accepted a generated `union soft` overlay; Yosys/Icarus recorded no-op)".to_string(),
            );
        }
        return gaps;
    }

    for strategy in ["sequential", "shuffled", "interleaved"] {
        if !coverage.construction_strategies.contains(strategy) {
            gaps.push(format!("missing construction strategy {strategy}"));
        }
    }

    // `SIGNOFF-AUTOMATION-EXPANSION.2b` — the focused richer-knob-sweep
    // gate's sole contract is to prove the four promoted unswept knobs
    // fire by construction. The broad motif/identity/category richness
    // the other sets enforce below is intentionally out of scope here,
    // so check exactly the four facts (plus the universal
    // construction-strategy coverage above) and return.
    if scenario_set == ScenarioSet::SignoffKnobSweep {
        if !coverage.saw_operand_duplication {
            gaps.push(
                "matrix never proved operand_duplication_rate (an Add/Mul gate with a duplicated operand)".to_string(),
            );
        }
        if !coverage.saw_mux_arm_duplication {
            gaps.push(
                "matrix never proved mux_arm_duplication_rate (a degenerate 2-to-1 mux with equal arms)".to_string(),
            );
        }
        if !coverage.saw_array_packed_aggregate_design {
            gaps.push(
                "matrix never proved aggregate_array_prob (an array-packed aggregate module)"
                    .to_string(),
            );
        }
        if !coverage.saw_memory_fsm_interplay_design {
            gaps.push(
                "matrix never proved memory×fsm interplay (a memory module and an FSM module in one design)".to_string(),
            );
        }
        return gaps;
    }

    // `STRUCTURED-EMISSION-EXPANSION.2b.2b` — the combinational `function
    // automatic` emit gate's sole contract is to prove the first
    // richer-structured emission surface fires by construction and is
    // downstream-accepted. The broad motif/identity/category richness the
    // other sets enforce below is intentionally out of scope (this is a
    // focused capability gate), so check exactly the one fact (plus the
    // universal construction-strategy coverage above) and return.
    if scenario_set == ScenarioSet::FunctionEmitSweep {
        if !coverage.saw_combinational_function_emit {
            gaps.push(
                "matrix never proved function_emit_prob (a combinational `function automatic` emit-projection accepted by Verilator + Yosys)".to_string(),
            );
        }
        return gaps;
    }

    // `STRUCTURED-EMISSION-EXPANSION.4b.2b` — the `generate for` loop emit
    // gate's sole contract is to prove the second richer-structured emission
    // surface fires by construction and is downstream-accepted. Like the
    // function-emit gate above, the broad motif/identity/category richness
    // the other sets enforce below is intentionally out of scope, so check
    // exactly the one fact (plus the universal construction-strategy coverage
    // above) and return.
    if scenario_set == ScenarioSet::GenerateLoopSweep {
        if !coverage.saw_generate_loop_emit {
            gaps.push(
                "matrix never proved generate_loop_emit_prob (a `generate for` loop emit-projection accepted by Verilator + Yosys)".to_string(),
            );
        }
        return gaps;
    }

    // `STRUCTURED-EMISSION-EXPANSION.6b.2b` — the combinational `task automatic`
    // emit gate's sole contract is to prove the third richer-structured emission
    // surface fires by construction and is downstream-accepted. Like the
    // function-emit / generate-loop gates above, the broad motif/identity/category
    // richness the other sets enforce below is intentionally out of scope, so
    // check exactly the one fact (plus the universal construction-strategy
    // coverage above) and return.
    if scenario_set == ScenarioSet::TaskEmitSweep {
        if !coverage.saw_combinational_task_emit {
            gaps.push(
                "matrix never proved task_emit_prob (a combinational `task automatic` emit-projection accepted by Verilator + Yosys)".to_string(),
            );
        }
        return gaps;
    }

    // `STRUCTURED-EMISSION-EXPANSION.10b.2` — the multi-gate-cone `function
    // automatic` emit gate's sole contract is to prove the fifth
    // richer-structured emission surface fires by construction and is
    // downstream-accepted. Like the function-emit / generate-loop / task-emit
    // gates above, the broad motif/identity/category richness the other sets
    // enforce below is intentionally out of scope, so check exactly the one fact
    // (plus the universal construction-strategy coverage above) and return.
    if scenario_set == ScenarioSet::ConeFunctionSweep {
        if !coverage.saw_cone_function_emit {
            gaps.push(
                "matrix never proved cone_function_emit_prob (a multi-gate-cone `function automatic` emit-projection accepted by Verilator + Yosys)".to_string(),
            );
        }
        return gaps;
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
        // Unreachable: the focused knob-sweep, sv-version, function-emit,
        // generate-loop, task-emit, and cone-function gates return above.
        ScenarioSet::SignoffKnobSweep
        | ScenarioSet::SvVersionSweep
        | ScenarioSet::FunctionEmitSweep
        | ScenarioSet::GenerateLoopSweep
        | ScenarioSet::TaskEmitSweep
        | ScenarioSet::ConeFunctionSweep => {}
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
        // Unreachable: the focused knob-sweep, sv-version, function-emit,
        // generate-loop, task-emit, and cone-function gates return above.
        ScenarioSet::SignoffKnobSweep
        | ScenarioSet::SvVersionSweep
        | ScenarioSet::FunctionEmitSweep
        | ScenarioSet::GenerateLoopSweep
        | ScenarioSet::TaskEmitSweep
        | ScenarioSet::ConeFunctionSweep => &[],
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
    if scenario_set == ScenarioSet::Default && !coverage.saw_multi_clock_design {
        gaps.push("matrix never emitted an opt-in multi-clock module".to_string());
    }
    if scenario_set == ScenarioSet::Default && !coverage.saw_cdc_2_flop_synchronizer {
        gaps.push("matrix never emitted an exact 2-flop CDC synchronizer".to_string());
    }
    if scenario_set == ScenarioSet::Default && !coverage.saw_cdc_nflop_synchronizer {
        gaps.push("matrix never emitted an N-flop CDC synchronizer".to_string());
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
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs
    {
        gaps.push(
            "matrix never proved recursive depth-7 hierarchy parent outputs mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage
            .saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs
    {
        gaps.push(
            "matrix never proved recursive depth-7 hierarchy unregistered parent-composed child-input bindings mixing parent ports, child outputs, and parent-local Qs below the top parent without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_three_stage_registered_parent_composed_chain
    {
        gaps.push(
            "matrix never proved recursive non-top registered parent-composed child-input bindings chaining through at least three parent-local flop stages without helper instances"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_parent_cone_helper_budget_5
    {
        gaps.push(
            "matrix never proved a recursive non-top internal parent saturating a parent-cone helper budget of 5 helpers"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_canonical_module_signature_diversity
    {
        gaps.push(
            "matrix never proved a recursive hierarchy design with at least two distinct canonical module signatures (first slice of hierarchy-aware identity instrumentation)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_design_with_structurally_duplicate_modules
    {
        gaps.push(
            "matrix never proved a design where the planner emitted structurally-duplicate Module definitions (HIERARCHY-AWARE-IDENTITY.2 — the future dedup pass needs at least one live example to exercise)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy
        && !coverage.saw_recursive_hierarchy_module_dedup_active
    {
        gaps.push(
            "matrix never proved a design where the module-dedup pass ran and produced a duplicate-free survivor set (HIERARCHY-AWARE-IDENTITY.4)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_width_parameterized_design {
        gaps.push(
            "matrix never proved a downstream-clean design with a width-parameterized module instantiated via #(.W(v)) (PHASE-5-PARAMETERIZATION.2.4)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_packed_aggregate_design {
        gaps.push(
            "matrix never proved a downstream-clean design with a packed-aggregate emitter projection (PHASE-5B-AGGREGATES.2.3)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_inferrable_memory_design {
        gaps.push(
            "matrix never proved a downstream-clean design with an inferrable memory (PHASE-6-ADVANCED-MOTIFS.2.3)"
                .to_string(),
        );
    }
    if scenario_set == ScenarioSet::Phase4Hierarchy && !coverage.saw_fsm_design {
        gaps.push(
            "matrix never proved a downstream-clean design with a generated-encoding FSM (PHASE-6-ADVANCED-MOTIFS.3.4)"
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
        // Unreachable: the focused knob-sweep, sv-version, function-emit,
        // generate-loop, task-emit, and cone-function gates return above.
        ScenarioSet::SignoffKnobSweep
        | ScenarioSet::SvVersionSweep
        | ScenarioSet::FunctionEmitSweep
        | ScenarioSet::GenerateLoopSweep
        | ScenarioSet::TaskEmitSweep
        | ScenarioSet::ConeFunctionSweep => &[],
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
        ScenarioSet::SignoffKnobSweep => "signoff-knob-sweep",
        ScenarioSet::SvVersionSweep => "sv-version-sweep",
        ScenarioSet::FunctionEmitSweep => "function-emit-sweep",
        ScenarioSet::GenerateLoopSweep => "generate-loop-sweep",
        ScenarioSet::TaskEmitSweep => "task-emit-sweep",
        ScenarioSet::ConeFunctionSweep => "cone-function-sweep",
    }
}

fn artifact_kind_slug(scenario_set: ScenarioSet) -> &'static str {
    match scenario_set {
        ScenarioSet::Phase4Hierarchy => "design",
        // The knob-sweep, sv-version, function-emit, generate-loop, task-emit,
        // and cone-function sets are mixed or single-module DUTs, like the
        // Default set; report the coarse "module" label and let each
        // per-scenario report carry its own module/design routing.
        ScenarioSet::Default
        | ScenarioSet::Phase2Share
        | ScenarioSet::Phase3Structured
        | ScenarioSet::SignoffKnobSweep
        | ScenarioSet::SvVersionSweep
        | ScenarioSet::FunctionEmitSweep
        | ScenarioSet::GenerateLoopSweep
        | ScenarioSet::TaskEmitSweep
        | ScenarioSet::ConeFunctionSweep => "module",
    }
}

impl ToolSummary {
    fn yosys_failed(&self) -> usize {
        self.yosys_without_abc_failed + self.yosys_with_abc_failed
    }

    fn iverilog_failed(&self) -> usize {
        self.iverilog_compile_failed
    }

    fn any_failed(&self) -> bool {
        self.verilator_failed > 0
            || self.yosys_failed() > 0
            || self.iverilog_failed() > 0
            || self.sv2v_failed > 0
    }
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
            signoff_knob_sweep_gate: false,
            sv_version_gate: false,
            function_emit_gate: false,
            generate_loop_gate: false,
            task_emit_gate: false,
            cone_function_gate: false,
            list_scenarios: false,
            skip_verilator: false,
            skip_yosys: false,
            verilator_bin: "verilator".to_string(),
            yosys_bin: "yosys".to_string(),
            iverilog_compile: false,
            iverilog_bin: "iverilog".to_string(),
            sv2v: false,
            sv2v_bin: "sv2v".to_string(),
            yosys_mode: YosysMode::WithoutAbc,
            fail_on_coverage_gap: false,
            resume: false,
            diff_sim: false,
            divergence: false,
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
        assert!(gaps.iter().any(|gap| gap.contains("multi-clock module")));
        assert!(gaps.iter().any(|gap| gap.contains("2-flop CDC")));
        assert!(gaps.iter().any(|gap| gap.contains("N-flop CDC")));
    }

    #[test]
    fn phase1_gate_raises_modules_per_scenario_to_cover_1000_modules() {
        let mut cli = test_cli();
        cli.phase1_gate = true;
        let scenario_count = build_scenarios(0, ScenarioSet::Default)
            .expect("build scenarios")
            .len();

        let plan = derive_run_plan(&cli, scenario_count);
        assert_eq!(
            plan.modules_per_scenario,
            PHASE1_MIN_TOTAL_MODULES.div_ceil(scenario_count)
        );
        assert!(plan.total_modules >= PHASE1_MIN_TOTAL_MODULES);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn phase1_gate_preserves_larger_explicit_module_count() {
        let mut cli = test_cli();
        cli.phase1_gate = true;
        cli.modules_per_scenario = 100;
        let scenario_count = build_scenarios(0, ScenarioSet::Default)
            .expect("build scenarios")
            .len();

        let plan = derive_run_plan(&cli, scenario_count);
        assert_eq!(plan.modules_per_scenario, 100);
        assert_eq!(plan.total_modules, scenario_count * 100);
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
        assert_eq!(plan.total_modules, 888);
        assert!(plan.fail_on_coverage_gap);
    }

    // ===============================================================
    // SIGNOFF-AUTOMATION-EXPANSION.2b — cargo-portable proofs of the
    // focused richer-knob-sweep gate wiring (scenario set, CLI flag,
    // run-plan, per-knob config shaping, gap requirements). The
    // downstream-clean bank is the repo-owned report, run separately
    // with real tools.
    // ===============================================================

    #[test]
    fn signoff_knob_sweep_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.signoff_knob_sweep_gate);
        let with_flag = Cli::try_parse_from([
            "tool_matrix",
            "--signoff-knob-sweep-gate",
            "--out",
            "/tmp/x",
        ])
        .expect("parse");
        assert!(with_flag.signoff_knob_sweep_gate);
    }

    #[test]
    fn signoff_knob_sweep_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.signoff_knob_sweep_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::SignoffKnobSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::SignoffKnobSweep).expect("build");
        // four knobs x three construction strategies.
        assert_eq!(scenarios.len(), 12);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(plan.modules_per_scenario, 4);
        assert_eq!(plan.total_modules, 48);
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn signoff_knob_sweep_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.signoff_knob_sweep_gate = true;
        cli.phase4_hierarchy_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn signoff_knob_sweep_scenarios_force_each_unswept_knob() {
        let scenarios = build_scenarios(0, ScenarioSet::SignoffKnobSweep).expect("build");
        let mut strategies = BTreeSet::new();
        let (mut operand, mut mux, mut array_agg, mut mem_fsm) = (0, 0, 0, 0);
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            let cfg = &scenario.config;
            if scenario.name.ends_with("signoff_operand_duplication") {
                operand += 1;
                assert_eq!(cfg.operand_duplication_rate, 1.0);
                assert!(cfg.gate_arith_weight > 0);
                // single-module DUT, not a hierarchy design.
                assert!(cfg.effective_hierarchy_depth_range().is_none());
            } else if scenario.name.ends_with("signoff_mux_arm_duplication") {
                mux += 1;
                assert_eq!(cfg.mux_arm_duplication_rate, 1.0);
                assert_eq!(cfg.min_mux_arms, 2);
                assert_eq!(cfg.max_mux_arms, 2);
                assert!(cfg.effective_hierarchy_depth_range().is_none());
            } else if scenario.name.ends_with("signoff_array_packed_aggregate") {
                array_agg += 1;
                assert_eq!(cfg.aggregate_prob, 1.0);
                assert_eq!(cfg.aggregate_array_prob, 1.0);
                // uniform width is load-bearing for the ArrayPacked projection.
                assert_eq!(cfg.min_width, cfg.max_width);
                assert!(cfg.effective_hierarchy_depth_range().is_some());
            } else if scenario.name.ends_with("signoff_memory_fsm_interplay") {
                mem_fsm += 1;
                // memory_prob strictly in (0,1) + fsm_prob = 1.0 so both
                // leaf kinds coexist despite mutually-exclusive selection.
                assert!(cfg.memory_prob > 0.0 && cfg.memory_prob < 1.0);
                assert_eq!(cfg.fsm_prob, 1.0);
                assert!(cfg.effective_hierarchy_depth_range().is_some());
            } else {
                panic!("unexpected signoff knob-sweep scenario {}", scenario.name);
            }
        }
        assert_eq!((operand, mux, array_agg, mem_fsm), (3, 3, 3, 3));
        assert_eq!(
            strategies,
            BTreeSet::from(["sequential", "shuffled", "interleaved"])
        );
    }

    #[test]
    fn signoff_knob_sweep_gaps_require_exactly_the_four_facts() {
        // All three strategies present, but no fact lit → exactly the
        // four knob-sweep gaps (no broad-motif gaps leak in).
        let mut coverage = CoverageSummary::default();
        for s in ["sequential", "shuffled", "interleaved"] {
            coverage.construction_strategies.insert(s.to_string());
        }
        let gaps = compute_coverage_gaps(ScenarioSet::SignoffKnobSweep, &coverage, None);
        assert_eq!(gaps.len(), 4, "unexpected gaps: {gaps:?}");
        assert!(gaps.iter().any(|g| g.contains("operand_duplication_rate")));
        assert!(gaps.iter().any(|g| g.contains("mux_arm_duplication_rate")));
        assert!(gaps.iter().any(|g| g.contains("aggregate_array_prob")));
        assert!(gaps.iter().any(|g| g.contains("memory×fsm interplay")));

        // All four facts lit → no gaps.
        coverage.saw_operand_duplication = true;
        coverage.saw_mux_arm_duplication = true;
        coverage.saw_array_packed_aggregate_design = true;
        coverage.saw_memory_fsm_interplay_design = true;
        let gaps = compute_coverage_gaps(ScenarioSet::SignoffKnobSweep, &coverage, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    // ===============================================================
    // STRUCTURED-EMISSION-EXPANSION.2b.2b — cargo-portable proofs of the
    // repo-owned combinational `function automatic` emit gate wiring (CLI
    // flag, scenario set, run-plan, per-strategy config shaping, gap
    // requirement). The downstream-clean bank is the repo-owned report,
    // run separately with real Verilator + both Yosys modes + Icarus.
    // ===============================================================

    #[test]
    fn function_emit_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.function_emit_gate);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--function-emit-gate", "--out", "/tmp/x"])
                .expect("parse");
        assert!(with_flag.function_emit_gate);
    }

    #[test]
    fn function_emit_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.function_emit_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::FunctionEmitSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::FunctionEmitSweep).expect("build");
        // one comb-only focus config x three construction strategies.
        assert_eq!(scenarios.len(), 3);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(
            plan.modules_per_scenario,
            FUNCTION_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert_eq!(
            plan.total_modules,
            3 * FUNCTION_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn function_emit_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.function_emit_gate = true;
        cli.sv_version_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn function_emit_sweep_scenarios_force_the_knob() {
        let scenarios = build_scenarios(0, ScenarioSet::FunctionEmitSweep).expect("build");
        let mut strategies = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            let cfg = &scenario.config;
            assert!(
                scenario.name.ends_with("function_emit"),
                "unexpected function-emit scenario {}",
                scenario.name
            );
            // function_emit_prob forced to 1.0 so every qualifying gate is
            // projected to a `function automatic`.
            assert_eq!(cfg.function_emit_prob, 1.0);
            // Comb-only single-module DUT (no flops, no hierarchy): the
            // first-cut surface projects combinational gates only.
            assert_eq!(cfg.flop_prob, 0.0);
            assert!(cfg.effective_hierarchy_depth_range().is_none());
        }
        assert_eq!(scenarios.len(), 3);
        assert_eq!(
            strategies,
            BTreeSet::from(["sequential", "shuffled", "interleaved"])
        );
    }

    #[test]
    fn function_emit_sweep_gaps_require_the_fact() {
        // All three strategies present, but the fact not lit → exactly the
        // one function-emit gap (no broad-motif gaps leak in).
        let mut coverage = CoverageSummary::default();
        for s in ["sequential", "shuffled", "interleaved"] {
            coverage.construction_strategies.insert(s.to_string());
        }
        let gaps = compute_coverage_gaps(ScenarioSet::FunctionEmitSweep, &coverage, None);
        assert_eq!(gaps.len(), 1, "unexpected gaps: {gaps:?}");
        assert!(gaps[0].contains("function_emit_prob"));

        // Fact lit → no gaps.
        coverage.saw_combinational_function_emit = true;
        let gaps = compute_coverage_gaps(ScenarioSet::FunctionEmitSweep, &coverage, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    #[test]
    fn generate_loop_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.generate_loop_gate);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--generate-loop-gate", "--out", "/tmp/x"])
                .expect("parse");
        assert!(with_flag.generate_loop_gate);
    }

    #[test]
    fn generate_loop_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.generate_loop_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::GenerateLoopSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::GenerateLoopSweep).expect("build");
        // one comb-only focus config x three construction strategies.
        assert_eq!(scenarios.len(), 3);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(
            plan.modules_per_scenario,
            GENERATE_LOOP_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert_eq!(
            plan.total_modules,
            3 * GENERATE_LOOP_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn generate_loop_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.generate_loop_gate = true;
        cli.function_emit_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn generate_loop_sweep_scenarios_force_the_knob() {
        let scenarios = build_scenarios(0, ScenarioSet::GenerateLoopSweep).expect("build");
        let mut strategies = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            let cfg = &scenario.config;
            assert!(
                scenario.name.ends_with("generate_loop"),
                "unexpected generate-loop scenario {}",
                scenario.name
            );
            // generate_loop_emit_prob forced to 1.0 so every qualifying
            // {N{x}} replication is projected to a `generate for` loop.
            assert_eq!(cfg.generate_loop_emit_prob, 1.0);
            // Comb-only single-module DUT (no flops, no hierarchy): the
            // first-cut surface projects combinational replications only.
            assert_eq!(cfg.flop_prob, 0.0);
            assert!(cfg.effective_hierarchy_depth_range().is_none());
        }
        assert_eq!(scenarios.len(), 3);
        assert_eq!(
            strategies,
            BTreeSet::from(["sequential", "shuffled", "interleaved"])
        );
    }

    #[test]
    fn generate_loop_sweep_gaps_require_the_fact() {
        // All three strategies present, but the fact not lit → exactly the
        // one generate-loop gap (no broad-motif gaps leak in).
        let mut coverage = CoverageSummary::default();
        for s in ["sequential", "shuffled", "interleaved"] {
            coverage.construction_strategies.insert(s.to_string());
        }
        let gaps = compute_coverage_gaps(ScenarioSet::GenerateLoopSweep, &coverage, None);
        assert_eq!(gaps.len(), 1, "unexpected gaps: {gaps:?}");
        assert!(gaps[0].contains("generate_loop_emit_prob"));

        // Fact lit → no gaps.
        coverage.saw_generate_loop_emit = true;
        let gaps = compute_coverage_gaps(ScenarioSet::GenerateLoopSweep, &coverage, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    // ===============================================================
    // STRUCTURED-EMISSION-EXPANSION.6b.2b — cargo-portable proofs of the
    // repo-owned combinational `task automatic` emit gate wiring (CLI flag,
    // scenario set, run-plan, knob forcing, gap requirement). The
    // downstream-clean bank is the repo-owned report, run separately with
    // real Verilator + Yosys + Icarus.
    // ===============================================================

    #[test]
    fn task_emit_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.task_emit_gate);
        let with_flag = Cli::try_parse_from(["tool_matrix", "--task-emit-gate", "--out", "/tmp/x"])
            .expect("parse");
        assert!(with_flag.task_emit_gate);
    }

    #[test]
    fn task_emit_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.task_emit_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::TaskEmitSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::TaskEmitSweep).expect("build");
        // one comb-only focus config x three construction strategies.
        assert_eq!(scenarios.len(), 3);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(
            plan.modules_per_scenario,
            TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert_eq!(
            plan.total_modules,
            3 * TASK_EMIT_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn task_emit_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.task_emit_gate = true;
        cli.generate_loop_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn task_emit_sweep_scenarios_force_the_knob() {
        let scenarios = build_scenarios(0, ScenarioSet::TaskEmitSweep).expect("build");
        let mut strategies = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            let cfg = &scenario.config;
            assert!(
                scenario.name.ends_with("task_emit"),
                "unexpected task-emit scenario {}",
                scenario.name
            );
            // task_emit_prob forced to 1.0 so every qualifying combinational
            // gate is projected to a `task automatic`.
            assert_eq!(cfg.task_emit_prob, 1.0);
            // Comb-only single-module DUT (no flops, no hierarchy): the
            // first-cut surface projects combinational gates only.
            assert_eq!(cfg.flop_prob, 0.0);
            assert!(cfg.effective_hierarchy_depth_range().is_none());
        }
        assert_eq!(scenarios.len(), 3);
        assert_eq!(
            strategies,
            BTreeSet::from(["sequential", "shuffled", "interleaved"])
        );
    }

    #[test]
    fn task_emit_sweep_gaps_require_the_fact() {
        // All three strategies present, but the fact not lit → exactly the
        // one task-emit gap (no broad-motif gaps leak in).
        let mut coverage = CoverageSummary::default();
        for s in ["sequential", "shuffled", "interleaved"] {
            coverage.construction_strategies.insert(s.to_string());
        }
        let gaps = compute_coverage_gaps(ScenarioSet::TaskEmitSweep, &coverage, None);
        assert_eq!(gaps.len(), 1, "unexpected gaps: {gaps:?}");
        assert!(gaps[0].contains("task_emit_prob"));

        // Fact lit → no gaps.
        coverage.saw_combinational_task_emit = true;
        let gaps = compute_coverage_gaps(ScenarioSet::TaskEmitSweep, &coverage, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    // ===============================================================
    // STRUCTURED-EMISSION-EXPANSION.10b.2 — cargo-portable proofs of the
    // repo-owned multi-gate-cone `function automatic` emit gate wiring
    // (CLI flag, scenario-set selection, forced knob, gap enforcement).
    // ===============================================================

    #[test]
    fn cone_function_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.cone_function_gate);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--cone-function-gate", "--out", "/tmp/x"])
                .expect("parse");
        assert!(with_flag.cone_function_gate);
    }

    #[test]
    fn cone_function_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.cone_function_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::ConeFunctionSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::ConeFunctionSweep).expect("build");
        // one comb-only focus config x three construction strategies.
        assert_eq!(scenarios.len(), 3);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(
            plan.modules_per_scenario,
            CONE_FUNCTION_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert_eq!(
            plan.total_modules,
            3 * CONE_FUNCTION_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn cone_function_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.cone_function_gate = true;
        cli.task_emit_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn cone_function_sweep_scenarios_force_the_knob() {
        let scenarios = build_scenarios(0, ScenarioSet::ConeFunctionSweep).expect("build");
        let mut strategies = BTreeSet::new();
        for scenario in &scenarios {
            strategies.insert(construction_strategy_slug(
                scenario.config.construction_strategy,
            ));
            let cfg = &scenario.config;
            assert!(
                scenario.name.ends_with("cone_function"),
                "unexpected cone-function scenario {}",
                scenario.name
            );
            // cone_function_emit_prob forced to 1.0 so every qualifying
            // combinational cone is projected to a multi-gate `function
            // automatic`.
            assert_eq!(cfg.cone_function_emit_prob, 1.0);
            // Comb-only single-module DUT (no flops, no hierarchy): the cone
            // surface projects combinational cones only.
            assert_eq!(cfg.flop_prob, 0.0);
            assert!(cfg.effective_hierarchy_depth_range().is_none());
        }
        assert_eq!(scenarios.len(), 3);
        assert_eq!(
            strategies,
            BTreeSet::from(["sequential", "shuffled", "interleaved"])
        );
    }

    #[test]
    fn cone_function_sweep_gaps_require_the_fact() {
        // All three strategies present, but the fact not lit → exactly the
        // one cone-function gap (no broad-motif gaps leak in).
        let mut coverage = CoverageSummary::default();
        for s in ["sequential", "shuffled", "interleaved"] {
            coverage.construction_strategies.insert(s.to_string());
        }
        let gaps = compute_coverage_gaps(ScenarioSet::ConeFunctionSweep, &coverage, None);
        assert_eq!(gaps.len(), 1, "unexpected gaps: {gaps:?}");
        assert!(gaps[0].contains("cone_function_emit_prob"));

        // Fact lit → no gaps.
        coverage.saw_cone_function_emit = true;
        let gaps = compute_coverage_gaps(ScenarioSet::ConeFunctionSweep, &coverage, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    // ===============================================================
    // SV-VERSION-TARGETING.2b.2b — cargo-portable proofs of the
    // repo-owned per-version acceptance gate wiring (CLI flag, scenario
    // set, run-plan, per-version scenario shaping, Verilator-language
    // selector, gap requirements). The downstream-clean bank is the
    // repo-owned report, run separately with real Verilator + Yosys.
    // ===============================================================

    #[test]
    fn sv_version_gate_flag_defaults_false_and_parses() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.sv_version_gate);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--sv-version-gate", "--out", "/tmp/x"])
                .expect("parse");
        assert!(with_flag.sv_version_gate);
    }

    #[test]
    fn sv_version_gate_selects_set_and_raises_units() {
        let mut cli = test_cli();
        cli.sv_version_gate = true;
        assert_eq!(
            select_scenario_set(&cli).expect("select"),
            ScenarioSet::SvVersionSweep
        );
        let scenarios = build_scenarios(0, ScenarioSet::SvVersionSweep).expect("build");
        // three versions x {comb leaf, seq leaf, hierarchy design} = 9,
        // plus the `.3b.2b` 2023 `union soft` up-opt scenario = 10.
        assert_eq!(scenarios.len(), 10);
        let plan = derive_run_plan(&cli, scenarios.len());
        assert_eq!(
            plan.modules_per_scenario,
            SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert_eq!(
            plan.total_modules,
            10 * SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO
        );
        assert!(plan.fail_on_coverage_gap);
    }

    #[test]
    fn sv_version_gate_is_mutually_exclusive_with_other_gates() {
        let mut cli = test_cli();
        cli.sv_version_gate = true;
        cli.signoff_knob_sweep_gate = true;
        assert!(select_scenario_set(&cli).is_err());
    }

    #[test]
    fn sv_version_sweep_scenarios_target_each_version() {
        let scenarios = build_scenarios(0, ScenarioSet::SvVersionSweep).expect("build");
        let mut by_version: BTreeMap<SvVersion, (u32, u32, u32)> = BTreeMap::new();
        for scenario in &scenarios {
            // Every scenario uses the Interleaved strategy (the gate's
            // contract is per-version acceptance, not strategy breadth).
            assert_eq!(
                scenario.config.construction_strategy,
                ConstructionStrategy::Interleaved
            );
            let v = scenario.config.sv_version;
            let entry = by_version.entry(v).or_default();
            let year = sv_version_year_slug(v);
            if scenario.name == format!("sv{year}_comb_egraph") {
                entry.0 += 1;
                assert!(scenario.config.effective_hierarchy_depth_range().is_none());
            } else if scenario.name == format!("sv{year}_seq_motif") {
                entry.1 += 1;
                assert!(scenario.config.effective_hierarchy_depth_range().is_none());
                assert!(scenario.config.flop_prob > 0.0);
            } else if scenario.name == format!("sv{year}_hier_recursive") {
                entry.2 += 1;
                assert!(scenario.config.effective_hierarchy_depth_range().is_some());
            } else if scenario.name == "sv2023_soft_union_upopt" {
                // `.3b.2b` — the up-opt scenario: a 2023-targeted slice-heavy
                // comb leaf that requests the `union soft` overlay. It is NOT
                // part of the per-version (comb/seq/hier) triple, so it does
                // not increment the triple counters.
                assert_eq!(v, SvVersion::Sv2023);
                assert!(scenario.config.soft_union_slice_prob > 0.0);
                assert!(scenario.config.effective_hierarchy_depth_range().is_none());
                assert!(scenario_emits_soft_union_overlay(scenario));
            } else {
                panic!("unexpected sv-version scenario {}", scenario.name);
            }
        }
        assert_eq!(by_version.len(), 3);
        for version in [SvVersion::Sv2012, SvVersion::Sv2017, SvVersion::Sv2023] {
            assert_eq!(
                by_version.get(&version),
                Some(&(1, 1, 1)),
                "missing comb/seq/hier triple for {version:?}"
            );
        }
    }

    #[test]
    fn verilator_language_for_only_targets_under_the_gate() {
        let scenario = make_scenario("sv2017_probe", "probe", {
            let mut cfg = relaxed_default_config(ConstructionStrategy::Interleaved, 0);
            cfg.sv_version = SvVersion::Sv2017;
            cfg
        })
        .expect("scenario");
        // Off the gate: today's byte-identical argv (no `--language`).
        assert_eq!(verilator_language_for(&scenario, false), None);
        // Under the gate: the scenario's matching standard mode.
        assert_eq!(verilator_language_for(&scenario, true), Some("1800-2017"));
    }

    #[test]
    fn sv_version_sweep_gaps_require_each_version_fact() {
        // No fact lit → exactly the three per-version gaps + the umbrella
        // gap + the `.3b.2b` up-opt gap. Crucially, an EMPTY
        // construction-strategy set produces no strategy gaps: the version
        // gate returns before the strategy loop (its contract is per-version
        // acceptance only).
        let coverage = CoverageSummary::default();
        let gaps = compute_coverage_gaps(ScenarioSet::SvVersionSweep, &coverage, None);
        assert_eq!(gaps.len(), 5, "unexpected gaps: {gaps:?}");
        assert!(gaps.iter().any(|g| g.contains("1800-2012")));
        assert!(gaps.iter().any(|g| g.contains("1800-2017")));
        assert!(gaps.iter().any(|g| g.contains("1800-2023")));
        assert!(gaps
            .iter()
            .any(|g| g.contains("any sv_version targeted acceptance")));
        assert!(gaps.iter().any(|g| g.contains("union soft")));

        // All per-version facts (and the umbrella) lit but NOT the up-opt
        // fact → exactly the one up-opt gap remains.
        let mut lit = CoverageSummary::default();
        light_sv_version_acceptance(&mut lit, SvVersion::Sv2012);
        light_sv_version_acceptance(&mut lit, SvVersion::Sv2017);
        light_sv_version_acceptance(&mut lit, SvVersion::Sv2023);
        let gaps = compute_coverage_gaps(ScenarioSet::SvVersionSweep, &lit, None);
        assert_eq!(gaps.len(), 1, "unexpected gaps: {gaps:?}");
        assert!(gaps[0].contains("union soft"));

        // Every fact (per-version + umbrella + up-opt) lit → no gaps, even
        // with no construction strategies recorded.
        lit.saw_sv_version_2023_soft_union_upopt = true;
        let gaps = compute_coverage_gaps(ScenarioSet::SvVersionSweep, &lit, None);
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
    }

    #[test]
    fn sv_version_sweep_has_verilator_only_soft_union_upopt_scenario() {
        let scenarios = build_scenarios(0, ScenarioSet::SvVersionSweep).expect("build");
        let upopt = scenarios
            .iter()
            .find(|s| s.name == "sv2023_soft_union_upopt")
            .expect("up-opt scenario present");

        // The up-opt scenario targets 2023, requests every qualifying
        // overlay, and is therefore Verilator-only (Yosys/Icarus no-op).
        assert_eq!(upopt.config.sv_version, SvVersion::Sv2023);
        assert_eq!(upopt.config.soft_union_slice_prob, 1.0);
        assert!(scenario_emits_soft_union_overlay(upopt));
        // Under the gate, Verilator runs in the matching 1800-2023 mode.
        assert_eq!(verilator_language_for(upopt, true), Some("1800-2023"));

        // The nine common-floor scenarios are NOT Verilator-only (they run
        // Yosys), and none requests the overlay.
        for s in scenarios
            .iter()
            .filter(|s| s.name != "sv2023_soft_union_upopt")
        {
            assert!(
                !scenario_emits_soft_union_overlay(s),
                "{} unexpectedly flagged as a soft-union overlay scenario",
                s.name
            );
            assert_eq!(s.config.soft_union_slice_prob, 0.0);
        }
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
                    || scenario
                        .name
                        .ends_with("phase4_recur_d7_stateful_parent_port_composed_output")
                    || scenario
                        .name
                        .ends_with("phase4_hier1_structurally_duplicate_modules")
                    || scenario.name.ends_with("phase4_hier1_module_dedup_active")
                    || scenario.name.ends_with("phase5_width_parameterized")
                    || scenario.name.ends_with("phase5b_packed_aggregate")
                    || scenario.name.ends_with("phase6_inferrable_memory")
                    || scenario.name.ends_with("phase6_fsm")
            );
        }
        assert_eq!(scenarios.len(), 222);
        assert_eq!(names.len(), 222);
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
            "phase4_recur_d7_stateful_parent_port_composed_output",
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
    fn phase5b_packed_aggregate_scenario_is_non_vacuous() {
        // PHASE-5B-AGGREGATES.2.3: the `phase5b_packed_aggregate`
        // anchor must actually produce a packed-aggregate-projected
        // module (the never-instantiated top wrapper), otherwise the
        // `saw_packed_aggregate_design` coverage fact would be
        // unreachable and `.2.4`'s gate would carry a permanent
        // coverage gap. Proven here so the scenario cannot silently go
        // vacuous.
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase4Hierarchy).expect("build phase4 scenarios");
        let mut checked = 0usize;
        let mut projected = 0usize;
        for scenario in &scenarios {
            if !scenario.name.ends_with("phase5b_packed_aggregate") {
                continue;
            }
            checked += 1;
            assert_eq!(scenario.config.aggregate_prob, 1.0);
            let design = Generator::new(scenario.config.clone()).generate_design();
            anvil::ir::validate::validate_design(&design)
                .expect("phase5b anchor design must validate");
            let m = anvil::metrics::compute_design(&design);
            if m.num_packed_aggregate_modules > 0 {
                projected += 1;
            }
        }
        assert!(checked > 0, "phase5b_packed_aggregate scenario must exist");
        assert_eq!(
            projected, checked,
            "every phase5b_packed_aggregate scenario must project ≥1 module \
             (got {projected}/{checked}); the coverage fact would be unreachable"
        );
    }

    #[test]
    fn phase6_inferrable_memory_scenario_is_non_vacuous() {
        // PHASE-6-ADVANCED-MOTIFS.2.3: the `phase6_inferrable_memory`
        // anchor must actually produce ≥1 `Memory`-bearing module
        // (the rules-first library leaves), otherwise the
        // `saw_inferrable_memory_design` coverage fact would be
        // unreachable and `.2.4`'s gate would carry a permanent
        // coverage gap. Proven here so the scenario cannot silently go
        // vacuous.
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase4Hierarchy).expect("build phase4 scenarios");
        let mut checked = 0usize;
        let mut with_memory = 0usize;
        for scenario in &scenarios {
            if !scenario.name.ends_with("phase6_inferrable_memory") {
                continue;
            }
            checked += 1;
            assert_eq!(scenario.config.memory_prob, 1.0);
            let design = Generator::new(scenario.config.clone()).generate_design();
            anvil::ir::validate::validate_design(&design)
                .expect("phase6 anchor design must validate");
            let m = anvil::metrics::compute_design(&design);
            if m.num_memory_modules > 0 {
                with_memory += 1;
            }
        }
        assert!(checked > 0, "phase6_inferrable_memory scenario must exist");
        assert_eq!(
            with_memory, checked,
            "every phase6_inferrable_memory scenario must build ≥1 memory module \
             (got {with_memory}/{checked}); the coverage fact would be unreachable"
        );
    }

    #[test]
    fn phase6_fsm_scenario_is_non_vacuous() {
        // PHASE-6-ADVANCED-MOTIFS.3.4a: the `phase6_fsm` anchor must
        // actually produce ≥1 `Fsm`-bearing module (the rules-first
        // library leaves), otherwise the `saw_fsm_design` coverage
        // fact would be unreachable and `.3.4b`'s gate would carry a
        // permanent coverage gap. Proven here so the scenario cannot
        // silently go vacuous.
        let scenarios =
            build_scenarios(0, ScenarioSet::Phase4Hierarchy).expect("build phase4 scenarios");
        let mut checked = 0usize;
        let mut with_fsm = 0usize;
        for scenario in &scenarios {
            if !scenario.name.ends_with("phase6_fsm") {
                continue;
            }
            checked += 1;
            assert_eq!(scenario.config.fsm_prob, 1.0);
            let design = Generator::new(scenario.config.clone()).generate_design();
            anvil::ir::validate::validate_design(&design)
                .expect("phase6 fsm anchor design must validate");
            let m = anvil::metrics::compute_design(&design);
            if m.num_fsm_modules > 0 {
                with_fsm += 1;
            }
        }
        assert!(checked > 0, "phase6_fsm scenario must exist");
        assert_eq!(
            with_fsm, checked,
            "every phase6_fsm scenario must build ≥1 fsm module \
             (got {with_fsm}/{checked}); the coverage fact would be unreachable"
        );
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
    fn iverilog_compile_cli_flag_defaults_to_false_and_parses_when_set() {
        use clap::Parser;

        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.iverilog_compile);
        assert_eq!(no_flag.iverilog_bin, "iverilog");

        let with_flag = Cli::try_parse_from([
            "tool_matrix",
            "--iverilog-compile",
            "--iverilog-bin",
            "/opt/homebrew/bin/iverilog",
            "--out",
            "/tmp/x",
        ])
        .expect("parse");
        assert!(with_flag.iverilog_compile);
        assert_eq!(with_flag.iverilog_bin, "/opt/homebrew/bin/iverilog");
    }

    /// `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` — the `--sv2v` opt-in column flag
    /// defaults off (so default runs stay byte-identical) and parses with a
    /// caller-supplied `--sv2v-bin`. (`sv2v` is also a valid `--tools` value —
    /// the `AcceptanceTool` clap `ValueEnum` derives the token `sv2v`.)
    #[test]
    fn sv2v_cli_flag_defaults_to_false_and_parses_when_set() {
        use clap::Parser;

        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.sv2v);
        assert_eq!(no_flag.sv2v_bin, "sv2v");

        let with_flag = Cli::try_parse_from([
            "tool_matrix",
            "--sv2v",
            "--sv2v-bin",
            "/opt/homebrew/bin/sv2v",
            "--out",
            "/tmp/x",
        ])
        .expect("parse");
        assert!(with_flag.sv2v);
        assert_eq!(with_flag.sv2v_bin, "/opt/homebrew/bin/sv2v");
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
                version: None,
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
                    version: None,
                },
                ToolInvocation {
                    tool: "yosys-with-abc".to_string(),
                    argv: vec![],
                    success: false,
                    exit_code: Some(1),
                    stdout_log: None,
                    stderr_log: Some("stderr.log".to_string()),
                    error: Some("ABC: Warning: example".to_string()),
                    version: None,
                },
            ],
            iverilog_compile: Some(ToolInvocation {
                tool: "iverilog-compile".to_string(),
                argv: vec![],
                success: true,
                exit_code: Some(0),
                stdout_log: None,
                stderr_log: None,
                error: None,
                version: None,
            }),
            sv2v: Some(ToolInvocation {
                tool: "sv2v".to_string(),
                argv: vec![],
                success: true,
                exit_code: Some(0),
                stdout_log: None,
                stderr_log: None,
                error: None,
                version: None,
            }),
            diff_sim: None,
            divergence: None,
            emitted_soft_union_overlay: false,
            emitted_combinational_function: false,
            emitted_generate_loop: false,
            emitted_combinational_task: false,
            emitted_cone_function: false,
        }];

        let summary = summarize_tools(&modules);
        assert_eq!(summary.verilator_passed, 1);
        assert_eq!(summary.yosys_without_abc_passed, 1);
        assert_eq!(summary.yosys_with_abc_failed, 1);
        assert_eq!(summary.iverilog_compile_passed, 1);
        assert_eq!(summary.sv2v_passed, 1);
        assert_eq!(summary.sv2v_failed, 0);
        assert_eq!(summary.yosys_failed(), 1);
        assert_eq!(summary.iverilog_failed(), 0);
        assert!(summary.any_failed());
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
            iverilog_compile: None,
            sv2v: None,
            diff_sim: None,
            divergence: None,
            emitted_soft_union_overlay: false,
            emitted_combinational_function: false,
            emitted_generate_loop: false,
            emitted_combinational_task: false,
            emitted_cone_function: false,
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
            iverilog_compile: None,
            sv2v: None,
            diff_sim: None,
            divergence: None,
            emitted_soft_union_overlay: false,
            emitted_combinational_function: false,
            emitted_generate_loop: false,
            emitted_combinational_task: false,
            emitted_cone_function: false,
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
                iverilog_compile: None,
                sv2v: None,
                yosys: vec![],
                diff_sim: None,
                divergence: None,
                emitted_soft_union_overlay: false,
                emitted_combinational_function: false,
                emitted_generate_loop: false,
                emitted_combinational_task: false,
                emitted_cone_function: false,
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
            false,
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
            false,
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
        let prepared0 = prepare_design(&mut baseline, &scenario_dir, 0, SvVersion::Sv2012).unwrap();
        for module in &prepared0.modules {
            fs::write(&module.sv_path, &module.sv_text).unwrap();
        }
        let report0 = run_design_tools(&cli, &prepared0, None).unwrap();
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

        let expected1 = prepare_design(&mut baseline, &scenario_dir, 1, SvVersion::Sv2012).unwrap();

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

        let actual1 = prepare_design(&mut resumed, &scenario_dir, 1, SvVersion::Sv2012).unwrap();
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
        let prepared = prepare_design(&mut generator, &scenario_dir, 0, SvVersion::Sv2012).unwrap();
        let report = run_design_tools(&cli, &prepared, None).unwrap();

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
        let prepared = prepare_design(&mut generator, &scenario_dir, 0, SvVersion::Sv2012).unwrap();

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
        let prepared = prepare_design(&mut generator, &scenario_dir, 0, SvVersion::Sv2012).unwrap();
        let report = run_design_tools(&cli, &prepared, None).unwrap();

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

    // ===============================================================
    // DIFFERENTIAL-SIMULATION.3b.2 — cargo-portable proofs of the
    // tool_matrix --diff-sim wiring (CLI flag, per-axis subset
    // selector, axis classifier, DUT-port parser, coverage merge,
    // ModuleReport.diff_sim threading). The end-to-end #[ignore]
    // gate lives separately so cargo test stays green tool-less
    // (Phase-1 doctrine).
    // ===============================================================

    #[test]
    fn diff_sim_cli_flag_defaults_to_false_and_parses_when_set() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.diff_sim);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--diff-sim", "--out", "/tmp/x"]).expect("parse");
        assert!(with_flag.diff_sim);
    }

    #[test]
    fn classify_diff_sim_axis_buckets_each_axis_correctly() {
        let comb = Config {
            memory_prob: 0.0,
            fsm_prob: 0.0,
            flop_prob: 0.0,
            ..Config::default()
        };
        assert_eq!(classify_diff_sim_axis(&comb), "combinational");
        let seq = Config {
            memory_prob: 0.0,
            fsm_prob: 0.0,
            flop_prob: 1.0,
            ..Config::default()
        };
        assert_eq!(classify_diff_sim_axis(&seq), "sequential-flop");
        // Memory and fsm take precedence over flop_prob; they
        // imply sequential state but the bucket name is more
        // specific.
        let mem = Config {
            memory_prob: 0.5,
            ..Config::default()
        };
        assert_eq!(classify_diff_sim_axis(&mem), "memory");
        let fsm = Config {
            memory_prob: 0.0,
            fsm_prob: 0.5,
            ..Config::default()
        };
        assert_eq!(classify_diff_sim_axis(&fsm), "fsm");
    }

    #[test]
    fn select_diff_sim_subset_picks_first_per_axis_and_caps_at_five() {
        // Build a synthetic scenario list covering all 5 axes
        // plus extras of the first axis (combinational) — the
        // selector should pick only the FIRST of each axis.
        let comb = Config {
            memory_prob: 0.0,
            fsm_prob: 0.0,
            flop_prob: 0.0,
            ..Config::default()
        };
        let seq = Config {
            memory_prob: 0.0,
            fsm_prob: 0.0,
            flop_prob: 1.0,
            ..Config::default()
        };
        let mem = Config {
            memory_prob: 0.5,
            ..Config::default()
        };
        let fsm = Config {
            memory_prob: 0.0,
            fsm_prob: 0.5,
            ..Config::default()
        };
        let scenarios = vec![
            Scenario {
                name: "comb-a".to_string(),
                description: "comb-a".to_string(),
                config: comb.clone(),
            },
            Scenario {
                name: "comb-b".to_string(),
                description: "comb-b".to_string(),
                config: comb,
            },
            Scenario {
                name: "seq-flop".to_string(),
                description: "seq-flop".to_string(),
                config: seq,
            },
            Scenario {
                name: "mem".to_string(),
                description: "mem".to_string(),
                config: mem,
            },
            Scenario {
                name: "fsm".to_string(),
                description: "fsm".to_string(),
                config: fsm,
            },
        ];

        let picked = select_diff_sim_subset(&scenarios);
        // First per axis, no duplicates.
        assert!(picked.contains(&"comb-a".to_string()));
        assert!(!picked.contains(&"comb-b".to_string()));
        assert!(picked.contains(&"seq-flop".to_string()));
        assert!(picked.contains(&"mem".to_string()));
        assert!(picked.contains(&"fsm".to_string()));
        assert!(picked.len() <= 5);
    }

    #[test]
    fn diff_sim_subset_against_default_scenarios_is_nonempty_and_capped() {
        let scenarios = build_scenarios(0, ScenarioSet::Default).expect("build scenarios");
        let picked = select_diff_sim_subset(&scenarios);
        assert!(
            !picked.is_empty(),
            "default scenarios must yield at least one axis"
        );
        assert!(picked.len() <= 5, "K=5 cap honored");
        for name in &picked {
            assert!(
                scenarios.iter().any(|s| &s.name == name),
                "picked name {name} must exist in scenarios"
            );
        }
    }

    #[test]
    fn merge_coverage_unions_saw_design_with_cross_simulator_agreement() {
        let mut dst = CoverageSummary::default();
        let src = CoverageSummary {
            saw_design_with_cross_simulator_agreement: true,
            ..CoverageSummary::default()
        };
        merge_coverage(&mut dst, &src);
        assert!(dst.saw_design_with_cross_simulator_agreement);
        // Re-merging with `false` source must not flip the dst.
        let zero = CoverageSummary::default();
        merge_coverage(&mut dst, &zero);
        assert!(dst.saw_design_with_cross_simulator_agreement);
    }

    #[test]
    fn summarize_coverage_lights_cross_simulator_agreement_from_any_passing_diff_sim() {
        let scenario = Scenario {
            name: "synthetic".to_string(),
            description: "synthetic".to_string(),
            config: Config::default(),
        };
        let mut modules: Vec<ModuleReport> = (0..3)
            .map(|i| ModuleReport {
                file: format!("mod_{i}.sv"),
                name: format!("mod_{i}"),
                metrics: Metrics::default(),
                verilator: None,
                iverilog_compile: None,
                sv2v: None,
                yosys: vec![],
                diff_sim: None,
                divergence: None,
                emitted_soft_union_overlay: false,
                emitted_combinational_function: false,
                emitted_generate_loop: false,
                emitted_combinational_task: false,
                emitted_cone_function: false,
            })
            .collect();
        // No DUTs ran diff-sim ⇒ fact stays false.
        let cov0 = summarize_coverage(&scenario, &modules, false);
        assert!(!cov0.saw_design_with_cross_simulator_agreement);
        // One DUT ran but failed ⇒ fact stays false.
        modules[1].diff_sim = Some(DiffSimReport {
            ran: true,
            success: false,
            n_samples: 8,
            skip_reason: String::new(),
            mismatch_excerpt: Some("iverilog | verilator\nA | B\n".to_string()),
        });
        let cov1 = summarize_coverage(&scenario, &modules, false);
        assert!(!cov1.saw_design_with_cross_simulator_agreement);
        // Another DUT ran AND succeeded ⇒ fact fires.
        modules[2].diff_sim = Some(DiffSimReport {
            ran: true,
            success: true,
            n_samples: 8,
            skip_reason: String::new(),
            mismatch_excerpt: None,
        });
        let cov2 = summarize_coverage(&scenario, &modules, false);
        assert!(cov2.saw_design_with_cross_simulator_agreement);
    }

    // ===============================================================
    // ACCEPTANCE-DIVERGENCE-HUNTING.2c.2 — cargo-portable proofs of
    // the tool_matrix --divergence column (CLI flag, the pure-projection
    // `unit_divergence` over already-run tools, coverage merge, and the
    // opportunistic `saw_acceptance_divergence` fact). The real-tool
    // end-to-end gate is `.2f` (kept separate so cargo test stays green
    // tool-less — Phase-1 doctrine).
    // ===============================================================

    #[test]
    fn divergence_cli_flag_defaults_to_false_and_parses_when_set() {
        use clap::Parser;
        let no_flag = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(!no_flag.divergence);
        let with_flag =
            Cli::try_parse_from(["tool_matrix", "--divergence", "--out", "/tmp/x"]).expect("parse");
        assert!(with_flag.divergence);
    }

    /// A synthetic `ToolInvocation` for the divergence proofs (no real tool
    /// spawned — the column is a pure projection of already-run invocations).
    fn divergence_test_inv(tool: &str, success: bool, exit_code: Option<i32>) -> ToolInvocation {
        ToolInvocation {
            tool: tool.to_string(),
            argv: vec![tool.to_string()],
            success,
            exit_code,
            stdout_log: None,
            stderr_log: None,
            error: if success {
                None
            } else {
                Some(format!("{tool}: not clean"))
            },
            version: None,
        }
    }

    #[test]
    fn unit_divergence_projects_already_run_tools_and_classifies_accept_reject() {
        use clap::Parser;
        // A unique parent with NO `.divergence-subset` sentinel ⇒ the
        // membership check defaults to "evaluate this scenario" (the helper is
        // testable without going through `run_matrix`).
        let parent = temp_test_dir("divergence-unit-proof");
        let scenario_dir = parent.join("scenario");
        let verilator = divergence_test_inv("verilator", true, Some(0));
        let yosys = vec![divergence_test_inv("yosys-without-abc", false, Some(1))];

        // Column off ⇒ no report at all (default-off / byte-identical).
        let off = Cli::try_parse_from(["tool_matrix", "--out", "/tmp/x"]).expect("parse");
        assert!(unit_divergence(
            &off,
            &scenario_dir,
            "module",
            "m",
            Some(&verilator),
            &yosys,
            None,
            None,
        )
        .is_none());

        // Column on ⇒ the tools the matrix already ran are projected and the
        // accept-vs-reject disagreement is classified — no extra tool spawned.
        let on =
            Cli::try_parse_from(["tool_matrix", "--divergence", "--out", "/tmp/x"]).expect("parse");
        let report = unit_divergence(
            &on,
            &scenario_dir,
            "module",
            "m",
            Some(&verilator),
            &yosys,
            None,
            None,
        )
        .expect("divergence column populated when enabled and in subset");
        assert_eq!(report.kind, "module");
        assert_eq!(report.top, "m");
        assert_eq!(report.verdicts.len(), 2);
        assert!(report.diverged);
        assert_eq!(report.divergences.len(), 1);
        assert_eq!(report.divergences[0].kind, "accept_reject");
        assert_eq!(
            report.divergences[0].tools,
            vec!["verilator", "yosys-without-abc"]
        );

        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn merge_coverage_unions_saw_acceptance_divergence() {
        let mut dst = CoverageSummary::default();
        let src = CoverageSummary {
            saw_acceptance_divergence: true,
            ..CoverageSummary::default()
        };
        merge_coverage(&mut dst, &src);
        assert!(dst.saw_acceptance_divergence);
        // Re-merging with a `false` source must not flip the dst.
        merge_coverage(&mut dst, &CoverageSummary::default());
        assert!(dst.saw_acceptance_divergence);
    }

    #[test]
    fn summarize_coverage_lights_acceptance_divergence_from_a_diverged_module() {
        let scenario = Scenario {
            name: "synthetic".to_string(),
            description: "synthetic".to_string(),
            config: Config::default(),
        };
        let mut modules: Vec<ModuleReport> = (0..2)
            .map(|i| ModuleReport {
                file: format!("mod_{i}.sv"),
                name: format!("mod_{i}"),
                metrics: Metrics::default(),
                verilator: None,
                iverilog_compile: None,
                sv2v: None,
                yosys: vec![],
                diff_sim: None,
                divergence: None,
                emitted_soft_union_overlay: false,
                emitted_combinational_function: false,
                emitted_generate_loop: false,
                emitted_combinational_task: false,
                emitted_cone_function: false,
            })
            .collect();
        // No unit carries a divergence report ⇒ the opportunistic fact stays
        // false.
        let cov0 = summarize_coverage(&scenario, &modules, false);
        assert!(!cov0.saw_acceptance_divergence);
        // An all-agree report (`diverged == false`) ⇒ still false: all-agree is
        // the valid-by-construction steady state, so it is NOT a finding.
        modules[0].divergence = Some(DivergenceReport {
            run_id: "mod_0".to_string(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: "mod_0".to_string(),
            sandbox: "/tmp/s".to_string(),
            verdicts: vec![],
            diverged: false,
            divergences: vec![],
            declined: None,
        });
        let cov1 = summarize_coverage(&scenario, &modules, false);
        assert!(!cov1.saw_acceptance_divergence);
        // A unit whose report diverged ⇒ the opportunistic fact fires.
        modules[1].divergence = Some(DivergenceReport {
            run_id: "mod_1".to_string(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: "mod_1".to_string(),
            sandbox: "/tmp/s".to_string(),
            verdicts: vec![],
            diverged: true,
            divergences: vec![divergence::Divergence {
                kind: "accept_reject".to_string(),
                tools: vec!["verilator".to_string(), "yosys-without-abc".to_string()],
            }],
            declined: None,
        });
        let cov2 = summarize_coverage(&scenario, &modules, false);
        assert!(cov2.saw_acceptance_divergence);
    }

    // NOTE: `parse_dut_ports_recognises_anvil_emitter_shape` and
    // `emit_testbench_for_ports_renders_combinational_and_sequential_shapes`
    // moved to `src/diff_sim/mod.rs` with their functions in
    // `BUG-HUNT-ORCHESTRATION.2a`.

    /// `DIFFERENTIAL-SIMULATION.3b.2` end-to-end tool-gated proof:
    /// run the matrix's per-module diff-sim helper against a real
    /// generated DUT and assert the `DiffSimReport` records a
    /// byte-equal trace. `#[ignore]` so `cargo test` stays green
    /// tool-less (Phase-1 doctrine). Run explicitly:
    /// `cargo test --bin tool_matrix -- --ignored
    /// run_diff_sim_for_module_end_to_end_gate`.
    #[test]
    #[ignore]
    fn run_diff_sim_for_module_end_to_end_gate() {
        use anvil::diff_sim;
        if !diff_sim::tools_present() {
            eprintln!(
                "run_diff_sim_for_module_end_to_end_gate: iverilog and/or verilator absent; skip"
            );
            return;
        }
        // Build a small combinational DUT (the diff-sim-portable
        // shape — same as tests/diff_sim.rs's seed=7 combinational
        // case so behavior is known-good per .2b.2's verification).
        let cfg = Config {
            seed: 7,
            flop_prob: 0.0,
            ..Config::default()
        };
        let mut gen = Generator::new(cfg);
        let top = gen.generate_module();
        let sv = anvil::emit::to_sv(&top);
        let dir = std::env::temp_dir().join(format!(
            "anvil-tool-matrix-diff-sim-e2e-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create dir");
        // Wire through the matrix's per-module helper.
        let report = run_diff_sim_for_module(&dir, "m_0_0000", &top.name, &sv);
        // Helper diagnostics → easier debugging.
        eprintln!("run_diff_sim_for_module ⇒ {report:?}");
        assert!(
            report.ran,
            "diff-sim should have run; skip_reason={:?}",
            report.skip_reason
        );
        assert!(
            report.success,
            "diff-sim should match byte-for-byte; excerpt={:?}",
            report.mismatch_excerpt
        );
        assert!(report.n_samples > 0, "diff-sim should report sample count");
        // Cleanup.
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ===============================================================
    // MULTI-CLOCK-CDC.3b.2 — cargo-portable proofs of the new
    // multi-clock coverage facts (CoverageSummary fields,
    // merge_coverage union, summarize_coverage lighting, the new
    // default-set scenario builder).
    // ===============================================================

    #[test]
    fn merge_coverage_unions_saw_multi_clock_design() {
        let mut dst = CoverageSummary::default();
        let src = CoverageSummary {
            saw_multi_clock_design: true,
            ..CoverageSummary::default()
        };
        merge_coverage(&mut dst, &src);
        assert!(dst.saw_multi_clock_design);
        assert!(!dst.saw_cdc_2_flop_synchronizer);
        // Re-merge with empty source must not clear.
        merge_coverage(&mut dst, &CoverageSummary::default());
        assert!(dst.saw_multi_clock_design);
    }

    #[test]
    fn merge_coverage_unions_saw_cdc_2_flop_synchronizer() {
        let mut dst = CoverageSummary::default();
        let src = CoverageSummary {
            saw_cdc_2_flop_synchronizer: true,
            ..CoverageSummary::default()
        };
        merge_coverage(&mut dst, &src);
        assert!(dst.saw_cdc_2_flop_synchronizer);
        assert!(!dst.saw_multi_clock_design);
    }

    #[test]
    fn merge_coverage_unions_saw_cdc_nflop_synchronizer() {
        let mut dst = CoverageSummary::default();
        let src = CoverageSummary {
            saw_cdc_nflop_synchronizer: true,
            ..CoverageSummary::default()
        };
        merge_coverage(&mut dst, &src);
        assert!(dst.saw_cdc_nflop_synchronizer);
        assert!(!dst.saw_cdc_2_flop_synchronizer);
    }

    #[test]
    fn summarize_coverage_lights_multi_clock_facts_from_module_metrics() {
        let scenario = Scenario {
            name: "synthetic".to_string(),
            description: "synthetic".to_string(),
            config: Config::default(),
        };
        // Baseline: K=1 module, no chains → both facts stay false.
        let mut modules: Vec<ModuleReport> = vec![ModuleReport {
            file: "m.sv".into(),
            name: "m".into(),
            metrics: Metrics::default(),
            verilator: None,
            iverilog_compile: None,
            sv2v: None,
            yosys: vec![],
            diff_sim: None,
            divergence: None,
            emitted_soft_union_overlay: false,
            emitted_combinational_function: false,
            emitted_generate_loop: false,
            emitted_combinational_task: false,
            emitted_cone_function: false,
        }];
        let cov0 = summarize_coverage(&scenario, &modules, false);
        assert!(!cov0.saw_multi_clock_design);
        assert!(!cov0.saw_cdc_2_flop_synchronizer);

        // Promote: num_clock_domains=2 lights saw_multi_clock_design.
        modules[0].metrics.num_clock_domains = 2;
        let cov1 = summarize_coverage(&scenario, &modules, false);
        assert!(cov1.saw_multi_clock_design);
        assert!(!cov1.saw_cdc_2_flop_synchronizer);

        // Add a synchronizer chain → both facts light.
        modules[0].metrics.num_cdc_2_flop_synchronizers = 1;
        let cov2 = summarize_coverage(&scenario, &modules, false);
        assert!(cov2.saw_multi_clock_design);
        assert!(cov2.saw_cdc_2_flop_synchronizer);

        modules[0].metrics.max_cdc_synchronizer_stages = 3;
        let cov3 = summarize_coverage(&scenario, &modules, false);
        assert!(cov3.saw_cdc_nflop_synchronizer);
    }

    #[test]
    fn build_default_scenarios_includes_multi_clock_scenario() {
        let scenarios = build_scenarios(0, ScenarioSet::Default).expect("build scenarios");
        let multi_clock = scenarios
            .iter()
            .find(|s| s.name == "int_multi_clock_2flop_sync")
            .expect("multi-clock scenario should be in the default set");
        assert!(multi_clock.config.multi_clock_prob > 0.0);
        assert_eq!(multi_clock.config.flop_prob, 1.0);
        assert_eq!(multi_clock.config.min_width, 1);
        assert_eq!(multi_clock.config.max_width, 1);
        assert_eq!(multi_clock.config.cdc_synchronizer_stages, 2);
        let nflop = scenarios
            .iter()
            .find(|s| s.name == "int_multi_clock_3flop_sync")
            .expect("N-flop multi-clock scenario should be in the default set");
        assert_eq!(nflop.config.cdc_synchronizer_stages, 3);
    }
}
