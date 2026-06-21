//! DOWNSTREAM-ADAPTER-EXPANSION.2c — the real-tool end-to-end gate for the
//! `slang` downstream adapter (decision `0020`), plus a portable public-API proof.
//!
//! Structured exactly like `tests/sv2v_e2e.rs` / `tests/divergence_e2e.rs`: the
//! real-tool proof is `#[ignore]` + tool-gated, so a plain `cargo test` stays
//! green on a host without `slang` (the common case — `slang` is absent on most
//! machines), while `cargo test --test slang_e2e -- --ignored` (with `slang` on
//! PATH) exercises the whole `validate` → `SlangAdapter::run` → real `slang` path.
//!
//! ## What this gate proves
//!
//! 1. **`slang` is a public, selectable, discoverable, *fact-bearing* adapter
//!    (portable).** Built from the *public* [`anvil::downstream`] surface — exactly
//!    as an external crate user or an MCP agent would — `slang` parses off the
//!    allow-list ([`anvil::downstream::AcceptanceTool::from_name`]) and appears in
//!    the adapter catalog ([`anvil::downstream::adapter_catalog`]) as the first
//!    entry with `supports_facts = true` (the `extract_facts` JSON-AST hook landed
//!    at `.2c.1`). Needs no tool, so it always runs.
//! 2. **Real `slang` accepts ANVIL's valid-by-construction RTL (tool-gated).** On
//!    a default DUT seed, `slang` elaborates the emitted SystemVerilog cleanly — a
//!    real `slang` *rejection* here would be a candidate **downstream-tool bug**,
//!    the very thing this lane exists to surface, not a fixture we can fabricate.
//!    Skips green when `slang` is absent. (Surfacing the extracted `--ast-json`
//!    facts in the `tool_matrix` report is `.2c.2b`; this gate proves acceptance.)

use anvil::config::Config;
use anvil::downstream::{
    self, adapter_catalog, validate, AcceptanceTool, AdapterRunCx, AdapterTarget, ValidateOptions,
};
use std::path::{Path, PathBuf};

/// Portable: the public downstream API exposes `slang` as a fifth registered,
/// allow-listed adapter — selectable + discoverable with no tool installed — and
/// the first one that advertises richer structured facts.
#[test]
fn slang_is_a_public_fact_bearing_adapter() {
    assert_eq!(
        AcceptanceTool::from_name("slang"),
        Some(AcceptanceTool::Slang)
    );
    assert_eq!(AcceptanceTool::Slang.binary(), "slang");

    let catalog = adapter_catalog();
    let slang = catalog
        .iter()
        .find(|a| a.id == "slang")
        .expect("slang must be in the adapter catalog");
    assert_eq!(slang.binary, "slang");
    // slang is the first fact-bearing adapter — the `extract_facts` JSON-AST hook.
    assert!(slang.supports_facts);
}

/// Tool-gated: with `slang` on PATH, it elaborates ANVIL's valid-by-construction
/// DUT output cleanly (accepts) **and** the `.2c.2b` `extract_facts` hook projects
/// real `slang --ast-json` into `AdapterFacts` (a named top with at least one
/// port). This is the eventual real-tool confirmation that the parser written
/// against slang's published schema (slang was absent at landing) matches actual
/// slang output. Skips green when `slang` is absent — the friendly no-op precedent
/// (`tool_version` probe); the portable test above always runs.
#[test]
#[ignore = "requires slang on PATH"]
fn slang_accepts_anvil_dut_output_and_extracts_facts_end_to_end() {
    if downstream::tool_version("slang").is_none() {
        eprintln!("slang not on PATH; skipping the real-tool gate (portable proof still ran)");
        return;
    }
    let cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let opts = ValidateOptions {
        tools: vec![AcceptanceTool::Slang],
        sandbox_root: std::env::temp_dir().join("anvil-slang-e2e"),
        // Keep the sandbox so the `<top>.slang.json` side file `run_slang` wrote
        // survives for the `extract_facts` projection below.
        keep_sandbox: true,
        ..Default::default()
    };
    let report = validate(42, &cfg, &opts).unwrap();
    assert!(report.declined.is_none());
    assert!(
        report.ok,
        "slang must accept ANVIL's valid-by-construction RTL by construction: {report:?}"
    );
    let inv = report
        .tools
        .iter()
        .find(|t| t.tool == "slang")
        .expect("the slang invocation must be recorded")
        .clone();

    // Project the kept-sandbox `<top>.slang.json` through the public adapter hook —
    // exactly the path `tool_matrix --slang` (`.2c.2b`) drives.
    let sandbox = PathBuf::from(&report.sandbox);
    let sv_path = sandbox.join(format!("{}.sv", report.top));
    let target = if report.kind == "design" {
        AdapterTarget::Design {
            sv_paths: std::slice::from_ref(&sv_path),
            top: &report.top,
        }
    } else {
        AdapterTarget::Module {
            sv_path: &sv_path,
            stem: &report.top,
        }
    };
    let cx = AdapterRunCx {
        binary: "slang",
        out_dir: &sandbox,
        target,
        yosys_mode: anvil::downstream::YosysMode::WithoutAbc,
        language: None,
    };
    let facts = AcceptanceTool::Slang
        .adapter()
        .extract_facts(&cx, &inv)
        .expect("real slang --ast-json must project into AdapterFacts");
    assert_eq!(facts.adapter, "slang");
    assert!(!facts.top.is_empty(), "facts must name the elaborated top");
    assert!(
        !facts.ports.is_empty(),
        "an ANVIL DUT module has ports, so the AST projection must too: {facts:?}"
    );

    // Clean up the kept sandbox so the test leaves no residue.
    let _ = std::fs::remove_dir_all::<&Path>(sandbox.as_ref());
}
