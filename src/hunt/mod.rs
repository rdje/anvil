//! BUG-HUNT-ORCHESTRATION — the turnkey downstream bug-hunt loop.
//!
//! Implements the design from decision record
//! `docs/decisions/0018-bug-hunt-orchestration-loop.md`: a **thin
//! orchestrator** (not a new engine) that composes the already-existing
//! `anvil::downstream::validate` / `anvil::downstream::minimize` surfaces into
//! one deterministic fuzz → detect → minimize loop. Both the MCP `hunt` tool
//! (`BUG-HUNT-ORCHESTRATION.2c`) and the `anvil hunt` CLI subcommand (`.2d`)
//! are thin shims over [`run`] — the CLI is a shim over the same API, never a
//! superset (decision `0017`).
//!
//! This module (`.2b.1`) is the **library core**: the request/report types and
//! the loop, with **reject / warning** detection. It adds no detector and no
//! minimizer of its own — detection is `!ValidateReport.ok` (and `validate`'s
//! `first_tool_warning` already folds a warning into `ok == false`, so reject
//! and warning are one unified failure signal). The optional **cross-simulator
//! mismatch** detector (`anvil::diff_sim::run_agreement`, available since
//! `.2a`) and the on-disk **reproducer bundle** are folded in by `.2b.2`.
//!
//! Every field of [`HuntReport`] is a `SCHEMA-DERIVED` projection of
//! `ValidateReport` / `MinimizeReport` / `ToolInvocation` (decision `0017`'s
//! queryable gate) — no new computed truth and no behavioural oracle (decision
//! `0004`'s no-shadow-simulator ceiling).
//!
//! Reproducibility + sandboxing are inherited by composing through
//! `downstream::validate` / `minimize`: seeded ChaCha8 (no wall-clock / no
//! `thread_rng`), the fixed `AcceptanceTool` allow-list, a caller-set sandbox
//! root (never agent-supplied, decision `0004`), and the `MemGuard`
//! decline-under-pressure surfaced as `declined`. The whole sweep is itself
//! reproducible from `(base_seed, seeds, config, validate-options, minimize,
//! max_oracle_calls)`.

use crate::config::Config;
use crate::diff_sim::{run_agreement, DiffSimReport};
use crate::downstream::{
    generate_dut_artifact, introspect_dut_artifact, minimize, validate, KnobReduction,
    MinimizeOptions, MinimizeReport, ToolInvocation, ValidateOptions, ValidateReport,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Baked input-vector count for the optional cross-simulator agreement check
/// (matches `tool_matrix`'s `--diff-sim` column).
const DIFF_SIM_VECTORS: usize = 8;

/// One bug-hunt request: a deterministic seed sweep over a fixed knob profile,
/// validated against a chosen set of downstream tools, with optional
/// auto-minimization of each failure.
///
/// The per-seed downstream run is configured by [`ValidateOptions`] (the tool
/// allow-list, Yosys mode, memory ceilings, and the **caller-set** sandbox
/// root — never agent-supplied, decision `0004`); the hunt only adds the sweep
/// extent and the minimize policy on top.
#[derive(Debug, Clone)]
pub struct HuntRequest {
    /// First seed in the sweep.
    pub base_seed: u64,
    /// Number of consecutive seeds to fuzz (`base_seed .. base_seed + seeds`).
    pub seeds: u32,
    /// The knob profile every seed is generated under (the DUT `Config`).
    pub config: Config,
    /// How each seed is validated downstream (tools, Yosys mode, memory
    /// ceilings, sandbox root). Reused verbatim as the minimize oracle's
    /// options, so a minimized reproducer is gated by the *same* guardrails.
    pub validate: ValidateOptions,
    /// Auto-minimize each failure to a smaller reproducer (`true`) or report
    /// the failure as-found (`false`).
    pub minimize: bool,
    /// Per-failure ceiling on minimize oracle (`validate`) evaluations.
    pub max_oracle_calls: u32,
    /// Run the optional **cross-simulator agreement** check on each
    /// downstream-clean artifact (iverilog ↔ verilator post-reset trace
    /// compare via [`run_agreement`]). A mismatch is a finding with
    /// `detection == "cross_sim_mismatch"`. Friendly no-op when either
    /// simulator is absent. Default-off behaviour is `false`.
    pub diff_sim: bool,
    /// When set, emit a self-contained **reproducer bundle** directory
    /// `<bundle_root>/<run_id>/` per finding (`repro.sv`, `knobs.json`,
    /// `introspection.json`, `tool-logs/`, `hunt-verdict.json`, and a
    /// one-command `repro.sh`), and attach a [`HuntBundle`] ref to the
    /// [`HuntFailure`]. `None` ⇒ no on-disk bundle (the default in-memory
    /// hunt). **Caller-set, never agent-supplied** (decision `0004`): the MCP
    /// shim fixes this to a sandboxed per-run scope; the `anvil hunt` CLI
    /// human may direct it via `--out`.
    pub bundle_root: Option<PathBuf>,
}

/// The per-seed outcome line: clean, declined (memory guard), or a failure
/// (the failure detail is carried separately in [`HuntReport::failures`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntVerdict {
    pub seed: u64,
    /// Content-addressed `run_id` of the artifact for this `(seed, config)`.
    pub run_id: String,
    /// `true` iff the artifact was downstream-clean (every selected tool
    /// accepted it). A finding is `ok == false` with `declined == None`.
    pub ok: bool,
    /// Set iff the memory guard declined a tool spawn for this seed; the seed
    /// is neither clean nor a confirmed finding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declined: Option<String>,
}

/// The minimize result for one failure, projected from [`MinimizeReport`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntMinimized {
    /// Whether the original `(seed, knobs)` reproduced the failure at all.
    pub reproduced_initial: bool,
    /// Each knob the search reduced, in deterministic registry order.
    pub reductions: Vec<KnobReduction>,
    /// How many oracle (`validate`) evaluations the search spent.
    pub oracle_calls: u32,
    /// `true` iff the search stopped on the oracle-call ceiling.
    pub budget_exhausted: bool,
    /// Content-addressed `run_id` of the minimized reproducer.
    pub minimized_run_id: String,
    /// The smallest reproducing config found.
    pub minimized_config: Config,
}

impl HuntMinimized {
    /// Project a [`MinimizeReport`] into the report shape. `fallback_run_id`
    /// is used when the search did not reproduce (so there is no
    /// `final_validation` to read a `run_id` from — the minimized config then
    /// echoes the input, so its `run_id` equals the original).
    fn from_report(m: &MinimizeReport, fallback_run_id: &str) -> Self {
        let minimized_run_id = m
            .final_validation
            .as_ref()
            .map(|v| v.run_id.clone())
            .unwrap_or_else(|| fallback_run_id.to_string());
        Self {
            reproduced_initial: m.reproduced_initial,
            reductions: m.reductions.clone(),
            oracle_calls: m.oracle_calls,
            budget_exhausted: m.budget_exhausted,
            minimized_run_id,
            minimized_config: m.minimized_config.clone(),
        }
    }
}

/// A confirmed finding — a candidate **downstream-tool** bug (never an ANVIL
/// bug; the output is legal by construction). The hunt **classifies**, it does
/// not adjudicate: the real tool's verdict and ANVIL's manifests remain the
/// source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntFailure {
    pub seed: u64,
    /// Content-addressed `run_id` of the failing artifact.
    pub run_id: String,
    /// The first tool that rejected/warned (e.g. `verilator`,
    /// `yosys-without-abc`).
    pub failing_tool: String,
    /// The exact reproducible command line of the failing tool.
    pub failing_argv: Vec<String>,
    /// The first warning/error line, when the invocation parsed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_error: Option<String>,
    /// `"reject"` (non-zero exit), `"warning"` (clean exit, warning folded into
    /// `success == false`), or `"cross_sim_mismatch"` (the two simulators
    /// disagreed on a validate-clean artifact).
    pub detection: String,
    /// The auto-minimize result, when `HuntRequest::minimize` was set. Absent
    /// for a `cross_sim_mismatch` finding — the `validate`-based minimize oracle
    /// only shrinks parse/synth failures, not a trace disagreement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimized: Option<HuntMinimized>,
    /// The cross-simulator agreement report, present on a `cross_sim_mismatch`
    /// finding (carries `ran` / `success` / `n_samples` / `mismatch_excerpt`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_sim: Option<DiffSimReport>,
    /// The on-disk reproducer bundle, present iff [`HuntRequest::bundle_root`]
    /// was set (and the bundle wrote). Absent for the default in-memory hunt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<HuntBundle>,
}

/// A reference to an emitted **reproducer bundle** directory
/// (`BUG-HUNT-ORCHESTRATION.2b.2b`). The bundle itself is the directory
/// `<bundle_root>/<run_id>/` (see [`HuntRequest::bundle_root`]); this carries
/// its filesystem path plus the `anvil://…` resource URIs an agent fetches the
/// parts through (the MCP artifact-resource scheme — `BUG-HUNT-ORCHESTRATION.2c`
/// populates the cache so those reads resolve). The `run_id` in every URI is the
/// content address of the artifact the bundle reproduces (the minimized
/// reproducer's when minimize shrank it, else the originally-detected one).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntBundle {
    /// The bundle directory `<bundle_root>/<run_id>/`.
    pub path: String,
    /// `anvil://artifact/<run_id>/sv` — the emitted reproducer SystemVerilog.
    pub sv: String,
    /// `anvil://artifact/<run_id>/introspection` — the construction-truth
    /// `IntrospectionDocument`.
    pub introspection: String,
    /// `anvil://artifact/<run_id>/manifest` — the expected-facts manifest, for
    /// the non-DUT lanes only. Absent for the DUT lane (which has no manifest,
    /// matching the introspect contract); the hunt is DUT-only today.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
}

/// Aggregate counts over the sweep (`n_seeds == n_clean + n_failures +
/// n_declined`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntSummary {
    pub n_seeds: usize,
    pub n_clean: usize,
    pub n_failures: usize,
    pub n_declined: usize,
    /// Of the failures, how many minimized to a reproducer that still fails.
    pub n_reproduced: usize,
}

/// The structured result of a [`run`] sweep — entirely `SCHEMA-DERIVED` from
/// `ValidateReport` / `MinimizeReport` / `ToolInvocation`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntReport {
    pub base_seed: u64,
    pub seeds: u32,
    /// The artifact lane. `"dut"` for now — `validate` / `minimize` are
    /// DUT-only; non-DUT lanes are a future extension.
    pub lane: String,
    /// One line per seed, in sweep order.
    pub verdicts: Vec<HuntVerdict>,
    /// One entry per confirmed finding, in sweep order.
    pub failures: Vec<HuntFailure>,
    pub summary: HuntSummary,
}

/// The first tool invocation that did not succeed (a reject or a warning;
/// `validate` folds both into `success == false`).
fn first_failing_tool(tools: &[ToolInvocation]) -> Option<&ToolInvocation> {
    tools.iter().find(|t| !t.success)
}

/// Classify a failing invocation: a clean exit (`Some(0)`) that still didn't
/// succeed is a **warning** (warning-clean output is the by-construction
/// contract, so any warning is a finding); a non-zero / unknown exit is a
/// **reject**.
fn classify_detection(t: &ToolInvocation) -> &'static str {
    if t.exit_code == Some(0) {
        "warning"
    } else {
        "reject"
    }
}

/// The per-iteration config for `seed`: the request's knob profile with its
/// `seed` field stamped to the sweep seed. Load-bearing — the generator seeds
/// from `Config::seed`, so without this every sweep position would regenerate
/// the same artifact (see [`run`]).
fn seed_config(req: &HuntRequest, seed: u64) -> Config {
    let mut cfg = req.config.clone();
    cfg.seed = seed;
    cfg
}

/// Run the cross-simulator agreement check on a downstream-clean `(seed, cfg)`
/// artifact. Returns `Some(report)` **only** when both simulators ran and
/// disagreed (a `cross_sim_mismatch` finding); `None` when they agreed or when
/// the check was a no-op (a simulator absent, or the SV port section
/// unparsable — `run_agreement` reports `ran == false`). The DUT SV is
/// regenerated through the shared `downstream::generate_dut_artifact` so the
/// two simulators see exactly the artifact `validate` accepted. The work dir is
/// a per-run sandbox under the caller-set `sandbox_root` (never agent-supplied,
/// decision `0004`), removed after the run unless `keep_sandbox`.
fn cross_sim_mismatch(req: &HuntRequest, cfg: &Config, run_id: &str) -> Option<DiffSimReport> {
    let (_kind, top, sv) = generate_dut_artifact(cfg);
    let work_dir = req
        .validate
        .sandbox_root
        .join(format!("anvil-hunt-diffsim-{run_id}"));
    let ds = run_agreement(&work_dir, &top, &sv, DIFF_SIM_VECTORS);
    if !req.validate.keep_sandbox {
        let _ = std::fs::remove_dir_all(&work_dir);
    }
    if ds.ran && !ds.success {
        Some(ds)
    } else {
        None
    }
}

/// What `tool-logs/NOTE.txt` says: ANVIL's `validate` runs each tool in an
/// ephemeral per-run sandbox that is removed after the run, so the raw stdout/
/// stderr streams cannot be copied post-hoc; the captured failing line lives in
/// `hunt-verdict.json`, and `repro.sh` regenerates the full output on demand.
const TOOL_LOGS_NOTE: &str = "\
The downstream tool's captured stdout/stderr logs are not copied into this
bundle: ANVIL's `validate` runs each tool in an ephemeral per-run sandbox that
is removed after the run, so the streams no longer exist by the time the bundle
is written. The first failing line is recorded in `hunt-verdict.json`
(`first_error`, and `diff_sim.mismatch_excerpt` for a cross-simulator finding).
Run `./repro.sh` to regenerate the exact artifact and re-emit the tool's full
output.
";

/// POSIX single-quote one shell token so an embedded path or Yosys `-p` script
/// survives verbatim through `repro.sh` (handles spaces, `;`, and `"`; an
/// embedded `'` becomes `'\''`). The vetted tool argv has no single quotes
/// today, but quoting unconditionally keeps the script robust.
fn shell_quote(tok: &str) -> String {
    format!("'{}'", tok.replace('\'', "'\\''"))
}

/// Mark `repro.sh` executable on Unix (best-effort; a failure is harmless —
/// `bash repro.sh` still works). A no-op on non-Unix hosts.
fn mark_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Render the one-command `repro.sh`: regenerate the exact `.sv` from the
/// recorded `(seed, knobs.json)` (byte-identical by ANVIL's reproducibility
/// contract), then re-run the tool that failed against the regenerated
/// `repro.sv`.
///
/// The failing tool's captured `argv` references the now-deleted sandbox SV
/// path; we substitute that exact path string with `repro.sv` (a plain
/// substring replace, which also rewrites the path embedded in a Yosys `-p`
/// script — temp paths need no double-quote escaping, so they appear verbatim).
/// A `cross_sim_mismatch` finding has no rejecting tool, so step 2 points the
/// filer at the recorded `diff_sim` excerpt instead.
fn repro_script(seed: u64, repro_validate: &ValidateReport, detection: &str) -> String {
    let mut s = String::new();
    s.push_str("#!/usr/bin/env bash\n");
    s.push_str(&format!(
        "# ANVIL hunt reproducer — run_id {}\n",
        repro_validate.run_id
    ));
    s.push_str(&format!("#   seed:      {seed}\n"));
    s.push_str(&format!("#   detection: {detection}\n"));
    s.push_str("# Step 1 regenerates the exact artifact (byte-identical by ANVIL's\n");
    s.push_str("# reproducibility contract); step 2 re-runs the downstream tool that\n");
    s.push_str("# failed. Requires `anvil` and the downstream tool on PATH.\n");
    s.push_str("set -euo pipefail\n");
    s.push_str("cd \"$(dirname \"$0\")\"\n\n");
    s.push_str("# 1. Regenerate the reproducer RTL from the recorded (seed, knobs).\n");
    s.push_str(&format!(
        "anvil --seed {seed} --config knobs.json > repro.sv\n\n"
    ));
    s.push_str("# 2. Re-run the failing tool against repro.sv.\n");

    let sandbox_sv = Path::new(&repro_validate.sandbox)
        .join(format!("{}.sv", repro_validate.top))
        .display()
        .to_string();
    match first_failing_tool(&repro_validate.tools) {
        Some(t) => {
            let cmd = t
                .argv
                .iter()
                .map(|tok| shell_quote(&tok.replace(&sandbox_sv, "repro.sv")))
                .collect::<Vec<_>>()
                .join(" ");
            s.push_str(&cmd);
            s.push('\n');
        }
        None => {
            // A cross_sim_mismatch on a validate-clean artifact: no tool
            // rejected it, so there is nothing to replay. The trace
            // disagreement is in hunt-verdict.json (diff_sim).
            s.push_str(
                "echo 'cross-simulator mismatch — see hunt-verdict.json (diff_sim) for the excerpt;'\n",
            );
            s.push_str(
                "echo 'reproduce by simulating repro.sv under iverilog + verilator (ANVIL --diff-sim).'\n",
            );
        }
    }
    s
}

/// Write the self-contained reproducer bundle directory
/// `<bundle_root>/<run_id>/` for one finding and return its [`HuntBundle`] ref.
///
/// `repro_cfg` is the artifact the bundle reproduces (the minimized reproducer's
/// config when minimize shrank it, else the originally-detected one);
/// `repro_validate` is the matching [`ValidateReport`] (its `run_id` names the
/// directory and its failing tool drives `repro.sh`); `verdict` is the finding
/// written to `hunt-verdict.json` (its `bundle` field is `None` at write time —
/// the ref would point back at this very directory). Pure composition over
/// [`generate_dut_artifact`] (the `.sv`) and [`introspect_dut_artifact`] (the
/// document); runs no tool. Returns `Err` only on a genuine filesystem failure.
fn write_bundle(
    bundle_root: &Path,
    seed: u64,
    repro_cfg: &Config,
    repro_validate: &ValidateReport,
    verdict: &HuntFailure,
) -> Result<HuntBundle> {
    let run_id = &repro_validate.run_id;
    let dir = bundle_root.join(run_id);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create bundle dir {}", dir.display()))?;

    // repro.sv — the exact artifact `validate` ran (same `generate_dut_artifact`
    // path, deterministic from `repro_cfg`).
    let (_kind, _top, sv) = generate_dut_artifact(repro_cfg);
    let sv_path = dir.join("repro.sv");
    std::fs::write(&sv_path, &sv).with_context(|| format!("write {}", sv_path.display()))?;

    // knobs.json — the (minimized) effective Config: the exact (seed, knobs).
    let knobs = serde_json::to_string_pretty(repro_cfg)?;
    let knobs_path = dir.join("knobs.json");
    std::fs::write(&knobs_path, format!("{knobs}\n"))
        .with_context(|| format!("write {}", knobs_path.display()))?;

    // introspection.json — construction truth (IntrospectionDocument).
    let doc = introspect_dut_artifact(seed, repro_cfg);
    let doc_path = dir.join("introspection.json");
    std::fs::write(&doc_path, format!("{}\n", doc.to_json_pretty()?))
        .with_context(|| format!("write {}", doc_path.display()))?;

    // hunt-verdict.json — the finding facts (the `bundle` ref is omitted; it
    // would point back here).
    let verdict_json = serde_json::to_string_pretty(verdict)?;
    let verdict_path = dir.join("hunt-verdict.json");
    std::fs::write(&verdict_path, format!("{verdict_json}\n"))
        .with_context(|| format!("write {}", verdict_path.display()))?;

    // tool-logs/ — a note (the sandbox logs are ephemeral; repro.sh re-emits).
    let logs_dir = dir.join("tool-logs");
    std::fs::create_dir_all(&logs_dir).with_context(|| format!("create {}", logs_dir.display()))?;
    let note_path = logs_dir.join("NOTE.txt");
    std::fs::write(&note_path, TOOL_LOGS_NOTE)
        .with_context(|| format!("write {}", note_path.display()))?;

    // repro.sh — one-command regenerate + re-run.
    let script_path = dir.join("repro.sh");
    std::fs::write(
        &script_path,
        repro_script(seed, repro_validate, &verdict.detection),
    )
    .with_context(|| format!("write {}", script_path.display()))?;
    mark_executable(&script_path);

    Ok(HuntBundle {
        path: dir.display().to_string(),
        sv: format!("anvil://artifact/{run_id}/sv"),
        introspection: format!("anvil://artifact/{run_id}/introspection"),
        manifest: None,
    })
}

/// Run the bug-hunt loop: fuzz a deterministic seed sweep, detect any
/// reject/warning, auto-minimize each failure (when requested), and return a
/// structured [`HuntReport`].
///
/// This is the single entry point both the MCP `hunt` tool and the `anvil
/// hunt` CLI shim over. It performs no I/O of its own beyond what
/// `downstream::validate` / `minimize` already do (generate into a caller-set
/// sandbox, run the allow-listed tools, audit-log each call). It returns `Err`
/// only on a genuine I/O failure from a `validate` / `minimize` call; a
/// memory-guard decline is a normal per-seed outcome (`HuntVerdict::declined`),
/// not an error.
pub fn run(req: &HuntRequest) -> Result<HuntReport> {
    let mut verdicts: Vec<HuntVerdict> = Vec::with_capacity(req.seeds as usize);
    let mut failures: Vec<HuntFailure> = Vec::new();

    for k in 0..req.seeds {
        let seed = req.base_seed.wrapping_add(k as u64);
        // The generator seeds from `Config::seed` (`Generator::new`), and
        // `validate`/`minimize` follow the caller convention of threading the
        // sweep seed into the config (`config_from_args` sets `cfg.seed = seed`).
        // So the sweep MUST stamp `seed` into the per-iteration config; passing
        // the request config unchanged would regenerate the *same* artifact for
        // every seed under different run_ids.
        let cfg = seed_config(req, seed);
        let report = validate(seed, &cfg, &req.validate)?;
        let run_id = report.run_id.clone();

        if let Some(reason) = &report.declined {
            verdicts.push(HuntVerdict {
                seed,
                run_id,
                ok: false,
                declined: Some(reason.clone()),
            });
            continue;
        }

        if report.ok {
            // Optional cross-simulator agreement on the parse/synth-clean
            // artifact. A mismatch is a *new* kind of finding (a tool accepted
            // it, but two simulators disagree on its semantics); a no-op (a
            // simulator absent) leaves the artifact clean.
            if req.diff_sim {
                if let Some(ds) = cross_sim_mismatch(req, &cfg, &run_id) {
                    verdicts.push(HuntVerdict {
                        seed,
                        run_id: run_id.clone(),
                        ok: false,
                        declined: None,
                    });
                    let mut failure = HuntFailure {
                        seed,
                        run_id: run_id.clone(),
                        failing_tool: "diff-sim".to_string(),
                        failing_argv: Vec::new(),
                        first_error: ds.mismatch_excerpt.clone(),
                        detection: "cross_sim_mismatch".to_string(),
                        // The validate-based minimize oracle can't reproduce a
                        // trace disagreement, so cross-sim findings aren't shrunk.
                        minimized: None,
                        diff_sim: Some(ds),
                        bundle: None,
                    };
                    // The bundle reproduces the validate-clean artifact (no
                    // rejecting tool — `repro.sh` points at the diff_sim excerpt).
                    if let Some(root) = &req.bundle_root {
                        failure.bundle = Some(write_bundle(root, seed, &cfg, &report, &failure)?);
                    }
                    failures.push(failure);
                    continue;
                }
            }
            verdicts.push(HuntVerdict {
                seed,
                run_id,
                ok: true,
                declined: None,
            });
            continue;
        }

        // A finding: at least one tool rejected/warned on legal-by-construction
        // RTL — a candidate downstream-tool bug.
        verdicts.push(HuntVerdict {
            seed,
            run_id: run_id.clone(),
            ok: false,
            declined: None,
        });

        let (failing_tool, failing_argv, first_error, detection) =
            match first_failing_tool(&report.tools) {
                Some(t) => (
                    t.tool.clone(),
                    t.argv.clone(),
                    t.error.clone(),
                    classify_detection(t).to_string(),
                ),
                // No per-tool row failed but the overall verdict did — defensive
                // (shouldn't happen given `ok == false`); record a bare reject.
                None => (String::new(), Vec::new(), None, "reject".to_string()),
            };

        let mut failure = HuntFailure {
            seed,
            run_id: run_id.clone(),
            failing_tool,
            failing_argv,
            first_error,
            detection,
            minimized: None,
            diff_sim: None,
            bundle: None,
        };

        // The bundle reproduces the minimized artifact when minimize confirmed a
        // smaller still-failing reproducer; otherwise the originally-detected one.
        let mut bundled_minimized = false;
        if req.minimize {
            let mopts = MinimizeOptions {
                validate: req.validate.clone(),
                max_oracle_calls: req.max_oracle_calls,
            };
            let m = minimize(seed, &cfg, &mopts)?;
            failure.minimized = Some(HuntMinimized::from_report(&m, &run_id));
            if let (Some(root), true, Some(fv)) =
                (&req.bundle_root, m.reproduced_initial, &m.final_validation)
            {
                failure.bundle = Some(write_bundle(root, seed, &m.minimized_config, fv, &failure)?);
                bundled_minimized = true;
            }
        }
        if let (Some(root), false) = (&req.bundle_root, bundled_minimized) {
            // No minimize, or it did not produce a confirmed smaller reproducer:
            // bundle the originally-detected `(cfg, report)`.
            failure.bundle = Some(write_bundle(root, seed, &cfg, &report, &failure)?);
        }

        failures.push(failure);
    }

    let summary = HuntSummary {
        n_seeds: verdicts.len(),
        n_clean: verdicts.iter().filter(|v| v.ok).count(),
        n_failures: failures.len(),
        n_declined: verdicts.iter().filter(|v| v.declined.is_some()).count(),
        n_reproduced: failures
            .iter()
            .filter(|f| f.minimized.as_ref().is_some_and(|m| m.reproduced_initial))
            .count(),
    };

    Ok(HuntReport {
        base_seed: req.base_seed,
        seeds: req.seeds,
        lane: "dut".to_string(),
        verdicts,
        failures,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique sandbox root per test so parallel test runs never collide on a
    /// per-`run_id` subdirectory. Cargo-portable: the sandbox is a fresh temp
    /// subtree, removed by `validate` after each run.
    fn test_validate_opts(tag: &str) -> ValidateOptions {
        ValidateOptions {
            tools: vec![], // no-tool smoke: generate + sandbox only, no real tools
            sandbox_root: std::env::temp_dir().join(format!("anvil-hunt-test-{tag}")),
            ..ValidateOptions::default()
        }
    }

    fn tool_inv(success: bool, exit_code: Option<i32>) -> ToolInvocation {
        ToolInvocation {
            tool: "verilator".to_string(),
            argv: vec!["verilator".to_string(), "--lint-only".to_string()],
            success,
            exit_code,
            stdout_log: None,
            stderr_log: None,
            error: if success {
                None
            } else {
                Some("%Warning-WIDTH: ...".to_string())
            },
        }
    }

    /// With no tools selected, every seed is downstream-clean (vacuously), so
    /// the sweep records all-clean verdicts, zero failures, and a summary whose
    /// counts add up. This proves the loop sweeps seeds + aggregates correctly
    /// without any real downstream tool (cargo-portable).
    #[test]
    fn run_no_tool_smoke_is_all_clean() {
        let req = HuntRequest {
            base_seed: 42,
            seeds: 3,
            config: Config::default(),
            validate: test_validate_opts("no-tool-smoke"),
            minimize: false,
            max_oracle_calls: 50,
            diff_sim: false,
            bundle_root: None,
        };
        let report = run(&req).expect("hunt run");
        assert_eq!(report.lane, "dut");
        assert_eq!(report.verdicts.len(), 3);
        assert!(report.failures.is_empty());
        assert!(report.verdicts.iter().all(|v| v.ok && v.declined.is_none()));
        assert_eq!(report.summary.n_seeds, 3);
        assert_eq!(report.summary.n_clean, 3);
        assert_eq!(report.summary.n_failures, 0);
        assert_eq!(report.summary.n_declined, 0);
        assert_eq!(report.summary.n_reproduced, 0);
        // Seeds are swept consecutively from base_seed.
        let seeds: Vec<u64> = report.verdicts.iter().map(|v| v.seed).collect();
        assert_eq!(seeds, vec![42, 43, 44]);
    }

    /// The sweep stamps each position's seed into the config the generator
    /// actually seeds from — without this the "sweep" would regenerate one
    /// artifact under N run_ids (the bug `seed_config` fixes).
    #[test]
    fn seed_config_threads_the_swept_seed() {
        let req = HuntRequest {
            base_seed: 100,
            seeds: 4,
            // A profile whose own seed field is deliberately unrelated to the
            // sweep, to prove the sweep — not the profile — sets the seed.
            config: Config {
                seed: 999,
                ..Config::default()
            },
            validate: test_validate_opts("seed-thread"),
            minimize: false,
            max_oracle_calls: 50,
            diff_sim: false,
            bundle_root: None,
        };
        assert_eq!(seed_config(&req, 100).seed, 100);
        assert_eq!(seed_config(&req, 103).seed, 103);
        assert_ne!(seed_config(&req, 100).seed, seed_config(&req, 101).seed);
        // The rest of the profile is preserved verbatim (only `seed` changes).
        let base = Config {
            seed: 100,
            ..req.config.clone()
        };
        assert_eq!(
            serde_json::to_string(&seed_config(&req, 100)).unwrap(),
            serde_json::to_string(&base).unwrap(),
        );
    }

    /// The sweep is reproducible: same request ⇒ identical run_ids (content
    /// addressing), so a hunt run is itself deterministic.
    #[test]
    fn run_is_reproducible() {
        let mk = || HuntRequest {
            base_seed: 7,
            seeds: 2,
            config: Config::default(),
            validate: test_validate_opts("reproducible"),
            minimize: false,
            max_oracle_calls: 50,
            diff_sim: false,
            bundle_root: None,
        };
        let a = run(&mk()).expect("hunt run a");
        let b = run(&mk()).expect("hunt run b");
        let ids_a: Vec<&str> = a.verdicts.iter().map(|v| v.run_id.as_str()).collect();
        let ids_b: Vec<&str> = b.verdicts.iter().map(|v| v.run_id.as_str()).collect();
        assert_eq!(ids_a, ids_b);
        assert!(ids_a.iter().all(|id| !id.is_empty()));
    }

    /// A clean exit with a warning is a `warning`; a non-zero (or unknown) exit
    /// is a `reject`.
    #[test]
    fn classify_detection_distinguishes_warning_from_reject() {
        assert_eq!(classify_detection(&tool_inv(false, Some(0))), "warning");
        assert_eq!(classify_detection(&tool_inv(false, Some(1))), "reject");
        assert_eq!(classify_detection(&tool_inv(false, None)), "reject");
    }

    /// `first_failing_tool` returns the first non-success invocation.
    #[test]
    fn first_failing_tool_picks_first_non_success() {
        let tools = vec![
            tool_inv(true, Some(0)),
            tool_inv(false, Some(1)),
            tool_inv(false, Some(0)),
        ];
        let f = first_failing_tool(&tools).expect("a failing tool");
        assert_eq!(f.exit_code, Some(1));
        assert!(first_failing_tool(&[tool_inv(true, Some(0))]).is_none());
    }

    /// The report round-trips through serde (the wire contract the MCP `hunt`
    /// tool and `--introspect` will serve).
    #[test]
    fn report_serializes_and_round_trips() {
        let report = HuntReport {
            base_seed: 1,
            seeds: 1,
            lane: "dut".to_string(),
            verdicts: vec![HuntVerdict {
                seed: 1,
                run_id: "abc".to_string(),
                ok: false,
                declined: None,
            }],
            failures: vec![HuntFailure {
                seed: 1,
                run_id: "abc".to_string(),
                failing_tool: "yosys-with-abc".to_string(),
                failing_argv: vec!["yosys".to_string(), "-p".to_string()],
                first_error: Some("Warning: ...".to_string()),
                detection: "warning".to_string(),
                minimized: None,
                diff_sim: None,
                bundle: None,
            }],
            summary: HuntSummary {
                n_seeds: 1,
                n_clean: 0,
                n_failures: 1,
                n_declined: 0,
                n_reproduced: 0,
            },
        };
        let json = serde_json::to_string(&report).expect("serialize");
        let back: HuntReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.failures.len(), 1);
        assert_eq!(back.failures[0].detection, "warning");
        assert_eq!(back.summary.n_failures, 1);
        // `declined`/`first_error`/`minimized`/`diff_sim`/`bundle` use
        // skip_serializing_if, so an absent `declined`/`diff_sim`/`bundle` stays
        // absent (no `null`) in the wire form.
        assert!(!json.contains("\"declined\""));
        assert!(!json.contains("\"diff_sim\""));
        assert!(!json.contains("\"bundle\""));
    }

    /// With `diff_sim` on but no simulators present, the cross-sim check is a
    /// friendly no-op (`run_agreement` reports `ran == false`), so every
    /// validate-clean artifact stays clean — the fold never turns a clean
    /// sweep into a spurious finding. Skipped when both simulators are present
    /// (that path runs the real tools — the `tool_matrix` `#[ignore]` e2e gate
    /// and `tests/diff_sim.rs` own the present-tools case).
    #[test]
    fn diff_sim_on_clean_artifact_no_ops_without_simulators() {
        if crate::diff_sim::tools_present() {
            return;
        }
        let req = HuntRequest {
            base_seed: 5,
            seeds: 2,
            config: Config::default(),
            validate: test_validate_opts("diff-sim-noop"),
            minimize: false,
            max_oracle_calls: 50,
            diff_sim: true,
            bundle_root: None,
        };
        let report = run(&req).expect("hunt run");
        assert!(report.failures.is_empty());
        assert!(report.verdicts.iter().all(|v| v.ok));
        assert_eq!(report.summary.n_clean, 2);
        assert_eq!(report.summary.n_failures, 0);
    }

    /// `shell_quote` single-quotes a token and escapes an embedded single quote.
    #[test]
    fn shell_quote_wraps_and_escapes() {
        assert_eq!(shell_quote("repro.sv"), "'repro.sv'");
        assert_eq!(shell_quote("a b;c"), "'a b;c'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    /// A synthetic failing `(seed, cfg, ValidateReport)` exercises the bundle
    /// emitter end-to-end **without any real downstream tool**: it writes the
    /// directory, every file is present, the regenerated `repro.sv` is the real
    /// artifact, `knobs.json`/`introspection.json`/`hunt-verdict.json` round-trip,
    /// `repro.sh` regenerates + replays the failing tool against `repro.sv` (the
    /// ephemeral sandbox path substituted out), and the `HuntBundle` ref carries
    /// the `anvil://` resource URIs. Cargo-portable.
    #[test]
    fn write_bundle_emits_a_self_contained_reproducer_directory() {
        let seed = 1u64;
        let cfg = Config::default();
        let (kind, top, _sv) = generate_dut_artifact(&cfg);
        assert_eq!(kind, "module"); // default config is a combinational leaf
        let run_id = crate::introspect::content_run_id("dut", seed, &cfg);
        // A plausible (now-deleted) sandbox SV path the captured argv references.
        let sandbox = std::env::temp_dir()
            .join(format!("anvil-validate-{run_id}"))
            .display()
            .to_string();
        let sv_in_sandbox = format!("{sandbox}/{top}.sv");
        let failing = ToolInvocation {
            tool: "verilator".to_string(),
            argv: vec![
                "verilator".to_string(),
                "--lint-only".to_string(),
                sv_in_sandbox.clone(),
            ],
            success: false,
            exit_code: Some(0),
            stdout_log: None,
            stderr_log: None,
            error: Some("%Warning-WIDTH: trunc".to_string()),
        };
        let report = ValidateReport {
            run_id: run_id.clone(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: top.clone(),
            sandbox,
            tools: vec![failing.clone()],
            ok: false,
            declined: None,
        };
        let failure = HuntFailure {
            seed,
            run_id: run_id.clone(),
            failing_tool: "verilator".to_string(),
            failing_argv: failing.argv.clone(),
            first_error: failing.error.clone(),
            detection: "warning".to_string(),
            minimized: None,
            diff_sim: None,
            bundle: None,
        };

        let root = std::env::temp_dir().join("anvil-hunt-bundle-emit-test");
        let _ = std::fs::remove_dir_all(&root);
        let bundle = write_bundle(&root, seed, &cfg, &report, &failure).expect("write bundle");

        // The ref points at <root>/<run_id> with the anvil:// resource URIs.
        let dir = root.join(&run_id);
        assert_eq!(bundle.path, dir.display().to_string());
        assert_eq!(bundle.sv, format!("anvil://artifact/{run_id}/sv"));
        assert_eq!(
            bundle.introspection,
            format!("anvil://artifact/{run_id}/introspection")
        );
        assert!(bundle.manifest.is_none()); // DUT has no manifest

        // Every documented bundle file exists.
        for rel in [
            "repro.sv",
            "knobs.json",
            "introspection.json",
            "hunt-verdict.json",
            "repro.sh",
            "tool-logs/NOTE.txt",
        ] {
            assert!(dir.join(rel).exists(), "missing bundle file `{rel}`");
        }

        // repro.sv is the real regenerated artifact (deterministic from cfg):
        // byte-identical to `generate_dut_artifact` and a well-formed module.
        let repro_sv = std::fs::read_to_string(dir.join("repro.sv")).unwrap();
        assert_eq!(repro_sv, generate_dut_artifact(&cfg).2);
        assert!(repro_sv.contains(&format!("module {top}")));
        assert!(repro_sv.contains("endmodule"));

        // knobs.json round-trips to the exact effective Config.
        let knobs: Config =
            serde_json::from_str(&std::fs::read_to_string(dir.join("knobs.json")).unwrap())
                .unwrap();
        assert_eq!(
            serde_json::to_value(&knobs).unwrap(),
            serde_json::to_value(&cfg).unwrap()
        );

        // introspection.json round-trips and echoes the request.
        let doc: crate::introspect::IntrospectionDocument =
            serde_json::from_str(&std::fs::read_to_string(dir.join("introspection.json")).unwrap())
                .unwrap();
        assert_eq!(doc.request.seed, seed);
        assert_eq!(doc.request.run_id, run_id);

        // hunt-verdict.json is the finding, with the self-referential `bundle`
        // ref omitted (skip_serializing_if) — it would point back here.
        let verdict_text = std::fs::read_to_string(dir.join("hunt-verdict.json")).unwrap();
        assert!(!verdict_text.contains("\"bundle\""));
        let verdict: HuntFailure = serde_json::from_str(&verdict_text).unwrap();
        assert_eq!(verdict.detection, "warning");
        assert_eq!(verdict.failing_tool, "verilator");

        // repro.sh regenerates then replays the failing tool against repro.sv,
        // with the ephemeral sandbox path substituted away.
        let script = std::fs::read_to_string(dir.join("repro.sh")).unwrap();
        assert!(script.contains("anvil --seed 1 --config knobs.json > repro.sv"));
        assert!(script.contains("'verilator' '--lint-only' 'repro.sv'"));
        assert!(!script.contains(&sv_in_sandbox)); // the dead sandbox path is gone

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A `cross_sim_mismatch` finding has no rejecting tool, so `repro.sh` step 2
    /// points the filer at the recorded `diff_sim` excerpt instead of replaying a
    /// command. The regenerate line is still present.
    #[test]
    fn repro_script_handles_a_cross_sim_finding_with_no_failing_tool() {
        let clean = ValidateReport {
            run_id: "deadbeef".to_string(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: "mod_x".to_string(),
            sandbox: "/tmp/anvil-validate-deadbeef".to_string(),
            tools: vec![], // validate-clean: no rejecting tool to replay
            ok: true,
            declined: None,
        };
        let script = repro_script(9, &clean, "cross_sim_mismatch");
        assert!(script.contains("anvil --seed 9 --config knobs.json > repro.sv"));
        assert!(script.contains("cross-simulator mismatch"));
        assert!(script.contains("hunt-verdict.json"));
    }

    /// With `bundle_root` set but no tools selected, the sweep is all-clean, so
    /// no finding fires and **no bundle directory is written** — `bundle_root`
    /// never perturbs a clean run. Cargo-portable.
    #[test]
    fn bundle_root_writes_nothing_on_a_clean_sweep() {
        let root = std::env::temp_dir().join("anvil-hunt-bundle-clean-test");
        let _ = std::fs::remove_dir_all(&root);
        let req = HuntRequest {
            base_seed: 70,
            seeds: 2,
            config: Config::default(),
            validate: test_validate_opts("bundle-clean"),
            minimize: false,
            max_oracle_calls: 50,
            diff_sim: false,
            bundle_root: Some(root.clone()),
        };
        let report = run(&req).expect("hunt run");
        assert!(report.failures.is_empty());
        assert!(report.failures.iter().all(|f| f.bundle.is_none()));
        // No finding ⇒ the bundle root was never created.
        assert!(!root.exists());
    }
}
