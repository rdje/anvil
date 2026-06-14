# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `CAPABILITY-LANE-OWNERSHIP.1` commit (hash backfills next update; prior: `ac4cebf` = `AGENT-INTROSPECTION-MCP.7`; `b6f02ea` = `.6`; `381ec01` = `.5.3`). Working tree clean after this commit.
- active_work_unit: **`AGENT-MCP-EXPANSION`** (Lane 2, `active`) → frontier leaf **`.1`** (design/decision, `pending`). Owner authorized 3 post-phase capability lanes on `2026-06-15` ("do these in any order"); registered via `CAPABILITY-LANE-OWNERSHIP.1` (now `done`). Agent execution order **`2 → 3 → 1`**: `AGENT-MCP-EXPANSION` → `SIGNOFF-AUTOMATION-EXPANSION` (`proposed`) → `IDENTITY-DEEPENING` (`proposed`). Index: `docs/TASK_TREE.md`.
- next_action: **owner asked for handoff readiness after registration (fresh session likely next).** When implementation resumes, do `AGENT-MCP-EXPANSION.1` — design leaf: re-confirm the 0004 lane invariants, locate the matrix-side `CoverageSummary` gap source, decide the read-only exposure path (project a recorded summary, not new computed truth), and finalize/split `.2`–`.5` (coverage-gap MCP tool / non-DUT lanes over MCP / optional HTTP transport / closeout). NO code yet this turn — `.1` is a docs/decision leaf.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit. Repo is handoff-ready: tree clean, gates green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: `origin/main` is at `381ec01` (`.5.3`); **3 commits ahead after this commit (`.6`,`.7`,`CAPABILITY-LANE-OWNERSHIP.1`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
