---
id: gate-frequency-is-not-evidence
title: A gate firing often is not evidence its remedy is wrong — it is evidence the constrained thing is at its limit; state the falsification test and RUN it before escalating "this mechanism misfires"
answers:
  - "the MEMORY.md byte cap keeps firing, is the doctrine broken"
  - "a check fires every session, should I change it"
  - "is high gate-firing frequency a defect"
  - "should I raise a cap that keeps blocking commits"
  - "does routed-out MEMORY.md content come back"
  - "how do I test whether a doctrine's remedy is the wrong one"
  - "when is it safe to escalate a mechanism-misfires finding"
  - "is MEMORY.md a rotating log with entries to overwrite"
  - "can the MEMORY.md cap squeeze out the next-frontier pointer itself"
  - "how much headroom is left in MEMORY.md before something load-bearing must go"
date: 2026-08-01
status: current
tags: [doctrine, enforcement, memory-architecture, gate-quality, measurement, gotcha]
reverify: "git log --format=%h -40 -- MEMORY.md | while read c; do git show $c:MEMORY.md | wc -c; done   # the file sits in a narrow band just under the 6144 B cap, for 40 commits across many sessions — a steady state, not a drift. Then diff consecutive versions for phrases removed and later re-added: 185 removed, 1 re-added."
evidence: docs/tasks/RESUME-POINTER-CONTRACT.md (registered on the hypothesis, closed the same day by its own falsification test); docs/decisions/0040-memory-pointer-byte-cap.md (where the cap comes from); scripts/check_memory_architecture.sh (the cap itself)
---

`MEMORY-ARCH`'s **byte** cap on `MEMORY.md` blocked three commits in one session. The agent
reported that as evidence the doctrine's **remedy** (*route content down to layer B/C, leave a
pointer*) was the wrong one, and registered a tree for it. **The hypothesis was false**, and the
way it was killed is the reusable part.

## First, a factual correction that the wrong reasoning rested on

`MEMORY.md` is **not a rotating log**. It has no entries and no "oldest" to evict. It is a
fixed-shape file — five `Current state` bullets plus standing sections — **rewritten in place**,
with the cap applied to the *whole file's bytes*. So nothing is evicted automatically; when the
file is full, an author decides what leaves. That is why "it fires, so what" deserved a check
rather than a shrug — and also why the check came back clean.

## The hypothesis, and the test it named

> *A pointer is not equivalent to the thing it points at: it converts "the next session will see
> this" into "the next session may look this up". So each session re-inlines the rules it most
> fears being ignored, the cap re-fires, and the same content returns under a new heading. Three
> firings is one loop running three times.*

Plausible, mechanistic, and **falsifiable in one query**: if that is happening, content routed
*out* of `MEMORY.md` must come *back* later.

## The measurement

| over the last **40** commits touching `MEMORY.md` | |
| --- | ---: |
| distinct phrases removed at some point | **185** |
| phrases removed and later **re-added** | **1** |
| file size across those commits | **5,877 – 6,125 B** (95–99.7 % of the 6,144 cap) |

One re-add in 185, and that one a deliberate judgement call. **Routed content stays routed.** The
file is in a **steady state** at its ceiling and has been for dozens of commits across many
sessions — long predating the session that raised the alarm.

## The rule

**A gate that fires often is telling you the constrained thing lives at its limit. That is what a
binding constraint does.** Frequency alone says nothing about whether the remedy is right; it is
a property of *where the subject sits relative to the bound*, not of *what you do when you hit
it*. A file parked at 98 % of its cap will fire on almost every edit, forever, and correctly.

The failure mode to avoid is the inference, not the observation:

- ✅ *"This fired three times"* — an observation, cheap and true.
- ❌ *"…therefore the remedy is wrong"* — an inference that needs its own evidence.

**Before escalating any "this mechanism misfires" claim: write down what observation would prove
you wrong, then go and make that observation.** In this case the test took one query and cost
less than the tree that was registered to hold the untested hypothesis.

## Why this is worth a card

The tree that raised it **had already written the falsification test into its own acceptance
criteria** — *"whether the routed content returned … if it did not return, the loop hypothesis is
wrong and this tree closes"* — and then escalated to the owner without running it. So this is not
"we lacked the test"; it is **specifying the test and skipping it**, which is the same shape as
this repo's catalogued instrument errors ([[cli-flag-audit-must-be-command-scoped]],
[[extractor-charset-narrower-than-source]]) moved up one level: not a wrong *number*, but a wrong
*inference from a number*. A plausible narrative published ahead of the measurement that would
kill it is the recurring defect, whatever layer it happens at.

## When to reopen

Not when the cap fires again — it will, and that is expected. Reopen only on evidence that routed
content **returns**, or that a firing caused something to be **lost**. Neither has ever been
observed. **A concrete numeric trigger is given in the amendment below**, so "when to reopen" is a
measurement rather than a judgement call.

## Amendment `2026-08-07` — the direction of the squeeze, and the runway

Asked by the owner, on the right instinct: *`MEMORY.md` names the next frontier — if the cap can
block updating it, is the pointer itself at risk?* The concern is **the one clause above that says
reopen** ("a firing caused something to be lost"), so it was measured rather than answered.

**It is the opposite direction.** Over the same 40-commit window, re-run at `4925847`:

| | oldest `5e3e9a0` | newest `4925847` |
| --- | ---: | ---: |
| whole file | 5,955 B | 6,064 B |
| the `## Current state` block — *the pointer's actual job* | 2,250 B (37.8 %) | 2,359 B (**38.9 %**) |

**The pointer block grew, absolutely and as a share, while the file stayed flat.** Eviction pressure
lands on everything *else*, which is the design intent. The structural reason it must land that way:
you hit the cap **while writing the new frontier**, so the content under pressure is the oldest
ballast, never the sentence you are in the middle of authoring. The feared failure is
counter-pressured by the authoring path — the same property decision `0045` keyed on.

**Cost of the firing that prompted the question, measured rather than asserted:** the forced trim
dropped exactly three words — `all`, `at`, `exactly` — a rewording. The leaf id, the decision-record
pointer and the whole work queue survived byte-for-byte. **Zero facts lost.**

**The runway, which is the part the card did not previously cover and where the instinct is right.**
The band has crept **5,955 → 6,064 B** across those 40 commits — **+2.7 B/commit** — leaving **79 B**
of headroom. The remaining sections are **already pointers**: *Standing directives* (953 B),
*Operating gotchas* (610 B) and *Validation policy* (421 B) all read *"RECORDED, not summarised
here"* and link out, so there is no third demotion to perform on them. So the compressible surface is
finite and roughly **29 commits** from exhausted, after which the only thing left to compress is the
pointer block itself — and *that* would be the loss this card says to reopen on.

**Trigger, so nobody has to judge:** when a firing can no longer be absorbed by rewording — i.e. the
`## Current state` block has to give up a **leaf id, a decision-record pointer, or a queue entry** —
that is the reopen condition, and it is a `RESUME-POINTER-CONTRACT`-shaped question with genuinely
new evidence.

## Correction `2026-08-07`, same day — this card was cited for a question it does not answer

The amendment above closed *"until then this stays a forecast, and a forecast does not get a tree."*
**That conclusion was wrong, and the way it was reached is the reusable part.**

The owner asked a third time, narrowing to the **blocking** itself. The measurement that answers
*that* question had not been taken, and when taken it showed a defect rather than a forecast:
`COMMIT.md` requires `MEMORY.md` to be amended before **every** commit, **820 of 840 commits
(97.6 %)** touch it, and it sits at **98.7 %** of a hard fail-closed cap. Worse, when the cap fired
the remedy taken was a **prose rewording** — which the gate's own routing hint explicitly forbids
(*"Move content down, do not trim prose"*) — because the prescribed remedy was **exhausted**:
`OVERFLOW-DESTINATION-INSTRUMENTATION.5a`/`.5b` had already demoted every other section to a pure
pointer. Nothing observed the substitution, and it was recorded in a commit message as compliance.
Now owned by [`RESUME-POINTER-COMMIT-PATH-COUPLING`](../tasks/RESUME-POINTER-COMMIT-PATH-COUPLING.md).

**The lesson is about this card's own use, so it belongs here rather than only in that tree.** This
card answers *"does a gate firing often prove its remedy wrong?"* — **no**, and that is still true
and still measured. It was cited to answer *"is a fail-closed cap on the mandatory commit path
safe?"* — a **different question about coupling**, which the card never addressed and its 40-commit
band cannot see. Both citations felt identical from the inside, because both start from *"the cap
fired."*

**A card that answers a neighbouring question is the most expensive kind of wrong answer: it arrives
with evidence attached, so it ends the inquiry instead of starting it.** Before citing a fact card,
check that the question you actually have is in its `answers:` list *as asked* — not merely that the
card's subject and yours share a noun. Retrieval hitting is not retrieval matching.

## One residual, deliberately not inflated

A blocked commit leaves **no trace in git** — only the successful commit lands — so "three
firings" is knowable only from inside the session. That is an observability gap in the
*narration*, not a defect in the gate, and nothing depends on counting them. Named here so the
next reader does not rediscover it and mistake it for a finding.
