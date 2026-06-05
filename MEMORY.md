# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `1f28eaf` - `LIVE-DOC-IDENTITY-ALIGNMENT.1 - align identity live docs`
- active_work_unit: none; `LIVE-DOC-ROADMAP-ALIGNMENT.1` is docs-only and closes with the next commit.
- next_action: commit `LIVE-DOC-ROADMAP-ALIGNMENT.1`, clear `git_message_brief.txt`, then pick the next task-tree-owned roadmap slice.
- in_flight_uncommitted: roadmap follow-up status alignment edits for `ROADMAP.md`, `CODEBASE_ANALYSIS.md`, task-tree registry, `MEMORY.md`, and `CHANGES.md`.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
