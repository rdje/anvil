# PHASE-5-PARAMETERIZATION: Parameterized modules and instances

## Metadata

- Tree ID: `PHASE-5-PARAMETERIZATION`
- Status: `active`
- Roadmap lane: Phase 5 — Parameterization
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Generated modules take `parameter` declarations for widths;
instantiation picks parameter values from allowed ranges;
parameter-dependent widths propagate correctly through cone generation;
and parameter-aware identity stays sound (different parameter values
never alias to one `NodeId` or one module instance unless genuinely
equivalent).

## Non-Goals

- Source-level package/typedef parameter flows for an accept corpus —
  that is Phase 8.
- Parameter-driven generate/`for` elaboration corpora with
  expected-facts manifests — that is Phase 7.
- Non-width parameters (e.g., behavioural-mode switches) beyond what
  width parameterization needs.

## Acceptance Criteria

- A concrete Phase 5 implementation plan derived from
  `book/src/ir.md` "Future extensions / Parameters and generics".
- `parameter`-bearing modules emitted, valid by construction,
  downstream-clean (Verilator + both Yosys modes).
- Parameter-aware identity proof: distinct parameter values do not
  collapse under `NodeId`/module dedup unless structurally equivalent.
- Live docs + a Phase 5 matrix gate shape.

## Task Tree

- ID: `PHASE-5-PARAMETERIZATION`
  Status: `active`
  Goal: `Deliver parameterized modules/instances with sound parameter-aware identity.`
  Children: `PHASE-5-PARAMETERIZATION.1`, `PHASE-5-PARAMETERIZATION.2`

- ID: `PHASE-5-PARAMETERIZATION.1`
  Status: `pending`
  Goal: `Lift book/src/ir.md "Parameters and generics" into a concrete Phase 5 implementation + identity-soundness plan in DEVELOPMENT_NOTES.md (IR shape, propagation, identity rule, proof shape, rejected alternatives). Design-only; not blocked by Phase 4.`
  Acceptance: `DEVELOPMENT_NOTES.md Phase 5 design entry with >=1 rejected alternative; no code change; mdbook clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-5-PARAMETERIZATION.2`
  Status: `pending`
  Goal: `Implement the Phase 5 plan from .1 across the IR/emit/generator, default-off until proven, with a Phase 5 matrix gate. Blocked by Phase 4 being a real design/instance layer (PHASE-4-HIERARCHY done).`
  Acceptance: `Parameterized designs downstream-clean; parameter-aware identity proof passes; ROADMAP Phase 5 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5-PARAMETERIZATION.1` | `pending` | Design work can proceed before Phase 4 closes; `.2` (implementation) is blocked by `PHASE-4-HIERARCHY`. |

## Decisions

- `2026-05-16`: Split design (`.1`, unblocked) from implementation
  (`.2`, blocked by Phase 4) so Phase 5 thinking can advance in
  parallel without violating the roadmap's hard Phase 4 prerequisite.

## Open Questions

- Whether parameter-aware identity extends `NodeId`/the dedup signature
  or is a separate guard. Owner: `.1` design. Does not block `.1`.

## Blockers

- `PHASE-5-PARAMETERIZATION.2` is blocked by: `PHASE-4-HIERARCHY` not
  yet `done`. Unblock condition: Phase 4 promoted to `done`. Run
  `PHASE-5-PARAMETERIZATION.1` (design) meanwhile.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-5-PARAMETERIZATION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-5-PARAMETERIZATION.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
