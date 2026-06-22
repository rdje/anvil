# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `DOCTRINE-ENFORCEMENT-ADOPTION.3` **TOOLBOX.md (ANVIL's own diagnostic tools) + CODE-CHANGE-EVIDENCE check** commit — workflow/docs only, **DUT byte-identical** (no `src/`). New `TOOLBOX.md` (ANVIL's diagnostic instruments by diagnosis-type + the acceptance-checklist, each box citing a named oracle) + new executable `scripts/check_diagnosis_evidence.sh` (scope-aware: code staged ⇒ `CHANGES.md`+`MEMORY.md` co-staged; non-code exempt; bash-3.2-safe) + `CODE-CHANGE-EVIDENCE` registered in the driver (now **3 doctrines**). Evidence check proven on 4 staged-set cases (exempt / pass / fail / Cargo.lock-as-code). Prior: `fbe6849`=`…2` (driver+hook/CI). Push cadence: `origin/main` at `605ec44` → **14 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`DOCTRINE-ENFORCEMENT-ADOPTION` tree `active`, frontier `.4`** — adopt portable architecture #4. `.1` standard+decision 0026 · `.2` driver+hook/CI · `.3` TOOLBOX + `CODE-CHANGE-EVIDENCE` — all done. Remaining: `.4` `scripts/check_task_tree_ownership.sh` (`TASK-TREE-OWNERSHIP`, scope-aware: code staged ⇒ owning `docs/tasks/*.md` co-staged); `.5` discovery (bootstrap pointers + `GEMINI.md`/`.windsurfrules`); `.6` closeout (CODEBASE_ANALYSIS/DEVELOPMENT_NOTES/book/KM card; verify full driver+gate). STRUCTURED-EMISSION lane stays `active`/no-frontier (open-ended, six surfaces delivered).
- next_action: **PNT `DOCTRINE-ENFORCEMENT-ADOPTION.4`** — add `scripts/check_task_tree_ownership.sh` (scope-aware structural: code staged ⇒ a `docs/tasks/*.md` task file other than `TEMPLATE.md` co-staged — mechanizes the 2026-05-17 ownership doctrine + COMMIT.md task-tree rule #2; non-code commits exempt; bash-3.2-safe, `DOCTRINE_STAGED_OVERRIDE` test seam) + register `TASK-TREE-OWNERSHIP` in the driver. Prove green across the staged-set cases. Workflow only / DUT byte-identical.
- handoff: repo handoff-ready after this commit — `scripts/check_doctrines.sh` green (3 doctrines: MEMORY-ARCH + KNOWLEDGE-MAP + CODE-CHANGE-EVIDENCE; 63 facts / 577 keys); pre-commit + CI run the driver; no `src/` touched ⇒ `cargo check/clippy/fmt/test` + snapshots unaffected.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
