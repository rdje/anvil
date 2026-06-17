//! ACCEPTANCE-DIVERGENCE-HUNTING.2f — the real-tool end-to-end gate for the
//! acceptance-divergence detector (decision `0019`), plus the portable
//! synthetic-classification proof over the *public* API surface.
//!
//! Structured exactly like `tests/hunt_e2e.rs` / `tests/diff_sim.rs`: the
//! real-tool proofs are `#[ignore]` + tool-gated, so a plain `cargo test` stays
//! green on a tool-less host, while `cargo test --test divergence_e2e --
//! --ignored` (with Verilator — and optionally Yosys — installed) exercises the
//! whole `divergence::run` → `downstream::validate` → real-tool path and the
//! `anvil hunt --divergence` CLI surface.
//!
//! ## What this gate proves
//!
//! 1. **The classifier is correct over the public API (portable).** A
//!    synthetic-injected accept/reject pair — built from the public
//!    [`anvil::downstream::ValidateReport`] / [`anvil::downstream::ToolInvocation`]
//!    and classified by the public [`anvil::divergence::classify_report`], i.e.
//!    exactly as an external crate user would — is classified `accept_reject` and
//!    the resulting [`anvil::divergence::DivergenceReport`] is queryable
//!    (serde-round-trips). This is the one outcome we *can* manufacture without a
//!    real tool, and it needs no tool, so it always runs.
//! 2. **An all-agree real-tool run records `diverged=false` and is queryable.**
//!    On valid-by-construction RTL every enabled tool accepts, so the steady
//!    state is agreement. A genuine divergence here would be a candidate
//!    **downstream-tool bug** — the very thing this lane exists to surface — not
//!    a fixture we can fabricate (fabricating one would mean emitting illegal
//!    RTL, which the project forbids). So the real-tool gate proves the steady
//!    state end-to-end: the matrix of per-tool verdicts is produced, every tool
//!    agrees, `diverged == false`, and the report serialises for querying.
//! 3. **The `anvil hunt --divergence` CLI surface is inert on a clean sweep.**
//!    The user-facing divergence axis drives the real loop and, on a clean
//!    valid-by-construction sweep, surfaces no `acceptance_divergence` finding.

use std::path::PathBuf;
use std::process::Command;

use anvil::config::Config;
use anvil::divergence::{self, DivergenceOptions, DivergenceReport};
use anvil::downstream::{
    AcceptanceTool, ToolInvocation, ToolVerdict, ValidateOptions, ValidateReport, YosysMode,
};

/// Cheap presence probe: the binary spawns at all (`--version` may exit
/// non-zero, but a spawn failure means the tool is absent). Mirrors
/// `hunt_e2e::tool_present` / `diff_sim::tools_present`.
fn tool_present(bin: &str) -> bool {
    Command::new(bin).arg("--version").output().is_ok()
}

/// The freshly-built `anvil` binary under test (Cargo sets this for integration
/// tests), so the e2e gate drives the real CLI, not a library re-entry.
fn anvil() -> &'static str {
    env!("CARGO_BIN_EXE_anvil")
}

/// A fresh per-test sandbox root under Cargo's integration-test tmp dir.
fn sandbox_root(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create e2e sandbox root");
    dir
}

/// Build a synthetic per-tool invocation through the public `ToolInvocation`
/// shape: `accept` ⇒ a clean success; otherwise a non-zero-exit reject.
fn inv(tool: &str, accept: bool) -> ToolInvocation {
    ToolInvocation {
        tool: tool.to_string(),
        argv: vec![tool.to_string()],
        success: accept,
        exit_code: Some(if accept { 0 } else { 1 }),
        stdout_log: None,
        stderr_log: None,
        error: if accept {
            None
        } else {
            Some(format!("{tool}: rejected"))
        },
        version: None,
    }
}

/// (1) Portable, no tool required: a synthetic-injected accept/reject pair —
/// assembled and classified entirely through ANVIL's *public* API, exactly as an
/// external consumer would — is classified `accept_reject`, and the resulting
/// `DivergenceReport` is queryable (serde-round-trips with the divergence
/// preserved). This is the manufacturable half of the `.2f` acceptance criteria
/// and the integration-surface mirror of the in-crate
/// `divergence::classify_report_projects_a_validate_report` unit proof.
#[test]
fn injected_accept_reject_pair_classifies_accept_reject() {
    // One tool accepts, another rejects the *same* valid-by-construction artifact.
    let report = ValidateReport {
        run_id: "synthetic-rid".to_string(),
        lane: "dut".to_string(),
        kind: "module".to_string(),
        top: "m".to_string(),
        sandbox: "/tmp/anvil-divergence-e2e-synthetic".to_string(),
        tools: vec![inv("verilator", true), inv("yosys-without-abc", false)],
        ok: false,
        declined: None,
    };

    let dr: DivergenceReport = divergence::classify_report(&report);

    assert!(dr.diverged, "an accept+reject pair must diverge");
    assert_eq!(dr.divergences.len(), 1, "exactly one disagreement");
    assert_eq!(dr.divergences[0].kind, "accept_reject");
    assert_eq!(
        dr.divergences[0].tools,
        vec!["verilator", "yosys-without-abc"],
        "the divergence names both disagreeing tools, sorted"
    );
    assert_eq!(dr.verdicts.len(), 2);

    // Queryable: the report serialises and round-trips with the divergence intact.
    let json = serde_json::to_string(&dr).expect("DivergenceReport serialises");
    assert!(json.contains("accept_reject"));
    let back: DivergenceReport = serde_json::from_str(&json).expect("round-trips");
    assert!(back.diverged);
    assert_eq!(back.divergences[0].kind, "accept_reject");
}

/// (2) Real-tool, `#[ignore]`: an all-agree run over real downstream tools
/// records `diverged=false`, produces the full per-tool verdict matrix, and is
/// queryable. Swept over several seeds (distinct content-addressed `run_id`s) so
/// the agreement holds across varied valid-by-construction artifacts, not one.
#[test]
#[ignore]
fn divergence_all_agree_against_real_tools() {
    if !tool_present("verilator") {
        eprintln!(
            "divergence_all_agree_against_real_tools: verilator not on $PATH \
             (skipping; rerun with it installed for the real-tool divergence gate)"
        );
        return;
    }

    // Enable Verilator, and Yosys (both ABC modes ⇒ two more labelled tools) when
    // present — so this is a genuine cross-tool agreement, not a single verdict.
    let mut tools = vec![AcceptanceTool::Verilator];
    let mut yosys_mode = YosysMode::WithoutAbc;
    if tool_present("yosys") {
        tools.push(AcceptanceTool::Yosys);
        yosys_mode = YosysMode::Both;
    }
    let expected_labels = if tool_present("yosys") { 3 } else { 1 };

    let opts = DivergenceOptions {
        validate: ValidateOptions {
            tools,
            yosys_mode,
            sandbox_root: sandbox_root("divergence-all-agree"),
            ..ValidateOptions::default()
        },
        tool_specs: vec![],
    };

    let mut run_ids = std::collections::BTreeSet::new();
    for seed in [1u64, 2, 3] {
        // run_id + artifact are both keyed on cfg.seed == seed (the hunt-sweep
        // convention); struct-update keeps clippy's field_reassign_with_default quiet.
        let cfg = Config {
            seed,
            ..Config::default()
        };

        let report = divergence::run(seed, &cfg, &opts).expect("divergence run");

        assert_eq!(report.lane, "dut");
        assert_eq!(
            report.verdicts.len(),
            expected_labels,
            "seed {seed}: every enabled labelled tool reports a verdict"
        );
        // Valid-by-construction ⇒ every tool accepts ⇒ no divergence.
        assert!(
            report
                .verdicts
                .iter()
                .all(|d| d.verdict == ToolVerdict::Accept),
            "seed {seed}: a non-accept verdict is a candidate downstream-tool bug: {:?}",
            report.verdicts
        );
        assert!(
            !report.diverged && report.divergences.is_empty(),
            "seed {seed}: all-agree is the valid-by-construction steady state, got {:?}",
            report.divergences
        );
        assert!(report.declined.is_none(), "seed {seed}: nothing declined");
        assert!(!report.run_id.is_empty());

        // Queryable: the real-run report serialises and round-trips.
        let json = serde_json::to_string(&report).expect("serialises");
        let back: DivergenceReport = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(back.diverged, report.diverged);
        assert_eq!(back.verdicts.len(), report.verdicts.len());

        run_ids.insert(report.run_id);
    }
    assert_eq!(
        run_ids.len(),
        3,
        "each seed must content-address distinctly"
    );
    eprintln!(
        "divergence_all_agree_against_real_tools: clean 3-seed all-agree sweep \
         ({expected_labels} tools/seed), run_ids={run_ids:?}"
    );
}

/// (3) Real-tool, `#[ignore]`: the user-facing `anvil hunt --divergence` CLI
/// drives the loop end-to-end against real Verilator and, on a clean
/// valid-by-construction sweep, surfaces no `acceptance_divergence` finding — the
/// divergence axis is inert on the steady state (a finding here would be a
/// candidate downstream-tool bug). Proves the CLI shim over the same detector.
#[test]
#[ignore]
fn hunt_divergence_axis_clean_against_real_verilator() {
    if !tool_present("verilator") {
        eprintln!(
            "hunt_divergence_axis_clean_against_real_verilator: verilator not on $PATH (skipping)"
        );
        return;
    }
    let out = Command::new(anvil())
        .args([
            "hunt",
            "--seed",
            "1",
            "--seeds",
            "3",
            "--tools",
            "verilator",
            "--divergence",
        ])
        .output()
        .expect("run anvil hunt --divergence");
    assert!(
        out.status.success(),
        "anvil hunt --divergence exited non-zero:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: anvil::hunt::HuntReport =
        serde_json::from_slice(&out.stdout).expect("anvil hunt must print a HuntReport JSON");

    assert_eq!(report.lane, "dut");
    assert_eq!(report.seeds, 3);
    // Clean sweep ⇒ no findings at all, hence no acceptance_divergence finding.
    assert_eq!(
        report.summary.n_failures, 0,
        "ANVIL output should be downstream-clean; a finding here is a candidate \
         downstream-tool bug. failures={:?}",
        report.failures
    );
    assert!(
        report
            .failures
            .iter()
            .all(|f| f.detection != "acceptance_divergence"),
        "no acceptance_divergence finding on a clean sweep: {:?}",
        report.failures
    );
    eprintln!(
        "hunt_divergence_axis_clean_against_real_verilator: divergence axis inert on a clean sweep"
    );
}
