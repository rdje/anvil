# PHASE-5B-AGGREGATES: Synthesizable aggregates

## Metadata

- Tree ID: `PHASE-5B-AGGREGATES`
- Status: `active`
- Roadmap lane: Phase 5b — Synthesizable aggregates
- Created: `2026-05-16`
- Last updated: `2026-05-18` (`.2.4` real-gate verified clean → ROADMAP Phase 5b `done`; tree closed; frontier none — Phase 5b complete)
- Owner: repo-local workflow

## Goal

Emit synthesizable aggregate types — primarily **packed
struct/union/array** — to widen downstream parser/elaboration coverage,
while keeping the circuit IR flat (emitter-layer change only). Order is
not fixed relative to Phase 5; this can land independently of Phase 4.

## Non-Goals

- Unpacked arrays (memory-inference pattern) — that is Phase 6.
- Unpacked struct/union for datapath and enums — deprioritised per
  roadmap (mostly non-synthesizable / no distinct stress value).
- Any IR restructuring; aggregates are an emitter projection over the
  existing flat IR.

## Acceptance Criteria

- Packed struct/union/array emission as an opt-in projection over the
  current flat IR, valid by construction.
- Downstream-clean (Verilator + both Yosys modes) with an aggregate
  matrix scenario.
- `book/src/ir.md` "Future extensions / Synthesizable aggregates"
  reconciled with what actually landed; knobs/docs updated.

## Task Tree

- ID: `PHASE-5B-AGGREGATES`
  Status: `done`
  Goal: `Land packed struct/union/array emission as a flat-IR projection, downstream-clean.`
  Children: `PHASE-5B-AGGREGATES.1` (done), `PHASE-5B-AGGREGATES.2` (done container)

- ID: `PHASE-5B-AGGREGATES.1`
  Status: `done`
  Goal: `Design the packed-aggregate emitter projection in DEVELOPMENT_NOTES.md: how flat IR bits map to a packed struct/union/array surface without changing semantics, knob surface, proof shape, rejected alternatives. Design-only; not blocked.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5b design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)" entry landed: codebase-grounded (file-anchored audit of src/emit/sv.rs dumb-serialiser chokepoints, src/ir/types.rs flat Port/width reality, the Phase 5 param_env annotation precedent). Chosen architecture (P) emitter-only packed-aggregate projection driven by an additive Default-able per-module AggregateLayout annotation (construction + validate + CSE + dedup all unchanged; bijective bit-layout-preserving regrouping; opt-in aggregate_*_prob serde-default 0.0 → byte-identical). Three rejected alternatives: (A) first-class aggregate IR nodes, (B) post-hoc SV text rewrite, (C) unpacked aggregates/enums now. Identity Open Question resolved: annotation NOT hashed into canonical_module_signature (opposite of Phase 5; aggregates change nothing semantic → projected twin dedup-collapses, correct). Proof shape for .2 specified. Doc-only; no code. cargo fmt/clippy(-D warnings)/check green; cargo test unchanged-green (no src/tests touched since b5cto7m8m exit 0); mdbook build clean.`
  Commit: `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design`

- ID: `PHASE-5B-AGGREGATES.2`
  Status: `done`
  Goal: `Implement the packed-aggregate projection per .1, opt-in, with a matrix scenario and downstream-clean proof. Split per the Splitting Rules + the r87 no-aspirational-claims precedent (gate scenario lands before any ROADMAP promotion); mirrors the proven Phase 5 .2.1–.2.4 decomposition.`
  Children: `PHASE-5B-AGGREGATES.2.1` (done), `.2.2` (done), `.2.3` (done), `.2.4` (done)

- ID: `PHASE-5B-AGGREGATES.2.1`
  Status: `done`
  Goal: `IR + emitter scaffold (architecture (P)). Additive Default-able Module.aggregate_layout: Option<AggregateLayout> ({kind: Struct|Union|Array packed, type_name, ordered (field_name, PortId)} ); Config::aggregate_prob (f64, serde-default 0.0, probability-range validated); post-construction opt-in pass that records a layout over a contiguous same-direction port group; emitter renders typedef <name> packed + a single aggregate port + projects grouped-port references to .fieldN at the port boundary. Internal flat wires/assigns unchanged. No matrix scenario yet.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: default-off byte-identical for fixed seeds across all ConstructionStrategy values; forced-on a projected module round-trips IR->validate->emit and the SV declares a typedef ... packed + one aggregate port; validate_design passes (IR unchanged). No book/ change (book reconciliation is .2.4).`
  Verification: `Additive Default-able Module.aggregate_layout: Option<AggregateLayout> + AggregateKind{StructPacked} + AggregateGroup{type_name,port_name,fields:Vec<(String,PortId)>} in src/ir/types.rs (zero churn to ..Module::default() sites). Config::aggregate_prob (serde-default 0.0 + probability-range validation tuple entry). New src/ir/aggregate.rs::annotate_aggregate non-rolling post-construction pass (idempotent; skips param_env modules; data-input/output groups >=2; clk/rst_n excluded via emitted_data_input_ports) with 6 unit tests. Per-module Bernoulli roll at the gen/mod.rs post-pass via the seeded ChaCha8 RNG (reproducible; never thread_rng), scoped to NON-instantiated modules so hierarchy child connections stay byte-identical. Emitter: boundary-alias projection (typedef struct packed before module; grouped ports replaced by one aggregate port/side; input fields alias to same-named wires; grouped outputs become internal logic + boundary assign) — the flat IR body emission is byte-for-byte unchanged. Focused proof packed_aggregate_is_default_off_and_projects_when_forced_on: default-off byte-identical (no aggregate artifacts) across 4 ConstructionStrategy x 6 seeds; forced-on projects every single-module design, SV declares typedef struct packed + aggregate port(s) + boundary alias/assign, validate_design clean, IR shape unchanged. Real verilator --lint-only spot-check of a projected hierarchy design (top mod_5_0001 with packed-struct out port): EXIT 0 (clean). cargo fmt/clippy -D warnings clean; aggregate:: 6/0; focused proof green; full cargo test (Verification Log). No book/ change.`
  Commit: `Phase 5b: PHASE-5B-AGGREGATES.2.1 packed-aggregate IR annotation + knob + emitter projection`

- ID: `PHASE-5B-AGGREGATES.2.2`
  Status: `done`
  Goal: `Soundness + organic-existence proof, and identity-invariance. (a) Prove the unconstrained generator actually yields group-eligible modules (>=2 contiguous same-direction ports) at usable rates so the projection is non-inert — if a forced-on sweep shows it inert, pivot to a rules-first eligible-interface construction rule (Phase-5 rules-first-pivot discipline; no generate-then-filter). (b) Unit test: a module and its aggregate-projected twin produce the same canonical_module_signature and dedup-collapse (annotation not hashed; IR unchanged).`
  Acceptance: `cargo gates green; existence proof reproducible (or the rules-first pivot landed + recorded in Decisions); identity-invariance unit test passes; default-off still byte-identical.`
  Verification: `(a) tests/pipeline.rs::packed_aggregate_organic_existence_is_not_inert — with DEFAULT port ranges (NO forcing) + aggregate_prob=1.0 across 4 ConstructionStrategy x 20 seeds, the projection fires on 68/80 (~85%) organic single-module designs (observed; threshold pinned at >=50% — robust, not marginal). Conclusion: unlike Phase 5 width-homogeneity, packed-aggregate eligibility (>=2 same-direction data ports) is the common organic shape, so the .2.1 post-construction pass is NOT inert and NO rules-first constructor is needed (recorded in Decisions). (b) src/ir/aggregate.rs: canonical_signature_is_invariant_under_projection (sig identical before/after annotate_aggregate; flat ports unchanged) + aggregate_projected_twin_dedup_collapses (a concrete module and its projected twin share canonical_module_signature and dedup_modules collapses them under a top, removed==1). aggregate:: 10/0 (8 prior + 2 new). Default-off byte-identical unaffected (annotate only sets the annotation; .2.1 proof still green). cargo fmt/clippy -D warnings/check clean; full cargo test (COMMIT.md gate). No book/ change.`
  Commit: `Phase 5b: PHASE-5B-AGGREGATES.2.2 organic-existence proof + identity-invariance`

- ID: `PHASE-5B-AGGREGATES.2.3`
  Status: `done`
  Goal: `tool_matrix scenario + metrics + gap (no ROADMAP promotion). New packed_aggregate scenario; DesignMetrics.num_packed_aggregate_modules (+ aggregate-port count); CoverageSummary.saw_packed_aggregate_design set + merged + a compute_coverage_gaps arm; bin-test scenario/design counts updated + exception-list entry.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green incl. tool_matrix phase4 bin tests; NO ROADMAP phase label change yet.`
  Verification: `src/metrics.rs: DesignMetrics.num_packed_aggregate_modules (count of modules with aggregate_layout.is_some()), populated in compute_design. src/bin/tool_matrix.rs: new phase5b_packed_aggregate_focus_config (depth-1 wrapper, library, aggregate_prob=1.0, shaped EXACTLY like the phase5/dedup anchor — 4 leaves / 4 instances, all routing 0.0 — so leaf/child/range/source shape-coverage sets are unperturbed) + phase5b_packed_aggregate scenario tuple + CoverageSummary.saw_packed_aggregate_design (set when config.aggregate_prob>0 && num_packed_aggregate_modules>0) + merge_coverage + Phase4Hierarchy compute_coverage_gaps arm. Bin tests: scenario_count 213→216, total_modules 852→864 (observed deterministically, not guessed), exception-list entry added; tool_matrix phase4 bin tests 3/3. New phase5b_packed_aggregate_scenario_is_non_vacuous bin test proves every phase5b_packed_aggregate scenario projects ≥1 module (the top wrapper) so the coverage fact is reachable (3/3 strategies). cargo fmt/clippy -D warnings/check clean; full cargo test (COMMIT.md gate). ROADMAP unchanged (promotion is .2.4). No book/ change.`
  Commit: `Phase 5b: PHASE-5B-AGGREGATES.2.3 packed_aggregate matrix scenario + metrics + gap`

- ID: `PHASE-5B-AGGREGATES.2.4`
  Status: `done`
  Goal: `Run the real repo-owned gate (now including packed_aggregate) and VERIFY downstream-clean (coverage_gaps=[], Verilator + both Yosys all-pass, saw_packed_aggregate_design=true) BEFORE any promotion. Then author an explicit ROADMAP Phase 5b "Exit criteria (met)" block tied to that artifact, promote ROADMAP Phase 5b (not started) -> (done), reconcile book/src/ir.md "Synthesizable aggregates" + book/src/knobs.md (aggregate_prob), sync README/CODEBASE_ANALYSIS/MEMORY, and close the PHASE-5B-AGGREGATES tree.`
  Acceptance: `A banked gate report shows coverage_gaps=[] + all-pass Verilator/Yosys + saw_packed_aggregate_design=true; ROADMAP Phase 5b = done with exit criteria; tree -> done. No aspirational claims (verified artifact precedes promotion).`
  Verification: `Real repo-owned Phase4Hierarchy gate run to completion (background bifczmcw7, exit 0): ./target/release/tool_matrix --phase4-hierarchy-gate --yosys-mode both --base-seed 0 --out /tmp/anvil-tool-matrix-phase5b-p1. Banked /tmp/anvil-tool-matrix-phase5b-p1/tool_matrix_report.json verified CLEAN: scenario_count 216, total_modules 864, coverage_gaps [], tool_summary verilator 864/0, yosys_without_abc 864/0, yosys_with_abc 864/0, coverage.saw_packed_aggregate_design true, coverage.saw_width_parameterized_design true (Phase 5 regression clean), coverage.saw_recursive_hierarchy_module_dedup_active true (Phase 4 regression clean). Promotion strictly followed the verified artifact (r87 no-aspirational-claims): ROADMAP.md Phase 5b (not started)->(done) + "Status: done as of 2026-05-18" + explicit 3-point "Exit criteria (met)" block tied to that report + scope note (StructPacked-only / non-instantiated / param_env-skipped are open-ended post-phase, not blockers; no mode retired). Reconciled book/src/ir.md "Synthesizable aggregates" (Delivered note) + book/src/knobs.md (aggregate_prob + width_parameterization_prob in the config-only knob list and the effectiveness map); mdbook build clean. Synced README.md, CODEBASE_ANALYSIS.md (new 5b phase-coverage row → done), MEMORY.md (Phase line; Phase 6 next). Tree closed: .2.4/.2/root → done, frontier none. No code change in this leaf (gate run + docs/closure only); cargo gates green from .2.3.`
  Commit: `Phase 5b: PHASE-5B-AGGREGATES.2.4 real-gate verify + ROADMAP Phase 5b (not started)->(done) + tree closure`

## Current Frontier

None — `PHASE-5B-AGGREGATES` is `done` (Phase 5b closed
`2026-05-18`, verified artifact `/tmp/anvil-tool-matrix-phase5b-p1`).
The next numbered roadmap phase is **Phase 6 — Advanced motifs**
(`docs/tasks/PHASE-6-ADVANCED-MOTIFS.md`).

## Decisions

- `2026-05-16`: Scoped to packed aggregates only; unpacked datapath /
  enums explicitly deferred per roadmap rationale (recorded so the
  deferral is not silently revisited).
- `2026-05-17` (`.1` outcome): chose architecture **(P) emitter-only
  packed-aggregate projection** over (A) first-class aggregate IR
  nodes, (B) post-hoc SV text rewrite, (C) unpacked aggregates/enums
  now. (P) mirrors the Phase 5 annotation-consulted-only-by-emitter
  shape: an additive `Default`-able per-module `AggregateLayout`
  annotation, a bijective bit-layout-preserving regrouping, opt-in
  `aggregate_*_prob` (serde-default 0.0 → byte-identical),
  construction/validate/CSE/dedup all unchanged. Rationale + the
  full rejected-alternatives trail in `DEVELOPMENT_NOTES.md` "Phase 5b
  packed-aggregate emitter projection design".
- `2026-05-17`: **`.2` split** per the Splitting Rules (not signoff-able
  in one slice; mixes IR/knob/emitter/tests/matrix-gate/docs+promotion
  that review independently) and the r87 no-aspirational-claims
  precedent (the gate scenario must land before any ROADMAP
  promotion). Children mirror the proven Phase 5 `.2.1`–`.2.4`
  decomposition: `.2.1` IR+knob+emitter scaffold (default-off
  byte-identical), `.2.2` soundness/organic-existence + identity
  invariance, `.2.3` matrix scenario+metrics+gap (no promotion),
  `.2.4` real-gate verify → ROADMAP Phase 5b `done` + tree closure.
  No node renumbered; `.2` is now a container. Frontier → `.2.1`.
- `2026-05-17` (`.2.1` scaffold scoping — recorded so it is not
  silently revisited): (i) **`AggregateKind::StructPacked` only.** A
  packed `struct` is the general always-sound surface for a
  differing-width group; `UnionPacked`/`ArrayPacked` need a same-width
  group and are a later `.2.x` calibration sub-slice (the enum is
  defined now so the IR shape is stable). (ii) **Non-instantiated
  modules only.** A projected child would change its emitted port
  surface while the parent-side instance connection still uses the
  flat port names; rewriting parent-side aggregate connections is
  deferred. Single-module designs and the never-instantiated top are
  projected; hierarchy children stay flat. Soundness-scoped in the
  same spirit as Phase 5's planned-child loop. (iii) **Parameterized
  modules skipped.** The param/aggregate cross-product is out of
  scope for the scaffold; `annotate_aggregate` skips `param_env`
  modules. `aggregate_prob == 0` keeps both features off and
  byte-identical regardless.
- `2026-05-17` (`.2.2` outcome — **no rules-first pivot**): a DEFAULT
  port-range sweep (4 `ConstructionStrategy` × 20 seeds, no forcing,
  `aggregate_prob = 1.0`) projects **68/80 ≈ 85 %** of organic
  single-module designs. Unlike Phase 5 width-homogeneity (which the
  unconstrained generator essentially never produced — forcing the
  rules-first `build_parameterizable_leaf` pivot), packed-aggregate
  eligibility (≥2 same-direction data ports) is the *common* organic
  shape. Conclusion (recorded so it is not silently revisited): the
  `.2.1` post-construction pass is **not** inert and is **not** the
  generate-then-filter anti-pattern; **no rules-first eligible-
  interface constructor is added**. Existence threshold pinned at
  ≥ 50 % (robust headroom over the observed 85 %).

## Open Questions

- Resolved by `.1` (design) and **proven in code by `.2.2`**:
  aggregate emission does **not** interact with the module-dedup
  signature. `canonical_module_signature` is computed from the flat
  IR, which the projection never mutates; the `AggregateLayout`
  annotation is deliberately **not** hashed into the signature (the
  opposite of Phase 5's `param_env`, because aggregates change nothing
  semantic). `canonical_signature_is_invariant_under_projection` +
  `aggregate_projected_twin_dedup_collapses` (`src/ir/aggregate.rs`)
  assert exactly this: a module and its aggregate-projected twin share
  one signature and `dedup_modules` collapses them. `dedup_modules`
  unchanged. (No open questions remain.)

## Blockers

- None. Independent of Phase 4.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `PHASE-5B-AGGREGATES.1` | `DEVELOPMENT_NOTES.md` Phase 5b design entry landed (codebase-grounded; architecture (P) chosen; 3 rejected alternatives; identity-invariance resolved; proof shape). Doc-only, no code; `cargo fmt`/`clippy -D warnings`/`check` green; `cargo test` unchanged-green (no `src/`/`tests/` touched since `b5cto7m8m` exit 0); `mdbook build book` clean. | Done. |
| `2026-05-17` | `PHASE-5B-AGGREGATES.2.1` | IR `aggregate_layout` annotation + `AggregateKind`/`AggregateGroup`; `Config::aggregate_prob` (serde-default 0.0 + validation); `src/ir/aggregate.rs::annotate_aggregate` non-rolling pass (6 unit tests); seeded per-module Bernoulli roll at `gen/mod.rs` post-pass scoped to non-instantiated modules; boundary-alias emitter projection (typedef struct packed + aggregate port + alias wires/assigns, flat body byte-identical). Focused proof `packed_aggregate_is_default_off_and_projects_when_forced_on` (default-off byte-identical 4 strategies × 6 seeds; forced-on projects + validates + SV tokens). Real `verilator --lint-only` of a projected hierarchy design: EXIT 0. `cargo fmt`/`clippy -D warnings`/`check` clean; `aggregate::` 6/0 + focused proof green; full `cargo test` (COMMIT.md gate). No `book/` change (reconciliation is `.2.4`). | Done. |
| `2026-05-17` | `PHASE-5B-AGGREGATES.2.2` | (a) `packed_aggregate_organic_existence_is_not_inert` — 68/80 (~85%) organic single-module designs projected with DEFAULT port ranges (4 strategies × 20 seeds), threshold ≥50%; **no rules-first pivot** (recorded in Decisions). (b) `canonical_signature_is_invariant_under_projection` + `aggregate_projected_twin_dedup_collapses` (`src/ir/aggregate.rs`): signature identical before/after `annotate_aggregate`, flat ports unchanged; projected twin shares `canonical_module_signature` and `dedup_modules` collapses it (removed==1, survivor+top). `aggregate::` 10/0; `cargo fmt`/`clippy -D warnings`/`check` clean; full `cargo test` (COMMIT.md gate). No `book/` change. | Done. |
| `2026-05-17` | `PHASE-5B-AGGREGATES.2.3` | `DesignMetrics.num_packed_aggregate_modules` + populate; `phase5b_packed_aggregate_focus_config` (depth-1 wrapper, library, `aggregate_prob=1.0`, dedup-anchor 4 leaves/4 instances → shape-coverage sets unperturbed) + `phase5b_packed_aggregate` scenario tuple; `CoverageSummary.saw_packed_aggregate_design` set/merge + Phase4Hierarchy `compute_coverage_gaps` arm; bin counts 213→216 / 852→864 (observed) + exception-list entry; tool_matrix phase4 bin tests 3/3; new `phase5b_packed_aggregate_scenario_is_non_vacuous` proves the scenario projects ≥1 module per strategy (coverage fact reachable). `cargo fmt`/`clippy -D warnings`/`check` clean; full `cargo test` (COMMIT.md gate). ROADMAP unchanged (promotion is `.2.4`). No `book/` change. | Done. |
| `2026-05-18` | `PHASE-5B-AGGREGATES.2.4` | Real repo-owned `Phase4Hierarchy` gate (incl. `phase5b_packed_aggregate`) ran to completion (bg `bifczmcw7`, exit 0). Banked `/tmp/anvil-tool-matrix-phase5b-p1/tool_matrix_report.json` verified CLEAN: scenario_count 216, total_modules 864, `coverage_gaps=[]`, Verilator 864/0, `yosys_without_abc` 864/0, `yosys_with_abc` 864/0, `saw_packed_aggregate_design=true`, `saw_width_parameterized_design=true` (P5 regression), `saw_recursive_hierarchy_module_dedup_active=true` (P4 regression). ROADMAP Phase 5b `(not started)`→`(done)` + 3-point "Exit criteria (met)" + scope note; `book/src/ir.md`+`knobs.md` reconciled (`mdbook build` clean); README/CODEBASE_ANALYSIS/MEMORY synced; tree closed (`.2.4`/`.2`/root → done, frontier none). Gate-run + docs/closure only — no code change; cargo gates green from `.2.3`. | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5B-AGGREGATES.1` | `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design` | Design-only; `DEVELOPMENT_NOTES.md` entry, architecture (P), 3 rejected alternatives. No code. |
| `PHASE-5B-AGGREGATES.2.1` | `Phase 5b: PHASE-5B-AGGREGATES.2.1 packed-aggregate IR annotation + knob + emitter projection` | Scaffold: `aggregate_layout` annotation + `aggregate_prob` knob + boundary-alias emitter projection; default-off byte-identical; projected design verilator-clean. StructPacked / non-instantiated / non-param scoped (Decisions). |
| `PHASE-5B-AGGREGATES.2.2` | `Phase 5b: PHASE-5B-AGGREGATES.2.2 organic-existence proof + identity-invariance` | Existence 68/80 (~85%) → no rules-first pivot; signature-invariant + projected-twin dedup-collapses. `aggregate::` 10/0. No code change to the feature (proofs only). |
| `PHASE-5B-AGGREGATES.2.3` | `Phase 5b: PHASE-5B-AGGREGATES.2.3 packed_aggregate matrix scenario + metrics + gap` | `num_packed_aggregate_modules` metric + `phase5b_packed_aggregate` scenario + `saw_packed_aggregate_design` fact/gap; bin 213→216 / 852→864; non-vacuity test. No ROADMAP promotion (that is `.2.4` on verified evidence). |
| `PHASE-5B-AGGREGATES.2.4` | `Phase 5b: PHASE-5B-AGGREGATES.2.4 real-gate verify + ROADMAP Phase 5b (not started)->(done) + tree closure` | Closes `.2`, the container, and the `PHASE-5B-AGGREGATES` tree. Promotion strictly follows the verified `/tmp/anvil-tool-matrix-phase5b-p1` artifact (216/864, `coverage_gaps=[]`, 864/0 Verilator + both Yosys, `saw_packed_aggregate_design=true`). Gate-run + docs/closure only — no code change. |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
- `2026-05-17`: `.1` design landed (design-only, no code) —
  `DEVELOPMENT_NOTES.md` "Phase 5b packed-aggregate emitter projection
  design". Architecture **(P)** emitter-only projection over the flat
  IR via an additive `Default`-able `AggregateLayout` annotation
  (Phase-5-style annotation-consulted-only-by-emitter), opt-in
  `aggregate_*_prob` serde-default 0.0 (byte-identical),
  construction/validate/CSE/dedup unchanged; rejected (A) first-class
  aggregate IR nodes, (B) post-hoc SV text rewrite, (C) unpacked
  aggregates/enums now. Identity Open Question resolved: annotation
  not hashed into `canonical_module_signature` (aggregates change
  nothing semantic; projected twin dedup-collapses, correct).
  `mdbook` clean. Frontier → `.2` (implementation).
- `2026-05-17`: `.2` split per the Splitting Rules + r87
  no-aspirational-claims into `.2.1` (IR+knob+emitter scaffold,
  default-off byte-identical), `.2.2` (soundness/organic-existence +
  identity-invariance), `.2.3` (matrix scenario+metrics+gap, no
  promotion), `.2.4` (real-gate verify → ROADMAP Phase 5b `done` +
  tree closure). `.2` became a container; no renumbering. Frontier →
  `.2.1`.
- `2026-05-17`: **`.2.1` scaffold landed.** IR: additive
  `Default`-able `Module.aggregate_layout: Option<AggregateLayout>` +
  `AggregateKind{StructPacked}` + `AggregateGroup` (`src/ir/types.rs`).
  Knob: `Config::aggregate_prob` serde-default 0.0 + validation. New
  `src/ir/aggregate.rs::annotate_aggregate` non-rolling pass (6 unit
  tests); per-module seeded Bernoulli roll at the `gen/mod.rs`
  post-pass, scoped to non-instantiated modules. Emitter:
  boundary-alias projection (`typedef struct packed` + one aggregate
  port/side + input alias wires + grouped-output internal logic +
  boundary assigns) leaving the flat IR body byte-identical. Focused
  proof `packed_aggregate_is_default_off_and_projects_when_forced_on`
  (default-off byte-identical 4 strategies × 6 seeds; forced-on
  projects + `validate_design` clean + SV tokens). Real
  `verilator --lint-only` of a projected hierarchy design: EXIT 0
  (clean). Scoping decisions (StructPacked-only / non-instantiated /
  non-param) recorded in Decisions. No `book/` change. Frontier →
  `.2.2` (organic-existence + identity-invariance).
- `2026-05-17`: **`.2.2` landed (proofs only — no feature code
  change).** (a) `packed_aggregate_organic_existence_is_not_inert`:
  with DEFAULT port ranges (no forcing) + `aggregate_prob = 1.0`,
  68/80 ≈ 85 % of organic single-module designs are projected across
  4 `ConstructionStrategy` × 20 seeds → the projection is **not
  inert**; **no rules-first pivot** (recorded in Decisions; threshold
  pinned ≥ 50 %). (b) `canonical_signature_is_invariant_under_projection`
  + `aggregate_projected_twin_dedup_collapses` (`src/ir/aggregate.rs`):
  `annotate_aggregate` leaves `canonical_module_signature` and the
  flat ports unchanged, and a projected twin dedup-collapses into its
  concrete equal (annotation deliberately not hashed — the Open
  Question, now proven in code). `aggregate::` 10/0. Frontier →
  `.2.3` (matrix scenario + metrics + gap, no promotion).
- `2026-05-17`: **`.2.3` landed (matrix scenario + metrics + gap; no
  ROADMAP promotion).** `src/metrics.rs`
  `DesignMetrics.num_packed_aggregate_modules` + populate.
  `src/bin/tool_matrix.rs`: `phase5b_packed_aggregate_focus_config`
  (depth-1 wrapper, library, `aggregate_prob = 1.0`, shaped exactly
  like the phase5/dedup anchor so leaf/child/range/source shape sets
  are unperturbed) + `phase5b_packed_aggregate` scenario tuple;
  `CoverageSummary.saw_packed_aggregate_design` set/merge +
  `Phase4Hierarchy` `compute_coverage_gaps` arm; bin counts
  213 → 216 / 852 → 864 (observed deterministically) + exception-list
  entry; tool_matrix phase4 bin tests 3/3. New
  `phase5b_packed_aggregate_scenario_is_non_vacuous` proves every such
  scenario projects ≥ 1 module (the top wrapper) so
  `saw_packed_aggregate_design` is reachable — `.2.4`'s gate cannot
  carry a permanent coverage gap. ROADMAP unchanged. Frontier →
  `.2.4` (real-gate verify → promote Phase 5b `done` + book
  reconciliation + tree closure).
- `2026-05-18`: **`.2.4` landed — closes `.2`, the container, and the
  `PHASE-5B-AGGREGATES` tree; Phase 5b is `done`.** The real
  repo-owned `Phase4Hierarchy` gate (now including
  `phase5b_packed_aggregate`) ran to completion (bg `bifczmcw7`,
  exit 0) and was verified CLEAN on the banked artifact
  `/tmp/anvil-tool-matrix-phase5b-p1/tool_matrix_report.json`: 216
  scenarios / 864 designs, `coverage_gaps=[]`, Verilator 864/0,
  `yosys_without_abc` 864/0, `yosys_with_abc` 864/0,
  `saw_packed_aggregate_design=true`, `saw_width_parameterized_design=true`
  (Phase 5 regression clean), `saw_recursive_hierarchy_module_dedup_active=true`
  (Phase 4 regression clean). Promotion strictly followed the verified
  artifact (r87 no-aspirational-claims): `ROADMAP.md` Phase 5b
  `(not started)`→`(done)` + `Status: done as of 2026-05-18` + an
  explicit 3-point **"Exit criteria (met)"** block + scope note
  (StructPacked-only / non-instantiated / `param_env`-skipped are
  open-ended post-phase sub-slices, not blockers; no mode retired).
  Reconciled `book/src/ir.md` "Synthesizable aggregates" + `book/src/knobs.md`
  (`aggregate_prob` + `width_parameterization_prob` in the config-only
  list and effectiveness map); `mdbook build book` clean. Synced
  `README.md`, `CODEBASE_ANALYSIS.md` (new `5b` phase-coverage row →
  done), `MEMORY.md` (Phase line; Phase 6 next). Gate-run + docs /
  closure only — no code change. The next numbered roadmap phase is
  **Phase 6 — Advanced motifs**. Frontier → none (Phase 5b closed).
