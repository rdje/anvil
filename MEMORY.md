# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `STRUCTURED-EMISSION-EXPANSION.2b.1` commit (combinational `function automatic` emit-projection — **live surface, real emitter change**; default-off / **DUT byte-identical**; hash backfills next update; prior: `e9be3c7` = `.2a`; `095e471` = `.1`). Working tree clean after this commit. Push cadence: `origin/main` at `63d2622` → **19 commits ahead after this commit; push at ~30** (`feedback_push_cadence`).
- active_work_unit: **`STRUCTURED-EMISSION-EXPANSION` tree `active`; frontier = `.2b.2`.** `.1` design (decision `0012`) + `.2a` design-detail + `.2b.1` live surface all done. The first richer-structured surface — a default-off combinational `function automatic` projection — is now live and downstream-clean. Index: `docs/TASK_TREE.md`.
- `.2b.1` delivered (real emitter change, default-off byte-identical): `Config::function_emit_prob` (default `0.0`) + `Module.function_emit_gates: BTreeSet<NodeId>` (emitter-surface annotation only; identity/CSE/validators untouched) + new `src/ir/function_emit.rs` `annotate_function_emit_gates` (gen-time mark; the `soft_union.rs` precedent; **excludes `Slice`** + structured + soft-union gates + param_env modules) + call-site rolls in `generate_module`/`generate_design` (after soft_union) + `src/emit/sv.rs` rendering (`<wire>__f` `function automatic` decl + positional behaviour-preserving body + call site). 9 lib proofs. **Slice exclusion** was an empirical `-Wall UNUSEDSIGNAL` finding (a bit-select uses only a sub-range ⇒ unused param bits); Slice still emits inline, nothing retired; slice-aware projection = recorded follow-up. Forced `function_emit_prob=1.0` sweep (5 seeds) clean: Verilator `--lint-only` 5/5, Yosys both modes 5/5, Icarus 5/5 (`/tmp/anvil-fe-r2/`). No schema bump (default-off prob-knob precedent; `1.7` consumer ignores the serde-default key; `.2b.2` confirms).
- next_action: **PNT — execute `STRUCTURED-EMISSION-EXPANSION.2b.2`** (the repo-owned downstream gate + closeout): add a `saw_combinational_function_emit` coverage fact + a `num_emitted_combinational_functions` metric (structural scan of `function_emit_gates`) + a `tool_matrix` scenario (or dedicated bank) proving Verilator + both Yosys modes + Icarus accept the emitted functions warning-clean + book (a structured-emission chapter) / USER_GUIDE (the `function_emit_prob` knob) / KM fact (decision `0012` already carries `answers:`; add a knowledge card if a durable how-to is warranted) / README CLI-truth knob entry. Default-off / DUT byte-identical. Forced-sweep evidence already banked at `/tmp/anvil-fe-r2/` (5 seeds, 3 tools, both Yosys modes). Doctrine: rules-first, default-off/byte-identical, no code without a task-tree leaf owning it.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); every capability is an opt-in knob; decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
