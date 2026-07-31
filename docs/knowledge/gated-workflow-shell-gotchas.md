---
id: gated-workflow-shell-gotchas
title: `cmd | tail` **destroys** `cmd`'s exit status, and `scripts/check_doctrines.sh` must run **after** `git add` — two shell facts that have each produced a green result proving nothing
answers:
  - "why did cargo test appear to pass when it failed"
  - "does piping to tail hide a command's exit status"
  - "why was my commit message file emptied after a rejected commit"
  - "why does a doctrine check not see my new file"
  - "when should I run check_doctrines.sh relative to git add"
  - "how do I capture a command's exit status safely in bash"
date: 2026-07-30
status: current
tags: [shell, workflow, gate-quality, git, commit, gotcha]
reverify: "false | tail -1; echo $?"
evidence: docs/tasks/COVERAGE-STEERED-GENERATION.md (both incidents on 2026-07-30); scripts/check_doctrines.sh; .githooks/pre-commit
---

**`cmd | tail` destroys `cmd`'s exit status.** Bash reports the *last* stage's status. This bit
twice in one day:

- `cargo test 2>&1 | tail -30` reported success while proving nothing;
- `git commit -F brief && truncate -s 0 brief` ran the `truncate` **after the hook rejected the
  commit**, emptying the message file that still contained the unlanded message.

**Redirect to a log and echo `$?` on its own line. Never chain `&&` cleanup off a piped command.**

**Run `scripts/check_doctrines.sh` AFTER `git add`.** `git grep` and `git ls-files` see **tracked**
content only, so a new file's contents are invisible to every structural check until it is staged.
Three self-catches in two days were all this one blind spot — and the failure mode is a **green
run that examined nothing**, which is the worst available direction.

See also [[negative-control-must-be-able-to-fail]] — both of these produce a pass that proves
nothing, which is the same class of defect as a control too weak to fail.
