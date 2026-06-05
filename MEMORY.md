# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `72a05dc` - `MEMORY-IDENTITY-BOUNDARY.1 - keep memories instance-local`
- active_work_unit: `HIERARCHY-IDENTITY-BOUNDARY` -> frontier leaf `HIERARCHY-IDENTITY-BOUNDARY.1` (`done`; this commit closes the tree)
- next_action: commit `HIERARCHY-IDENTITY-BOUNDARY.1`, clear `git_message_brief.txt`, then pick the next roadmap-aligned task-tree slice.
- in_flight_uncommitted: implemented and verified structural-only hierarchy module-dedup boundary plus synced live docs/book/Knowledge Map for `HIERARCHY-IDENTITY-BOUNDARY.1`.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
