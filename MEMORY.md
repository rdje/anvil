# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `447da5b` - `HIERARCHY-SEMANTIC-IDENTITY.1 - add semantic module dedup`
- active_work_unit: `HIERARCHY-SEMANTIC-IDENTITY.2` extends bounded semantic module dedup to pure-combinational wrappers with recursively proven children.
- next_action: finish focused validation, commit `HIERARCHY-SEMANTIC-IDENTITY.2`, clear `git_message_brief.txt`, then close `HIERARCHY-SEMANTIC-IDENTITY.3`.
- in_flight_uncommitted: code/docs/task-tree/knowledge-map edits for `.2` are under validation and ready for commit workflow after gates pass.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
