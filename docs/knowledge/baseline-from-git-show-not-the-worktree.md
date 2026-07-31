---
id: baseline-from-git-show-not-the-worktree
title: Baseline a metric from `git show HEAD:<file>`, never from the worktree you have already edited — and never run `git checkout -- <file>` during a sweep, which silently discards unrelated edits to that file
answers:
  - "how do I measure a file's size before my own changes"
  - "why was my recorded baseline wrong by a few lines"
  - "is it safe to git checkout a file during a negative-control sweep"
  - "how do I revert one experimental edit without losing others"
  - "why did a commit message claim something that was not in the diff"
date: 2026-07-30
status: current
tags: [measurement, git, baseline, sweep, gotcha, evidence]
reverify: "git show HEAD:README.md | wc -lc"
evidence: docs/tasks/README-POLICY-ADOPTION.md (the tree first registered `1773 lines / 122,920 B`, measured after a 2-line edit; true HEAD was `1771 / 122,767`); docs/tasks/EVIDENCE-BANK-DURABILITY.md (the discarded README citation)
---

**Never baseline a metric against a file you have already edited.** The README-policy tree was
registered with **`1773 lines / 122,920 B`** — measured in the worktree *after* that session's own
two-line README edit. True HEAD was **`1771 / 122,767`**. The error is small, survives review, and
contaminates every later claim derived from it.

Measure from `git show HEAD:<file>`, not from the worktree.

**And never `git checkout -- <file>` during a negative-control sweep.** Reverting one experimental
edit that way discards *every* uncommitted change to that file. It once silently cost a `README.md`
citation that the commit message then claimed was present. Because `CHANGES.md` is append-only
([`0031`](../decisions/0031-ssd-volume-exclusivity.md)), the repair is to **make the claim true in
the next leaf**, never to rewrite the entry.

**The habit that prevents both:** re-read `git diff --cached --name-only` *before* writing commit
prose, and derive every number in that prose from the staged tree rather than from memory.

See also [[negative-control-must-be-able-to-fail]] for what a control has to prove before it is
trusted.
