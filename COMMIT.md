# anvil Commit Workflow (AI Handoff Reference)
This file defines the exact commit workflow that must be followed after completing a task/activity.

## Purpose
- Provide a deterministic, repeatable commit process for any new AI or contributor.
- Preserve technical continuity across sessions.
- Keep git history clean, auditable, and aligned with project documentation policy.

## When to run this workflow
- Run after each completed task/activity (typically once per accepted implementation batch).
- Do not delay documentation sync until much later; update docs as part of the same workflow.
- If multiple tiny edits are part of one logical task, commit them together as one coherent change.

## Mandatory pre-commit doc updates
**`CHANGES.md` and `MEMORY.md` MUST be amended before every git commit, without exception.**
- `CHANGES.md` gets a new entry at the top describing the slice in full detail.
- `MEMORY.md` is updated to reflect the new current state, the new "next up", and any newly resolved or newly discovered open questions. After the commit lands, the new commit hash is added to `MEMORY.md`'s recent-commits list (either in a follow-up commit or in the next slice's `MEMORY.md` update).

A commit that does not include amendments to both `CHANGES.md` and `MEMORY.md` is a workflow violation. Stop and amend before proceeding.

## Non-negotiable pre-commit checklist

Before running `git commit`, walk through **every item** below explicitly. Do not paraphrase. Do not skip. State the answer out loud (in the response to the user) for each item that is load-bearing for the slice.

1. **Code hygiene** — all four green?
   - [ ] `cargo check --all-targets`
   - [ ] `cargo test` (this already runs `tests/snapshots.rs`, the
     `insta` byte-identical-reproducibility guard — see the
     **snapshot-acceptance protocol** below; equivalently
     `cargo insta test` when `cargo-insta` is installed)
   - [ ] `cargo clippy --all-targets -- -D warnings`
   - [ ] `cargo fmt --all --check`
   - **Snapshot-acceptance protocol (INSTA-SNAPSHOTS).** A failing
     `tests/snapshots.rs` snapshot is the byte-identical contract
     catching a *real* change in generated SystemVerilog for a
     canonical `(seed, config)`. It is **never** silently
     regenerated. Either the change is an *unintended* drift (a bug
     — fix the cause, do not touch the `.snap`) **or** it is an
     intended output change, in which case accepting the new
     snapshot is a **deliberate, separately-reviewed act**: review
     the `.snap` diff, then accept via `cargo insta accept` /
     `cargo insta review` (or, without `cargo-insta`,
     `INSTA_UPDATE=accept cargo test --test snapshots` after
     hand-reviewing the diff) and commit the updated `.snap`
     **in the same slice as, and explained by, the generator
     change that caused it**. An unexplained `.snap` change in
     `git status` is a workflow violation.
2. **`CHANGES.md`** — new entry at the top, with What/Why/Validation/Impact/Files touched. Previous entry has the landed commit hash filled in.
3. **`MEMORY.md`** — Current state refreshed. Next-up refreshed. Open questions refreshed if the slice introduced calibration assumptions or rejected alternatives with knobs. Recent-commits list updated with the *previous* commit's hash (the one being superseded by this slice).
4. **`DEVELOPMENT_NOTES.md`** — Did the slice introduce any of: new design decision, rejected alternative, non-obvious gotcha, new invariant, or a new calibration knob? If yes, append an entry. **If the last commit touched `src/` and `DEVELOPMENT_NOTES.md` has not been updated in that same commit or since, you are likely skipping this step — audit.**
5. **`CODEBASE_ANALYSIS.md`** — Did the slice change module boundaries, add/remove helpers, change enforced invariants, add/remove knobs, change the phase coverage map, or change the testing surface? If yes, amend.
6. **`ROADMAP.md`** — Did a phase label change (`done`/`mostly done`/`in progress`/`not started`)? Did an exit criterion change? Did phases get renumbered? If yes, amend.
7. **`USER_GUIDE.md`** — Did any CLI flag, knob default, or user-visible behavior change? If yes, amend.
8. **`README.md`** — Did the project objective, ramp-up flow, key paths, or CLI surface change materially? If yes, amend.
9. **`book/src/*.md`** — The mdBook is a live doc of equal standing, carrying load-bearing design context. Did the slice change a documented concept (algorithm, IR, knobs, synthesizability, non-triviality, sequential motifs, hierarchy, core idea, non-goals)? If yes, amend the relevant chapter(s). **If the slice added a new design decision or rejected alternative that deserves permanence beyond the commit message, add it to the book — short-form docs and commit messages are not adequate substitutes for a session that recovers cold.**
10. **`git status`** — Only the files intended for this slice are staged. No accidental swaps of `Cargo.lock`, no accidental `target/` inclusions, no staged `git_message_brief.txt`.
11. **Commit message** — `git_message_brief.txt` is written, concise, has the co-author trailer, and is untracked.
12. **Post-commit** — `truncate -s 0 git_message_brief.txt` is run so the next slice starts with an empty scratchpad.

If any item cannot be affirmatively answered, the commit does not proceed. No exceptions. No "I'll catch it in the next commit." No partial workflow runs.

## Files involved and exact role
- `git_message_brief.txt`
  - **Git-untracked.** Listed in `.gitignore`.
  - Temporary commit-message input file for `git commit -F`.
  - Must contain concise title + short bullet summary.
  - Must be cleared after commit (`truncate -s 0 git_message_brief.txt`) and stay empty between commits.
- `CHANGES.md`
  - Fully detailed change history.
  - Record: what changed, why (root cause or motivation), validation, and impact.
  - One entry per commit, newest at the top.
- `DEVELOPMENT_NOTES.md`
  - Engineering rationale behind decisions.
  - Record architectural insights, rejected alternatives, and known constraints/gaps.
- `MEMORY.md`
  - Compact, operational continuity/handoff snapshot.
  - Must be updated with latest completed batch context before commit.
  - Must list the new commit hash (after commit) on the next update.
- `CODEBASE_ANALYSIS.md`
  - Live Rust-workspace analysis. Typically refreshed at session bootstrap from a deep-dive into the code (see `SESSION_BOOTSTRAP.md`), and amended at any point during the session — including immediately before a commit — when the slice about to be committed materially changes workspace reality (crate layout, module ownership, IR shape, generator flow, phase-gating, currently-enforced invariants, known weaknesses).
  - The goal is resilience to session loss or crash: at any commit point, this file must reflect the code as it now is, not as it was at session start. Do not rewrite cosmetically.
- `ROADMAP.md`
  - Live phase plan. Update when phase status, scope, or exit criteria change.
  - During commit workflow, state explicitly which phase/items changed, or that no phase labels changed.
- `USER_GUIDE.md`
  - User-facing CLI and workflow reference.
  - Update when user-visible behavior, flags, or expected flow changes.
- `README.md`
  - Project entry point. Update when objective, ramp-up flow, key paths, or CLI surface change materially.
- `book/` (mdBook)
  - **A live doc of equal standing to the short-form files above.** The mdBook carries the deepest design context in the project: the core idea, the algorithm, the IR, the motifs, the rejected alternatives, the non-goals. For session recovery, the mdBook is load-bearing — a new AI or contributor that skims short-form docs and skips the book will make decisions that are locally coherent but globally wrong.
  - The content evolves with the project. Update the relevant chapter whenever a code change affects a concept the book describes (algorithm, IR, knobs, synthesizability, non-triviality, sequential motifs, hierarchy, etc.).
  - Do **not** modify `book/src/core-idea.md`, `book/src/non-goals.md`, or `book/src/why-not-grammar.md` casually — those capture load-bearing design decisions; changes need explicit justification in `DEVELOPMENT_NOTES.md`.
- `COMMIT.md` (this file)
  - Canonical commit workflow reference. Updated only when the workflow itself changes.

## Exact workflow steps
1. **Confirm task is complete and validated.**
   - Run at minimum:
     - `cargo check --all-targets`
     - `cargo test`
     - `cargo clippy --all-targets -- -D warnings` (when lint-clean is reached)
     - `cargo fmt --all --check`
   - If the change touches generator output, spot-check one seed with `verilator --lint-only` and/or `yosys -p "read_verilog -sv ...; synth"` when those tools are available locally. Record the result in `CHANGES.md`.

2. **Sync live docs with factual changes.**
   - **MANDATORY every commit:** `CHANGES.md` (new entry at top) and `MEMORY.md` (state refreshed).
   - `DEVELOPMENT_NOTES.md` when rationale applies (new decision, rejected alternative, new gotcha).
   - `ROADMAP.md` if the phase status changed.
   - `USER_GUIDE.md` if any user-visible behavior changed.
   - `README.md` if the entry-point surface changed.
   - `CODEBASE_ANALYSIS.md` whenever the slice changes workspace reality beyond what the file currently captures (new module, renamed section, new IR node kind, new generator stage, new enforced invariant, new known weakness). Amending this file before commit is encouraged whenever warranted; the bootstrap pass is not the only legitimate edit window. Do not edit cosmetically.
   - The relevant `book/src/*.md` chapter(s) if the slice changed a documented concept. The mdBook is a live doc.

3. **Display the current roadmap phase state.**
   - Show the current phase block from `ROADMAP.md` to the user.
   - Explicitly state: which phase items changed because of this slice, or state that no phase labels changed.

4. **Review pending changes.**
   - `git --no-pager status --short`
   - `git --no-pager diff --stat`

5. **Write `git_message_brief.txt`.**
   - Concise subject line (imperative, ≤70 chars).
   - Short bullet summary (what + why, not how).
   - Include the required co-author trailer.

6. **Stage intended files only.**
   - `git --no-pager add <files...>`
   - Never `git add -A` / `git add .` unless you have verified the full set matches the slice.

7. **Commit using the message file.**
   - `git --no-pager commit -F git_message_brief.txt`

8. **Clear the temporary commit-message file.**
   - `truncate -s 0 git_message_brief.txt`

9. **Verify commit completion.**
   - `git --no-pager status --short`  (should show only `git_message_brief.txt` empty, or clean)
   - `git --no-pager log -1 --oneline`
   - Record the new commit hash in `MEMORY.md` as the most recent entry (this can be part of the next commit, or a tiny follow-up commit; do not fake-edit history).

## Task-tree-managed commits

When a commit completes a leaf of an active task tree
(see [docs/TASK_TREE.md](docs/TASK_TREE.md)), the standard checklist above
still applies in full. Three additional rules:

1. The commit subject or the first body line must contain the leaf ID
   (e.g., `HIERARCHY-AWARE-IDENTITY.1`). The leaf ID is the durable
   join key between commits and the task tree — hashes can be backfilled
   into the tree later, but the leaf ID cannot.
2. The owning `docs/tasks/<TREE>.md` file must be updated in the same
   commit with the leaf's new status, verification-log entry, and
   commit-log entry. The corresponding row in `docs/TASK_TREE.md`'s
   `Active Task Trees` table must be updated when the current frontier
   changes.
3. Commit one completed leaf at a time. Do not bundle two task-tree
   leaves into one commit even when they touch the same files; the
   commit log entry in the tree expects a 1:1 mapping.

**Doctrine (2026-05-17, non-negotiable, owner directive): no code
change may be committed unless a task-tree leaf owns it.** "Code" =
anything that changes program/generator behaviour or generated RTL
(`src/`, `tests/`, `examples/`, build/codegen logic, behaviour-altering
`Cargo` manifests). The owning leaf must exist *before* the edit; its
ID goes in the commit subject / first body line. A bare `rN` slice
that no task-tree leaf owns is no longer a legal way to land a code
change — `rN` survives only as the optional within-leaf slice cadence
*inside* a tree.

Commits that are **not** task-tree-managed are limited to changes that
are **not code**: pure-docs / live-doc / mdBook edits, workflow tweaks
(including this file), and recording doctrine. Those follow the
standard checklist without the additional task-tree rules above. See
[docs/TASK_TREE.md](docs/TASK_TREE.md) "ANVIL Adoption Scope" for the
full doctrine and the code/not-code boundary.

## Commit quality rules
- Keep each commit scoped to one coherent task.
- Do not include unrelated files in the same commit.
- Ensure documentation statements match actual code/test state. No aspirational claims.
- Ensure `CODEBASE_ANALYSIS.md` reflects the current Rust workspace. Do not let it drift.
- Ensure `ROADMAP.md` phase labels match actual completion state before commit. Use only `done`, `mostly done`, `in progress`, `not started`.
- Do not promote a phase to `done` from narrative progress alone; the phase's exit criteria must be satisfied and visible in repo-owned artifacts (tests passing, example output, smoke-check evidence).
- Do not run the workflow silently; always state which live docs changed and which roadmap items moved.
- Prefer explicit, factual commit summaries over vague wording.
- `git_message_brief.txt` stays untracked and empty between commits. Never commit its contents as a tracked file.
