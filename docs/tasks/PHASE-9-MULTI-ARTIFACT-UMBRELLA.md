# PHASE-9-MULTI-ARTIFACT-UMBRELLA: Multi-artifact ANVIL umbrella

## Metadata

- Tree ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
- Status: `active`
- Roadmap lane: Phase 9 — Multi-artifact ANVIL umbrella
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Add an **artifact-family selector** so one tool drives every
valid-by-construction synthesizable family without overloading one
generator path with contradictory promises; unify reproducibility,
manifests, seed handling, knob plumbing, corpus output layout, and
downstream checking across families, while preserving the doctrinal
lane separation (synthesizable DUT RTL; oracle-backed positive
micro-design; frontend/elaboration accept; future valid synthesizable
lanes).

## Non-Goals

- Inventing new artifact families here — Phase 9 unifies the families
  delivered by Phases 1–8, it does not add new ones.
- Blurring the lanes into one "random SV files" notion (the explicit
  anti-goal this phase exists to prevent).

## Acceptance Criteria

- An explicit mode/lane selector covering the DUT RTL lane, the
  oracle-backed micro-design lane, and the frontend/elaboration accept
  lane.
- Unified reproducibility/manifest/seed/knob/output/downstream-check
  plumbing across lanes.
- ANVIL can honestly present itself as the go-to multi-lane tool.

## Task Tree

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA`
  Status: `active`
  Goal: `Unify all delivered artifact lanes under one explicit selector with shared plumbing.`
  Children: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`, `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1`
  Status: `pending`
  Goal: `Design the artifact-family selector + shared plumbing abstraction in DEVELOPMENT_NOTES.md / book: lane interface, shared reproducibility/manifest/seed/output contract, CLI/selector surface, migration of existing lanes, rejected alternatives. Design-only.`
  Acceptance: `Design entry with selector/plumbing abstraction + lane-migration plan + >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2`
  Status: `pending`
  Goal: `Implement the selector + shared plumbing per .1 and migrate the delivered lanes onto it. Blocked until at least two artifact lanes exist (Phase 1–4 DUT lane + one of Phase 7/8).`
  Acceptance: `One tool selects all delivered lanes; shared plumbing proven; ROADMAP Phase 9 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `pending` | The selector/plumbing abstraction can and should be designed early so Phases 7/8 build lane-compatible; design is unblocked. |

## Decisions

- `2026-05-16`: Designing the umbrella abstraction early (`.1`) is
  in-scope even though implementation (`.2`) is far off, so Phase 7/8
  artifact lanes are built selector-compatible rather than retrofitted.

## Open Questions

- Whether the selector is a CLI subcommand, a top-level `--artifact`
  flag, or separate binaries. Owner: `.1` design.

## Blockers

- `PHASE-9-MULTI-ARTIFACT-UMBRELLA.2` is blocked by: fewer than two
  delivered artifact lanes. Unblock condition: the DUT lane plus at
  least one of Phase 7/8 lanes exist. Run `.1` (design) meanwhile.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-9-MULTI-ARTIFACT-UMBRELLA.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
