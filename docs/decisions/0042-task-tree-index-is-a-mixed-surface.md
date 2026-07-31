---
id: task-tree-index-is-a-mixed-surface
title: The task-tree **index** is a mixed surface — its *"Current frontier"* column, a field whose contract is **one value**, re-states **416 of the 508 leaves (81.9 %)** the tree files already own; `0040` §(f)'s *"ANVIL has no mixed surface today"* is **superseded on measurement**
answers:
  - "is docs/TASK_TREE.md a bounded surface or an append-only record"
  - "does ANVIL have a mixed surface"
  - "why is docs/TASK_TREE.md not classified by decision 0040"
  - "what class does the task-tree index belong to"
  - "how do I detect a mixed surface without a distinct-date count"
  - "why does a one-row status edit produce a 37 KB git diff"
  - "what belongs in a task-tree index row and what does not"
  - "where does cross-tree status go when README overflows"
  - "why was the mixed-surface category left with no detector"
date: 2026-07-31
status: accepted
tags: [live-docs, task-tree, classification, mixed-surface, shadow, overflow-destination, doctrine-policy]
reverify: "the frontier-column probe recorded in docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md leaf .8 — for each row, take the leaf IDs the owning docs/tasks/<TREE>.md itself declares as the authoritative set, then count how many the index's Current-frontier cell re-states; it reported 416/508 = 81.9 % with 22 of 74 cells re-stating >=5 of their own tree's leaves"
evidence: "docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md leaves .6 (the tree-wide measurement) and .8 (this classification); docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md §(a) (the three-class scheme), §(f) (the claim this record supersedes) and §(g) (the routing gap, of which this is the second instance); docs/decisions/0036-readme-landing-page-restoration.md (the lossy-copy signature); docs/decisions/0039-sweep-exemption-past-vs-present-and-recorded-recall.md rule (a) (past-tense closure facts inside a present-tense document are correct) and rule (b) (record the match count)"
supersedes: "0040 §(f), in its factual claim only — see §3"
---

# Context

Decision [`0040`](0040-overflow-destination-classification-and-the-unmeasured-axis.md)
classified every overflow destination into three kinds — **B1** bounded by a stated contract,
**B2** bounded in kind but not in size, **A** append-only record — and named a fourth shape it
could not instrument: a **mixed surface**, a bounded status view with an unbounded dated changelog
inside it, whose repair is *separation before instrumentation, never a cap on the mixture*. Its
§(f) then recorded, on the evidence available at the time, that **"ANVIL has no mixed surface
today."**

`0040` §(g) separately noted that `CODEBASE_ANALYSIS.md` is named by **neither** routing
enumeration. `OVERFLOW-DESTINATION-INSTRUMENTATION.6` found a **second** file in exactly that
position — and this record classifies it.

# The measurement

`docs/TASK_TREE.md` is the task-tree **index**. `0040` §(a) classified `docs/tasks/` (class **A**,
append-only layer-B history), but the index is a **different file on a different path**, and unlike
the per-tree files it is **rewritten**: a frontier row changes every time a leaf lands. Class A
cannot be inherited; the classification has to be argued from what the file is *for*.

| measure | value |
| --- | ---: |
| size | **246,172 B** over **376** lines = **654 B/line** |
| for scale — the density that made `MEMORY.md` this tree's central finding | 406 B/line |
| share of the file that is one table | **95 %** |
| the *"Current frontier"* column alone | **219,585** chars, mean **2,967**/cell, max **39,095** |
| leaves declared across the 74 owning `docs/tasks/<TREE>.md` files | **508** |
| **of those, re-stated inside the index's frontier column** | **416 — 81.9 %** |
| status cells re-stating **≥5** of their own tree's leaves | **22 of 74** |
| the worst single cell (`STRUCTURED-EMISSION-EXPANSION`) | **75 of 75** leaves, 60 completion words, in **one cell** |
| git cost of a one-row status edit (5 consecutive real commits, each `+1/−1` **lines**) | **30–42 KB** of diff |

# Decision

**`docs/TASK_TREE.md` is a MIXED surface**, and it is the first one measured in ANVIL.

A column named *"Current frontier"* has a contract that is **one value**: the leaf that is next.
Measured, it carries a **per-leaf journal** — dated completion entries accumulated as each leaf
landed — and that journal is a **lossy copy of layer B**, since **81.9 %** of what it re-states is
already owned, in full, by the `docs/tasks/<TREE>.md` file the same row links to. That is the
`0036` signature (a bounded view accreting a duplicate of a durable layer) inside the exact shape
`0040` named and could not find.

**The repair is therefore separation before instrumentation, never a cap.** Capping the index would
be `0040`'s own rejected move applied to a mixture: it would pressure an author into deleting
recorded facts, and the facts are not the problem — their *location* is. Registered as
`OVERFLOW-DESTINATION-INSTRUMENTATION.9`; **this record classifies, it does not sweep.**

# The classification had to be earned by READING, not by counting dates

`0040` §5 retired the distinct-date instrument because it puts `ROADMAP.md` in the log band on 15
dates that are **all** correct phase-closure facts — `0039` rule (a). That retirement binds here.
So every dated occurrence in all **8** flagged cells was read individually, and the count alone
would have been the wrong verdict:

- **6** are *registration/opening* facts (`Registered 2026-06-17`, `Activated 2026-06-16`) — one
  per tree, and exactly what a status row should carry. Correct.
- **4** are dates **inside quoted evidence** — e.g. `DATED-COUNT-SWEEP-EXEMPTION` quoting the very
  strings it exists to repair (`"(cargo test, 2026-05-02)"`). Citing a date is not being a log.
- **~15 are per-leaf completion entries** — `` `.7` design-leaf done `2026-06-17` ``,
  `` `.11` design done `2026-06-22` `` — and *those* are the journal.

**The date count is not the signal; the accretion of per-leaf entries is.** The instrument that
actually works, and that this record contributes, is the one in `reverify:` above: **take the leaf
IDs the owning tree file declares as the authoritative set, and count how many the status cell
re-states.** It needs no baseline, no cap and no judgement, and it cannot fire on `ROADMAP.md`,
which has no leaf IDs to re-state. It is offered as a **finding, not as a registered doctrine** —
one surface is not a class, and `DOCTRINE_ENFORCEMENT.md` §9 is followed rather than quoted.

# What this supersedes, and what it does not

**Superseded:** `0040` §(f)'s sentence *"ANVIL has no mixed surface today."* That was a factual
claim about a measurement not yet made, and it is now false. Per `MEMORY_ARCHITECTURE.md` §10 the
old record is **not rewritten** — it is superseded here, and the reasoning that produced it stands:
§(f) was right that both *candidate detectors* were disqualified, and this record does not
resurrect either of them. It found the surface with a **third** instrument, which is precisely why
§(f)'s conclusion needed re-testing rather than trusting.

**Not superseded:** `0040`'s three-class scheme, every per-file classification in §(a), the rule
that append-only records are never capped, and the mixed-surface repair rule itself — which this
record does not amend but **confirms against a real instance** for the first time.

**Not licensed by this record:** deleting any recorded fact. The frontier journals are duplicates
of layer B, but *"duplicate"* is a claim about location, and `.9` must prove it per row before
moving anything — the `.5a` lesson that a coarse probe reports *"has a home"* for entries whose only
hits point back at the source.

# Consequences

1. `docs/TASK_TREE.md` is classified **mixed**, pending `.9`'s separation; afterwards the index row
   becomes a **B1** surface (bounded by a stated contract: one row, one frontier) and can carry a
   derived cap, while the journal returns to the class-**A** tree files that already hold it.
2. **The routing gap is closed** — the index is now named beside `docs/tasks/` in both routing
   enumerations. This routes *cross-tree status* to the index's **row-level contract**; it is not a
   licence to append a journal there, which is the defect `.9` separates.
3. `0040` §(g)'s lesson now has **two** instances — `CODEBASE_ANALYSIS.md` and `docs/TASK_TREE.md`,
   both unclassified, both unrouted, both among the worst byte profiles in the repo. Two instances
   make it a **pattern**, and `OVERFLOW-DESTINATION-INSTRUMENTATION.4` carries it into the portable
   policy: **a routing enumeration must be checked against the set of files it can send you to, or
   the busiest destinations are the ones nobody declared.**
4. The general lesson, and the one worth carrying: **a "no mixed surface" finding is only as good
   as the instrument that looked.** `0040` disqualified two detectors honestly and then recorded the
   absence as fact. The surface was there; it needed a third instrument, keyed on what the surface
   *duplicates* rather than on how many dates it holds.
