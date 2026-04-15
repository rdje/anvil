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
  - A live doc. The content evolves with the project.
  - Update the relevant chapter when the code change affects a concept the book describes (algorithm, IR, knobs, synthesizability, non-triviality, hierarchy, etc.).
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
