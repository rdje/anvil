# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `CAPABILITY-BREADTH-EXPANSION.2a` **Mealy FSM output design ADR** commit — **DOCS-ONLY** (no `src` ⇒ DUT byte-identical). New `docs/decisions/0024-mealy-fsm-outputs.md` (KM `answers:` front-matter) pins the Mealy model: a default-off combinational output decode of `(state_q, sel)` — a per-`(state, sel_value)` constant table mirroring `transitions`, rendered as the **already-proven** nested `case(state_q)→case(sel)` emitter form driving the **opaque** `Node::FsmOut` leaf (only its decode reads the input-dependent `sel`; state stays Moore-clocked). Pins `fsm_mealy_prob` knob + `num_mealy_fsm_modules` metric (introspection schema `1.12→1.13` at impl) + `saw_mealy_fsm_design` gate + `--fsm-mealy-prob`/MCP. `.2` split into `.2a` (done) + `.2b` (impl, proposed). Updated INDEX + tree + `docs/TASK_TREE.md` + CHANGES + DEVELOPMENT_NOTES; KM regen folds `0024` (was **58 facts / 519 keys**). Prior: `605ec44`=`COVERAGE-STEERED-GENERATION.2c.2` (closed that tree). Push cadence: `origin/main` at `605ec44` → **1 commit ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`CAPABILITY-BREADTH-EXPANSION` active** — `.2a` Mealy design ADR (decision `0024`) **done**; **frontier = `.2b`** (Mealy impl). `.1` (next SV up-opt) **deferred, not retired**: the `.2a` empirical probe re-confirmed (per decision `0010`) that the named candidates (enum/typedef, packed multidim arrays) are accepted at every Verilator `--language` mode + Yosys + Icarus ⇒ NOT version-distinctive (no down-gating teeth); the genuinely-2023 clean space is thin (essentially `union soft`, shipped).
- next_action: **PNT — pick `CAPABILITY-BREADTH-EXPANSION.2b`** (Mealy impl, the first **code** slice of this lane; task-tree-owned ⇒ code allowed). Pre-split `.2b` into `.2b.1` design-detail / `.2b.2` impl / `.2b.3` docs per the `.2b` precedent. The `Fsm` IR layout + `FsmOut` virtual-deps folding `sel`'s support + Mealy dedup keying resolve at `.2b.1`. Re-check `docs/TASK_TREE.md` active table before picking.
- handoff: repo handoff-ready after this commit — docs-only; `check_memory_architecture` + KM gen/check green (KM regen folds decision `0024`); introspection schema still `1.12` (the `1.13` bump lands with `.2b` code). Last full `cargo test` green at `.2c.1` (`12416c1`); this slice changes no `src`, so the code gate is unchanged.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary.
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
