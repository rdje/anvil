# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `a1f5729` - `COMBINATIONAL-SEMANTIC-IDENTITY.2 - widen semantic proof budget safely`
- active_work_unit: `COMBINATIONAL-SEMANTIC-IDENTITY.3` is implemented/validated; this commit closes the tree with an empty frontier.
- next_action: commit `COMBINATIONAL-SEMANTIC-IDENTITY.3`, clear `git_message_brief.txt`, then continue at the next active follow-up tree (`SEQUENTIAL-COINDUCTIVE-IDENTITY.1`).
- in_flight_uncommitted: combinational semantic identity closeout is docs-only; task tree, roadmap, CHANGES, and MEMORY are synced.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
