# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `e5234e5` - `MEMORY-ARCHITECTURE-DOC.5 - validate and close`
- active_work_unit: `KNOWLEDGE-MAP-DOC` -> frontier leaf `KNOWLEDGE-MAP-DOC.2` (`pending`)
- next_action: install Knowledge Map generation/enforcement, generate the first map, validate, and commit `.2`.
- in_flight_uncommitted: `KNOWLEDGE-MAP-DOC.1` commit in progress.
- blockers: none.

## Validation policy
- For workflow-doc memory-architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
