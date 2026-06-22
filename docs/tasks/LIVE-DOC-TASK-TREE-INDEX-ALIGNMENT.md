# LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT: Align Task-Tree Index Status With Tree Files

## Metadata

- Tree ID: `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT`
- Status: `done`
- Roadmap lane: `Live docs / task-tree index ↔ tree-file status alignment`
- Created: `2026-06-22`
- Last updated: `2026-06-22`
- Owner: repo-local workflow

## Goal

`docs/TASK_TREE.md`'s `Active Task Trees` index row for
`LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT` still read `active` even though
that tree's own file (`docs/tasks/LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.md`)
is `done` and was closed by commit `d3eb968`. The index (the resume
surface a fresh session reads to find the live frontier) therefore
drifted from the authoritative tree-file state. Bring the index back
into lockstep with the tree files so the roadmap ↔ codebase ↔ live docs
stay aligned with no drift, per the standing owner mandate.

This is a **docs-only / live-doc** alignment (exempt from the code
task-tree-ownership doctrine), tracked as a tree by the repo's
`LIVE-DOC-*-ALIGNMENT` convention for traceability.

## Non-Goals

- No code changes.
- No status change to any tree whose index row already matches its file.
- No rewrite of historical verification logs or closed-tree prose.

## Acceptance Criteria

- The `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT` index row in
  `docs/TASK_TREE.md` reads `done` and its frontier cell describes
  closure (not a pending `.1` frontier).
- A full audit of every `docs/tasks/*.md` `- Status:` value against its
  `docs/TASK_TREE.md` index row confirms no other index/file status
  mismatch.
- This tree has its own index row.
- No aspirational claims: the closure is grounded in the on-disk tree
  file + the `d3eb968` commit.
- `CHANGES.md` and `MEMORY.md` are refreshed.
- Doctrine checks pass (`scripts/check_doctrines.sh`).

## Task Tree

- ID: `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT`
  Status: `done`
  Goal: `Correct the stale task-tree index status row and confirm no other index drift.`
  Children: `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1`

- ID: `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1`
  Status: `done`
  Goal: `Patch the LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT index row (active→done) and add this tree's row.`
  Acceptance: `Index row reads done with a closure frontier cell; full per-file vs per-index status audit shows no remaining mismatch.`
  Verification: `scripts/check_doctrines.sh`; `git diff --check`; per-file Status grep cross-check against the index.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-22`: Edit only the one drifted index row. A full audit of all
  56 tree files' `- Status:` metadata against their index rows found this
  as the single mismatch (every other `active`/`done` row already
  matched its file), so no other index cell is touched. The root cause
  was that `d3eb968` closed the `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT`
  tree file but left its index row at `active`.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-22` | `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1` | `scripts/check_doctrines.sh` (4 doctrines PASS; code-scoped checks exempt — docs commit); `git diff --check`; per-file `- Status:` grep cross-checked against every `docs/TASK_TREE.md` index row | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1` | `LIVE-DOC-TASK-TREE-INDEX-ALIGNMENT.1 — align task-tree index status` | Pending hash; closes tree. |

## Changelog

- `2026-06-22`: Created after a session-bootstrap audit found the
  `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT` index row stale at `active`
  while its tree file was already `done`.
- `2026-06-22`: `.1` completed and the tree closed — index row corrected
  to `done`, full-index audit confirmed no other drift;
  `scripts/check_doctrines.sh` green; docs-only / DUT byte-identical.
