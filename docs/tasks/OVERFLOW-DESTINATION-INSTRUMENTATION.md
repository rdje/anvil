# OVERFLOW-DESTINATION-INSTRUMENTATION: a cap that redirects overflow must check where the overflow lands

## Metadata

- Tree ID: `OVERFLOW-DESTINATION-INSTRUMENTATION`
- Status: `active`
- Roadmap lane: Live-doc hygiene / doctrine-policy completeness
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` **done** — decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md): **three** classes not two, and the pressure went to the **unmeasured AXIS** of `MEMORY.md`; `.3` **done** — the derived **6,144-byte** cap is live as a second assertion in `check_memory_architecture.sh`, negative-controlled both ways and vacuity-probed; frontier **`.6`**)
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
  Children: `.1` (audit + register), `.2` (classify + decide the instrument), `.5a` (the owner's standing directives -> decision 0041), `.5b` (the operating gotchas + the remaining layer-C sections -> Knowledge Map cards) — both ordered BEFORE `.3`, which cannot land while the file is over the derived cap, `.3` (apply the instrument), `.6` (own 0040 §(g)'s unrepaired long-line finding), `.4` (feed the correction back into the portable policy)

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

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.5a`
  Status: `done`
  Goal: `Relocate the owner's STANDING DIRECTIVES out of the overwrite-only resume pointer into layer C, verbatim and with owner+date provenance, leaving a pointer behind.`
  Acceptance: `Every directive lands in a durable layer-C home BEFORE anything is removed from MEMORY.md; MEMORY.md keeps a POINTER, never a second copy and never a deletion; the owner's wording is preserved verbatim; nothing is lost, proven by sweeping EVERY token in the removed range rather than a phrase list (0036 §(iii)); the resulting byte count is measured and stated against the 6,144 cap, and any gap is reported rather than met by raising the cap.`
  Verification: `done — OWNER AUTHORISED THE RELOCATION on 2026-07-31 ("OK then to relocate"), asked before acting because these are owner-set content. MEASURED FIRST, AND THE MEASUREMENT FOUND SOMETHING SHARPER THAN A SIZE PROBLEM: 13 citations across 9 decision records point at these directives, and FOUR name their location literally as "(MEMORY.md standing directives, 2026-07-30)" — i.e. nine decision records cited, as durable provenance, a file MEMORY_ARCHITECTURE.md §3/§6 guarantees is OVERWRITTEN on every update with a hard cap. The first directive says exactly this in the owner's own words ("If you just record in MEMORY.md it will be lost there"), so THE DIRECTIVE WARNING AGAINST PARKING THINGS IN THE RESUME POINTER WAS ITSELF PARKED IN THE RESUME POINTER, and it is the most-cited directive in the project. SPLIT MEASURED, NOT ASSUMED — probing each of the seven entries against docs/decisions/ and docs/knowledge/ separated CITED from RECORDED (the ODI.1 caution that a reference count is not a dependency measure, applied to this leaf's own probe: the coarse key reported "has a home" for entries whose only hits were citations pointing BACK at MEMORY.md, so every hit was opened and read): FOUR are genuinely recorded in 0031 with the owner's verbatim quotes — never rewrite history (the "Keep it raw, keep honest" quote at :72 plus the explicit prohibitions at :80-82), the SSD volume rule (the whole record), shared-means-shared (the 2026-07-29 clarification), and harness runtime files (:138, stated as an honest limit) — so MEMORY.md's copies of those were SHADOWS of 0031 under 0033 (derivable, growth-coupled, silent) and are repaired at rung R1 BY POINTER, not restated, since a second copy is the thing 0038 §(b) rejected. THREE had NO layer-C home at all and are now recorded VERBATIM in decision 0041: (a) a defect is only handled if a task-tree owns it, (b) decide don't ask, (c) ~/Documents/github is owner-owned. NOTHING IS LOST, PROVEN BY TOKEN SWEEP not a phrase list (0036 §(iii)): all 29 backticked tokens and all 162 words of >=5 chars in the removed 2,696-byte range were swept against docs/decisions/ + docs/knowledge/ + docs/tasks/ + MEMORY.md + MEMORY_ARCHITECTURE.md + README_POLICY.md. Backticked residue ZERO; word residue ONE — "doc/code" — which is a COMPOSITE OF COVERED PARTS ("doc" 63 hits, "code" 57 in docs/decisions/ alone), exactly the residue shape 0036 predicts for a working sweep, and the fact it belongs to ("no live doc or code points at a boot-volume path") is not merely recorded but MECHANICALLY GATED by NO-BOOT-VOLUME-REFS. No orphaned fact. NO CITATION NEEDED EDITING: all 13 cite by OWNER + DATE, which is stable across the move — the provenance style README-POLICY-PROVENANCE.1 mandated is what made the relocation cheap, and that is the reusable half. BYTES: MEMORY.md 19,885 -> 18,157 (-1,728 net of the 2-line pointer block), 49 -> 44 lines. THE 6,144 CAP IS NOT REACHED BY THIS LEAF AND THE GAP IS REPORTED RATHER THAN MET BY RAISING THE CAP (0040 non-license 3): 12,013 B remain over, and .5b carries them. Checks: cargo check --all-targets clean; check_knowledge_map OK; check_doctrines.sh 8/8 after git add. Docs-only => DUT byte-identical.`
  Commit: `4f3d508` — `OVERFLOW-DESTINATION-INSTRUMENTATION.5a — directives to layer C`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.5b`
  Status: `done`
  Goal: `Relocate the remaining layer-C content out of MEMORY.md — ## Operating gotchas (10,491 B in 14 lines = 749 B/line), and, if the cap is still unreachable, ## Lane invariants (915 B) and ## Validation policy (337 B) — into Knowledge Map fact cards, leaving pointers; then tighten ## Current state so it POINTS rather than summarises.`
  Acceptance: `Gotchas go to docs/knowledge/ fact cards, question-keyed, one atomic fact per card, each POINTING at the decision record that holds the full story where one exists. Probe each lesson against the durable layers FIRST and open every hit individually. Nothing lost, proven by whole-range token sweep with the residue reported. State the resulting byte count against the 6,144 cap; report any gap rather than raising the cap.`
  Verification: `done — MEMORY.md IS NOW UNDER ITS DERIVED CAP: 19,885 B at the start of .5 -> 18,157 (.5a) -> 6,037 (.5b), a 69.6% reduction, against the 6,144-byte cap derived at 0040 §(c). Lines 49 -> 30 (60% of the 50-line cap). .3 is therefore UNBLOCKED. TWELVE FACT CARDS WRITTEN, one per coherent retrieval question, each pointing at its canonical home rather than copying it: sandbox-clear-during-test-run, sweep-must-not-rewrite-its-own-subject (folds the anchored-path-rewrite lesson in, same subject), baseline-from-git-show-not-the-worktree, deleting-a-live-doc-safely (-> 0036), negative-control-must-be-able-to-fail (cross-links coverage-check-vacuity, which covers the complementary vacuity probe), probability-is-priority-under-mutual-exclusion (-> 0032), defect-class-audit-rules (-> 0033/0034), run-a-new-tool-against-real-output, never-parse-a-formatter-for-a-semantic-set (-> 0037), doctrine-check-must-classify-not-guess (-> 0030), gated-workflow-shell-gotchas, verilator-declfilename-on-fixed-filename-dumps. KM 94 -> 106 facts / 955 -> 1,022 question keys, check_knowledge_map OK. LANE INVARIANTS AND VALIDATION POLICY WERE POINTED AT, NOT MOVED — probed and every one is already recorded: the four feedback_* invariants in decisions 0006/0007/0017/0009, rules-first in book/src/by-construction.md + algorithm.md, SCHEMA-DERIVED in 0004/0011, doctrine-enforcement in 0026 + the existing KM card; and BOTH halves of the validation policy (the 90% ram_guard threshold + environment-stop framing, and the focused-checks-for-workflow-leaves owner instruction) are in decision 0003, whose own answers keys already ask "what RAM threshold stops a full suite" and "when is focused workflow validation enough". So those two sections were 0033 shadows and are repaired at R1 by pointer. THE GOTCHAS POINTER IS A DERIVATION, NOT A LIST, and this was a deliberate call: naming the 12 cards in MEMORY.md would be derivable + growth-coupled + silent, i.e. a fresh shadow minted by the leaf that removes shadows, so the pointer publishes `grep -l 'gotcha' docs/knowledge/*.md` instead and every card carries the `gotcha` tag. SCOPE WIDENING STATED AND REQUIRED, not opportunistic: 0040 §(d) scoped .5 to two sections, but removing only those leaves 6,698 B — still over — so Lane invariants, Validation policy and the ## Current state tightening are REQUIRED BY THE LEAF'S OWN GOAL. ## Current state was itself 5,028 B of layer-A prose summarising decisions 0040/0041 that already hold the content; rewritten to POINT, which is the same defect one level in. LOSS PROOF by whole-range token sweep over the removed 11,743 bytes (0036 §(iii)), not a phrase list: 104 backticked tokens + 565 words of >=5 chars + 37 numerals = 706 swept, RESIDUE 7, and every one opened and verified a composite or re-phrasing of covered parts with ZERO orphaned facts — "135 |=" / "13 extend" / "1 max" are recorded verbatim at 0033:106-107 as 135 x |= + 13 x .extend() + 1 x .max() (only the multiplication sign differs); "10,297 B / 141 lines" at deleting-a-live-doc-safely.md:36; "sites/" at 0034:139 as "8 sites in 2 files"; "dialects" and "policy/history" are re-phrasings whose facts are carried in full. TWO OF THE NEW CARDS FIRED DURING THEIR OWN VERIFICATION, which is the best evidence they were worth writing: git grep reported the 10,297 residue as missing because the new cards were still UNTRACKED (gated-workflow-shell-gotchas: git grep sees tracked content only, run checks after git add), and a single-line grep missed "policy and history documents first" because it wraps across a line break (never-parse-a-formatter-for-a-semantic-set). Checks: cargo check --all-targets clean; check_knowledge_map OK; check_doctrines.sh 8/8 after git add; MEMORY.md required fields (active_work_unit / next_action / in_flight_uncommitted / blockers) all present. Docs-only => DUT byte-identical.`
  Commit: `6c0c953` — `OVERFLOW-DESTINATION-INSTRUMENTATION.5b — MEMORY.md under its cap`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.3`
  Status: `done`
  Goal: `Apply .2: implement the instrument, register it if it is a new doctrine, and prove it works in both directions.`
  Acceptance: `Negative-controlled both ways (breach the cap => fail; restore => pass) and vacuity-probed per decision 0037 — and per the standing gotcha, CHECK THAT THE CONTROL CAN FAIL before trusting that it did. Fails with a routing hint of its own, and that hint must not create the same hole one layer down. check_doctrines.sh green. Per decision 0040 §(b) this is NOT a new doctrine: MEMORY-ARCH already owns MEMORY.md's size.`
  Verification: `done — THE BYTE CAP IS LIVE, as MEMORY_POINTER_BYTE_CAP=6144 in scripts/check_memory_architecture.sh, beside the existing line cap. NOT a ninth doctrine, deliberately: MEMORY-ARCH already owns this file's size and a second registered mechanism for one job is what feedback_full_factorization forbids; the DOCTRINES registry is unchanged at 8 and the ENUMERATION-PARITY pair still holds (that extractor reads only the first column of §10's table, so amending the MEMORY-ARCH description cell cannot perturb it — checked before editing, per the standing gotcha about grepping the doctrine checks for a file first). THREE CONTROLS RUN, and the middle one is the one that matters. BASELINE: exit 0, "MEMORY.md is 6021 bytes (<= cap 6144)". CONTROL 1 — BREACH THE BYTE AXIS ONLY: 400 characters appended to an EXISTING line so the line count stays at 30, isolating the new axis from the pre-existing line cap; exit 1, and the emitted failure is specifically "MEMORY.md is 6422 bytes (> cap 6144)" plus the routing hint, proving the NEW assertion fired rather than the old one. Restore => exit 0. CONTROL 2 — VACUITY PROBE (decision 0037, "delete the subject and re-run"): the byte comparison was neutered to `if true; then` and the SAME 6,422-byte file re-checked; it PASSED, which is what proves Control 1's failure came from THIS assertion and nothing else. That is the step the standing CSG.6 gotcha demands — three controls once passed on the first try and all three were too weak to fail — so the control was shown CAPABLE of failing, for the RIGHT reason, before being trusted. Both files restored byte-for-byte afterwards; final tree re-checked green. THE ROUTING HINT DOES NOT REPRODUCE THE HOLE ONE LAYER DOWN, which .3's acceptance required explicitly: it names docs/decisions/ (layer C), docs/knowledge/ fact cards, docs/tasks/ (layer B) and git+CHANGES.md — every one CLASSIFIED by decision 0040 as an append-only record that is SUPPOSED to grow and is never capped. It also states the two rules a router needs and a size number cannot carry: leave a POINTER, never a second copy (0033/0038), and raising the cap requires a new decision record rather than an edit to the constant. DOCTRINE_ENFORCEMENT.md §10's MEMORY-ARCH row amended to record the second axis, the measurement that motivated it, the derivation of 6,144, and why it is an assertion rather than a doctrine. Checks: scripts/check_doctrines.sh 8/8 after git add; MEMORY.md 30 lines / 6,021 bytes (98% of the line cap headroom unused, 98% of the byte cap consumed). scripts/-only + docs => no src/, tests/ or examples/ change => DUT byte-identical.`
  Commit: `e801617` — `OVERFLOW-DESTINATION-INSTRUMENTATION.3 — the second axis is live`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.6`
  Status: `pending`
  Goal: `Own the finding 0040 §(g) surfaced but did not repair: CODEBASE_ANALYSIS.md carries a single line of 24,990 characters — 2.03x the entire README byte cap — plus a second at 13,472, and it is named by NEITHER routing enumeration despite being a mandatory bootstrap read (SESSION_BOOTSTRAP.md step 3) and a per-commit checklist item (COMMIT.md §5).`
  Acceptance: `Registered here rather than left in decision prose, per the owner's standing directive that a defect is only handled if a task-tree owns it — a finding parked in a record nobody re-reads is the same failure mode 0039 was written for. FIRST MEASURE, THEN DECIDE, and the two questions are separable: (i) is the long line a DEFECT at all, and for whom? It renders correctly, so the cost is not the reader — it is that git produces a useless diff for any edit to it, that a line-oriented grep or extractor can backtrack catastrophically on it (this leaf's own sibling hit exactly that: a bounded-context regex over docs/TASK_TREE.md's 7,021- and 39,370-char rows timed out at 120 s), and that decision 0040 classified the file B2, so a size cap is NOT available as the answer. Quantify the class across the tree before deciding — measure the distribution of longest-line lengths over every tracked *.md, not just this file, and record the match count per 0039 rule (b). (ii) if it is worth repairing, the repair is REFLOW ONLY: not one word of content changes, and the file must be byte-compared for content equality (strip newlines, hash, assert equal) so the diff is provably whitespace-only. NON-GOALS, inherited: this is not a content sweep (the tree's Non-Goals), and CHANGES.md / DEVELOPMENT_NOTES.md / docs/tasks/ are append-only and out of scope entirely even though they contain the longest lines in the repo (docs/tasks/ tops out at 33,890) — 0031, and 0040 non-license 1. If the honest answer is that reflow is not worth the churn, record THAT with the measurement rather than leaving the finding open; a measured "no" closes a leaf just as well as a repair.`
  Verification: `pending`
  Commit: `pending`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.4`
  Status: `pending`
  Goal: `Feed the correction back into the PORTABLE policy, since the owner located the hole at policy level rather than in one project.`
  Acceptance: `README_POLICY.md, DOCTRINE_ENFORCEMENT.md AND MEMORY_ARCHITECTURE.md gain the rule in project-neutral language — §6 of that standard says the resume-pointer cap is "roughly one screen (<= ~50 lines)" and its §9 reference self-check script enforces LINES ONLY, so the portable standard still ships the exact single-axis hole .3 just closed locally, and .3's measurement (lines flat at the cap while bytes went x16.6) is the evidence for changing it: a policy that redirects overflow must require each destination to be classified and each bounded one instrumented, and must name the mixed-surface category whose repair is separation before instrumentation. Neutrality is verified the way README-POLICY-PROVENANCE.1 verified it — the policy BODY must contain zero project-specific and zero harness-specific tokens, with any ANVIL-specific content fenced in an adoption note. Provenance cited by OWNER + DATE + a repo-side durable home, never a harness file (README-POLICY-PROVENANCE.1). Credit the measurement to the owner's PGEN finding of 2026-07-31, since a rule is more re-checkable when its evidence is named.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `done` | Audited and registered. Hole confirmed identical in kind (six destinations named only in `note` strings; exactly two instrumented files repo-wide), severity measured and found **far milder** than PGEN's (7.8–13.9 % run-log density vs 94.7 %), but the byte exposure is real — one `CODEBASE_ANALYSIS.md` line is **2.03×** the whole README byte cap. |
| 2 | `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `done` | Classified as decision [`0040`](../decisions/0040-overflow-destination-classification-and-the-unmeasured-axis.md), and measuring first changed **both** the classification and the target. The classes are **three**, not two — `.1`'s binary would have forced five *bounded-in-kind-but-not-in-size* surfaces (`USER_GUIDE.md`, `book/src/`, `ROADMAP.md`, `TOOLBOX.md`, `CODEBASE_ANALYSIS.md`) into "bounded ⇒ must be instrumented", which is exactly where *"cap every destination"* turns harmful. And the pressure went **one category beyond** what the tree opened on: not to a neighbouring *surface* but to the **unmeasured axis of an already-capped file** — `MEMORY.md` sits at **exactly 50 of 50 lines** and **20,311 bytes = 1.65× the byte cap `README.md` is held to**, having gone **19 → 50 lines (capped) while bytes went 1,227 → 20,311 (16.6×)** since the line cap landed. Both instruments `.1` proposed are **retired on measurement**. |
| 3 | `OVERFLOW-DESTINATION-INSTRUMENTATION.5a` | `done` | The owner authorised relocation (`2026-07-31`). Measuring first found more than a size problem: **13 citations across 9 decision records** pointed at these directives, **4** naming the location literally as *"(`MEMORY.md` standing directives, `2026-07-30`)"* — nine records citing, as durable provenance, a file the standard guarantees is **overwritten**. **The directive that says *"if you just record in `MEMORY.md` it will be lost there"* was itself parked in `MEMORY.md`.** Four of the seven were already recorded in `0031` with the owner's verbatim quotes ⇒ **shadows, repaired at R1 by pointer**; the other three had no layer-C home ⇒ decision `0041`, verbatim. Token sweep over the whole removed range: **0** backticked residue, **1** word residue (`doc/code`, a composite of covered parts). **19,885 → 18,157 B.** |
| 4 | `OVERFLOW-DESTINATION-INSTRUMENTATION.3` | `e801617` — `OVERFLOW-DESTINATION-INSTRUMENTATION.3 — the second axis is live` | The 6,144-byte cap live as a **second assertion**, not a ninth doctrine. Negative-controlled on the byte axis alone, then vacuity-probed to prove *which* assertion fired. |
| `OVERFLOW-DESTINATION-INSTRUMENTATION.5b` | `done` | **`MEMORY.md` is under its derived cap: 19,885 → 6,037 B (−69.6 %), 49 → 30 lines.** Twelve fact cards written (KM **94 → 106** facts / **955 → 1,022** keys). `## Lane invariants` and `## Validation policy` were **pointed at, not moved** — probed, and every invariant is already in `0003`/`0004`/`0006`/`0007`/`0011`/`0017`/`0026` or the book, so both were `0033` shadows repaired at **R1**. The gotchas pointer publishes a **derivation** (`grep -l 'gotcha' docs/knowledge/*.md`), not a list of card names, which would have been a fresh shadow minted by the leaf that removes shadows. Loss proof: **706 tokens swept, residue 7, all verified composites, zero orphaned facts** — and two of the new cards fired during their own verification. |
| 5 | `OVERFLOW-DESTINATION-INSTRUMENTATION.3` | `done` | The **6,144-byte** cap is live as a second assertion in `check_memory_architecture.sh` — **not** a ninth doctrine (`MEMORY-ARCH` already owns the file; registry unchanged at 8). Three controls: baseline green; **byte axis breached alone** (400 chars onto an existing line, line count unchanged at 30) ⇒ fails with the *byte* message; **vacuity probe** — assertion neutered, same over-cap file **passes**, proving the failure came from this assertion and not another. The routing hint names only destinations `0040` already classified as append-only. |
| 6 | `OVERFLOW-DESTINATION-INSTRUMENTATION.6` | `pending` | Owns the finding `0040` §(g) **surfaced but deliberately did not repair**: `CODEBASE_ANALYSIS.md`'s **24,990-character** line (2.03× the whole README byte cap), on a file named by **neither** routing enumeration. Registered rather than left in decision prose — a finding parked in a record nobody re-reads is the failure mode `0039` exists for. Measures the class tree-wide before deciding, and a measured *"not worth the churn"* closes it just as well as a reflow. |
| 7 | `OVERFLOW-DESTINATION-INSTRUMENTATION.4` | `pending` | Push the corrected rule back into the portable policy — the owner's point was that this is a policy hole, so fixing only ANVIL leaves every other adopter exposed. `.2` sharpened what must travel: not just *classify your destinations* but **instrument every axis of a bounded surface, or the cap measures the one thing that is not growing.** |

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
- `2026-07-31` (`.5a`): **Asked before acting, because the content is owner-set.** `0040` §(d)
  designed the demotion; the owner authorised it explicitly (*"OK then to relocate"*). The
  directives are recorded **verbatim** and `0041` states that an agent may relocate and point at
  them but may **never** reword them.
- `2026-07-31` (`.5a`): **The size problem was the smaller half.** Measured, **13 citations across
  9 decision records** pointed at these directives and **4** named the location literally as
  *"(`MEMORY.md` standing directives, `2026-07-30`)"* — nine records citing, as durable
  provenance, a file `MEMORY_ARCHITECTURE.md` §3/§6 guarantees is **overwritten** on every update.
  The most-cited directive of the set is the one that says *"if you just record in `MEMORY.md` it
  will be lost there."*
- `2026-07-31` (`.5a`): **Four of seven were pointed at, not moved — the probe had to separate
  CITED from RECORDED.** A coarse key reported *"has a layer-C home"* for entries whose only hits
  were citations pointing **back** at `MEMORY.md`; every hit was opened and read. Four are
  genuinely recorded in `0031` **with the owner's verbatim quotes**, so `MEMORY.md`'s copies were
  `0033` shadows and are repaired at **R1 by pointer**. This is `ODI.1`'s own caution — *a
  reference count is not a dependency measure* — applied to this leaf's own instrument.
- `2026-07-31` (`.5a`): **No citation needed editing, and that is the reusable half.** All 13 cite
  by **owner + date**, which is stable across a move. The provenance style
  `README-POLICY-PROVENANCE.1` mandated — *cite by owner and date, never by a harness file* — is
  exactly what made the relocation cheap. A citation keyed on a **location** would have needed 13
  edits across 9 landed records.
- `2026-07-31` (`.5b`): **The gotchas pointer is a DERIVATION, not a list — the leaf that removes
  shadows very nearly minted one.** Naming the twelve cards in `MEMORY.md` would be derivable,
  growth-coupled and silent: every future gotcha card would have to remember to update it. The
  pointer publishes `grep -l 'gotcha' docs/knowledge/*.md` instead, and every card carries the tag.
- `2026-07-31` (`.5b`): **`## Lane invariants` and `## Validation policy` were pointed at, not
  moved.** Probed individually: all four `feedback_*` invariants, rules-first, SCHEMA-DERIVED and
  doctrine-enforcement are recorded in `0003`/`0004`/`0006`/`0007`/`0011`/`0017`/`0026` or the
  book — and decision `0003`'s own `answers:` keys already ask *"what RAM threshold stops a full
  suite"*. Both sections were `0033` shadows; R1 by pointer.
- `2026-07-31` (`.5b`): **Scope widening stated and required, not opportunistic.** `0040` §(d)
  scoped `.5` to two sections; removing only those leaves **6,698 B**, still over the cap. So the
  widening to `## Lane invariants`, `## Validation policy` and the `## Current state` tightening is
  demanded by the leaf's own goal — the `DATED-COUNT-SWEEP-EXEMPTION.2` precedent, where every
  added line was measured before it was touched.
- `2026-07-31` (`.5b`): **`## Current state` was the same defect one level in.** 5,028 B of layer-A
  prose summarising decisions `0040`/`0041`, which already hold the content. Layer A's job is to
  **point**; rewritten accordingly.
- `2026-07-31` (`.5a`): **`.5` split into `.5a` / `.5b`.** Different content kind (owner policy vs
  operational lessons) and different destination (a decision record vs Knowledge Map cards), so
  each gets its own loss-proof and its own commit. The cap is reached by neither alone.

- `2026-07-31` (`.2`): **The two routing enumerations must STAY different.** They disagree by
  exactly `CHANGES.md` + `DEVELOPMENT_NOTES.md` — the two files append-only by `0031` — because
  the hint routes **new content** (which may never enter an append-only record) while README's
  table routes a **reader** (and change history does live in `CHANGES.md`). Recorded because
  "harmonising the inconsistency" would instruct authors to append new prose to history.

## Blockers

- None. `.5` is next and needs only `0040`'s measurement. `.3` is **blocked on `.5`** by design,
  not by an external dependency: the instrument it installs cannot pass at HEAD. `.6` was opened
  in the same turn that surfaced its finding, per the standing directive that **a defect is only
  handled if a task-tree owns it** — `0040` §(g) had recorded it as *"for a future leaf"*, which is
  exactly the prose-only parking the directive forbids.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.5b` | `MEMORY.md 19,885 -> 6,037 B (-69.6%), 49 -> 30 lines, UNDER the 6,144 derived cap => .3 unblocked. 12 fact cards written; KM 94 -> 106 facts / 955 -> 1,022 question keys; check_knowledge_map OK. Lane invariants and Validation policy probed individually and POINTED AT, not moved (all recorded in 0003/0004/0006/0007/0011/0017/0026 or the book) => 0033 shadows, R1 by pointer. Gotchas pointer is a DERIVATION (grep -l gotcha docs/knowledge/*.md), not a list of card names, which would have been a fresh shadow. LOSS PROOF over the removed 11,743 bytes: 104 backticked + 565 words + 37 numerals = 706 tokens swept, residue 7, EVERY ONE opened and verified a composite or re-phrasing, zero orphaned facts. Two of the new cards fired during their own verification (git grep blind to the untracked cards; a single-line grep missing a line-wrapped phrase). cargo check --all-targets clean; check_doctrines.sh 8/8 after git add; MEMORY.md required fields all present` | `cap reached without raising it; .3 unblocked` |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.5a` | `Owner authorised the relocation. Probe over all 7 entries separating CITED from RECORDED, every hit opened individually: 4 genuinely recorded in 0031 with the owner's verbatim quotes (:72 the "keep it raw" quote, :80-82 the prohibitions, :111/:156 CARGO_HOME/RUSTUP_HOME, :138 the harness limit) => 0033 shadows, repaired at R1 by pointer; 3 with no layer-C home => recorded verbatim in decision 0041. Citation census: 13 citations across 9 decision records, 4 naming "(MEMORY.md standing directives, 2026-07-30)" — all cite by owner+date, so ZERO needed editing. LOSS PROOF by whole-range token sweep (0036 §(iii)), not a phrase list: 29 backticked tokens swept, residue 0; 162 words of >=5 chars swept, residue 1 ("doc/code"), verified a composite of covered parts (doc 63 / code 57 hits in docs/decisions/) whose underlying fact is mechanically gated by NO-BOOT-VOLUME-REFS. MEMORY.md 19,885 -> 18,157 B, 49 -> 44 lines; 12,013 B still over the 6,144 cap, REPORTED not met by raising it. cargo check --all-targets clean; check_knowledge_map OK; check_doctrines.sh 8/8 after git add` | `3 directives durably recorded, 4 de-duplicated to pointers, 13 citations resolved without editing any of them; cap gap reported for .5b` |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `Re-measured at e4b4fd5, every sweep recording its key, authoritative set and MATCH COUNT per decision 0039 rule (b). Sweep 1 (routing-hint destinations, set = check_readme_growth.sh:61-70): 7 — .1 recorded six. Sweep 2 (structures mapping content KIND -> canonical home, set = whole tree): 2, and they DISAGREE by exactly CHANGES.md + DEVELOPMENT_NOTES.md, the two files append-only by 0031. Sweep 3 (lines/bytes/longest-line/distinct-dates per surface, 12 surfaces): README 159/10,375/378/65 B-per-line; MEMORY 50/20,311/2,874/406; TOOLBOX 106/10,987/1,606; USER_GUIDE 2,466/148,554/512; ROADMAP 2,874/182,894/3,653; book/src 15,121/758,735/6,327; CODEBASE_ANALYSIS 2,734/282,136/24,990; docs/evidence 415/18,479; docs/decisions 10,376/673,282; docs/tasks 21,489/2,751,963; DEVELOPMENT_NOTES 16,209/1,005,505; CHANGES 44,787/2,353,851. Sweep 4 (MEMORY.md git history, 55 sampled revisions): cap installed 2d01e8e 2026-06-05 truncating 2,399 lines/306,099 B -> 19/1,227/64 B-per-line; since then lines 19 -> 50 AND STOPPED while bytes 1,227 -> 20,311 (16.6x) and density 64 -> 406 (6.3x). Sweep 5 (MEMORY.md by section): ## Operating gotchas 10,491 B = 51.6% in 14 lines = 749 B/line; ## Standing directives 2,696 B = 13.3%; layer-A proper 5,801 B = 28.6% => 65% is layer-C content. Instrument evaluation: ROADMAP.md's 15 distinct dates ALL READ INDIVIDUALLY, every one a phase/lane closure fact => the distinct-date test cries wolf on the principal status surface. cargo check --all-targets clean; check_knowledge_map OK; mdbook build clean; check_doctrines.sh 8/8 after git add` | `three classes not two; the one real gap is MEMORY.md's missing byte axis at 1.65x README's cap; both candidate instruments retired on measurement; tree grows to five leaves with .5 ordered before .3` |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `measured at 43aad06: check_readme_growth.sh names six destinations at :63-67, all inside note strings (advice, not assertions); repo-wide exactly two size instruments exist (README.md line+byte at :89-90; MEMORY.md line at check_memory_architecture.sh:38). Live-doc sizes in lines/bytes: README 159/10,375; MEMORY 50/21,406; TOOLBOX 106/10,987; USER_GUIDE 2,466/148,554; ROADMAP 2,874/182,894; CODEBASE_ANALYSIS 2,727/281,724; book/src 15,115/758,375; docs/decisions 10,049/646,803; docs/tasks 20,881/2,678,491; DEVELOPMENT_NOTES 15,968/987,868; CHANGES 44,189/2,311,825. Run-log density: ROADMAP 13.9%, architecture.md 13.8%, USER_GUIDE 8.8%, CODEBASE_ANALYSIS 7.8% (vs PGEN 94.7%). Longest single lines: CODEBASE_ANALYSIS 24,990 and 13,472 chars, ROADMAP 3,653 — the first being 2.03x the entire README byte cap. Density 103 B/line for CODEBASE_ANALYSIS vs 64-65 for README/ROADMAP` | `hole confirmed identical in kind, far milder in degree; byte-first instrument required; four destinations excluded from any cap by 0031 + 0033 test (2)` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `OVERFLOW-DESTINATION-INSTRUMENTATION.5b` | `6c0c953` — `OVERFLOW-DESTINATION-INSTRUMENTATION.5b — MEMORY.md under its cap` | 12 KM fact cards + pointer blocks; **19,885 → 6,037 B**. Loss proof: 706 tokens swept, residue 7, all composites. |
| `OVERFLOW-DESTINATION-INSTRUMENTATION.5a` | `4f3d508` — `OVERFLOW-DESTINATION-INSTRUMENTATION.5a — directives to layer C` | Decision `0041` records 3 directives verbatim; 4 more repaired to pointers at `0033` R1. Owner-authorised. |
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
