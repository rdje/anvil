# UNGATED-PRACTICE-AUDIT: a practice survives where a gate observes it — which practices here are unobserved?

## Metadata

- Tree ID: `UNGATED-PRACTICE-AUDIT`
- Status: `active`
- Roadmap lane: Workflow / gate quality
- Created: `2026-08-01`
- Last updated: `2026-08-01` (registered from a `NEGATIVE-CONTROL-HARNESS.1` finding; frontier `.1`)
- Owner: repo-local workflow

## Goal

`NEGATIVE-CONTROL-HARNESS.1` measured one practice and found a general rule underneath it.

The practice was *"prove the mutation landed"*. The measurement: the **identical** primitive —
compare a file against a known baseline — is run **27 times across 6 trees** proving a control's
**revert** landed, and **twice** proving its **mutation** did. Same authors, same rule, same files.

The cause is not attention. A failed **revert** leaves a **dirty** tree, and three mechanisms punish
that (the pre-commit driver, `COMMIT.md` §10's `git status` review, the pivot rule). A failed
**mutation** leaves a **clean** tree, which all three read as success.

> **A practice survives where a gate observes it, and erodes where none does.**

That rule makes a **testable prediction**, and this tree exists to test it rather than to admire it:
*other* practices this project believes it follows are unobserved by any gate, and should therefore
be measurably eroding right now. The rule is either useful or it is a nice sentence; only a
measurement separates the two.

## The honest obstacle, stated up front

**This is a hypothesis, not a known defect.** No specific erosion has been measured. `.1` may find
that every unobserved practice here is in fact healthy — in which case the rule is *weaker* than
`NEGATIVE-CONTROL-HARNESS.1` implies, and saying so plainly is the deliverable. A tree that can only
confirm its own premise is not a measurement.

The second obstacle is **selection**: "practices this project believes it follows" is not a set
anyone has enumerated, and inventing one to fit the conclusion is the failure
`USER-GUIDE-CLI-TABLE-SHADOW.7` recorded as *do not manufacture a finding to justify a leaf*. `.1`
must derive the candidate set from something authoritative — `COMMIT.md`'s checklist, `TOOLBOX.md`
Part 2's boxes, the `DOCTRINE_ENFORCEMENT.md` §10 registry's complement — and state the derivation.

## Why a new tree rather than a leaf of an existing one

`feedback_full_factorization` applied, not waved through:

- `NEGATIVE-CONTROL-HARNESS` is **closed** and its subject was one practice, mechanised at `.3`.
  Re-opening it to hold a different, wider question would make its own record harder to read.
- `DOCTRINE-ENFORCEMENT-ADOPTION` owns *turning a doctrine into a check*. This tree asks the prior
  question — *which practices are not doctrines at all, and does that matter* — and its likely
  output is a **classification**, not a new check.
- `SHADOW-ENUMERATION-SWEEP` is the nearest shape (a defect **class** swept across the repo) and is
  closed; its subject was hand-maintained lists, not practices.

## Non-Goals

- **Not a mandate to gate everything.** `DOCTRINE_ENFORCEMENT.md` §9 is explicit that a gate which
  cries wolf gets deleted, and `0033` test (2) makes over-gating a defect in its own right. Several
  practices here are *correctly* ungated.
- **Not a re-statement of the rule.** It is already recorded in `DEVELOPMENT_NOTES.md` and decision
  `0047`. This tree measures whether it predicts anything.
- **No code change** in the generator sense.

## Acceptance Criteria

- `.1` **derives** its candidate set from an authoritative source rather than listing practices from
  memory, states the derivation and the denominator, and classifies each candidate as
  **gate-observed**, **unobserved-and-healthy**, or **unobserved-and-eroding** — with the evidence
  for each verdict, not an impression.
- *"The rule does not predict erosion here"* is a legitimate and fully acceptable result, and is
  stated plainly if that is what the measurement says.
- `.2` decides what, if anything, follows — for each eroding practice, whether the repair is a gate,
  a redesign that removes the need, or a recorded acceptance. Per `0047`'s precedent, *removing the
  need* outranks *watching harder*.
- `scripts/check_doctrines.sh` stays green; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `UNGATED-PRACTICE-AUDIT`
  Status: `active`
  Goal: `Test whether "a practice survives where a gate observes it" predicts anything about THIS repo, and act only on what it predicts.`
  Children: `.1` (derive the candidate set and classify it), `.2` (decide what follows, per candidate)

- ID: `UNGATED-PRACTICE-AUDIT.1`
  Status: `pending`
  Goal: `Derive the candidate set of practices from an authoritative source, state the denominator, and classify each as gate-observed, unobserved-and-healthy, or unobserved-and-eroding — on evidence.`
  Acceptance: `The derivation is stated and re-runnable, not a remembered list. Each verdict cites what was measured. "No erosion found" is a legitimate result and must be reported as such rather than reframed. Where a practice is unobserved AND healthy, say why it survives without a gate — that is the more interesting half.`
  Verification: `pending`
  Commit: `pending`

- ID: `UNGATED-PRACTICE-AUDIT.2`
  Status: `pending`
  Goal: `Decide what follows for each eroding practice — a gate, a redesign that removes the need, or a recorded acceptance — before building anything.`
  Acceptance: `Per candidate, with its failure mode stated. Removing the need outranks adding a gate (decision 0047's R1-over-R2 precedent). Over-gating is a defect, not thoroughness.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `UNGATED-PRACTICE-AUDIT.1` | `pending` | **Next.** Measure before concluding. The temptation is to assume the rule generalises because it was satisfying to find — and a tree that can only confirm its premise measures nothing. |
| 2 | `UNGATED-PRACTICE-AUDIT.2` | `pending` | Decide per candidate, against what `.1` actually found. |

## Decisions

- `2026-08-01` (registration): **Registered rather than left as prose.** The rule is recorded in
  `DEVELOPMENT_NOTES.md` and decision `0047`, but a *prediction* recorded only as narrative is never
  tested — and the standing directive is that a finding is handled only when a tree owns it.
  Registered at a clean tree boundary, immediately after `NEGATIVE-CONTROL-HARNESS` closed, per the
  pivot rule.
- `2026-08-01` (registration): **Framed as a hypothesis under test, not a defect to repair.** No
  erosion has been measured. Framing it as a known problem would pre-load `.1` toward finding one,
  which is exactly how a confident wrong answer gets manufactured.

## Open Questions

- **What is the authoritative source for "practices this project believes it follows"?** Candidates:
  `COMMIT.md`'s non-negotiable checklist, `TOOLBOX.md` Part 2's eight boxes, the complement of
  `DOCTRINE_ENFORCEMENT.md` §10's registry. Each yields a different denominator; `.1` picks one,
  states why, and reports what the others would have given.
- **Is "observed by a gate" binary?** `COMMIT.md`'s checklist items are *self-ticked* — §6.1 says a
  tick is a claim, not proof — so some practices are watched by something that cannot actually see
  them. That may be a third class, worse than unobserved, because it looks observed.
- **Does the rule survive its own test?** If `.1` finds unobserved-but-healthy practices, what
  distinguishes them? A plausible answer is that they leave a visible artifact anyway; if so the
  real rule is about *observability*, not gates, and this tree should say so.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `.0` | `registered from NEGATIVE-CONTROL-HARNESS.1's measured finding — 27 recorded verifications that a control's REVERT landed against 2 that its MUTATION did, with the asymmetry explained by which side a gate watches. Ownership search run, not assumed: NEGATIVE-CONTROL-HARNESS is closed and held one practice; DOCTRINE-ENFORCEMENT-ADOPTION owns turning a doctrine into a check, not deciding whether a practice needs one; SHADOW-ENUMERATION-SWEEP is closed and held hand-maintained lists.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `ef6413c` — `UNGATED-PRACTICE-AUDIT.0 — register the generalisation from NEGATIVE-CONTROL-HARNESS.1` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |

## Changelog

- `2026-08-01`: Created. `NEGATIVE-CONTROL-HARNESS.1` measured one practice and found a general rule
  underneath it — **a practice survives where a gate observes it, and erodes where none does** —
  evidenced by a **27-to-2** split between proving a control's *revert* landed and proving its
  *mutation* did, explained entirely by which of the two leaves a dirty tree behind. The rule makes a
  testable prediction about *other* practices here, and a prediction recorded only as narrative is
  never tested. Registered as a hypothesis under test rather than as a defect: **no erosion has been
  measured**, and *"the rule does not predict anything here"* is an acceptable and fully reportable
  outcome.
