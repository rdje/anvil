---
id: defect-class-audit-rules
title: Auditing a defect class — never write *"currently correct"* without measuring it; rank by what one omission **corrupts**, not by list length; and search from the **authoritative set**, never from the shape of the instance you found first
answers:
  - "how do I audit a defect class properly"
  - "how should I rank which duplicated list to fix first"
  - "why is searching for the shape of a known bug not enough"
  - "how severe is a forgotten merge_coverage line"
  - "why can omitting a gate flag produce a gate that cannot fail"
  - "is a long hand-maintained list more dangerous than a short one"
  - "how do I check whether all X really funnel through one place"
date: 2026-07-30
status: current
tags: [audit, defect-class, shadow-list, measurement, method, gotcha, north-star]
evidence: docs/decisions/0033-shadow-enumeration-classification.md (the classification rule and the severity ordering); docs/decisions/0034-one-steering-aware-knob-roll-primitive.md (the second-funnel measurement); docs/tasks/SHADOW-ENUMERATION-SWEEP.md
---

Three rules, all earned by `SHADOW-ENUMERATION-SWEEP`, plus the one that governs new instruments.

**(0) Never write *"currently correct"* without measuring it, and treat a COUNT beside a list as a
shadow OF that list.** The tree opened on *"nothing here is a live bug; correctness merely rests on
diligence."* Measured, **five** omissions had already happened — and all five were **prose**, the
copy nobody greps when adding an entry. A stale fact card is worst of all: it is read *instead of*
re-deriving, so it misinforms rather than omits. Redundant counts are repaired by **deletion**,
never by gating the count too.

**(1) Rank by what one omission CORRUPTS, not by list length — and check the consumer's polarity
before ranking the producer.** `merge_coverage` (149 entries) looked worst until measured: all
merges are monotone and 134 of 149 fields are read as `if !saw_x`, so a forgotten merge produces a
*spurious* gap and the gate **bails loudly**. Its true silent surface is **15**, and fail-safe. The
real severity-3 site was a 15-term `||` chain whose omission leaves a gate that **cannot fail**.

**(2) Search from the AUTHORITATIVE SET, never from the shadow you found first.** Sweeping for the
*shape* of the known instance found 4 copies; sweeping for the *content* found **7**, and the 3
extra were the only ones that had already rotted. A shape-keyed search silently narrows the class
to *structured* copies — prose passes every test too.

**Rule (2) again, second lane:** before claiming *"all X funnel through one place"*, search the
**effect** (the write/record/insert), not the funnel. A second funnel is *defined* by producing the
same effect a different way — and a proof that samples **one** member of a set cannot detect that
the set is **partitioned**.

Full rule, the four hard cases it must reject, and the repair ladder:
[[shadow-enumeration-classification]]. Conduct rules for running the sweep itself:
[[sweep-exemption-past-vs-present-and-recorded-recall]].
