# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-MCP-EXPANSION.2` commit (hash backfills next update; prior: `6af0690` = `AGENT-MCP-EXPANSION.1`; `2c9d81c` = `CAPABILITY-LANE-OWNERSHIP.1`; `ac4cebf` = `AGENT-INTROSPECTION-MCP.7`). Working tree clean after this commit.
- active_work_unit: **`AGENT-MCP-EXPANSION`** (Lane 2, `active`). `.1` design/decision leaf **done** (decision `0005`); `.2` coverage_gaps pure-projection MCP tool **done**. Frontier now **`.3a`** (non-DUT introspection projection design, `pending`), then **`.3b`** (impl). `.3` is a container split into `.3a`/`.3b`. Lane order **`2 → 3 → 1`**: `AGENT-MCP-EXPANSION` → `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) → `IDENTITY-DEEPENING` (`proposed`). Index: `docs/TASK_TREE.md`.
- next_action: do **`AGENT-MCP-EXPANSION.3a`** — design leaf: decide whether each non-DUT lane (`microdesign`, `frontend`) already exposes a manifest the introspection layer can project verbatim, or whether a thin per-lane projection must be defined, keeping the introspection doc a serde projection of the existing manifest (no new computed truth). Inspect `src/introspect/mod.rs` (DUT-only today), `src/umbrella/` (`ArtifactLane`), and each lane's manifest. Record the chosen shape; NO code (design leaf). Then `.3b` routes MCP generate/introspect through the umbrella dispatch keyed by a `lane` arg (default `dut`).
- lane invariants (0004/0005): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`. Coverage-gap source = bin-private `CoverageSummary`/`compute_coverage_gaps` (`src/bin/tool_matrix.rs:286,6552`); recorded in `MatrixReport.coverage_gaps` (`:489`).
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For `.2` (a code leaf) ran fmt/check/clippy + focused `mcp::` tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **5 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`,`AGENT-MCP-EXPANSION.1`,`.2`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
