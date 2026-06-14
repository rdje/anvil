# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `f9cf50a` - `RESOURCE-SAFE-TOOLING.2 - document ram_guard in USER_GUIDE` (the `WORKLOAD-MEMORY-SAFETY.1` design commit lands on top of this; its hash backfills next update).
- active_work_unit: `WORKLOAD-MEMORY-SAFETY` (owner directive 2026-06-14) → frontier leaf `WORKLOAD-MEMORY-SAFETY.2` (`pending`). `.1` (audit + bounded-memory design, docs-only) is `done`. Tree: `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`. All 9 roadmap phases + all prior trees remain `done`.
- next_action: implement `WORKLOAD-MEMORY-SAFETY.2` — stream the directory-output `manifest.json` so a huge `--count` no longer accumulates all per-artifact metadata in RAM (`src/main.rs:551-575` flat lane + `509-550` hierarchical lane). Must stay byte-identical to today's `manifest.json` (incremental same-bytes JSON-array write); snapshots + `tests/book_examples.rs` byte-identical. Code change ⇒ owned by leaf `.2`.
- design fixed (`.1`): all 3 mechanisms default-off / byte-identical; bounding is construction-time / decline-to-start, never truncation. `.3` makes `max_nodes_per_module` a real cap with sentinel `0 = unlimited` default (it is a ghost knob today: declared `config.rs:337`, default `1000` `config.rs:729`, enforced nowhere). `.4` opt-in internal RAM/RSS self-governor. `.5` closeout.
- in_flight_uncommitted: `WORKLOAD-MEMORY-SAFETY.1` docs (tree file + TASK_TREE row + DEVELOPMENT_NOTES + CHANGES + this file) staged for the `.1` commit.
- blockers: no active blocker. Full `cargo test` remains resource-risky on this host; monitor RAM and stop above 90% (>95% reboots). For code leaves `.2`–`.4`, prefer incremental `cargo check` / focused `cargo test <name>`; wrap any heavy build in `scripts/ram_guard.sh --`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
