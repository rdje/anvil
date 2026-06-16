# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` commit (`input_reach` MCP surface + schema `1.5`; SOURCE+docs, DUT byte-identical; closes `.3b`/`.3`; hash backfills next update; prior: `42f3ea9` = `.3b.1`; `05527b2` = `.3a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **10 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION` tree stays `active`, NO active frontier.** Two derived queries delivered end-to-end: `output_support` (`.1`/`.2`) + `input_reach` (`.3` = `.3a`/`.3b.1`/`.3b.2` all done), introspection schema `1.5`. Remaining kinds `flop_reset_provenance` / `module_reachability` = open-ended `.4+` (none retired, not a blocker). Index: `docs/TASK_TREE.md`.
- `.3b.2` delivered (DUT byte-identical): `analyze::supported_query_kinds()` = `[output_support, input_reach]`; `src/mcp/mod.rs` `run_analyze` branches by query kind (`module_input_reach`/`design_input_reach` vs support builders) + vec-aware `-32602` guard; `analyze_schema` enum + tool/instructions text; `SCHEMA_VERSION` `1.4→1.5` + 6 `"1.4"→"1.5"` test asserts; stale `introspect` "schema 1.0" desc made version-agnostic. Docs: schema-doc §6.7 + `1.4→1.5` changelog; book agent-mcp (input_reach worked example); USER_GUIDE + README; new KM card `semantic-introspection-input-reach` (+ cross-link, KM regenerated); CODEBASE_ANALYSIS + ROADMAP. Validation: `cargo test --lib` 443/0/2 (2 new MCP input_reach proofs); snapshots 6/6 byte-identical; clippy/fmt clean; mdbook + book_examples 3/3; e2e `anvil-mcp` smoke (schema 1.5, 37 reach results, unknown source → -32602).
- next_action: **PNT — pick the next lane (owner-directed order).** `SEMANTIC-INTROSPECTION-EXPANSION` has no active frontier (two queries done). Other `active` trees in `docs/TASK_TREE.md`: `SIGNOFF-AUTOMATION-EXPANSION` (no active frontier) + `IDENTITY-DEEPENING` (no current frontier; deeper memory/FSM/wrapper/retimed boundaries are open-ended). `STRUCTURED-EMISSION-EXPANSION` is `proposed`. If the owner names no lane, candidate next work = a `.4+` query kind here (`flop_reset_provenance` / `module_reachability`) OR activating `STRUCTURED-EMISSION-EXPANSION.1`. **API-audience steering (owner, `feedback_api_for_agents_not_humans`):** machine-friendly completeness within the SCHEMA-DERIVED ceiling for any introspection query.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
