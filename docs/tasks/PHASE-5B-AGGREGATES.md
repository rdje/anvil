# PHASE-5B-AGGREGATES: Synthesizable aggregates

## Metadata

- Tree ID: `PHASE-5B-AGGREGATES`
- Status: `active`
- Roadmap lane: Phase 5b — Synthesizable aggregates
- Created: `2026-05-16`
- Last updated: `2026-05-17` (`.1` design landed; frontier → `.2`)
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
  Children: `PHASE-5B-AGGREGATES.1`, `PHASE-5B-AGGREGATES.2`

- ID: `PHASE-5B-AGGREGATES.1`
  Status: `done`
  Goal: `Design the packed-aggregate emitter projection in DEVELOPMENT_NOTES.md: how flat IR bits map to a packed struct/union/array surface without changing semantics, knob surface, proof shape, rejected alternatives. Design-only; not blocked.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5b design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `DEVELOPMENT_NOTES.md "Phase 5b packed-aggregate emitter projection design (2026-05-17, PHASE-5B-AGGREGATES.1)" entry landed: codebase-grounded (file-anchored audit of src/emit/sv.rs dumb-serialiser chokepoints, src/ir/types.rs flat Port/width reality, the Phase 5 param_env annotation precedent). Chosen architecture (P) emitter-only packed-aggregate projection driven by an additive Default-able per-module AggregateLayout annotation (construction + validate + CSE + dedup all unchanged; bijective bit-layout-preserving regrouping; opt-in aggregate_*_prob serde-default 0.0 → byte-identical). Three rejected alternatives: (A) first-class aggregate IR nodes, (B) post-hoc SV text rewrite, (C) unpacked aggregates/enums now. Identity Open Question resolved: annotation NOT hashed into canonical_module_signature (opposite of Phase 5; aggregates change nothing semantic → projected twin dedup-collapses, correct). Proof shape for .2 specified. Doc-only; no code. cargo fmt/clippy(-D warnings)/check green; cargo test unchanged-green (no src/tests touched since b5cto7m8m exit 0); mdbook build clean.`
  Commit: `Docs: PHASE-5B-AGGREGATES.1 packed-aggregate emitter-projection design`

- ID: `PHASE-5B-AGGREGATES.2`
  Status: `pending`
  Goal: `Implement the packed-aggregate projection per .1, opt-in, with a matrix scenario and downstream-clean proof.`
  Acceptance: `Aggregate designs downstream-clean; opt-in default preserves current output; ROADMAP Phase 5b -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5B-AGGREGATES.2` | `pending` | `.1` design landed (architecture (P) emitter-only projection, 3 rejected alternatives, identity-invariance resolved). `.2` implements it: additive `AggregateLayout` annotation + opt-in `aggregate_*_prob` + emitter `typedef … packed` projection + matrix scenario + downstream-clean proof, then ROADMAP Phase 5b → done. |

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
