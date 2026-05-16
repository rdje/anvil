# PHASE-4-HIERARCHY: Exhaust the Phase 4 hierarchy surface

## Metadata

- Tree ID: `PHASE-4-HIERARCHY`
- Status: `active`
- Roadmap lane: Phase 4 — Hierarchy
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Drive Phase 4 (Hierarchy) to honest, complete exhaustion: every remaining
hierarchy structural surface that ANVIL should support before Phase 5 is
landed, proven downstream-clean in the repo-owned Phase 4 matrix gate, and
documented — at which point `ROADMAP.md` Phase 4 can be promoted from
`in progress` to `done`.

"Exhaustion" here is finite and concrete: the set of remaining surfaces is
enumerated up front (leaf `.1`) so the tree has a bounded frontier rather
than an open-ended coverage grind. The completed sub-objective
`HIERARCHY-AWARE-IDENTITY` (r85→r87) is **not** re-tracked here; this tree
owns the remaining *breadth* of hierarchy structure.

## Non-Goals

- Parameterization of hierarchy (parameter-aware child selection,
  parameter-driven parent generation). That is Phase 5 by roadmap
  decree; this tree only ensures Phase 4 is a "real design/instance
  layer" so Phase 5 is unblocked.
- Hierarchy-aware identity. Already delivered by the closed
  `HIERARCHY-AWARE-IDENTITY` tree (`Config::hierarchy_module_dedup`).
- Multi-clock / CDC hierarchy. That is Phase 6.
- Retiring the `rN` cadence. `rN` remains the within-leaf slice cadence;
  this tree tracks the sub-objective decomposition, exactly as
  `HIERARCHY-AWARE-IDENTITY` did (its leaves landed as r85/r86/r87).

## Acceptance Criteria

- A definitive landed-vs-missing audit of the hierarchy surface exists,
  enumerating every remaining registered/helper/composition pattern as a
  concrete leaf (no open-ended "broader patterns" hand-wave).
- Each enumerated remaining surface is implemented as a construction-time
  rule (no generate-then-filter), default-off where it changes existing
  output, and proven downstream-clean in the Phase 4 hierarchy matrix
  gate (`coverage_gaps = []`, Verilator + both Yosys modes all-pass).
- `ROADMAP.md` gains an explicit Phase 4 exit-criteria block, and Phase 4
  is promoted to `done` only once those criteria are satisfied and
  visible in repo-owned artifacts.
- Live docs (`README.md`, `ROADMAP.md`, `USER_GUIDE.md`,
  `CODEBASE_ANALYSIS.md`, `book/src/hierarchy.md`, `book/src/knobs.md`)
  describe every new surface and knob.

## Task Tree

- ID: `PHASE-4-HIERARCHY`
  Status: `active`
  Goal: `Land every remaining Phase 4 hierarchy surface, prove it downstream-clean, and exit-criteria-gate Phase 4 to done.`
  Children: `PHASE-4-HIERARCHY.1`, `PHASE-4-HIERARCHY.2`, `PHASE-4-HIERARCHY.3`

- ID: `PHASE-4-HIERARCHY.1`
  Status: `pending`
  Goal: `Audit gen/hierarchy.rs + the Phase 4 coverage-fact set + the matrix scenario list and produce a definitive landed-vs-missing table of hierarchy surfaces (registered routing variants, helper-placement combinations, parent-composition variants, recursive non-top variants). Output: a "Surface Inventory" section appended to this file enumerating each missing surface as a future leaf with a one-line acceptance. No behaviour change.`
  Acceptance: `This file gains a Surface Inventory table; every "missing" row becomes a concrete leaf with acceptance criteria; the tree's Children list and Current Frontier are rewritten from it; mdbook/sanity unaffected (doc-only slice).`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-4-HIERARCHY.2`
  Status: `pending`
  Goal: `Implement the first missing hierarchy surface identified by .1 (single rN-style construction-time slice), default-off if it changes existing output, with a focused proof and a matrix scenario.`
  Acceptance: `Focused proof passes for all four ConstructionStrategy values; the surface's saw_* fact fires in the Phase 4 gate; gate stays coverage_gaps=[] and downstream-clean; cargo fmt/clippy/test/mdbook clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-4-HIERARCHY.3`
  Status: `pending`
  Goal: `Phase 4 exit-criteria closure: once .1's inventory is exhausted (all missing surfaces landed downstream-clean), add an explicit Phase 4 exit-criteria block to ROADMAP.md and promote Phase 4 to done with the closing matrix-gate artifact recorded.`
  Acceptance: `ROADMAP.md has a Phase 4 exit-criteria block; all inventory leaves are done; the closing Phase 4 gate is downstream-clean at the final scenario count; ROADMAP Phase 4 label = done; README/CODEBASE_ANALYSIS/book updated.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-4-HIERARCHY.1` | `pending` | Cannot claim "exhaustion" without first enumerating the finite remaining surface set. Diagnostic, no-risk, unblocks every other leaf. |

`.2` and `.3` are intentionally placeholders until `.1` rewrites the
tree from the real Surface Inventory. `.1` is expected to split `.2`
into N concrete per-surface leaves.

## Decisions

- `2026-05-16`: Phase 4 "complete exhaustion" is bounded by an explicit
  up-front Surface Inventory (leaf `.1`) rather than treated as an
  open-ended coverage grind. Rationale: the roadmap's "broader registered
  hierarchy patterns remain future work" is not a finite exit condition;
  a concrete enumerated inventory makes the frontier finite and the
  `done` promotion defensible.
- `2026-05-16`: `rN` is preserved as the within-leaf slice cadence. This
  tree only owns the decomposition, mirroring the closed
  `HIERARCHY-AWARE-IDENTITY` precedent. Consistent with the
  never-retire-strategies rule and `docs/TASK_TREE.md` adoption scope.
- `2026-05-16`: `ROADMAP.md` currently has no Phase 4 exit-criteria
  block. Authoring one is in-scope for this tree (leaf `.3`); Phase 4
  must not be promoted to `done` from narrative progress alone.

## Open Questions

- Exactly which enumerated surfaces are "must-have before Phase 5" vs
  "nice-to-have breadth"? Owner: leaf `.1` audit will classify each
  inventory row as `blocking-phase5` or `breadth`; only `blocking-phase5`
  rows gate the `done` promotion, `breadth` rows may be deferred with a
  recorded consequence. Does not block `.1`.

## Blockers

- None. Frontier (`.1`) is a no-risk diagnostic slice.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-4-HIERARCHY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-4-HIERARCHY.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase. Frontier opens at `.1` (Surface
  Inventory audit).
