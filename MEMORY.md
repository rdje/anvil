# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `d916b08` - `AGGREGATE-ARRAY-PACKING.5 - book/docs sync + close` (the `RESOURCE-SAFE-TOOLING.2` ram-guard-docs commit lands on top; its hash backfills next update).
- active_work_unit: none. ALL task trees are `done`: `AGGREGATE-ARRAY-PACKING` closed (`.1`–`.5`; `.4b` deferred) and `RESOURCE-SAFE-TOOLING` closed (`.1` ram_guard.sh + `.2` USER_GUIDE docs). All 9 roadmap phases remain `done`. No open frontier.
- next_action: pick a NEW roadmap-aligned task tree before any `src/` change. Strongest candidate (owner directive 2026-06-14): a new "ANVIL workload memory-safety" tree so ANVIL's own runs never crash/reboot the host on huge workloads — bounded-memory generation, streamed/chunked output, internal node/RAM governors. Other deferred-boundary capabilities: reset/pulse-synchronizer CDC, Mealy FSM, parameter-aware child selection, `AGGREGATE-ARRAY-PACKING.4b` (matrix CI). NOT `UnionPacked` (aliases distinct ports — not a faithful projection). Heavy builds via `scripts/ram_guard.sh --`.
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
