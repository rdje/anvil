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
use crate::downstream::{
    minimize, validate, KnobReduction, MinimizeOptions, MinimizeReport, ToolInvocation,
    ValidateOptions,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    /// `"reject"` (non-zero exit) or `"warning"` (clean exit, warning folded
    /// into `success == false`). `"cross_sim_mismatch"` is added by `.2b.2`.
    pub detection: String,
    /// The auto-minimize result, when `HuntRequest::minimize` was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimized: Option<HuntMinimized>,
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
        let report = validate(seed, &req.config, &req.validate)?;
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

        let minimized = if req.minimize {
            let mopts = MinimizeOptions {
                validate: req.validate.clone(),
                max_oracle_calls: req.max_oracle_calls,
            };
            let m = minimize(seed, &req.config, &mopts)?;
            Some(HuntMinimized::from_report(&m, &run_id))
        } else {
            None
        };

        failures.push(HuntFailure {
            seed,
            run_id,
            failing_tool,
            failing_argv,
            first_error,
            detection,
            minimized,
        });
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
        // `declined`/`first_error`/`minimized` use skip_serializing_if, so an
        // absent `declined` stays absent (no `null`) in the wire form.
        assert!(!json.contains("\"declined\""));
    }
}
