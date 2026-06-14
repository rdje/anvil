# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `8f7fb34` - `WORKLOAD-MEMORY-SAFETY.1 - audit + bounded-memory design` (the `WORKLOAD-MEMORY-SAFETY.2` streaming-manifest commit lands on top; its hash backfills next update).
- active_work_unit: `WORKLOAD-MEMORY-SAFETY` → frontier leaf `WORKLOAD-MEMORY-SAFETY.3` (`pending`). `.1` (design) + `.2` (stream directory-output manifest — byte-identical, peak metadata RAM O(1) in `--count`) are `done`. Tree: `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `WORKLOAD-MEMORY-SAFETY.3` — turn `max_nodes_per_module` (ghost knob: declared `config.rs:337`, default `1000` `config.rs:729`, enforced nowhere) into a real rules-first construction-time per-module node budget. Steer cone construction to terminal reuse / stop opening sub-cones near budget; NEVER truncate a finished cone. Default = sentinel `0 = unlimited` so generated RTL + SV snapshots stay byte-identical (only `--dump-config` / `manifest.json` config echo shifts). Add a `Metrics` field measuring realized node count (knob-effectiveness doctrine). Code change ⇒ owned by leaf `.3`.
- after `.3`: `.4` opt-in internal RAM/RSS self-governor (e.g. `--max-rss-mb`, abort cleanly with seed+knobs before host danger zone; default off byte-identical). `.5` closeout (USER_GUIDE + book + CODEBASE_ANALYSIS + roadmap sync, deferred boundaries, close tree).
- in_flight_uncommitted: `WORKLOAD-MEMORY-SAFETY.2` (src/manifest.rs + lib.rs + main.rs + tree + TASK_TREE + DEVELOPMENT_NOTES + CODEBASE_ANALYSIS + CHANGES + this file) staged for the `.2` commit.
- blockers: no active blocker. Full `cargo test` runs OK here under `scripts/ram_guard.sh --threshold 88` (RAM comfortable this session; debug builds ~3-6s). Still monitor RAM; stop above 90% (>95% reboots).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
