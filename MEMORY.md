# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `d0fd9f6` - `MEMORY-STATE-IDENTITY.3 - close memory frontier`
- active_work_unit: `HIERARCHY-SEMANTIC-IDENTITY.1` adds default-off bounded semantic module dedup for pure combinational leaves plus design metrics/docs.
- next_action: finish validation, commit `HIERARCHY-SEMANTIC-IDENTITY.1`, clear `git_message_brief.txt`, then evaluate `HIERARCHY-SEMANTIC-IDENTITY.2`.
- in_flight_uncommitted: code/docs/task-tree/knowledge-map edits are validated and ready to commit.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
