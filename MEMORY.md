# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `915850f` - `CONE-DECOMPOSITION.3 - extract cone/semantic.rs` (the `CONE-DECOMPOSITION.4` primitives-extraction commit lands on top; its hash backfills next update).
- active_work_unit: `CONE-DECOMPOSITION` → frontier leaf `.5`. `.1`–`.4` done (snapshot, semantic ~1360 lines, primitives core gate makers), all byte-identical (lib 307/307 + snapshots 6/6). cone.rs 5551→4048. Tree: `docs/tasks/CONE-DECOMPOSITION.md`. `WORKLOAD-MEMORY-SAFETY` `.1`–`.3` done; its `.4`/`.5` deferred behind this. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `CONE-DECOMPOSITION.5` — extract `src/gen/cone/terminals.rs` (terminal/pool selection + gate-shape policy: `pick_terminal`, `pick_terminal_dep_bearing` [pub(super), used by module.rs+hierarchy.rs → keep reachable via re-export], `emit_terminal_constant`, `pick_datas_with_dup_cap`, `pick_signals_with_dup_rate`, `make_width_adapter` [pub(super), used externally → re-export], `pick_gate`, `pick_structured_gate`, `pick_slice_gate`, `pick_concat_operand_widths`, `input_widths_for`, `violates_anti_collapse`, `has_duplicate_operand`, `try_share`, `node_deps` [pub(super), used externally → re-export], `pick_reset_value`). Verify contiguity first (`grep -nE '^(pub\(super\) )?fn ' src/gen/cone.rs`); extract the contiguous run(s) with the recipe in DEVELOPMENT_NOTES. NOTE node_deps moving here is fine — it's still re-exported from the cone root so primitives/semantic's `use super::node_deps` keeps working. Validate byte-identical. Code change ⇒ owned by leaf `.5`.
- extraction recipe (proven on `.2`/`.3`): (1) `sed -n 'A,Bp' cone.rs > cone/<name>.rs`; (2) `perl -i -pe 's/^(fn |struct |enum )/pub(crate) $1/' cone/<name>.rs`; (3) prepend header + imports (`use super::{...}` for sibling/root calls + `use crate::ir::{...}`); (4) `perl -i -ne 'print unless $.>=A && $.<=B' cone.rs`; (5) add `mod <name>; pub(crate) use <name>::*;`; (6) `cargo fmt` + check, fix imports the compiler names; (7) lib+snapshots+clippy. GOTCHAS: moved-type fields read by root tests need `pub(crate)`; an import used only by moved code becomes unused in the root (move it, e.g. to the test module if the tests use it via `use super::*`).
- decomposition plan (`.1`): root `cone.rs` keeps strategies (`build_cone_with_retry`/`build_graph_first`/`grow_pool_one_unit`/`build_outputs_interleaved`/`process_signal_frame`/`deliver`/`build_cone`), `roll_knob`, `node_budget_reached`, frame types, `FlopWorklist`, tests. Order: snapshot(done)→semantic(done)→primitives→terminals→flops→motifs(+closeout `.7`, full suite).
- deferred (WMS): `.4` opt-in internal RAM/RSS self-governor; `.5` closeout. `.3` made `max_nodes_per_module` real (sentinel `0`=unlimited, byte-identical).
- in_flight_uncommitted: `CONE-DECOMPOSITION.1` (tree file + TASK_TREE row + DEVELOPMENT_NOTES + CHANGES + this file) staged for the `.1` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~19 min wall). Monitor RAM; stop above 90% (>95% reboots). Push cadence: 30 commits (currently ~4 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
