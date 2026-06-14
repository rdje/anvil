# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.5.1` commit (hash backfills next update; prior: `5db5ebc` = `.4` MCP server; `aec51e2` = `.3` emission surface). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`). `.1` (decision `0004`), `.2` (schema), `.3` (`src/introspect/` + `--introspect`), `.4` (`src/mcp/` + `anvil-mcp` bin) are `done`. `.5` was **split** into `.5.1`/`.5.2`/`.5.3`; **`.5.1` done** — hardened downstream-tool invocations extracted from the `tool_matrix` binary into the shared lib module `src/downstream/mod.rs` (`run_verilator`/`run_yosys`/`run_iverilog_compile` + `run_tool` + `first_tool_warning` + `ToolInvocation` + `YosysMode`), `tool_matrix` rewired to `use anvil::downstream::{…}`; behavior-preserving (snapshots 6/6, matrix tool tests pass). Frontier leaf **`.5.2`**. All 9 roadmap phases + every other tree remain `done`.
- next_action: **PNT `.5.2`** — the controlled `validate` MCP tool over the `.5.1` surface: generate `(seed, knobs)` into a sandboxed temp dir under a project-root/tmp scope, run the selected `anvil::downstream` acceptance tools, ram-guard via `mem_guard` (+ document `scripts/ram_guard.sh`), return structured `ToolInvocation` reports + overall verdict, audit-log the reproducible `(seed, knobs)` + exact argv per call; **no arbitrary shell**. Then `.5.3` minimize → `.6` prompts → `.7` book/USER_GUIDE/README closeout (user-facing docs deferred to `.7` by design). Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection) not re-implemented — invariant SCHEMA-DERIVED, zero new computed truth; `validate` runs external tools **only via the existing hardened `downstream` (ex-`tool_matrix`) invocations**, no second source of truth; AI agent never a signoff oracle; default `--artifact dut` stays byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **28 commits ahead of `origin/main` after this commit; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
