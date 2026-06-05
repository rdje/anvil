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
//!   `--ast-json`, or a Verilator AST dump) and banks a verified
//!   clean artifact before ROADMAP Phase 8 → done (r87).
//!
//! The cargo-portable proofs DO NOT shell yosys / slang / verilator.
//! Test must pass on machines without those tools — the Phase-1
//! doctrine recorded in `PHASE-7-ORACLE-MICRODESIGN`'s `Decisions`
//! and applied at `.2c.1`/`.2c.2a`.

use anvil::frontend::{
    build_acceptable_unit, build_manifest, compare_manifest_to_tool_report,
    compare_manifest_to_tool_report_in_scope, emit_manifest, emit_sv,
    synthetic_tool_report_from_manifest, Divergence, FactCategory, InstanceToolReport, Manifest,
    ParityScope, ToolReport,
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
// PHASE-8-FRONTEND-ACCEPT.2c.2a — yosys-specific `write_json` extractor.
//
// Empirical probe (recorded in the tree's Decisions, 2026-05-20):
// `yosys hierarchy -top acc_<seed>; write_json <out>` on a Phase-8
// `acc_<seed>.sv` exposes 5 of the 7 manifest fact categories:
//
//   .modules.<top>.parameter_default_values
//     - binary-string form, identical to Phase 7's
//       parameter_default_values; parsed as SV `int` (signed 32-bit)
//       → `i128` via the same sign-extend-through-i32 pattern.
//
//   .modules.<top>.cells[<inst>].{type, parameters}
//     - the LOAD-BEARING hierarchy-aware Phase-8 axis. Each cell has:
//       * `type`: the child module name (must match the manifest's
//         `child_module`);
//       * `parameters`: a `name → binary-string` map of the resolved
//         per-binding values (`CP0` → "00...111001" = 57, etc.).
//
//   .modules.<top>.netnames keys
//     - prefix `g_taken.` (the if-branch was elaborated) or `g_else.`
//       (the else-branch). Identical convention to Phase 7's emit.
//
// Folded by yosys (and therefore NOT extracted into the ToolReport):
//   - top_localparams (the `localparam int L<i> = …` chains —
//     resolved into the elaborated netlist but not exposed by name);
//   - package_constants (the `acc_<seed>_pkg::K` reference — folded
//     into the W-derived expressions but not exposed by name).
//
// Crucially: the probe also discovered that `proc; opt` (the Phase
// 7 yosys invocation pipeline) collapses the empty-bodied child
// instances away from `.cells`. The fix is to invoke yosys with
// `hierarchy -top` only — NOT `proc; opt`.
// ===================================================================

/// Parse a yosys `parameter_default_values` (or per-cell
/// `parameters`) binary string as a signed 32-bit SV `int`. Returns
/// `None` on malformed inputs. Mirrors
/// `PHASE-7-ORACLE-MICRODESIGN.2c.2a`'s `parse_yosys_binary_param`
/// (Phase 8 doesn't currently emit negative values via the `.2a`
/// builder, but the sign-extension is kept for symmetry + so the
/// extractor remains correct if future Phase 8 builder variants do).
fn parse_yosys_binary_param(s: &str) -> Option<i128> {
    if s.is_empty() || s.len() > 32 || !s.chars().all(|c| c == '0' || c == '1') {
        return None;
    }
    let u = u32::from_str_radix(s, 2).ok()?;
    Some(u as i32 as i128) // sign-extend through i32
}

/// Build a `ToolReport` from yosys 0.64's `write_json` output for a
/// single elaborated `acc_<seed>` top module.
///
/// Populates only what yosys actually carries (per today's
/// empirical probe; see `.2c.2`'s Decisions entry):
///   * `seed` — supplied by the caller (corpus key, not a yosys fact)
///   * `top` — `acc_<seed>` (also tied to the corpus key)
///   * `package_constants` — EMPTY (folded by yosys; the yosys
///     scope skips this axis)
///   * `top_params` — from `.modules.<top>.parameter_default_values`
///     (binary-string → signed-32-bit → `i128`)
///   * `top_localparams` — EMPTY (folded by yosys)
///   * `instances` — from `.modules.<top>.cells`: each cell's `type`
///     becomes the `child_module` and `parameters` becomes the
///     `resolved_bindings` map (also parsed via
///     `parse_yosys_binary_param`)
///   * `generate_branches["g_taken"]` — `true` iff any netname key
///     is prefixed `g_taken.` (and not `g_else.`)
fn yosys_hierarchy_write_json_to_tool_report(json: &serde_json::Value, seed: u64) -> ToolReport {
    let top = format!("acc_{seed}");
    let module = &json["modules"][&top];

    let mut top_params: BTreeMap<String, i128> = BTreeMap::new();
    if let Some(pdv) = module["parameter_default_values"].as_object() {
        for (name, value) in pdv {
            if let Some(s) = value.as_str() {
                if let Some(v) = parse_yosys_binary_param(s) {
                    top_params.insert(name.clone(), v);
                }
            }
        }
    }

    let mut instances: Vec<InstanceToolReport> = Vec::new();
    if let Some(cells) = module["cells"].as_object() {
        for (inst_name, cell) in cells {
            let child_module = cell["type"].as_str().unwrap_or("").to_string();
            let mut resolved_bindings: BTreeMap<String, i128> = BTreeMap::new();
            if let Some(params) = cell["parameters"].as_object() {
                for (b_name, b_value) in params {
                    if let Some(s) = b_value.as_str() {
                        if let Some(v) = parse_yosys_binary_param(s) {
                            resolved_bindings.insert(b_name.clone(), v);
                        }
                    }
                }
            }
            instances.push(InstanceToolReport {
                inst_name: inst_name.clone(),
                child_module,
                resolved_bindings,
            });
        }
    }

    let mut g_taken_alive = false;
    let mut g_else_alive = false;
    if let Some(netnames) = module["netnames"].as_object() {
        for name in netnames.keys() {
            if name.starts_with("g_taken.") {
                g_taken_alive = true;
            }
            if name.starts_with("g_else.") {
                g_else_alive = true;
            }
        }
    }
    let mut generate_branches: BTreeMap<String, bool> = BTreeMap::new();
    generate_branches.insert("g_taken".to_string(), g_taken_alive && !g_else_alive);

    ToolReport {
        seed,
        top,
        package_constants: BTreeMap::new(), // folded by yosys
        top_params,
        top_localparams: BTreeMap::new(), // folded by yosys
        instances,
        generate_branches,
    }
}

/// The yosys parity scope: only the categories yosys's
/// `hierarchy -top; write_json` actually exposes
/// (Seed/Top/TopParams/Instances/GenerateBranches). The folded
/// `TopLocalparams` + `PackageConstants` axes are deliberately
/// skipped — they're recorded post-Phase-8 follow-up via richer-AST
/// tools (slang/verilator-with-debug).
fn yosys_hierarchy_scope() -> ParityScope {
    ParityScope::only(&[
        FactCategory::Seed,
        FactCategory::Top,
        FactCategory::TopParams,
        FactCategory::Instances,
        FactCategory::GenerateBranches,
    ])
}

/// Cargo-portable extractor proof: a hand-built yosys-like JSON for
/// seed 0 produces the expected `ToolReport`. Exercises every
/// branch of `yosys_hierarchy_write_json_to_tool_report` — the
/// `parameter_default_values` parser, the `.cells[<inst>].{type,
/// parameters}` mapping for two instances with two bindings each,
/// the netname-prefix scan, and the folded-axes-stay-empty
/// invariant.
#[test]
fn yosys_extractor_reads_a_synthetic_hierarchy_write_json_correctly() {
    let synthetic = serde_json::json!({
        "modules": {
            "acc_0": {
                "parameter_default_values": {
                    "P0": "00000000000000000000000000111001", // 57
                    "P1": "00000000000000000000000000100110"  // 38
                },
                "cells": {
                    "u_0_0": {
                        "type": "child_0",
                        "parameters": {
                            "CP0": "00000000000000000000000000111001", // 57
                            "CP1": "00000000000000000000000000101000"  // 40
                        }
                    },
                    "u_0_1": {
                        "type": "child_0",
                        "parameters": {
                            "CP0": "00000000000000000000000000111100", // 60
                            "CP1": "00000000000000000000000000111011"  // 59
                        }
                    }
                },
                "netnames": {
                    "g_taken.gflag": { "hide_name": 0, "bits": ["0"] }
                }
            }
        }
    });
    let report = yosys_hierarchy_write_json_to_tool_report(&synthetic, 0);
    assert_eq!(report.seed, 0);
    assert_eq!(report.top, "acc_0");
    // Top params
    assert_eq!(report.top_params.get("P0").copied(), Some(57));
    assert_eq!(report.top_params.get("P1").copied(), Some(38));
    // Generate branch
    assert_eq!(report.generate_branches.get("g_taken").copied(), Some(true));
    // Instances (BTreeMap iteration order for parameters lets us
    // assert exact values name-by-name)
    assert_eq!(report.instances.len(), 2);
    let u00 = report
        .instances
        .iter()
        .find(|i| i.inst_name == "u_0_0")
        .expect("u_0_0");
    assert_eq!(u00.child_module, "child_0");
    assert_eq!(u00.resolved_bindings.get("CP0").copied(), Some(57));
    assert_eq!(u00.resolved_bindings.get("CP1").copied(), Some(40));
    let u01 = report
        .instances
        .iter()
        .find(|i| i.inst_name == "u_0_1")
        .expect("u_0_1");
    assert_eq!(u01.child_module, "child_0");
    assert_eq!(u01.resolved_bindings.get("CP0").copied(), Some(60));
    assert_eq!(u01.resolved_bindings.get("CP1").copied(), Some(59));
    // Folded axes deliberately empty.
    assert!(report.package_constants.is_empty());
    assert!(report.top_localparams.is_empty());
}

/// Negative-side proof: an `else`-surviving netnames map produces
/// `generate_branches["g_taken"] = false`. Same convention as
/// Phase 7's `yosys_extractor_reports_g_else_when_else_branch_survives`.
#[test]
fn yosys_extractor_reports_g_else_when_else_branch_survives() {
    let synthetic = serde_json::json!({
        "modules": {
            "acc_99": {
                "parameter_default_values": {
                    "P0": "00000000000000000000000000000000"
                },
                "cells": {},
                "netnames": {
                    "g_else.gflag": { "hide_name": 0, "bits": ["0"] }
                }
            }
        }
    });
    let report = yosys_hierarchy_write_json_to_tool_report(&synthetic, 99);
    assert_eq!(
        report.generate_branches.get("g_taken").copied(),
        Some(false)
    );
}

// ===================================================================
// PHASE-8-FRONTEND-ACCEPT.2c.2a — end-to-end-runnable `#[ignore]`.
//
// `#[ignore]` so the portable `cargo test` stays green tool-less
// (Phase-1 doctrine reaffirmed in PHASE-7-ORACLE-MICRODESIGN's
// `Decisions`, applied at `.2c.1`/`.2c.2a` in Phase 7 + this slice
// in Phase 8). `.2c.2b` runs this end-to-end against real yosys
// and banks a verified-clean artifact before ROADMAP Phase 8 → done.
// ===================================================================

/// Real-tool parity gate, invocable with
/// `cargo test --test frontend_parity -- --ignored
///  parity_against_real_yosys_hierarchy_write_json`.
///
/// For each seed in the reproducibility set:
/// 1. Build `unit` via `build_acceptable_unit(seed, N_PARAMS, N_CHILDREN)`.
/// 2. Write `emit_sv(&unit)` to `CARGO_TARGET_TMPDIR/
///    frontend-parity-phase8-yosys/acc_<seed>.sv`.
/// 3. Write `emit_manifest(&unit)` to
///    `…/acc_<seed>.json`.
/// 4. Shell `yosys -q -p "read_verilog -sv <sv>; hierarchy -top
///    acc_<seed>; write_json <out>.json"` (deliberately NO
///    `proc; opt` — the probe in `.2c.2`'s Decisions confirmed it
///    collapses the empty-bodied child instances away from
///    `.cells`).
/// 5. Parse the yosys output → `ToolReport` via
///    `yosys_hierarchy_write_json_to_tool_report`.
/// 6. Call `compare_manifest_to_tool_report_in_scope(manifest,
///    report, &yosys_hierarchy_scope())` and assert `Ok(())` (or
///    retain a counterexample tuple).
///
/// On `yosys` absent: friendly no-op (matches the
/// `iverilog`-not-installed convention from
/// `DIFFERENTIAL-SIMULATION.1`).
#[test]
#[ignore]
fn parity_against_real_yosys_hierarchy_write_json() {
    // Tool presence guard.
    let probe = std::process::Command::new("yosys").arg("-V").output();
    if probe.is_err() || !probe.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        eprintln!(
            "parity_against_real_yosys_hierarchy_write_json: yosys not on $PATH \
             (skipping; rerun with yosys installed for the real-tool gate)"
        );
        return;
    }

    let dir =
        std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("frontend-parity-phase8-yosys");
    std::fs::create_dir_all(&dir).expect("can create the harness output dir");

    let scope = yosys_hierarchy_scope();
    let mut counterexamples: Vec<(u64, Vec<Divergence>)> = Vec::new();

    for &seed in SEEDS {
        let unit = build_acceptable_unit(seed, N_PARAMS, N_CHILDREN);
        let manifest = build_manifest(&unit);

        let sv_path = dir.join(format!("acc_{seed}.sv"));
        let manifest_path = dir.join(format!("acc_{seed}.json"));
        let yosys_path = dir.join(format!("acc_{seed}.yosys.json"));
        std::fs::write(&sv_path, emit_sv(&unit)).expect("write .sv");
        std::fs::write(&manifest_path, emit_manifest(&unit)).expect("write manifest .json");

        // NB: deliberately NO `proc; opt` — see the .2c.2 Decisions
        // entry. `proc; opt` collapses the empty-bodied child
        // instances out of `.cells`, dropping the hierarchy-aware
        // facts the Phase-8 comparator needs.
        let script = format!(
            "read_verilog -sv {sv}; hierarchy -top acc_{seed}; write_json {out}",
            sv = sv_path.display(),
            out = yosys_path.display(),
        );
        let status = std::process::Command::new("yosys")
            .arg("-q")
            .arg("-p")
            .arg(&script)
            .output()
            .expect("run yosys");
        assert!(
            status.status.success(),
            "yosys exited non-zero on seed {seed}: stderr=\n{}",
            String::from_utf8_lossy(&status.stderr)
        );

        let yosys_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&yosys_path).expect("read yosys json"))
                .expect("parse yosys json");
        let report = yosys_hierarchy_write_json_to_tool_report(&yosys_json, seed);

        match compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope) {
            Ok(()) => {}
            Err(divergences) => {
                counterexamples.push((seed, divergences));
            }
        }
    }

    if !counterexamples.is_empty() {
        for (seed, divs) in &counterexamples {
            eprintln!(
                "parity counterexample at seed={seed} (artifacts in {}/): divergences={divs:?}",
                dir.display()
            );
        }
        panic!(
            "parity gate retained {} counterexample(s); artifact dir: {}",
            counterexamples.len(),
            dir.display()
        );
    }

    eprintln!(
        "parity gate clean across {} seeds; artifacts in {}",
        SEEDS.len(),
        dir.display()
    );
}

/// `.2c.1`'s any-of-`yosys`/`slang`/`verilator` scaffold preserved
/// as a friendly no-op (since `.2c.2a` picks yosys; `.2c.2b` runs
/// against real yosys via the named test above). Kept so the
/// scaffold's intent remains documented in-tree.
#[test]
#[ignore]
fn parity_against_real_downstream_elaborator() {
    eprintln!(
        "parity_against_real_downstream_elaborator: .2c.2a picked yosys; \
         see parity_against_real_yosys_hierarchy_write_json for the live gate"
    );
}

// ===================================================================
// SIGNOFF-SURFACE-EXPANSION.2 — Verilator `--json-only` AST extractor.
//
// Empirical local-tool probe (2026-06-05): Verilator 5.046 on this
// machine does NOT accept the older `--xml-only` option recorded as a
// Phase-8 follow-up, but it does expose `--json-only` plus
// `--json-only-output` / `--json-only-meta-output`.
//
// That JSON AST carries a richer fact surface than yosys 0.64's
// `hierarchy -top; write_json` report for the Phase-8 frontend lane:
//
//   * top GPARAMs: direct `VAR varType=GPARAM` entries under the top
//     module's `stmtsp`, each with a resolved `valuep[CONST]`;
//   * top LPARAMs: direct `VAR varType=LPARAM` entries under the top
//     module's `stmtsp`;
//   * package constants: `PACKAGE` entries in `modulesp`, again as
//     direct `VAR varType=LPARAM` entries;
//   * instances: top-module `CELL` entries point via `modp` to
//     Verilator's specialized child modules, whose `origName` is the
//     source child module and whose GPARAM values are the resolved
//     instance bindings;
//   * generate branches: the surviving branch is represented as a
//     direct `GENBLOCK` named `g_taken` or `g_else`.
//
// Keep this extractor in the test harness: it is a signoff surface,
// not production DUT generation.
// ===================================================================

fn parse_verilator_int_const(name: &str) -> Option<i128> {
    let literal = name.trim().replace('_', "");
    if literal.is_empty() {
        return None;
    }

    if let Some(apostrophe_idx) = literal.find('\'') {
        let width: u32 = literal[..apostrophe_idx].parse().ok()?;
        if width == 0 || width > 127 {
            return None;
        }
        let mut suffix = &literal[apostrophe_idx + 1..];
        let explicit_signed = suffix.starts_with('s') || suffix.starts_with('S');
        if explicit_signed {
            suffix = &suffix[1..];
        }
        let (base, digits) = suffix.split_at(1);
        if digits.is_empty()
            || digits
                .chars()
                .any(|c| matches!(c, 'x' | 'X' | 'z' | 'Z' | '?'))
        {
            return None;
        }
        let radix = match base {
            "b" | "B" => 2,
            "o" | "O" => 8,
            "d" | "D" => 10,
            "h" | "H" => 16,
            _ => return None,
        };

        let magnitude = if let Some(stripped) = digits.strip_prefix('-') {
            let value = i128::from_str_radix(stripped, radix).ok()?;
            return Some(-value);
        } else {
            u128::from_str_radix(digits, radix).ok()?
        };
        if magnitude >= (1u128 << width) && radix != 10 {
            return None;
        }

        // Phase-8 frontend facts are SystemVerilog `int` values.  In
        // Verilator JSON those appear as both `32'sh...` and `32'h...`;
        // sign-extend all 32-bit facts so future negative values stay
        // aligned with the manifest instead of becoming large unsigned
        // integers.
        let should_sign_extend = explicit_signed || width == 32;
        if should_sign_extend && (magnitude & (1u128 << (width - 1))) != 0 {
            let signed = magnitude as i128 - (1i128 << width);
            Some(signed)
        } else {
            Some(magnitude as i128)
        }
    } else {
        literal.parse::<i128>().ok()
    }
}

fn verilator_value_type(value: &serde_json::Value) -> Option<&str> {
    value.get("type").and_then(serde_json::Value::as_str)
}

fn verilator_value_name(value: &serde_json::Value) -> Option<&str> {
    value.get("name").and_then(serde_json::Value::as_str)
}

fn verilator_stmtsp(value: &serde_json::Value) -> &[serde_json::Value] {
    value
        .get("stmtsp")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn verilator_modulesp(value: &serde_json::Value) -> &[serde_json::Value] {
    value
        .get("modulesp")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn verilator_var_value(var: &serde_json::Value) -> Option<i128> {
    let valuep = var.get("valuep")?.as_array()?;
    valuep
        .iter()
        .find(|v| verilator_value_type(v) == Some("CONST"))
        .and_then(verilator_value_name)
        .and_then(parse_verilator_int_const)
}

fn verilator_param_map(
    module_or_package: &serde_json::Value,
    var_type: &str,
) -> BTreeMap<String, i128> {
    let mut out = BTreeMap::new();
    for stmt in verilator_stmtsp(module_or_package) {
        if verilator_value_type(stmt) != Some("VAR") {
            continue;
        }
        if stmt.get("varType").and_then(serde_json::Value::as_str) != Some(var_type) {
            continue;
        }
        if stmt.get("isParam").and_then(serde_json::Value::as_bool) != Some(true) {
            continue;
        }
        if let (Some(name), Some(value)) = (verilator_value_name(stmt), verilator_var_value(stmt)) {
            out.insert(name.to_string(), value);
        }
    }
    out
}

fn verilator_json_to_tool_report(json: &serde_json::Value, seed: u64) -> ToolReport {
    let top = format!("acc_{seed}");
    let modulesp = verilator_modulesp(json);
    let top_module = modulesp.iter().find(|m| {
        verilator_value_type(m) == Some("MODULE") && verilator_value_name(m) == Some(top.as_str())
    });

    let mut modules_by_addr: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    for module in modulesp {
        if verilator_value_type(module) == Some("MODULE") {
            if let Some(addr) = module.get("addr").and_then(serde_json::Value::as_str) {
                modules_by_addr.insert(addr.to_string(), module);
            }
        }
    }

    let mut package_constants = BTreeMap::new();
    for package in modulesp {
        if verilator_value_type(package) != Some("PACKAGE") {
            continue;
        }
        let Some(package_name) = verilator_value_name(package) else {
            continue;
        };
        for (name, value) in verilator_param_map(package, "LPARAM") {
            package_constants.insert(format!("{package_name}::{name}"), value);
        }
    }

    let mut top_params = BTreeMap::new();
    let mut top_localparams = BTreeMap::new();
    let mut instances = Vec::new();
    let mut g_taken_alive = false;
    let mut g_else_alive = false;

    if let Some(top_module) = top_module {
        top_params = verilator_param_map(top_module, "GPARAM");
        top_localparams = verilator_param_map(top_module, "LPARAM");

        for stmt in verilator_stmtsp(top_module) {
            match verilator_value_type(stmt) {
                Some("CELL") => {
                    let Some(inst_name) = verilator_value_name(stmt) else {
                        continue;
                    };
                    let Some(child_addr) = stmt.get("modp").and_then(serde_json::Value::as_str)
                    else {
                        continue;
                    };
                    let Some(child_module) = modules_by_addr.get(child_addr) else {
                        continue;
                    };
                    let child_name = child_module
                        .get("origName")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| verilator_value_name(child_module))
                        .unwrap_or("");
                    instances.push(InstanceToolReport {
                        inst_name: inst_name.to_string(),
                        child_module: child_name.to_string(),
                        resolved_bindings: verilator_param_map(child_module, "GPARAM"),
                    });
                }
                Some("GENBLOCK") => match verilator_value_name(stmt) {
                    Some("g_taken") => g_taken_alive = true,
                    Some("g_else") => g_else_alive = true,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    let mut generate_branches = BTreeMap::new();
    generate_branches.insert("g_taken".to_string(), g_taken_alive && !g_else_alive);

    ToolReport {
        seed,
        top,
        package_constants,
        top_params,
        top_localparams,
        instances,
        generate_branches,
    }
}

fn verilator_json_scope() -> ParityScope {
    ParityScope::all()
}

#[test]
fn verilator_const_parser_reads_sv_int_literals() {
    assert_eq!(parse_verilator_int_const("32'sh39"), Some(57));
    assert_eq!(parse_verilator_int_const("32'h35"), Some(53));
    assert_eq!(parse_verilator_int_const("32'hffff_ffff"), Some(-1));
    assert_eq!(parse_verilator_int_const("8'shff"), Some(-1));
    assert_eq!(parse_verilator_int_const("1'h1"), Some(1));
    assert_eq!(parse_verilator_int_const("32'hx"), None);
}

#[test]
fn verilator_json_extractor_reads_a_synthetic_ast_correctly() {
    let synthetic = serde_json::json!({
        "type": "NETLIST",
        "modulesp": [
            {
                "type": "MODULE",
                "name": "acc_0",
                "origName": "acc_0",
                "addr": "(top)",
                "stmtsp": [
                    {
                        "type": "VAR",
                        "name": "P0",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'sh39" }]
                    },
                    {
                        "type": "VAR",
                        "name": "P1",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'sh26" }]
                    },
                    {
                        "type": "VAR",
                        "name": "L0",
                        "varType": "LPARAM",
                        "isParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'h35" }]
                    },
                    {
                        "type": "CELL",
                        "name": "u_0_0",
                        "modp": "(child0)"
                    },
                    {
                        "type": "CELL",
                        "name": "u_0_1",
                        "modp": "(child1)"
                    },
                    {
                        "type": "GENBLOCK",
                        "name": "g_taken"
                    }
                ]
            },
            {
                "type": "PACKAGE",
                "name": "acc_0_pkg",
                "stmtsp": [
                    {
                        "type": "VAR",
                        "name": "K",
                        "varType": "LPARAM",
                        "isParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'sh1" }]
                    }
                ]
            },
            {
                "type": "MODULE",
                "name": "child_0__C39_CB28",
                "origName": "child_0",
                "addr": "(child0)",
                "stmtsp": [
                    {
                        "type": "VAR",
                        "name": "CP0",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'h39" }]
                    },
                    {
                        "type": "VAR",
                        "name": "CP1",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'h28" }]
                    }
                ]
            },
            {
                "type": "MODULE",
                "name": "child_0__C3c_CB3b",
                "origName": "child_0",
                "addr": "(child1)",
                "stmtsp": [
                    {
                        "type": "VAR",
                        "name": "CP0",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'h3c" }]
                    },
                    {
                        "type": "VAR",
                        "name": "CP1",
                        "varType": "GPARAM",
                        "isParam": true,
                        "isGParam": true,
                        "valuep": [{ "type": "CONST", "name": "32'h3b" }]
                    }
                ]
            }
        ]
    });

    let report = verilator_json_to_tool_report(&synthetic, 0);
    assert_eq!(report.seed, 0);
    assert_eq!(report.top, "acc_0");
    assert_eq!(report.top_params.get("P0").copied(), Some(57));
    assert_eq!(report.top_params.get("P1").copied(), Some(38));
    assert_eq!(report.top_localparams.get("L0").copied(), Some(53));
    assert_eq!(
        report.package_constants.get("acc_0_pkg::K").copied(),
        Some(1)
    );
    assert_eq!(report.generate_branches.get("g_taken").copied(), Some(true));
    assert_eq!(report.instances.len(), 2);

    let u00 = report
        .instances
        .iter()
        .find(|i| i.inst_name == "u_0_0")
        .expect("u_0_0");
    assert_eq!(u00.child_module, "child_0");
    assert_eq!(u00.resolved_bindings.get("CP0").copied(), Some(57));
    assert_eq!(u00.resolved_bindings.get("CP1").copied(), Some(40));

    let u01 = report
        .instances
        .iter()
        .find(|i| i.inst_name == "u_0_1")
        .expect("u_0_1");
    assert_eq!(u01.child_module, "child_0");
    assert_eq!(u01.resolved_bindings.get("CP0").copied(), Some(60));
    assert_eq!(u01.resolved_bindings.get("CP1").copied(), Some(59));
}

#[test]
fn verilator_json_extractor_reports_g_else_when_else_branch_survives() {
    let synthetic = serde_json::json!({
        "type": "NETLIST",
        "modulesp": [
            {
                "type": "MODULE",
                "name": "acc_99",
                "origName": "acc_99",
                "addr": "(top)",
                "stmtsp": [
                    {
                        "type": "GENBLOCK",
                        "name": "g_else"
                    }
                ]
            }
        ]
    });

    let report = verilator_json_to_tool_report(&synthetic, 99);
    assert_eq!(
        report.generate_branches.get("g_taken").copied(),
        Some(false)
    );
}

/// Real-tool parity gate, invocable with
/// `cargo test --test frontend_parity -- --ignored
///  parity_against_real_verilator_json_frontend_ast`.
///
/// This gate is optional: if Verilator is absent, or if a local
/// Verilator build predates `--json-only`, it prints a skip reason and
/// returns. When available, it enforces the full Phase-8 fact scope:
/// seed, top, package constants, top params, top localparams,
/// per-instance bindings, and surviving generate branch.
#[test]
#[ignore]
fn parity_against_real_verilator_json_frontend_ast() {
    let probe = std::process::Command::new("verilator")
        .arg("--version")
        .output();
    if probe.is_err() || !probe.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        eprintln!(
            "parity_against_real_verilator_json_frontend_ast: verilator not on $PATH \
             (skipping; rerun with verilator installed for the real-tool gate)"
        );
        return;
    }

    let help = std::process::Command::new("verilator")
        .arg("--help")
        .output();
    let supports_json_only = help
        .as_ref()
        .ok()
        .map(|o| {
            let mut text = String::from_utf8_lossy(&o.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&o.stderr));
            text.contains("--json-only") && text.contains("--json-only-output")
        })
        .unwrap_or(false);
    if !supports_json_only {
        eprintln!(
            "parity_against_real_verilator_json_frontend_ast: local verilator lacks \
             --json-only / --json-only-output (skipping; richer AST gate unavailable)"
        );
        return;
    }

    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("frontend-parity-signoff-verilator-json");
    std::fs::create_dir_all(&dir).expect("can create the harness output dir");

    let scope = verilator_json_scope();
    let mut counterexamples: Vec<(u64, Vec<Divergence>)> = Vec::new();

    for &seed in SEEDS {
        let unit = build_acceptable_unit(seed, N_PARAMS, N_CHILDREN);
        let manifest = build_manifest(&unit);

        let sv_path = dir.join(format!("acc_{seed}.sv"));
        let manifest_path = dir.join(format!("acc_{seed}.json"));
        let verilator_json_path = dir.join(format!("acc_{seed}.verilator.tree.json"));
        let verilator_meta_path = dir.join(format!("acc_{seed}.verilator.meta.json"));
        std::fs::write(&sv_path, emit_sv(&unit)).expect("write .sv");
        std::fs::write(&manifest_path, emit_manifest(&unit)).expect("write manifest .json");

        let status = std::process::Command::new("verilator")
            .arg("--json-only")
            .arg("--json-only-output")
            .arg(&verilator_json_path)
            .arg("--json-only-meta-output")
            .arg(&verilator_meta_path)
            .arg(&sv_path)
            .output()
            .expect("run verilator");
        assert!(
            status.status.success(),
            "verilator exited non-zero on seed {seed}: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&status.stdout),
            String::from_utf8_lossy(&status.stderr)
        );

        let verilator_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&verilator_json_path).expect("read verilator json"),
        )
        .expect("parse verilator json");
        let report = verilator_json_to_tool_report(&verilator_json, seed);

        match compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope) {
            Ok(()) => {}
            Err(divergences) => {
                counterexamples.push((seed, divergences));
            }
        }
    }

    if !counterexamples.is_empty() {
        for (seed, divs) in &counterexamples {
            eprintln!(
                "verilator-json parity counterexample at seed={seed} \
                 (artifacts in {}/): divergences={divs:?}",
                dir.display()
            );
        }
        panic!(
            "verilator-json parity gate retained {} counterexample(s); artifact dir: {}",
            counterexamples.len(),
            dir.display()
        );
    }

    eprintln!(
        "verilator-json parity gate clean across {} seeds; artifacts in {}",
        SEEDS.len(),
        dir.display()
    );
}
