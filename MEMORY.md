# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` commit (pure `input_reach` analysis core; SOURCE+docs, DUT byte-identical; hash backfills next update; prior: `05527b2` = `.3a`; `e635dd1` = `.3` handoff). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **9 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION.3` (`input_reach`, the dual fan-OUT of `output_support`)**. `.3a` design + `.3b.1` pure core **done**; frontier = **`.3b.2`** (surface wiring). Index: `docs/TASK_TREE.md`.
- `.3b.1` delivered (DUT byte-identical): `src/introspect/analyze.rs` gained `QUERY_INPUT_REACH` + `ReachResult { target, reaches_outputs[], reaches_flops[], fanout_targets }` + the SECOND `DerivedAnalysis.reach_results` field (`#[serde(default, skip_serializing_if="Vec::is_empty")]` ⇒ `output_support` docs byte-identical) + `module_input_reach`/`design_input_reach` (+ helpers `input_reach_with`/`cone_support_keys`/`source_universe`/`make_reach_result`) that INVERT the support relation. `supported_query_kinds()` UNCHANGED (input_reach joins it WITH the `run_analyze` dispatch in `.3b.2`, else an intermediate commit mislabels). `"flop:<id>"` source = the Q's fan-out; source universe = inputs decl-order + flop Qs asc + instance outs sorted; control ports show empty reach. Validation: `cargo test --lib` 441/0/2 (15 analyze proofs, 7 new = transpose of cone proofs); snapshots 6/6 byte-identical; clippy/fmt clean.
- next_action: **execute `SEMANTIC-INTROSPECTION-EXPANSION.3b.2`** (the surface): in `src/introspect/analyze.rs` add `QUERY_INPUT_REACH` to `supported_query_kinds()` AND in `src/mcp/mod.rs` branch `run_analyze` by query kind (support builders vs `module_input_reach`/`design_input_reach`) in the SAME commit, updating the empty-result→`-32602` guard to check the vec the query populates; bump `SCHEMA_VERSION` `1.4→1.5` in `src/introspect/mod.rs` (+ the `"1.4"` test-assertion updates in `introspect`/`mcp` tests); add `"input_reach"` to the `analyze_schema` `enum` + refresh the tool description; schema-doc §6.7 + a `1.4→1.5` changelog entry + the `input_reach` row; book `agent-mcp` (input_reach row + worked example) + USER_GUIDE (tool enum + `1.4→1.5`) + a KM fact. Default-off / DUT byte-identical (snapshots 6/6). **API-audience steering (owner, `feedback_api_for_agents_not_humans`):** machine-friendly completeness within the SCHEMA-DERIVED ceiling.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
