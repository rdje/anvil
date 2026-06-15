# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-MCP-EXPANSION.3b` commit (hash backfills next update; prior: `bfab27e` = `AGENT-MCP-EXPANSION.3a`; `edd716d` = `.2`; `6af0690` = `.1`). Working tree clean after this commit.
- active_work_unit: **`AGENT-MCP-EXPANSION`** (Lane 2, `active`). `.1` (decision `0005`), `.2` (coverage_gaps tool), `.3` (`.3a` design + `.3b` non-DUT lanes over MCP) all **done**. Frontier now **`.4`** (optional HTTP transport, `pending`), then **`.5`** (closeout: book/USER_GUIDE/README sync). Lane order **`2 → 3 → 1`**: `AGENT-MCP-EXPANSION` → `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) → `IDENTITY-DEEPENING` (`proposed`). Index: `docs/TASK_TREE.md`.
- next_action: do **`AGENT-MCP-EXPANSION.4`** (code) — optional HTTP transport for `anvil-mcp` beside stdio. **OWNER DECISION (2026-06-15): hand-rolled minimal HTTP/1.1 over `std::net::TcpListener` — NO new crate dependency** (keep the dependency-light MCP design); `--http <addr>` flag on the existing bin, **loopback-only default**; stdio stays default; same `McpServer::handle` dispatcher; per-call `downstream` guardrails unchanged; transport-level test (listener on 127.0.0.1:0, round-trip). Consider a `.4a` design leaf to pin HTTP framing first. Then `.5` closeout syncs `book/src/agent-mcp.md` + `USER_GUIDE.md` + `README.md` for the whole expanded surface (coverage_gaps + non-DUT lanes + HTTP). **Owner requested a FRESH SESSION for the `.4` network leaf.**
- lane invariants (0004/0005): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`. Coverage-gap source = bin-private `CoverageSummary`/`compute_coverage_gaps` (`src/bin/tool_matrix.rs:286,6552`); recorded in `MatrixReport.coverage_gaps` (`:489`).
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For code leaves (`.2`,`.3b`) ran fmt/check/clippy + focused `mcp::`/`introspect::` tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **7 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`,`AGENT-MCP-EXPANSION.1`,`.2`,`.3a`,`.3b`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
