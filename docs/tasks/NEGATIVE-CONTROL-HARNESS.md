# NEGATIVE-CONTROL-HARNESS: "prove the sabotage landed" is carried as a habit, and habits fail

## Metadata

- Tree ID: `NEGATIVE-CONTROL-HARNESS`
- Status: `active`
- Roadmap lane: Workflow / gate quality
- Created: `2026-08-01`
- Last updated: `2026-08-01` (registered from a `BOOK-LINK-INTEGRITY.3` finding; frontier `.1`)
- Owner: repo-local workflow

## Goal

A negative control is how this repo proves a gate can actually fire. Three legs are needed, and
`USER-GUIDE-CLI-TABLE-SHADOW.7` recorded that only the third is routinely skipped:

1. a control proves the check **can** fire,
2. the `DOCTRINE_ENFORCEMENT.md` §9 vacuity probe proves it fires on the **right input**,
3. asserting the mutation **landed** proves **the experiment ran at all**.

Leg 3 is the one that fails silently, and it fails in the dangerous direction: a substitution that
does not match leaves the tree unchanged, the check passes, and that is **indistinguishable from a
control that correctly did not fire**. The author reads it as a finding about the gate.

**The lesson is already written down, and it still cost two false results.** That is the subject
of this tree — not the lesson, which is fine, but its *carrier*.

## How it was found (`2026-08-01`, at `9ad7385`)

Not by a sweep. `BOOK-LINK-INTEGRITY.3` had to prove that its new check's whole-file extractor was
load-bearing, by reverting it to the line-wise form and confirming the gate then **misses** a
wrapped escape. **That single control took three attempts to actually run.** Twice the mutating
substitution silently failed to apply — once a `sed` whose escaping was wrong, once a `perl -pe`
whose was — and both times the check "passed", which is exactly the reading the rule warns about.

The distribution of the failures is the finding:

| where the probe was written | leg-3 failures |
| --- | ---: |
| inside the reusable harness (`.cache/book_link_controls.sh`, whose `probe` helper asserts its marker before reading any verdict) | **0** of 10 |
| ad-hoc, inline, outside the harness | **2** of 3 |

Same author, same leaf, same hour, same rule in mind. The variable was not diligence; it was
**whether the assertion was structural or remembered**.

## The honest obstacle, stated up front

**A control runs *during* a leaf and may leave no artifact in the commit.** Every doctrine in this
repo checks *repository state*; a check cannot see that an experiment was conducted correctly an
hour before the commit. So this may be **not doctrine-shaped at all** — the reflex to reach for
`scripts/check_*.sh` should be resisted until `.2` decides. A `TOOLBOX.md` instrument, or a
sourceable helper that makes the correct thing the easy thing, may be the honest answer. So may
"nothing" (`DOCTRINE_ENFORCEMENT.md` §9).

## Why a new tree rather than a leaf of an existing one

`feedback_full_factorization` applied, not waved through — and the search was **run**, not assumed:

- `DOCTRINE_ENFORCEMENT.md` §6.1 (*a box is EARNED, not ticked*) is the nearest neighbour and does
  **not** cover this. It governs a **gated checklist box** citing a named re-runnable oracle. A
  negative control is not a gated box; nothing ticks, and its output is a verdict about the *gate*,
  not about the change.
- `ENUMERATION-PARITY` owns declared list pairs. `TABLE-RENDER-FIDELITY` owns markdown
  well-formedness. `BOOK-LINK-TARGETS` owns book link targets. None touches control conduct.
- `grep` over `scripts/` for an existing sabotage/mutation helper returns **nothing**. There is no
  mechanism to extend, so this would create the first, not a second.

## Non-Goals

- **Not a re-statement of the rule.** It is already recorded (`USER-GUIDE-CLI-TABLE-SHADOW.7`,
  `DEVELOPMENT_NOTES.md`). Writing it down again is precisely the remedy that has already failed.
- **Not a mandate that every leaf run controls.** Controls are for gates and instruments, not for
  every edit.
- **No code change** in the generator sense; this is workflow tooling and docs.

## Acceptance Criteria

- `.1` measures rather than asserts: how many recorded control episodes exist across
  `docs/tasks/*.md` verification logs, and in how many is there **any way to tell** whether leg 3
  was performed. The denominator is stated, per decision `0039`. A likely and legitimate outcome is
  *"unknowable from the record"* — which is itself the finding, and is stronger than a guess.
- `.2` decides the **carrier** before building anything, with each candidate's failure mode stated:
  a sourceable `scripts/` helper (makes the right thing easy; cannot compel use), a `TOOLBOX.md`
  instrument (discoverable; still a habit), a doctrine check (structural — *if* an artifact exists
  for it to read, which `.1` must establish), or nothing.
- If a mechanism lands it obeys `DOCTRINE_ENFORCEMENT.md` §4 and is negative-controlled both
  ways — **including a control on the control**: the harness must fail loudly when its own marker
  assertion is removed.
- `scripts/check_doctrines.sh` stays green; docs/workflow-only ⇒ DUT byte-identical.

## Task Tree

- ID: `NEGATIVE-CONTROL-HARNESS`
  Status: `active`
  Goal: `Decide whether "prove the sabotage landed" can be carried by something other than the author's attention, and if so by what.`
  Children: `.1` (measure the record), `.2` (decide the carrier), `.3` (build it, or record why not)

- ID: `NEGATIVE-CONTROL-HARNESS.1`
  Status: `pending`
  Goal: `Measure how the repo's recorded control episodes report leg 3, and whether any commit-visible artifact exists that a check could read. No mechanism in this leaf.`
  Acceptance: `A stated denominator over docs/tasks/*.md verification logs, not a sample. Names the episodes where leg 3 is visibly reported, visibly absent, or unknowable. "Unknowable from the record" is a legitimate and likely result — say so plainly rather than inferring compliance from silence.`
  Verification: `pending`
  Commit: `pending`

- ID: `NEGATIVE-CONTROL-HARNESS.2`
  Status: `pending`
  Goal: `Decide the carrier — sourceable helper, TOOLBOX instrument, doctrine check, or nothing — and record it BEFORE building.`
  Acceptance: `Each candidate's failure mode stated, including the structural objection that a control leaves no commit artifact, which may disqualify the doctrine shape outright. The decision cites .1's measurement rather than intuition.`
  Verification: `pending`
  Commit: `pending`

- ID: `NEGATIVE-CONTROL-HARNESS.3`
  Status: `pending`
  Goal: `Implement .2's choice, or record why nothing is warranted per DOCTRINE_ENFORCEMENT.md section 9.`
  Acceptance: `If a harness lands it is itself negative-controlled: removing its marker assertion must make it fail loudly, not silently pass. If nothing lands, the reason is recorded where a future session will meet it.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `NEGATIVE-CONTROL-HARNESS.1` | `pending` | **Next.** Measure before choosing a carrier. The temptation is to write the helper immediately — it is twenty lines and obviously useful — and that is the same reflex that produced a written-down rule nobody could execute reliably. |
| 2 | `NEGATIVE-CONTROL-HARNESS.2` | `pending` | The carrier decision, against what `.1` finds is actually visible in the record. |
| 3 | `NEGATIVE-CONTROL-HARNESS.3` | `pending` | Build it, or record why not. |

## Decisions

- `2026-08-01` (registration): **Registered rather than fixed in passing.** A `probe` helper
  already exists at `.cache/book_link_controls.sh` and could have been promoted to `scripts/` in
  one commit during `BOOK-LINK-INTEGRITY.3`. It was deliberately not: the standing directive is
  that a defect is only handled if a tree owns it, `COMMIT.md` lands one leaf per commit, and the
  pivot rule wants a clean tree first. Promoting it silently would also have **assumed** the
  carrier question that `.2` exists to decide.

## Open Questions

- **Is this doctrine-shaped at all?** A control leaves no artifact in the commit, so a check may be
  structurally unable to see it. `.1` must establish whether *any* commit-visible trace exists.
- **Can a helper compel its own use?** A sourceable helper only protects probes written inside it —
  and both failures here were written *outside* it, by an author who knew it existed.
- **Is the real subject narrower — shell substitutions?** Both failures were escaping mistakes in
  `sed` / `perl -pe` one-liners. A helper that owns *the mutation* (apply, verify, revert) rather
  than *the probe* might address the actual mechanism. `.2` should consider it.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `.0` | `found while BOOK-LINK-INTEGRITY.3 proved its extractor load-bearing: 3 attempts for 1 control, 2 silently-failed mutations. Distribution: 0 leg-3 failures in 10 harness-written probes, 2 in 3 ad-hoc probes. Ownership search run, not assumed: DOCTRINE_ENFORCEMENT.md 6.1 governs gated checklist boxes (not controls); grep over scripts/ for a sabotage/mutation helper returns nothing.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `NEGATIVE-CONTROL-HARNESS.0 — register the finding from BOOK-LINK-INTEGRITY.3` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention (`RESUME-POINTER-CONTRACT.0`, `EMIT-SURFACE-INTERACTION-GATE.0`), required because `.githooks/commit-msg` rejects a subject that names no leaf. |

## Changelog

- `2026-08-01`: Created. `BOOK-LINK-INTEGRITY.3` needed one load-bearing control and it took
  **three attempts to run**, twice passing on a mutation that had silently failed to apply — the
  exact trap `USER-GUIDE-CLI-TABLE-SHADOW.7` recorded and wrote into `DEVELOPMENT_NOTES.md`. The
  rule was known, recent, and in mind. What separated the 10 clean probes from the 2 broken ones
  was not care but **carrier**: the clean ones ran inside a helper that asserts its marker before
  reading a verdict, the broken ones were ad-hoc one-liners. Registered rather than repaired in
  passing, because promoting that helper would have pre-decided the carrier question — and because
  the honest obstacle (a control leaves no artifact in the commit) may mean the answer is not a
  doctrine at all.
