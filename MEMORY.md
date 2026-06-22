# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `DOCTRINE-ENFORCEMENT-ADOPTION.4` **mechanize the flagship TASK-TREE-OWNERSHIP doctrine** commit — workflow only, **DUT byte-identical** (no `src/`). New executable `scripts/check_task_tree_ownership.sh` (scope-aware: code staged ⇒ an owning `docs/tasks/*.md` ≠ `TEMPLATE.md` co-staged; non-code exempt; bash-3.2-safe) registered as `TASK-TREE-OWNERSHIP` in the driver (now **4 doctrines**). Proven on 4 staged-set cases (exempt / pass / fail-no-task / fail-template-only). Prior: `fb5ecac`=`…3` (TOOLBOX + CODE-CHANGE-EVIDENCE). Push cadence: `origin/main` at `605ec44` → **15 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`DOCTRINE-ENFORCEMENT-ADOPTION` tree `active`, frontier `.5`** — adopt portable architecture #4. `.1` standard+decision 0026 · `.2` driver+hook/CI · `.3` TOOLBOX + `CODE-CHANGE-EVIDENCE` · `.4` `TASK-TREE-OWNERSHIP` — all done. Remaining: `.5` discovery (route every harness bootstrap pointer to `DOCTRINE_ENFORCEMENT.md` + `TOOLBOX.md`; add `GEMINI.md` + `.windsurfrules`); `.6` closeout (CODEBASE_ANALYSIS/DEVELOPMENT_NOTES/book architecture chapter/KM card; verify full driver + full COMMIT.md gate; close tree). STRUCTURED-EMISSION lane stays `active`/no-frontier (open-ended, six surfaces delivered).
- next_action: **PNT `DOCTRINE-ENFORCEMENT-ADOPTION.5`** — discovery layer: update `CLAUDE.md` / `AGENTS.md` / `.cursorrules` / `.github/copilot-instructions.md` / the `README.md` ramp-up list to also point at `DOCTRINE_ENFORCEMENT.md` + `TOOLBOX.md` (preserve the `README.md` + `MEMORY_ARCHITECTURE.md` mentions the memory-arch check requires); add new `GEMINI.md` + `.windsurfrules` (byte-identical content); optionally add them to the memory-arch BOOTSTRAP_FILES. Driver stays green. Workflow/docs only / DUT byte-identical.
- handoff: repo handoff-ready after this commit — `scripts/check_doctrines.sh` green (4 doctrines: MEMORY-ARCH + KNOWLEDGE-MAP + CODE-CHANGE-EVIDENCE + TASK-TREE-OWNERSHIP; 63 facts / 577 keys); pre-commit + CI run the driver; no `src/` touched ⇒ `cargo check/clippy/fmt/test` + snapshots unaffected.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
