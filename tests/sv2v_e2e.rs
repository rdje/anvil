//! DOWNSTREAM-ADAPTER-EXPANSION.2b — the real-tool end-to-end gate for the
//! `sv2v` downstream adapter (decision `0020`), plus a portable public-API proof.
//!
//! Structured exactly like `tests/divergence_e2e.rs` / `tests/hunt_e2e.rs`: the
//! real-tool proof is `#[ignore]` + tool-gated, so a plain `cargo test` stays
//! green on a host without `sv2v` (the common case — `sv2v` is absent on most
//! machines), while `cargo test --test sv2v_e2e -- --ignored` (with `sv2v` on
//! PATH, e.g. `brew install sv2v`) exercises the whole
//! `validate` → `Sv2vAdapter::run` → real `sv2v` path.
//!
//! ## What this gate proves
//!
//! 1. **`sv2v` is a public, selectable, discoverable adapter (portable).** Built
//!    from the *public* [`anvil::downstream`] surface — exactly as an external
//!    crate user or an MCP agent would — `sv2v` parses off the allow-list
//!    ([`anvil::downstream::AcceptanceTool::from_name`]) and appears in the
//!    adapter catalog ([`anvil::downstream::adapter_catalog`]). Needs no tool, so
//!    it always runs.
//! 2. **Real `sv2v` accepts ANVIL's valid-by-construction RTL (tool-gated).** On
//!    a default DUT seed, `sv2v` transpiles the emitted SystemVerilog cleanly — a
//!    real `sv2v` *rejection* here would be a candidate **downstream-tool bug**,
//!    the very thing this lane exists to surface, not a fixture we can fabricate
//!    (that would mean emitting illegal RTL, which the project forbids). Skips
//!    green when `sv2v` is absent.

use anvil::config::Config;
use anvil::downstream::{self, adapter_catalog, validate, AcceptanceTool, ValidateOptions};

/// Portable: the public downstream API exposes `sv2v` as a fourth registered,
/// allow-listed adapter — selectable + discoverable with no tool installed.
#[test]
fn sv2v_is_a_public_selectable_adapter() {
    assert_eq!(
        AcceptanceTool::from_name("sv2v"),
        Some(AcceptanceTool::Sv2v)
    );
    assert_eq!(AcceptanceTool::Sv2v.binary(), "sv2v");

    let catalog = adapter_catalog();
    let sv2v = catalog
        .iter()
        .find(|a| a.id == "sv2v")
        .expect("sv2v must be in the adapter catalog");
    assert_eq!(sv2v.binary, "sv2v");
    // sv2v is the minimal accept/reject column — no richer fact hook (that is slang, .2c).
    assert!(!sv2v.supports_facts);
}

/// Tool-gated: with `sv2v` on PATH, it transpiles ANVIL's valid-by-construction
/// DUT output cleanly (accepts). Skips green when `sv2v` is absent — the friendly
/// no-op precedent (`tools_present()`); the portable test above always runs.
#[test]
#[ignore = "requires sv2v on PATH"]
fn sv2v_accepts_anvil_dut_output_end_to_end() {
    if downstream::tool_version("sv2v").is_none() {
        eprintln!("sv2v not on PATH; skipping the real-tool gate (portable proof still ran)");
        return;
    }
    let cfg = Config {
        seed: 42,
        ..Config::default()
    };
    let opts = ValidateOptions {
        tools: vec![AcceptanceTool::Sv2v],
        sandbox_root: std::env::temp_dir().join("anvil-sv2v-e2e"),
        ..Default::default()
    };
    let report = validate(42, &cfg, &opts).unwrap();
    assert!(report.declined.is_none());
    assert!(
        report.ok,
        "sv2v must accept ANVIL's valid-by-construction RTL by construction: {report:?}"
    );
    assert!(
        report.tools.iter().any(|t| t.tool == "sv2v"),
        "the sv2v invocation must be recorded"
    );
}
