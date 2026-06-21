# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `CAPABILITY-BREADTH-EXPANSION.2b.2a` **Mealy metric + introspection schema `1.13`** commit — **default-off ⇒ DUT byte-identical**. `DesignMetrics::num_mealy_fsm_modules` (filter over `Module::fsms` for `is_mealy()`; `<= num_fsm_modules`; serde-projected into `--introspect`) + the additive introspection schema bump **`1.12 → 1.13`** (const + doc comment + the 3 introspect + 8 MCP `schema_version` test assertions + `docs/AGENT_INTROSPECTION_SCHEMA.md` §6.3/§7). Makes the Mealy capability **queryable** (decision `0017`). Prior: `dc7df68`=`…2b.1` mechanism; `10f53ad`=`…2a` ADR. Push cadence: `origin/main` at `605ec44` → **3 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`CAPABILITY-BREADTH-EXPANSION.2b` active** — `.2b.1` (mechanism) + `.2b.2a` (metric + schema `1.13`) **done**; `.2b.2` split into `.2b.2a` (done) / `.2b.2b` (tool_matrix gate). `.1` (next SV up-opt) deferred-not-retired (the `.2a` probe: enum/typedef + packed multidim arrays NOT version-distinctive per decision `0010`).
- next_action: **PNT — pick `CAPABILITY-BREADTH-EXPANSION.2b.2b`** (the Mealy `tool_matrix` gate): a repo-owned `saw_mealy_fsm_design` coverage fact + a focused `fsm_mealy_prob=1.0` scenario (full Verilator + both Yosys + Icarus plan — Mealy is universally synthesizable) + `ModuleReport`/`DesignReport` detection + gap enforcement, mirroring the FSM/memory motif gates (`src/bin/tool_matrix.rs`). Then `.2b.3` (book `sequential.md`/`knobs.md` + USER_GUIDE + README + KM card, **incl. the deferred schema-`1.12→1.13` book-example refresh** in `api-tools.md`/`agent-mcp.md`/`api-introspection.md` — the coverage-steered-lane precedent). Re-check `docs/TASK_TREE.md` before picking.
- handoff: repo handoff-ready after this commit — **full `cargo test` green (exit 0)**, lib 589/0, snapshots 6/6 (byte-identical), clippy `-D warnings` + fmt clean; live `--introspect` (hierarchy, `fsm_mealy_prob=1.0`) → `num_mealy_fsm_modules: 2` at schema `1.13`; `check_memory_architecture` + KM gen/check green; introspection schema **now `1.13`** (README + `docs/AGENT_INTROSPECTION_SCHEMA.md` at `1.13`; the book prose/JSON `1.12` examples refresh at `.2b.3`, precedented deferral).
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
