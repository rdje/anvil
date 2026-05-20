//! PHASE-7-ORACLE-MICRODESIGN.2c.1 — parity harness for the
//! oracle-backed micro-design lane.
//!
//! `src/microdesign/` carries the comparator core
//! (`compare_manifest_to_tool_report`, `ToolReport`, `Divergence`,
//! `synthetic_tool_report_from_manifest`). This test file is the
//! harness wiring:
//!
//! - **Cargo-portable proofs** exercise the comparator against the
//!   always-agreeing reference (`synthetic_tool_report_from_manifest`)
//!   for the reproducibility-set seeds, then against deliberately
//!   perturbed reports for each fact axis (param / localparam / width
//!   / generate-branch / package-constant). Every divergence kind
//!   surfaces with the right structure, not a single "facts disagree"
//!   bit — `.1`'s rejected-alternatives discussion called out exactly
//!   that requirement.
//!
//! - A **tool-equipped `#[ignore]`-gated** scaffold
//!   (`parity_against_real_yosys_write_json`) compiles and is
//!   invocable but is skipped by the portable suite. `.2c.2` is the
//!   gated step that actually runs it end-to-end and banks a verified
//!   clean artifact before promoting ROADMAP Phase 7 (r87
//!   no-aspirational-claims).
//!
//! The cargo-portable proofs DO NOT shell yosys / verilator / slang.
//! Test must pass on machines without those tools — the Phase-1
//! doctrine recorded in this tree's Decisions and reaffirmed by
//! Phase 6 memory `.2.2` and DIFFERENTIAL-SIMULATION `.2b`.

use anvil::microdesign::{
    build_constexpr_unit, build_manifest, compare_manifest_to_tool_report,
    synthetic_tool_report_from_manifest, Divergence, Manifest, WidthFact,
};

/// The reproducibility set from `.2a`'s
/// `build_is_reproducible_and_seed_sensitive`.
const SEEDS: &[u64] = &[0, 1, 7, 42, 12345];
const N_PARAMS: usize = 5;

fn manifest_for(seed: u64) -> Manifest {
    // `build_constexpr_unit` resolves in-place (the builder IS the
    // oracle; no separate `resolve` call needed).
    let unit = build_constexpr_unit(seed, N_PARAMS);
    build_manifest(&unit, seed)
}

/// Agreement: a `ToolReport` synthesised from the manifest itself —
/// "what a perfectly-conforming downstream tool would have produced" —
/// must compare exactly. Across the reproducibility-set seeds this is
/// the load-bearing precondition for every divergence test below
/// (perturbations begin from a known-agreeing baseline).
#[test]
fn comparator_agrees_on_synthetic_tool_report_built_from_the_oracle() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let report = synthetic_tool_report_from_manifest(&manifest);
        assert_eq!(
            compare_manifest_to_tool_report(&manifest, &report),
            Ok(()),
            "synthetic report built from the manifest must agree exactly (seed={seed})"
        );
    }
}

/// Param axis: perturbing a single parameter's resolved value yields
/// exactly the corresponding `ParamMismatch` divergence (and nothing
/// else triggers — every other category remains in agreement).
#[test]
fn comparator_surfaces_param_mismatch_when_a_param_is_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        // `.2a`'s builder always makes decl 0 a `Parameter`, so every
        // manifest has at least one parameter to perturb.
        let (name, orig) = manifest
            .params
            .iter()
            .next()
            .map(|(n, e)| (n.clone(), e.value))
            .expect("manifest has at least one parameter");
        let actual = orig.wrapping_add(1);

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.params.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed-param report must diverge");
        assert!(
            err.contains(&Divergence::ParamMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected ParamMismatch{{{name}, {orig}, {actual}}} (seed={seed}); got {err:?}"
        );
    }
}

/// Localparam axis: same shape as the param test but on the
/// localparams map. Seeds whose builder happens to emit zero
/// localparams are skipped (rare but possible — every decl after the
/// first is `Localparam` only with ~70% probability per `.2a`).
#[test]
fn comparator_surfaces_localparam_mismatch_when_perturbed() {
    let mut exercised = 0usize;
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let Some((name, orig)) = manifest
            .localparams
            .iter()
            .next()
            .map(|(n, e)| (n.clone(), e.value))
        else {
            continue;
        };
        let actual = orig.wrapping_add(1);

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.localparams.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed-localparam report must diverge");
        assert!(
            err.contains(&Divergence::LocalparamMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected LocalparamMismatch{{{name}, {orig}, {actual}}} (seed={seed}); got {err:?}"
        );
        exercised += 1;
    }
    // Reproducibility-set sanity: not every seed need carry a
    // localparam, but at least one must — otherwise this axis is
    // structurally unreachable in the proof.
    assert!(
        exercised > 0,
        "expected at least one reproducibility-set seed to carry a localparam"
    );
}

/// Width axis: `.2b` always emits the expr-derived `widths["sig"]`
/// fact, so this is exercised on every reproducibility-set seed.
/// Perturbing the bit count surfaces `WidthMismatch` with both old
/// and new `WidthFact` values.
#[test]
fn comparator_surfaces_width_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let name = "sig".to_string();
        let orig = manifest
            .widths
            .get(&name)
            .expect("`.2b` always emits widths[\"sig\"]")
            .clone();
        let actual = WidthFact {
            msb: orig.msb + 1,
            lsb: orig.lsb,
            bits: orig.bits + 1,
        };

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.widths.get_mut(&name).unwrap() = actual.clone();

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed-width report must diverge");
        assert!(
            err.contains(&Divergence::WidthMismatch {
                name: name.clone(),
                expected: orig.clone(),
                actual: actual.clone(),
            }),
            "expected WidthMismatch on sig (seed={seed}); got {err:?}"
        );
    }
}

/// Generate-branch axis: `.2b` always emits `generate["g_taken"]`.
/// Flipping the boolean surfaces `GenerateMismatch` with both
/// directions exercised across the reproducibility set (some seeds
/// take the true branch, others take false — `.2a`/`.2b` together
/// span both).
#[test]
fn comparator_surfaces_generate_branch_mismatch_when_taken_is_flipped() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let name = "g_taken".to_string();
        let orig = manifest
            .generate
            .get(&name)
            .map(|g| g.taken)
            .expect("`.2b` always emits generate[\"g_taken\"]");
        let actual = !orig;

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.generate.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("flipped-generate report must diverge");
        assert!(
            err.contains(&Divergence::GenerateMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected GenerateMismatch on g_taken (seed={seed}); got {err:?}"
        );
    }
}

/// Package-constant axis: `.2b` always emits one entry — the
/// `mc_<seed>_pkg::K` constant. Perturbing its value surfaces
/// `PackageConstantMismatch`.
#[test]
fn comparator_surfaces_package_constant_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let (name, orig) = manifest
            .package_constants
            .iter()
            .next()
            .map(|(n, v)| (n.clone(), *v))
            .expect("manifest has at least one package constant");
        let actual = orig.wrapping_add(1);

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.package_constants.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed-package-constant report must diverge");
        assert!(
            err.contains(&Divergence::PackageConstantMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected PackageConstantMismatch on {name} (seed={seed}); got {err:?}"
        );
    }
}

/// Missing-on-tool-side axis: removing a manifest-declared name from
/// the tool report must surface the corresponding `MissingInTool`
/// variant (we test the param axis as the representative; the other
/// `MissingInTool` variants share the same code path in the
/// comparator).
#[test]
fn comparator_surfaces_param_missing_in_tool_when_dropped() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let (name, expected) = manifest
            .params
            .iter()
            .next()
            .map(|(n, e)| (n.clone(), e.value))
            .expect("manifest has at least one parameter");

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        report.params.remove(&name).expect("name was present");

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("missing-param report must diverge");
        assert!(
            err.contains(&Divergence::ParamMissingInTool {
                name: name.clone(),
                expected,
            }),
            "expected ParamMissingInTool{{{name}, {expected}}} (seed={seed}); got {err:?}"
        );
    }
}

/// Missing-on-manifest-side axis: adding a tool-reported name that
/// the manifest does not declare must surface `MissingInManifest`
/// (the spurious-extra-report case — defensive against a tool that
/// over-elaborates).
#[test]
fn comparator_surfaces_param_missing_in_manifest_when_extra() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        let name = format!("P_EXTRA_{seed}");
        let actual = 0xDEAD_BEEF_i128;
        report.params.insert(name.clone(), actual);

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("extra-in-tool report must diverge");
        assert!(
            err.contains(&Divergence::ParamMissingInManifest {
                name: name.clone(),
                actual,
            }),
            "expected ParamMissingInManifest{{{name}, {actual}}} (seed={seed}); got {err:?}"
        );
    }
}

/// Top-name and seed axes: perturbing the report's seed or top name
/// must surface the matching top-level divergence variants. These are
/// load-bearing because the harness uses them to detect a stale or
/// mis-routed tool report.
#[test]
fn comparator_surfaces_seed_and_top_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);

        let mut report = synthetic_tool_report_from_manifest(&manifest);
        report.seed = manifest.seed.wrapping_add(1);
        report.top = format!("{}_WRONG", manifest.top);

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("seed+top-perturbed report must diverge");
        assert!(
            err.contains(&Divergence::SeedMismatch {
                expected: manifest.seed,
                actual: report.seed,
            }),
            "expected SeedMismatch{{{}, {}}} (seed={seed}); got {err:?}",
            manifest.seed,
            report.seed
        );
        assert!(
            err.contains(&Divergence::TopMismatch {
                expected: manifest.top.clone(),
                actual: report.top.clone(),
            }),
            "expected TopMismatch{{{}, {}}} (seed={seed}); got {err:?}",
            manifest.top,
            report.top
        );
    }
}

// ===================================================================
// PHASE-7-ORACLE-MICRODESIGN.2c.1 — tool-equipped `#[ignore]` scaffold.
//
// This test is `#[ignore]` so the portable `cargo test` stays green
// tool-less (Phase-1 doctrine; mirrors DIFFERENTIAL-SIMULATION `.2b`).
// `.2c.1` lands the scaffold so it COMPILES + is INVOCABLE today; the
// actual end-to-end real-tool run + verified-clean banked artifact +
// ROADMAP Phase 7 promotion is `.2c.2`.
// ===================================================================

/// Real-tool parity gate, invocable with `cargo test -- --ignored`.
/// If the downstream consumer (currently `yosys`) is not on `$PATH`
/// the test is a friendly no-op (the harness must remain invocable
/// on machines where the tool is missing — exactly the
/// `iverilog`-not-installed convention from
/// `DIFFERENTIAL-SIMULATION.1`).
///
/// The actual `write_json` parsing + per-name fact extraction is
/// `.2c.2`'s responsibility: once the extractor is wired in, the
/// gated test drives a fixed deterministic corpus through
/// `emit_sv`/`emit_manifest`, shells the consumer on each `.sv`,
/// builds a `ToolReport` from the tool's resolved-facts artifact,
/// and runs `compare_manifest_to_tool_report` to assert exact
/// agreement (or retain a counterexample tuple). The comparator and
/// the corpus driver are already proven cargo-portably above.
#[test]
#[ignore]
fn parity_against_real_yosys_write_json() {
    // Tool presence guard. If yosys is absent, this is `#[ignore]`d
    // out of the portable suite anyway and would simply be a no-op
    // when explicitly invoked; the friendly message keeps the
    // scaffold debuggable.
    let probe = std::process::Command::new("yosys").arg("-V").output();
    if probe.is_err() || !probe.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        eprintln!(
            "parity_against_real_yosys_write_json: yosys not on $PATH \
             (scaffold-only at .2c.1; .2c.2 banks the real run)"
        );
        return;
    }

    // Corpus driver scaffold — `.2c.2` wires the per-seed
    // emit-then-shell-then-extract-then-compare loop. We instantiate
    // the corpus here so the scaffold has a real reference point and
    // the harness compiles against the same `SEEDS`/`N_PARAMS`
    // constants the cargo-portable proofs use.
    for &seed in SEEDS {
        let unit = build_constexpr_unit(seed, N_PARAMS);
        let _manifest = build_manifest(&unit, seed);
        // `.2c.2` adds, here:
        //   1. write `emit_sv(&unit, seed)`            -> tmpdir/mc_<seed>.sv
        //   2. write `emit_manifest(&unit, seed)`      -> tmpdir/mc_<seed>.json
        //   3. shell yosys on the `.sv` -> tmpdir/mc_<seed>.tool.json
        //   4. extract a `ToolReport` from (3)         -> in-memory
        //   5. compare_manifest_to_tool_report(...)    -> Ok(()) or
        //      retain the counterexample tuple.
        //
        // `.2c.1`'s commitment is the comparator + scaffold; the
        // missing pieces are the deterministic file dance + the
        // yosys-specific extractor.
    }
}
