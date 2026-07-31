# OVERFLOW-DESTINATION-INSTRUMENTATION: a cap that redirects overflow must check where the overflow lands

## Metadata

- Tree ID: `OVERFLOW-DESTINATION-INSTRUMENTATION`
- Status: `active`
- Roadmap lane: Live-doc hygiene / doctrine-policy completeness
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — audited + registered; frontier `.2`)
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
  Children: `.1` (audit + register), `.2` (classify + decide the instrument), `.3` (apply), `.4` (feed the correction back into the portable policy)

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.1`
  Status: `done`
  Goal: `Measure whether ANVIL inherits the hole the owner found in PGEN, and register the tree before anything is changed, per the standing directive that a defect is only handled if a task-tree owns it.`
  Acceptance: `The routing hint's destinations read directly from the check script; every live doc measured in BOTH lines and bytes; run-log density measured per surface rather than asserted; the append-only destinations identified and excluded from any future cap with the governing doctrine named; no repair attempted in this leaf.`
  Verification: `done — MEASURED at 43aad06. (1) HOLE CONFIRMED, IDENTICAL IN KIND: scripts/check_readme_growth.sh caps README.md on two axes and names six destinations at lines 63-67 — all inside `note` strings, i.e. advice, never assertions. Across every check in the repo exactly TWO files carry a size instrument: README.md (line+byte, check_readme_growth.sh:89-90) and MEMORY.md (line only, check_memory_architecture.sh:38). Every named destination is unmeasured. (2) SEVERITY IS NOT PGEN'S, and this is stated rather than assumed: run-log/banked-evidence density is ROADMAP.md 400/2874 = 13.9%, book/src/architecture.md 123/892 = 13.8%, USER_GUIDE.md 218/2466 = 8.8%, CODEBASE_ANALYSIS.md 212/2727 = 7.8% — against PGEN's 94.7%, an order of magnitude cleaner. Claiming parity would have been as wrong as missing the hole. (3) BUT LINE COUNTS HIDE THE EXPOSURE: the longest single line in CODEBASE_ANALYSIS.md is 24,990 chars — 2.03x ANVIL'S ENTIRE README BYTE CAP (12,288) — with a second at 13,472; ROADMAP.md's longest is 3,653. Density: CODEBASE_ANALYSIS.md 103 B/line vs 64-65 for README.md/ROADMAP.md. Any instrument must be BYTE-FIRST; a line cap on these files would be decorative. This is decision 0036 §(c) at its limit. (4) THE POLICY LESSON NEEDS A CORRECTION ANVIL CAN SEE AND PGEN CANNOT: "cap every destination" is WRONG here — CHANGES.md (2,311,825 B) and DEVELOPMENT_NOTES.md (987,868 B) are append-only by absolute owner directive (0031), and docs/tasks/ (2,678,491 B) + docs/decisions/ (646,803 B) are layer-B/C records; capping any of them would pressure authors into the history rewrite 0031 forbids, which is decision 0033 TEST (2) exactly. The rule is therefore CLASSIFY, then instrument only the bounded ones. (5) A THIRD CATEGORY IS NAMED: a MIXED surface (bounded status view + unbounded dated changelog in one file) has no valid cap BECAUSE of the mixture, and the mixture is what hides the growth — PGEN's LIVE_ACHIEVEMENT_STATUS.md at 94.7% changelog is that category, and its repair is SEPARATION BEFORE INSTRUMENTATION. NO REPAIR ATTEMPTED, deliberately: which destinations are bounded, and what instrument each gets, is .2's decision. Checks: check_doctrines.sh green after git add. Docs-only ⇒ DUT byte-identical.`
  Commit: `pending`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.2`
  Status: `pending`
  Goal: `Classify every overflow destination as bounded surface or append-only record, decide which bounded ones get an instrument and of what shape, and record it — including the mixed-surface category and its separate-before-you-cap repair.`
  Acceptance: `Each destination classified with its reasoning, applying decision 0033 test (2) and decision 0031 explicitly per entry — an append-only record MUST be excluded and MUST say why, so the exclusion cannot later read as an oversight. The instrument is BYTE-FIRST (.1 §4: a single 24,990-char line clears the README byte cap twice over). Caps are DERIVED from what the surface legitimately holds, per decision 0036 §(c), never chosen to fit current size — and the decision must state that raising a cap requires a new record, not an edit to the check. One check over a declared classified table (the ENUMERATION-PARITY shape), not one doctrine per file. The decision must state what it does NOT license, so it cannot be cited to justify trimming history.`
  Verification: `pending`
  Commit: `pending`

- ID: `OVERFLOW-DESTINATION-INSTRUMENTATION.3`
  Status: `pending`
  Goal: `Apply .2: implement the instrument, register it if it is a new doctrine, and prove it works in both directions.`
  Acceptance: `Negative-controlled both ways (breach the cap ⇒ fail; restore ⇒ pass) and vacuity-probed per decision 0037 — and per the standing gotcha, CHECK THAT THE CONTROL CAN FAIL before trusting that it did (three controls passed on the first try in CSG.6 and all three were too weak). Fails with a routing hint of its own, and that hint must not create the same hole one layer down — if it names a further destination, that destination is classified too, or the hint names no destination at all. check_doctrines.sh green.`
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
| 2 | `OVERFLOW-DESTINATION-INSTRUMENTATION.2` | `pending` | **Next.** Nothing may be capped until each destination is classified — four of them are append-only by doctrine and capping them would breach `0031`. |
| 3 | `OVERFLOW-DESTINATION-INSTRUMENTATION.3` | `pending` | Apply, with a control proven capable of failing before it is trusted. |
| 4 | `OVERFLOW-DESTINATION-INSTRUMENTATION.4` | `pending` | Push the corrected rule back into the portable policy — the owner's point was that this is a policy hole, so fixing only ANVIL leaves every other adopter exposed. |

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

## Blockers

- None. `.2` is a decision leaf and needs only the measurement recorded here.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `OVERFLOW-DESTINATION-INSTRUMENTATION.1` | `measured at 43aad06: check_readme_growth.sh names six destinations at :63-67, all inside note strings (advice, not assertions); repo-wide exactly two size instruments exist (README.md line+byte at :89-90; MEMORY.md line at check_memory_architecture.sh:38). Live-doc sizes in lines/bytes: README 159/10,375; MEMORY 50/21,406; TOOLBOX 106/10,987; USER_GUIDE 2,466/148,554; ROADMAP 2,874/182,894; CODEBASE_ANALYSIS 2,727/281,724; book/src 15,115/758,375; docs/decisions 10,049/646,803; docs/tasks 20,881/2,678,491; DEVELOPMENT_NOTES 15,968/987,868; CHANGES 44,189/2,311,825. Run-log density: ROADMAP 13.9%, architecture.md 13.8%, USER_GUIDE 8.8%, CODEBASE_ANALYSIS 7.8% (vs PGEN 94.7%). Longest single lines: CODEBASE_ANALYSIS 24,990 and 13,472 chars, ROADMAP 3,653 — the first being 2.03x the entire README byte cap. Density 103 B/line for CODEBASE_ANALYSIS vs 64-65 for README/ROADMAP` | `hole confirmed identical in kind, far milder in degree; byte-first instrument required; four destinations excluded from any cap by 0031 + 0033 test (2)` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
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
