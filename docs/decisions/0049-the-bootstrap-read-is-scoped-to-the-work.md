---
id: bootstrap-read-is-scoped-to-the-work
title: The session-bootstrap read is **scoped to the change in hand**, not to a corpus — a fixed tier invariant of the work (measured 114,689 B, 2.7 % of a context window, budgeted to 256 KiB), a subject tier derived per leaf, and a queried tier that is never loaded; `MEMORY_ARCHITECTURE.md` §5 owns the shape and `SESSION_BOOTSTRAP.md` only binds it to filenames
answers:
  - "what must a fresh session read before it starts working"
  - "do I have to read every live doc in full at session start"
  - "how much of the repo should I read before making a change"
  - "why is the session bootstrap read tiered"
  - "do I need to read CHANGES.md or DEVELOPMENT_NOTES.md at bootstrap"
  - "do I need to read the whole mdBook at session start"
  - "do I have to walk every rust file under src before changing code"
  - "is CODEBASE_ANALYSIS.md a substitute for reading the source"
  - "what is the working read budget for a session"
  - "why do the harness bootstrap pointers and SESSION_BOOTSTRAP.md name different files"
  - "how does SESSION_BOOTSTRAP.md relate to MEMORY_ARCHITECTURE.md section 5"
  - "the bootstrap read is bigger than my context window, what do I do"
date: 2026-08-07
status: accepted
tags: [workflow, bootstrap, session-recovery, memory-architecture, owner-directive, budget, docs, north-star]
evidence: docs/tasks/BOOTSTRAP-READ-CONTRACT.md (`.0`'s 12.7 MB / 3.0x measurement and the owner directive recorded verbatim; `.1`'s tier-by-tier loss measurements); MEMORY_ARCHITECTURE.md §5 (the read path this binds to filenames) and §3 (layer D is queried, not loaded); .claude/settings.json (the `PostCompact` hook that re-injects SESSION_BOOTSTRAP.md verbatim, making the fixed tier a per-compaction cost); docs/decisions/0042-task-tree-index-is-a-mixed-surface.md (416 of 508 leaves re-stated in the index); docs/decisions/0039-sweep-exemption-past-vs-present-and-recorded-recall.md (9 of 13 CODEBASE_ANALYSIS.md per-file claims stale); docs/decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md (the B1/B2/A surface classification and the derived-not-fitted cap method)
reverify: "python3 -c \"import os,glob;b=os.path.getsize;W=4*1024*1024;F=['README.md','MEMORY.md','MEMORY_ARCHITECTURE.md','DOCTRINE_ENFORCEMENT.md','TOOLBOX.md','COMMIT.md','SESSION_BOOTSTRAP.md'];f=sum(map(b,F));L=['README.md','ROADMAP.md','MEMORY.md','CHANGES.md','DEVELOPMENT_NOTES.md','CODEBASE_ANALYSIS.md','USER_GUIDE.md','COMMIT.md','docs/TASK_TREE.md'];l=sum(map(b,L))+sum(map(b,glob.glob('book/src/*.md')));print('fixed tier',f,'B =',round(100*f/W,2),'pct of window,',round(100*f/262144,1),'pct of the 262144 B budget; replaced section-1 mandate',l,'B =',round(l/W,2),'x window')\"   # expect ~120,178 B / 45.8 pct of budget AT THIS COMMIT and rising slowly; the budget was derived from the 114,689 B measured one commit earlier, at 13c9cdf. The replaced mandate exceeds a whole window on its own."
---

# 0049 - BOOTSTRAP-READ-CONTRACT.1: the bootstrap read is scoped to the work

- Date: 2026-08-07
- Status: accepted
- Tree: `BOOTSTRAP-READ-CONTRACT.1` (the design leaf; `.0` supplied the measurement and
  recorded the owner directive)
- Activated by: the owner directive quoted below, answering the tree's central question

## Context

`SESSION_BOOTSTRAP.md` §1 instructed every session to *"Read every live doc, in this order,
**in full**"* and §2 to *"Walk **every** source file under `src/`, every test under
`tests/`, and every example under `examples/`"*, both stated as preconditions on any change.

`.0` measured the mandate rather than complaining about it. Re-measured at `13c9cdf`
(`2026-08-07`), against a ~1 M-token context window taken as **4 MiB = 4,194,304 B**:

| Mandate | bytes | of a window |
| --- | --- | --- |
| §1 live docs (9 files) + `book/src/*.md` | 5,507,558 | **1.31×** |
| §1's *"if any tree is `active`, read the linked file too"* — `docs/tasks/*.md` | 3,252,947 | 0.78× |
| §2 walk every `.rs` under `src/`, `tests/`, `examples/` | 3,933,732 | 0.94× |
| **total** | **12,694,237** | **3.03×** |

**§1 alone exceeds a full context window**, before a task tree is opened, before the source
walk, and before any work. The contract was unsatisfiable by construction, and **68.6 %**
of the §1 mass is `CHANGES.md` + `DEVELOPMENT_NOTES.md`, which are append-only by owner
direction — so it receded further with every commit made under it.

**The owner answered the tree's central question directly** (`2026-08-07`, verbatim):

> *"read whatever is necessary for the new session. The idea is to be knowledgable about
> what need to be worked on, that is, because I find it dangerous to fix something you
> don't understand."*

That reframing is what makes this decision possible. The 3.03× is **not** an argument for a
shorter list: an exhaustive read was never the goal, and a session that read all 12.7 MB
and still did not understand its subject would be doing precisely the dangerous thing the
directive forbids. The enumeration is the wrong **shape**, not the wrong **length**.

## Decision

**The bootstrap read is scoped to the change in hand. Its sufficiency test is understanding
the subject of that change, never coverage of a corpus.** `SESSION_BOOTSTRAP.md` is
rewritten as three tiers:

- **Tier 1 — fixed.** Invariant *of the work*: a session cannot know in advance whether it
  needs them, because they define how to work here at all. `README.md`, `MEMORY.md`,
  `MEMORY_ARCHITECTURE.md`, `DOCTRINE_ENFORCEMENT.md`, `TOOLBOX.md`, `COMMIT.md`, and
  `docs/TASK_TREE.md`'s **rule** sections.
- **Tier 2 — the subject.** Derived per leaf, and deliberately **not a list**: the owning
  task tree in full, its one index row, the source the change touches read at source level,
  the book chapters documenting the concept being changed, and the decision records the leaf
  cites.
- **Tier 3 — queried.** Reached by lookup on a question, never read through: the Knowledge
  Map, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `USER_GUIDE.md`, `ROADMAP.md`,
  `docs/decisions/`, `docs/evidence/`, the remaining book chapters, and the rest of `src/`.

**The reconciliation with `MEMORY_ARCHITECTURE.md` §5 is a merge, not a balance.** §5
already prescribes *"a resume reads A + one unit of B + a few C records — never a
monolith"* — which is this directive expressed as a read path, written down before the
directive was given. Two mechanisms for one job is what `feedback_full_factorization`
forbids, so **§5 owns the shape and `SESSION_BOOTSTRAP.md` owns only the binding to this
repository's filenames**, plus the project-specific sanity checks and the task-tree
doctrine. The rewritten file says so in its own text rather than restating §5's rule in
different words.

### (a) The working budget, derived rather than fitted

The tree's open question demanded a number and warned against the `0036` §(c) / `0040` §(c)
failure of picking one that fits what already exists. Taken the same way those two took
theirs — from what was **demonstrated achievable**, then budgeted with a stated multiple:

- **The denominator is not a window.** A context window is not a budget; a session must
  leave room to work, and it performs a **batch** (`BWFSC`), not one leaf — so the work half
  repeats while the fixed read is amortised.
- **The demonstrated-sufficient fixed read is measured, not posited.** `.0`'s session
  recorded exactly what it had read when it shipped a correct leaf. Its invariant part —
  the Tier 1 set above, minus the tree-specific items — measures **114,689 B, 2.73 %** of a
  4 MiB window at `13c9cdf`.
- **The budget is that, doubled, rounded up to the next binary round number: 256 KiB =
  262,144 B = 6.25 %** of a window. Tier 1 occupied **43.8 %** of it at `13c9cdf`.
- **The test that it is a bound and not a description:** a cap fitted to the present file
  set sits at ~100 % of itself on the day it lands and can only be raised. This one lands
  with **~55 % headroom**, which is the same property `0036` §(c) required of the README caps.

**The circularity is real and is reported rather than absorbed.** `SESSION_BOOTSTRAP.md` is
*itself* in Tier 1, and this decision rewrites it — so the number the budget was derived
from is a number this change moves. Handled by deriving from the **pre-change** state and
then stating the cost: at `13c9cdf` the tier was **114,689 B (2.73 %, 43.8 % of budget)**;
after this commit it is **120,178 B (2.87 %, 45.8 % of budget)** — the file grew **4,277 →
9,709 B** to carry the tiers. That **+4.7 %** is the price of the contract, and quoting only
the smaller figure would be fitting the evidence to the conclusion. The `reverify` line above
expects the **post-landing** number, because that is what a future reader's run will print.

**The number that matters is the complement.** At the budget, **≥ 93.75 %** of the window
stays available for the work — and that is the axis the old contract destroyed, since at
3.03× there was no complement at all.

**The `PostCompact` hook makes this a per-cycle cost, not a per-session one.**
`.claude/settings.json` re-injects `SESSION_BOOTSTRAP.md` verbatim after every compaction
and the file's own text says the protocol is *freshly in force*. So a session that compacts
*N* times pays Tier 1 *N+1* times. At 2.73 % that is sustainable; at 1.31× it is not
survivable even once. **This is the constraint that makes the tiering structural rather
than stylistic**, and it comes from the repository's own configuration, not from taste.

### (b) The tier assignment, with the loss of every demotion measured

Every item §1/§2 mandated, classified. Measurements at `13c9cdf`, `2026-08-07`.

| Item | bytes | Tier | What a session loses, measured |
| --- | --- | --- | --- |
| `README.md` | 10,676 | **fixed** | — it is the canonical-navigation table that routes every other read, and the `README-GROWTH` doctrine already holds it to a landing page |
| `MEMORY.md` | 6,143 | **fixed** | — layer A. The `MEMORY-ARCH` byte cap (6,144) exists *so that* this read is always affordable; the cap and this contract are the same design |
| `COMMIT.md` | 14,746 | **fixed** | — executed by every commit |
| `docs/TASK_TREE.md`, rule sections | 11,466 | **fixed** | — **3.8 %** of that file |
| `docs/TASK_TREE.md`, the 91 index rows | 288,109 | **subject: your row only** | ~0 unique fact. [`0042`](0042-task-tree-index-is-a-mixed-surface.md) measured the frontier column re-stating **416 of 508 leaves (81.9 %)** that the owning tree files already hold in full. Mean row **3,166 B** against **288,109** for all of them |
| `ROADMAP.md` | 185,517 | **queried** | Nothing needed to *execute* a leaf: every tree file carries its own `- Roadmap lane:` field (**84 of 84**). Lost is cross-lane phase status, which is needed when **choosing** the next tree — so it is read at PNT selection, not at bootstrap |
| `CHANGES.md` | 2,644,624 | **queried** | Nothing retrievable. Layer D, which `MEMORY_ARCHITECTURE.md` §3 already classes *queried, not loaded*; **632 of 709** entries carry a `**Landed as:**` hash and **459** name a leaf id in the heading, so `grep` reaches the entry by key |
| `DEVELOPMENT_NOTES.md` | 1,134,982 | **queried** | Same class. With `CHANGES.md` this pair is **68.6 %** of the replaced §1 mass — the two files that made the contract recede monotonically |
| `CODEBASE_ANALYSIS.md` | 292,661 | **queried; also §2's derived substitute** | It names **55 of 55** `src/*.rs` files in **7.44 %** of the source mass (13.4× compression) — so it is a complete **map**. It is **not** a substitute for reading the file you are changing: [`0039`](0039-sweep-exemption-past-vs-present-and-recorded-recall.md) measured **9 of 13** per-file claims stale, every error an under-count. Tier 2 therefore requires the source itself, in full, for what the change touches |
| `USER_GUIDE.md` | 163,444 | **queried** | Nothing a grep for the flag or knob does not return. [`0040`](0040-overflow-destination-classification-and-the-unmeasured-axis.md) classes it **B2** — length tracks the product surface — which is a reference's signature |
| the mdBook, 30 chapters | 752,754 | **split** | The chapters carrying non-reconstructible design context are the three `SESSION_BOOTSTRAP.md` **itself** already named load-bearing in its *"what not to do"* section — `core-idea.md`, `non-goals.md`, `why-not-grammar.md` — which with `SUMMARY.md` cost **20,391 B, 2.7 %** of the book. The rest are subject-scoped through `SUMMARY.md` |
| §2: every `.rs` under `src/`, `tests/`, `examples/` | 3,933,732 | **derived + subject-scoped** | The walk was never achievable at **0.94×** a window. What replaces it is strictly stronger where it counts: the map above for orientation, and the touched files read **in full, at source level**, which the old §2 never actually secured because nobody completed it |

**The book question the tree flagged as sharpest resolved itself from the file's own text.**
`SESSION_BOOTSTRAP.md` mandated all thirty chapters in §1 while, four sections later,
naming exactly three as too load-bearing to edit casually. The file already knew which
chapters were invariant; §1 simply never used that knowledge.

### (c) A second read contract exists, and this decision does not silently absorb it

Measured while classifying: the repository has **two** mandatory-read contracts, and they
were never reconciled with each other.

| Contract | Items | Bytes | Of a window |
| --- | --- | --- | --- |
| the six harness bootstrap pointers (identical bodies) | 7 | 801,434 | 0.19× |
| `SESSION_BOOTSTRAP.md` §1 (replaced here) | 9 files + 30 chapters | 5,507,558 | 1.31× |

**Their intersection is exactly two files** — `README.md` and `MEMORY.md` — and **neither
pointer routes to `SESSION_BOOTSTRAP.md` at all**; it is discoverable only from
`README.md`'s closing line. The pointer list also mandates `KNOWLEDGE_MAP.md`
(**694,469 B**), a *derived index* whose own standard prescribes lookup
(`knowledge-map/KNOWLEDGE_MAP_ARCHITECTURE.md` §7) and states the map may be skipped in
favour of grepping the fact files (§1) — so reading it in full is contraindicated by the
standard that generates it.

This is registered as `BOOTSTRAP-READ-CONTRACT.3` and **not** repaired here. The reason is
scope discipline, stated so it does not read as an oversight: `.1` owns the contract's
*content*, the pointers are six files under a Group-C *identical-body* constraint
(`DOCTRINE_ENFORCEMENT.md` §8) with a hard assertion in
`scripts/check_memory_architecture.sh` that each names `README.md` **and**
`MEMORY_ARCHITECTURE.md`, and `COMMIT.md` requires one completed leaf per commit. The
residue is stated rather than hidden: **until `.3` lands, one of the two contracts is
correct and the other still names a list.**

## The candidates, each with its failure mode

### A — shorten the list ❌ **answers the arithmetic, not the directive**

The obvious reading of a 3.03× overrun. It fails on the owner's own terms: any static list
short enough to fit is, for most leaves, both too much (files the change does not touch) and
too little (the one file it does). A shorter universal list still asserts that comprehension
is a function of the corpus rather than of the subject.

### B — delete or truncate the append-only files ❌ **forbidden, and aimed at the wrong object**

`CHANGES.md` + `DEVELOPMENT_NOTES.md` are **68.6 %** of the §1 mass, so it is the tempting
lever. `0031` forbids rewriting history and the owner has directed both files stay raw. The
defect was never in the files; it was in a contract that demanded they be *read through*
when `MEMORY_ARCHITECTURE.md` §3 already classed them as queried.

### C — delete `SESSION_BOOTSTRAP.md`, leaving `MEMORY_ARCHITECTURE.md` §5 ❌ **loses the binding**

Attractive under `feedback_full_factorization`, and it is the `0033` R1 repair-by-deletion.
Rejected on what it destroys: §5 is deliberately **project-agnostic** (it names no file in
any codebase), so deleting this file would leave nothing that says *which* file is layer A
here, which is layer B, what the sanity checks are, or that the task-tree doctrine is in
force immediately on recovery. It would also empty the `PostCompact` hook. The merge is
therefore by **subordination** — §5 owns the shape, this file owns the binding — not by
deletion.

### D — tier the read, scoped to the work ✅ **chosen**

The only candidate that takes the directive as the design input rather than as pressure to
negotiate the number down. Its cost is stated below: the mandatory tier gets smaller while
the subject tier gets **stricter**, and nothing verifies the latter.

## Honest limits (§9 — stated, not discovered later)

- **Nothing verifies comprehension, and nothing here pretends to.** The sufficiency test is
  *understanding the subject*, which no gate can read. What is observable is the work: a
  change grounded in its subject survives `cargo test`, the doctrine driver and the
  downstream gates. Whether anything should *watch* bootstrap conduct is `.2`'s question,
  and [`0047`](0047-negative-control-carrier-is-the-mutation.md) prefers removing the need
  to watching harder.
- **Tier 2 is stricter than what it replaces, and that is the real trade.** The old §2 asked
  for 3.9 MB and got nothing, because nobody completed it. The new contract asks for the
  touched files in full and will be *felt*. A session that reads less than its subject is
  now visibly out of contract rather than invisibly so.
- **The budget bounds Tier 1 only.** Tiers 2 and 3 are bounded by the subject and by the
  question, and deliberately carry no byte figure — one would be fitted, and would
  re-import the corpus-coverage thinking this decision removes.
- **`SESSION_BOOTSTRAP.md` is a `0040` class-B1 surface** — bounded by a stated contract
  (*what a session must always have*) and paid again on every compaction — so it is a
  legitimate cap subject. **No cap is set here.** Recorded as an input to `.2`, whose
  question that is.
- **The two-contract residue is live until `.3`.** See §(c).
- **The measurements are dated, and the demoted files keep growing.** The percentages above
  describe `13c9cdf`; the *classifications* are the durable part, and they were chosen so
  that growth in an append-only file cannot invalidate them — a queried surface stays
  queried at any size.

## Explicit non-licenses

1. **Not a licence to read less about the change.** The mandatory tier shrank; the subject
   tier tightened. Reading the touched source *in full* is now explicit where §2 only
   implied it inside an unachievable sweep.
2. **Not a licence to shrink, truncate, or reorganise any file.** `0031` stands intact;
   `README_POLICY.md`, `README-GROWTH` and `MEMORY-ARCH` are untouched.
3. **Not a resumption of `OVERFLOW-DESTINATION-INSTRUMENTATION`**, which owns file *size*,
   is **paused**, and must not be resumed without an owner nudge. This decision classifies
   *reads*, never sizes, and sets no cap on any file.
4. **Does not amend `MEMORY_ARCHITECTURE.md` §5** — it binds it. §5 remains the owner of the
   read path's shape, and a future change to §5 governs this file, not the reverse.
5. **Grants no exemption from any doctrine.** The pre-commit driver, the acceptance
   checklist and the task-tree ownership rule apply exactly as before; a smaller read is not
   a smaller gate.
6. **Does not decide `.2`.** Recorded acceptance remains a legitimate outcome there.

## Consequences

- `SESSION_BOOTSTRAP.md` is rewritten to the three tiers, and states its subordination to
  `MEMORY_ARCHITECTURE.md` §5 in its own text.
- `BOOTSTRAP-READ-CONTRACT` grows a leaf **`.3`** for the harness-pointer reconciliation
  measured in §(c).
- `.2` inherits two inputs it did not have: the class-B1 classification of
  `SESSION_BOOTSTRAP.md`, and the observation that the only readable trace of a bootstrap is
  a self-report — which §6.1 disqualifies before the question is even asked.
- The transferable rule, and it outlives this file: **a precondition nobody can satisfy is
  not a high standard, it is an unowned defect** — it converts every session into a silent
  violator and gives the project no signal at all.
