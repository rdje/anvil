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
    compare_manifest_to_tool_report_in_scope, emit_manifest, emit_sv,
    synthetic_tool_report_from_manifest, Divergence, FactCategory, Manifest, ParityScope,
    ToolReport, WidthFact,
};
use std::collections::BTreeMap;

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
// PHASE-7-ORACLE-MICRODESIGN.2c.2a — scoped comparator proofs.
//
// The scoped comparator (`compare_manifest_to_tool_report_in_scope`)
// only enforces the categories named in its `ParityScope`. Out-of-scope
// axes are skipped entirely — a perturbed value on an unscoped axis
// must NOT surface as a `Divergence`. Cargo-portable; matches the
// scoping the yosys gate below uses (yosys 0.64 folds localparams +
// package-constants, so a yosys-scope parity gate ignores them).
// ===================================================================

/// The capability set yosys 0.64's `write_json` exposes. Localparams
/// and package-constants are folded by the elaborator and not
/// name-introspectable from `write_json` alone; the yosys gate scopes
/// the comparator accordingly. (Richer-AST consumers — `slang
/// --ast-json`, `verilator --xml-only` — see the folded categories and
/// would supply a wider scope; recorded follow-up that does not block
/// Phase 7 closure.)
fn yosys_write_json_scope() -> ParityScope {
    ParityScope::only(&[
        FactCategory::Seed,
        FactCategory::Top,
        FactCategory::Params,
        FactCategory::Widths,
        FactCategory::Generate,
    ])
}

/// Load-bearing scoping proof: an out-of-scope category divergence
/// must NOT surface. With `ParityScope::only(&[Params])`, perturbing
/// the report's width must compare `Ok(())` because `Widths` is
/// outside the scope; perturbing the report's param must surface
/// `ParamMismatch` because `Params` is inside.
#[test]
fn scoped_comparator_only_enforces_scoped_categories() {
    let manifest = manifest_for(0);
    let scope_params_only = ParityScope::only(&[FactCategory::Params]);

    // Perturb the (out-of-scope) sig width: must be `Ok(())` under
    // the params-only scope.
    let mut report = synthetic_tool_report_from_manifest(&manifest);
    *report.widths.get_mut("sig").unwrap() = WidthFact {
        msb: 99,
        lsb: 0,
        bits: 100,
    };
    assert_eq!(
        compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope_params_only),
        Ok(()),
        "an out-of-scope Widths perturbation must not surface a divergence"
    );

    // Perturb the (in-scope) first param: must surface ParamMismatch.
    let (name, orig) = manifest
        .params
        .iter()
        .next()
        .map(|(n, e)| (n.clone(), e.value))
        .expect("manifest has >=1 parameter");
    let mut report = synthetic_tool_report_from_manifest(&manifest);
    let actual = orig.wrapping_add(1);
    *report.params.get_mut(&name).unwrap() = actual;
    let err = compare_manifest_to_tool_report_in_scope(&manifest, &report, &scope_params_only)
        .expect_err("an in-scope Params perturbation must diverge");
    assert!(
        err.contains(&Divergence::ParamMismatch {
            name: name.clone(),
            expected: orig,
            actual,
        }),
        "expected ParamMismatch on {name}; got {err:?}"
    );
}

/// The yosys-scope-specific proof: with `yosys_write_json_scope`, an
/// always-agreeing `ToolReport` whose `localparams` and
/// `package_constants` maps are deliberately empty (matching what
/// yosys would actually report — folded) must compare `Ok(())`. Under
/// the strict `ParityScope::all()` the same empty maps would surface
/// `LocalparamMissingInTool` / `PackageConstantMissingInTool`
/// divergences for every manifest entry.
#[test]
fn yosys_scope_ignores_localparams_and_package_constants() {
    for &seed in SEEDS {
        let manifest = manifest_for(seed);
        let mut report = synthetic_tool_report_from_manifest(&manifest);
        // Empty the categories yosys cannot introspect.
        report.localparams = BTreeMap::new();
        report.package_constants = BTreeMap::new();

        // Under the yosys scope, those folded categories are ignored.
        assert_eq!(
            compare_manifest_to_tool_report_in_scope(&manifest, &report, &yosys_write_json_scope()),
            Ok(()),
            "yosys scope must tolerate folded localparams/package_constants (seed={seed})"
        );

        // Under the strict-all scope, the same empty maps must
        // surface the corresponding `MissingInTool` divergences
        // (assuming the manifest actually carries those categories
        // for this seed — `.2a` always emits >=1 package constant,
        // and may emit 0 or more localparams).
        let strict =
            compare_manifest_to_tool_report_in_scope(&manifest, &report, &ParityScope::all());
        if let Err(divs) = strict {
            // The package_constants map is non-empty for every seed
            // (the `mc_<seed>_pkg::K` entry); so we should always see
            // PackageConstantMissingInTool.
            let (pkg_name, pkg_val) = manifest
                .package_constants
                .iter()
                .next()
                .map(|(n, v)| (n.clone(), *v))
                .expect("manifest carries the mc_<seed>_pkg::K entry");
            assert!(
                divs.contains(&Divergence::PackageConstantMissingInTool {
                    name: pkg_name.clone(),
                    expected: pkg_val,
                }),
                "strict-all-scope should surface PackageConstantMissingInTool \
                 on {pkg_name} (seed={seed}); got {divs:?}"
            );
        } else {
            // Strict-all returning Ok would mean the manifest happened
            // to carry zero localparams AND the yosys-empty
            // localparams/package_constants do agree — that's only
            // possible if both maps are empty on the manifest side,
            // which never happens for our corpus (always >=1 pkg).
            panic!(
                "strict-all-scope unexpectedly returned Ok despite empty \
                 localparams + package_constants in the tool report (seed={seed})"
            );
        }
    }
}

/// `ParityScope::none()` returns `Ok(())` even on a maximally
/// disagreeing report — useful as a self-check on the scoping
/// implementation itself.
#[test]
fn empty_scope_ignores_every_disagreement() {
    let manifest = manifest_for(0);
    let report = ToolReport {
        seed: manifest.seed.wrapping_add(1),
        top: format!("{}_WRONG", manifest.top),
        params: BTreeMap::new(),
        localparams: BTreeMap::new(),
        widths: BTreeMap::new(),
        generate: BTreeMap::new(),
        package_constants: BTreeMap::new(),
    };
    assert_eq!(
        compare_manifest_to_tool_report_in_scope(&manifest, &report, &ParityScope::none()),
        Ok(()),
        "ParityScope::none() must tolerate any disagreement"
    );
}

// ===================================================================
// PHASE-7-ORACLE-MICRODESIGN.2c.2a — yosys `write_json` extractor.
//
// Given a `serde_json::Value` carrying yosys 0.64's `write_json`
// output for a single `mc_<seed>` module, extract the resolved facts
// yosys exposes and pack them into a `ToolReport`. The yosys-scoped
// comparator only inspects the four axes yosys actually carries
// (Seed/Top/Params/Widths/Generate); the extractor leaves the folded
// `localparams` and `package_constants` maps empty, and the gate
// configures `ParityScope` to skip them.
//
// `parse_yosys_binary_param` interprets yosys's `int`-typed parameter
// values, which yosys emits as a fixed-width binary string
// (`"00...101110"` = 46 = `0x2E`). The SV `parameter int` type is
// signed 32-bit, so the parser reads the binary string as `u32` and
// re-interprets it as `i32` (sign-extension) before widening to
// `i128`. The `.2b` builder keeps every value well inside `i32`'s
// range, so no wider-than-32-bit handling is needed.
// ===================================================================

/// Parse a yosys `parameter_default_values` binary string as the SV
/// `parameter int` (signed 32-bit) it represents. Returns `None` on
/// malformed inputs (non-binary chars, or wider than 32 bits).
fn parse_yosys_binary_param(s: &str) -> Option<i128> {
    if s.is_empty() || s.len() > 32 || !s.chars().all(|c| c == '0' || c == '1') {
        return None;
    }
    let u = u32::from_str_radix(s, 2).ok()?;
    Some(u as i32 as i128) // sign-extend through i32
}

/// Build a `ToolReport` from yosys 0.64's `write_json` output for a
/// single `mc_<seed>` module. The extractor populates only what yosys
/// actually carries:
///
///   * `seed` — supplied by the caller (it is the corpus key, not a
///     yosys fact).
///   * `top` — read from `.modules` keys (must contain `mc_<seed>`).
///   * `params` — from `.modules.<top>.parameter_default_values`,
///     interpreted as signed 32-bit per the SV `int` type the `.2b`
///     emitter uses.
///   * `widths["sig"]` — from `.modules.<top>.netnames.sig.bits`
///     (length of the bit-id list = wire width).
///   * `generate["g_taken"]` — `true` iff any netname key is
///     prefixed `g_taken.`. The `.2b` emitter wraps the elaborated
///     content of each branch in `g_taken : begin` / `g_else : begin`,
///     so yosys's surviving prefix tells us which branch was kept.
///
/// `localparams` and `package_constants` are intentionally left empty:
/// yosys 0.64 folds them, and the yosys-scoped comparator skips them.
fn yosys_write_json_to_tool_report(json: &serde_json::Value, seed: u64) -> ToolReport {
    let top = format!("mc_{seed}");
    let module = &json["modules"][&top];

    // Params: parse the binary-string values.
    let mut params: BTreeMap<String, i128> = BTreeMap::new();
    if let Some(pdv) = module["parameter_default_values"].as_object() {
        for (name, value) in pdv {
            if let Some(s) = value.as_str() {
                if let Some(v) = parse_yosys_binary_param(s) {
                    params.insert(name.clone(), v);
                }
            }
        }
    }

    // Generate-branch decision: scan netname-key prefixes.
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
    let mut generate: BTreeMap<String, bool> = BTreeMap::new();
    // The manifest carries one entry, `g_taken`, whose `taken` records
    // whether the `if`-branch was kept. Yosys's surviving prefix is
    // the direct observation.
    let taken_observed = g_taken_alive && !g_else_alive;
    generate.insert("g_taken".to_string(), taken_observed);

    // Width: yosys reports `netnames.sig.bits` as an array of bit-ids;
    // its length is the wire width.
    let mut widths: BTreeMap<String, WidthFact> = BTreeMap::new();
    if let Some(bits_arr) = module["netnames"]["sig"]["bits"].as_array() {
        let bits = bits_arr.len() as u32;
        widths.insert(
            "sig".to_string(),
            WidthFact {
                msb: bits as i64 - 1,
                lsb: 0,
                bits,
            },
        );
    }

    ToolReport {
        seed,
        top,
        params,
        localparams: BTreeMap::new(),
        widths,
        generate,
        package_constants: BTreeMap::new(),
    }
}

/// Sanity proof for the extractor: a hand-constructed
/// yosys-like JSON for seed-0 produces exactly the same
/// `ToolReport` the live tool would produce on `mc_0.sv`. Pure
/// cargo-portable — no `yosys` invocation. Exercises every
/// branch of the extractor (parameter parsing, generate-branch
/// inference, width extraction). The actual end-to-end run is
/// `.2c.2b`.
#[test]
fn yosys_extractor_reads_a_synthetic_write_json_correctly() {
    let synthetic = serde_json::json!({
        "modules": {
            "mc_0": {
                "parameter_default_values": {
                    "P0": "00000000000000000000000000101110"
                },
                "netnames": {
                    "g_taken.gflag": { "hide_name": 0, "bits": ["0"] },
                    "sig": { "hide_name": 0, "bits": ["0","0","0","0","0","0"] }
                }
            }
        }
    });
    let report = yosys_write_json_to_tool_report(&synthetic, 0);
    assert_eq!(report.seed, 0);
    assert_eq!(report.top, "mc_0");
    assert_eq!(report.params.get("P0").copied(), Some(46));
    assert_eq!(report.generate.get("g_taken").copied(), Some(true));
    assert_eq!(
        report.widths.get("sig").cloned(),
        Some(WidthFact {
            msb: 5,
            lsb: 0,
            bits: 6
        })
    );
    // The folded axes are deliberately empty.
    assert!(report.localparams.is_empty());
    assert!(report.package_constants.is_empty());
}

/// Negative-side proof for the extractor: an `else`-surviving
/// netnames map (the `.2b` corpus's seed-12345 case) must produce
/// `generate["g_taken"] = false`. Pure cargo-portable.
#[test]
fn yosys_extractor_reports_g_else_when_else_branch_survives() {
    let synthetic = serde_json::json!({
        "modules": {
            "mc_12345": {
                "parameter_default_values": {
                    "P0": "00000000000000000000000000101011" // 43
                },
                "netnames": {
                    "g_else.gflag": { "hide_name": 0, "bits": ["0"] },
                    "sig": { "hide_name": 0, "bits": ["0","0","0","0","0"] }
                }
            }
        }
    });
    let report = yosys_write_json_to_tool_report(&synthetic, 12345);
    assert_eq!(report.generate.get("g_taken").copied(), Some(false));
    assert_eq!(report.params.get("P0").copied(), Some(43));
}

/// Sign-extension proof: a yosys binary string `"111...1"` must
/// decode to `-1`, not `0xFFFFFFFFu32 as i128`. The `.2b` builder
/// can produce negative resolved values (e.g. seed 7's `P4 = -1`),
/// and the parity gate must preserve the sign.
#[test]
fn parse_yosys_binary_param_sign_extends() {
    assert_eq!(
        parse_yosys_binary_param("11111111111111111111111111111111"),
        Some(-1)
    );
    assert_eq!(
        parse_yosys_binary_param("00000000000000000000000000000001"),
        Some(1)
    );
    assert_eq!(
        parse_yosys_binary_param("00000000000000000000000000101110"),
        Some(46)
    );
    assert_eq!(parse_yosys_binary_param(""), None);
    assert_eq!(parse_yosys_binary_param("01z"), None);
    assert_eq!(
        parse_yosys_binary_param("100000000000000000000000000000000"),
        None
    ); // 33 bits
}

// ===================================================================
// PHASE-7-ORACLE-MICRODESIGN.2c.2a — end-to-end-runnable `#[ignore]`.
//
// `parity_against_real_yosys_write_json` is `#[ignore]` so the
// portable `cargo test` stays green tool-less (Phase-1 doctrine;
// mirrors DIFFERENTIAL-SIMULATION `.2b`). `.2c.2a` lands the
// full corpus drive (no longer a scaffold-with-placeholder); `.2c.2b`
// is the gated step that runs it end-to-end against real yosys and
// banks a verified-clean artifact before ROADMAP Phase 7 → done.
// ===================================================================

/// Real-tool parity gate, invocable with
/// `cargo test -- --ignored parity_against_real_yosys_write_json`.
///
/// For each seed in the reproducibility set:
/// 1. Build `unit` via `build_constexpr_unit(seed, N_PARAMS)`.
/// 2. Write `emit_sv(&unit, seed)` to
///    `target/microdesign-parity/mc_<seed>.sv`.
/// 3. Write `emit_manifest(&unit, seed)` to
///    `target/microdesign-parity/mc_<seed>.json`.
/// 4. Shell yosys with the canonical script
///    (`read_verilog -sv … ; hierarchy -top mc_<seed>; proc; opt;
///    write_json …`) into
///    `target/microdesign-parity/mc_<seed>.yosys.json`.
/// 5. Parse the yosys output → `ToolReport` via
///    `yosys_write_json_to_tool_report`.
/// 6. Call `compare_manifest_to_tool_report_in_scope(manifest,
///    report, &yosys_write_json_scope())` and assert `Ok(())`
///    (or, on disagreement, dump the counterexample tuple
///    `{sv, manifest, yosys.json, divergences}` and fail).
///
/// On `yosys` absent: the test is a friendly no-op (matches the
/// `iverilog`-not-installed convention from
/// `DIFFERENTIAL-SIMULATION.1`).
#[test]
#[ignore]
fn parity_against_real_yosys_write_json() {
    // Tool presence guard.
    let probe = std::process::Command::new("yosys").arg("-V").output();
    if probe.is_err() || !probe.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        eprintln!(
            "parity_against_real_yosys_write_json: yosys not on $PATH \
             (skipping; rerun with yosys installed for the real-tool gate)"
        );
        return;
    }

    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("microdesign-parity-phase7-yosys");
    std::fs::create_dir_all(&dir).expect("can create the harness output dir");

    let scope = yosys_write_json_scope();
    let mut counterexamples: Vec<(u64, Vec<Divergence>)> = Vec::new();

    for &seed in SEEDS {
        let unit = build_constexpr_unit(seed, N_PARAMS);
        let manifest = build_manifest(&unit, seed);

        let sv_path = dir.join(format!("mc_{seed}.sv"));
        let manifest_path = dir.join(format!("mc_{seed}.json"));
        let yosys_path = dir.join(format!("mc_{seed}.yosys.json"));
        std::fs::write(&sv_path, emit_sv(&unit, seed)).expect("write .sv");
        std::fs::write(&manifest_path, emit_manifest(&unit, seed)).expect("write manifest .json");

        let script = format!(
            "read_verilog -sv {sv}; hierarchy -top mc_{seed}; proc; opt; write_json {out}",
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
        let report = yosys_write_json_to_tool_report(&yosys_json, seed);

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
