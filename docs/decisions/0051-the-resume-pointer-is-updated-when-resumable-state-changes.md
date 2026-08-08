---
id: the-resume-pointer-is-updated-when-resumable-state-changes
title: The resume pointer is amended when **resumable state changes**, not before every commit — `MEMORY.md` is a `bounded_snapshot` holding only what the repository cannot derive, so `latest_commit` and the cross-tree priority queue are deleted and `CODE-CHANGE-EVIDENCE` stops asserting the file
answers:
  - "must I update MEMORY.md before every commit"
  - "why does MEMORY.md no longer record latest_commit"
  - "where do I find the current commit if MEMORY.md does not say"
  - "why did CODE-CHANGE-EVIDENCE stop requiring MEMORY.md"
  - "should MEMORY.md carry a queue of what to do after the next action"
  - "which tree do I pick after the current one closes"
  - "when should a doctrine check assert that a file was co-staged"
  - "MEMORY.md keeps hitting its byte cap on commits that changed nothing in it"
date: 2026-08-08
status: accepted
tags: [memory-architecture, doctrine, resume-pointer, commit-workflow, shadow-list, gate-quality, bounded-snapshot, north-star]
evidence: docs/tasks/RESUME-POINTER-COMMIT-PATH-COUPLING.md (`.2`'s measurement — the queue reconstructed as `docs/TASK_TREE.md`'s active-row order minus one silently dropped tree, and the 5,235 -> 5,175 B / four-fields effect); docs/decisions/0050-resume-pointer-holds-one-work-unit-not-a-roster.md (`.1`, which deleted the roster field and left the queue open); docs/decisions/0033-shadow-enumeration-classification.md (the three-part test and the R1 ladder); DOCTRINE_ENFORCEMENT.md §6.1 (a box is earned, not ticked — the reason the dropped assertion was never evidence); scripts/check_memory_architecture.sh (the forbidden-field assertion, negative-controlled both ways)
reverify: "bash -c \"grep -qE '^[-*[:space:]]*\\`?latest_commit\\`?[[:space:]]*:' MEMORY.md && echo 'BREACH: latest_commit field is back' || echo 'ok: HEAD stays derived'; DOCTRINE_STAGED_OVERRIDE=\\$'src/gen/mod.rs\\nCHANGES.md' bash scripts/check_diagnosis_evidence.sh >/dev/null && echo 'ok: a code change no longer needs MEMORY.md staged'\""
---

# 0051 - RESUME-POINTER-COMMIT-PATH-COUPLING.2: the resume pointer is updated when resumable state changes

- Date: 2026-08-08
- Status: accepted
- Tree: `RESUME-POINTER-COMMIT-PATH-COUPLING.2` (closes the tree)
- Activated by: an owner directive — *"Update `MEMORY.md` when resumable state changes, not before
  every commit unconditionally. Remove unconditional `MEMORY.md` co-staging from
  `CODE-CHANGE-EVIDENCE`."*

## Context

`.0` measured the coupling: a fail-closed **6,144 B** cap on the path **820 of 840 (97.6 %)** commits
were *required* to take, with **79 B** of slack. `.1` found the term that grew — a roster field
shadowing the active-tree set, already drifted to **7 of 16** — and deleted it, recording in `0050`
two residues it explicitly left open:

> **`next_action` still carries a priority queue** … It is *not* a shadow — a priority ordering is
> not derivable from any set … **This does not fix the coupling itself.**

`.2` owns both. The first of those two claims turned out to be **wrong**, and measurably so.

## What `.2` measured

**(a) The queue was a shadow after all — and its authoritative set was already named.**
`docs/TASK_TREE.md` §*PNT Selection Rules* says, in the workflow's own words: *"choose the first
active tree in the table."* So the *Active Task Trees* table's row order **is** the cross-tree
selection authority, and the queue mirrored it. `0033`'s three-part test therefore passes:

| Test | Verdict at `596e624` |
| --- | --- |
| **derivable** | yes — and *proven by reconstruction*: the queue's entries appear in **exact** table order |
| **growth-coupled** | yes — it named 3 of the **15** trees carrying `Status: active` |
| **silent** | **demonstrated** — `TASK-LEAF-COMMIT-SHADOW` sits at table position **3** with a `pending` leaf marked *"Next."*, and is **absent from the queue**, while `UNGATED-PRACTICE-AUDIT` (position **33**) is present. Nothing failed |

The reconstruction is what settles it. The queue was not an independent editorial ordering that
merely resembled the table — it *was* the table's active-row order **with one tree dropped**. So the
R1 deletion loses no judgement; the only thing it discards is the drift. `0050`'s *"a priority
ordering is derivable from no set"* was true of orderings in general and false of this one, because
this repository had already elected an ordering and written the election down.

**(b) `latest_commit` could not be correct even once.** It is written *before* the commit that
contains it exists, so it can only name that commit's **predecessor**. Observed directly at
bootstrap: the field read `f9e1c61` while HEAD was `596e624`, three commits later. `COMMIT.md` had
institutionalised the error rather than removed it, mandating a backfill *"in a follow-up commit or in
the next slice"* — a hand-maintained copy of `git log -1` that cost a commit per slice to keep
half-right.

**(c) The `CODE-CHANGE-EVIDENCE` assertion was evidence-shaped without being evidence.** This is the
deep finding, and it is not about frequency — [[gate-frequency-is-not-evidence]] stands, and `.0`
already recorded that mis-citing it was the original sin here. The distinction is:

- `CHANGES.md` is a function of **the diff**. One entry describes *this* commit; staging it is a
  claim that can be false, and `CHANGES-ENTRY-PLACEMENT` plus the `cargo`/`tool_matrix` oracle can
  falsify it.
- `MEMORY.md` is a function of **the work**. It records where a session stopped. A commit that
  changes no resumable state has *nothing true to write in it*.

So for that class of commit the requirement could only be discharged by a no-op diff in an
overwrite-only, hard-capped file — **the cheapest possible box to tick, hence `DOCTRINE_ENFORCEMENT.md`
§6.1's self-tick rather than proof.** Worse, it was a self-tick that *consumed a finite resource*:
each no-op update spent headroom under the cap, and when the cap fired at `4925847` the remedy taken
was the **prose trim the gate's own routing hint forbids** — reported as compliance, with no check
comparing remedy-taken against remedy-prescribed.

## Decision

**1. `MEMORY.md` is classified `bounded_snapshot`** (`LIVE_DOCUMENT_SIZE_CONTAINMENT.md`'s lifecycle
vocabulary): overwrite semantics, no embedded chronology, line and byte limits. **Both existing caps
are kept unchanged** — 50 lines / 6,144 B. This decision lowers *demand*, never raises supply;
`0040` non-license 3 stands.

**2. It holds only what the repository cannot derive.** Deleted accordingly: the `latest_commit`
field (→ `git log -1 --oneline`), its predecessor list (→ `git log --oneline`), the recent-decisions
range (→ `ls docs/decisions/`), the cross-tree priority queue (→ `docs/TASK_TREE.md`'s first `active`
row, per its own PNT rule), and a dated count of gotcha cards (→ the `grep` beside it, which *is* the
derivation).

**3. Exactly one active work unit and one next action.** `0050` established the first; this extends
the same singular reading of `MEMORY_ARCHITECTURE.md` §6 to the second. Within-tree ordering stays in
each tree's `Current Frontier` — *"ordered by intended priority"* is already that section's contract.

**4. It is amended when resumable state changes — that is, when any of its four fields changes —
and not otherwise.** `CHANGES.md` remains mandatory on **every** commit, unchanged.

**5. `CODE-CHANGE-EVIDENCE` no longer asserts `MEMORY.md` co-staging.** The general rule, recorded in
`DOCTRINE_ENFORCEMENT.md` §10: **assert co-staging only where the artifact's content is a function of
the diff.** `MEMORY-ARCH` keeps sole ownership of `MEMORY.md`'s size, shape and required fields —
one mechanism per job (`feedback_full_factorization`).

**6. `MEMORY-ARCH` gains one assertion: no `latest_commit:` field.** Not a new doctrine — a second
assertion inside the check that already owns this file, exactly as `0040` seated the byte cap there.

## Why a gate here when `0050` declined one

`0050` deliberately left `active_work_unit` ungated: *"a roster reappearing is obvious on sight."*
That reasoning is kept, and it is why **no** check asserts "exactly one work unit" or "no queue". A
hash field is the opposite case: it looks precisely like correct bookkeeping, so its return would be
**silent** — and silence is what earns a check (`DOCTRINE_ENFORCEMENT.md` §2).

The assertion **matches the field shape, not the word**, because the file legitimately explains in
prose why the field is gone. A whole-file substring match would fire on that explanation — §9's
vacuity failure inverted: *a check that cannot distinguish its subject from a mention of its subject
is wrong in whichever direction it errs.* Negative-controlled **both ways** via
`scripts/negative_control.sh`, whose `apply` refuses a substitution that matches nothing, so each
result is known to be an experiment that ran: reintroducing the field **fires**; adding a mid-sentence
mention stays **silent**.

## Effect, measured

| | before `.1` | after `.1` | after `.2` |
| --- | ---: | ---: | ---: |
| `MEMORY.md` bytes | 6,071 | 5,235 | **5,175** |
| headroom to the 6,144 B cap | 73 B | 909 B | **969 B** |
| growth term per new task tree | ~40–60 B | 0 | **0** |
| commits *required* to touch the file | **97.6 %** | 97.6 % | **only those that change resumable state** |
| follow-up commits per slice to backfill a hash | 1 | 1 | **0** |

The fourth row is the one `.0` opened the tree for. The bytes were always the symptom.

## Honest limits (§9 — stated, not discovered later)

- **"Resumable state changed" is an author's judgement, not a mechanical predicate.** Nothing
  computes it, and a check that tried would need to know what the next action *should* be. The
  failure mode is now a **stale** pointer rather than a **blocked** commit — a deliberate trade,
  because a stale pointer is visible to the next session that reads it while the blocked commit was
  paid on every commit by every author.
- **This removes an assertion; it does not add a currency check.** Nothing verifies `next_action`
  still describes reality. `LIVE_DOCUMENT_SIZE_CONTAINMENT.md`'s named currency-contract mechanism is
  the right shape for that, and it is the adoption tree's to decide — not smuggled in here.
- **The queue's deletion is not gated**, per the reasoning above; a queue reappearing is visible.
- **`docs/TASK_TREE.md`'s row order is now load-bearing** in a way it was not before. It was already
  the PNT authority, but the queue masked drift in it. If the true priority differs from the row
  order, the repair is to **move the row**, never to keep a second list.
- **`0026` quotes the superseded mandate** and is left untouched: layer C is append-only and is
  superseded by record, never silently rewritten (`MEMORY_ARCHITECTURE.md` §3).

## Explicit non-licenses

1. **Not a licence to raise either cap.** Both are unchanged; `0040` non-license 3 stands.
2. **Not a licence to skip `CHANGES.md`.** Its mandate is untouched and remains unconditional.
3. **Not a licence to let `MEMORY.md` go stale** because it is no longer gated. The obligation
   moved from *"every commit"* to *"every commit that changes resumable state"* — which is stricter
   in substance and weaker only in ceremony.
4. **Not a general licence to delete co-staging assertions.** The test is decision-rule 5: is the
   artifact's content a function of the diff? For `CHANGES.md` it is, so it stays.
5. **Does not resume `OVERFLOW-DESTINATION-INSTRUMENTATION`**, which remains `deferred` per `0050`.

## Consequences

- `RESUME-POINTER-COMMIT-PATH-COUPLING` closes: `.1` removed the growth, `.2` removed the coupling.
- `COMMIT.md`, `TOOLBOX.md` Part 2, `DOCTRINE_ENFORCEMENT.md` §10 and `MEMORY_ARCHITECTURE.md` §3/§5/§6
  now state one rule between them, with the standard's portable template corrected at source so the
  deleted field is not re-imported by the next reader.
- The rule that outlives the file: **a gate that a compliant author can satisfy with a no-op diff is
  not measuring compliance — it is charging rent for it.**
