# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: `b825022` - `WORKLOAD-MEMORY-SAFETY.4 - internal RAM/RSS self-governor` — the `.5` closeout commit lands on top of this (hash backfilled next slice).
- active_work_unit: **none** — `WORKLOAD-MEMORY-SAFETY` is now CLOSED (`.1`–`.5` all `done`), and it was the only `active` tree. All 9 numbered roadmap phases + every task tree in `docs/TASK_TREE.md` are `done`. There is no open PNT task to pick.
- WMS delivered (tree closed): `.1` design; `.2` streaming manifest (peak metadata RAM O(--count)→O(1)); `.3` per-module node budget (`max_nodes_per_module`, sentinel-0-unlimited); `.4` opt-in process RAM/RSS governor — `src/mem_guard.rs`, `--max-rss-mb` / `--ram-abort-pct` (sentinel 0=off, `serde(default)`, validate rejects pct>100), checked BETWEEN units in both `--out` closures, clean abort exit **99** with seed+knobs message, RSS-before-host, best-effort `/proc` + `ps`/`memory_pressure`; `.5` closeout (book architecture.md resource-envelope narrative + deferred boundaries). All default-off ⇒ byte-identical (snapshots 6/6).
- next_action: no active tree remains. If the owner wants more work, options are the recorded WMS deferred boundaries (intra-cone-worklist RSS sampling; JSONL manifest sidecar; soft node-budget feedback — see `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`) or a new owner-directed lane. Any code change still requires a task-tree leaf to own it first (doctrine). Also: **push is due** (see blockers).
- in_flight_uncommitted: the `.5` docs-only closeout is staged-ready in the working tree. If a fresh session sees it uncommitted, run the COMMIT.md workflow with subject `WORKLOAD-MEMORY-SAFETY.5 - closeout: safe-envelope narrative + close tree`.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). **Push cadence: ≈12 commits since last push at `f9cf50a`; push at ~30 — getting close, consider pushing soon.**

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
