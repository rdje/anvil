# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `SEMANTIC-INTROSPECTION-EXPANSION.4a` commit (`flop_reset_provenance` impl design-detail, DOCS-ONLY, DUT byte-identical; hash backfills next update; prior: `b2e3ea7` = `.3b.2`; `42f3ea9` = `.3b.1`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **11 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`SEMANTIC-INTROSPECTION-EXPANSION.4` (`flop_reset_provenance`, the third derived query)**. Two queries already delivered (`output_support` `.1`/`.2` + `input_reach` `.3`, schema `1.5`). `.4a` design **done**; `.4b` pre-split → `.4b.1` (pure core, **frontier**) + `.4b.2` (surface). Index: `docs/TASK_TREE.md`.
- `.4a` delivered (DOCS-ONLY): `DEVELOPMENT_NOTES.md` design entry grounded in the `Flop` type. **Decisions for `.4b`:** (1) a THIRD parallel vec `flop_provenance: Vec<FlopProvenance>` on `DerivedAnalysis` (`#[serde(default, skip_serializing_if="Vec::is_empty")]` ⇒ output_support/input_reach byte-identical); `FlopProvenance { flop, width, has_reset, reset_kind, reset_value (u128-safe DECIMAL STRING), default_behavior ("zero"|"hold"), mux_kind ("none"|"one_hot"|"encoded"), mux_arms, has_d }` (enums → strings). (2) derivation = a DIRECT projection of `Module.flops` (ascending id, no graph walk). (3) addressing: `None`⇒all flops; `Some("flop:<id>")`⇒one; unknown⇒`-32602`; flopless+None⇒empty. (4) schema MINOR `1.5 → 1.6`, envelope reused.
- next_action: **execute `SEMANTIC-INTROSPECTION-EXPANSION.4b.1`** (pure core): in `src/introspect/analyze.rs` add `QUERY_FLOP_RESET_PROVENANCE="flop_reset_provenance"` + the `FlopProvenance` struct + the `flop_provenance` field on `DerivedAnalysis` + `module_flop_provenance(&Module, Option<&str>)`/`design_flop_provenance(&Design, Option<&str>)` (project `Module.flops`; map `ResetKind`/`FlopKind`/`FlopMux` → strings; `reset_value` decimal). **Do NOT add to `supported_query_kinds()` yet** — registry + `run_analyze` dispatch land together in `.4b.2`. Lib proofs: each ResetKind/FlopKind/FlopMux variant; `None`⇒all flops asc; flopless⇒empty; `"flop:<id>"`+unknown⇒none; determinism. Snapshots 6/6 byte-identical. Then `.4b.2` (surface: registry+dispatch, schema `1.5→1.6`, analyze_schema enum, schema-doc/book/USER_GUIDE/README/KM). **API-audience steering (`feedback_api_for_agents_not_humans`):** machine-friendly completeness within the SCHEMA-DERIVED ceiling.
- after `.4`: remaining named kind = `module_reachability` (open-ended `.5+`, none retired). Other trees (`SIGNOFF-AUTOMATION-EXPANSION`, `IDENTITY-DEEPENING`) are `active` with no current frontier; `STRUCTURED-EMISSION-EXPANSION` is `proposed` (owner-gated activation).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
