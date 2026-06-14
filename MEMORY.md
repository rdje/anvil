# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `70636e8` - `AGGREGATE-ARRAY-PACKING.4 - array metric + downstream-clean proof` (the `.5` book/close commit lands on top; its hash backfills next update).
- active_work_unit: none. `AGGREGATE-ARRAY-PACKING` is CLOSED (`.1`–`.5` done: `ArrayPacked` variant + emitter + `aggregate_array_prob` selection + metric + 7/7 Verilator5.046/Yosys0.64 downstream-clean + book/docs sync; `.4b` matrix CI instrumentation `deferred`, default-off byte-identical). Still open: `RESOURCE-SAFE-TOOLING.2` (ram-guard USER_GUIDE docs). All 9 phases + every other tree remain `done`.
- next_action: pick the next roadmap-aligned task tree. Candidates: `RESOURCE-SAFE-TOOLING.2` (document ram_guard in USER_GUIDE); a new "ANVIL workload memory-safety" tree (owner directive 2026-06-14: ANVIL's own runs must never crash/reboot the host — bounded/chunked/guarded huge workloads); or another deferred-boundary capability (UnionPacked is NOT a faithful projection — skip; reset/pulse-synchronizer CDC; Mealy FSM; parameter-aware child selection). Heavy builds via `scripts/ram_guard.sh --`.
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
