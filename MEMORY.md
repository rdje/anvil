# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `CAPABILITY-BREADTH-EXPANSION.2b.1` **Mealy FSM output mechanism** commit — first **code** slice of the lane (decision `0024`); **default-off ⇒ DUT byte-identical**. `Fsm.mealy_outputs: Option<Vec<Vec<u128>>>` (`None`=Moore default / `Some`=per-`(state, sel_value)` table) + `Fsm::is_mealy()`; new default-off `fsm_mealy_prob` knob (`--fsm-mealy-prob` CLI + dump-config + overlay + `config_category` `"fsm"`); the table rolled+built in `build_fsm_block`; the emitter Mealy decode = nested `case(state_q)→case(sel)` (Moore `else`-branch byte-identical); `validate.rs` Mealy-table check; Mealy FSMs excluded from `merge_equivalent_fsms` (sound, nothing retired); `FsmOut` unchanged opaque (sel-fold deferred fidelity refinement). 2 new lib tests; `CODEBASE_ANALYSIS`/`DEVELOPMENT_NOTES` updated. Prior: `10f53ad`=`…2a` design ADR; `605ec44`=`COVERAGE-STEERED-GENERATION.2c.2`. Push cadence: `origin/main` at `605ec44` → **2 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`CAPABILITY-BREADTH-EXPANSION.2b` active** — `.2b.1` (Mealy **mechanism**) **done** + all-tool-clean; `.2b` split into `.2b.1` (done) / `.2b.2` (metric+gate) / `.2b.3` (docs). `.1` (next SV up-opt) deferred-not-retired (the `.2a` probe: enum/typedef + packed multidim arrays NOT version-distinctive per decision `0010`).
- next_action: **PNT — pick `CAPABILITY-BREADTH-EXPANSION.2b.2`** (Mealy introspection + gate): add the `num_mealy_fsm_modules` `DesignMetrics` field (mirror `num_fsm_modules`) surfaced in `--introspect` with the additive introspection schema MINOR bump **`1.12 → 1.13`**; add the `tool_matrix` `saw_mealy_fsm_design` coverage fact + a focused `fsm_mealy_prob=1.0` scenario (full Verilator + both Yosys + Icarus plan) + gap enforcement; default-off byte-identical. Then `.2b.3` (book/USER_GUIDE/README/KM docs). Re-check `docs/TASK_TREE.md` before picking.
- handoff: repo handoff-ready after this commit — **full `cargo test` green (exit 0)**, snapshots 6/6 (Moore byte-identical), clippy `-D warnings` + fmt clean; downstream probe all-tool-clean (Verilator -Wall 2012/2017/2023 + both Yosys + Icarus); `check_memory_architecture` + KM gen/check green; introspection schema still `1.12` (the `1.13` bump lands at `.2b.2`).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
