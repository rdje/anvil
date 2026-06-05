# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `6d27a4b` - `SIGNOFF-SURFACE-EXPANSION.1 - add N-flop CDC synchronizer`
- active_work_unit: `SIGNOFF-SURFACE-EXPANSION.2` adds optional Verilator JSON frontend parity and closes the richer AST/source extractor leaf.
- next_action: commit `SIGNOFF-SURFACE-EXPANSION.2`, clear `git_message_brief.txt`, then task-tree-own `SIGNOFF-SURFACE-EXPANSION.3` before any broader simulator/tool parity or sweep code changes.
- in_flight_uncommitted: Verilator JSON extractor/tests, optional real-tool gate, mdBook/user docs, Knowledge Map, task tree, roadmap, CHANGES update ready for commit.
- blockers: full `cargo test` remains resource-risky in this environment; prior monitored run stopped at 90.7% RAM per owner policy. Focused `.2` validation is clean.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
