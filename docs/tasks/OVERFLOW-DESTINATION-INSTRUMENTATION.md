# OVERFLOW-DESTINATION-INSTRUMENTATION: a cap that redirects overflow must check where the overflow lands

## Metadata

- Tree ID: `OVERFLOW-DESTINATION-INSTRUMENTATION`
- Status: `active`
- Roadmap lane: Live-doc hygiene / doctrine-policy completeness
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` **done** — decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md): **three** classes not two, and the pressure went to the **unmeasured AXIS** of `MEMORY.md`; tree grows to five leaves, frontier **`.5`**)
- Owner: repo-local workflow — **opened on an owner finding measured in PGEN**

## Goal

**Owner finding (`2026-07-31`), measured in PGEN and sent back as a lesson:**

> *"A cap that redirects overflow must also check where the overflow lands.
> `check_readme_stability.sh:83` routes 'family status / Done-bar claims' out of
> `README.md` and into `LIVE_ACHIEVEMENT_STATUS.md`. README is capped on two axes; the
> destination had no cap, no staleness check, nothing — and reached 1,547,057 bytes, of
> which 94.7 % was a dated changelog. The cap didn't remove the pressure, it moved it to
> the neighbouring surface with no instrument. Naming four overflow destinations without
> requiring any of them to be bounded is a policy-level hole, not a PGEN accident."*

The owner's closing clause is the load-bearing one: **this is a hole in the shared policy,
not a defect in one project.** ANVIL adopted that policy (`README_POLICY.md`, decision
[`0036`](../decisions/0036-readme-landing-page-restoration.md), doctrine `README-GROWTH`),
so ANVIL inherits the hole by construction unless it is measured and closed here too.

## The audit (`.1`, measured at `43aad06`)

### 1. ANVIL has the identical structural hole

`scripts/check_readme_growth.sh` caps `README.md` on two axes and, on failure, prints a
routing hint naming six canonical destinations. Read directly, those destinations appear
**only inside `note` strings** (lines 63–67) — they are advice, never assertions:

```
  note "    user-facing flags/knobs/examples -> USER_GUIDE.md, book/src/"
  note "    current work, priorities, status  -> ROADMAP.md, docs/tasks/"
  note "    diagnostics and procedures        -> TOOLBOX.md"
```

Measured across every check in the repo, **exactly two files carry a size instrument**:

| file | instrument | source |
| --- | --- | --- |
| `README.md` | line **and** byte cap (250 / 12,288) | `check_readme_growth.sh:89-90` |
| `MEMORY.md` | line cap (50) | `check_memory_architecture.sh:38` |

Every other live doc — including every named overflow destination — is **unmeasured**.

### 2. What the destinations actually weigh

| destination | lines | bytes | instrument | kind |
| --- | ---: | ---: | --- | --- |
| `README.md` | 159 | 10,375 | **line + byte cap** | bounded surface |
| `MEMORY.md` | 50 | 21,406 | **line cap only** | bounded surface |
| `TOOLBOX.md` | 106 | 10,987 | none | bounded surface |
| `USER_GUIDE.md` | 2,466 | 148,554 | none | bounded surface |
| `ROADMAP.md` | 2,874 | 182,894 | none | bounded surface |
| `CODEBASE_ANALYSIS.md` | 2,727 | **281,724** | none | bounded surface |
| `book/src/` | 15,115 | 758,375 | none | bounded surface |
| `docs/decisions/` | 10,049 | 646,803 | none | append-only record |
| `docs/tasks/` | 20,881 | 2,678,491 | none | append-only record |
| `DEVELOPMENT_NOTES.md` | 15,968 | 987,868 | none | **append-only by `0031`** |
| `CHANGES.md` | 44,189 | 2,311,825 | none | **append-only by `0031`** |

### 3. ANVIL is NOT in PGEN's state — measured, and stated rather than assumed

The tempting move is to agree with the finding and act. Measured, ANVIL's status surfaces
are **not** mostly changelog. Run-log / banked-evidence line density
(`anvil-<bank>` · `` `rNN` `` · `saw_*` · `NNN/0` · `coverage_gaps`):

| file | run-log lines | share |
| --- | ---: | ---: |
| `ROADMAP.md` | 400 / 2,874 | **13.9 %** |
| `book/src/architecture.md` | 123 / 892 | **13.8 %** |
| `USER_GUIDE.md` | 218 / 2,466 | **8.8 %** |
| `CODEBASE_ANALYSIS.md` | 212 / 2,727 | **7.8 %** |

Against PGEN's **94.7 %**, ANVIL's surfaces are an order of magnitude cleaner. The hole is
**identical in kind and far milder in degree**, and saying otherwise would be as wrong as
missing it (`PARITY-EXTRACTOR-ARM-SHAPE-GAP` corollary (c): measure whether the guarded
thing actually drifted, separately from whether the guard exists).

### 4. But one measurement is worse than the percentages suggest

Line counts hide the real exposure. The longest **single line** in each surface:

| file | longest line | length |
| --- | ---: | ---: |
| `CODEBASE_ANALYSIS.md` | 2270 | **24,990 chars** |
| `CODEBASE_ANALYSIS.md` | 2456 | 13,472 chars |
| `ROADMAP.md` | 875 | 3,653 chars |

**One line of `CODEBASE_ANALYSIS.md` is 2.03× ANVIL's entire README byte cap** (12,288).
Prose density confirms it: `CODEBASE_ANALYSIS.md` runs **103 B/line** against 64–65 for
`README.md` and `ROADMAP.md`. This is decision `0036` §(c)'s "both caps, because a file can
sit under the line cap while over the byte cap" — at its limit, where a *single line* clears
the byte cap twice over. Any instrument this tree adds must therefore be **byte-first**; a
line cap on these files would be decorative.

### 5. The correction the policy lesson needs — and this is the part PGEN cannot see alone

**"Cap every overflow destination" would be wrong here, and actively harmful.**
`CHANGES.md` (2.3 MB) and `DEVELOPMENT_NOTES.md` (988 KB) are **append-only by absolute
owner directive** (decision [`0031`](../decisions/0031-ssd-volume-exclusivity.md):
*"Keep it raw, keep honest, so that people can follow the whole history"*). They are
*supposed* to grow without bound. A cap on either would pressure authors into precisely the
history rewrite the project forbids — which is decision
[`0033`](../decisions/0033-shadow-enumeration-classification.md) **test (2)** exactly: an
artifact that is *supposed* to differ from the bound is authoritative, and deriving or
bounding it destroys the property it exists to hold. `docs/tasks/` (2.7 MB) and
`docs/decisions/` are layer-B/C records with the same shape.

So the generalisation is **not** "bound every destination". It is:

> **Every overflow destination must be CLASSIFIED — bounded surface or append-only record —
> and every *bounded* one must carry an instrument. A destination that is neither
> instrumented nor deliberately, doctrinally append-only is the hole.**

And PGEN's actual defect is a **third category the policy has no name for**: a file that
**mixes** the two kinds — a bounded status view with an unbounded dated changelog inside it.
Such a file has no valid cap *because of the mixture*, and the mixture is exactly what hides
the growth. `LIVE_ACHIEVEMENT_STATUS.md` at 94.7 % changelog is that category. The repair
for a mixed surface is **separation before instrumentation**, never a cap on the mixture.

### 6. The mixed-surface category, now MEASURED rather than posited (`2026-07-31`)

`.1` named the mixed surface from PGEN's headline number alone. Asked *"is that file even
needed any more?"*, it was measured directly (read-only, no PGEN edit — that repo has its own
task-tree doctrine and an edit from here would breach it). The category survives contact:

| region of `LIVE_ACHIEVEMENT_STATUS.md` | bytes | share | kind |
| --- | ---: | ---: | --- |
| `## Purpose` + `## Status Rules` + `## Update Policy` | 3,743 | **0.24 %** | the actual document |
| 856 dated `Tracker note (…)` entries, *above* `## Purpose` | 376,317 | 24.1 % | append-only changelog |
| `## Live Snapshot` | 1,183,494 | **75.7 %** | accreted history |

Three findings sharpen the rule this tree will write:

1. **"Snapshot" was a misnomer, and measurably so.** `## Live Snapshot` contains **85 distinct
   dates spanning `2026-02-20` → `2026-07-30`**. A snapshot carries one date. **The count of
   distinct dates inside a surface is a cheap, derivable test for whether it is a status view
   or a log** — and it needs no cap, no baseline, and no judgement. `.2` should consider it as
   the instrument, or as a companion to the byte cap.
2. **The file's own freshness field disproved its own liveness.** It declares
   `Last updated: 2026-06-02` while carrying content dated `2026-07-31` — **two months stale
   and contradicted by its own body**. A self-declared staleness marker that disagrees with the
   file's newest content is a *self-refuting* signal, and it is derivable without any external
   baseline. This is the staleness check the owner's finding said the destination lacked.
3. **It is a lossy copy, on the `0036` signature.** A token probe over `## Live Snapshot`:
   **3,834** distinct backticked tokens, **3,769 (98.3 %)** already present in
   `docs/tasks/` · `docs/decisions/` · `docs/knowledge/` · `KNOWLEDGE_MAP.md` · `CHANGES.md`;
   the **65 (1.6 %)** residue is **composites of covered parts and run noise** — `265/2393`,
   `1584/1560`, `ast_based_generator.rs:2692/4903/7108`, `17061ms`, `1b094142..cfb268ff` — with
   **no orphaned fact**. That is exactly the residue shape `0036` predicts for a *working*
   sweep, and it is the evidence that the durable layers already hold the content.

**And the deletability lesson, which is the reusable half.** By reference count the file looks
load-bearing: 4 check scripts, 36 live docs, 57 task files, 6 decisions. Read directly, only
**one** is a content consumer (`audit_done_bar.sh`, which parses it to audit `Done` claims) —
and that one is already env-parameterized (`PGEN_DONE_BAR_TRACKER`) with its claim already
migrated by that repo's `LIVE-MEANS-LIVE.1a`. The other three are a doc-path glob, a comment,
and **routing-hint text** — i.e. the very hole this tree exists to close, counted as a
dependency. **Reference count is not a dependency measure; it inflates with hint text and with
append-only history that must keep its references raw.** `.2` and `.3` must classify referents
by *what they require*, never by how many there are.

## Why it matters

`README-GROWTH` exists because growth was *structural* — the workflow asked every new knob
for a README bullet. Decision `0036` fixed the symptom at one file. If the pressure simply
relocates to an unmeasured neighbour, the doctrine has moved the problem rather than solved
it, and the next audit finds a 1.5 MB file instead of a 1,771-line one. The owner measured
that outcome in a sibling project; ANVIL is one workflow-habit away from it.

## Non-Goals

- **Not "add a cap to every live doc."** §5. Capping `CHANGES.md` / `DEVELOPMENT_NOTES.md` /
  `docs/tasks/` / `docs/decisions/` would breach `0031` and fail `0033` test (2).
- **Not a content sweep.** Nothing in `ROADMAP.md` or `CODEBASE_ANALYSIS.md` is deleted by
  this tree. `.2` decides the classification and the instrument; any trimming is a separate,
  separately-justified leaf.
- **Not a PGEN change.** ANVIL does not edit a sibling repository. `.4` feeds the corrected
  rule back into the **portable** policy documents, which is where the owner located the hole.
- **Not a new doctrine per file.** One instrument over a declared, classified table — the
  `ENUMERATION-PARITY` shape — or nothing.

## Acceptance Criteria

- Every destination named by `check_readme_growth.sh`'s routing hint is **classified** in a
  recorded decision as *bounded surface* or *append-only record*, with the `0031` / `0033`
  test (2) reasoning stated per entry.
- Every destination classified *bounded* either carries an instrument or has a recorded
  reason why not. **Byte-first**, per §4.
- The **mixed-surface** category is named in the policy, with *separate before you cap* as
  its stated repair — this is the reusable half of the owner's finding.
- The correction lands in the **portable** documents (`README_POLICY.md`,
  `DOCTRINE_ENFORCEMENT.md`), not only in ANVIL's local instance, because the owner
  identified it as policy-level.
- Any check written obeys `DOCTRINE_ENFORCEMENT.md` §4, is negative-controlled both ways,
  and is vacuity-probed per decision `0037` (delete the subject; the check must fail).
- `scripts/check_doctrines.sh` green; docs/scripts-only ⇒ DUT byte-identical.

## Task Tree

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION`
  Status: `active`
  Goal: `Close the policy-level hole the owner measured in PGEN: a cap that redirects overflow must require its destinations to be bounded or to be deliberately unbounded records, and must name the mixed-surface category that has no valid cap at all.`
  Children: `.1` (audit + register), `.2` (classify + decide the instrument), `.5` (demote MEMORY.md's layer-C content — ordered BEFORE `.3`, which cannot land while the file is 3.3x over the derived cap), `.3` (apply the instrument), `.4` (feed the correction back into the portable policy)

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.1`
  Status: `done`
  Goal: `Measure whether ANVIL inherits the hole the owner found in PGEN, and register the tree before anything is changed, per the standing directive that a defect is only handled if a task-tree owns it.`
  Acceptance: `The routing hint's destinations read directly from the check script; every live doc measured in BOTH lines and bytes; run-log density measured per surface rather than asserted; the append-only destinations identified and excluded from any future cap with the governing doctrine named; no repair attempted in this leaf.`
  Verification: `done — MEASURED at 43aad06. (1) HOLE CONFIRMED, IDENTICAL IN KIND: scripts/check_readme_growth.sh caps README.md on two axes and names six destinations at lines 63-67 — all inside `note` strings, i.e. advice, never assertions. Across every check in the repo exactly TWO files carry a size instrument: README.md (line+byte, check_readme_growth.sh:89-90) and MEMORY.md (line only, check_memory_architecture.sh:38). Every named destination is unmeasured. (2) SEVERITY IS NOT PGEN'S, and this is stated rather than assumed: run-log/banked-evidence density is ROADMAP.md 400/2874 = 13.9%, book/src/architecture.md 123/892 = 13.8%, USER_GUIDE.md 218/2466 = 8.8%, CODEBASE_ANALYSIS.md 212/2727 = 7.8% — against PGEN's 94.7%, an order of magnitude cleaner. Claiming parity would have been as wrong as missing the hole. (3) BUT LINE COUNTS HIDE THE EXPOSURE: the longest single line in CODEBASE_ANALYSIS.md is 24,990 chars — 2.03x ANVIL'S ENTIRE README BYTE CAP (12,288) — with a second at 13,472; ROADMAP.md's longest is 3,653. Density: CODEBASE_ANALYSIS.md 103 B/line vs 64-65 for README.md/ROADMAP.md. Any instrument must be BYTE-FIRST; a line cap on these files would be decorative. This is decision 0036 §(c) at its limit. (4) THE POLICY LESSON NEEDS A CORRECTION ANVIL CAN SEE AND PGEN CANNOT: "cap every destination" is WRONG here — CHANGES.md (2,311,825 B) and DEVELOPMENT_NOTES.md (987,868 B) are append-only by absolute owner directive (0031), and docs/tasks/ (2,678,491 B) + docs/decisions/ (646,803 B) are layer-B/C records; capping any of them would pressure authors into the history rewrite 0031 forbids, which is decision 0033 TEST (2) exactly. The rule is therefore CLASSIFY, then instrument only the bounded ones. (5) A THIRD CATEGORY IS NAMED: a MIXED surface (bounded status view + unbounded dated changelog in one file) has no valid cap BECAUSE of the mixture, and the mixture is what hides the growth — PGEN's LIVE_ACHIEVEMENT_STATUS.md at 94.7% changelog is that category, and its repair is SEPARATION BEFORE INSTRUMENTATION. NO REPAIR ATTEMPTED, deliberately: which destinations are bounded, and what instrument each gets, is .2's decision. Checks: check_doctrines.sh green after git add. Docs-only ⇒ DUT byte-identical.`
  Commit: `bb8a835` — `OVERFLOW-DESTINATION-INSTRUMENTATION.1 — audit + register: the cap moved the pressure`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.2`
  Status: `done`
  Goal: `Classify every overflow destination as bounded surface or append-only record, decide which bounded ones get an instrument and of what shape, and record it — including the mixed-surface category and its separate-before-you-cap repair.`
  Acceptance: `Each destination classified with its reasoning, applying decision 0033 test (2) and decision 0031 explicitly per entry — an append-only record MUST be excluded and MUST say why, so the exclusion cannot later read as an oversight. The instrument is BYTE-FIRST (.1 §4: a single 24,990-char line clears the README byte cap twice over). Caps are DERIVED from what the surface legitimately holds, per decision 0036 §(c), never chosen to fit current size — and the decision must state that raising a cap requires a new record, not an edit to the check. One check over a declared classified table (the ENUMERATION-PARITY shape), not one doctrine per file. The decision must state what it does NOT license, so it cannot be cited to justify trimming history.`
  Verification: `done — DECIDED as decision 0040, and MEASURING FIRST CHANGED BOTH THE CLASSIFICATION AND THE TARGET. (1) THE CLASSIFICATION IS THREE CLASSES, NOT TWO. .1 posited bounded-surface vs append-only-record; measured, the bounded side SPLITS, and README_POLICY.md:32 had already stated the missing class without naming it — "USER_GUIDE.md's length is its purpose". B1 = bounded BY A STATED CONTRACT, so the cap is derivable from what the doc is FOR (README.md, MEMORY.md) => caps required on BOTH axes. B2 = bounded in KIND but unbounded in SIZE, because length tracks the product surface (USER_GUIDE.md, book/src/, ROADMAP.md, TOOLBOX.md, CODEBASE_ANALYSIS.md) => NO size cap, since a cap there is either raised once per feature (decorative) or forces deleting content with nowhere to go (harmful). A = append-only record (CHANGES.md, DEVELOPMENT_NOTES.md, docs/tasks/, docs/decisions/, docs/evidence/) => NEVER capped, 0031 + 0033 test (2), stated PER ENTRY with the doctrine named so an exclusion can never later read as an oversight. .1's binary would have forced five B2 surfaces into "bounded => must be instrumented", which is exactly where "cap every destination" turns harmful. (2) THE REAL FINDING IS ONE CATEGORY BEYOND WHAT THE TREE OPENED ON. .1 framed the hole as "the pressure relocates to a neighbouring SURFACE". Measured, ANVIL's sharpest instance is that the pressure relocated to the neighbouring AXIS OF AN ALREADY-INSTRUMENTED FILE: MEMORY.md carries a LINE cap (50) and NO byte cap, and now sits at EXACTLY 50 of 50 lines — 100% of its cap — at 20,311 BYTES, which is 1.65x the byte cap README.md is held to (12,288), with a longest line of 2,874 chars, in a file whose own first line says "keep <= 50 lines" and whose standard (MEMORY_ARCHITECTURE.md §6) says "roughly one screen". THE HISTORY MAKES THE MECHANISM UNAMBIGUOUS: the 50-line cap was installed at 2d01e8e (MEMORY-ARCHITECTURE-DOC.4, 2026-06-05), truncating the file from a pre-cap peak of 2,399 lines / 306,099 B to 19 lines / 1,227 B / 64 B-per-line; since then LINES went 19 -> 50 AND STOPPED while BYTES went 1,227 -> 20,311 (16.6x) and DENSITY went 64 -> 406 B/line (6.3x), against README's measured 65. The cap binds perfectly on the axis it measures and the growth continued undisturbed on the axis it does not — decision 0036 §(c)'s "prose density is the hidden variable" as a live instance, in the file every session reads first and re-reads after every compaction. Recorded honestly: THIS SESSION contributed 19,905 -> 20,311, which is the argument for a cap over a review habit. (3) AND 65% OF IT IS IN THE WRONG LAYER. Measured by section: ## Operating gotchas 10,491 B (51.6%) in 14 lines = 749 B/line; ## Standing directives 2,696 B (13.3%); layer-A proper (## How to resume + ## Current state) only 5,801 B (28.6%). MEMORY_ARCHITECTURE.md §3 defines layer C as "constraints, learnings, conventions, preferences, environment quirks" — those two sections verbatim — and §6 prescribes the remedy without ambiguity: "if it exceeds the cap, information is in the wrong layer; move it down to B or C". (4) BOTH INSTRUMENTS .1 PROPOSED ARE DISQUALIFIED ON MEASUREMENT. Instrument #1 (distinct-date count as a status-view-vs-log test) puts ROADMAP.md in the log band at 15 dates — and all 15 were READ: every one is a phase/lane CLOSURE fact ("done as of 2026-05-16", "closed 2026-06-16", "landed 2026-06-22", "TREE CLOSED 2026-07-31"), i.e. correct past-tense statements inside a present-tense status document, which is decision 0039 rule (a) exactly. It therefore CRIES WOLF ON THE REPO'S PRINCIPAL STATUS SURFACE, and a gate that cries wolf gets deleted. Instrument (b) (self-declared date vs newest content) was disqualified at DATED-COUNT-SWEEP-EXEMPTION.3: 0 subjects across the 108-file live-doc set. So the MIXED-SURFACE category is honestly left with NO detector (DOCTRINE_ENFORCEMENT.md §9 followed, not quoted) and keeps only its rule — separate before you cap — carried into the portable policy at .4. ANVIL has no mixed surface today. (5) THE CAP IS DERIVED, NOT FITTED: contract "roughly one screen" at <= 50 lines; demonstrated-achievable density for THIS file is 64 B/line (5043547, the first commit after the architecture landed); README, a LARGER contract, runs 65; 0036 measured 118 as the highest legitimate density. Budgeting generously at ~120 B/line x 50 = 6,144 BYTES, deliberately HALF README's byte cap because a resume pointer is a strictly smaller contract than a landing page. The file is currently 3.3x that — WHICH IS THE FINDING, NOT AN ARGUMENT FOR A BIGGER NUMBER; raising a cap requires a new record stating the contract expanded. (6) THE TREE GROWS TO FIVE LEAVES: the gate cannot land at HEAD without blocking every commit, so a NEW .5 (demote MEMORY.md's layer-C content to pointers) is inserted BEFORE .3, since a defect is only handled if a task-tree leaf owns it. Not folded into .3 deliberately — these are the owner's standing directives and the project's hardest-won gotchas. NO NEW DOCTRINE: MEMORY-ARCH already owns MEMORY.md's size, so the byte cap is a second assertion inside check_memory_architecture.sh; a second registered mechanism for one job is what feedback_full_factorization forbids. TWO LATENT HAZARDS RECORDED RATHER THAN LEFT TO BE DISCOVERED: (i) the repo has TWO routing enumerations (the check script's hint, 7 destinations; README's "Where content goes" table, 6 rows) and they DISAGREE by exactly CHANGES.md + DEVELOPMENT_NOTES.md — the two files append-only by 0031 — which is CORRECT and must stay, because the hint routes NEW content (which may never enter an append-only record) while README's table routes a READER (and change history does live in CHANGES.md); an editor "harmonising" them would instruct authors to append to history; (ii) CODEBASE_ANALYSIS.md is named by NEITHER enumeration yet is a mandatory bootstrap read and the worst byte profile of any B2 surface (282,136 B, 103 B/line, one line of 24,990 chars = 2.03x the entire README byte cap) — classified B2 so it takes no cap, its long line recorded as a FORMATTING defect for a future leaf rather than swept here. ALSO CORRECTED: .1 recorded "six canonical destinations"; enumerated mechanically the routing hint names SEVEN — a count beside a list, off by one, inside the tree auditing where lists go stale. Recorded, not silently fixed: .1 is layer-B history. Per decision 0039 rule (b) every sweep above records its key, its authoritative set and its match count. Checks: cargo check --all-targets clean; check_knowledge_map OK; mdbook build clean; scripts/check_doctrines.sh 8/8 after git add. Docs-only => DUT byte-identical.`
  Commit: `4a91c45` — `OVERFLOW-DESTINATION-INSTRUMENTATION.2 — the unmeasured axis`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.5`
  Status: `pending`
  Goal: `Demote MEMORY.md's layer-C content (## Operating gotchas, ## Standing directives — 13,187 B, 65% of the resume pointer) into layers B/C, leaving POINTERS behind, so the derived 6,144-byte cap .3 installs is reachable rather than commit-blocking.`
  Acceptance: `Every gotcha and every standing directive lands in a durable layer-C home (a docs/decisions/ record and/or a Knowledge Map fact card, question-keyed so it is retrieved rather than re-derived) BEFORE anything is removed from MEMORY.md, and MEMORY.md keeps a POINTER to each — never a second copy (decision 0038 §(b) rejected re-publication for exactly this reason), and never a deletion. NOTHING IS LOST: prove it the way decision 0036 §(iii) requires, by sweeping EVERY token in the removed range rather than a hand-picked phrase list, and reporting the residue — a working sweep's residue is made only of composites of covered parts. The owner's STANDING DIRECTIVES are owner-set content: they may be relocated and pointed at, never reworded, and the relocation must preserve their owner+date provenance (README-POLICY-PROVENANCE.1: cite by owner+date, never by a harness file). Measure the resulting byte count and state it against the 6,144 cap; if it does not fit, report the gap rather than raising the cap (decision 0040 non-license 3). No history is touched — CHANGES.md and DEVELOPMENT_NOTES.md are out of scope entirely.`
  Verification: `pending`
  Commit: `pending`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.3`
  Status: `pending`
  Goal: `Apply .2: implement the instrument, register it if it is a new doctrine, and prove it works in both directions.`
  Acceptance: `Negative-controlled both ways (breach the cap ⇒ fail; restore ⇒ pass) and vacuity-probed per decision 0037 — and per the standing gotcha, CHECK THAT THE CONTROL CAN FAIL before trusting that it did (three controls passed on the first try in CSG.6 and all three were too weak). Fails with a routing hint of its own, and that hint must not create the same hole one layer down — if it names a further destination, that destination is classified too, or the hint names no destination at all. check_doctrines.sh green. Per decision 0040 §(b) this is NOT a new doctrine: MEMORY-ARCH already owns MEMORY.md's size, so the byte cap is a second assertion inside scripts/check_memory_architecture.sh (BYTE cap 6,144, derived at 0040 §(c)) — a second registered mechanism for one job is what feedback_full_factorization forbids. Blocked on .5: the file is 3.3x the derived cap at HEAD, so landing the assertion first would block every commit.`
  Verification: `pending`
  Commit: `pending`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.4`
  Status: `pending`
  Goal: `Feed the correction back into the PORTABLE policy, since the owner located the hole at policy level rather than in one project.`
  Acceptance: `README_POLICY.md and/or DOCTRINE_ENFORCEMENT.md gain the rule in project-neutral language: a policy that redirects overflow must require each destination to be classified and each bounded one instrumented, and must name the mixed-surface category whose repair is separation before instrumentation. Neutrality is verified the way README-POLICY-PROVENANCE.1 verified it — the policy BODY must contain zero project-specific and zero harness-specific tokens, with any ANVIL-specific content fenced in an adoption note. Provenance cited by OWNER + DATE + a repo-side durable home, never a harness file (README-POLICY-PROVENANCE.1). Credit the measurement to the owner's PGEN finding of 2026-07-31, since a rule is more re-checkable when its evidence is named.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `done` | Audited and registered. Hole confirmed identical in kind (six destinations named only in `note` strings; exactly two instrumented files repo-wide), severity measured and found **far milder** than PGEN's (7.8–13.9 % run-log density vs 94.7 %), but the byte exposure is real — one `CODEBASE_ANALYSIS.md` line is **2.03×** the whole README byte cap. |
| 2 | `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `done` | Classified as decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md), and measuring first changed **both** the classification and the target. The classes are **three**, not two — `.1`'s binary would have forced five *bounded-in-kind-but-not-in-size* surfaces (`USER_GUIDE.md`, `book/src/`, `ROADMAP.md`, `TOOLBOX.md`, `CODEBASE_ANALYSIS.md`) into "bounded ⇒ must be instrumented", which is exactly where *"cap every destination"* turns harmful. And the pressure went **one category beyond** what the tree opened on: not to a neighbouring *surface* but to the **unmeasured axis of an already-capped file** — `MEMORY.md` sits at **exactly 50 of 50 lines** and **20,311 bytes = 1.65× the byte cap `README.md` is held to**, having gone **19 → 50 lines (capped) while bytes went 1,227 → 20,311 (16.6×)** since the line cap landed. Both instruments `.1` proposed are **retired on measurement**. |
| 3 | `OVERFLOW-DESTINATION-INSTRUMENTATION.5` | `pending` | **Next.** Demote `MEMORY.md`'s layer-C content — **65 %** of the resume pointer (`## Operating gotchas` 10,491 B in 14 lines = **749 B/line**, `## Standing directives` 2,696 B) — into layers B/C behind **pointers**. Ordered **before** `.3` because the gate cannot land while the file is **3.3×** the derived cap: switching it on at HEAD would block every commit. `MEMORY_ARCHITECTURE.md` §6 prescribes exactly this: *"if it exceeds the cap, information is in the wrong layer; move it down to B or C."* |
| 4 | `OVERFLOW-DESTINATION-INSTRUMENTATION.3` | `pending` | Apply the derived **6,144-byte** cap as a second assertion inside `check_memory_architecture.sh` (not a new doctrine — `MEMORY-ARCH` already owns that file's size). Control proven capable of failing before it is trusted. |
| 5 | `OVERFLOW-DESTINATION-INSTRUMENTATION.4` | `pending` | Push the corrected rule back into the portable policy — the owner's point was that this is a policy hole, so fixing only ANVIL leaves every other adopter exposed. `.2` sharpened what must travel: not just *classify your destinations* but **instrument every axis of a bounded surface, or the cap measures the one thing that is not growing.** |

## Decisions

- `2026-07-31` (`.1`): Registered as its own tree rather than folded into
  `README-POLICY-PROVENANCE` (closed) or handled inline. The subject differs — that tree was
  about a doctrine's *provenance*, this one is about a doctrine's *blast radius* — and the
  standing directive makes registration mandatory rather than optional.
- `2026-07-31` (`.1`): **Severity is reported as measured, not as received.** The owner's
  finding is correct in kind and ANVIL genuinely inherits the hole; ANVIL's surfaces are
  nonetheless an order of magnitude cleaner than PGEN's. Recording the difference is what
  keeps the tree honest — decision `0033` rule 0: never write "currently correct" without
  measuring it, and the converse holds too.
- `2026-07-31` (`.1`): **The instrument will be byte-first.** A single 24,990-character line
  clears the README byte cap 2.03× over; a line cap on these surfaces would pass while the
  file doubled.
- `2026-07-31` (`.1`): **"Cap every destination" is rejected before `.2` opens.** Four
  destinations are append-only by doctrine (`0031`) and satisfy `0033` test (2) — they are
  *supposed* to be unbounded. The rule is classify-then-instrument.
- `2026-07-31` (`.2`): **Three classes, not two — and `README_POLICY.md` had already said so
  without naming it.** `.1`'s binary (bounded surface / append-only record) puts
  `USER_GUIDE.md`, `book/src/`, `ROADMAP.md`, `TOOLBOX.md` and `CODEBASE_ANALYSIS.md` in
  "bounded ⇒ must carry an instrument". They are **bounded in kind, unbounded in size**: their
  length tracks the *product surface*, so a cap is raised once per feature (decorative) or forces
  deleting content with nowhere to go (harmful). `README_POLICY.md:32` states it exactly —
  *"`USER_GUIDE.md`'s length is its purpose."*
- `2026-07-31` (`.2`): **The tree's central finding moved one category out, on measurement.**
  `.1` framed the hole as *the pressure relocates to a neighbouring surface*. The sharpest
  ANVIL instance is that it relocated to the **neighbouring axis of the capped file itself**:
  `MEMORY.md` has a line cap and no byte cap, and since the line cap landed (`2d01e8e`,
  `2026-06-05`) **lines went 19 → 50 and stopped while bytes went 1,227 → 20,311 (16.6×) and
  density 64 → 406 B/line (6.3×)**. The cap binds perfectly on the axis it measures. This is
  what `.4` must carry into the portable policy, and it is not what the tree opened on.
- `2026-07-31` (`.2`): **Both of `.1`'s candidate instruments are retired on measurement, not
  on argument.** The distinct-date test puts `ROADMAP.md` in the log band at 15 dates — all
  fifteen read, all phase/lane **closure** facts, i.e. correct past-tense statements inside a
  present-tense document (`0039` rule (a)) — so it cries wolf on the repo's principal status
  surface. The self-declared-date instrument has **0** subjects in the live-doc set
  (`DATED-COUNT-SWEEP-EXEMPTION.3`). The mixed-surface category therefore keeps its **rule** and
  honestly gets **no detector**.
- `2026-07-31` (`.2`): **The tree grows to five leaves; `.5` is ordered before `.3`.** The
  derived cap is **6,144 bytes** and the file is **3.3×** that today, so landing the assertion
  first would block every commit. The demotion is a content change of consequence — the owner's
  standing directives and the project's hardest-won gotchas — so it gets its own leaf rather
  than riding inside the implementation leaf.
- `2026-07-31` (`.2`): **The two routing enumerations must STAY different.** They disagree by
  exactly `CHANGES.md` + `DEVELOPMENT_NOTES.md` — the two files append-only by `0031` — because
  the hint routes **new content** (which may never enter an append-only record) while README's
  table routes a **reader** (and change history does live in `CHANGES.md`). Recorded because
  "harmonising the inconsistency" would instruct authors to append new prose to history.

## Blockers

- None. `.5` is next and needs only `0040`'s measurement. `.3` is **blocked on `.5`** by design,
  not by an external dependency: the instrument it installs cannot pass at HEAD.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `Re-measured at e4b4fd5, every sweep recording its key, authoritative set and MATCH COUNT per decision 0039 rule (b). Sweep 1 (routing-hint destinations, set = check_readme_growth.sh:61-70): 7 — .1 recorded six. Sweep 2 (structures mapping content KIND -> canonical home, set = whole tree): 2, and they DISAGREE by exactly CHANGES.md + DEVELOPMENT_NOTES.md, the two files append-only by 0031. Sweep 3 (lines/bytes/longest-line/distinct-dates per surface, 12 surfaces): README 159/10,375/378/65 B-per-line; MEMORY 50/20,311/2,874/406; TOOLBOX 106/10,987/1,606; USER_GUIDE 2,466/148,554/512; ROADMAP 2,874/182,894/3,653; book/src 15,121/758,735/6,327; CODEBASE_ANALYSIS 2,734/282,136/24,990; docs/evidence 415/18,479; docs/decisions 10,376/673,282; docs/tasks 21,489/2,751,963; DEVELOPMENT_NOTES 16,209/1,005,505; CHANGES 44,787/2,353,851. Sweep 4 (MEMORY.md git history, 55 sampled revisions): cap installed 2d01e8e 2026-06-05 truncating 2,399 lines/306,099 B -> 19/1,227/64 B-per-line; since then lines 19 -> 50 AND STOPPED while bytes 1,227 -> 20,311 (16.6x) and density 64 -> 406 (6.3x). Sweep 5 (MEMORY.md by section): ## Operating gotchas 10,491 B = 51.6% in 14 lines = 749 B/line; ## Standing directives 2,696 B = 13.3%; layer-A proper 5,801 B = 28.6% => 65% is layer-C content. Instrument evaluation: ROADMAP.md's 15 distinct dates ALL READ INDIVIDUALLY, every one a phase/lane closure fact => the distinct-date test cries wolf on the principal status surface. cargo check --all-targets clean; check_knowledge_map OK; mdbook build clean; check_doctrines.sh 8/8 after git add` | `three classes not two; the one real gap is MEMORY.md's missing byte axis at 1.65x README's cap; both candidate instruments retired on measurement; tree grows to five leaves with .5 ordered before .3` |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `measured at 43aad06: check_readme_growth.sh names six destinations at :63-67, all inside note strings (advice, not assertions); repo-wide exactly two size instruments exist (README.md line+byte at :89-90; MEMORY.md line at check_memory_architecture.sh:38). Live-doc sizes in lines/bytes: README 159/10,375; MEMORY 50/21,406; TOOLBOX 106/10,987; USER_GUIDE 2,466/148,554; ROADMAP 2,874/182,894; CODEBASE_ANALYSIS 2,727/281,724; book/src 15,115/758,375; docs/decisions 10,049/646,803; docs/tasks 20,881/2,678,491; DEVELOPMENT_NOTES 15,968/987,868; CHANGES 44,189/2,311,825. Run-log density: ROADMAP 13.9%, architecture.md 13.8%, USER_GUIDE 8.8%, CODEBASE_ANALYSIS 7.8% (vs PGEN 94.7%). Longest single lines: CODEBASE_ANALYSIS 24,990 and 13,472 chars, ROADMAP 3,653 — the first being 2.03x the entire README byte cap. Density 103 B/line for CODEBASE_ANALYSIS vs 64-65 for README/ROADMAP` | `hole confirmed identical in kind, far milder in degree; byte-first instrument required; four destinations excluded from any cap by 0031 + 0033 test (2)` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `4a91c45` — `OVERFLOW-DESTINATION-INSTRUMENTATION.2 — the unmeasured axis` | Decision `0040`. **Three** classes, not two; the one real gap is `MEMORY.md`'s missing byte axis; both candidate instruments retired on measurement; tree grows to five leaves with `.5` inserted **before** `.3`. |
| `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `bb8a835` — `OVERFLOW-DESTINATION-INSTRUMENTATION.1 — audit + register: the cap moved the pressure` | Registration only; no cap added and no content trimmed, deliberately — classifying the destinations is `.2`'s decision, and four of them must never be capped. |

## Changelog

- `2026-07-31`: Opened on an **owner finding measured in PGEN** and sent back as a lesson:
  a cap that redirects overflow must check where the overflow lands. The finding's own
  closing clause is what made this a tree rather than a note — *"a policy-level hole, not a
  PGEN accident"* — because ANVIL adopted that policy and therefore inherits the hole by
  construction. Measuring it here produced one correction the source project could not see
  alone: **not every destination may be capped**, since four of ANVIL's are append-only by
  absolute owner directive, so the rule is *classify, then instrument the bounded ones* —
  and a **mixed** surface, which is what PGEN actually hit, has no valid cap at all until it
  is separated.
- `2026-07-31` (`.2`): The classification came back **three-way**, and the pressure came back
  **one category out**. `.1` looked for it on a neighbouring *surface*; it was on the
  neighbouring **axis of the capped file itself**. `MEMORY.md` — the file every session reads
  first — has held at exactly **50 of 50 lines** while growing **1,227 → 20,311 bytes**, and is
  now at **1.65×** the byte cap its sibling landing page is held to, with **65 %** of it layer-C
  content the memory standard says belongs in layers B/C. Both instruments `.1` proposed were
  measured and retired; the one that survives is the axis that was never there.
