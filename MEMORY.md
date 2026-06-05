# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `06b89f2` - `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1 - key flops by domain`
- active_work_unit: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2` is implemented/validated; exact reset-defined self-hold flops now merge when width/reset/domain match, while resetless/reset/domain/width mismatch and non-exact feedback stay distinct.
- next_action: commit `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2`, clear `git_message_brief.txt`, then run `SEQUENTIAL-COINDUCTIVE-IDENTITY.3` closeout.
- in_flight_uncommitted: source, tests, mdBook, USER_GUIDE, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES, Knowledge Map, task tree, CHANGES, and MEMORY are synced; focused validation passed.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
