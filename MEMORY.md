# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `7097cf3` - `CONE-DECOMPOSITION.6 - extract cone/flops.rs` (the `CONE-DECOMPOSITION.7` motifs+close commit lands on top; its hash backfills next update).
- active_work_unit: `CONE-DECOMPOSITION` **CLOSED** (`.1`–`.7` done; `src/gen/cone.rs` 5551→2446, 56% reduction; root = recursion strategy + 6 cohesive `cone/` submodules — semantic/motifs/terminals/flops/primitives/snapshot — re-exported via `pub(crate) use <sub>::*`; every leaf byte-identical, full suite green at `.2`+`.7`). Tree: `docs/tasks/CONE-DECOMPOSITION.md`. Extraction recipe + gotchas recorded durably in `DEVELOPMENT_NOTES.md`. All 9 roadmap phases + all prior trees remain `done`.
- next_action: resume `WORKLOAD-MEMORY-SAFETY.4` (deferred behind the decomposition) — add an opt-in internal RAM/RSS self-governor so one pathological module/design can't OOM/reboot the host. Suggested `--max-rss-mb` (per-process RSS) and/or `--ram-abort-pct` (host %-used; reuse `scripts/ram_guard.sh`'s macOS `memory_pressure` / Linux `/proc/meminfo` reads); sample at safe checkpoints (between modules in the `--out` loop in `src/main.rs`; optionally at cone worklist-drain boundaries) and abort with a deterministic non-zero exit + a message naming seed + effective knobs BEFORE the danger zone. Default unset ⇒ no sampling ⇒ byte-identical. Then `.5` closeout (USER_GUIDE + book + roadmap sync, deferred boundaries, close WMS tree). Code change ⇒ owned by leaf `.4`.
- WMS done: `.1` design, `.2` streaming manifest (peak metadata RAM O(--count)→O(1), byte-identical), `.3` real per-module node budget (`max_nodes_per_module` sentinel-0-unlimited default, byte-identical). Tree: `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`.
- in_flight_uncommitted: `CONE-DECOMPOSITION.7` (src/gen/cone/motifs.rs + cone.rs + flops.rs doc fix + CODEBASE_ANALYSIS + tree + TASK_TREE + CHANGES + this file) staged for the `.7` commit.
- blockers: no active blocker. Full `cargo test` runs OK under `scripts/ram_guard.sh --threshold 88` (RAM comfortable; ~17-19 min). Monitor RAM; stop above 90% (>95% reboots). Push cadence: 30 commits (≈10 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
