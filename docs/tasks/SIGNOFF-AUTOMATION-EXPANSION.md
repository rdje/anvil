# SIGNOFF-AUTOMATION-EXPANSION: Broaden Downstream Signoff Automation

## Metadata

- Tree ID: `SIGNOFF-AUTOMATION-EXPANSION`
- Status: `proposed`
- Roadmap lane: `Quality — downstream signoff automation breadth`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow

## Goal

Broaden the downstream signoff-acceptance automation beyond the current
per-phase gates, in service of the `project_anvil_north_star` purpose
(surface downstream-tool bugs via valid-by-construction,
downstream-acceptance-quality output). Candidate directions, to be
prioritized in `.1`:

- richer **tool/knob sweep coverage** — exercise the adversarial axis
  matrix (construction strategy, identity mode, factorization level,
  motif/category mix, sequential density, width/depth ranges, the
  probability knobs) without hidden bias toward whichever path is
  currently easiest;
- additional **simulator/frontend acceptance columns** where available
  (extending the Verilator / Yosys / Icarus / `--diff-sim` surface);
- additional **valid-by-construction synthesizable artifact families**
  beyond today's DUT / microdesign / frontend lanes.

This lane is Lane 3 of the three owner-directed post-phase capability
lanes; it is opened `proposed` and promoted to `active` after
`AGENT-MCP-EXPANSION` reaches handoff.

## Non-Goals

- No weakening of the warning-as-failure discipline; warnings stay
  failures, counterexamples are retained with exact seed + effective
  knobs and fed back into invariants/rewrites, never hidden behind
  suppressions.
- No generate-then-filter (rules-first only, per
  `feedback_rules_first_generation`).
- No whole-module spec/oracle or shadow simulator (per `ROADMAP.md`
  non-goal 4 / `book/src/non-goals.md`).
- No retirement of any existing gate or scenario axis (per
  `feedback_never_retire_strategies`).

## Acceptance Criteria

- Each landed leaf adds proven downstream-acceptance coverage (new
  sweep, column, or artifact family) with repo-owned, banked evidence
  (a `tool_matrix`/parity report or equivalent), not narrative claims.
- Default-off / byte-identical wherever a knob could change emitted RTL.
- Live docs (`USER_GUIDE.md`, `ROADMAP.md`, the relevant book chapter)
  updated for any new user-visible gate or column.
- Every leaf committed through `COMMIT.md` with its leaf ID in the
  subject.

## Task Tree

- ID: `SIGNOFF-AUTOMATION-EXPANSION`
  Status: `proposed`
  Goal: `Broaden downstream signoff automation (sweeps, columns, artifact families).`
  Children: `SIGNOFF-AUTOMATION-EXPANSION.1`

- ID: `SIGNOFF-AUTOMATION-EXPANSION.1`
  Status: `pending`
  Goal: `Design/decision leaf: inventory the current gates/columns/axes, identify the single highest-value next signoff-automation increment, and scope the first implementation leaf (and split the tree).`
  Acceptance: `A decision record naming the chosen first increment with rationale; no source change; docs/workflow validation clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SIGNOFF-AUTOMATION-EXPANSION.1` | `pending` | Not on the active frontier yet; this lane activates after `AGENT-MCP-EXPANSION` reaches handoff. |

## Decisions

- `2026-06-15`: Opened `proposed` as Lane 3 (execution order `2 → 3 →
  1`). The first leaf is a design/decision leaf because "broaden signoff
  automation" is open-ended; `.1` must pick one concrete, evidenced
  increment before any code edit.

## Open Questions

- `.1` decides which increment leads: a richer adversarial knob sweep, a
  new acceptance column, or a new artifact family. The deciding factor is
  expected bug-surfacing value per unit of implementation + validation
  cost.

## Blockers

- None. (Sequenced after Lane 2 by choice, not by dependency.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `SIGNOFF-AUTOMATION-EXPANSION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SIGNOFF-AUTOMATION-EXPANSION.1` | `pending` | `pending` |

## Changelog

- `2026-06-15`: Created task tree (Lane 3), opened `proposed`, via
  `CAPABILITY-LANE-OWNERSHIP.1`.
