# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `31571a5` - `CONE-DECOMPOSITION.1 - decomposition design` (the `CONE-DECOMPOSITION.2` snapshot-extraction commit lands on top; its hash backfills next update).
- active_work_unit: `CONE-DECOMPOSITION` → frontier leaf `.3`. `.1` (design) + `.2` (extract `src/gen/cone/snapshot.rs`, byte-identical, mechanic validated by full suite) done. Tree: `docs/tasks/CONE-DECOMPOSITION.md`. `WORKLOAD-MEMORY-SAFETY` `.1`–`.3` done; its `.4`/`.5` deferred behind this. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `CONE-DECOMPOSITION.3` — extract `src/gen/cone/semantic.rs` (the value-set / unsigned-bounds / exact-value proof machinery, ~1360 lines: `width_mask`, `exact_bound`, `casez_pattern_matches`, `shift_interval_by_exact_addend`, `prove_node_exact_value[_from_bounds]`, `obvious_unsigned_compare_from_bounds`/`_result`, `exact_gate_value`, `collect_small_set`, `Small/TinyValueSet*`, all `*_value_set` fns, `node_support_size`, `can_*`, `small_value_set_min_at_least`, `node_unsigned_bounds`). Bump moved fns to `pub(crate)`; `mod semantic; pub(crate) use semantic::*;`. `crate::ir::compact` uses `cone::obvious_unsigned_compare_result` + `cone::prove_node_exact_value_from_bounds` → must stay reachable via the re-export. Submodule header: `use super::*;` + explicit `use crate::ir::{...}`. Validate byte-identical (lib + snapshots + check + clippy/fmt; full suite optional at this leaf). Code change ⇒ owned by leaf `.3`.
- decomposition plan (`.1`): root `cone.rs` keeps strategies (`build_cone_with_retry`/`build_graph_first`/`grow_pool_one_unit`/`build_outputs_interleaved`/`process_signal_frame`/`deliver`/`build_cone`), `roll_knob`, `node_budget_reached`, frame types, `FlopWorklist`, tests. `pub(crate) use <sub>::*` re-exports keep `crate::gen::cone::<symbol>` stable for `module.rs`/`hierarchy.rs`/`ir/compact.rs`. Order: snapshot(done)→semantic→primitives→terminals→flops→motifs(+closeout `.7`). GOTCHA: moved-type fields read by root tests need `pub(crate)`.
- deferred (WMS): `.4` opt-in internal RAM/RSS self-governor; `.5` closeout. `.3` made `max_nodes_per_module` real (sentinel `0`=unlimited, byte-identical).
- in_flight_uncommitted: `CONE-DECOMPOSITION.1` (tree file + TASK_TREE row + DEVELOPMENT_NOTES + CHANGES + this file) staged for the `.1` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~19 min wall). Monitor RAM; stop above 90% (>95% reboots). Push cadence: 30 commits (currently ~4 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
