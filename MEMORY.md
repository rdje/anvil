# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.2` schema-spec commit (hash backfills next update; prior: `9ac5ef3` = `AGENT-INTROSPECTION-MCP.1`; `5cd6f56` = `LIVE-DOC-CODEBASE-ALIGNMENT.1`). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`, now **design-complete**). `.1` (design + decision record `0004`) and `.2` (schema spec — `docs/AGENT_INTROSPECTION_SCHEMA.md`) are `done`. `.3` (first **code** leaf) is parked on owner acceptance, not a technical blocker. All 9 roadmap phases + every other tree remain `done`.
- next_action: **await owner acceptance of the `.1`/`.2` design** before starting any code (`.3`+ = introspection emission surface → MCP server → validate/minimize → prompts → book/USER_GUIDE closeout). No other task tree is open. If continuing without owner input, only docs/hygiene work is eligible (no code may start). Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004-agent-introspection-mcp-lane.md`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection) not re-implemented — invariant SCHEMA-DERIVED, zero new computed truth; no stateful simulator-style session; AI agent never a signoff oracle; default `--artifact dut` stays byte-identical; reuse `tool_matrix`/`diff_sim`/`metrics`/`ram_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **~25 commits ahead of `origin/main`; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
