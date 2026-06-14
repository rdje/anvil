# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `AGENT-INTROSPECTION-MCP.5.3` commit (hash backfills next update; prior: `65db6c3` = `.5.2` validate tool; `64f0bbe` = `.5.1` downstream surface). Working tree clean after this commit.
- active_work_unit: **`AGENT-INTROSPECTION-MCP`** (`active`). `.1`–`.4` done. `.5` split into `.5.1`/`.5.2`/`.5.3`; **all three done → `.5` container closed**. **`.5.3`** = controlled `minimize`: `downstream::minimize(seed, &Config, &MinimizeOptions) -> MinimizeReport` — deterministic coordinate-descent (`search_minimal`, generic over the oracle) that bisects integer size bounds toward their floors and drives optional-motif probs to `0.0`, to a fixpoint, using `.5.2`'s `validate` as a pure failure oracle; **seed held fixed**; every candidate re-checked with `Config::validate` before the generator; hard-capped by `max_oracle_calls` (default 200); `final_validation` captured from the last failing oracle call (no extra run). MCP `minimize` tool reuses the new shared `parse_validate_tools`/`parse_yosys_mode_arg` helpers + audit-logs each call. Frontier leaf **`.6`** (agent-workflow prompts). All 9 roadmap phases + every other tree remain `done`.
- next_action: **PNT `.6`** — package the agent-workflow prompts (`find_downstream_bug`, `close_coverage_gap`, `minimize_reproducer`, `triage_tool_failures`, `explain_artifact`), each driving its tool chain end-to-end on a sample. Then `.7` book/USER_GUIDE/README closeout (user-facing docs deferred to `.7` by design). Contract: `docs/AGENT_INTROSPECTION_SCHEMA.md`; architecture: `docs/decisions/0004`.
- lane invariants (0004): MCP is a thin read-mostly adapter beside the deterministic core; schema derived (serde projection), invariant SCHEMA-DERIVED; `validate`/`minimize` run external tools **only via the hardened `downstream` (ex-`tool_matrix`) invocations** (fixed allow-list, fixed binary names, sandboxed temp dir, no arbitrary shell/path), no second source of truth; AI agent never a signoff oracle; `minimize` searches the **input** `(seed, knobs)` space (never mutates/repairs RTL); default `--artifact dut` byte-identical; reuse `tool_matrix`/`downstream`/`diff_sim`/`metrics`/`mem_guard`.
- in_flight_uncommitted: none after this commit.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 88` for heavy builds; full `cargo test` not required per owner resource policy (monitor RAM; stop above 90%, >95% reboots). Push cadence: **~30 commits ahead of `origin/main` after this commit; at the ~30 threshold → push after this slice's commit workflow completes** (`feedback_push_cadence`).

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
