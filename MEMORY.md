# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-MCP-EXPANSION.5` commit (hash backfills next update; prior: `ce395de` = `.4b`; `6101c15` = `.4a`; `bc70aee` = `.4` handoff; `54ccb25` = `.3b`; `edd716d` = `.2`). Working tree clean after this commit.
- active_work_unit: **none open** — `AGENT-MCP-EXPANSION` (Lane 2) is **CLOSED** (all leaves `.1`–`.5` done). Next per lane order **`2 → 3 → 1`**: promote **`SIGNOFF-AUTOMATION-EXPANSION`** (Lane 3, `proposed`→`active`) and start its **`.1` design** leaf, then **`IDENTITY-DEEPENING`** (Lane 1, `proposed`). Index: `docs/TASK_TREE.md`.
- next_action: PNT into **`SIGNOFF-AUTOMATION-EXPANSION`**. Read `docs/tasks/SIGNOFF-AUTOMATION-EXPANSION.md`, promote the tree `proposed`→`active`, and execute its frontier `.1` design leaf (a design/decision leaf — scope the downstream signoff-automation breadth, record a decision if a durable fact emerges; no source change). Then continue PNT through that tree to exhaustion, then `IDENTITY-DEEPENING`. (Continuous PNT, no self-pause while trees remain — `feedback_no_self_pause_until_trees_closed`.)
- lane invariants (0004/0005): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`. `anvil-mcp` now also serves an opt-in loopback-default `--http <addr>` transport over the SAME `McpServer::handle_line` (NO new Cargo dependency).
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For code leaves run fmt/check/clippy + focused tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **11 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`,`AGENT-MCP-EXPANSION.1`,`.2`,`.3a`,`.3b`,`.4`-handoff,`.4a`,`.4b`,`.5`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
