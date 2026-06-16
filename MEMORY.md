# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.4b.1` commit (pure `flop_reset_provenance` core; SOURCE+docs, DUT byte-identical; hash backfills next update; prior: `2b0de36` = `.4a`; `b2e3ea7` = `.3b.2`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **12 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION.4` (`flop_reset_provenance`, the third derived query)**. Two queries already delivered (`output_support` `.1`/`.2` + `input_reach` `.3`, schema `1.5`). `.4a` design + `.4b.1` pure core **done**; frontier = **`.4b.2`** (surface). Index: `docs/TASK_TREE.md`.
- `.4b.1` delivered (DUT byte-identical): `src/introspect/analyze.rs` gained `QUERY_FLOP_RESET_PROVENANCE` + `FlopProvenance { flop, width, has_reset, reset_kind, reset_value (decimal string), default_behavior, mux_kind, mux_arms, has_d }` + the THIRD `DerivedAnalysis.flop_provenance` field (`#[serde(default, skip_serializing_if="Vec::is_empty")]` ⇒ output_support/input_reach byte-identical) + `module_flop_provenance`/`design_flop_provenance` (+ `flop_provenance_with`/`flop_provenance_of`) projecting `Module.flops` (ascending id; ResetKind→none/sync/async, FlopKind→zero/hold, FlopMux→none/one_hot/encoded). The 4 existing `DerivedAnalysis` literals gained `flop_provenance: Vec::new()`. `supported_query_kinds()` UNCHANGED (joins WITH the `run_analyze` dispatch in `.4b.2`). Validation: `cargo test --lib` 448/0/2 (20 analyze proofs, 5 new); snapshots 6/6 byte-identical; clippy/fmt clean.
- next_action: **execute `SEMANTIC-INTROSPECTION-EXPANSION.4b.2`** (the surface): in `src/introspect/analyze.rs` add `QUERY_FLOP_RESET_PROVENANCE` to `supported_query_kinds()` AND in `src/mcp/mod.rs` branch `run_analyze` by query kind (add `module_flop_provenance`/`design_flop_provenance`; the empty-result→`-32602` guard checks `flop_provenance` for this kind) in the SAME commit; bump `SCHEMA_VERSION` `1.5→1.6` in `src/introspect/mod.rs` (+ the `"1.5"` test-assertion updates — 2 introspect, 4 mcp); add `"flop_reset_provenance"` to the `analyze_schema` enum + tool/instructions text; schema-doc §6.7 + a `1.5→1.6` changelog + the row; book agent-mcp (row + worked example) + USER_GUIDE + README (schema 1.5→1.6 in ~3 spots) + a KM card. Default-off / DUT byte-identical (snapshots 6/6). **API-audience steering (`feedback_api_for_agents_not_humans`):** machine-friendly completeness within the SCHEMA-DERIVED ceiling.
- after `.4`: remaining named kind = `module_reachability` (open-ended `.5+`, none retired). Other trees (`SIGNOFF-AUTOMATION-EXPANSION`, `IDENTITY-DEEPENING`) are `active` with no current frontier; `STRUCTURED-EMISSION-EXPANSION` is `proposed` (owner-gated activation).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
