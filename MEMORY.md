# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.2b.2a` commit (`num_emitted_combinational_functions` metric + introspection schema `1.7 → 1.8`; **DUT byte-identical** — post-hoc metric, no RTL change; hash backfills next update; prior: `15844d9` = `.2b.1`; `e9be3c7` = `.2a`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **20 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`; frontier = `.2b.2b`.** `.1`+`.2a`+`.2b.1` (live surface) done; `.2b.2` pre-split (`2026-06-16`) into `.2b.2a` (metric+schema — **done**) + `.2b.2b` (the `tool_matrix` gate — frontier) + `.2b.2c` (book/USER_GUIDE/KM/README closeout). Index: `docs/TASK_TREE.md`.
- `.2b.2a` delivered: `Metrics::num_emitted_combinational_functions` (`= m.function_emit_gates.len()`, computed in `metrics::compute()`; `#[serde(default)]`) surfaced in introspection `module_metrics` ⇒ schema MINOR bump `1.7 → 1.8` (`SCHEMA_VERSION` + 9 test assertions + schema-doc changelog + README/USER_GUIDE/book current-output refs + the `CODEBASE_ANALYSIS` envelope line, which was stale at `1.4`). The metric bumps; the `.2b.1` knob did not (Metrics-field-bumps-but-prob-knob-rides precedent). Lib proof + end-to-end introspect (default ⇒ `0`; forced ⇒ `1256`). 468 lib tests / snapshots 6/6 / mdbook build all green.
- next_action: **PNT — execute `STRUCTURED-EMISSION-EXPANSION.2b.2b`** (the repo-owned `tool_matrix` gate — **large, fragile change; a fresh session is reasonable here**): add a `saw_combinational_function_emit` coverage fact + a `--function-emit-gate` flag (or a `ScenarioSet::FunctionEmitGate`) forcing `function_emit_prob=1.0` over comb-only DUTs across the three construction strategies + a `ModuleReport.emitted_combinational_function` detection (from emitted SV `"function "` or `num_emitted_combinational_functions > 0`) + coverage-gap enforcement, proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean; bank a clean report. Template: `--signoff-knob-sweep-gate` (`src/bin/tool_matrix.rs`); emitted-construct-detection precedent: the soft_union `emitted_soft_union_overlay` / `saw_sv_version_2023_soft_union_upopt` path. WATCH: many `ModuleReport`/`Cli` test fixtures must gain the new field or compilation breaks. Forced-sweep evidence already banked at `/tmp/anvil-fe-r2/` (5 seeds, 3 tools, both Yosys modes). Then `.2b.2c` (book/USER_GUIDE/KM/README closeout). Default-off / DUT byte-identical. Doctrine: rules-first, no code without a task-tree leaf owning it.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
