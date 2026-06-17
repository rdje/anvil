//! ACCEPTANCE-DIVERGENCE-HUNTING — the acceptance-divergence detector.
//!
//! Implements the design from decision record
//! `docs/decisions/0019-acceptance-divergence-hunting.md`: a **default-off,
//! `SCHEMA-DERIVED` detector** for where two independent downstream tools (or,
//! at `.2e`, two versions of one tool) **disagree** on whether a
//! valid-by-construction artifact is legal — one accepts, another warns or
//! rejects. On valid-by-construction RTL every such disagreement is a
//! downstream-tool bug, not an RTL fault, which is exactly the north star
//! (`project_anvil_north_star`).
//!
//! ## A composer, not a new engine
//!
//! [`run`] composes the **one hardened orchestration**
//! [`crate::downstream::validate`] (which already generates the DUT into a
//! sandbox and runs *every* enabled tool/mode to completion — it does **not**
//! short-circuit on the first reject; only the `MemGuard` can decline before a
//! spawn) and projects its per-tool [`crate::downstream::ToolInvocation`] rows
//! into accept/warn/reject verdicts via the shared
//! [`crate::downstream::tool_verdict`] classifier
//! (`ACCEPTANCE-DIVERGENCE-HUNTING.2a`). It adds **no** generator path, **no**
//! second sandbox loop, **no** behavioural oracle, and **no** second classifier
//! — the full-factorization doctrine and decision `0004`'s
//! no-shadow-simulator ceiling. [`DivergenceOptions`] wraps [`ValidateOptions`]
//! exactly as `MinimizeOptions` does, so the one allow-list / sandbox /
//! RAM-guard / audit discipline is inherited unchanged; the
//! tool-version-vs-version axis (`.2e`) extends this struct.
//!
//! ## What this leaf (`.2b`) is and is not
//!
//! This is the **library core**: the [`ToolDecision`] / [`Divergence`] /
//! [`DivergenceReport`] / [`DivergenceOptions`] types + [`run`] with **multi-tool
//! same-version** divergence classification. Folding the detector into the
//! `hunt` loop and adding the `tool_matrix` column is `.2c`; the MCP `divergence`
//! controlled tool + the CLI shim is `.2d`; the tool-version-vs-version axis is
//! `.2e`. Default `anvil` build / `--artifact dut` stays byte-identical.
//!
//! Naming note: the accept/warn/reject *enum* is
//! [`crate::downstream::ToolVerdict`] (landed by `.2a`, reused here); the
//! per-tool *record* that pairs one labelled tool with its verdict is
//! [`ToolDecision`] (the decision ADR sketched it as a `ToolVerdict { tool, … }`
//! record — renamed here to avoid clashing with the enum).

use crate::config::Config;
use crate::downstream::{
    tool_verdict, validate, ToolInvocation, ToolVerdict, ValidateOptions, ValidateReport,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Options for a divergence run. Wraps [`ValidateOptions`] (the `MinimizeOptions`
/// precedent) so the one hardened `validate` orchestration — the
/// `AcceptanceTool` allow-list, the Yosys mode, the memory ceilings, and the
/// **caller-set** sandbox root (never agent-supplied, decision `0004`) — is
/// reused verbatim. The tool-version-vs-version axis (`.2e`) extends this struct
/// with explicit per-version tool specs.
#[derive(Debug, Clone, Default)]
pub struct DivergenceOptions {
    /// The downstream-run configuration (tool allow-list, Yosys mode, memory
    /// limits, sandbox root, keep-sandbox). `>= 2` labelled tools must run for a
    /// divergence to be possible (Yosys `both` alone yields two labels).
    pub validate: ValidateOptions,
}

/// One labelled tool's acceptance verdict on the artifact — a `SCHEMA-DERIVED`
/// projection of one [`ToolInvocation`] (no new computed truth). The label is
/// the tool's own report name (`"verilator"` / `"yosys-without-abc"` /
/// `"yosys-with-abc"` / `"iverilog"`), so Yosys `both` contributes two rows and a
/// without-abc-vs-with-abc disagreement is itself a divergence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDecision {
    /// The labelled tool (the `ToolInvocation.tool` report name).
    pub tool: String,
    /// The accept/warn/reject verdict (shared `downstream::tool_verdict`).
    pub verdict: ToolVerdict,
    /// The tool's exit code (`None` on a spawn failure).
    pub exit_code: Option<i32>,
    /// The first reject/warning line (`ToolInvocation.error`), when not clean.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
}

/// A classified disagreement among the labelled tools' verdicts. ANVIL only
/// *classifies* the disagreement — the tools' own verdicts are the source of
/// truth, there is no adjudication (decision `0004`, ROADMAP steering gap 4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Divergence {
    /// `"accept_reject"` | `"accept_warn"` | `"warn_reject"` (the `.2e`
    /// tool-version axis adds `"version_mismatch"`).
    pub kind: String,
    /// The labelled tools holding either of the two differing verdicts, sorted +
    /// deduped for determinism.
    pub tools: Vec<String>,
}

/// The result of a divergence run over one artifact. Every field is a
/// `SCHEMA-DERIVED` projection of [`ValidateReport`](crate::downstream::ValidateReport)
/// / [`ToolInvocation`] — no new computed truth, no behavioural oracle. Lives
/// beside [`DiffSimReport`](crate::diff_sim::DiffSimReport) and
/// [`HuntReport`](crate::hunt::HuntReport).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    /// Content-addressed `(seed, knobs)` id (reproducible).
    pub run_id: String,
    pub lane: String,
    /// `"design"` or `"module"`.
    pub kind: String,
    pub top: String,
    /// The sandbox the artifact was written to (and tools ran in).
    pub sandbox: String,
    /// One verdict per labelled tool, in run order.
    pub verdicts: Vec<ToolDecision>,
    /// `true` iff at least one [`Divergence`] was found.
    pub diverged: bool,
    /// The classified disagreements (empty when all tools agree — the
    /// valid-by-construction steady state).
    pub divergences: Vec<Divergence>,
    /// Set when the run stopped before all tools ran (the `MemGuard` declined a
    /// spawn); the verdict set is then partial and `diverged` reflects only what
    /// ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declined: Option<String>,
}

/// Detect acceptance divergence on the DUT artifact for `(seed, cfg)` across the
/// tools in `opts.validate.tools`.
///
/// Composes [`crate::downstream::validate`] (one hardened orchestration: generate
/// → sandbox → run every enabled tool/mode to completion → cleanup) and projects
/// its per-tool invocations into accept/warn/reject verdicts, then classifies any
/// disagreement. Reproducible + sandboxed + RAM-guarded + audit-logged by
/// inheritance (decision `0004`). Default-off everywhere it is surfaced; this
/// function changes no emitted RTL.
pub fn run(seed: u64, cfg: &Config, opts: &DivergenceOptions) -> Result<DivergenceReport> {
    let report = validate(seed, cfg, &opts.validate)?;
    Ok(classify_report(&report))
}

/// Classify an already-run [`ValidateReport`] into a [`DivergenceReport`] — the
/// pure projection half of [`run`] (it runs **no** tool). Reused by the `hunt`
/// loop (`ACCEPTANCE-DIVERGENCE-HUNTING.2c`), which classifies the tools
/// [`crate::downstream::validate`] already ran on a finding rather than
/// re-validating — so the one orchestration runs once. Every field is a
/// `SCHEMA-DERIVED` projection of the report (`run_id` / `lane` / `kind` / `top`
/// / `sandbox` / `declined` carried through; `verdicts` projected per tool;
/// `divergences` classified).
pub fn classify_report(report: &ValidateReport) -> DivergenceReport {
    let verdicts: Vec<ToolDecision> = report.tools.iter().map(to_decision).collect();
    let divergences = classify_divergences(&verdicts);
    DivergenceReport {
        run_id: report.run_id.clone(),
        lane: report.lane.clone(),
        kind: report.kind.clone(),
        top: report.top.clone(),
        sandbox: report.sandbox.clone(),
        diverged: !divergences.is_empty(),
        divergences,
        verdicts,
        declined: report.declined.clone(),
    }
}

/// Project one [`ToolInvocation`] into its [`ToolDecision`] via the shared
/// `downstream::tool_verdict` classifier.
fn to_decision(inv: &ToolInvocation) -> ToolDecision {
    ToolDecision {
        tool: inv.tool.clone(),
        verdict: tool_verdict(inv),
        exit_code: inv.exit_code,
        first_message: inv.error.clone(),
    }
}

/// Classify the disagreements among a set of per-tool verdicts. A divergence
/// exists for each *pair of distinct verdict values both present*: accept-vs-
/// reject, accept-vs-warn, warn-vs-reject. Up to all three can co-occur (when all
/// three verdict values are present). Output order is fixed and the tool lists
/// are sorted, so the result is deterministic (no hash-map iteration —
/// reproducibility contract).
fn classify_divergences(verdicts: &[ToolDecision]) -> Vec<Divergence> {
    let tools_with = |want: ToolVerdict| -> Vec<String> {
        verdicts
            .iter()
            .filter(|d| d.verdict == want)
            .map(|d| d.tool.clone())
            .collect()
    };
    let accepts = tools_with(ToolVerdict::Accept);
    let warns = tools_with(ToolVerdict::Warn);
    let rejects = tools_with(ToolVerdict::Reject);

    let mk = |kind: &str, a: &[String], b: &[String]| -> Option<Divergence> {
        if a.is_empty() || b.is_empty() {
            return None;
        }
        let mut tools: Vec<String> = a.iter().chain(b.iter()).cloned().collect();
        tools.sort();
        tools.dedup();
        Some(Divergence {
            kind: kind.to_string(),
            tools,
        })
    };

    [
        mk("accept_reject", &accepts, &rejects),
        mk("accept_warn", &accepts, &warns),
        mk("warn_reject", &warns, &rejects),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inv(tool: &str, success: bool, exit_code: Option<i32>) -> ToolInvocation {
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
        }
    }

    /// A unique sandbox root per test so parallel runs never collide.
    fn opts(tag: &str) -> DivergenceOptions {
        DivergenceOptions {
            validate: ValidateOptions {
                tools: vec![], // no-tool smoke: generate + sandbox only, no real tools
                sandbox_root: std::env::temp_dir().join(format!("anvil-divergence-test-{tag}")),
                ..ValidateOptions::default()
            },
        }
    }

    /// `to_decision` projects a `ToolInvocation` into the labelled accept/warn/
    /// reject record, carrying the label, the shared verdict, the exit code, and
    /// the first message.
    #[test]
    fn to_decision_projects_a_tool_invocation() {
        let d = to_decision(&inv("verilator", true, Some(0)));
        assert_eq!(d.tool, "verilator");
        assert_eq!(d.verdict, ToolVerdict::Accept);
        assert_eq!(d.exit_code, Some(0));
        assert_eq!(d.first_message, None);

        let r = to_decision(&inv("yosys-without-abc", false, Some(1)));
        assert_eq!(r.verdict, ToolVerdict::Reject);
        assert_eq!(
            r.first_message.as_deref(),
            Some("yosys-without-abc: not clean")
        );
    }

    /// One tool accepts, another rejects ⇒ a single `accept_reject` divergence
    /// naming both tools (the headline signal). This is the synthetic
    /// accept/reject set the design ADR (`.1`) requires the detector to classify.
    #[test]
    fn classify_flags_accept_reject() {
        let verdicts = vec![
            to_decision(&inv("verilator", true, Some(0))),
            to_decision(&inv("yosys-without-abc", false, Some(1))),
        ];
        let divs = classify_divergences(&verdicts);
        assert_eq!(divs.len(), 1);
        assert_eq!(divs[0].kind, "accept_reject");
        assert_eq!(divs[0].tools, vec!["verilator", "yosys-without-abc"]);
    }

    /// All tools agree (all accept) ⇒ no divergence — the valid-by-construction
    /// steady state.
    #[test]
    fn classify_all_agree_is_no_divergence() {
        let verdicts = vec![
            to_decision(&inv("verilator", true, Some(0))),
            to_decision(&inv("yosys-without-abc", true, Some(0))),
            to_decision(&inv("iverilog", true, Some(0))),
        ];
        assert!(classify_divergences(&verdicts).is_empty());
    }

    /// Accept + warn + reject all present ⇒ all three pair-classes diverge, in
    /// the fixed order `accept_reject`, `accept_warn`, `warn_reject`.
    #[test]
    fn classify_flags_all_three_pair_classes() {
        let verdicts = vec![
            to_decision(&inv("verilator", true, Some(0))), // accept
            to_decision(&inv("yosys-without-abc", false, Some(0))), // warn (clean exit, !success)
            to_decision(&inv("yosys-with-abc", false, Some(2))), // reject
        ];
        let divs = classify_divergences(&verdicts);
        let kinds: Vec<&str> = divs.iter().map(|d| d.kind.as_str()).collect();
        assert_eq!(kinds, vec!["accept_reject", "accept_warn", "warn_reject"]);
    }

    /// A without-abc-vs-with-abc disagreement is a first-class divergence (the
    /// two Yosys modes are distinct labelled tools).
    #[test]
    fn classify_distinguishes_yosys_modes() {
        let verdicts = vec![
            to_decision(&inv("yosys-without-abc", true, Some(0))),
            to_decision(&inv("yosys-with-abc", false, Some(0))), // warn
        ];
        let divs = classify_divergences(&verdicts);
        assert_eq!(divs.len(), 1);
        assert_eq!(divs[0].kind, "accept_warn");
        assert_eq!(divs[0].tools, vec!["yosys-with-abc", "yosys-without-abc"]);
    }

    /// `run` with no tools selected generates + sandboxes but spawns nothing, so
    /// the verdict set is empty and nothing diverges — a friendly no-op
    /// (cargo-portable; needs no real downstream tool). The `run_id` is the
    /// content-addressed id, so the run is reproducible.
    #[test]
    fn run_no_tool_smoke_is_a_friendly_no_op() {
        let report = run(0, &Config::default(), &opts("no-tool-smoke")).expect("divergence run");
        assert_eq!(report.lane, "dut");
        assert!(report.verdicts.is_empty());
        assert!(!report.diverged);
        assert!(report.divergences.is_empty());
        assert!(report.declined.is_none());
        assert!(!report.run_id.is_empty());
    }

    /// `classify_report` projects an already-run `ValidateReport` (the tools the
    /// hunt loop already ran) into a `DivergenceReport` without re-validating —
    /// carrying run_id/lane/kind/top/sandbox/declined through and classifying the
    /// per-tool verdicts. An accept+reject report ⇒ `accept_reject`.
    #[test]
    fn classify_report_projects_a_validate_report() {
        let report = ValidateReport {
            run_id: "rid".to_string(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: "m".to_string(),
            sandbox: "/tmp/s".to_string(),
            tools: vec![
                inv("verilator", true, Some(0)),
                inv("yosys-without-abc", false, Some(1)),
            ],
            ok: false,
            declined: None,
        };
        let dr = classify_report(&report);
        assert_eq!(dr.run_id, "rid");
        assert_eq!(dr.top, "m");
        assert!(dr.diverged);
        assert_eq!(dr.divergences.len(), 1);
        assert_eq!(dr.divergences[0].kind, "accept_reject");
        assert_eq!(dr.verdicts.len(), 2);
    }

    /// The report serialises and round-trips; absent optional fields stay off the
    /// wire (`skip_serializing_if`).
    #[test]
    fn report_serde_round_trips() {
        let report = DivergenceReport {
            run_id: "abc".to_string(),
            lane: "dut".to_string(),
            kind: "module".to_string(),
            top: "m".to_string(),
            sandbox: "/tmp/x".to_string(),
            verdicts: vec![to_decision(&inv("verilator", true, Some(0)))],
            diverged: false,
            divergences: vec![],
            declined: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        // The verdict's clean `first_message` and the report's `declined` are absent.
        assert!(!json.contains("first_message"));
        assert!(!json.contains("declined"));
        // The verdict enum uses the snake_case wire form.
        assert!(json.contains("\"accept\""));
        let back: DivergenceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.verdicts, report.verdicts);
    }
}
