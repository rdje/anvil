# USABILITY-LANE-OWNERSHIP: register the owner-directed usability/capability lanes

## Metadata

- Tree ID: `USABILITY-LANE-OWNERSHIP`
- Status: `active`
- Roadmap lane: `Workflow — task-tree ownership of the owner-directed usability/capability lanes`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Register, before any implementation, the seven owner-directed lanes named on
`2026-06-17` (the six "make ANVIL more usable" ideas plus the capability-breadth
lane), each as a tracked top-level task tree, and record the cross-cutting
**API-first mandate** (decision `0017`) that binds them all. Mirrors the
`CAPABILITY-LANE-OWNERSHIP` / `ROADMAP-FOLLOWUP-OWNERSHIP` registration precedent:
ownership-before-work so no code change is ever made without a task-tree leaf
owning it first.

## Non-Goals

- No implementation of any lane here — this tree only registers them and records
  the binding decision. Each lane's work happens under its own tree.
- No code change (docs/workflow only).

## Acceptance Criteria

- Seven lane trees exist under `docs/tasks/` with full metadata, goal,
  non-goals, acceptance (incl. the decision-`0017` API-completeness gate),
  initial task tree, frontier, and logs.
- Decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md)
  (API-first mandate) is recorded and indexed; each lane references it.
- All eight new trees (the seven lanes + this ownership tree) are rows in
  `docs/TASK_TREE.md`; `ROADMAP.md` records the new lanes; `MEMORY.md` +
  `CHANGES.md` updated; self-checks (`check_memory_architecture`,
  KM gen+check) green; committed through `COMMIT.md` with this leaf id.

## Task Tree

- ID: `USABILITY-LANE-OWNERSHIP`
  Status: `active`
  Goal: `Register the seven owner-directed usability/capability lanes + the API-first decision 0017 before any implementation.`
  Children: `USABILITY-LANE-OWNERSHIP.1`

- ID: `USABILITY-LANE-OWNERSHIP.1`
  Status: `done`
  Goal: `Create the seven lane trees (BUG-HUNT-ORCHESTRATION, ACCEPTANCE-DIVERGENCE-HUNTING, DOWNSTREAM-ADAPTER-EXPANSION, KNOB-ERGONOMICS-AND-PRESETS, CI-PACKAGING-DISTRIBUTION, COVERAGE-STEERED-GENERATION, CAPABILITY-BREADTH-EXPANSION), record decision 0017 (API-first: everything MCP-accessible/controllable/steerable/queryable; deep semantic introspection first-class), and wire the index/roadmap/memory.`
  Acceptance: `Seven lane trees + decision 0017 + INDEX row + 8 docs/TASK_TREE.md rows + ROADMAP note + MEMORY + CHANGES; check_memory_architecture + KM gen/check green; docs-only / no code.`
  Result: `Done. Created docs/decisions/0017-api-first-everything-mcp-accessible.md (the cross-cutting API-first mandate, KM answers: front-matter, extends 0004/0005/0011 + feedback_api_for_agents_not_humans) and added its INDEX row. Created seven lane trees in docs/tasks/ — BUG-HUNT-ORCHESTRATION, ACCEPTANCE-DIVERGENCE-HUNTING, DOWNSTREAM-ADAPTER-EXPANSION, KNOB-ERGONOMICS-AND-PRESETS, CI-PACKAGING-DISTRIBUTION, COVERAGE-STEERED-GENERATION, CAPABILITY-BREADTH-EXPANSION — each with the decision-0017 API-completeness gate in its Acceptance Criteria and a design-first .1 ADR frontier (CAPABILITY-BREADTH-EXPANSION carries parallel .1 SV-up-opt + .2 Mealy-FSM design leaves). Added 8 rows to docs/TASK_TREE.md (the 7 lanes + this ownership tree), a ROADMAP "Owner-directed usability + capability lanes (2026-06-17)" section, and refreshed MEMORY.md + CHANGES.md. Docs/workflow only — no src/ touched, so cargo check/clippy/fmt/test are unaffected; DUT byte-identical. Self-checks green (check_memory_architecture; KM gen+check with the new 0017 card).`
  Verification: `bash scripts/check_memory_architecture.sh OK; bash knowledge-map/scripts/gen_knowledge_map.sh + check_knowledge_map.sh OK (new 0017 decision card folded in); no src/ touched ⇒ cargo check/clippy/fmt/test unaffected; docs-only / DUT byte-identical.`
  Commit: `this USABILITY-LANE-OWNERSHIP.1 commit`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none) | — | `.1` done. The lanes themselves are now the live frontiers (each tree's `.1` design ADR); pick per PNT / `feedback_pick_and_roll_at_no_frontier`. |

## Decisions

- `2026-06-17`: Owner directed seven new lanes (six usability + one
  capability-breadth) + the API-first mandate. Registered as task trees before
  any implementation (the `CAPABILITY-LANE-OWNERSHIP` precedent). Decision
  [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md) binds all
  seven (and the existing `SEMANTIC-INTROSPECTION-EXPANSION` +
  `AGENT-MCP-EXPANSION` lanes). Nothing retired.

## Open Questions

- Execution order across the seven lanes — owner said "register for future
  work"; `BUG-HUNT-ORCHESTRATION` is the recommended highest-leverage first lane,
  with `KNOB-ERGONOMICS-AND-PRESETS` + `DOWNSTREAM-ADAPTER-EXPANSION` feeding it.
  *(Not a blocker — PNT self-selects.)*

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `USABILITY-LANE-OWNERSHIP.1` | `check_memory_architecture OK; KM gen+check OK; docs-only (no src/)` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `USABILITY-LANE-OWNERSHIP.1` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Registers the 7 lanes + decision 0017; docs/workflow only; DUT byte-identical. |

## Changelog

- `2026-06-17`: Created task tree + `.1` registered the 7 lanes + decision 0017.
