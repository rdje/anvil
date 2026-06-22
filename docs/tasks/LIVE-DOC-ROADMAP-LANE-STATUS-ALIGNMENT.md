# LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT: Align Roadmap Owner-Directed Lane Status

## Metadata

- Tree ID: `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT`
- Status: `done`
- Roadmap lane: `Live docs / roadmap owner-directed-lane status`
- Created: `2026-06-22`
- Last updated: `2026-06-22`
- Owner: repo-local workflow

## Goal

`ROADMAP.md`'s three owner-directed-lane sections had drifted behind the
delivered reality already recorded in `docs/TASK_TREE.md` (current),
`README.md` (current), the `docs/decisions/` records, and the code/git
log. Bring the roadmap's lane-status prose back into lockstep so the
roadmap, the codebase, and the live docs are aligned with no drift, per
the standing owner mandate (roadmap ↔ codebase ↔ book locked).

This is a **docs-only / live-doc** alignment (exempt from the code
task-tree-ownership doctrine), tracked as a tree by the repo's
`LIVE-DOC-*-ALIGNMENT` convention for traceability.

## Non-Goals

- No code changes.
- No rewrite of historical verification logs or closed-phase prose.
- No roadmap phase promotion (all 9 numbered phases already `done`).
- No edits to `docs/TASK_TREE.md` lane rows (verified current) beyond
  adding this tree's own index row.

## Acceptance Criteria

- `ROADMAP.md`'s owner-directed-lane status matches the authoritative
  current status in `docs/TASK_TREE.md` / `README.md` / `docs/decisions/`
  for every lane, specifically:
  - `STRUCTURED-EMISSION-EXPANSION`: the **fifth** (`cone_function`,
    decision `0016`) and **sixth** (`multi_output_task`, decision `0025`,
    widened to `k>2`) surfaces are recorded as delivered (was stale at
    the fourth surface).
  - `AGENT-MCP-EXPANSION`: marked `done` / closed (was `active`).
  - `SIGNOFF-AUTOMATION-EXPANSION`: marked `active` with `.2` delivered
    (was `proposed`).
  - `IDENTITY-DEEPENING`: `.3b.2b` cross-module sequential dedup recorded
    delivered (was "`.3` is the named future leaf").
  - `DOWNSTREAM-ADAPTER-EXPANSION`: `sv2v` (`.2b`) + `slang` (`.2c`)
    recorded delivered (was stale at `.2a.2`).
  - `KNOB-ERGONOMICS-AND-PRESETS`: recorded delivered (decision `0021`).
  - `CI-PACKAGING-DISTRIBUTION`: recorded delivered (decision `0022`;
    `release.yml` + `action.yml` on disk).
  - `CAPABILITY-BREADTH-EXPANSION`: Mealy FSM recorded delivered
    (decision `0024`; was framed as planned).
- No aspirational claims: every "delivered" statement is grounded in an
  on-disk artifact (decision record, source symbol, or workflow file).
- `CHANGES.md` and `MEMORY.md` are refreshed.
- Doctrine checks pass (`scripts/check_doctrines.sh`).

## Task Tree

- ID: `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT`
  Status: `done`
  Goal: `Refresh ROADMAP owner-directed-lane status to delivered reality.`
  Children: `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1`

- ID: `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1`
  Status: `done`
  Goal: `Patch the eight stale owner-directed-lane status statements in ROADMAP.md.`
  Acceptance: `Every owner-directed-lane status statement in ROADMAP.md matches docs/TASK_TREE.md / README.md / docs/decisions; verified by spot grep against the decision records and on-disk artifacts.`
  Verification: `scripts/check_doctrines.sh`; `git diff --check`; grep cross-check.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-22`: Update `ROADMAP.md` current-status lane prose only. The
  `docs/TASK_TREE.md` index rows were audited and found already current
  (their tails reflect all delivered work), so they are not edited.
  Closed-phase narrative (e.g. Phase 6 "Moore-output only") stays as the
  historically-accurate description of that phase's delivered scope; the
  later delivery of Mealy is recorded in its owning capability lane.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-22` | `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1` | `scripts/check_doctrines.sh` (4 doctrines PASS; code-scoped checks exempt — docs commit); `git diff --check`; grep cross-check of every "delivered" claim against `docs/decisions/`, source symbols, git log, and on-disk workflow files | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1` | `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1 — align roadmap owner-directed-lane status` | Pending hash; closes tree. |

## Changelog

- `2026-06-22`: Created the roadmap lane-status alignment tree after a
  session-bootstrap audit found `ROADMAP.md` drifted behind delivered
  reality on eight owner-directed-lane statements.
- `2026-06-22`: `.1` completed and the tree closed — all eight statements
  aligned; `scripts/check_doctrines.sh` green; docs-only / DUT
  byte-identical.
