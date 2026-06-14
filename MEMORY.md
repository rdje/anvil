# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `0341624` - `AGGREGATE-ARRAY-PACKING.3 - aggregate_array_prob selection` (the `.4` metric/proof commit lands on top; its hash backfills next update).
- active_work_unit: `AGGREGATE-ARRAY-PACKING` (new capability) — `.1`–`.4` DONE: `ArrayPacked` variant + emitter + `aggregate_array_prob` selection + `num_array_packed_aggregate_modules` metric + 7/7 Verilator5.046/Yosys0.64 downstream-clean proof on array designs. Capability delivered + proven. Frontier `.5` (book/USER_GUIDE/knobs/ir.md sync + close), then optional `.4b` (matrix CI scenario+coverage fact, kept out of hard gate). Also open: `RESOURCE-SAFE-TOOLING.2` (ram-guard docs). All 9 phases + other trees remain `done`.
- next_action: implement `AGGREGATE-ARRAY-PACKING.5` — book chapter/section for packed aggregates (progressive struct→array, prose-only, copy/paste-runnable `--config` example via the hierarchy design path), `knobs.md` `aggregate_array_prob`, USER_GUIDE, ir.md aggregates subsection marks ArrayPacked delivered; `mdbook build` + `cargo test --test book_examples` clean; then close the tree. Owner directive (2026-06-14): ANVIL's own runs must never crash/reboot the host — candidate future tree: ANVIL workload memory-safety (bounded/chunked/guarded).
- in_flight_uncommitted: none.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). Prior monitored run stopped at 90.7% RAM per owner policy.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
