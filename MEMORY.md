# MEMORY - resume pointer (layer A; overwrite-only, keep <= 50 lines)

## How to resume
- Read `README.md`, then `MEMORY_ARCHITECTURE.md`.
- Work is task-tree tracked under `docs/tasks/`; index: `docs/TASK_TREE.md`.
- Durable cross-task facts live in `docs/decisions/`.
- Question-keyed retrieval facts are indexed in `KNOWLEDGE_MAP.md`.
- Commit completed leaves per `COMMIT.md`; include the leaf id in the subject.

## Current state (overwrite this block; do not append history)
- latest_commit: this `DOCTRINE-ENFORCEMENT-ADOPTION.6` **closeout: live-doc + book + KM alignment; tree done** commit — workflow/docs only, **DUT byte-identical** (no `src/`). `CODEBASE_ANALYSIS.md` (driver+4 doctrines bullet) + `book/src/architecture.md` (`## Doctrine enforcement` section) + `DEVELOPMENT_NOTES.md` (rationale entry) + new KM card `docs/knowledge/doctrine-enforcement.md` (reverify=`bash scripts/check_doctrines.sh`; KM 63→64 facts / 577→589 keys) + tree/`docs/TASK_TREE.md` row `done`. **`DOCTRINE-ENFORCEMENT-ADOPTION` tree CLOSED — portable architecture #4 adopted end-to-end (`.1`–`.6`).** Prior: `ea04769`=`…5`. Push cadence: `origin/main` at `605ec44` → **17 commits ahead** after this ⇒ no push (threshold 30, `feedback_push_cadence`).
- active_work_unit: **NO active doctrine-enforcement frontier — `DOCTRINE-ENFORCEMENT-ADOPTION` is `done`.** ANVIL now runs all four portable architectures (task-trees, memory, knowledge-map, doctrine-enforcement). The doctrine driver `scripts/check_doctrines.sh` gates `MEMORY-ARCH` + `KNOWLEDGE-MAP` + `CODE-CHANGE-EVIDENCE` + `TASK-TREE-OWNERSHIP` at E3 (pre-commit) + E4 (CI). The open-ended **`STRUCTURED-EMISSION-EXPANSION`** lane remains `active` with no current frontier (six surfaces delivered).
- next_action: **PNT at a no-frontier boundary** (`feedback_pick_and_roll_at_no_frontier` — self-select; don't surface a steer question). Options: (a) a NEW structured surface `.13+` for STRUCTURED-EMISSION (wider `k>2` co-supported task groups is the natural next), or (b) re-scan `docs/TASK_TREE.md` for another `active` lane with a live frontier. A discovered out-of-scope follow-up exists: `book/src/agent-mcp.md` has untagged ``` fences that fail `mdbook test` under mdbook 0.5.2 (CI pins 0.4.40, green) — a `LIVE-DOC-BOOK-ALIGNMENT`-style fix (tag fences `text` and/or bump the CI mdbook pin), if/when prioritized.
- handoff: repo handoff-ready after this commit — `scripts/check_doctrines.sh` green (4 doctrines, 6 bootstrap pointers; 64 facts / 589 keys); `cargo check/clippy/fmt` clean; `mdbook build` clean; pre-commit + CI run the driver; no `src/` touched ⇒ `cargo test` + snapshots unaffected.
- lane invariants (all lanes): rules-first / no generate-then-filter (valid-by-construction); default-off / byte-identical where output could change; `tests/snapshots.rs` untouched by default; **no retirement** (`feedback_never_retire_strategies`); one runner + one classifier not two (`feedback_full_factorization`); every capability opt-in + MCP-invocable + queryable + CLI-as-shim (decision `0017`); design the API for agents not humans (`feedback_api_for_agents_not_humans`); the book is the user-facing surface and must not drift (`feedback_book_doctrine`); decision/KM fact per durable capability or boundary. Doctrine-enforcement (decision `0026`): every doctrine = a deterministic check in `scripts/check_doctrines.sh`; code-scoped checks are scope-aware (non-code commits exempt).
- in_flight_uncommitted: none after this commit. Repo handoff-ready: tree clean, self-checks green, resume pointer current.
- blockers: none. Validation policy: focused checks + `scripts/ram_guard.sh --threshold 90 -- <cmd>` for heavy builds (note the `--` separator); monitor RAM, stop >90% per `0003-resource-safe-validation`.

## Validation policy
- For workflow-doc memory/retrieval architecture leaves, use focused functional checks; full `cargo test` is not required per owner instruction.
- If a future full suite is run, monitor RAM; stop immediately above 90% RAM and record it as an environment/resource stop.
