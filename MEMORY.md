# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.6` commit (hash backfills next update; prior: `381ec01` = `.5.3` minimize tool; `65db6c3` = `.5.2` validate tool; `64f0bbe` = `.5.1` downstream surface). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`). `.1`–`.5` done (`.5` container closed). **`.6` done** = the five agent-workflow prompts shipped as first-class **MCP prompts** in `src/mcp/`: `initialize` advertises the `prompts` capability; `prompts/list` + `prompts/get` over a fixed `PROMPTS` registry (`PromptSpec { name, description, args, render: PromptRender }`). Pure renderers instantiate each ordered tool chain (`find_downstream_bug`, `close_coverage_gap` (req `target`), `minimize_reproducer` (req `seed`), `triage_tool_failures`, `explain_artifact`) with sample-arg substitution (`seed`/`tools`-as-JSON-array/`yosys_mode`/`target`); `prompts/get` validates name + string-arg type + required args → clean `-32602`. Prompts add **no** new capability and **no** new truth (read-mostly doctrine). Frontier leaf **`.7`** (book/USER_GUIDE/README/CODEBASE_ANALYSIS closeout). All 9 roadmap phases + every other tree remain `done`.
- next_action: **PNT `.7`** — the user-facing closeout: mdBook chapter + USER_GUIDE section + README CLI surface + CODEBASE_ANALYSIS final sync for the full agent-introspection/MCP lane (introspection schema, the `anvil-mcp` tools/resources/prompts, the bug-hunting loop), then close the tree. Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` (ex-`tool_matrix`) invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: `origin/main` is at `381ec01` (`.5.3`); **1 commit ahead after this `.6` commit — well under the ~30 threshold, no push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
