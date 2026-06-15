# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-MCP-EXPANSION.3a` commit (hash backfills next update; prior: `edd716d` = `AGENT-MCP-EXPANSION.2`; `6af0690` = `.1`; `2c9d81c` = `CAPABILITY-LANE-OWNERSHIP.1`). Working tree clean after this commit.
- active_work_unit: **`AGENT-MCP-EXPANSION`** (Lane 2, `active`). `.1` done (decision `0005`); `.2` coverage_gaps tool done; `.3a` non-DUT introspection projection design **done**. Frontier now **`.3b`** (impl, `pending`), then **`.4`** (HTTP transport). `.3` container = `.3a`(done)/`.3b`(pending). Lane order **`2 → 3 → 1`**: `AGENT-MCP-EXPANSION` → `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) → `IDENTITY-DEEPENING` (`proposed`). Index: `docs/TASK_TREE.md`.
- next_action: implement **`AGENT-MCP-EXPANSION.3b`** (code). Per `.3a`: (1) add a manifest-carrying introspection builder reusing `ArtifactDescriptor.manifest: Option<ResourceRef>` → `anvil://artifact/<run_id>/manifest`; (2) extend MCP `CachedArtifact` with `manifest: Option<String>` + serve that URI in `resources_read`/`resources_list`; (3) generalize `build_artifact` (`src/mcp/mod.rs`) to dispatch on a `lane` arg (default `dut`) through umbrella `MicrodesignLane`/`FrontendLane`; (4) non-DUT args carry `lane` + scoped knobs (`n_params`,`n_children`), NOT the DUT `Config`; (5) feed those knobs into `content_run_id` so non-DUT run_ids stay content-addressed. NO schema-version bump; DUT byte-identical (snapshots 6/6). In-process `McpServer::handle` tests.
- lane invariants (0004/0005): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`. Coverage-gap source = bin-private `CoverageSummary`/`compute_coverage_gaps` (`src/bin/tool_matrix.rs:286,6552`); recorded in `MatrixReport.coverage_gaps` (`:489`).
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). For `.2` (a code leaf) ran fmt/check/clippy + focused `mcp::` tests + snapshot byte-identical guard. Push cadence: `origin/main` is at `381ec01` (`.5.3`); **6 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`,`AGENT-MCP-EXPANSION.1`,`.2`,`.3a`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
