# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-MCP-EXPANSION.1` commit (hash backfills next update; prior: `2c9d81c` = `CAPABILITY-LANE-OWNERSHIP.1`; `ac4cebf` = `AGENT-INTROSPECTION-MCP.7`; `b6f02ea` = `.6`). Working tree clean after this commit.
- active_work_unit: **`AGENT-MCP-EXPANSION`** (Lane 2, `active`). `.1` design/decision leaf **done** (decision `0005`). Frontier now **`.2`** (coverage-gaps pure-projection MCP tool, `pending`), then **`.3a`** (non-DUT introspection projection design). `.3` was split into `.3a`/`.3b`. Lane order **`2 → 3 → 1`**: `AGENT-MCP-EXPANSION` → `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) → `IDENTITY-DEEPENING` (`proposed`). Index: `docs/TASK_TREE.md`.
- next_action: implement **`AGENT-MCP-EXPANSION.2`** — a **pure** MCP tool that projects a recorded `tool_matrix_report.json` (inline OR `report_path`) and returns its already-computed `coverage_gaps` + selected dark coverage facts + tool pass/fail, via a `serde_json::Value` key projection (do NOT mirror the ~150-field bin-private `CoverageSummary`). Read-only: no generation, no tool spawn, no recompute. In-process `McpServer::handle` test. This IS code ⇒ `.2` owns it. See decision `0005`.
- lane invariants (0004/0005): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`. Coverage-gap source = bin-private `CoverageSummary`/`compute_coverage_gaps` (`src/bin/tool_matrix.rs:286,6552`); recorded in `MatrixReport.coverage_gaps` (`:489`).
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: `origin/main` is at `381ec01` (`.5.3`); **4 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`,`AGENT-MCP-EXPANSION.1`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
