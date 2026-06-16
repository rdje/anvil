# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.4b.2` commit (`flop_reset_provenance` MCP surface + schema `1.6`; SOURCE+docs, DUT byte-identical; closes `.4b`/`.4`; hash backfills next update; prior: `3ee0417` = `.4b.1`; `2b0de36` = `.4a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **13 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION` tree stays `active`, NO active frontier.** THREE derived queries delivered end-to-end: `output_support` (`.1`/`.2`) + `input_reach` (`.3`) + `flop_reset_provenance` (`.4` = `.4a`/`.4b.1`/`.4b.2`), introspection schema `1.6`. Last named kind `module_reachability` = open-ended `.5+` (none retired, not a blocker). Index: `docs/TASK_TREE.md`.
- `.4b.2` delivered (DUT byte-identical): `analyze::supported_query_kinds()` now has all 3 kinds; `src/mcp/mod.rs` `run_analyze` branches to `module/design_flop_provenance` + the `flop_provenance` `-32602` guard; `analyze_schema` enum + tool/instructions text; `SCHEMA_VERSION` `1.5→1.6` + 6 `"1.5"→"1.6"` test asserts. Docs: schema-doc §6.7 + `1.5→1.6` changelog; book agent-mcp (worked example, 3 JSON examples bumped); USER_GUIDE + README; new KM card `semantic-introspection-flop-reset-provenance` (+ cross-link, KM regenerated); CODEBASE_ANALYSIS + ROADMAP. Validation: `cargo test --lib` 450/0/2 (2 new MCP proofs); snapshots 6/6 byte-identical; clippy/fmt clean; mdbook + book_examples 3/3; e2e `anvil-mcp` smoke (seed 3 → schema 1.6, 31 flops; unknown flop:99999 → -32602).
- next_action: **PNT — the decidable continuation is `SEMANTIC-INTROSPECTION-EXPANSION.5` = `module_reachability`** (the last named query kind in decision `0011`: which modules in a design are reachable from the top via the instance graph). Register `.5`/`.5a`(design)/`.5b`(impl, pre-split `.5b.1`/`.5b.2` per the `.3b`/`.4b` precedent); a pure projection of `Design.modules` + the instance edges (a fourth `DerivedAnalysis` vec, schema `1.6 → 1.7`); same SCHEMA-DERIVED / default-off / DUT-byte-identical contract; registry+dispatch land together. Pattern is fully established by `.3`/`.4` — follow it. **API-audience steering (`feedback_api_for_agents_not_humans`):** machine-friendly completeness within the SCHEMA-DERIVED ceiling. **Fresh session RECOMMENDED — clean lane boundary, repo handoff-ready** (tree clean, all gates green); long session, a fresh start keeps signoff-level quality. Other lanes: `SIGNOFF-AUTOMATION-EXPANSION`/`IDENTITY-DEEPENING` `active` no-frontier; `STRUCTURED-EMISSION-EXPANSION` `proposed` (owner-gated).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
