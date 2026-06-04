# MEMORY-ARCHITECTURE-DOC: Durable Agent Memory Architecture Adoption

## Metadata

- Tree ID: `MEMORY-ARCHITECTURE-DOC`
- Status: `active`
- Roadmap lane: `Workflow / memory architecture`
- Created: `2026-06-04`
- Last updated: `2026-06-04`
- Owner: repo-local workflow

## Goal

Adopt the portable durable-agent-memory architecture in ANVIL so project
continuity is repo-local, git-tracked, bounded on resume, structured by
lifecycle, reachable from tool-neutral entrypoints, and mechanically enforced.

## Non-Goals

- No source-code or generated-RTL behavior changes.
- No roadmap phase reclassification.
- No replacement of the task-tree system; task trees remain layer B.
- No push; commit locally per `COMMIT.md`.

## Acceptance Criteria

- `MEMORY_ARCHITECTURE.md` is present at repo root and referenced from
  `README.md`.
- `docs/decisions/` exists as layer C with an index and migrated durable
  decision/fact records.
- `MEMORY.md` is demoted to a bounded layer-A resume pointer.
- Enforcement exists through a self-check script, local git hooks, CI, and
  tool-neutral bootstrap pointer files.
- End-to-end validation is green and the tree closes through `COMMIT.md`.
- Each completed leaf lands as its own commit.

## Task Tree

- ID: `MEMORY-ARCHITECTURE-DOC`
  Status: `active`
  Goal: `Adopt the full durable memory architecture recommended by the portable standard.`
  Children: `MEMORY-ARCHITECTURE-DOC.1`, `MEMORY-ARCHITECTURE-DOC.2`, `MEMORY-ARCHITECTURE-DOC.3`, `MEMORY-ARCHITECTURE-DOC.4`, `MEMORY-ARCHITECTURE-DOC.5`

- ID: `MEMORY-ARCHITECTURE-DOC.1`
  Status: `done`
  Goal: `Add the portable memory-architecture standard and point README's ramp-up map at it.`
  Acceptance: `MEMORY_ARCHITECTURE.md is present, README names it in the reading order, task-tree/live docs record the slice, and focused docs validation passes.`
  Verification: `MEMORY_ARCHITECTURE.md present; README reading order points at it; rg focused pointer check clean; git diff --check clean; mdbook build book clean; cargo check --all-targets clean. Full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf.`
  Commit: `MEMORY-ARCHITECTURE-DOC.1 — add portable memory standard`

- ID: `MEMORY-ARCHITECTURE-DOC.2`
  Status: `done`
  Goal: `Create docs/decisions/ as layer C and migrate durable decisions/facts out of harness-only or bloated live memory into tracked decision records.`
  Acceptance: `docs/decisions/INDEX.md indexes the records; initial records use Context/Decision/Consequences; live docs reference layer C.`
  Verification: `docs/decisions/INDEX.md indexes ANVIL-specific 0001/0002/0003 records; each record has Context/Decision/Consequences/Links; README points at docs/decisions/*.md; rg focused decision-link check clean; rg donor-residue check clean; git diff --check clean; mdbook build book clean; cargo check --all-targets clean. Full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf.`
  Commit: `MEMORY-ARCHITECTURE-DOC.2 — add layer-C decision records`

- ID: `MEMORY-ARCHITECTURE-DOC.3`
  Status: `done`
  Goal: `Demote MEMORY.md from append-only operational history to a bounded layer-A resume pointer.`
  Acceptance: `MEMORY.md is short, overwrite-only, points to the active work unit/frontier, and retains no historical monolith content beyond what git/task trees/decisions own.`
  Verification: `MEMORY.md reduced to 18 lines; points at README/MEMORY_ARCHITECTURE.md, docs/tasks, docs/TASK_TREE.md, docs/decisions, COMMIT.md, active work unit, next action, in-flight state, blockers, and RAM-safe validation policy; git diff --check clean; mdbook build book clean; cargo check --all-targets clean. Full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf.`
  Commit: `MEMORY-ARCHITECTURE-DOC.3 — demote MEMORY.md to resume pointer`

- ID: `MEMORY-ARCHITECTURE-DOC.4`
  Status: `pending`
  Goal: `Install enforcement: self-check script, local git hooks, CI wiring, hooksPath, and bootstrap pointer files.`
  Acceptance: `scripts/check_memory_architecture.sh checks the memory invariants; .githooks/pre-commit and commit-msg call/enforce them; CI runs the check first; bootstrap files point at README and MEMORY_ARCHITECTURE.md; hooks are active locally.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-ARCHITECTURE-DOC.5`
  Status: `pending`
  Goal: `Run full end-to-end validation, sync live docs/status, and close the tree.`
  Acceptance: `memory-architecture check, mdBook, cargo checks, and full tests pass; live docs reflect the adopted architecture; tree and registry close.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `MEMORY-ARCHITECTURE-DOC.4` | `pending` | Install enforcement after the layer-A resume pointer exists. |

## Decisions

- `2026-06-04`: Adopt the portable memory architecture standard in five
  independently committable leaves matching the owner-provided breakdown.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-04` | `MEMORY-ARCHITECTURE-DOC.1` | `rg` focused pointer check; `git diff --check`; `mdbook build book`; `cargo check --all-targets` | Done — focused checks clean; full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf. |
| `2026-06-04` | `MEMORY-ARCHITECTURE-DOC.2` | `rg` focused decision-link check; `rg` donor-residue check; `git diff --check`; `mdbook build book`; `cargo check --all-targets` | Done — focused checks clean; full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf. |
| `2026-06-04` | `MEMORY-ARCHITECTURE-DOC.3` | `wc -l MEMORY.md`; `git diff --check`; `mdbook build book`; `cargo check --all-targets` | Done — `MEMORY.md` is 18 lines; focused checks clean; full cargo test intentionally skipped per owner instruction/resource policy for this workflow-doc leaf. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MEMORY-ARCHITECTURE-DOC.1` | `MEMORY-ARCHITECTURE-DOC.1 — add portable memory standard` | Hash can be backfilled in a later live-doc update per `COMMIT.md`. |
| `MEMORY-ARCHITECTURE-DOC.2` | `MEMORY-ARCHITECTURE-DOC.2 — add layer-C decision records` | Hash can be backfilled in a later live-doc update per `COMMIT.md`. |
| `MEMORY-ARCHITECTURE-DOC.3` | `MEMORY-ARCHITECTURE-DOC.3 — demote MEMORY.md to resume pointer` | Hash can be backfilled in a later live-doc update per `COMMIT.md`. |
| `MEMORY-ARCHITECTURE-DOC.4` | `pending` | `pending` |
| `MEMORY-ARCHITECTURE-DOC.5` | `pending` | `pending` |

## Changelog

- `2026-06-04`: Created task tree and opened `MEMORY-ARCHITECTURE-DOC.1`.
- `2026-06-04`: Completed `MEMORY-ARCHITECTURE-DOC.1`; frontier moves to `.2`.
- `2026-06-04`: Completed `MEMORY-ARCHITECTURE-DOC.2`; frontier moves to `.3`.
- `2026-06-04`: Completed `MEMORY-ARCHITECTURE-DOC.3`; frontier moves to `.4`.
