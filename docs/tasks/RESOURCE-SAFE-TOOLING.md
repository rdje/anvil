# RESOURCE-SAFE-TOOLING: Host-Crash-Safe Job Runner

## Metadata

- Tree ID: `RESOURCE-SAFE-TOOLING`
- Status: `done`
- Roadmap lane: `Quality / workflow — resource-safe validation`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow

## Goal

Give the repo a reusable RAM watchdog so heavy `cargo` builds/tests and
`tool_matrix` sweeps cannot drive this RAM-limited host to the ~95%-used
reboot level. Operationalizes the resource policy in
`docs/decisions/0003-resource-safe-validation.md`.

## Non-Goals

- No change to generator behaviour, IR, knobs, or generated RTL.
- Not a replacement for focused validation — it is a safety net for the
  cases where a heavier run is genuinely needed.

## Acceptance Criteria

- A tracked, executable `scripts/ram_guard.sh` wraps an arbitrary
  command, aborts it before a configurable threshold (default 88%),
  exits `99` on abort, and otherwise propagates the wrapped status.
- Works on this macOS host (`memory_pressure`); degrades sensibly on
  Linux (`/proc/meminfo`).
- Verified with a pass case, an abort case, and a failure-propagation
  case.
- Live docs refreshed; mdBook/USER_GUIDE updated if it becomes a
  user-facing workflow.

## Task Tree

- ID: `RESOURCE-SAFE-TOOLING`
  Status: `done`
  Goal: `Provide a host-crash-safe job runner and document it.`
  Children: `RESOURCE-SAFE-TOOLING.1`, `RESOURCE-SAFE-TOOLING.2`

- ID: `RESOURCE-SAFE-TOOLING.1`
  Status: `done`
  Goal: `Add scripts/ram_guard.sh with a configurable RAM-abort threshold.`
  Acceptance: `Tracked executable script; pass/abort/propagation behaviour verified; exit 99 on abort.`
  Verification: `scripts/ram_guard.sh --threshold 99 -- bash -c 'exit 0' (0); --threshold 1 -- sleep 8 (99); --threshold 99 -- bash -c 'exit 7' (7).`
  Commit: `RESOURCE-SAFE-TOOLING.1 - add RAM watchdog runner`

- ID: `RESOURCE-SAFE-TOOLING.2`
  Status: `done`
  Goal: `Document the watchdog in USER_GUIDE.md once its role in the validation workflow is settled.`
  Acceptance: `USER_GUIDE.md describes ram_guard.sh usage; no drift between script flags and docs.`
  Verification: `USER_GUIDE.md "Resource-safe runs on RAM-limited hosts" section documents scripts/ram_guard.sh (--threshold, -- CMD, exit 99) + complementary tactics (parallelism caps, focused targets, chunked --resume sweeps) consistent with the script and docs/decisions/0003; mdbook unaffected (USER_GUIDE is not in the book). git diff --check clean.`
  Commit: `RESOURCE-SAFE-TOOLING.2 - document ram_guard in USER_GUIDE`

## Current Frontier

Empty — the tree is `done`. `.1` (ram_guard.sh) and `.2` (USER_GUIDE
docs) are both complete.

## Decisions

- `2026-06-14`: Default abort threshold is 88% used — below the 90%
  danger line and well below the ~95% reboot line. Pollable via
  `--threshold`.
- `2026-06-14`: Keep `target/` warm during iterative compile work
  (incremental builds are far lighter on RAM than cold full rebuilds);
  reclaim disk with `cargo clean` only when no further compiles are
  imminent.

## Open Questions

- Whether to make the guard the default wrapper for the documented
  full-suite command in `COMMIT.md` (deferred to `.2`).

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `RESOURCE-SAFE-TOOLING.1` | pass case (exit 0), abort case (exit 99, read 26% used), failure-propagation (exit 7); `git diff --check`; self-checks | passed |
| `2026-06-14` | `RESOURCE-SAFE-TOOLING.2` | USER_GUIDE "Resource-safe runs" section added; `git diff --check` clean | passed (docs-only) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RESOURCE-SAFE-TOOLING.1` | `RESOURCE-SAFE-TOOLING.1 - add RAM watchdog runner` | Pending hash. |
| `RESOURCE-SAFE-TOOLING.2` | `RESOURCE-SAFE-TOOLING.2 - document ram_guard in USER_GUIDE` | Pending hash; closes tree. |

## Changelog

- `2026-06-14`: Created tree; landed `.1` (ram_guard.sh); `.2` docs pending.
- `2026-06-14`: Landed `.2` (USER_GUIDE ram_guard docs); tree CLOSED.
