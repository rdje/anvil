---
id: negative-control-must-be-able-to-fail
title: Check that a negative control **can** fail before trusting that it did — three passed on the first try and all three were too weak — and recognise `e3b0c44298fc…` on sight: it is the SHA-256 of the empty string
answers:
  - "how do I know my negative control actually proves anything"
  - "my control passed on the first try is that suspicious"
  - "what is e3b0c44298fc"
  - "two hashes matched but did the comparison compare anything"
  - "why did a byte-identity check pass when the run had errored"
  - "does hashing stdout capture --metrics or --trace output"
date: 2026-07-31
status: current
tags: [testing, control, evidence, gate-quality, hashing, gotcha]
evidence: docs/tasks/COVERAGE-STEERED-GENERATION.md (`.6` — three controls that passed and were all too weak); docs/decisions/0033-shadow-enumeration-classification.md (b) (the both-directions control requirement)
---

**A control that passes on the first attempt deserves suspicion, not relief.** Three did in
`COVERAGE-STEERED-GENERATION.6`, and all three were too weak to fail:

- a `datapath` → `datapathXX` mask that the predicate still **substring**-matched;
- a reshaped row for a category that has three rows, so the other two carried it;
- a prediction that was never actually run.

**Check that the control is capable of failing before you trust that it did.** Break the thing it
guards in a way the predicate must see, and watch it go red.

**Two hashes to recognise on sight.**

- **`e3b0c44298fc…` is the SHA-256 of the empty string.** If both sides of a byte-identity
  comparison "match" at that value, they compared **nothing** — a run errored with stderr
  suppressed and produced no bytes.
- **A comparison that hashes only stdout silently ignores `--metrics` and `--trace`**, which write
  to **stderr**. Two runs can be declared byte-identical while their telemetry differs completely.

This sits alongside [[coverage-check-vacuity]], not instead of it: a control proves the check *can*
fire; the vacuity probe (*delete the subject and re-run*) proves it fires on the **right input**.
Decision [`0033`](../decisions/0033-shadow-enumeration-classification.md) (b) requires both
directions — delete an entry, the guard fails; restore it, the guard passes.
