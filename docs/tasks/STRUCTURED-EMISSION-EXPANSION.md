# STRUCTURED-EMISSION-EXPANSION: richer structured SystemVerilog surfaces

## Metadata

- Tree ID: `STRUCTURED-EMISSION-EXPANSION`
- Status: `proposed`
- Roadmap lane: `Capability / breadth — richer structured emission (ROADMAP steering gap 1)`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow
- Note: registered `proposed` by owner roadmap steering (`2026-06-15`) as a named
  sibling of `SV-VERSION-TARGETING` (the activated lane). Bigger, more
  open-ended; not active until selected. Captured here so it is not overlooked.

## Goal

Broaden ANVIL's emitted SystemVerilog surface beyond today's flat
module/`always`/instance shape into richer **structured** constructs —
synthesizable, valid-by-construction — to give downstream tools more legal
structural variety to ingest: e.g. `function` / `task` bodies, `interface` /
`modport` boundaries, and nested / multi-level `generate` constructs. Each is a
new legal interaction surface (ROADMAP steering gap 1), not whole-module
behaviour.

## Non-Goals

- No generate-then-filter; every structured construct is valid-by-construction
  (`feedback_rules_first_generation`).
- No default output change until a construct is proven downstream-clean and
  opt-in (`feedback_never_retire_strategies`).
- Not whole-module specification / functional correctness (structure-first per
  ROADMAP steering gap 4).

## Acceptance Criteria

- Each landed structured surface is rules-first, opt-in / default byte-identical
  where it could change output, and proven downstream-clean (Verilator + both
  Yosys modes, and Icarus where applicable).
- Live docs + book + a Knowledge Map fact per durable surface.
- Every leaf committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `STRUCTURED-EMISSION-EXPANSION`
  Status: `proposed`
  Goal: `Richer structured synthesizable SV surfaces (functions / interfaces / nested generate), valid-by-construction.`
  Children: `STRUCTURED-EMISSION-EXPANSION.1`

- ID: `STRUCTURED-EMISSION-EXPANSION.1`
  Status: `proposed`
  Goal: `(Future) Design/decision leaf: inventory candidate structured surfaces (function/task, interface/modport, nested generate), pick the first concrete synthesizable + downstream-clean one, define its valid-by-construction discipline + opt-in knob + downstream gate, and split the tree — before any code.`
  Acceptance: `A decision record naming the first surface, its construction discipline, and its downstream gate; no source change; self-checks clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `STRUCTURED-EMISSION-EXPANSION.1` | `proposed` | Not active. Eligible once this lane is selected for work; first leaf is a design/decision leaf (pick the first structured surface before code). |

## Decisions

- `2026-06-15`: Registered `proposed` by owner roadmap steering as a named future
  capability lane. Not started; `SV-VERSION-TARGETING` was activated first.

## Open Questions

- Which structured surface is highest-leverage first (function/task vs
  interface/modport vs nested generate) — resolved by `.1` when activated.

## Blockers

- None (not active by choice, not dependency).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `STRUCTURED-EMISSION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STRUCTURED-EMISSION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-15`: Created and registered `proposed` (owner-directed sibling lane).
