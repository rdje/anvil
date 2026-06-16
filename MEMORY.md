# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.3a` commit (`input_reach` impl design-detail, DOCS-ONLY; hash backfills next update; prior: `e635dd1` = `.3` handoff; `fcddc1c` = `IDENTITY-DEEPENING.3b.2b.2b`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **8 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION.3` (`input_reach`, the dual fan-OUT of the delivered `output_support` cone)**. `.3a` design-detail **done**; `.3b` pre-split → `.3b.1` (pure core, **frontier**) + `.3b.2` (surface). Index: `docs/TASK_TREE.md`.
- `.3a` delivered (DOCS-ONLY, DUT byte-identical): `DEVELOPMENT_NOTES.md` design-detail entry resolving the four points, grounded in `src/introspect/analyze.rs`/`mod.rs` + `src/mcp/mod.rs`. **Decisions for `.3b`:** (1) result shape = a new `ReachResult { target, reaches_outputs[], reaches_flops[], fanout_targets }` + a SECOND parallel vec `reach_results: Vec<ReachResult>` on `DerivedAnalysis` with `#[serde(default, skip_serializing_if="Vec::is_empty")]` (⇒ `output_support` docs byte-identical; rejected enum/shoehorn). (2) derivation = INVERT the support relation (enumerate outputs + `"flop:<id>"` targets → build each via existing `module_support_cones` → bucket `T` under each `X∈support(T)`); dual-consistency free, no IR/gen change. (3) addressing: `None`⇒all sources (inputs decl-order, flop Qs, instance outs) incl. empty; `Some(input)`/`Some("flop:<id>")`=Q fan-out/`Some("<inst>.<port>")`; `"flop:<id>"` direction set by query kind; unknown⇒`-32602`. (4) schema MINOR `1.4 → 1.5`, envelope reused.
- next_action: **execute `SEMANTIC-INTROSPECTION-EXPANSION.3b.1`** (pure core, no MCP/schema yet): in `src/introspect/analyze.rs` add `QUERY_INPUT_REACH="input_reach"` + the `ReachResult` struct + the `reach_results` field on `DerivedAnalysis` + the pure builders `module_input_reach(&Module, Option<&str>)`/`design_input_reach(&Design, Option<&str>)` (enumerate-targets → reuse `module_support_cones` → invert → resolve-source). **Do NOT add `input_reach` to `supported_query_kinds()` yet** — that registry entry + the `run_analyze` dispatch land together in `.3b.2` (else an intermediate commit mislabels). Lib proofs = the transpose of the support-cone proofs + flop-Q/instance-output sources + `None`-all-sources + determinism + unknown-source. Snapshots 6/6 byte-identical. Then `.3b.2` (surface: registry+dispatch, schema `1.4→1.5`, `analyze_schema` enum, schema-doc/book/USER_GUIDE/KM). **API-audience steering (owner, `feedback_api_for_agents_not_humans`):** machine-friendly completeness (full reach sets, all-sources, explicit ids) within the SCHEMA-DERIVED ceiling.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
