# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `08c6bc9` - `WORKLOAD-MEMORY-SAFETY.3 - real per-module node budget` (the `CONE-DECOMPOSITION.1` design commit lands on top; its hash backfills next update).
- active_work_unit: `CONE-DECOMPOSITION` (owner request 2026-06-14) → frontier leaf `.2`. `.1` (decomposition design, docs-only) done. Tree: `docs/tasks/CONE-DECOMPOSITION.md`. `WORKLOAD-MEMORY-SAFETY` `.1`+`.2`+`.3` done; its `.4`/`.5` are deferred behind this decomposition. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `CONE-DECOMPOSITION.2` — extract `src/gen/cone/snapshot.rs` (the rollback machinery: `ConstructionSnapshot`, `take_/rollback_construction_snapshot`, `prune_intern_tables_after_node_truncate`) as the warm-up that validates the mechanic. Add `mod snapshot; pub(crate) use snapshot::*;` to `cone.rs`; bump moved fns to `pub(crate)`; submodule header `use super::*;` + any extra imports. Validate byte-identical: `cargo check --all-targets` + `cargo test --lib` + `cargo test --test snapshots` + clippy/fmt + FULL `cargo test` (first-extraction milestone). Code change ⇒ owned by leaf `.2`.
- decomposition plan (`.1`): root `cone.rs` keeps strategies (`build_cone_with_retry`/`build_graph_first`/`grow_pool_one_unit`/`build_outputs_interleaved`/`process_signal_frame`/`deliver`/`build_cone`), `roll_knob`, `node_budget_reached`, frame types, `FlopWorklist`, tests. `pub(crate) use <sub>::*` re-exports keep `crate::gen::cone::<symbol>` stable for `module.rs`/`hierarchy.rs`/`ir/compact.rs`. Order: snapshot→semantic→primitives→terminals→flops→motifs(+closeout `.7`).
- deferred (WMS): `.4` opt-in internal RAM/RSS self-governor; `.5` closeout. `.3` made `max_nodes_per_module` real (sentinel `0`=unlimited, byte-identical).
- in_flight_uncommitted: `CONE-DECOMPOSITION.1` (tree file + TASK_TREE row + DEVELOPMENT_NOTES + CHANGES + this file) staged for the `.1` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~19 min wall). Monitor RAM; stop above 90% (>95% reboots). Push cadence: 30 commits (currently ~4 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
