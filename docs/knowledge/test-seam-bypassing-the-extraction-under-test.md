---
id: test-seam-bypassing-the-extraction-under-test
title: A test seam that bypasses the extraction under test converts an oracle into a tautology — use a seam when the bypassed thing is INPUT, refuse one when it is the SUBJECT
answers:
  - "should my check script have an env seam for its self-test"
  - "how do I negative-control a doctrine check"
  - "can I reuse DOCTRINE_STAGED_OVERRIDE in a new check"
  - "why do the CHANGES-ENTRY-PLACEMENT controls run in a throwaway git repo"
  - "how do I test a script that reads git diff --cached"
  - "my control passes but proves nothing"
  - "where do I put a scratch git repo for a shell control"
  - "does a gate made only of exemptions need a floor"
  - "why does an untracked CHANGES.md fail the placement check"
date: 2026-08-01
status: current
tags: [doctrine, enforcement, testing, negative-control, vacuity, gotcha, shell, git]
reverify: "grep -n 'DOCTRINE_STAGED_OVERRIDE' scripts/*.sh   # exactly two checks carry the seam (check_diagnosis_evidence.sh, check_task_tree_ownership.sh) and NEITHER is check_changes_entry_placement.sh — for those two the staged list is INPUT to the assertion; for the placement check the two git extractions ARE the assertion. Then: grep -n 'git ls-files --error-unmatch' scripts/check_changes_entry_placement.sh   # the anti-vacuity floor that keeps an all-exemptions gate from going green forever."
evidence: scripts/check_changes_entry_placement.sh; scripts/check_diagnosis_evidence.sh; scripts/check_task_tree_ownership.sh; docs/decisions/0045-changes-entry-placement-authoring-path-check.md; docs/tasks/CHANGES-ENTRY-PLACEMENT.md; DEVELOPMENT_NOTES.md
---

When negative-controlling a check, ask what the seam **bypasses**.

- If it bypasses an **input** to the assertion, a seam is fine.
  `check_diagnosis_evidence.sh` and `check_task_tree_ownership.sh` assert *"does this
  list contain `CHANGES.md`"*; a synthetic list via `DOCTRINE_STAGED_OVERRIDE` exercises
  that faithfully.
- If it bypasses the **subject** of the assertion, the control becomes a tautology.
  `check_changes_entry_placement.sh` (`CHANGES-ENTRY-PLACEMENT`, decision `0045`) *is*
  a relation between two git extractions — `git diff --cached -U0 -- CHANGES.md` and
  `git show :CHANGES.md`. Those two commands are the only places it can be wrong, so a
  seam supplying their answers tests one line of `grep -qxF` and skips the `-U0` choice,
  the `+++ b/…` header, and the deliberate read of the **index** rather than the worktree.

The repair is cheap: run the controls in a throwaway git repository under
`.cache/anvil-sandbox/` holding a copy of the script (untracked by design —
[[reference-cache-stays-untracked-public-repo]], decisions `0031`/`0043`), so the real
plumbing executes.

**Corollary — a gate whose every path is an exemption needs a floor.** The placement
check is *correctly* silent on a provenance backfill, a typo fix, and any commit that
does not stage `CHANGES.md` (99 of 768 commits in history). That same shape means it
would go green forever, reporting `ok`, the instant its subject stopped existing — hence
the untracked-`CHANGES.md` hard failure, on the same reasoning as the count floors in
[[extractor-charset-narrower-than-source]] and `check_enumeration_parity.sh`.

Full story: decision `0045`, `docs/tasks/CHANGES-ENTRY-PLACEMENT.md` `.5`, and the
`2026-08-01` `DEVELOPMENT_NOTES.md` entry of the same name. See also
[[doctrine-enforcement]] for the registry+driver a new check plugs into.
