# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `367dca1` - `HIERARCHY-SEMANTIC-IDENTITY.3 - close hierarchy semantic frontier`
- active_work_unit: `SIGNOFF-SURFACE-EXPANSION.1` adds configurable N-flop 1-bit CDC synchronizer chains and closes the first signoff-surface leaf.
- next_action: commit `SIGNOFF-SURFACE-EXPANSION.1`, clear `git_message_brief.txt`, then pick `SIGNOFF-SURFACE-EXPANSION.2` for richer AST/source extractor parity.
- in_flight_uncommitted: N-flop CDC code, metrics, matrix coverage, mdBook/user docs, Knowledge Map, task tree, roadmap, CHANGES update ready for commit.
- blockers: full `cargo test` is resource-blocked in this environment; monitored run stopped at 90.7% RAM per owner policy. Focused validation and 17-scenario matrix smoke are clean.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
