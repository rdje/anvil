# LIVE-DOC-IDENTITY-ALIGNMENT: Align Identity Live Docs

## Metadata

- Tree ID: `LIVE-DOC-IDENTITY-ALIGNMENT`
- Status: `done`
- Roadmap lane: `Live docs / NodeId identity status`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Remove stale live-doc prose about ANVIL's identity/factorization status so
the codebase analysis matches the just-landed FSM identity merge and the
existing hierarchy module-dedup layer.

## Non-Goals

- No code changes.
- No new identity behavior.
- No phase-status promotion.

## Acceptance Criteria

- `CODEBASE_ANALYSIS.md` no longer describes identity status as flop-only.
- The hierarchy section acknowledges the existing module-dedup identity
  layer instead of describing hierarchy-aware identity as entirely open.
- `CHANGES.md`, `MEMORY.md`, and `docs/TASK_TREE.md` are updated.
- Focused documentation checks pass.

## Task Tree

- ID: `LIVE-DOC-IDENTITY-ALIGNMENT`
  Status: `done`
  Goal: `Align live identity docs with current code reality.`
  Children: `LIVE-DOC-IDENTITY-ALIGNMENT.1`

- ID: `LIVE-DOC-IDENTITY-ALIGNMENT.1`
  Status: `done`
  Goal: `Patch stale identity-status prose in CODEBASE_ANALYSIS.md.`
  Acceptance: `CODEBASE_ANALYSIS.md reflects combinational, flop, FSM, and opt-in module-dedup identity coverage, with remaining gaps limited to broader sequential, memory-state, and deeper hierarchical equivalence.`
  Verification: `scripts/check_memory_architecture.sh`; `git diff --check`; `mdbook build book`.
  Commit: `LIVE-DOC-IDENTITY-ALIGNMENT.1 - align identity live docs`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-IDENTITY-ALIGNMENT.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Treat the stale status text as live-doc drift, not a new
  implementation slice. The correction is docs-only and does not change
  roadmap phase labels.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `LIVE-DOC-IDENTITY-ALIGNMENT.1` | `scripts/check_memory_architecture.sh`; `git diff --check`; `mdbook build book` | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-IDENTITY-ALIGNMENT.1` | `LIVE-DOC-IDENTITY-ALIGNMENT.1 - align identity live docs` | Pending hash; closes tree. |

## Changelog

- `2026-06-05`: Created, completed, and closed the docs-alignment tree.
