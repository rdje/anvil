# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `99a2682` - `SIGNOFF-SURFACE-EXPANSION.4 - close signoff frontier` (HEAD is now a docs-only book-drift-correction commit landing on top; its hash backfills next update).
- active_work_unit: none. All 9 roadmap phases and every task tree are `done`; no active task-tree frontier remains. Latest closed leaf `LIVE-DOC-BOOK-ALIGNMENT.1` (2026-06-14) realigned the mdBook where delivered Phase 5–9 motifs were still labelled "future" (`synthesizability.md`/`ir.md`/`faq.md`/`core-idea.md`).
- next_action: no open frontier — any new code capability must open a new roadmap-aligned task-tree leaf first. Optional task-tree-exempt follow-up: add dedicated mdBook chapters for the delivered Phase 5/5b/6/7-9 motifs (currently no Motif-Catalogue chapter for parameterization/aggregates/memory/FSM/CDC/artifact-lanes).
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
