# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `71c0fd1` - `HIERARCHY-SEMANTIC-IDENTITY.2 - extend semantic dedup to wrappers`
- active_work_unit: `HIERARCHY-SEMANTIC-IDENTITY.3` closes the hierarchy semantic identity tree with landed classes and deferred proof boundaries.
- next_action: finish docs validation, commit `HIERARCHY-SEMANTIC-IDENTITY.3`, clear `git_message_brief.txt`, then pick `SIGNOFF-SURFACE-EXPANSION.1`.
- in_flight_uncommitted: docs/task-tree/live-doc closeout edits for `.3` are ready for focused validation and commit workflow.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
