# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.7` commit (hash backfills next update; prior: `b6f02ea` = `.6` prompts; `381ec01` = `.5.3` minimize tool; `65db6c3` = `.5.2` validate tool). Working tree clean after this commit.
- active_work_unit: **none — `AGENT-INTROSPECTION-MCP` CLOSED `2026-06-15`** (all leaves `.1`–`.7` done). **`.7`** = the user-facing closeout: new mdBook chapter `book/src/agent-mcp.md` (Reference, in `SUMMARY.md`) documenting the whole lane (`--introspect`, the `anvil-mcp` tools/resources/prompts, the bug-hunting loop, guardrails) with real captured examples; `USER_GUIDE.md` "Agent introspection and the MCP server" section; `README.md` CLI-truth + key-paths sync. `mdbook build` clean; `book_examples` gate 3/3 (runnable `cargo run --release -- --seed 42 --introspect` block proven; two MCP-setup blocks skip-sentinelled). Pure-docs — snapshots remain 6/6 from `.6`. **All 9 roadmap phases + EVERY task tree are now `done`; no active trees remain (full exhaustion).**
- next_action: **No open task-trees.** PNT is at full exhaustion — all roadmap phases and all post-phase trees are closed. A future activity needs a new task-tree (per doctrine, code changes require a leaf to own them first) or owner direction. Optional, owner-gated future breadth for the agent lane: expose `coverage_gaps` as an MCP tool, an HTTP transport, or the microdesign/frontend lanes over MCP — none reopen the closed tree.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` (ex-`tool_matrix`) invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: `origin/main` is at `381ec01` (`.5.3`); **2 commits ahead after this `.7` commit (`.6`,`.7`) — under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
