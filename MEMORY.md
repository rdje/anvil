# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `7ac349a` - `CONE-DECOMPOSITION.5 - extract cone/terminals.rs` (the `CONE-DECOMPOSITION.6` flops-extraction commit lands on top; its hash backfills next update).
- active_work_unit: `CONE-DECOMPOSITION` → frontier leaf `.7` (final). `.1`–`.6` done (snapshot, semantic ~1360 lines, primitives, terminals ~537 lines, flops ~270 lines), all byte-identical (lib 307/307 + snapshots 6/6). cone.rs 5551→3260. Tree: `docs/tasks/CONE-DECOMPOSITION.md`. `WORKLOAD-MEMORY-SAFETY` `.1`–`.3` done; its `.4`/`.5` deferred behind this. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `CONE-DECOMPOSITION.7` (final) — extract `src/gen/cone/motifs.rs` (linear-combination / shift / comparand / priority-encoder / comb-mux+case+casez+for-fold builders [recursive + pool-only], `make_none_selected`, `or_reduce_terms`, `is_comparison_op`, `pick_coefficient` & friends, `make_case_mux`/`make_casez_mux`/`make_for_fold`, `pick_for_fold_*`, `build_casez_patterns`) via contiguous ranges (one perl delete pass, original line numbers). The recursive block builders (`build_comb_mux`/`build_case_mux_recursive`/etc.) may stay in the root strategy or move with motifs — keep the root containing only strategy orchestration (`build_cone*`, `build_graph_first`, `grow_pool_one_unit`, `build_outputs_interleaved`, `process_signal_frame`, `deliver`, pool-only builders, `roll_knob`, `node_budget_reached`, frames, FlopWorklist, tests). Then CLOSEOUT: update CODEBASE_ANALYSIS module map (cone.rs → cone/ submodules), run FULL cargo test (closeout milestone), close tree. Code change ⇒ owned by leaf `.7`.
- extraction recipe (proven on `.2`/`.3`): (1) `sed -n 'A,Bp' cone.rs > cone/<name>.rs`; (2) `perl -i -pe 's/^(fn |struct |enum )/pub(crate) $1/' cone/<name>.rs`; (3) prepend header + imports (`use super::{...}` for sibling/root calls + `use crate::ir::{...}`); (4) `perl -i -ne 'print unless $.>=A && $.<=B' cone.rs`; (5) add `mod <name>; pub(crate) use <name>::*;`; (6) `cargo fmt` + check, fix imports the compiler names; (7) lib+snapshots+clippy. GOTCHAS: moved-type fields read by root tests need `pub(crate)`; an import used only by moved code becomes unused in the root (move it, e.g. to the test module if the tests use it via `use super::*`).
- decomposition plan (`.1`): root `cone.rs` keeps strategies (`build_cone_with_retry`/`build_graph_first`/`grow_pool_one_unit`/`build_outputs_interleaved`/`process_signal_frame`/`deliver`/`build_cone`), `roll_knob`, `node_budget_reached`, frame types, `FlopWorklist`, tests. Order: snapshot(done)→semantic(done)→primitives→terminals→flops→motifs(+closeout `.7`, full suite).
- deferred (WMS): `.4` opt-in internal RAM/RSS self-governor; `.5` closeout. `.3` made `max_nodes_per_module` real (sentinel `0`=unlimited, byte-identical).
- in_flight_uncommitted: `CONE-DECOMPOSITION.1` (tree file + TASK_TREE row + DEVELOPMENT_NOTES + CHANGES + this file) staged for the `.1` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~19 min wall). Monitor RAM; stop above 90% (>95% reboots). Push cadence: 30 commits (currently ~4 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
