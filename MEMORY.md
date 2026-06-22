# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1` **align roadmap owner-directed-lane status** commit — docs-only / no source / **DUT byte-identical**. A session-bootstrap audit found `ROADMAP.md`'s three owner-directed-lane sections drifted behind delivered reality (`docs/TASK_TREE.md` rows + `README.md` verified current); fixed 8 stale lane statements: structured-emission 5th (`cone_function`, `0016`) + 6th (`multi_output_task`, `0025`) surfaces; `AGENT-MCP-EXPANSION` closed; `SIGNOFF-AUTOMATION-EXPANSION` active+`.2`; `IDENTITY-DEEPENING.3b.2b` (`0008`); `DOWNSTREAM-ADAPTER` sv2v+slang; `KNOB-ERGONOMICS` (`0021`); `CI-PACKAGING` (`0022`); `CAPABILITY-BREADTH` Mealy (`0024`). New `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT` tree + index row. Prior: `f6c9c1a`=`…13c`. Push cadence: `origin/main` at `605ec44` → **21 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **NO active frontier — `LIVE-DOC-ROADMAP-LANE-STATUS-ALIGNMENT.1` is `done` (tree closed) by this commit.** All `active` lanes (`STRUCTURED-EMISSION-EXPANSION`, `IDENTITY-DEEPENING`, `SIGNOFF-AUTOMATION-EXPANSION`, `SEMANTIC-INTROSPECTION-EXPANSION`, `DOWNSTREAM-ADAPTER-EXPANSION`, `KNOB-ERGONOMICS-AND-PRESETS`, `CI-PACKAGING-DISTRIBUTION`, `CAPABILITY-BREADTH-EXPANSION`, `LOCAL-REFERENCE-CACHE`) remain at no-frontier boundaries.
- next_action: **PNT at a no-frontier boundary** (`feedback_pick_and_roll_at_no_frontier` — self-select; don't surface a steer question). Candidates: (a) a NEW STRUCTURED-EMISSION surface `.14+` (nested/multi-level `generate`; `interface`/`modport` empirically DISQUALIFIED on Yosys/Icarus — fresh angle needed), or (b) re-scan `docs/TASK_TREE.md` for another `active` lane with a pickable next increment. Latent docs follow-up: `book/src/agent-mcp.md` untagged ``` fences fail `mdbook test` under mdbook 0.5.2 (CI pins 0.4.40, green) — a `LIVE-DOC-BOOK-ALIGNMENT`-style fix if/when prioritized.
- handoff: repo handoff-ready after this commit — `scripts/check_doctrines.sh` green (4 doctrines); `git diff --check` clean; docs-only so `cargo`/snapshots untouched ⇒ DUT byte-identical, no `src/`. New tree file + index row + `ROADMAP.md` + `CHANGES.md` + `MEMORY.md` co-staged.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
