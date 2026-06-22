# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `DOCTRINE-ENFORCEMENT-ADOPTION.5` **discovery layer: route every harness pointer to the doctrine kit** commit — workflow/docs only, **DUT byte-identical** (no `src/`). `CLAUDE.md`/`AGENTS.md`/`.cursorrules`/`.github/copilot-instructions.md` now also point at `DOCTRINE_ENFORCEMENT.md` + `TOOLBOX.md` (README + MEMORY_ARCHITECTURE mentions preserved); new `GEMINI.md` + `.windsurfrules` (byte-identical body); `check_memory_architecture.sh` `BOOTSTRAP_FILES` extended with both (six pointers uniformly enforced); `README.md` ramp-up items 18+19. Driver green (6 pointers ok, PASS x4). Prior: `4a49681`=`…4` (TASK-TREE-OWNERSHIP). Push cadence: `origin/main` at `605ec44` → **16 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **`DOCTRINE-ENFORCEMENT-ADOPTION` tree `active`, frontier `.6`** — adopt portable architecture #4. `.1`–`.5` done (standard+decision 0026 · driver+hook/CI · TOOLBOX+CODE-CHANGE-EVIDENCE · TASK-TREE-OWNERSHIP · discovery). Remaining: `.6` closeout — align `CODEBASE_ANALYSIS.md` (new enforcement layer) + `DEVELOPMENT_NOTES.md` (rationale) + `book/src/architecture.md` (enforcement narrative) + a `docs/knowledge/doctrine-enforcement.md` KM card (working reverify); verify the full driver + full COMMIT.md gate (cargo check/clippy/fmt/test + mdbook); close the tree. STRUCTURED-EMISSION lane stays `active`/no-frontier.
- next_action: **PNT `DOCTRINE-ENFORCEMENT-ADOPTION.6`** (closeout) — update `CODEBASE_ANALYSIS.md` + `DEVELOPMENT_NOTES.md` + `book/src/architecture.md` to record portable architecture #4; add KM card `docs/knowledge/doctrine-enforcement.md` (answers/reverify = `bash scripts/check_doctrines.sh`); run `cargo check/clippy/fmt/test` + `mdbook build book` + the full driver; mark the tree `done`; update `docs/TASK_TREE.md` row. Then the adoption is complete end-to-end.
- handoff: repo handoff-ready after this commit — `scripts/check_doctrines.sh` green (4 doctrines, 6 bootstrap pointers; 63 facts / 577 keys); pre-commit + CI run the driver; no `src/` touched ⇒ `cargo check/clippy/fmt/test` + snapshots unaffected.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
