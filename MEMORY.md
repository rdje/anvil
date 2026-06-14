# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.4` MCP-server commit (hash backfills next update; prior: `aec51e2` = `.3` emission surface; `defc196` = `.2` schema). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`). Owner **accepted** the `.1`/`.2` design (`2026-06-14`) → code leaves unblocked. `.1` (decision `0004`), `.2` (schema `docs/AGENT_INTROSPECTION_SCHEMA.md`), `.3` (`src/introspect/` + `--introspect`), `.4` (`src/mcp/` + `anvil-mcp` bin, stdio JSON-RPC; generate/introspect/dump_config + resources; no external-tool exec) are `done`. Frontier leaf **`.5`** (controlled validate/minimize). All 9 roadmap phases + every other tree remain `done`.
- next_action: **PNT `.5`** — controlled `validate` + `minimize` MCP tools: external tools (Verilator/Yosys/iverilog) **only** via existing hardened `tool_matrix` invocations, sandboxed to project-root/tmp + ram-guarded (`scripts/ram_guard.sh` + `--max-rss-mb`/`--ram-abort-pct`); `minimize` shrinks `(seed, knobs)` to a smaller failing reproducer; audit log + reproducible command line per call; no arbitrary shell. Then `.6` prompts → `.7` book/USER_GUIDE/README closeout (user-facing docs deferred to `.7` by design). Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection) not re-implemented — invariant SCHEMA-DERIVED, zero new computed truth; no stateful simulator-style session; AI agent never a signoff oracle; default `--artifact dut` stays byte-identical; reuse `tool_matrix`/`diff_sim`/`metrics`/`ram_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **~27 commits ahead of `origin/main`; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
