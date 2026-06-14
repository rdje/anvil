# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `95201a2` - `LIVE-DOC-BOOK-ALIGNMENT.1 - realign mdBook with delivered motifs` (the `RESOURCE-SAFE-TOOLING.1` ram-guard commit lands on top; its hash backfills next update).
- active_work_unit: `RESOURCE-SAFE-TOOLING` — `.1` done (`scripts/ram_guard.sh`), frontier `.2` (document the runner in USER_GUIDE) pending. All 9 roadmap phases and every other task tree remain `done`.
- next_action: owner directed "build new capability" (2026-06-14) — open a NEW roadmap-aligned capability task tree (candidate from deferred boundaries) BEFORE any `src/` change, then implement leaf-by-leaf with the book/roadmap/live-docs synced in the SAME slice. Also: `RESOURCE-SAFE-TOOLING.2` (ram-guard docs). Run heavy builds via `scripts/ram_guard.sh --`.
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
