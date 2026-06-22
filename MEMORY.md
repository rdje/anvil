# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `DOCTRINE-ENFORCEMENT-ADOPTION.1` **register tree + land the doctrine-enforcement standard + decision 0026** commit — workflow/docs only, **DUT byte-identical** (no `src/`, no check wired yet ⇒ nothing can break). New `DOCTRINE_ENFORCEMENT.md` (the portable standard for architecture #4) + `docs/decisions/0026-doctrine-enforcement-adoption.md` (KM front-matter, 12 keys) + `docs/tasks/DOCTRINE-ENFORCEMENT-ADOPTION.md` (leaves `.1`–`.6`) + `docs/TASK_TREE.md` / `docs/decisions/INDEX.md` rows. Prior: `ca2ffb7`=`…12b.3` (STRUCTURED-EMISSION sixth surface, end-to-end). Push cadence: `origin/main` at `605ec44` → **12 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`DOCTRINE-ENFORCEMENT-ADOPTION` tree `active`, frontier `.2`** — adopt portable architecture #4 (doctrine enforcement: pair every doctrine with a deterministic check run from one registry+driver, gated by hook E3 + CI E4). `.1` done (standard + decision). Remaining: `.2` driver (`scripts/check_doctrines.sh` over the existing `MEMORY-ARCH` + `KNOWLEDGE-MAP` checks) + rewire `.githooks/pre-commit` + CI; `.3` `TOOLBOX.md` (ANVIL's own diagnostic tools + acceptance checklist) + `check_diagnosis_evidence.sh` (`CODE-CHANGE-EVIDENCE`); `.4` `check_task_tree_ownership.sh` (`TASK-TREE-OWNERSHIP`); `.5` discovery (bootstrap pointers + `GEMINI.md`/`.windsurfrules`); `.6` closeout (CODEBASE_ANALYSIS/DEVELOPMENT_NOTES/book/KM card; verify full driver+gate). STRUCTURED-EMISSION lane stays `active`/no-frontier (open-ended, six surfaces delivered).
- next_action: **PNT `DOCTRINE-ENFORCEMENT-ADOPTION.2`** — add `scripts/check_doctrines.sh` (collect-all-results, per-doctrine PASS/FAIL, meta-check each check exists+executable, nonzero on any fail) registering `MEMORY-ARCH` + `KNOWLEDGE-MAP`; rewire `.githooks/pre-commit` (preserve the KM derive-and-stage step, then run the driver) + `.github/workflows/ci.yml`. Prove green. Workflow only / DUT byte-identical.
- handoff: repo handoff-ready after this commit — `check_memory_architecture` + KM check (63 facts / 577 keys) green; decisions index in sync with new `0026`; no `src/` touched ⇒ `cargo check/clippy/fmt/test` + snapshots unaffected.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
