# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `5d683ff` - `MEMORY-ARCHITECTURE-DOC.2 - add layer-C decisions`
- active_work_unit: `MEMORY-ARCHITECTURE-DOC` -> frontier leaf `MEMORY-ARCHITECTURE-DOC.4` (`pending`)
- next_action: install the memory-architecture enforcement kit: self-check script, hooks, CI step, and bootstrap pointers.
- in_flight_uncommitted: `MEMORY-ARCHITECTURE-DOC.3` commit in progress.
- blockers: none.

## Validation policy
- For workflow-doc memory-architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
