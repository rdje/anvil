# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.5.2` commit (hash backfills next update; prior: `64f0bbe` = `.5.1` downstream surface; `5db5ebc` = `.4` MCP server). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`). `.1`–`.4` done. `.5` split into `.5.1`/`.5.2`/`.5.3`; **`.5.1` + `.5.2` done**. `.5.1` extracted the hardened downstream-tool invocations into `src/downstream/`. **`.5.2`** = controlled `validate`: `downstream::validate(seed, &Config, &ValidateOptions) -> ValidateReport` (regenerate → fresh per-run sandbox `<tmp>/anvil-validate-<run_id>/` → run the `AcceptanceTool` allow-list `verilator`/`yosys`/`iverilog` via the `.5.1` runners → `MemGuard` decline-before-spawn → verdict) + MCP `validate` tool + `anvil://audit/log` resource; `introspect::content_run_id` now `pub`; `MemGuard::from_limits` added. e2e clean vs real Verilator+Yosys; snapshots 6/6. Frontier leaf **`.5.3`**. All 9 roadmap phases + every other tree remain `done`.
- next_action: **PNT `.5.3`** — the `minimize` tool: deterministic, bounded delta-debug of `(seed, knobs)` toward a smaller failing reproducer, using `.5.2`'s `downstream::validate` as a pure failure oracle (shrink knobs/structure that still reproduce a downstream-tool failure); audit-logged; build on `downstream`. Then `.6` prompts → `.7` book/USER_GUIDE/README closeout (user-facing docs deferred to `.7` by design). Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` (ex-`tool_matrix`) invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **29 commits ahead of `origin/main` after this commit; below the ~30 threshold → hold, do not push yet** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
