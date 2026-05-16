# PHASE-8-FRONTEND-ACCEPT: Frontend/elaboration accept corpora

## Metadata

- Tree ID: `PHASE-8-FRONTEND-ACCEPT`
- Status: `active`
- Roadmap lane: Phase 8 — Frontend/elaboration accept corpora
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Add a source-level artifact family of **compact elaboratable
hierarchies** (not gate-level circuit-IR leaf modules): ANSI ports /
parameter lists, parameter/localparam flows, module instantiation
variants (named/ordered overrides, named/ordered/wildcard ports,
instance arrays), package imports and package-qualified constants/types,
typedef-backed types/structs/unions/enums/atoms, the full `assign` /
`always_comb` / `always @(*)` / `always_ff` / `always_latch` set, and
generate `if`/`for` — backed by a **source-level parameter/hierarchy/
package IR** and an expected-facts manifest.

## Non-Goals

- Forcing this family through the existing gate-level circuit IR; Phase
  8 explicitly introduces a *source-level* IR.
- Behavioural correctness of the elaborated design beyond the declared
  expected elaboration facts.
- The cross-lane selector — that is Phase 9.

## Acceptance Criteria

- A source-level parameter/hierarchy/package IR distinct from the
  circuit IR.
- Reproducible 1–3 module accept corpora with clear tops and
  expected-elaboration-fact manifests.
- Downstream parity checks against those facts.

## Task Tree

- ID: `PHASE-8-FRONTEND-ACCEPT`
  Status: `active`
  Goal: `Source-level elaboratable accept corpora with a dedicated source IR and expected-facts parity.`
  Children: `PHASE-8-FRONTEND-ACCEPT.1`, `PHASE-8-FRONTEND-ACCEPT.2`

- ID: `PHASE-8-FRONTEND-ACCEPT.1`
  Status: `pending`
  Goal: `Design the source-level parameter/hierarchy/package IR and the accept-corpus expected-facts schema in DEVELOPMENT_NOTES.md / book: why a separate IR, what surfaces it must express, manifest schema, parity harness, rejected alternatives. Design-only.`
  Acceptance: `Design entry with source-IR sketch + manifest schema + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-8-FRONTEND-ACCEPT.2`
  Status: `pending`
  Goal: `Implement the source-level IR + accept-corpus generator + manifest + parity harness per .1, behind the artifact-family selector, with a parity gate.`
  Acceptance: `Reproducible accept corpora + manifests; parity green or retained counterexamples; ROADMAP Phase 8 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-8-FRONTEND-ACCEPT.1` | `pending` | The separate source-level IR is the load-bearing design decision; must be designed before any code. Independent of Phase 4/5/6/7. |

## Decisions

- `2026-05-16`: Phase 8 uses a dedicated source-level IR by roadmap
  decree; reusing the gate-level circuit IR is a recorded rejected
  direction (it cannot express the required source surfaces).

## Open Questions

- Degree of reuse of Phase 7's expected-facts manifest machinery.
  Owner: `.1` design (coordinate with `PHASE-7-ORACLE-MICRODESIGN.1`).

## Blockers

- None for `.1`. `.2` coordinates with Phase 7's manifest/parity
  infrastructure; `.1` records the dependency direction.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-8-FRONTEND-ACCEPT.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-8-FRONTEND-ACCEPT.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
