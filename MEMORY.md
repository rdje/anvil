# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.1` design commit (hash backfills next update; prior: `5cd6f56` = `LIVE-DOC-CODEBASE-ALIGNMENT.1`; `00e1317` = WMS.5 refresh). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (new owner-directed lane; `active`) → frontier leaf `.2` (introspection schema spec, **docs**, `pending`). `.1` (design + decision record `0004`) is `done`. All 9 roadmap phases + every other tree remain `done`.
- next_action: **design-first** — pick `AGENT-INTROSPECTION-MCP.2` (specify the versioned introspection JSON schema, derived strictly from existing `metrics`/`manifest`/`config`; docs-only). Implementation leaves `.3`+ are **code** and gated on owner acceptance of the `.1`/`.2` design (do not start code without that). Architecture/guardrails: `docs/decisions/0004-agent-introspection-mcp-lane.md`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived not re-implemented; no stateful simulator-style session; AI agent never a signoff oracle; default `--artifact dut` stays byte-identical; reuse `tool_matrix`/`diff_sim`/`metrics`/`ram_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **~24 commits ahead of `origin/main`; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
