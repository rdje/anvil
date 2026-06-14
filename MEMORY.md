# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `9325444` - `RESOURCE-SAFE-TOOLING.1 - add RAM watchdog runner` (the `AGGREGATE-ARRAY-PACKING.1` commit lands on top; its hash backfills next update).
- active_work_unit: `AGGREGATE-ARRAY-PACKING` (new capability) — `.1` done (`AggregateKind::ArrayPacked` variant); frontier `.2` (emitter renders packed-array typedef + `port[i]` aliases) → `.3` (`aggregate_array_prob` knob + uniform-width selection) → `.4` (matrix proof) → `.5` (book/docs+close). Also open: `RESOURCE-SAFE-TOOLING.2` (ram-guard docs). All 9 phases + other trees remain `done`.
- next_action: implement `AGGREGATE-ARRAY-PACKING.2` (emitter ArrayPacked rendering; hand-built-layout test; StructPacked output byte-identical). Sync book/roadmap/live-docs in the SAME slice. Run heavy builds via `scripts/ram_guard.sh --`. Owner directive (2026-06-14): ANVIL's own runs must never crash/reboot the host — handle huge workloads via bounded/chunked/guarded execution (candidate future tree: ANVIL workload memory-safety).
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
