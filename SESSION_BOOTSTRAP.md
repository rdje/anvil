# Session Bootstrap
Read this first when starting or recovering a session.

> This file is also **automatically re-injected after context
> compaction** by a `PostCompact` hook in `.claude/settings.json`
> (it emits this file as `additionalContext`). So after an auto- or
> manual `/compact`, treat the recovery protocol below as freshly in
> force and re-run it — do not assume prior in-context state survived.

## What you (the AI/LLM) must do at session start

1. **Read every live doc, in this order, in full:**
   - `README.md`
   - `ROADMAP.md`
   - `MEMORY.md`
   - `CHANGES.md` (most recent entries at minimum)
   - `DEVELOPMENT_NOTES.md`
   - `CODEBASE_ANALYSIS.md`
   - `USER_GUIDE.md`
   - `COMMIT.md`
   - `docs/TASK_TREE.md` — repo-local task-tree workflow and active-tree index. If any tree is listed as `active`, read the linked `docs/tasks/<TREE>.md` file too; its current frontier names the next eligible leaf for PNT-style execution.
   - **The mdBook** (`book/src/`) — not reference material, a *live doc*. Read `SUMMARY.md` and every chapter it points to. The mdBook carries the deepest design context (core idea, algorithm, IR, motifs, non-goals, rejected alternatives) that cannot be reconstructed from code or short-form docs. A session that skips the mdBook will make locally-correct but globally-wrong decisions.
2. **Then perform a precise and thorough analysis of the Rust code base.** Walk every source file under `src/`, every test under `tests/`, and every example under `examples/`. Build a mental model of the actual code reality: types, function signatures, control flow, invariants enforced in code, gaps between code and the live docs.
3. **Amend `CODEBASE_ANALYSIS.md` if your deep-dive surfaces facts the existing analysis misses or misstates.** This file is the authoritative snapshot of the workspace. The bootstrap pass is the most common amendment window, but it is not the only one — during the session you may also need to update it to propagate aspects of recent changes. The discipline is: at any moment the project might be interrupted, `CODEBASE_ANALYSIS.md` should still describe the code as it actually is. Do not rewrite cosmetically.
4. **Run the sanity checks** below before making changes.

This bootstrap protocol exists so a fresh session reaches the same operational state as the session that just ended, with no silent assumptions and no drift between docs and code.

## Non-negotiable doctrine: task-tree ownership of code

**(2026-05-17, owner directive, no compromise.)** It is **strictly
forbidden to make any code change without it being task-tree tracked
or task-tree owned first.** Before touching `src/`, `tests/`,
`examples/`, or any build/codegen logic, confirm a task-tree leaf owns
the change; if none exists, create or extend a tree
(`docs/tasks/<TREE>.md` + a `docs/TASK_TREE.md` row) and name the
owning leaf **before** editing code. The leaf ID goes in the commit
subject. Pure-docs / live-doc / mdBook / workflow-config edits are not
"code changes" and are exempt. Full statement and the code/not-code
boundary: `docs/TASK_TREE.md` "ANVIL Adoption Scope" and `COMMIT.md`
"Task-tree-managed commits". A recovering session must treat this as
in force immediately.

## Sanity checks
```bash
cargo check --all-targets
cargo test
git --no-pager log -5 --oneline
git --no-pager status --short
```

Expected state:
- `cargo check` passes.
- `cargo test` passes.
- `git status` is clean, or shows an in-progress slice consistent with `MEMORY.md`.

Any deviation is a signal to stop and investigate before making changes.

## What not to do on bootstrap
- Do not edit `book/src/core-idea.md`, `book/src/non-goals.md`, or `book/src/why-not-grammar.md` as a warm-up. Those capture load-bearing design decisions; revising them requires a deliberate task.
- Do not reorganize the crate layout to match a mental model formed before reading `CODEBASE_ANALYSIS.md` and walking the code.
- Do not commit without running the full `COMMIT.md` workflow.

## When in doubt
Open `MEMORY.md`. It records what the last session was doing, what landed, and what was about to happen next. If `MEMORY.md` is stale or contradicts `git log`, trust `git log` and update `MEMORY.md` as part of the next commit per `COMMIT.md`.
