# SV-VERSION-TARGETING: target a chosen IEEE 1800 standard valid-by-construction

## Metadata

- Tree ID: `SV-VERSION-TARGETING`
- Status: `done`
- Roadmap lane: `Capability / breadth — version-targeted synthesizable RTL (ROADMAP steering gaps 1 + 3)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (`.3b.2b` landed — the repo-owned matrix up-opt gate: a tenth `--sv-version-gate` scenario (`soft_union_slice_prob = 1.0` + `sv_version 2023`) emitting the live `union soft` overlay, Verilator-only (Yosys/Icarus recorded no-op), with the `saw_sv_version_2023_soft_union_upopt` coverage fact enforced; banked clean `/tmp/anvil-sv-version-gate-upopt-r1`. Closes `.3b.2`/`.3b`/`.3` and the whole tree. **Tree CLOSED.** Further version-distinctive up-opts are open-ended post-tree breadth — no finite completion point, not a blocker, nothing retired.)
- Owner: repo-local workflow
- Note: opened `2026-06-15` by owner roadmap steering as the recommended
  highest-leverage capability lane (over the two registered-`proposed` siblings
  `STRUCTURED-EMISSION-EXPANSION` and `SEMANTIC-INTROSPECTION-EXPANSION`).

## Goal

Give ANVIL a `--sv-version <2012|2017|2023>` capability gate
(`Config::sv_version`) that makes the generator/emitter target a chosen IEEE
1800 SystemVerilog standard **valid-by-construction**, serving the north star
(`project_anvil_north_star`): expose version-specific downstream-tool bugs via
legal, standard-valid, downstream-acceptance-quality output. Two effects, both
rules-first: **down-gating** (never emit a construct newer than the target — a
standard-validity guarantee) and **up-opting** (deliberately emit a higher
standard's distinctive synthesizable constructs, each proven downstream-clean in
the matching tool standard mode). Default reproduces today's output
byte-identical.

## Non-Goals

- No generate-then-filter: the version is a construction-time capability bound,
  not a post-hoc reject (`feedback_rules_first_generation`).
- No default output change: the default `--sv-version` is byte-identical to
  today (`tests/snapshots.rs` untouched); the gate is opt-in
  (`feedback_never_retire_strategies`).
- No aspirational constructs: an up-opted construct lands only once proven
  accepted by a downstream tool in its matching standard mode.
- Not classic Verilog / SV-2005: ANVIL emits SystemVerilog; the floor is the
  2012-era synthesizable SV subset.

## Acceptance Criteria

- A `Config::sv_version` enum + `--sv-version` CLI flag + `--dump-config` /
  introspection field; the default value is byte-identical to current emission.
- The emitter (and any version-relevant generator choice) honours the target as
  a read-only capability bound; down-gating is a guarantee.
- A per-version downstream acceptance axis proves the targeted corpus is accepted
  in the matching tool standard mode, with retained seed + `sv_version` + knobs
  counterexamples.
- Each up-opted version-distinctive construct is design-first and proven
  downstream-clean before default-on for that version.
- Live docs (book chapter, USER_GUIDE, README CLI truth, ROADMAP, knobs) updated
  where the surface changes; a Knowledge Map fact per durable capability/boundary.
- Every leaf committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `SV-VERSION-TARGETING`
  Status: `done`
  Goal: `Version-targeted valid-by-construction SystemVerilog emission.`
  Children: `SV-VERSION-TARGETING.1`, `SV-VERSION-TARGETING.2`, `SV-VERSION-TARGETING.3`
  Result: `Complete (closed 2026-06-16). .1 (decision 0009) + .2 (plumbing + down-gating + per-version downstream acceptance axis) + .3 (the first version-distinctive up-opt — the IEEE 1800-2023 union soft overlay, live and proven both as a generator capability (.3b.2a) and as a repo-owned matrix gate (.3b.2b)) all done. Default byte-identical throughout. Further version-distinctive up-opts are open-ended post-tree breadth (no finite completion point, not a blocker, nothing retired).`

- ID: `SV-VERSION-TARGETING.1`
  Status: `done`
  Goal: `Design/decision leaf: fix the gate semantics (down-gating guarantee + up-opting), the default (byte-identical) value, the valid-by-construction discipline, the per-version downstream acceptance gate, the first-increment scope, and rejected alternatives — before any code.`
  Acceptance: `A decision record naming the gate, its two construction-time effects, its byte-identical default, its downstream proof, its first-increment scope, and its rejected alternatives; no source change; docs/workflow self-checks clean.`
  Result: `Decision 0009 — opt-in --sv-version <2012|2017|2023> gate (Config::sv_version). Down-gating = never emit a construct newer than the target (standard-validity guarantee); up-opting = deliberately emit a higher standard's distinctive synthesizable constructs, each proven downstream-clean in the matching tool mode. Default = the floor value byte-identical to today's emission (tests/snapshots untouched). Rules-first (construction-time bound, no generate-then-filter). Per-version downstream acceptance axis (verilator --language 1800-20xx, yosys -sv, iverilog -g2012 gated/no-op beyond its newest generation). First increment (.2) = plumbing + down-gating + per-version acceptance over the existing subset; first up-opted construct = .3 (design-first). Tree split into .1 (done) + .2 (impl) + .3 (future up-opt).`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.2`
  Status: `done`
  Goal: `Implement the plumbing + down-gating + per-version acceptance axis over the existing subset (default byte-identical).`
  Children: `SV-VERSION-TARGETING.2a`, `SV-VERSION-TARGETING.2b`
  Result: `Complete. .2a (design detail) + .2b.1 (SvVersion knob + versioned emitter capability bound + introspection 1.2) + .2b.2a (downstream --language selector + focused real-tool proof) + .2b.2b (repo-owned tool_matrix --sv-version-gate + per-version coverage facts, banked clean) all done. Plumbing + down-gating + per-version downstream acceptance axis delivered; default byte-identical throughout. Remaining lane frontier is .3 (first up-opted construct).`

- ID: `SV-VERSION-TARGETING.2a`
  Status: `done`
  Goal: `Design-detail leaf: resolve decision 0009's five open questions before code — the SvVersion enum spelling + the byte-identical floor default value, where the capability bound lives and how it threads to the emitter, the down-gating byte-identity proof shape, the introspection field + schema MINOR-bump procedure, and the per-version downstream acceptance axis shape (Verilator language selector, Yosys/Icarus handling, the gate shape). Split .2 into .2a + .2b and pre-split .2b into .2b.1 + .2b.2.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving all five open questions grounded in the real src/config.rs / src/emit/sv.rs / src/introspect/mod.rs / src/downstream/mod.rs / src/bin/tool_matrix.rs code; the task tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `SvVersion { Sv2012 < Sv2017 < Sv2023 } (PartialOrd/Ord) in src/config.rs, bare-year CLI/serde value names, default = Sv2012 (the honest floor; byte-identical; down-gating to the floor is a provable no-op — supersedes decision 0009's "working name Sv2017"). Bound threads to the emitter as a parameter (NOT onto the IR — keeps CSE keys / canonical_module_signature / Module-serde untouched): new to_sv_versioned / to_sv_in_design / to_sv_design versioned entry points, old ones delegate with SvVersion::default() so every caller stays byte-identical; SvVersion::permits(introduced) predicate is the bound, gating nothing in .2b.1 (whole subset <= 2012). Down-gating proof = a cross-version byte-identity test over a corpus. Introspection: serde-automatic; schema MINOR bump 1.1 -> 1.2 + 5 "1.1" test-assertion updates. Per-version axis (.2b.2): SvVersion::verilator_language_arg -> "1800-20xx", optional --language selector on run_verilator* (None = today's argv), Yosys stays -sv, Icarus -g2012 runs on the g2012-valid subset; focused --sv-version-gate + ScenarioSet::SvVersionSweep mirroring --signoff-knob-sweep-gate. .2b pre-split: .2b.1 knob+emitter bound (byte-identical), .2b.2 downstream acceptance axis.`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.2b`
  Status: `done`
  Goal: `Implement the .2a design: knob plumbing + emitter capability bound (.2b.1) and the per-version downstream acceptance axis (.2b.2).`
  Children: `SV-VERSION-TARGETING.2b.1`, `SV-VERSION-TARGETING.2b.2`
  Result: `Complete. .2b.1 (knob + emitter capability bound) + .2b.2 (downstream acceptance axis: .2b.2a selector + .2b.2b matrix gate) both done. Default byte-identical.`

- ID: `SV-VERSION-TARGETING.2b.1`
  Status: `done`
  Goal: `Config::sv_version (SvVersion enum) + --sv-version CLI + Overrides + apply_cli_overrides; --dump-config + --introspect surface it (serde-automatic) with schema MINOR bump 1.1 -> 1.2 (+ schema doc + 5 test-assertion updates); SvVersion::permits capability bound threaded through new versioned emitter entry points (old entry points delegate with SvVersion::default()); DUT emit sites pass cfg.sv_version; a cross-version byte-identity test proving the current subset is a 2012/2017/2023 common floor; USER_GUIDE/book(knobs+new surface)/README/knobs docs.`
  Acceptance: `cargo fmt/check/clippy --all-targets -D warnings clean; cargo test --lib green; default --sv-version byte-identical (tests/snapshots.rs 6/6 untouched); cross-version byte-identity test passes; --dump-config + --introspect expose sv_version; schema_version = 1.2 everywhere; book/USER_GUIDE/README/knobs updated; committed through COMMIT.md with the leaf id.`
  Result: `SvVersion {Sv2012<Sv2017<Sv2023} (Ord, #[default] Sv2012, bare-year CLI/serde spelling) + permits()/ieee_standard() in src/config.rs; Config::sv_version (#[serde(default)]) + Overrides + apply + --sv-version CLI + 2 config unit tests. Emitter: to_sv_versioned/to_sv_in_design_versioned/to_sv_design_versioned in src/emit/sv.rs (+ re-exports in src/emit/mod.rs); old to_sv* delegate with SvVersion::default() ⇒ byte-identical; sv_version threaded into to_sv_with_modules (info! trace only), bound gates nothing yet (subset ≤2012). Threaded cfg.sv_version at all DUT emit sites: main (stdout + --out), introspect (sv_len), mcp (generate), umbrella (DutLane). Introspection schema 1.1→1.2 (SCHEMA_VERSION + schema doc changelog/version/self-check + 5 "1.1" test assertions). New tests/sv_version.rs (cross-version byte-identity over leaf + design spreads). Verified: cargo check/clippy(-D warnings)/fmt clean; cargo test --lib 405/0; snapshots 6/6; tests/sv_version 2/2; CLI smoke default==2012==2023 md5-equal, dump-config/introspect carry field + schema 1.2, bad value rejected. tool_matrix/downstream deferred to .2b.2.`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.2b.2`
  Status: `done`
  Goal: `Per-version downstream acceptance axis.`
  Children: `SV-VERSION-TARGETING.2b.2a`, `SV-VERSION-TARGETING.2b.2b`
  Result: `Complete. .2b.2a landed the downstream --language selector + a focused #[ignore] real-tool proof; .2b.2b industrialized it into the repo-owned tool_matrix --sv-version-gate with per-version coverage facts, banked downstream-clean.`

- ID: `SV-VERSION-TARGETING.2b.2a`
  Status: `done`
  Goal: `Downstream --language selector + a focused real-tool per-version acceptance proof: add language: Option<&str> to run_verilator(_design) (None = today's exact argv; Some = --language 1800-20xx, spelling probed against the installed Verilator first); an #[ignore]-gated test that runs Verilator at each --language mode (clean) + Icarus -g2012 on a representative corpus, banked clean.`
  Acceptance: `cargo fmt/check/clippy(-D warnings)/test --lib clean; default tool invocation byte-identical (selector None; existing callers pass None); the #[ignore] gate banked clean against the installed Verilator + Icarus; CODEBASE_ANALYSIS + DEVELOPMENT_NOTES updated; committed through COMMIT.md with the leaf id.`
  Result: `Probed Verilator 5.046: both --language and --default-language accept 1800-2012/2017/2023 and lint clean; chose --language <std> (the documented standard selector). run_verilator(_design) gained language: Option<&str> in src/downstream/mod.rs (Some prepends --language <std>; None = byte-identical argv); 4 callers (validate ×2, tool_matrix ×2) pass None. New tests/sv_version_downstream.rs (#[ignore]): leaf corpus (comb/seq/structured/memory/fsm) + hierarchy design × 3 versions; asserts Verilator --language clean + Icarus -g2012 accepts. Banked clean: 2 passed / 6.18s vs Verilator 5.046 + Icarus 13.0. cargo test --lib 405/0; snapshots 6/6; clippy/fmt clean.`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.2b.2b`
  Status: `done`
  Goal: `Repo-owned per-version gate in src/bin/tool_matrix.rs: --sv-version-gate CLI flag + ScenarioSet::SvVersionSweep (mirror --signoff-knob-sweep-gate) sweeping the three targets, running Verilator in the matching --language mode (via the .2b.2a selector) + threading cfg.sv_version into the matrix to_sv* emits; a saw_sv_version_targeted_acceptance coverage fact (+ per-version sub-facts) under coverage_gaps enforcement; MatrixReport.sv_version_gate field; banked clean against real Verilator + Yosys; ROADMAP/README/USER_GUIDE/book + KM docs.`
  Acceptance: `cargo fmt/check/clippy/test (incl. heavy tests/pipeline.rs once) clean; the gate runs the three targets downstream-clean in the matching tool standard mode with coverage_gaps = []; banked-clean evidence recorded; default matrix run byte-identical (selector None unless the gate is active); docs + KM updated; committed through COMMIT.md with the leaf id.`
  Result: `All in src/bin/tool_matrix.rs. --sv-version-gate → ScenarioSet::SvVersionSweep (mutually exclusive, auto fail-on-coverage-gap, SV_VERSION_SWEEP_MIN_UNITS_PER_SCENARIO=2). build_sv_version_sweep_scenarios: per target (2012/2017/2023) × {comb e-graph leaf, seq motif leaf, recursive depth-2 hierarchy design} = 9 Interleaved scenarios, each carrying Config::sv_version. Emit threaded via to_sv_versioned / to_sv_in_design_versioned (byte-identical at the Sv2012 floor every non-gate scenario uses). verilator_language_for(scenario, version_targeted) → Some(ieee_standard()) only under the gate (the .2b.2a run_verilator(_design) selector), else None. version_targeted + sv_version + verilator_language threaded through run_scenario → run_{module,design}_scenario → prepare_design / materialize_* / run_{module,design}_tools / resume_existing_{module,design}. CoverageSummary gains saw_sv_version_targeted_acceptance (umbrella) + saw_sv_version_{2012,2017,2023}_targeted_acceptance, lit by light_sv_version_acceptance from summarize_{coverage,design_coverage} only when version_targeted AND Verilator ran-and-succeeded AND Yosys clean; merged in merge_coverage; enforced by an early-return arm in compute_coverage_gaps placed BEFORE the strategy loop (Interleaved-only sweep valid). MatrixReport.sv_version_gate field. 6 new cargo-portable proofs (flag parse, set-select+plan, mutual-excl, 9-scenario shaping, verilator_language_for on/off, gap requirements incl. no-strategy-gap). Banked clean: /tmp/anvil-sv-version-gate-r1 — 9 scenarios / 18 units / coverage_gaps=[] / Verilator 18/0 / Yosys without-abc 18/0 / with-abc 18/0; report confirms each scenario's Verilator argv carries the matching --language 1800-20xx and all four saw_sv_version_* facts lit.`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.3`
  Status: `done`
  Goal: `The first version-distinctive up-opted synthesizable construct, design-first then impl, gated on sv_version >= that_standard, proven downstream-clean in the matching tool standard mode.`
  Children: `SV-VERSION-TARGETING.3a`, `SV-VERSION-TARGETING.3b`
  Result: `Complete. .3a (decision 0010 — first up-opt = heterogeneous-width packed union soft, IEEE 1800-2023 §7.3.1) + .3b (impl: .3b.1 mechanism design-detail + .3b.2 = .3b.2a generator capability + .3b.2b repo-owned matrix up-opt gate) all done. The live union soft overlay is proven downstream-clean by Verilator --language 1800-2023 both in tests/sv_version_downstream.rs and the banked --sv-version-gate report; Yosys/Icarus recorded no-op. Default byte-identical.`

- ID: `SV-VERSION-TARGETING.3a`
  Status: `done`
  Goal: `Design leaf: pick the first up-opted construct (which construct, why genuinely version-distinctive + synthesizable + tool-accepted), the construction-time gate, the rules-first / default-off / byte-identical discipline, and the per-version downstream-proof handling — grounded in a real probe of the installed tools. Split .3 into .3a + .3b.`
  Acceptance: `A decision record naming the first up-opt construct, its LRM/version teeth, its tool-acceptance evidence, the sv_version >= that_standard gate, the byte-identical default, the rejected alternatives, and the .3b impl shape; no source change; docs/workflow self-checks clean.`
  Result: `Decision 0010. First up-opt = a heterogeneous-width packed union emitted as union soft (IEEE 1800-2023 §7.3.1), a new default-off aggregate projection (sibling of AggregateKind::StructPacked/ArrayPacked) gated on sv_version >= Sv2023. Down-gate fallback < 2023 = the existing packed struct projection ⇒ default byte-identical. Empirical finding (probe of Verilator 5.046 / Yosys 0.64 / Icarus 13.0): the installed tools do NOT enforce 1800-version acceptance (Verilator accepts/​reserves identically across --language 1800-2012/2017/2023; Yosys/Icarus have no 1800 selector + fixed subset), so the up-opt's teeth are LRM correctness + construction-time down-gating + matching-mode acceptance (verilator --language 1800-2023, proven by --binary y=a5), NOT tool-side rejection. Real down-gating teeth confirmed: a NON-soft heterogeneous-width packed union is rejected by all three tools (Verilator cites "Hard packed union members must have equal size (IEEE 1800-2023 7.3.1)"). Yosys/Icarus reject the union soft syntax ⇒ recorded no-op for the up-opt scenario (0009's authorized path); the existing saw_sv_version_2023_targeted_acceptance fact requires Yosys-clean, so .3b adds a dedicated saw_sv_version_2023_soft_union_upopt fact requiring only Verilator matching-mode acceptance. Tree split .3 → .3a (done) + .3b (impl).`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.3b`
  Status: `done`
  Goal: `Implement the union soft up-opt gated on sv_version >= Sv2023, default-off / byte-identical, proven Verilator matching-mode-clean (Yosys/Icarus recorded no-op).`
  Children: `SV-VERSION-TARGETING.3b.1`, `SV-VERSION-TARGETING.3b.2`
  Result: `Complete. .3b.1 (mechanism design-detail) + .3b.2 (= .3b.2a generator capability + .3b.2b repo-owned matrix up-opt gate) both done.`

- ID: `SV-VERSION-TARGETING.3b.1`
  Status: `done`
  Goal: `Design-detail leaf: resolve the mechanism 0010 left open (port-boundary union fold vs internal-only union soft overlay) against the real src/ir/aggregate.rs + src/emit/sv.rs code, and fix the .3b.2 impl shape. Split .3b into .3b.1 + .3b.2.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the mechanism grounded in real code, the tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Mechanism = an emitter-level, sv_version >= Sv2023-gated, default-off alternative rendering of a PROPER LOW-BITS Slice gate (GateOp::Slice{hi, lo:0} over a non-constant multi-bit source, hi < W-1) as an internal union soft overlay: typedef union soft { logic[W-1:0] w; logic[hi:0] n; } u; assign u.w = src; slice_wire = u.n. Behaviour-preserving (packed-union members are LSB-aligned ⇒ u.n == src[hi:0], confirmed by the .3a probe y=a5), genuinely 2023, Verilator-accepted, surgical (decl+drive in the gate region; render_gate stays a pure expression; member ref via the existing name machinery like MemRead/FsmOut). REJECTED: an AggregateKind union sibling (a union is not concatenation-equivalent ⇒ breaks the bijective semantically-empty boundary-aggregate invariant) and the port-boundary union fold (changes the interface ⇒ generation-time, large blast radius; deferred breadth, nothing retired). .3b.2 scope: a default-off knob (working name soft_union_slice_prob) rolled rules-first + the permits(Sv2023) gate; emitter overlay; tests/sv_version.rs divergence at 2023; tests/sv_version_downstream.rs verilator --language 1800-2023; a matrix up-opt scenario + saw_sv_version_2023_soft_union_upopt fact (Verilator-only, Yosys/Icarus no-op); book/USER_GUIDE/README/ROADMAP + KM; snapshots 6/6 byte-identical (knob off). Open verification risk: confirm the overlay is verilator --lint-only warning-clean (real drives, not the toy -Wall artifact).`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.3b.2`
  Status: `done`
  Goal: `Implement the .3b.1 mechanism + industrialize it in the repo-owned matrix.`
  Children: `SV-VERSION-TARGETING.3b.2a`, `SV-VERSION-TARGETING.3b.2b`
  Result: `Complete. .3b.2a landed the live generator capability (the union soft overlay, default-off / byte-identical, banked Verilator --language 1800-2023 clean); .3b.2b industrialized it into the repo-owned tool_matrix --sv-version-gate with the saw_sv_version_2023_soft_union_upopt coverage fact, banked downstream-clean.`

- ID: `SV-VERSION-TARGETING.3b.2a`
  Status: `done`
  Goal: `Core capability: the default-off soft_union_slice_prob knob + the gen-time annotation pass + the permits(Sv2023)-gated internal union soft overlay rendering of a proper low-bits Slice, default-off / byte-identical, with the divergence/down-gate proofs + a banked Verilator --language 1800-2023 acceptance test. Defer the repo-owned matrix gate to .3b.2b. Split .3b.2 into .3b.2a + .3b.2b.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green; snapshots 6/6 byte-identical (knob default-off); a lib divergence proof (overlay only at Sv2023, down-gate below, byte-identical when off); tests/sv_version_downstream.rs (#[ignore]) banked clean vs verilator --language 1800-2023; book(knobs/sv-version)/USER_GUIDE/README/ROADMAP/CODEBASE_ANALYSIS + KM fact updated; committed through COMMIT.md with the leaf id.`
  Result: `Config::soft_union_slice_prob (default 0.0, serde-only, validated 0.0..=1.0) in src/config.rs; new Module.soft_union_slice_gates: BTreeSet<NodeId> (emitter-surface marker, not hashed into canonical_module_signature; Module has no serde so zero snapshot impact); new src/ir/soft_union.rs annotate_soft_union_slices(m, rng, prob) gen-time pass (rolls per qualifying gate; skips param_env modules) wired into generate_module + generate_design like aggregate_prob; emit/sv.rs soft_union_slice_overlay helper + the union soft var decl + the u.w=src / gate=u.n assigns, gated on SvVersion::permits(Sv2023). 8 lib proofs (5 pass unit + param-env skip + emit divergence/down-gate) + the byte-identity corpus still green. Banked downstream-clean: tests/sv_version_downstream.rs::verilator_accepts_soft_union_slice_overlay_at_2023 → 159 overlays across 7 seeds, all Verilator --language 1800-2023 clean, 2012 down-gate confirmed. cargo check/clippy(-D warnings)/fmt clean; cargo test --lib 412/0 (2 ignored); snapshots 6/6 byte-identical; mdbook build clean; book_examples 3/3. Verilator-only proof (Yosys/Icarus reject union soft → recorded no-op, decision 0010); matrix gate + saw_sv_version_2023_soft_union_upopt fact deferred to .3b.2b.`
  Verification: `done`
  Commit: `done`

- ID: `SV-VERSION-TARGETING.3b.2b`
  Status: `done`
  Goal: `Industrialize the up-opt in the repo-owned tool_matrix: a dedicated --sv-version-gate up-opt scenario (soft_union_slice_prob = 1.0 + sv_version 2023 + a slice-heavy config) + a saw_sv_version_2023_soft_union_upopt coverage fact that requires Verilator --language 1800-2023 acceptance and records Yosys/Icarus as a no-op (the existing .2b.2b facts require Yosys-clean, so the union scenario needs its own fact), banked clean.`
  Acceptance: `cargo check/clippy/fmt/test clean; the gate produces + verilator-accepts the overlay in 1800-2023 mode with Yosys/Icarus recorded no-op; saw_sv_version_2023_soft_union_upopt enforced under coverage_gaps; banked-clean evidence recorded; default matrix byte-identical; ROADMAP/README/USER_GUIDE/book + KM updated; committed through COMMIT.md with the leaf id.`
  Result: `All in src/bin/tool_matrix.rs. build_sv_version_sweep_scenarios gains a tenth scenario sv2023_soft_union_upopt (soft_union_upopt_config: the proven .3b.2a slice-heavy recipe — gate_struct_weight 10, widths 4..16, soft_union_slice_prob 1.0, sv_version 2023, Interleaved). scenario_emits_soft_union_overlay(scenario) = soft_union_slice_prob > 0.0 && sv_version.permits(Sv2023) detects it; threaded as verilator_only through run_module_scenario → resume_existing_module / materialize_prepared_module → run_module_tools (Yosys vec empty + Icarus None ⇒ recorded no-op). ModuleReport.emitted_soft_union_overlay (from prepared.sv_text.contains("union soft")) gives summarize_coverage honest emission evidence; the new CoverageSummary.saw_sv_version_2023_soft_union_upopt fact is lit when version_targeted && sv_version == Sv2023 && emitted_soft_union_overlay && Verilator succeeded (never Yosys), merged in merge_coverage, enforced by compute_coverage_gaps' SvVersionSweep arm. The general per-version fact lighting gained a !yosys.is_empty() honesty guard so the Yosys-no-op union module cannot vacuously light the Yosys-requiring fact. 4 cargo-portable proofs updated/added (10-scenario count, target-each-version arm, gaps require the up-opt fact, the new sv_version_sweep_has_verilator_only_soft_union_upopt_scenario). Two #[allow(clippy::too_many_arguments)] on the two now-8-arg plumbing fns (matches the repo's wide-plumbing precedent). Banked downstream-clean /tmp/anvil-sv-version-gate-upopt-r1: 10 scenarios / 20 units / coverage_gaps=[] / Verilator 20/0 / Yosys without-abc 18/0 / with-abc 18/0; the up-opt scenario's 2 modules emitted real union soft overlays (1 + 28 occurrences), Verilator --language 1800-2023 accepted, Yosys ran 0 times. Default matrix byte-identical (verilator_only false for every non-up-opt scenario; snapshots 6/6). Closes .3b.2/.3b/.3 and the whole tree.`
  Verification: `done`
  Commit: `done`

## Current Frontier

**None — tree `done` (closed `2026-06-16`).** Down-gating + up-opting +
per-version downstream acceptance axis all delivered; the first
version-distinctive up-opt (the IEEE 1800-2023 `union soft` overlay) ships
both as a generator capability and as a repo-owned matrix gate. Further
version-distinctive up-opts are open-ended post-tree breadth (no finite
completion point, not a blocker, nothing retired).

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SV-VERSION-TARGETING.3b.2b` | `done` | Industrialized the now-live up-opt into the repo-owned `tool_matrix --sv-version-gate`: a tenth `sv2023_soft_union_upopt` scenario (slice-heavy, `soft_union_slice_prob = 1.0`, 2023) + the `saw_sv_version_2023_soft_union_upopt` coverage fact requiring Verilator `--language 1800-2023` acceptance of a genuinely-emitted overlay with Yosys/Icarus recorded no-op (Verilator-only via `scenario_emits_soft_union_overlay`). Banked clean `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units / `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0 both modes). Closes the tree. |
| — | `SV-VERSION-TARGETING.3b.2a` | `done` | Landed the live first up-opt: `Config::soft_union_slice_prob` + `src/ir/soft_union.rs` gen-time pass + the `permits(Sv2023)`-gated emitter `union soft` overlay of a proper low-bits `Slice`; default-off / byte-identical (snapshots 6/6); banked Verilator `--language 1800-2023` clean (159 overlays / 7 seeds) with the 2012 down-gate proven. Split `.3b.2` → `.3b.2a`/`.3b.2b`. |
| — | `SV-VERSION-TARGETING.3b.1` | `done` | Resolved `0010`'s mechanism open question: the up-opt is an emitter-level `sv_version >= Sv2023`-gated internal `union soft` overlay rendering of a proper low-bits `Slice` (behaviour-preserving, surgical), **not** an `AggregateKind` sibling (a union is not concatenation-equivalent) nor a port-boundary fold (changes the interface). Split `.3b` → `.3b.1`/`.3b.2`. No source change. |
| — | `SV-VERSION-TARGETING.3a` | `done` | Landed decision `0010` — first up-opt = heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1); the empirical tool-reality finding (installed tools don't enforce 1800-version acceptance); the `sv_version >= Sv2023` gate; default byte-identical (struct down-gate fallback); rejected alternatives; the `.3b` impl shape. No source change. |
| — | `SV-VERSION-TARGETING.2b.2b` | `done` | Landed the repo-owned `tool_matrix --sv-version-gate` + `ScenarioSet::SvVersionSweep` (9 scenarios) + per-version emit threading + matching-mode Verilator (`verilator_language_for`) + `saw_sv_version_*_targeted_acceptance` coverage facts + `MatrixReport.sv_version_gate` + 6 proofs. Banked clean: `/tmp/anvil-sv-version-gate-r1` (9 scenarios / 18 units / `coverage_gaps=[]` / 18/0 Verilator + both Yosys). Default matrix byte-identical. |
| — | `SV-VERSION-TARGETING.2b.2a` | `done` | Landed the `run_verilator(_design)` `language: Option<&str>` selector (`--language 1800-20xx`, spelling probed; `None` = byte-identical) + `tests/sv_version_downstream.rs` (`#[ignore]`) banked clean: Verilator accepts all 3 `--language` modes + Icarus `-g2012` accepts. |
| — | `SV-VERSION-TARGETING.2b.1` | `done` | Landed the `SvVersion` enum + `Config::sv_version` + `--sv-version` CLI + versioned emitter entry points (`permits` capability bound) + introspection schema `1.1→1.2` + `tests/sv_version.rs` cross-version byte-identity proof. Default byte-identical (snapshots 6/6). |
| — | `SV-VERSION-TARGETING.2a` | `done` | Resolved decision `0009`'s five open questions; split `.2` → `.2a`/`.2b` and pre-split `.2b` → `.2b.1`/`.2b.2`. No source change. |
| — | `SV-VERSION-TARGETING.1` | `done` | Landed decision `0009` — gate semantics, byte-identical default, valid-by-construction discipline, per-version downstream proof, first-increment scope, rejected alternatives. No source change. |

## Decisions

- `2026-06-16` (`.3b.2b`, impl): industrialized the up-opt into the
  repo-owned `tool_matrix --sv-version-gate` and **closed the tree**.
  Design choices: (1) the up-opt is a *tenth* scenario inside the existing
  `SvVersionSweep` (not a new `ScenarioSet`) — the task asked for "a
  dedicated `--sv-version-gate` up-opt scenario". (2) The Yosys/Icarus
  no-op is derived from the scenario config (`scenario_emits_soft_union_overlay`
  = `soft_union_slice_prob > 0.0 && sv_version.permits(Sv2023)`) and
  threaded as a `verilator_only` bool through the existing module-path
  plumbing — the tool plan is a pure function of the config, not a separate
  `Scenario` field. **Rejected** adding a `Metrics` counter for the overlay
  (it would force an introspection schema MINOR bump 1.3→1.4 touching the
  schema doc + MCP/introspect assertions + DUT-facing output — too wide for
  a matrix-scoped leaf); chose a matrix-local `ModuleReport.emitted_soft_union_overlay`
  bool (from the emitted SV text — proves *actual emission*, not just the
  knob), which also carries through checkpoints for resume correctness.
  (3) Added a `!yosys.is_empty()` honesty guard to the general per-version
  fact lighting so the new Yosys-no-op union module cannot vacuously light
  the Yosys-requiring `saw_sv_version_2023_targeted_acceptance` fact via
  `all_yosys_invocations_ok(&[])`. The new fact requires only matching-mode
  Verilator acceptance of a genuinely-emitted overlay — never Yosys. Banked
  clean `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units /
  `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0 both modes). Default
  matrix byte-identical (snapshots 6/6). Closes `.3b.2`/`.3b`/`.3` and the
  whole tree.
- `2026-06-16` (`.3b.2a`, impl): landed the live first up-opt exactly per the
  `.3b.1` mechanism. `Config::soft_union_slice_prob` (default `0.0`, serde-only,
  validated `0.0..=1.0`); a new emitter-surface `Module.soft_union_slice_gates`
  (`BTreeSet<NodeId>`, not hashed into identity — `Module` derives no serde so
  there is zero snapshot impact); `src/ir/soft_union.rs::annotate_soft_union_slices`
  gen-time pass rolled at the `generate_module` + `generate_design` call sites
  (skips `param_env` modules); `emit/sv.rs::soft_union_slice_overlay` realizes the
  `union soft` overlay only under `SvVersion::permits(Sv2023)`, down-gating to the
  plain slice below. Default-off / byte-identical (snapshots 6/6). The divergence
  proof lives in-crate (`src/ir/soft_union.rs` tests) because hand-building a
  `Module` needs crate-private CSE fields; the real-tool acceptance is
  `tests/sv_version_downstream.rs` (banked: 159 overlays / 7 seeds, Verilator
  `--language 1800-2023` clean, 2012 down-gate proven). Verilator-only proof
  (Yosys/Icarus reject `union soft` → recorded no-op per `0010`). Split `.3b.2`
  → `.3b.2a` (done) + `.3b.2b` (the repo-owned matrix up-opt gate +
  `saw_sv_version_2023_soft_union_upopt` fact).
- `2026-06-16` (`.3b.1`, design-detail in `DEVELOPMENT_NOTES.md`): resolved the
  mechanism `0010` left open. The up-opt is an **emitter-level, `sv_version >=
  Sv2023`-gated, default-off alternative rendering of a proper low-bits `Slice`**
  (`GateOp::Slice { hi, lo: 0 }`, non-constant multi-bit source, `hi < W-1`) as an
  internal `union soft` overlay (`typedef union soft { logic[W-1:0] w; logic[hi:0]
  n; }`; `assign u.w = src;` then `slice = u.n`). Behaviour-preserving
  (packed-union members are LSB-aligned ⇒ `u.n == src[hi:0]`, confirmed by the
  `.3a` `y=a5` probe), genuinely 2023 (heterogeneous-width members are legal only
  as `union soft`), Verilator-accepted, surgical (decl+drive in the gate region;
  `render_gate` stays a pure expression). **Rejected**: an `AggregateKind` union
  sibling — a packed union overlays (width `max`, aliased bits), it is *not*
  concatenation-equivalent, so it would break the boundary-aggregate machinery's
  bijective / `canonical_module_signature`-invariant guarantee; and the
  port-boundary union fold — it changes the module interface (generation-time,
  large blast radius), deferred as later breadth (nothing retired). Split `.3b`
  → `.3b.1` (done) + `.3b.2` (impl).
- `2026-06-16` (`.3a`, decision [`0010`](../decisions/0010-sv-version-first-upopt-soft-packed-union.md)):
  the first up-opt is a **heterogeneous-width packed `union soft` (IEEE 1800-2023
  §7.3.1)**, a new default-off aggregate projection gated on `sv_version >=
  Sv2023` (sibling of `AggregateKind::StructPacked`/`ArrayPacked`); the `< 2023`
  down-gate fallback is the existing packed `struct` projection ⇒ byte-identical
  default. Grounded in a probe of the installed Verilator 5.046 / Yosys 0.64 /
  Icarus 13.0: **the installed tools do not enforce 1800-version acceptance**
  (Verilator accepts + reserves keywords identically across all `--language`
  modes; Yosys/Icarus have no 1800 selector), so the up-opt's teeth are LRM
  correctness + construction-time down-gating + matching-mode acceptance
  (`verilator --language 1800-2023`, proven by `--binary`), not tool-side
  rejection. Real teeth confirmed: a non-soft heterogeneous-width packed union is
  rejected by all three tools (Verilator cites the standard). Yosys/Icarus reject
  the `union soft` syntax ⇒ recorded no-op for the up-opt scenario; `.3b` adds a
  dedicated `saw_sv_version_2023_soft_union_upopt` fact (Verilator-only) because
  the existing `.2b.2b` facts require Yosys-clean. Rejected: 2012-floor constructs
  with no down-gating teeth (`genvar`-in-for, unbased-unsized, signed cast,
  default args, `parameter type` defaults), a 2017-distinctive construct (none
  synthesizable found — first up-opt is 2023), Yosys+Icarus-rejected alternatives
  with no cleaner 2023 story, claiming tool-side version rejection (aspirational),
  generate-then-filter, non-byte-identical default. Split `.3` → `.3a` (done) +
  `.3b` (impl).
- `2026-06-15` (`.2a`, design detail in `DEVELOPMENT_NOTES.md`): resolved decision
  `0009`'s five open questions. (1) `SvVersion { Sv2012 < Sv2017 < Sv2023 }`
  (`PartialOrd`/`Ord`), bare-year CLI/serde value names; **default = `Sv2012`**
  (the honest floor — byte-identical and makes down-gating to the floor a provable
  no-op; supersedes decision `0009`'s "working name `Sv2017`", a free label choice
  while no version-distinctive construct exists). (2) The bound threads to the
  **emitter as a parameter, not onto the IR** (keeps CSE keys /
  `canonical_module_signature` / Module-serde untouched): new versioned entry
  points, old ones delegate with `SvVersion::default()` (byte-identical callers);
  `SvVersion::permits(introduced)` is the bound, gating nothing in `.2b.1`.
  (3) Down-gating proof = a cross-version byte-identity test over a corpus.
  (4) Introspection is serde-automatic; schema MINOR bump `1.1 → 1.2` + the five
  `"1.1"` test-assertion updates. (5) Per-version axis = optional `--language`
  selector on `run_verilator*` (`None` = today's argv), Yosys `-sv`, Icarus
  `-g2012` over the g2012-valid subset, a focused `--sv-version-gate` +
  `ScenarioSet::SvVersionSweep`. Split `.2` → `.2a`/`.2b`; pre-split `.2b` →
  `.2b.1` (knob + emitter bound) / `.2b.2` (downstream acceptance axis).
- `2026-06-15` (`.1`, decision [`0009`](../decisions/0009-sv-version-targeting.md)):
  Opened the lane `active` by owner roadmap steering. First leaf designs the
  `--sv-version <2012|2017|2023>` gate: down-gating guarantee + up-opting stress,
  byte-identical default, rules-first construction-time bound, per-version
  downstream acceptance proof. Rejected: generate-then-filter, single-newest no
  selector, unproven up-opted constructs, non-byte-identical default, classic
  Verilog targets. Tree split into `.2` (plumbing impl) + `.3` (first up-opt).

## Open Questions

- Resolved by `.2a` (see Decisions). The `.2b.2` Verilator language-selector
  spelling question was resolved in `.2b.2a`: the installed Verilator 5.046
  accepts both `--language <std>` and `--default-language <std>`; ANVIL uses
  `--language 1800-20xx` (the documented standard selector), now wired into the
  `.2b.2b` matrix gate.
- Resolved by `.3a` (decision `0010`): the first up-opt is a heterogeneous-width
  packed `union soft` (IEEE 1800-2023 §7.3.1), proven matching-mode-accepted by
  Verilator `--language 1800-2023` (`--binary`); Yosys/Icarus are recorded
  no-ops.
- `.3b` (impl): the projection shape (port-boundary union fold vs lower-risk
  internal-only `union soft` overlay), the exact `AggregateKind` variant +
  `render_aggregate_typedef` emit site + `permits(Sv2023)` gate, the new
  union-projection knob + default, the matrix up-opt scenario/fact +
  Yosys/Icarus no-op recording, and the `tests/sv_version.rs` divergence update.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-16` | `SV-VERSION-TARGETING.3b.2b` | `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --bin tool_matrix` 53/0 (incl. the updated/new sv-version proofs + the new `sv_version_sweep_has_verilator_only_soft_union_upopt_scenario`); `cargo test --lib` 427/0 (2 ignored); `cargo test --test snapshots` 6/6 byte-identical; `cargo test --test sv_version` 2/2. **Banked downstream-clean:** `./target/release/tool_matrix --sv-version-gate --yosys-mode both --out /tmp/anvil-sv-version-gate-upopt-r1` → exit 0; report: 10 scenarios / 20 units / `coverage_gaps = []` / Verilator 20/0 / Yosys without-abc 18/0 / with-abc 18/0; the `sv2023_soft_union_upopt` scenario's 2 modules carry `emitted_soft_union_overlay=true` (1 + 28 `union soft` occurrences), Verilator argv carries `--language 1800-2023`, Yosys ran 0 times; `saw_sv_version_2023_soft_union_upopt` + all four `saw_sv_version_*_targeted_acceptance` facts lit. | `done` |
| `2026-06-16` | `SV-VERSION-TARGETING.3b.2a` | `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 412/0 (2 ignored; incl. 8 new `soft_union` proofs); `cargo test --test snapshots` 6/6 byte-identical; `cargo test --test sv_version` 2/2 (byte-identity corpus untouched). **Banked downstream-clean:** `cargo test --test sv_version_downstream -- --ignored verilator_accepts_soft_union_slice_overlay_at_2023` → ok, "159 overlays across 7 seeds" all Verilator `--language 1800-2023` clean + 2012 down-gate confirmed (vs Verilator 5.046). `mdbook build book` clean; `cargo test --test book_examples` 3/3 (81s). CLI smoke: a `--config` with `soft_union_slice_prob=1.0 + sv_version=2023` over a slice-heavy shape → 47/48 seeds emit `union soft`; same seeds at `sv_version=2012` emit none (down-gate). | `done` |
| `2026-06-16` | `SV-VERSION-TARGETING.3b.1` | Design-detail leaf, no source change (grounded in a fresh read of `src/ir/aggregate.rs`, `src/emit/sv.rs` `render_gate` `Slice` arm at `sv.rs:1040` + the gate decl/assign region, `src/config.rs`). Established that the boundary-aggregate machinery's soundness rests on packed-`struct`/array = concatenation bit-equivalence (which a union violates), so the mechanism is an internal `union soft` overlay of a low-bits `Slice`, not an `AggregateKind` sibling. `DEVELOPMENT_NOTES.md` design-detail entry + tree split. Baseline `cargo check --all-targets` clean (session start). `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-16` | `SV-VERSION-TARGETING.3a` | Design leaf, no source change (grounded in a fresh deep-dive of `src/emit/sv.rs`, `src/config.rs`, `src/ir/aggregate.rs`, `src/downstream/mod.rs`, `src/bin/tool_matrix.rs` + a direct acceptance probe of the installed Verilator 5.046 / Yosys 0.64 / Icarus 13.0: 22 candidate snippets across all three `--language` modes; `verilator --binary` build of a `union soft` overlay produced `y=a5`; the non-soft heterogeneous packed union is rejected by all three tools citing IEEE 1800-2023 §7.3.1). Baseline `cargo check --all-targets` clean before the leaf. Decision `0010` + `DEVELOPMENT_NOTES.md` design-detail entry + tree split recorded; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated. | `done` |
| `2026-06-16` | `SV-VERSION-TARGETING.2b.2b` | `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 405/0; `cargo test --bin tool_matrix` 52/0 (incl. 6 new sv-version proofs); `cargo test --test snapshots` 6/6 byte-identical; `cargo test --test sv_version` 2/2; heavy `cargo test --test pipeline` re-run (touches `tool_matrix`). **Banked downstream-clean:** `cargo run --release --bin tool_matrix -- --sv-version-gate --yosys-mode both --out /tmp/anvil-sv-version-gate-r1` → exit 0; report: 9 scenarios / 18 units / `coverage_gaps = []` / Verilator 18/0 / Yosys without-abc 18/0 / with-abc 18/0; each scenario's Verilator argv carries the matching `--language 1800-20xx`; all four `saw_sv_version_*_targeted_acceptance` lit. | `done` |
| `2026-06-15` | `SV-VERSION-TARGETING.2b.2a` | `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 405/0; `cargo test --test snapshots` 6/6 (default tool argv byte-identical at `language=None`). Banked per-version acceptance: `cargo test --test sv_version_downstream -- --ignored` → 2 passed / 6.18s vs Verilator 5.046 (all 3 `--language` modes clean) + Icarus 13.0 (`-g2012`). Heavy `tests/pipeline.rs` not re-run (downstream argv byte-identical at the `None` default every committed caller uses). | `done` |
| `2026-06-15` | `SV-VERSION-TARGETING.2b.1` | `cargo check --all-targets` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test --lib` 405/0 (incl. 2 new config tests + bumped introspect/mcp schema assertions); `cargo test --test snapshots` 6/6 byte-identical; `cargo test --test sv_version` 2/2 (cross-version byte-identity over leaf + design spreads). CLI smoke: `--seed 42` default == `--sv-version 2012` == `--sv-version 2023` md5-equal; `--dump-config` → `"sv_version": "2012"`; `--introspect` → `"schema_version": "1.2"` + `"sv_version": "2012"`; `--sv-version 2005` rejected with the possible-values list. Heavy `tests/pipeline.rs` not re-run (no generation-path change; emitter byte-identical + snapshot-locked); full `cargo test` baseline green at session start. | `done` |
| `2026-06-15` | `SV-VERSION-TARGETING.2a` | Design-detail leaf, no source change (grounded by a fresh read of `src/config.rs`, `src/emit/sv.rs`, `src/introspect/mod.rs` + `docs/AGENT_INTROSPECTION_SCHEMA.md`, `src/downstream/mod.rs`, `src/bin/tool_matrix.rs`, `src/main.rs`). `DEVELOPMENT_NOTES.md` design-detail entry; task tree split recorded. Baseline `cargo check --all-targets` clean and `cargo test` green before the leaf; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-15` | `SV-VERSION-TARGETING.1` | Design/decision leaf, no source change (grounded in `src/emit/sv.rs` current subset + `src/downstream/mod.rs` fixed tool standards + confirming no existing `sv_version` knob). Decision `0009` with KM `answers:`; `KNOWLEDGE_MAP.md` regenerated; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-VERSION-TARGETING.3b.2b` | `SV-VERSION-TARGETING.3b.2b — repo-owned union soft up-opt matrix gate` | Tenth `--sv-version-gate` scenario `sv2023_soft_union_upopt` (Verilator-only) + `saw_sv_version_2023_soft_union_upopt` coverage fact + `ModuleReport.emitted_soft_union_overlay` emission evidence + `scenario_emits_soft_union_overlay`/`verilator_only` plumbing + the `!yosys.is_empty()` honesty guard. Banked clean `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units / `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0). **Closes `.3b.2`/`.3b`/`.3` and the whole tree.** Docs: README/USER_GUIDE/book(knobs)/ROADMAP/CODEBASE_ANALYSIS + KM fact. |
| `SV-VERSION-TARGETING.3b.2a` | `SV-VERSION-TARGETING.3b.2a — live union soft low-bits-slice up-opt` | `Config::soft_union_slice_prob` + `src/ir/soft_union.rs` gen-time pass + `Module.soft_union_slice_gates` + the `permits(Sv2023)`-gated `emit/sv.rs` `union soft` overlay; default-off / byte-identical (snapshots 6/6); banked Verilator `--language 1800-2023` clean (159 overlays / 7 seeds, 2012 down-gate). Split `.3b.2` → `.3b.2a`/`.3b.2b`. Docs: book/USER_GUIDE/README/ROADMAP/CODEBASE_ANALYSIS + KM fact. |
| `SV-VERSION-TARGETING.3b.1` | `SV-VERSION-TARGETING.3b.1 — soft-union up-opt mechanism (impl design-detail)` | Resolved `0010`'s mechanism open question: an emitter-level `sv_version >= Sv2023`-gated internal `union soft` overlay of a proper low-bits `Slice` (behaviour-preserving), not an `AggregateKind` sibling nor a port-boundary fold. Split `.3b` → `.3b.1`/`.3b.2`. No source change. |
| `SV-VERSION-TARGETING.3a` | `SV-VERSION-TARGETING.3a — first up-opt design (soft packed union)` | Decision `0010` — first up-opt = heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1), default-off / byte-identical (struct down-gate fallback); the installed-tool version-acceptance finding; the `sv_version >= Sv2023` gate; rejected alternatives. Split `.3` → `.3a`/`.3b`. No source change. |
| `SV-VERSION-TARGETING.2b.2b` | `SV-VERSION-TARGETING.2b.2b — repo-owned per-version acceptance gate` | `tool_matrix --sv-version-gate` + `ScenarioSet::SvVersionSweep` (9 scenarios) + per-version emit threading + matching-mode Verilator + `saw_sv_version_*_targeted_acceptance` facts + `MatrixReport.sv_version_gate` + 6 proofs. Banked clean `/tmp/anvil-sv-version-gate-r1` (18/0). Closes `.2b.2`/`.2b`/`.2`. |
| `SV-VERSION-TARGETING.2b.2a` | `SV-VERSION-TARGETING.2b.2a — per-version downstream acceptance proof` | `run_verilator(_design)` `language: Option<&str>` selector + `tests/sv_version_downstream.rs` (`#[ignore]`) banked clean (Verilator 3× `--language` + Icarus `-g2012`). Default byte-identical (`None`). |
| `SV-VERSION-TARGETING.2b.1` | `SV-VERSION-TARGETING.2b.1 — --sv-version knob + emitter capability bound` | `SvVersion` enum + `Config::sv_version` + `--sv-version` CLI + versioned emitter entry points + introspection schema `1.1→1.2` + `tests/sv_version.rs`. Default byte-identical (snapshots 6/6). |
| `SV-VERSION-TARGETING.2a` | `SV-VERSION-TARGETING.2a — SV-version impl design detail + .2 split` | Design-detail in `DEVELOPMENT_NOTES.md`; `.2` split into `.2a`/`.2b`, `.2b` pre-split into `.2b.1`/`.2b.2`. No source change. |
| `SV-VERSION-TARGETING.1` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Decision record `0009`; opened the lane + registered the two sibling `proposed` lanes. No source change. |

## Changelog

- `2026-06-16`: `.3b.2b` landed and **the tree is CLOSED**. The repo-owned
  `tool_matrix --sv-version-gate` gains a tenth `sv2023_soft_union_upopt`
  scenario (slice-heavy, `soft_union_slice_prob = 1.0`, 2023) that
  genuinely emits the IEEE 1800-2023 `union soft` overlay. Detected by
  `scenario_emits_soft_union_overlay` and threaded as `verilator_only`
  through the module-path plumbing so Yosys/Icarus are a recorded no-op;
  `ModuleReport.emitted_soft_union_overlay` (from the emitted SV text)
  gives `summarize_coverage` honest evidence to light the dedicated
  `saw_sv_version_2023_soft_union_upopt` fact (Verilator-only — never
  Yosys), enforced by `compute_coverage_gaps`. The general per-version
  fact lighting gained a `!yosys.is_empty()` honesty guard. Banked clean
  `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units /
  `coverage_gaps=[]` / Verilator 20/0 / Yosys 18/0 both modes). Default
  matrix byte-identical (snapshots 6/6). Closes `.3b.2`/`.3b`/`.3` and the
  whole `SV-VERSION-TARGETING` tree; further version-distinctive up-opts
  are open-ended post-tree breadth (nothing retired).
- `2026-06-16`: `.3b.2a` landed — **the first version-distinctive up-opt now
  ships and is downstream-proven**. `Config::soft_union_slice_prob` (default-off)
  + `src/ir/soft_union.rs` gen-time pass + `Module.soft_union_slice_gates` marker
  + the `permits(Sv2023)`-gated `emit/sv.rs` `union soft` overlay of a proper
  low-bits `Slice` (`u.w = src; gate = u.n`, behaviour-preserving). Default-off /
  byte-identical (snapshots 6/6); banked Verilator `--language 1800-2023` clean
  (159 overlays / 7 seeds) with the 2012 down-gate proven; Yosys/Icarus recorded
  no-op. Live docs synced in lockstep (book knobs + SV-version chapter,
  USER_GUIDE, README, ROADMAP, CODEBASE_ANALYSIS, KM fact). Split `.3b.2` →
  `.3b.2a` (done) + `.3b.2b` (the repo-owned matrix up-opt gate +
  `saw_sv_version_2023_soft_union_upopt` fact). Frontier advances to `.3b.2b`.
- `2026-06-16`: `.3b.1` design-detail landed (no source change): resolved the
  mechanism `0010` left open for `.3b`. The up-opt is an emitter-level
  `sv_version >= Sv2023`-gated, default-off internal `union soft` overlay
  rendering of a proper low-bits `Slice` (`u.w = src; slice = u.n`,
  behaviour-preserving because packed-union members are LSB-aligned), **not** an
  `AggregateKind` boundary projection (a union is not concatenation-equivalent, so
  it would break the bijective semantically-empty aggregate invariant) nor a
  port-boundary fold (changes the interface; deferred breadth). Split `.3b` into
  `.3b.1` (done) + `.3b.2` (impl). Frontier advances to `.3b.2`.
- `2026-06-16`: `.3a` design landed (no source change): decision `0010` names the
  first up-opt = a heterogeneous-width packed `union soft` (IEEE 1800-2023
  §7.3.1), a default-off aggregate projection gated on `sv_version >= Sv2023`
  with the existing packed-`struct` projection as the `< 2023` down-gate fallback
  ⇒ byte-identical default. Recorded the empirical tool-reality finding (the
  installed Verilator/Yosys/Icarus do not enforce 1800-version acceptance; the
  up-opt's teeth are LRM + construction-time down-gating + matching-mode
  acceptance) and the Yosys/Icarus recorded-no-op handling + the dedicated
  `.3b` up-opt coverage fact. Split `.3` into `.3a` (done) + `.3b` (impl, the
  `union soft` projection). Frontier advances to `.3b`.
- `2026-06-16`: `.2b.2b` landed (default matrix byte-identical): repo-owned
  `tool_matrix --sv-version-gate` + `ScenarioSet::SvVersionSweep` (9 Interleaved
  scenarios = 3 targets × {comb leaf, seq leaf, recursive hierarchy design}) +
  per-version emit threading (`to_sv_versioned` / `to_sv_in_design_versioned`) +
  matching-mode Verilator (`verilator_language_for`) + per-version
  `saw_sv_version_*_targeted_acceptance` coverage facts (early-return arm in
  `compute_coverage_gaps`, before the strategy loop) + `MatrixReport.sv_version_gate`
  + 6 cargo-portable proofs. Banked clean `/tmp/anvil-sv-version-gate-r1` (18/0
  Verilator + both Yosys, `coverage_gaps=[]`). Closes `.2b.2`, `.2b`, and `.2`;
  the lane frontier advances to `.3` (first up-opted construct). Tree stays
  `active` for `.3`.
- `2026-06-15`: `.2b.2a` landed (byte-identical at default): split `.2b.2` into
  `.2b.2a` (downstream `--language` selector + focused real-tool acceptance
  proof) + `.2b.2b` (repo-owned matrix gate). `run_verilator(_design)` gained
  the `language: Option<&str>` selector; `tests/sv_version_downstream.rs`
  banked clean (Verilator 3× `--language` + Icarus `-g2012`). Frontier advances
  to `.2b.2b`.
- `2026-06-15`: `.2b.1` landed (first code slice, byte-identical): `SvVersion`
  enum + `Config::sv_version` + `--sv-version` CLI + versioned emitter entry
  points (`permits` down-gating bound) threaded at all DUT emit sites,
  introspection schema `1.1→1.2`, new `tests/sv_version.rs` cross-version
  byte-identity proof. Frontier advances to `.2b.2` (per-version downstream
  acceptance axis).
- `2026-06-15`: `.2a` design-detail landed (no source change): resolved decision
  `0009`'s five open questions in `DEVELOPMENT_NOTES.md`; split `.2` into `.2a`
  (done) + `.2b` (active), and pre-split `.2b` into `.2b.1` (knob + emitter
  capability bound, byte-identical) + `.2b.2` (per-version downstream acceptance
  axis). Frontier advances to `.2b.1`.
- `2026-06-15`: Created task tree (owner-directed capability lane), opened
  `active`, landed `.1` (decision `0009`); split into `.2` (plumbing impl) +
  `.3` (first up-opted construct). Frontier advances to `.2`.
