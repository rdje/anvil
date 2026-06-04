# 0001 - Task-tree ownership before work; strict commit workflow

- Date: 2026-06-04
- Status: accepted
- Tags: process, doctrine, continuity

## Context

ANVIL's owner has repeatedly stated that continuity and signoff quality
depend on task-tree ownership, strict commit discipline, and live-doc/book
alignment. These facts must be discoverable outside any single chat or AI
harness.

## Decision

Follow these rules:

1. Task-tree ownership comes first. No code change occurs without an
   owning task-tree leaf first. The index is `docs/TASK_TREE.md`, and
   task files live under `docs/tasks/`.
2. Code quality remains signoff-level. Prefer correct, reviewed, scoped
   work over speed.
3. Completed leaves use `COMMIT.md`: update required live docs, use
   `git_message_brief.txt`, include the work-unit id in the commit
   subject, commit one leaf at a time, and clear the message scratchpad.
4. Keep `ROADMAP.md`, the codebase, live docs, and the mdBook aligned.
   User-visible behavior changes require mdBook updates.
5. Regularly remove unused generated artifacts when it is 100% safe.

## Consequences

- A fresh agent can recover the process from tracked repo files rather
  than chat history.
- Task trees remain layer B of `MEMORY_ARCHITECTURE.md`.
- Once `MEMORY-ARCHITECTURE-DOC.4` installs enforcement, git hooks and CI
  reinforce the process.

## Links

- Authoritative docs: `README.md`, `COMMIT.md`, `docs/TASK_TREE.md`,
  `docs/TASK_TREE_README.md`, `MEMORY_ARCHITECTURE.md`
- Task-tree: `MEMORY-ARCHITECTURE-DOC.2`
