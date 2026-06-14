# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `e5bb86b` - `AGGREGATE-ARRAY-PACKING.2 - emit ArrayPacked` (the `.3` selection commit lands on top; its hash backfills next update).
- active_work_unit: `AGGREGATE-ARRAY-PACKING` (new capability) — `.1`+`.2`+`.3` done (variant + emitter + `aggregate_array_prob` selection wired end-to-end, default-off byte-identical); frontier `.4` (tool_matrix scenario + `saw_array_packed_aggregate_design` coverage fact + downstream-clean proof) → `.5` (book/USER_GUIDE/knobs/ir.md sync + close). Also open: `RESOURCE-SAFE-TOOLING.2` (ram-guard docs). All 9 phases + other trees remain `done`.
- next_action: implement `AGGREGATE-ARRAY-PACKING.4` — add a `phase5b_array_aggregate` (uniform width via min_width==max_width, aggregate_array_prob=1.0) scenario + `saw_array_packed_aggregate_design` coverage fact in `src/bin/tool_matrix.rs` + metrics, then prove Verilator+both-Yosys clean via a focused smoke under `scripts/ram_guard.sh`. Sync docs in the SAME slice. Owner directive (2026-06-14): ANVIL's own runs must never crash/reboot the host — candidate future tree: ANVIL workload memory-safety (bounded/chunked/guarded).
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
