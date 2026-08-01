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
observed.

## One residual, deliberately not inflated

A blocked commit leaves **no trace in git** — only the successful commit lands — so "three
firings" is knowable only from inside the session. That is an observability gap in the
*narration*, not a defect in the gate, and nothing depends on counting them. Named here so the
next reader does not rediscover it and mistake it for a finding.
