---
id: practice-survives-as-a-by-product-not-by-a-gate
title: A practice survives where its output is a **by-product of work the author is already forced to do** — a gate is one way to force that, not the variable itself; measured against five *equally ungated* sections that split 100 %/94 %/94 % vs 64 %/50 %
answers:
  - "does a practice erode when no gate watches it"
  - "why did this checklist item decay when that one did not"
  - "should I add a gate to stop a practice eroding"
  - "what predicts whether an ungated practice stays healthy"
  - "why do some unobserved practices stay at 100 percent"
  - "how do I test whether a rule about process actually predicts anything"
  - "is a self-ticked checklist box observed"
  - "what is the difference between an eroding practice and a stale specification"
  - "how do I pick the denominator for a practice audit"
  - "can I use the complement of the doctrine registry as a candidate set"
date: 2026-08-02
status: current
tags: [workflow, gate-quality, doctrine-enforcement, measurement, gotcha, process, honest-limits]
evidence: docs/tasks/UNGATED-PRACTICE-AUDIT.md (`.1`'s measurement — a 20-obligation denominator derived by recorded command from `COMMIT.md`'s checklist, five classes, 811 commits / 698 `CHANGES.md` entries / a 250-commit `MEMORY.md` hash trace, plus the gated control group); docs/decisions/0047-negative-control-carrier-is-the-mutation.md (the R1-over-R2 ladder this refinement generalises); DOCTRINE_ENFORCEMENT.md §6.1 (a box is EARNED, not ticked) and §9 (honest limits)
reverify: "awk '/^## Non-negotiable pre-commit checklist$/{f=1;next} /^## /{f=0} f && /^[0-9]+\\. \\*\\*/' COMMIT.md   # the 12 items the denominator derives from. Then re-score the CHANGES.md template: count entries carrying a line-leading **Why. / **Impact. label in the newest 50 vs entries 401-500."
---

**The tempting rule is wrong as stated.** `NEGATIVE-CONTROL-HARNESS.1` found the identical
file-versus-baseline check run **27** times to prove a control's *revert* landed and **twice** to
prove its *mutation* did, and explained it with: *a practice survives where a gate observes it, and
erodes where none does.* `UNGATED-PRACTICE-AUDIT.1` tested that prediction and it **failed**.

## The measurement that refutes it

`COMMIT.md` item 2 asks every `CHANGES.md` entry for five sections. **No script reads any of the
five** — verified, not assumed: the only match in `scripts/*.sh` is a *comment*, and
`check_diagnosis_evidence.sh` asserts `CHANGES.md` is **staged**, never what is in it. So all five
are equally ungated. They did not decay equally:

| Section | newest 50 | the by-product behind it |
| --- | --- | --- |
| `Files touched.` | 100 % | `git diff --stat`, forced by checklist item 10 |
| `What.` | 94 % | the diff itself |
| `Validation.` | 94 % | the four `cargo` runs, forced by item 1 |
| `Why.` | 64 % | **none** — recall and composition only |
| `Impact.` | 50 % | **none** — recall and composition only |

Equal gate coverage, a 50-point spread. Gate-presence cannot be the discriminator.

> **What discriminates is whether the output falls out of work the author is already forced to do.**

## Why the refinement is the stronger rule

It explains everything the literal rule explains **and** the cases it cannot:

- the **27-to-2** split that started this — a failed *revert* leaves a dirty tree as a by-product; a
  failed *mutation* leaves nothing;
- five *unobserved-and-healthy* practices measured in the same audit — the `CHANGES.md` landed hash
  (**621/623**) and `MEMORY.md`'s previous-commit hash both fall out of the commit that just ran; the
  co-author trailer (**619 consecutive**) is emitted by the *authoring harness*, not by any repo
  gate; `git_message_brief.txt` staying untracked (**0/811 ever tracked**) is forced by `.gitignore`;
- and it demotes gating from *the* mechanism to *one* mechanism, which is what
  [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md)'s **R1 over R2** already
  says — prefer a form that *cannot* fail silently over a check that watches for the failure.

**Practical consequence:** before proposing a gate, ask *what work already produces this as a
by-product, and can I route the obligation through that instead?* Only if nothing does is a gate the
right tool. See [[doctrine-enforcement]].

## Two classes the rule does not name, both found by the same audit

**1. Observed by something that cannot see it.** `MEMORY-ARCH` is the gate on *"`MEMORY.md` state
refreshed"*, and what it asserts is that four field **names** are present (`active_work_unit`,
`next_action`, `in_flight_uncommitted`, `blockers`) plus the line/byte caps. **Presence is not
freshness** — a file frozen for fifty commits passes every leg. This is worse than unobserved,
because it looks observed. `DOCTRINE_ENFORCEMENT.md` §6.1 predicted the shape; this is a live one.

**2. The specification eroded, not the practice.** The single "eroding" candidate looks like a
collapse (**100 % → 32 %** template conformance) and is not. Reading six of the non-conforming
entries showed the *Why* and *Impact* content **present**, under bespoke headings
(`**The finding.**`, `**The claim under test.**`, `**Docs-only ⇒ DUT byte-identical**`). The practice
outgrew the template; the template was never updated, because nothing reads it. The real cost is not
lost information — it is that the item became **unverifiable by anything short of reading the
prose**, so it cannot be gated as written, and an auditor scoring the repo against its own document
records 32 %. Before calling a metric erosion, **read the artifacts**: a label detector measures
form, and form is not substance. See [[coverage-check-vacuity]].

## How to pick the denominator (this is where such an audit actually fails)

*"Practices this project believes it follows"* is not an enumerated set, so it must be **derived**
and the derivation stated — otherwise the candidate list gets invented to fit the conclusion, which
is `USER-GUIDE-CLI-TABLE-SHADOW.7`'s *do not manufacture a finding to justify a leaf*. Three sources
were available and only one works:

- **`COMMIT.md`'s non-negotiable checklist — usable.** Normative for every commit, and *self-ticked*,
  which is the property that makes the question testable. **12 items ⇒ 20 atomic obligations.**
- **`TOOLBOX.md` Part 2 — rejected, and the reason generalises.** It declares itself a *mirror* of
  the same checklist, and **every one of its 8 boxes already cites a named oracle** — so its
  membership rule pre-selects the gate-observed half. *A source whose selection criterion is the
  variable under test cannot measure that variable.*
- **The complement of the `DOCTRINE_ENFORCEMENT.md` §10 registry — not derivable.** A complement
  needs a universe, and the universe is exactly the unenumerated set. Reporting that is the result;
  approximating it would have manufactured the denominator.

## The control group, and why an audit like this needs one

Report a gated practice over the **same window**, or "32 % is bad" has nothing to mean. The
leaf-id-in-subject rule is gated by `.githooks/commit-msg` (landed `2d01e8e`): **0 failures in the
410 commits since.** Same authors, same commits, same files — one gated, one not.

## The instrument failed its own control first

The `Landed as:` detector was probed against a constructed two-entry fixture
([`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md) rung **R1** — no match step
that can silently no-op) *before* its number was reported, and **it failed**: it matched the marker
inside a code span and false-negatived. Anchored to line start, the count moved **75 → 77** and the
verdict was unchanged. Run the control before you publish the number, not after — that is
[[negative-control-must-be-able-to-fail]] applied to a measuring instrument rather than to a gate.

## The live violation this audit committed while performing it

Decision [`0031`](../decisions/0031-ssd-volume-exclusivity.md) §1 puts **agent scratch** inside the
storage obligation. `scripts/check_no_boot_volume_refs.sh` scopes itself to **tracked files** — its
own comment says *"untracked scratch is the author's business"*, which is narrower than the decision
it enforces. Proved structurally rather than argued: the same banned string in an **untracked** file
⇒ the check exits `0`; the identical file **staged** ⇒ it fails and names it.

The audit's own session wrote three scratch files to a boot-volume path before re-reading `0031`,
under a fully green driver, having read the doctrine twenty minutes earlier. **The strongest single
data point in the audit is the one the auditor produced by accident** — and it is a by-product
failure, not a gate failure: nothing an agent is forced to do puts scratch on the repo volume, so
the harness default wins.
