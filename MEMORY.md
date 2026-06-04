# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `cf16846` - `KNOWLEDGE-MAP-DOC.1 - add Knowledge Map bundle`
- active_work_unit: `KNOWLEDGE-MAP-DOC` -> frontier leaf `KNOWLEDGE-MAP-DOC.3` (`pending`)
- next_action: seed ANVIL-specific retrieval keys on existing decision records, regenerate the map, validate, and close the tree.
- in_flight_uncommitted: `KNOWLEDGE-MAP-DOC.2` commit in progress.
- blockers: none.

## Validation policy
- For workflow-doc memory-architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
