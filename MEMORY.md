# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `576550a` - `LIVE-DOC-ROADMAP-ALIGNMENT.1 - align follow-up status`
- active_work_unit: `HIERARCHY-DEDUP-PRUNE` -> frontier leaf `HIERARCHY-DEDUP-PRUNE.1` (`done`; this commit closes the tree)
- next_action: commit `HIERARCHY-DEDUP-PRUNE.1`, clear `git_message_brief.txt`, then pick the next roadmap-aligned task-tree slice.
- in_flight_uncommitted: implemented and verified post-dedup unreachable-module pruning plus synced live docs/book/Knowledge Map for `HIERARCHY-DEDUP-PRUNE.1`.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
