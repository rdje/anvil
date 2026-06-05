# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `e39bf1c` - `SIGNOFF-SURFACE-EXPANSION.2 - add Verilator JSON frontend parity`
- active_work_unit: `SIGNOFF-SURFACE-EXPANSION.3` adds optional Icarus Verilog compile/elaboration acceptance to `tool_matrix` and static structured-gate `assign` lowering.
- next_action: commit `SIGNOFF-SURFACE-EXPANSION.3`, clear `git_message_brief.txt`, then task-tree-own/execute `SIGNOFF-SURFACE-EXPANSION.4` closeout before moving to another tree.
- in_flight_uncommitted: Icarus matrix column, emitter static structured-gate lowering, snapshot updates, mdBook/user docs, Knowledge Map, task tree, roadmap, CHANGES update ready for commit.
- blockers: full `cargo test` remains resource-risky in this environment; prior monitored run stopped at 90.7% RAM per owner policy. Focused `.3` validation is clean.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
