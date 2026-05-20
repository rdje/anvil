//! PHASE-8-FRONTEND-ACCEPT.2c.1 — hierarchy-aware parity harness for
//! the source-level frontend / elaboration accept-corpus lane.
//!
//! `src/frontend/` carries the comparator core (`ToolReport`,
//! `Divergence` with hierarchy-aware variants, `FactCategory`,
//! `ParityScope`, `compare_manifest_to_tool_report`,
//! `compare_manifest_to_tool_report_in_scope`,
//! `synthetic_tool_report_from_manifest`). This test file is the
//! harness wiring:
//!
//! - **Cargo-portable proofs** exercise the comparator against the
//!   always-agreeing reference (`synthetic_tool_report_from_manifest`)
//!   across the reproducibility-set seeds, then against deliberately
//!   perturbed reports for each fact axis (top-param /
//!   top-localparam / package-constant / instance-binding /
//!   generate-branch / instance-presence / scope mechanism itself).
//!   Every divergence kind surfaces with the right structure.
//!
//! - A **tool-equipped `#[ignore]`-gated scaffold**
//!   (`parity_against_real_downstream_elaborator`) compiles and is
//!   invocable but is skipped by the portable suite. `.2c.2` is the
//!   gated step that wires it end-to-end against a real elaborator
//!   (yosys hierarchy+write_json AFTER elaboration, slang
//!   `--ast-json`, or verilator `--xml-only`) and banks a verified
//!   clean artifact before ROADMAP Phase 8 → done (r87).
//!
//! The cargo-portable proofs DO NOT shell yosys / slang / verilator.
//! Test must pass on machines without those tools — the Phase-1
//! doctrine recorded in `PHASE-7-ORACLE-MICRODESIGN`'s `Decisions`
//! and applied at `.2c.1`/`.2c.2a`.

use anvil::frontend::{
    build_acceptable_unit, build_manifest, compare_manifest_to_tool_report,
    compare_manifest_to_tool_report_in_scope, synthetic_tool_report_from_manifest, Divergence,
    FactCategory, Manifest, ParityScope,
};
use std::collections::BTreeMap;

/// The reproducibility set from `.2a`'s
/// `unit_is_reproducible_and_seed_sensitive`.
const SEEDS: &[u64] = &[0, 1, 7, 42, 12345];
const N_PARAMS: usize = 4;
const N_CHILDREN: usize = 2;

fn manifest_for(seed: u64) -> Manifest {
    let unit = build_acceptable_unit(seed, N_PARAMS, N_CHILDREN);
    build_manifest(&unit)
}

/// Agreement baseline: a synthetic `ToolReport` constructed from the
/// manifest itself — "what a perfectly-conforming downstream tool
/// would have produced" — must compare exactly across the
/// reproducibility-set seeds. The load-bearing precondition for
/// every divergence test below.
#[test]
fn comparator_agrees_on_synthetic_tool_report_built_from_the_oracle() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let report = synthetic_tool_report_from_manifest(&manifest);
        assert_eq!(
            compare_manifest_to_tool_report(&manifest, &report),
            Ok(()),
            "synthetic report from the manifest must agree exactly (seed={seed})"
        );
    }
}

/// Top-param axis: perturbing a single top parameter's resolved
/// value surfaces exactly `TopParamMismatch`.
#[test]
fn comparator_surfaces_top_param_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let (name, orig) = manifest
            .top_params
            .iter()
            .next()
            .map(|(n, f)| (n.clone(), f.value))
            .expect("manifest has at least one top param");
        let actual = orig.wrapping_add(1);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.top_params.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed top param must diverge");
        assert!(
            err.contains(&Divergence::TopParamMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected TopParamMismatch{{{name}, {orig}, {actual}}} (seed={seed}); got {err:?}"
        );
    }
}

/// Top-localparam axis: perturbing a body localparam surfaces
/// `TopLocalparamMismatch`. The `.2a` builder always emits >=1
/// localparam per non-zero `n_params`, so this is exercised for
/// every seed without a sanity guard.
#[test]
fn comparator_surfaces_top_localparam_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let (name, orig) = manifest
            .top_localparams
            .iter()
            .next()
            .map(|(n, f)| (n.clone(), f.value))
            .expect("manifest has at least one top localparam");
        let actual = orig.wrapping_add(1);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        *report.top_localparams.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed top localparam must diverge");
        assert!(
            err.contains(&Divergence::TopLocalparamMismatch {
                name: name.clone(),
                expected: orig,
                actual,
            }),
            "expected TopLocalparamMismatch{{{name}, {orig}, {actual}}} (seed={seed}); got {err:?}"
        );
    }
}

/// Package-constant axis: perturbing the `acc_<seed>_pkg::K`
/// resolved value surfaces `PackageConstantMismatch`. `.2a`'s
/// builder always emits exactly one package constant.
#[test]
fn comparator_surfaces_package_constant_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        let (name, orig) = report
            .package_constants
            .iter()
            .next()
            .map(|(n, v)| (n.clone(), *v))
            .expect("synthetic report has the K constant");
        let actual = orig.wrapping_add(1);
        *report.package_constants.get_mut(&name).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed package constant must diverge");
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

/// Per-instance per-binding axis (the **hierarchy-aware** Phase-8
/// addition): perturbing one binding on one instance surfaces
/// `InstanceBindingMismatch` keyed by `(inst_name, binding_name)`.
/// The other instance's bindings stay clean — pin that by checking
/// no `InstanceBindingMismatch` divergence carries the wrong
/// `inst_name`.
#[test]
fn comparator_surfaces_instance_binding_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        // Pick the first instance, perturb its first binding.
        let target_inst_idx = 0usize;
        let target_inst_name = report.instances[target_inst_idx].inst_name.clone();
        let (b_name, b_orig) = report.instances[target_inst_idx]
            .resolved_bindings
            .iter()
            .next()
            .map(|(n, v)| (n.clone(), *v))
            .expect("instance has >=1 binding");
        let b_actual = b_orig.wrapping_add(1);
        *report.instances[target_inst_idx]
            .resolved_bindings
            .get_mut(&b_name)
            .unwrap() = b_actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("perturbed instance binding must diverge");
        assert!(
            err.contains(&Divergence::InstanceBindingMismatch {
                inst_name: target_inst_name.clone(),
                name: b_name.clone(),
                expected: b_orig,
                actual: b_actual,
            }),
            "expected InstanceBindingMismatch{{{target_inst_name}, {b_name}, \
             {b_orig}, {b_actual}}} (seed={seed}); got {err:?}"
        );
        // No divergence on the other instance.
        let other_inst_name = report.instances[1].inst_name.clone();
        for div in &err {
            if let Divergence::InstanceBindingMismatch { inst_name, .. } = div {
                assert_ne!(
                    *inst_name, other_inst_name,
                    "no instance-binding divergence should target the other instance (seed={seed})"
                );
            }
        }
    }
}

/// Generate-branch axis: flipping `g_taken`'s `taken` surfaces
/// `GenerateMismatch`. The `.2a` builder emits exactly one
/// generate-if labeled `g_taken`.
#[test]
fn comparator_surfaces_generate_branch_mismatch_when_flipped() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        let label = "g_taken".to_string();
        let orig = *report.generate_branches.get(&label).expect("g_taken");
        let actual = !orig;
        *report.generate_branches.get_mut(&label).unwrap() = actual;

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("flipped generate branch must diverge");
        assert!(
            err.contains(&Divergence::GenerateMismatch {
                label: label.clone(),
                expected: orig,
                actual,
            }),
            "expected GenerateMismatch on g_taken (seed={seed}); got {err:?}"
        );
    }
}

/// Instance-presence axis (the other hierarchy-aware Phase-8
/// addition): dropping an instance from the report surfaces
/// `InstanceMissingInTool`; adding a spurious one surfaces
/// `InstanceMissingInManifest`.
#[test]
fn comparator_surfaces_instance_presence_divergences() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);

        // Drop the first instance from the report — must surface
        // `InstanceMissingInTool` for that name.
        let dropped_name = {
            let mut r = synthetic_tool_report_from_manifest(&manifest);
            let n = r.instances.remove(0).inst_name.clone();
            let err = compare_manifest_to_tool_report(&manifest, &r)
                .expect_err("missing instance must diverge");
            assert!(
                err.contains(&Divergence::InstanceMissingInTool {
                    inst_name: n.clone(),
                }),
                "expected InstanceMissingInTool{{{n}}} (seed={seed}); got {err:?}"
            );
            n
        };

        // Add a spurious instance to the report — must surface
        // `InstanceMissingInManifest`.
        let mut r = synthetic_tool_report_from_manifest(&manifest);
        let extra_name = format!("u_extra_{seed}");
        r.instances.push(anvil::frontend::InstanceToolReport {
            inst_name: extra_name.clone(),
            child_module: format!("child_{seed}"),
            resolved_bindings: BTreeMap::new(),
        });
        let err = compare_manifest_to_tool_report(&manifest, &r)
            .expect_err("extra-in-tool instance must diverge");
        assert!(
            err.contains(&Divergence::InstanceMissingInManifest {
                inst_name: extra_name.clone(),
            }),
            "expected InstanceMissingInManifest{{{extra_name}}} (seed={seed}); got {err:?}"
        );

        // The dropped name is in the manifest but not the report.
        assert!(manifest
            .instances
            .iter()
            .any(|i| i.inst_name == dropped_name));
    }
}

/// Seed + top axes: perturbing both surfaces both top-level
/// divergence variants in one fixture. Defensive against a stale or
/// mis-routed tool report.
#[test]
fn comparator_surfaces_seed_and_top_mismatch_when_perturbed() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        report.seed = manifest.seed.wrapping_add(1);
        report.top = format!("{}_WRONG", manifest.top);

        let err = compare_manifest_to_tool_report(&manifest, &report)
            .expect_err("seed+top-perturbed must diverge");
        assert!(
            err.contains(&Divergence::SeedMismatch {
                expected: manifest.seed,
                actual: report.seed,
            }),
            "expected SeedMismatch (seed={seed}); got {err:?}"
        );
        assert!(
            err.contains(&Divergence::TopMismatch {
                expected: manifest.top.clone(),
                actual: report.top.clone(),
            }),
            "expected TopMismatch (seed={seed}); got {err:?}"
        );
    }
}

/// Scoping mechanism: `ParityScope::only(&[TopParams])` ignores a
/// perturbed instance binding (out-of-scope) but surfaces a
/// perturbed top param (in-scope). The Phase-8 counterpart of
/// Phase 7's `scoped_comparator_only_enforces_scoped_categories`.
#[test]
fn scoped_comparator_only_enforces_scoped_categories() {
    let manifest = manifest_for(7);
    let scope_top_params_only = ParityScope::only(&[FactCategory::TopParams]);

    // Perturb the (out-of-scope) instance binding: must be Ok(()).
    let mut report = synthetic_tool_report_from_manifest(&manifest);
    let (b_name, b_val) = report.instances[0]
        .resolved_bindings
        .iter()
        .next()
        .map(|(n, v)| (n.clone(), *v))
        .unwrap();
    *report.instances[0]
        .resolved_bindings
        .get_mut(&b_name)
        .unwrap() = b_val.wrapping_add(1);
    assert_eq!(
        compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope_top_params_only),
        Ok(()),
        "out-of-scope instance-binding perturbation must not surface"
    );

    // Perturb the (in-scope) first top param: must surface.
    let (name, orig) = manifest
        .top_params
        .iter()
        .next()
        .map(|(n, f)| (n.clone(), f.value))
        .unwrap();
    let actual = orig.wrapping_add(1);
    let mut report = synthetic_tool_report_from_manifest(&manifest);
    *report.top_params.get_mut(&name).unwrap() = actual;
    let err = compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope_top_params_only)
        .expect_err("in-scope perturbation must diverge");
    assert!(
        err.contains(&Divergence::TopParamMismatch {
            name: name.clone(),
            expected: orig,
            actual,
        }),
        "expected TopParamMismatch on {name}; got {err:?}"
    );
}

/// `ParityScope::none()` returns `Ok(())` even on a maximally
/// disagreeing report. Self-check on the scoping itself.
#[test]
fn empty_scope_ignores_every_disagreement() {
    let manifest = manifest_for(0);
    let report = anvil::frontend::ToolReport {
        seed: manifest.seed.wrapping_add(1),
        top: format!("{}_WRONG", manifest.top),
        package_constants: BTreeMap::new(),
        top_params: BTreeMap::new(),
        top_localparams: BTreeMap::new(),
        instances: Vec::new(),
        generate_branches: BTreeMap::new(),
    };
    assert_eq!(
        compare_manifest_to_tool_report_in_scope(&manifest, &report, &ParityScope::none()),
        Ok(()),
        "ParityScope::none() must tolerate any disagreement"
    );
}

// ===================================================================
// PHASE-8-FRONTEND-ACCEPT.2c.1 — tool-equipped `#[ignore]` scaffold.
//
// `#[ignore]` so the portable `cargo test` stays green tool-less.
// `.2c.2` is the gated step that wires it end-to-end and banks a
// verified-clean artifact before ROADMAP Phase 8 → done.
// ===================================================================

/// Real-tool parity gate scaffold, invocable with
/// `cargo test -- --ignored parity_against_real_downstream_elaborator`.
///
/// If no downstream elaborator is on `$PATH` (`yosys`, `slang`, or
/// `verilator`), the test is a friendly no-op (the harness must
/// remain invocable on machines without the tool — the Phase-1
/// doctrine reaffirmed in `PHASE-7-ORACLE-MICRODESIGN`'s
/// `Decisions`).
///
/// The actual fact extraction + downstream-tool integration is
/// `.2c.2`'s responsibility (mirrors how `PHASE-7-ORACLE-MICRODESIGN.2c.2a`
/// landed the yosys-specific extractor after `.2c.1` landed the
/// comparator + scaffold). `.2c.1` here lands the scaffold + the
/// corpus driver loop so the harness compiles + is invocable today,
/// with the comparator already proven cargo-portably above.
#[test]
#[ignore]
fn parity_against_real_downstream_elaborator() {
    // Tool presence guard — any of yosys / slang / verilator is
    // sufficient; `.2c.2` picks which one and wires the extractor.
    let any_present = ["yosys", "slang", "verilator"].iter().any(|name| {
        std::process::Command::new(name)
            .arg("-V")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    });
    if !any_present {
        eprintln!(
            "parity_against_real_downstream_elaborator: no elaborator on $PATH \
             (scaffold-only at .2c.1; .2c.2 wires the extractor + banks the real run)"
        );
        return;
    }

    // Corpus driver scaffold — `.2c.2` wires the per-seed
    // emit-then-shell-then-extract-then-compare loop. Instantiating
    // here so the scaffold has a real reference point and compiles
    // against the same SEEDS / N_PARAMS / N_CHILDREN constants the
    // cargo-portable proofs use.
    for &seed in SEEDS {
        let unit = build_acceptable_unit(seed, N_PARAMS, N_CHILDREN);
        let _manifest = build_manifest(&unit);
        // `.2c.2` adds, here:
        //   1. write `emit_sv(&unit)`     -> tmpdir/acc_<seed>.sv
        //   2. write `emit_manifest(&unit)` -> tmpdir/acc_<seed>.json
        //   3. shell the chosen consumer on the .sv ->
        //      tmpdir/acc_<seed>.tool.json (or .xml/.ast.json)
        //   4. extract a `ToolReport` from (3)        -> in-memory
        //   5. compare_manifest_to_tool_report_in_scope(...) ->
        //      Ok(()) or retain the counterexample tuple.
        //
        // `.2c.1`'s commitment is the comparator + scaffold; the
        // missing pieces are the deterministic file dance + the
        // chosen-elaborator-specific extractor.
    }
}
