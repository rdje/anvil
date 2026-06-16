# LIVE-DOC-CODEBASE-ALIGNMENT: Align Codebase-Analysis Snapshot With Live src/ Tree

## Metadata

- Tree ID: `LIVE-DOC-CODEBASE-ALIGNMENT`
- Status: `done`
- Roadmap lane: `Live docs / CODEBASE_ANALYSIS ↔ workspace alignment`
- Created: `2026-06-14`
- Last updated: `2026-06-17` (**`.2` landed** — session-bootstrap deep-dive
  surfaced that the Snapshot integration-test count had drifted again, now
  stale at "six" while `tests/` holds eight files: the two
  `SV-VERSION-TARGETING` test files `sv_version` (`.2b.1` down-gating /
  byte-identical proofs) and `sv_version_downstream` (`.2b.2a` per-version
  `--language 1800-20xx` real-tool acceptance proofs) were uncounted.
  Corrected to eight and the tool-gated `#[ignore]` note widened to four
  files. Live-doc edit only; tree returns to `done`.)
- Owner: repo-local workflow

## Goal

Bring `CODEBASE_ANALYSIS.md` — the authoritative live workspace snapshot —
back into agreement with the real `src/` tree after a session-bootstrap
deep-dive (`SESSION_BOOTSTRAP.md` step 3) found the module-map ASCII tree
and the Snapshot test count had lagged behind delivered modules.

## Non-Goals

- No code changes (live-doc edit only; exempt from task-tree-ownership of
  code, but tracked here for continuity per the owner tracking directive).
- No roadmap phase promotion or status change.
- No rewrite of historical verification logs in other trees.

## Acceptance Criteria

- `CODEBASE_ANALYSIS.md` module map lists every first-class `src/` module,
  including `ir/param.rs`, `ir/aggregate.rs`, `frontend/`, `umbrella/`, and
  `diff_sim/` (previously omitted while `microdesign/` was listed).
- `CODEBASE_ANALYSIS.md` Snapshot reports the real integration-test count
  (eight: `pipeline`, `book_examples`, `snapshots`, `diff_sim`,
  `microdesign_parity`, `frontend_parity`, `sv_version`,
  `sv_version_downstream`).
- No other code/doc/book drift remains (verified by read-only audits).
- `CHANGES.md` and `MEMORY.md` refreshed.
- Focused documentation checks pass.

## Task Tree

- ID: `LIVE-DOC-CODEBASE-ALIGNMENT`
  Status: `done`
  Goal: `Re-sync the CODEBASE_ANALYSIS workspace snapshot with src/.`
  Children: `LIVE-DOC-CODEBASE-ALIGNMENT.1`, `LIVE-DOC-CODEBASE-ALIGNMENT.2`

- ID: `LIVE-DOC-CODEBASE-ALIGNMENT.1`
  Status: `done`
  Goal: `Add the 5 omitted modules to the module map and correct the integration-test count 3 -> 6.`
  Acceptance: `CODEBASE_ANALYSIS.md module map and Snapshot match the live src/ tree and tests/ directory; two read-only audits confirm no further drift.`
  Verification: `git ls-files 'src/**'` + `grep 'pub mod' src/lib.rs`; `ls tests/`; `scripts/check_memory_architecture.sh`; `git diff --check`.
  Commit: `LIVE-DOC-CODEBASE-ALIGNMENT.1 - sync module map + test surface`

- ID: `LIVE-DOC-CODEBASE-ALIGNMENT.2`
  Status: `done`
  Goal: `Correct the integration-test count 6 -> 8 in the CODEBASE_ANALYSIS Snapshot (add the SV-VERSION-TARGETING test files sv_version + sv_version_downstream) and widen the tool-gated #[ignore] note from "the last three" to the four tool-gated files (diff_sim, microdesign_parity, frontend_parity, sv_version_downstream). Surfaced by the session-bootstrap deep-dive.`
  Acceptance: `CODEBASE_ANALYSIS Snapshot lists eight integration tests matching ls tests/*.rs; the #[ignore] note names exactly the four tool-gated files; check_memory_architecture clean.`
  Verification: `ls tests/*.rs` (8 files); `grep -c '#\[ignore' tests/sv_version*.rs` (sv_version 0, sv_version_downstream 4); `scripts/check_memory_architecture.sh`; `git diff --check`.
  Commit: `LIVE-DOC-CODEBASE-ALIGNMENT.2 - correct integration-test count 6->8`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-CODEBASE-ALIGNMENT.1` | `done` | Completed and closed the tree. |
| 2 | `LIVE-DOC-CODEBASE-ALIGNMENT.2` | `done` | Re-sync surfaced by the next bootstrap deep-dive; tree re-closed. |

## Decisions

- `2026-06-14`: Track this live-doc sync with a dedicated tree (mirroring
  the closed `LIVE-DOC-*-ALIGNMENT` trees) so the commit carries a leaf id
  the `commit-msg` hook accepts and so the work is continuity-tracked, even
  though live-doc edits are exempt from requiring a tree.
- `2026-06-14`: Edit the module map and Snapshot only; the file's
  phase-coverage table already described the five modules, so the fix is a
  completeness sync, not a rewrite.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `LIVE-DOC-CODEBASE-ALIGNMENT.1` | `git ls-files 'src/**'`; `grep 'pub mod' src/lib.rs`; `ls tests/`; `scripts/check_memory_architecture.sh`; `git diff --check` | passed |
| `2026-06-17` | `LIVE-DOC-CODEBASE-ALIGNMENT.2` | `ls tests/*.rs` (8); `grep -c '#\[ignore' tests/sv_version*.rs` (0 / 4); `scripts/check_memory_architecture.sh`; `git diff --check` | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-CODEBASE-ALIGNMENT.1` | `LIVE-DOC-CODEBASE-ALIGNMENT.1 - sync module map + test surface` | Closed tree (first pass). |
| `LIVE-DOC-CODEBASE-ALIGNMENT.2` | `LIVE-DOC-CODEBASE-ALIGNMENT.2 — correct integration-test count 6->8` | Pending hash; re-closes tree. |

## Changelog

- `2026-06-14`: Created, completed, and closed the codebase-analysis alignment tree.
- `2026-06-17`: Reopened for `.2` — the bootstrap deep-dive found the
  integration-test count had drifted again (six → eight after the two
  `SV-VERSION-TARGETING` test files landed). Corrected the Snapshot and
  re-closed the tree.
