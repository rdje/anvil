# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `7e1f12e` - `AGGREGATE-ARRAY-PACKING.1 - add ArrayPacked variant` (the `.2` emitter commit lands on top; its hash backfills next update).
- active_work_unit: `AGGREGATE-ARRAY-PACKING` (new capability) — `.1`+`.2` done (variant + emitter ArrayPacked rendering, StructPacked byte-identical); frontier `.3` (`aggregate_array_prob` knob + uniform-width selection in annotate + call-site roll) → `.4` (matrix proof + coverage fact) → `.5` (book/docs+close). Also open: `RESOURCE-SAFE-TOOLING.2` (ram-guard docs). All 9 phases + other trees remain `done`.
- next_action: implement `AGGREGATE-ARRAY-PACKING.3` — add `Config::aggregate_array_prob` (default 0.0, validated, dump-config); `annotate_aggregate` gains `prefer_array` (uniform-width → ArrayPacked, else StructPacked); `gen/mod.rs` adds the second seeded roll. Default 0.0 byte-identical (snapshots + book_examples). Sync docs in the SAME slice. Heavy builds via `scripts/ram_guard.sh --`. Owner directive (2026-06-14): ANVIL's own runs must never crash/reboot the host — candidate future tree: ANVIL workload memory-safety (bounded/chunked/guarded).
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
