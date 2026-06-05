# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `ca6a449` - `MEMORY-STATE-IDENTITY.2 - record reset-memory blocker`
- active_work_unit: `MEMORY-STATE-IDENTITY.3` docs-only closeout; memory-state identity is closed with reset-less memories remaining instance-local.
- next_action: commit `MEMORY-STATE-IDENTITY.3`, clear `git_message_brief.txt`, then start `HIERARCHY-SEMANTIC-IDENTITY.1`.
- in_flight_uncommitted: memory closeout docs/task-tree/live-doc edits are prepared; validation pending.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
