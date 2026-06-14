# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `LIVE-DOC-CODEBASE-ALIGNMENT.1` live-doc commit (hash backfills next update; prior: `00e1317` = `WORKLOAD-MEMORY-SAFETY.5` resume-pointer refresh; `9e906f7` = `.5` closeout). Working tree clean after this commit.
- active_work_unit: **none** — every task tree in `docs/TASK_TREE.md` is `done` (incl. the new `LIVE-DOC-CODEBASE-ALIGNMENT`, closed in this commit; verified against the index this session, not just trusted) and all 9 numbered roadmap phases are `done`. No open PNT task to pick; PNT is genuinely exhausted, not self-paused.
- this commit: docs-only live-doc sync owned by `LIVE-DOC-CODEBASE-ALIGNMENT.1` — `CODEBASE_ANALYSIS.md` module map gained the 5 omitted real modules (`ir/param.rs`, `ir/aggregate.rs`, `frontend/`, `umbrella/`, `diff_sim/`) and the Snapshot test count was corrected 3→6. Surfaced by the session-bootstrap deep-dive; no other code/doc/book drift found.
- next_action: no active tree remains. New work needs an owner directive; any code change requires a task-tree leaf to own it first (doctrine). Recorded options: WMS deferred boundaries (intra-cone-worklist RSS sampling; JSONL manifest sidecar; soft node-budget feedback — `docs/tasks/WORKLOAD-MEMORY-SAFETY.md`), or a new owner-directed lane. (Owner has floated a possible MCP/introspection-API lane — not yet scoped into a tree.)
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **~23 commits ahead of `origin/main`; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
