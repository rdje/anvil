# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `0e178a0` - `COMBINATIONAL-SEMANTIC-IDENTITY.3 - close combinational frontier`
- active_work_unit: `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` is a docs-only design inventory; it split the next code frontier into `.2.1` domain-aware flop signatures and `.2.2` exact self-hold coinductive merge.
- next_action: commit `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`, clear `git_message_brief.txt`, then implement `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`.
- in_flight_uncommitted: task tree, mdBook, USER_GUIDE, DEVELOPMENT_NOTES, CHANGES, and MEMORY are synced; docs-only validation passed.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
