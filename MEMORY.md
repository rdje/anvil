# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `8dc0e6d` - `ENDPOINT-IDENTITY-BOUNDARY.1 - preserve semantic endpoints`
- active_work_unit: `ROADMAP-FOLLOWUP-OWNERSHIP.1` is the docs/workflow registration leaf; first implementation frontier after commit is `COMBINATIONAL-SEMANTIC-IDENTITY.1`.
- next_action: commit `ROADMAP-FOLLOWUP-OWNERSHIP.1`, clear `git_message_brief.txt`, then implement `COMBINATIONAL-SEMANTIC-IDENTITY.1` before any source edit.
- in_flight_uncommitted: registered ownership for combinational semantic identity, sequential/coinductive identity, memory-state identity, hierarchy semantic identity, and signoff-surface expansion; synced `ROADMAP.md`, `docs/TASK_TREE.md`, `CODEBASE_ANALYSIS.md`, and mdBook architecture status.
- blockers: none.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
