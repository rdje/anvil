# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `3407f27` - `SEQUENTIAL-COINDUCTIVE-IDENTITY.3 - close sequential frontier`
- active_work_unit: `MEMORY-STATE-IDENTITY.1` is docs/probe design work; reset-all memory template is Verilator-clean but Yosys warns and lowers to registers, so current memory-inference lane remains reset-less and instance-local.
- next_action: commit `MEMORY-STATE-IDENTITY.1`, clear `git_message_brief.txt`, then run `MEMORY-STATE-IDENTITY.2` to record/defer the blocker.
- in_flight_uncommitted: task tree, mdBook, CODEBASE_ANALYSIS, DEVELOPMENT_NOTES, Knowledge Map, CHANGES, and MEMORY are synced; docs/probe validation passed.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
