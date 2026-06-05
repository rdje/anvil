# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `50746ef` - `SEQUENTIAL-COINDUCTIVE-IDENTITY.1 - inventory proof envelope`
- active_work_unit: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1` is implemented/validated; flop identity now keys on `Module::flop_domain`, and explicit `flop_domains` entries remap during merge/compaction.
- next_action: commit `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`, clear `git_message_brief.txt`, then implement `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2` exact self-hold coinductive merge.
- in_flight_uncommitted: source, tests, mdBook, USER_GUIDE, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES, Knowledge Map, task tree, CHANGES, and MEMORY are synced; focused validation passed.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
