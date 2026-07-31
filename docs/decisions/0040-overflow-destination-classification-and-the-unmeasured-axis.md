---
id: overflow-destination-classification-and-the-unmeasured-axis
title: Overflow destinations are classified in **three** kinds — bounded-by-contract, bounded-in-kind-but-not-in-size, and append-only record — and the pressure a cap redirects lands not only on a neighbouring **surface** but on the neighbouring **axis of the capped file itself**, which is where `MEMORY.md` is now **1.65× README's byte cap** at exactly 50 of 50 lines
answers:
  - "which live docs may be size-capped and which must never be"
  - "why can USER_GUIDE.md or the book not be given a size cap"
  - "is MEMORY.md within its size limit"
  - "why is a line cap alone not enough"
  - "how do I derive a cap for a live doc instead of fitting it to current size"
  - "what is a bounded-in-kind surface"
  - "does counting distinct dates in a file tell you whether it is a log or a status view"
  - "why is the README routing hint missing CHANGES.md and DEVELOPMENT_NOTES.md"
  - "do ANVIL's two routing tables disagree and is that a bug"
  - "what is a mixed surface and how is it repaired"
  - "where did the growth go after README.md was capped"
  - "how much of MEMORY.md is layer-C content"
date: 2026-07-31
status: accepted
tags: [docs, policy, cap, growth, instrument, memory-architecture, classification, shadow-list, gate-quality, doctrine, north-star]
reverify: "printf '%s %s %s\\n' \"$(wc -l < MEMORY.md)\" \"$(wc -c < MEMORY.md)\" \"$(( $(wc -c < MEMORY.md) / $(wc -l < MEMORY.md) ))\"  -> lines / bytes / bytes-per-line; the line cap is 50 (scripts/check_memory_architecture.sh:10) and README's byte cap is 12288 (scripts/check_readme_growth.sh:52)"
evidence: scripts/check_readme_growth.sh:52 (BYTE_CAP=12288), :61-70 (the routing hint — 7 destinations, all inside note strings); scripts/check_memory_architecture.sh:10 (MEMORY_POINTER_LINE_CAP=50, no byte cap); README.md "## Where content goes" (6 rows); README_POLICY.md:29-32 (the append-only exemption and "USER_GUIDE.md's length is its purpose"); MEMORY_ARCHITECTURE.md §6 (one screen; "if it exceeds the cap, information is in the wrong layer"); docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md (.1's audit at 43aad06, §6 the PGEN mixed-surface measurement); docs/decisions/0036-readme-landing-page-restoration.md §(c) (caps derived, both axes, prose density is the hidden variable); docs/decisions/0031-ssd-volume-exclusivity.md; docs/decisions/0033-shadow-enumeration-classification.md test (2); docs/decisions/0039-sweep-exemption-past-vs-present-and-recorded-recall.md rule (a). All sizes re-measured 2026-07-31 at e4b4fd5.
---

# 0040 - Classify overflow destinations in three kinds, and instrument the axis the cap cannot see

- Date: 2026-07-31
- Status: accepted
- Tree: `OVERFLOW-DESTINATION-INSTRUMENTATION.2` (the decision leaf; `.3` implements, `.4` feeds
  the correction back into the portable policy, `.5` performs the demotion `.3` depends on)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK**
  directive, from the frontier row set by `.1`
- Opened on an **owner finding measured in PGEN** (`2026-07-31`): *"a cap that redirects
  overflow must also check where the overflow lands … a policy-level hole, not a PGEN accident."*

## Context

`.1` confirmed ANVIL inherits the hole in kind — `check_readme_growth.sh` caps `README.md` on
two axes and names its overflow destinations only inside `note` strings, and across the whole
repo exactly **two** files carry any size instrument — while measuring ANVIL's status surfaces
an order of magnitude cleaner than PGEN's (7.8–13.9 % run-log density against 94.7 %). It then
stopped before touching anything, because four of the destinations are append-only by absolute
owner directive and capping them would breach [`0031`](0031-ssd-volume-exclusivity.md).

This leaf classifies, decides the instrument, and — measuring first — finds the pressure in a
place neither `.1` nor the originating finding looked.

Every number below was re-derived at `e4b4fd5`. Per
[`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) rule (b), each sweep
records its key, its authoritative set, and its **match count**.

### 1. The destination set: `.1`'s count was wrong by one, and there are two enumerations of it that disagree

**Sweep 1** — key: the destinations named on the right-hand side of the routing hint's arrows.
Set: `scripts/check_readme_growth.sh:61-70`. **Match count 7**, not the *six* `.1` recorded:

```
USER_GUIDE.md · book/src/ · ROADMAP.md · docs/tasks/ · docs/decisions/ ·
docs/evidence/ · TOOLBOX.md
```

A count beside a list, off by one, **inside the tree auditing where lists go stale** — decision
[`0033`](0033-shadow-enumeration-classification.md)'s own subject recurring. Recorded rather
than silently corrected: `.1` is layer-B history.

**Sweep 2** — key: a structure mapping a content *kind* to a canonical home. Set: the whole
tree. **Match count 2** — the routing hint, and `README.md`'s `## Where content goes` table.
They **disagree**, and the set difference is exact:

| | destinations |
| --- | --- |
| in both | `USER_GUIDE.md` · `book/src/` · `ROADMAP.md` · `docs/tasks/` · `docs/decisions/` · `docs/evidence/` · `TOOLBOX.md` |
| README's table only | **`CHANGES.md`** · **`DEVELOPMENT_NOTES.md`** |
| routing hint only | — |

The two files by which they differ are **exactly** the two that are append-only by `0031`. This
is not drift; it is the classification this leaf must write down, already latent in the repo —
see §(e).

### 2. Measured: the three kinds are three, not two

| destination / surface | lines | bytes | longest line | B/line | instrument |
| --- | ---: | ---: | ---: | ---: | --- |
| `README.md` | 159 | 10,375 | 378 | 65 | **line + byte** |
| `MEMORY.md` | **50** | **20,311** | **2,874** | **406** | **line only** |
| `TOOLBOX.md` | 106 | 10,987 | 1,606 | 104 | none |
| `USER_GUIDE.md` | 2,466 | 148,554 | 512 | 60 | none |
| `ROADMAP.md` | 2,874 | 182,894 | 3,653 | 64 | none |
| `book/src/` | 15,121 | 758,735 | 6,327 | 50 | none |
| `CODEBASE_ANALYSIS.md` | 2,734 | 282,136 | **24,990** | 103 | none |
| `docs/evidence/` | 415 | 18,479 | 366 | 45 | none |
| `docs/decisions/` | 10,376 | 673,282 | 3,841 | 65 | none |
| `docs/tasks/` | 21,489 | 2,751,963 | 33,890 | 128 | none |
| `DEVELOPMENT_NOTES.md` | 16,209 | 1,005,505 | 845 | 62 | none |
| `CHANGES.md` | 44,787 | 2,353,851 | 1,307 | 53 | none |

`.1` posited a binary — *bounded surface* or *append-only record* — and every bounded one was to
carry an instrument. Measured, the bounded side **splits**, and the split is load-bearing:
`README_POLICY.md:32` already states the distinction without naming it — *"`USER_GUIDE.md`'s
length is its purpose."* A reference manual's size tracks the **feature surface**, not a copy of
something else. Capping it is either decorative (raised once per feature, so the number means
nothing) or harmful (it forces deleting content that has nowhere else to go). That is a
different animal from a landing page, whose contract *names* its size.

### 3. The finding: the pressure went somewhere neither the owner's finding nor `.1` looked

`.1` framed the hole as *the pressure relocates to a neighbouring **surface***. Measured, ANVIL's
sharpest instance is one category over: **the pressure relocated to the neighbouring **axis of an
already-instrumented file**.**

`MEMORY.md` carries a **line** cap (50) and **no byte cap**. It now sits at **exactly 50 of 50
lines — 100 % of its cap — and 20,311 bytes, which is 1.65× the byte cap `README.md` is held
to**, in a file whose own first line says *"keep <= 50 lines"* and whose standard
(`MEMORY_ARCHITECTURE.md` §6) says *"roughly one screen"*. Its longest single line is **2,874
characters**.

The history makes the mechanism unambiguous. The 50-line cap was installed at `2d01e8e`
(`MEMORY-ARCHITECTURE-DOC.4`, `2026-06-05`), which truncated the file from a pre-cap peak of
**2,399 lines / 306,099 bytes** to **19 lines / 1,227 bytes / 64 B per line**:

| commit | date | lines | bytes | B/line |
| --- | --- | ---: | ---: | ---: |
| `ae8f3d1` (pre-cap peak) | 2026-05-20 | 2,399 | 306,099 | 127 |
| `5043547` (first post-cap) | 2026-06-05 | 19 | 1,227 | **64** |
| `747d9bf` | 2026-06-24 | 21 | 6,513 | 310 |
| `7a1fc50` | 2026-07-30 | **50** | 13,378 | 267 |
| `f335926` | 2026-07-31 | **50** | 17,390 | 347 |
| `1c9b865` | 2026-07-31 | **50** | 19,905 | 398 |
| `e4b4fd5` (HEAD) | 2026-07-31 | **50** | 20,311 | **406** |

**Lines went 19 → 50 and stopped. Bytes went 1,227 → 20,311 — a 16.6× increase. Density went
64 → 406 B/line — 6.3×.** The cap binds perfectly on the axis it measures, and the growth
continued, undisturbed, on the axis it does not. Against `README.md`'s measured **65 B/line**,
the resume pointer runs **6.2× denser**. This is decision [`0036`](0036-readme-landing-page-restoration.md)
§(c) — *prose density is the hidden variable; a file can sit under the line cap while over the
byte cap* — arriving as a live instance, in ANVIL's **most-read file**: `MEMORY.md` is read at
every session start and re-read after every context compaction.

The recorded honest note: **this leaf's own session contributed 19,905 → 20,311.** The pressure
is structural, not one author's slip, which is exactly the argument `0036` used for a cap over a
review habit.

### 4. And 65 % of it is content that belongs in another layer

Measured by section:

| section of `MEMORY.md` | bytes | share | lines | B/line |
| --- | ---: | ---: | ---: | ---: |
| `## Operating gotchas` | **10,491** | **51.6 %** | 14 | **749** |
| `## Current state` (layer A proper) | 5,454 | 26.9 % | 7 | 779 |
| `## Standing directives` | 2,696 | 13.3 % | 9 | 300 |
| `## Lane invariants` | 915 | 4.5 % | 8 | 114 |
| `## How to resume` + preamble + `## Validation policy` | 755 | 3.7 % | 12 | 63 |

`MEMORY_ARCHITECTURE.md` §3 defines layer **C** as *"durable cross-cutting facts: constraints,
learnings, **conventions**, **preferences**, **environment quirks**, 'tried X, failed because
Y'"*. `## Operating gotchas (earned the hard way — do not relearn)` and `## Standing directives
(owner-set)` are that definition verbatim — **13,187 bytes, 65 % of the resume pointer, is
layer-C content living in layer A** — and the line cap cannot see it because it is written as 23
very long lines. §6 of the same standard states the remedy without ambiguity: *"If it exceeds
the cap, information is in the wrong layer; move it down to B or C."*

### 5. Both candidate instruments `.1` proposed are disqualified — on measurement

**Sweep 3** — `.1` §6 finding 1 proposed *the count of distinct dates inside a surface* as a
cheap derivable test for status-view-versus-log. Key: distinct `YYYY-MM-DD` tokens per surface.
Measured: `TOOLBOX.md` 0 · `README.md` 1 · `USER_GUIDE.md` 1 · `MEMORY.md` 2 · `docs/evidence/`
2 · `CODEBASE_ANALYSIS.md` 6 · `book/src/` 9 · **`ROADMAP.md` 15** · `docs/decisions/` 19 ·
`docs/tasks/` 31 · `DEVELOPMENT_NOTES.md` 31 · `CHANGES.md` 51.

`ROADMAP.md` sits inside the log band, and it is **not** a log. All fifteen of its distinct
dates were read: every one is a phase- or lane-**closure** fact — *"`done` as of `2026-05-16`"*,
*"closed `2026-06-16`"*, *"landed `2026-06-22`"*, *"TREE CLOSED `2026-07-31`"*. `0039` rule (a)
says precisely why the test fails: **a record about the present legitimately contains dated
statements about the past.** A date count cannot separate *"this file is a log"* from *"this
file is the status of a project that has a history"* — so the instrument **cries wolf on the
single most important status surface in the repo**, and a gate that cries wolf gets deleted,
taking its real coverage with it.

**`.1` §6 finding 2's instrument (b)** — a self-declared date disagreeing with the file's newest
content — was measured at `DATED-COUNT-SWEEP-EXEMPTION.3` and is disqualified for this tree too:
**0** subjects across the 108-file live-doc set, and its 75 real subjects are all
`docs/tasks/*.md`, **60** of them `done`/`closed` trees where the field is correctly frozen. See
`0039` §(d).

So the **mixed-surface** category — the one PGEN actually hit — gets **no detector here**, and
that is stated rather than papered over (`DOCTRINE_ENFORCEMENT.md` §9). What it gets is the
*rule*, carried into the portable policy at `.4`.

## Decision

### (a) Three classes, not two — and every destination is classified with its reasoning

> **B1 — bounded by a stated contract.** The document's *purpose* names its size, so a cap is
> **derivable from the contract** rather than fitted to the file. **Caps required, both axes.**
>
> **B2 — bounded in KIND, unbounded in SIZE.** The document has a fixed job but its length
> tracks the product surface it describes. A cap would be raised once per feature (decorative)
> or would force deleting content with nowhere to go (harmful). **No size cap; a mixture rule
> instead.**
>
> **A — append-only record.** The document is a statement about the past and is *supposed* to
> grow without bound. **Never capped** — `0031`, and `0033` **test (2)**: an artifact that is
> supposed to differ from a bound is authoritative, and bounding it destroys the property it
> exists to hold.

| destination / surface | class | reasoning |
| --- | --- | --- |
| `README.md` | **B1** | a *landing page*; `0036` derived 250 / 12,288 from what survived the audit. Already instrumented on both axes. |
| `MEMORY.md` | **B1** | a *resume pointer*, `MEMORY_ARCHITECTURE.md` §6: *"roughly one screen"*. Instrumented on **one** axis only — §3. |
| `USER_GUIDE.md` | **B2** | a CLI reference; `README_POLICY.md:32`: *"its length is its purpose."* Grows one entry per flag. |
| `book/src/` | **B2** | the user-facing narrative; the owner's only window. Grows one chapter per capability. |
| `ROADMAP.md` | **B2** | phase/lane status; grows one lane per directive. 15 dated closure facts, all correct — §5. |
| `TOOLBOX.md` | **B2** | the instrument catalogue; grows one row per instrument. Smallest B2 at 10,987 B. |
| `CODEBASE_ANALYSIS.md` | **B2** | the workspace snapshot; grows with the workspace. **Named by neither routing enumeration** — §(g). |
| `docs/decisions/` | **A** | layer C. One record per durable fact, appended, superseded, never rewritten. |
| `docs/tasks/` | **A** | layer B. Per-unit work memory; `docs/TASK_TREE.md`'s own note: *"task files are layer-B history and are not retro-edited."* |
| `docs/evidence/` | **A** | banked closure digests (`0030`). A digest records what one run reported; it is a fact about the past. |
| `CHANGES.md` | **A** | append-only by `0031`, absolute owner directive. Position is itself a record (`0038`). |
| `DEVELOPMENT_NOTES.md` | **A** | append-only by `0031`, same directive, same reasoning. |

The exclusions are stated **per entry and with the governing doctrine named**, so that an
exclusion can never later read as an oversight — which was `.2`'s explicit acceptance criterion.

### (b) The instrument: one byte cap, on `MEMORY.md`, byte-first

`.1` required the instrument to be **byte-first** (a single `CODEBASE_ANALYSIS.md` line is
2.03× the whole README byte cap, so a line cap on these surfaces would be decorative). Applied
to the classification, exactly one gap remains: **`MEMORY.md` is B1 and carries only a line
cap.** That is the whole instrument this tree adds.

Not a new doctrine, and not a new script: `MEMORY-ARCH` already owns `MEMORY.md`'s size, so the
byte cap is a **second assertion inside `scripts/check_memory_architecture.sh`**. Registering a
second doctrine for one file would be the second-mechanism-for-one-job that
`feedback_full_factorization` forbids, and `.1`'s Non-Goals already ruled out one doctrine per
file.

### (c) The cap is DERIVED from the contract, and the number is stated with its derivation

`0036` §(c)'s discipline: derive from what the surface **legitimately holds**, never from what it
currently weighs, and set it deliberately below any illustrative figure so there is no room to
regrow into.

- The contract is *"roughly one screen"* (`MEMORY_ARCHITECTURE.md` §6) at ≤ 50 lines.
- The **demonstrated-achievable** density for this exact file is **64 B/line** — what it measured
  at `5043547`, the first commit after the architecture landed and the content was correctly
  layered.
- `README.md`, a *larger* contract, runs **65 B/line**; `0036` measured **118 B/line** for a
  dense numbered list, the highest legitimate density it found.
- Budgeting generously at the top of that range — pointer prose is dense — **50 × ~120 =
  6,144 bytes**, which is deliberately **half** README's byte cap, because a resume pointer is a
  strictly smaller contract than a landing page.

> **`MEMORY.md` byte cap: 6,144.**

The file is currently **20,311 bytes — 3.3× the derived cap.** That gap **is the finding**, not
an argument for a bigger number. Per `0036` and `README_POLICY.md`, **raising a cap is not a
fix**: it requires a new decision record stating that the *contract itself* expanded, never an
edit to the constant.

### (d) The repair precedes the gate — and it gets its own leaf

Switching the cap on at HEAD would block every commit, so `.3` cannot land it alone. §4 already
identifies what has to move and where: **65 % of `MEMORY.md` is layer-C content** — `## Operating
gotchas` (10,491 B) and `## Standing directives` (2,696 B) — and `MEMORY_ARCHITECTURE.md` §6
prescribes the remedy verbatim. Demoting those two sections to `docs/decisions/` records and/or
Knowledge Map cards, leaving **pointers** in `MEMORY.md`, brings the file to roughly 7,100 bytes
before any prose tightening.

That is a content change of real consequence — these are the owner's standing directives and the
project's hardest-won gotchas — so it is **not** folded into `.3`. It becomes **`.5`**, ordered
*before* `.3` in the frontier. A defect is only handled if a task-tree leaf owns it.

Two constraints bind `.5` in advance, both from this repo's own record: the demotion is a
**pointer, never a second copy** (`0038` §(b) rejected re-publication for exactly this reason),
and it is a **relocation of live content, not of history** — nothing in `CHANGES.md` or
`DEVELOPMENT_NOTES.md` is touched.

### (e) The two routing enumerations disagree, the disagreement is CORRECT, and that is now recorded

§1 measured the difference as exactly `CHANGES.md` + `DEVELOPMENT_NOTES.md`. Both enumerations
are **authoritative** — `0033` test (2) fails for each — because they answer different questions:

- the **routing hint** fires at an author whose `README.md` just exceeded its cap, and routes
  **new content**. New content may never be routed into an append-only record, so omitting the
  two class-**A** files is *required*, not an oversight.
- **README's table** routes a **reader** asking where content lives. Change history lives in
  `CHANGES.md`, so including it is *required* there.

Recorded because the failure mode is concrete: an editor "harmonising the inconsistency" would
add `CHANGES.md` to the routing hint and thereby instruct authors to append new prose to
append-only history — manufacturing the very `0031` breach the hint exists to avoid.

### (f) The mixed-surface category keeps its name and its repair, and honestly keeps no detector

A **mixed surface** is a file that combines a bounded status view with an unbounded dated log.
It has **no valid cap because of the mixture**, and the mixture is what hides the growth
(PGEN's `LIVE_ACHIEVEMENT_STATUS.md`: 0.24 % document, 99.7 % accreted log). Its repair is
**separation before instrumentation** — never a cap on the mixture.

Both candidate detectors are disqualified on measurement (§5). ANVIL has **no mixed surface
today**: every B2 file's dated content is closure facts inside a present-tense document, which
`0039` rule (a) classifies as correct. The category is therefore carried as a **rule in the
portable policy** (`.4`), not as a gate — `DOCTRINE_ENFORCEMENT.md` §9, stated rather than
quoted.

### (g) One destination is missing from both enumerations, and it is the worst byte offender

`CODEBASE_ANALYSIS.md` appears in **neither** routing enumeration, yet it is a mandatory
bootstrap read (`SESSION_BOOTSTRAP.md` step 3, *"the authoritative snapshot of the workspace"*)
and a per-commit checklist item (`COMMIT.md` §5), and it carries the worst byte profile of any
B2 surface: **282,136 bytes at 103 B/line, with a single line of 24,990 characters — 2.03× the
entire README byte cap.** An author routing "analysis that outgrew the README" has no hint
pointing there, and the file that would absorb it is the least measured.

It is classified **B2** in (a) and therefore takes no cap. Its one 24,990-character line is a
**formatting** defect, not a size defect, and is recorded here for a future leaf rather than
fixed by this tree — `.1`'s Non-Goals forbid a content sweep, and widening a repair beyond its
proven defect is how the `/tmp` sweep damaged `0030`.

## Decisive test applied

*"Would this classification have prevented PGEN's outcome, without breaking anything ANVIL
depends on?"*

Yes on both halves, and the second half is the one that matters. `LIVE_ACHIEVEMENT_STATUS.md`
was a **mixed** surface — a B1-shaped status view with a class-A log inside it — and the rule
*separate before you cap* addresses it directly. Meanwhile the naive form of the owner's finding,
*"cap every destination"*, would have capped `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/tasks/`
and `docs/decisions/` — pressuring authors into the history rewrite `0031` absolutely forbids —
**and** would have capped `USER_GUIDE.md` and `book/src/`, whose length is their purpose. The
three-way classification is what separates the one real gap (`MEMORY.md`'s missing axis) from
nine destinations that must be left alone.

## What this decision does NOT license

1. **It does not license capping any class-A record.** `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
   `docs/tasks/`, `docs/decisions/` and `docs/evidence/` are excluded by `0031` and `0033`
   test (2), per entry, with the doctrine named. No later leaf may cite this record to trim them.
2. **It does not license capping a B2 surface.** Not `USER_GUIDE.md`, not `book/src/`, not
   `ROADMAP.md`, not `TOOLBOX.md`, not `CODEBASE_ANALYSIS.md`. Their size tracks the product.
3. **It does not license raising the `MEMORY.md` byte cap to fit the current file.** The 3.3×
   gap is the finding. Raising a cap requires a new record stating the *contract* expanded.
4. **It does not license deleting anything from `MEMORY.md`.** `.5` **demotes** layer-C content
   to layers B/C and leaves pointers; nothing is lost, and re-publication is forbidden (`0038`).
5. **It does not license "harmonising" the two routing enumerations.** §(e) — their difference is
   the rule.
6. **It does not license reformatting `CODEBASE_ANALYSIS.md`'s long lines under this tree.**
   §(g) — recorded for a future leaf, not swept here.

## Rejected alternatives

- **`.1`'s two-way classification (bounded surface / append-only record).** Rejected on
  measurement — it forces `USER_GUIDE.md`, `book/src/`, `ROADMAP.md`, `TOOLBOX.md` and
  `CODEBASE_ANALYSIS.md` into "bounded ⇒ must carry an instrument", which is exactly where
  *"cap every destination"* becomes harmful. `README_POLICY.md:32` had already stated the missing
  class without naming it.
- **Cap every named destination.** Rejected before `.2` opened, and re-confirmed here: it
  breaches `0031` on four destinations and misclassifies five more.
- **A byte cap on `CODEBASE_ANALYSIS.md`, the largest uninstrumented B2 surface.** Rejected —
  its size is the workspace's size. Its 24,990-character line is a formatting defect and is
  recorded as such (§(g)); capping the file would answer a formatting problem with a content
  restriction.
- **Adopt the distinct-date count as the mixed-surface instrument** (`.1` §6 finding 1).
  Rejected on measurement — §5. It puts `ROADMAP.md` (15 dates, every one a correct closure
  fact) in the log band, so it cries wolf on the repo's principal status surface.
- **Adopt `.1` §6 finding 2's self-declared-date instrument.** Rejected on measurement at
  `DATED-COUNT-SWEEP-EXEMPTION.3` — 0 subjects in the live-doc set (`0039` §(d)).
- **Register a new doctrine for the `MEMORY.md` byte cap.** Rejected — `MEMORY-ARCH` already owns
  that file's size; a second registered mechanism for one job is what
  `feedback_full_factorization` forbids, and `.1`'s Non-Goals ruled out one doctrine per file.
- **Land the byte cap in `.3` without the demotion.** Rejected — the file is 3.3× the derived cap
  today, so the gate would block every commit. `.5` is ordered before `.3` (§(d)).
- **Lower `MEMORY.md`'s line cap instead of adding a byte cap.** Rejected — it is the *wrong
  axis*, demonstrably: the line cap has held at exactly 50 through a 16.6× byte increase. Tightening
  a cap that is already binding perfectly changes nothing about the growth it cannot see.
- **Do nothing, because ANVIL is an order of magnitude cleaner than PGEN.** Rejected — `.1`
  measured that difference and it remains true for the *status surfaces*, but the resume pointer
  is not one of them: it is at **1.65× the byte cap its sibling landing page is held to**, and it
  is the file every session reads first.

## Consequences

- ANVIL gains a **three-way classification** of every overflow destination, with the governing
  doctrine named per entry, so an exclusion can never later read as an oversight.
- The tree's central finding is sharper than the one it opened on: **a cap redirects pressure not
  only to a neighbouring surface but to the neighbouring axis of the capped file itself.** That
  category is new, it is measured, and it is what `.4` must carry into the portable policy —
  *instrument every axis of a bounded surface, or the cap measures the one thing that is not
  growing.*
- **`MEMORY.md` is the one real gap, and it is in the most-read file in the repo.** 50/50 lines
  at 20,311 bytes, 406 B/line against README's 65, with **65 %** of it layer-C content the
  standard says belongs in layers B/C.
- **The tree grows from four leaves to five.** `.5` (demote `MEMORY.md`'s layer-C content to
  pointers) is inserted **before** `.3` (implement the cap), because the gate cannot land while
  the file is 3.3× over.
- Both instruments `.1` proposed are **retired on measurement**, and the mixed-surface category is
  honestly left without a detector — `DOCTRINE_ENFORCEMENT.md` §9 followed rather than quoted.
- Two latent hazards are now recorded rather than left to be discovered: the two routing
  enumerations must **stay** different (§(e)), and `CODEBASE_ANALYSIS.md` is an unnamed
  destination with a 24,990-character line (§(g)).
- Docs-only leaf: no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched.

## Links

- Tree: [`docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md`](../tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  (`.1` audited and registered; this leaf is `.2`; `.5` demotes; `.3` implements; `.4` feeds the
  correction into the portable policy).
- Governing doctrines: [`0031`](0031-ssd-volume-exclusivity.md) (history stays raw — why class A
  is never capped), [`0033`](0033-shadow-enumeration-classification.md) **test (2)** (an artifact
  supposed to differ from a bound is authoritative), [`0036`](0036-readme-landing-page-restoration.md)
  §(c) (caps derived not fitted; both axes; prose density is the hidden variable).
- Method precedent: [`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) — rule
  (a) is why `ROADMAP.md`'s 15 dates do not make it a log, and rule (b) is why every sweep above
  records its match count.
- Standards: `MEMORY_ARCHITECTURE.md` §3 (the four layers) and §6 (*"if it exceeds the cap,
  information is in the wrong layer"* — the basis for `.5`), `DOCTRINE_ENFORCEMENT.md` §9 (honest
  limits — the basis for keeping the mixed-surface category detector-free).
- Origin: the owner's PGEN finding of `2026-07-31`, *"a cap that redirects overflow must also
  check where the overflow lands … a policy-level hole, not a PGEN accident."*
