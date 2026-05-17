# PHASE-5B-AGGREGATES: Synthesizable aggregates

## Metadata

- Tree ID: `PHASE-5B-AGGREGATES`
- Status: `active`
- Roadmap lane: Phase 5b — Synthesizable aggregates
- Created: `2026-05-16`
- Last updated: `2026-05-17` (`.2.1` scaffold landed; frontier → `.2.2`)
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
  Status: `active`
  Goal: `Land packed struct/union/array emission as a flat-IR projection, downstream-clean.`
  Children: `PHASE-5B-AGGREGATES.1` (done), `PHASE-5B-AGGREGATES.2` (active container)

- ID: `PHASE-5B-AGGREGATES.1`
  Status: `done`
  Goal: `Design the packed-aggregate emitter projection in DEVELOPMENT_NOTES.md: how flat IR bits map to a packed struct/union/array surface without changing semantics, knob surface, proof shape, rejected alternatives. Design-only; not blocked.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5b design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)" entry landed: codebase-grounded (file-anchored audit of src/emit/sv.rs dumb-serialiser chokepoints, src/ir/types.rs flat Port/width reality, the Phase 5 param_env annotation precedent). Chosen architecture (P) emitter-only packed-aggregate projection driven by an additive Default-able per-module AggregateLayout annotation (construction + validate + CSE + dedup all unchanged; bijective bit-layout-preserving regrouping; opt-in aggregate_*_prob serde-default 0.0 → byte-identical). Three rejected alternatives: (A) first-class aggregate IR nodes, (B) post-hoc SV text rewrite, (C) unpacked aggregates/enums now. Identity Open Question resolved: annotation NOT hashed into canonical_module_signature (opposite of Phase 5; aggregates change nothing semantic → projected twin dedup-collapses, correct). Proof shape for .2 specified. Doc-only; no code. cargo fmt/clippy(-D warnings)/check green; cargo test unchanged-green (no src/tests touched since b5cto7m8m exit 0); mdbook build clean.`
  Commit: `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design`

- ID: `PHASE-5B-AGGREGATES.2`
  Status: `active`
  Goal: `Implement the packed-aggregate projection per .1, opt-in, with a matrix scenario and downstream-clean proof. Split per the Splitting Rules + the r87 no-aspirational-claims precedent (gate scenario lands before any ROADMAP promotion); mirrors the proven Phase 5 .2.1–.2.4 decomposition.`
  Children: `PHASE-5B-AGGREGATES.2.1`, `.2.2`, `.2.3`, `.2.4`

- ID: `PHASE-5B-AGGREGATES.2.1`
  Status: `done`
  Goal: `IR + emitter scaffold (architecture (P)). Additive Default-able Module.aggregate_layout: Option<AggregateLayout> ({kind: Struct|Union|Array packed, type_name, ordered (field_name, PortId)} ); Config::aggregate_prob (f64, serde-default 0.0, probability-range validated); post-construction opt-in pass that records a layout over a contiguous same-direction port group; emitter renders typedef <name> packed + a single aggregate port + projects grouped-port references to .fieldN at the port boundary. Internal flat wires/assigns unchanged. No matrix scenario yet.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: default-off byte-identical for fixed seeds across all ConstructionStrategy values; forced-on a projected module round-trips IR->validate->emit and the SV declares a typedef ... packed + one aggregate port; validate_design passes (IR unchanged). No book/ change (book reconciliation is .2.4).`
  Verification: `Additive Default-able Module.aggregate_layout: Option<AggregateLayout> + AggregateKind{StructPacked} + AggregateGroup{type_name,port_name,fields:Vec<(String,PortId)>} in src/ir/types.rs (zero churn to ..Module::default() sites). Config::aggregate_prob (serde-default 0.0 + probability-range validation tuple entry). New src/ir/aggregate.rs::annotate_aggregate non-rolling post-construction pass (idempotent; skips param_env modules; data-input/output groups >=2; clk/rst_n excluded via emitted_data_input_ports) with 6 unit tests. Per-module Bernoulli roll at the gen/mod.rs post-pass via the seeded ChaCha8 RNG (reproducible; never thread_rng), scoped to NON-instantiated modules so hierarchy child connections stay byte-identical. Emitter: boundary-alias projection (typedef struct packed before module; grouped ports replaced by one aggregate port/side; input fields alias to same-named wires; grouped outputs become internal logic + boundary assign) — the flat IR body emission is byte-for-byte unchanged. Focused proof packed_aggregate_is_default_off_and_projects_when_forced_on: default-off byte-identical (no aggregate artifacts) across 4 ConstructionStrategy x 6 seeds; forced-on projects every single-module design, SV declares typedef struct packed + aggregate port(s) + boundary alias/assign, validate_design clean, IR shape unchanged. Real verilator --lint-only spot-check of a projected hierarchy design (top mod_5_0001 with packed-struct out port): EXIT 0 (clean). cargo fmt/clippy -D warnings clean; aggregate:: 6/0; focused proof green; full cargo test (Verification Log). No book/ change.`
  Commit: `Phase 5b: PHASE-5B-AGGREGATES.2.1 packed-aggregate IR annotation + knob + emitter projection`

- ID: `PHASE-5B-AGGREGATES.2.2`
  Status: `pending`
  Goal: `Soundness + organic-existence proof, and identity-invariance. (a) Prove the unconstrained generator actually yields group-eligible modules (>=2 contiguous same-direction ports) at usable rates so the projection is non-inert — if a forced-on sweep shows it inert, pivot to a rules-first eligible-interface construction rule (Phase-5 rules-first-pivot discipline; no generate-then-filter). (b) Unit test: a module and its aggregate-projected twin produce the same canonical_module_signature and dedup-collapse (annotation not hashed; IR unchanged).`
  Acceptance: `cargo gates green; existence proof reproducible (or the rules-first pivot landed + recorded in Decisions); identity-invariance unit test passes; default-off still byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5B-AGGREGATES.2.3`
  Status: `pending`
  Goal: `tool_matrix scenario + metrics + gap (no ROADMAP promotion). New packed_aggregate scenario; DesignMetrics.num_packed_aggregate_modules (+ aggregate-port count); CoverageSummary.saw_packed_aggregate_design set + merged + a compute_coverage_gaps arm; bin-test scenario/design counts updated + exception-list entry.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green incl. tool_matrix phase4 bin tests; NO ROADMAP phase label change yet.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5B-AGGREGATES.2.4`
  Status: `pending`
  Goal: `Run the real repo-owned gate (now including packed_aggregate) and VERIFY downstream-clean (coverage_gaps=[], Verilator + both Yosys all-pass, saw_packed_aggregate_design=true) BEFORE any promotion. Then author an explicit ROADMAP Phase 5b "Exit criteria (met)" block tied to that artifact, promote ROADMAP Phase 5b (not started) -> (done), reconcile book/src/ir.md "Synthesizable aggregates" + book/src/knobs.md (aggregate_prob), sync README/CODEBASE_ANALYSIS/MEMORY, and close the PHASE-5B-AGGREGATES tree.`
  Acceptance: `A banked gate report shows coverage_gaps=[] + all-pass Verilator/Yosys + saw_packed_aggregate_design=true; ROADMAP Phase 5b = done with exit criteria; tree -> done. No aspirational claims (verified artifact precedes promotion).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5B-AGGREGATES.2.2` | `pending` | `.2.1` scaffold landed (IR annotation + `aggregate_prob` knob + boundary-alias emitter projection; default-off byte-identical; projected design verilator-clean). `.2.2` proves organic existence (group-eligible modules at usable rates, else rules-first pivot) and identity-invariance (projected twin shares `canonical_module_signature` + dedup-collapses). |

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

## Open Questions

- Resolved by `.1`: aggregate emission does **not** interact with the
  module-dedup signature. `canonical_module_signature` is computed
  from the flat IR, which the projection never mutates; the
  `AggregateLayout` annotation is deliberately **not** hashed into the
  signature (the opposite of Phase 5's `param_env`, because aggregates
  change nothing semantic). A module and its aggregate-projected twin
  therefore share one signature and correctly dedup-collapse.
  `dedup_modules` unchanged.

## Blockers

- None. Independent of Phase 4.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `PHASE-5B-AGGREGATES.1` | `DEVELOPMENT_NOTES.md` Phase 5b design entry landed (codebase-grounded; architecture (P) chosen; 3 rejected alternatives; identity-invariance resolved; proof shape). Doc-only, no code; `cargo fmt`/`clippy -D warnings`/`check` green; `cargo test` unchanged-green (no `src/`/`tests/` touched since `b5cto7m8m` exit 0); `mdbook build book` clean. | Done. |
| `2026-05-17` | `PHASE-5B-AGGREGATES.2.1` | IR `aggregate_layout` annotation + `AggregateKind`/`AggregateGroup`; `Config::aggregate_prob` (serde-default 0.0 + validation); `src/ir/aggregate.rs::annotate_aggregate` non-rolling pass (6 unit tests); seeded per-module Bernoulli roll at `gen/mod.rs` post-pass scoped to non-instantiated modules; boundary-alias emitter projection (typedef struct packed + aggregate port + alias wires/assigns, flat body byte-identical). Focused proof `packed_aggregate_is_default_off_and_projects_when_forced_on` (default-off byte-identical 4 strategies × 6 seeds; forced-on projects + validates + SV tokens). Real `verilator --lint-only` of a projected hierarchy design: EXIT 0. `cargo fmt`/`clippy -D warnings`/`check` clean; `aggregate::` 6/0 + focused proof green; full `cargo test` (COMMIT.md gate). No `book/` change (reconciliation is `.2.4`). | Done. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5B-AGGREGATES.1` | `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design` | Design-only; `DEVELOPMENT_NOTES.md` entry, architecture (P), 3 rejected alternatives. No code. |
| `PHASE-5B-AGGREGATES.2.1` | `Phase 5b: PHASE-5B-AGGREGATES.2.1 packed-aggregate IR annotation + knob + emitter projection` | Scaffold: `aggregate_layout` annotation + `aggregate_prob` knob + boundary-alias emitter projection; default-off byte-identical; projected design verilator-clean. StructPacked / non-instantiated / non-param scoped (Decisions). |

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
