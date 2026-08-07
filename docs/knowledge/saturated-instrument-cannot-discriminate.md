---
id: saturated-instrument-cannot-discriminate
title: An instrument that is **already firing** on your carrier file cannot discriminate — it reports the same verdict whatever you do, so prove it is **silent before the mutation** or move the carrier to a file that still passes
answers:
  - "my proof fires no matter what I do to the file"
  - "is a negative control valid on a file the check already fails"
  - "why did my control fire before I mutated anything"
  - "how do I pick a carrier file for a negative control"
  - "the proof reports DIFF on a file I have not modified yet"
  - "my negative control passed but I am not sure it proved anything"
  - "what is a saturated instrument"
date: 2026-08-07
status: current
tags: [testing, control, gate-quality, instrument, measurement, gotcha]
evidence: TOOLBOX.md §7 ("A control on a file the proof is already firing on proves nothing — the instrument is SATURATED and fires whatever you do"); docs/tasks/BOOK-PARAGRAPH-BLOBS.md (`.3b` and `.3c` each hit this and had to move the carrier to a file that still passed cleanly); docs/decisions/0047-negative-control-carrier-is-the-mutation.md (the three legs a control needs, of which this is the precondition on leg 1)
reverify: "scripts/prove_clauses_unchanged.py --ref HEAD book/src/*.md   # every file must print OK BEFORE any mutation. A file that already prints CLAUSES DIFFER is SATURATED: it will print the same after your sabotage, so it cannot serve as a control carrier. The silence check IS the reverify."
---

A negative control asks *"does the check fire when I break the thing?"* That question is only
answerable if the check was **not already firing**. On a carrier the instrument is already failing,
the post-mutation verdict is identical to the pre-mutation one, and the control reports a confident
PASS having measured nothing.

## Why it is easy to miss

The saturated run looks exactly like a successful control: you mutate, the check fires, you conclude
the check works. The missing step is the cheap one — **read the verdict before you mutate.** Nothing
in the tooling prompts for it, because a check has no way to tell "failing for your reason" from
"failing for a reason that predates you".

Measured in this repository: **both** `BOOK-PARAGRAPH-BLOBS.3b` and `.3c` hit it, and both had to
move the carrier to a file that still passed cleanly before their controls meant anything.

## The rule

**Before reading any verdict from a control, prove the instrument is silent on that file.** If it is
not, the carrier is wrong — pick a file the proof still passes on. Do not "fix" the file to silence
the instrument; that changes the subject mid-experiment.

## Where it sits among the three legs

Decision [[negative-control-carrier-is-the-mutation]] (`0047`) names three legs a control needs: the
check **can** fire, it fires on the **right input**, and **the experiment ran at all**. Saturation is
the precondition on the first: a check that fires unconditionally has not demonstrated that it *can*
fire in the sense that matters. It is the mirror image of the `0047` failure — there the mutation
silently did nothing and the check stayed silent; here the mutation may do nothing and the check
shouts anyway. **Both produce a confident wrong finding, from opposite directions.**

## The transferable form

*A measurement taken at the instrument's rail carries no information.* This is the same shape as
[[coverage-check-vacuity]] (a check that passes with its subject deleted) one step along: there the
instrument is stuck at PASS, here it is stuck at FAIL. Ask of any instrument before trusting it:
**what reading would this give if I did nothing?**
