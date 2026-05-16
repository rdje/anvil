# PHASE-5B-AGGREGATES: Synthesizable aggregates

## Metadata

- Tree ID: `PHASE-5B-AGGREGATES`
- Status: `active`
- Roadmap lane: Phase 5b — Synthesizable aggregates
- Created: `2026-05-16`
- Last updated: `2026-05-16`
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
  Status: `pending`
  Goal: `Design the packed-aggregate emitter projection in DEVELOPMENT_NOTES.md: how flat IR bits map to a packed struct/union/array surface without changing semantics, knob surface, proof shape, rejected alternatives. Design-only; not blocked.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5b design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5B-AGGREGATES.2`
  Status: `pending`
  Goal: `Implement the packed-aggregate projection per .1, opt-in, with a matrix scenario and downstream-clean proof.`
  Acceptance: `Aggregate designs downstream-clean; opt-in default preserves current output; ROADMAP Phase 5b -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5B-AGGREGATES.1` | `pending` | Emitter-only; no prerequisite. Design first, then implement. |

## Decisions

- `2026-05-16`: Scoped to packed aggregates only; unpacked datapath /
  enums explicitly deferred per roadmap rationale (recorded so the
  deferral is not silently revisited).

## Open Questions

- Whether aggregate emission interacts with the module-dedup signature
  (it should not, since IR is unchanged). Owner: `.1` design.

## Blockers

- None. Independent of Phase 4.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-5B-AGGREGATES.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5B-AGGREGATES.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
