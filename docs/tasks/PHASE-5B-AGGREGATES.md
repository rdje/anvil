# PHASE-5B-AGGREGATES: Synthesizable aggregates

## Metadata

- Tree ID: `PHASE-5B-AGGREGATES`
- Status: `active`
- Roadmap lane: Phase 5b — Synthesizable aggregates
- Created: `2026-05-16`
- Last updated: `2026-05-17` (`.1` design landed; `.2` split into `.2.1`–`.2.4`; frontier → `.2.1`)
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
  Status: `pending`
  Goal: `IR + emitter scaffold (architecture (P)). Additive Default-able Module.aggregate_layout: Option<AggregateLayout> ({kind: Struct|Union|Array packed, type_name, ordered (field_name, PortId)} ); Config::aggregate_prob (f64, serde-default 0.0, probability-range validated); post-construction opt-in pass that records a layout over a contiguous same-direction port group; emitter renders typedef <name> packed + a single aggregate port + projects grouped-port references to .fieldN at the port boundary. Internal flat wires/assigns unchanged. No matrix scenario yet.`
  Acceptance: `cargo fmt/clippy(-D warnings)/check/test green; focused proof: default-off byte-identical for fixed seeds across all ConstructionStrategy values; forced-on a projected module round-trips IR->validate->emit and the SV declares a typedef ... packed + one aggregate port; validate_design passes (IR unchanged). No book/ change (book reconciliation is .2.4).`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `PHASE-5B-AGGREGATES.2.1` | `pending` | `.1` design done; `.2` split (Splitting Rules + r87 no-aspirational-claims). `.2.1` lands the IR `AggregateLayout` annotation + `aggregate_prob` knob + emitter `typedef … packed` projection, default-off byte-identical — the reviewable scaffold before existence-proof (`.2.2`), gate scenario (`.2.3`), and verified promotion (`.2.4`). |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5B-AGGREGATES.1` | `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design` | Design-only; `DEVELOPMENT_NOTES.md` entry, architecture (P), 3 rejected alternatives. No code. |

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
