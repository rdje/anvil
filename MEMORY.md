# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `1c5ac85` - `WORKLOAD-MEMORY-SAFETY.2 - stream the directory-output manifest` (the `WORKLOAD-MEMORY-SAFETY.3` node-budget commit lands on top; its hash backfills next update).
- active_work_unit: `WORKLOAD-MEMORY-SAFETY` `.1`+`.2`+`.3` done. **Next: open a new `CONE-DECOMPOSITION` tree (owner request 2026-06-14)** — break the 5551-line `src/gen/cone.rs` into cohesive interconnected submodules (semantic proofs / primitives / terminals+selection / flops / motifs / snapshot; strategy orchestration stays in the root), one byte-identical extraction per leaf, full tests green. Then resume `WORKLOAD-MEMORY-SAFETY.4`. All 9 roadmap phases + all prior trees remain `done`.
- next_action: create `docs/tasks/CONE-DECOMPOSITION.md` (+ TASK_TREE row), land design leaf `.1` (seam map: `cone/semantic.rs`, `cone/primitives.rs`, `cone/terminals.rs`, `cone/flops.rs`, `cone/motifs.rs`, `cone/snapshot.rs`; keep public entry points `build_cone_with_retry`/`build_outputs_interleaved`/`build_graph_first`/`build_cone`/`drain_flop_worklist`/`pick_terminal_dep_bearing`/`node_deps`/`make_width_adapter` re-exported from the cone root). Each extraction leaf: move fns to `src/gen/cone/<name>.rs`, fix visibility (pub(crate) re-exports), `cargo check --all-targets` + `cargo test --lib` + snapshots byte-identical, full suite at milestones.
- deferred (WMS): `.4` opt-in internal RAM/RSS self-governor (`--max-rss-mb`, abort cleanly with seed+knobs before host danger zone, default off byte-identical); `.5` closeout. `.3` made `max_nodes_per_module` real (sentinel `0`=unlimited default, byte-identical; `cone::node_budget_reached` OR-ed into both `force_leaf` sites + graph-first loop).
- in_flight_uncommitted: `WORKLOAD-MEMORY-SAFETY.3` (cone.rs + config.rs + book/knobs.md + tree + TASK_TREE + DEVELOPMENT_NOTES + CODEBASE_ANALYSIS + CHANGES + this file) staged for the `.3` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~19 min wall). Monitor RAM; stop above 90% (>95% reboots).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
