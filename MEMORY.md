# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `53bac04` - `CONE-DECOMPOSITION.7 - resume-pointer refresh (post-commit)` — the `WORKLOAD-MEMORY-SAFETY.4` commit lands on top of this (hash backfilled next slice).
- active_work_unit: `WORKLOAD-MEMORY-SAFETY` (`active`). `.1`–`.4` done; frontier `.5` (closeout). Tree: `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`. All 9 roadmap phases + every other tree (incl. `CONE-DECOMPOSITION`, closed) remain `done`.
- WMS done: `.1` design; `.2` streaming manifest (peak metadata RAM O(--count)→O(1), byte-identical); `.3` real per-module node budget (`max_nodes_per_module` sentinel-0-unlimited, byte-identical); `.4` opt-in internal RAM/RSS self-governor — new `src/mem_guard.rs`, `--max-rss-mb` / `--ram-abort-pct` (sentinel 0=off, `serde(default)`, validate rejects pct>100), checked BETWEEN units in both `--out` streaming closures (decline-to-start-more, never mid-cone), clean abort exit **99** with a seed+effective-knobs stderr message; RSS checked before host-%; best-effort `/proc` + `ps`/`memory_pressure` reads (None⇒no abort). Default-off ⇒ byte-identical (snapshots 6/6; SV diff identical off vs under-limit).
- next_action: run `WORKLOAD-MEMORY-SAFETY.5` — closeout. The mandatory `.4` CLI-surface doc sync (USER_GUIDE "Resource-safe runs", book `knobs.md`, README CLI-truth, CODEBASE_ANALYSIS module map) already landed in the `.4` commit per COMMIT.md. `.5` adds: a cohesive book "safe-envelope" narrative tying `.2`/`.3`/`.4` together; the explicit deferred boundaries (intra-cone-worklist sampling deferred; `count==1` stdout + non-DUT lanes unguarded by design); confirm ROADMAP status; then close the WMS tree. Pure-docs ⇒ no task-tree code-leaf needed, but `.5` owns it.
- in_flight_uncommitted: the `.4` slice is staged-ready in the working tree (code + all mandatory live docs). If a fresh session sees this uncommitted, run the COMMIT.md workflow with subject `WORKLOAD-MEMORY-SAFETY.4 - internal RAM/RSS self-governor`.
- blockers: no active blocker. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for any heavy build; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: 30 commits (≈11 since last push at `f9cf50a`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
