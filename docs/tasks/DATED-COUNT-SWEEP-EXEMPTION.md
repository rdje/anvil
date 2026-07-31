# DATED-COUNT-SWEEP-EXEMPTION: a date disguised a standing claim as history, and exempted it from the sweep built to catch it

## Metadata

- Tree ID: `DATED-COUNT-SWEEP-EXEMPTION`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — audited, measured and registered; frontier `.2`)
- Owner: repo-local workflow

## Goal

`BOOK-TEST-COUNT-SHADOWS` closed on `2026-07-31` having deleted five per-file test
counts from `book/src/architecture.md` and replaced them with runnable derivations.
**The grand total survived, 57 lines below the repair, in the same file** — and a
second copy of the same claim survived in `CODEBASE_ANALYSIS.md`.

Found `2026-07-31` during session bootstrap, while reading `book/src/architecture.md`
as required by `SESSION_BOOTSTRAP.md` step 1.

## The measurement (at `349eeb6`)

| site | claims | actual | error |
| --- | --- | ---: | ---: |
| `book/src/architecture.md:612` | 226 unit-target tests | **751** (`cargo test --lib -- --list`) | −525 |
| " | 68 integration tests | **195** (`grep -c '#[test]' tests/`) | −127 |
| " | **294** passing tests | **946** | **claim is 31 % of reality** |
| `CODEBASE_ANALYSIS.md:2425` | 228 unit-target tests | **751** | −523 |
| " | 79 integration tests | **195** | −116 |
| " | **307** passing tests | **946** | **claim is 32 % of reality** |
| `CODEBASE_ANALYSIS.md:2423` | `tests/pipeline.rs` — 79 integration tests | **133** | −54 |
| `CODEBASE_ANALYSIS.md:2424` | `tests/book_examples.rs` — 3 tests | **4** | −1 |
| " | 9 skip-sentineled book blocks | **39** sentinels present | −30 |

Every error is an **under**-count — the same monotone-decay signature
`BOOK-TEST-COUNT-SHADOWS` measured: tests get added, prose does not.

## The root cause — and it is NOT "the sweep missed the file it was editing"

That was the first hypothesis and **the evidence refutes it.** `BOOK-TEST-COUNT-SHADOWS.1`
swept 94 live-doc files keyed on *"a numeral immediately qualifying a countable repo
noun"*, and its own verification log shows it **did** examine `CODEBASE_ANALYSIS.md`
(it rejected that file's *"all 7 categories"* as a false positive, with a reason). Both
files were in scope. Both claims match the key. Neither was flagged.

**What actually carried them through is the date.** `.1` and `.2` both applied an explicit
**exclusion by kind** — recorded in `.2`'s verification log as: *"banked-run citations
('3 scenarios / 12 modules') are dated measurements of a specific run and are exempt by
the same reasoning `.1` used for append-only history."* That exemption is **correct** for a
`CHANGES.md` entry or a banked evidence citation, where the whole record is a statement
about a past run. It is **wrong** here, and the discriminator is exact:

| | dated? | outcome |
| --- | --- | --- |
| the five per-file counts `.1` deleted | **no** — bare `` `src/ir/types.rs` — 40 tests `` | **caught and deleted** |
| `architecture.md:612` | **yes** — *"(current HEAD, `cargo test` on 2026-04-30)"* | **survived** |
| `CODEBASE_ANALYSIS.md:2425` | **yes** — *"(`cargo test`, 2026-05-02)"* | **survived** |

`.1` deleted every *undated* count in that file and left every *dated* one. The date is
the whole discriminator.

**The reusable finding:** *a date attached to a standing claim disguises it as history,
and thereby exempts it from the very sweep designed to catch it.* The exemption should
have been keyed not on **"is it dated?"** but on **"is the enclosing record a statement
about the past, or about the present?"** — and both survivors answer *the present* **in
their own words**: one says *"**Current** executed counts"*, the other *"**current
HEAD**"*, while carrying a date two and three months stale. They are **self-refuting on
their own terms**, which is precisely the signature
[`OVERFLOW-DESTINATION-INSTRUMENTATION.1`](OVERFLOW-DESTINATION-INSTRUMENTATION.md)
independently measured in PGEN (`Last updated: 2026-06-02` over `2026-07-31` content).
That tree's candidate instrument (b) — *a self-declared date that disagrees with the
file's newest content* — generalises to exactly this class, and the two trees should be
read together.

## Why it matters more than the numbers

`CODEBASE_ANALYSIS.md` is not decoration either: `SESSION_BOOTSTRAP.md` step 3 names it
*"the authoritative snapshot of the workspace"* and instructs a recovering session to
amend it, and `COMMIT.md` §5 makes it a per-commit checklist item. A cold session reads
*"307 passing tests"* and forms a mental model of a codebase a third its actual test
mass — then makes scoping decisions on it. `book/src/architecture.md` is worse in one
way: the mdBook is, by owner directive, the **only window** into the project.

And there is a sharper edge. The **repair 57 lines above** at `:543-555` explicitly says
the counts *"had decayed — one by 72 %"* and that deriving them *"is the repair that
cannot rot."* The file therefore now contains, 57 lines apart, both the lesson and a live
instance of the thing the lesson forbids — and the instance is decayed by **69 %**, worse
than the 72 % case the prose holds up as the cautionary example.

## Non-Goals

- **Not "update the numbers."** Decision [`0033`](../decisions/0033-shadow-enumeration-classification.md)
  R1 is the rung, and the replacement already exists: `architecture.md:545-550` publishes
  three runnable derivations. A corrected number is correct until the next test lands.
- **Not "add a gate."** `0033` records that gating a redundant count *"keeps the shadow
  alive and spends a mechanism on it forever."*
- **Not a re-audit of `BOOK-TEST-COUNT-SHADOWS`'s closed leaves.** They are correct
  history; a closed tree's too-strong claim is repaired by a *new* tree that measures it,
  never by rewriting the old one — the precedent that tree itself set over
  `SHADOW-ENUMERATION-SWEEP`.
- **No test is added, removed or renamed.** Docs-only ⇒ DUT byte-identical.

## Acceptance Criteria

- Both dated totals are gone, at rung **R1 (deletion)**, replaced by the derivation a
  reader can run — not by fresh numbers.
- `CODEBASE_ANALYSIS.md`'s per-target claims (`tests/pipeline.rs — 79`,
  `book_examples.rs — 3 tests / 9 skip-sentineled`) are resolved the same way, or the
  measurement is recorded explaining why any of them is *not* a shadow.
- The **exemption rule itself is repaired**, not just its two victims: the recorded
  sweep heuristic must be restated as *"is the enclosing record about the past or the
  present?"* rather than *"is it dated?"*, somewhere durable enough that the next sweep
  inherits it. This is the part that stops the class recurring; deleting two lines does not.
- A re-sweep keyed on the **corrected** rule, run over the live docs, with whatever it
  finds recorded — including false positives and the reason each is rejected.
- `mdbook build` clean; `scripts/check_doctrines.sh` 8/8; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `DATED-COUNT-SWEEP-EXEMPTION`
  Status: `active`
  Goal: `Delete the two dated test-count totals that BOOK-TEST-COUNT-SHADOWS's sweep exempted, and repair the exemption rule that let them through.`
  Children: `.1` (audit + register), `.2` (delete the survivors + re-sweep under the corrected rule), `.3` (make the corrected rule durable)

- ID: `DATED-COUNT-SWEEP-EXEMPTION.1`
  Status: `done`
  Goal: `Measure the surviving claims against the real test surface, establish WHY the closing sweep did not catch them, and register the tree before anything is edited — per the standing directive that a defect is only handled if a task-tree owns it.`
  Acceptance: `Every claimed number re-derived from the repo using the derivations the book itself publishes, not inferred; the root cause established from evidence rather than assumed, with the first hypothesis explicitly tested and reported whichever way it falls; the class swept from the authoritative set so the tree owns every instance rather than the one that was tripped over; no repair attempted in this leaf.`
  Verification: `done — MEASURED at 349eeb6 using the three derivations book/src/architecture.md:545-550 itself publishes. book/src/architecture.md:612 claims "226 unit-target tests + 68 integration tests = 294 passing tests (current HEAD, cargo test on 2026-04-30)"; actual is 751 (cargo test --lib -- --list) + 195 (grep -c '#[test]' tests/) = 946, so the claim is 31% of reality. CODEBASE_ANALYSIS.md:2425 claims "228 + 79 = 307 (cargo test, 2026-05-02)" => 32% of reality; :2423 claims tests/pipeline.rs has 79 integration tests (actual 133); :2424 claims tests/book_examples.rs has 3 tests (actual 4) and 9 skip-sentineled blocks (actual 39 sentinels present). Every error is an UNDER-count — the monotone-decay signature BOOK-TEST-COUNT-SHADOWS measured. ROOT CAUSE — FIRST HYPOTHESIS TESTED AND REFUTED, reported as it fell: the miss is NOT "the sweep skipped the file it was editing". BOOK-TEST-COUNT-SHADOWS.1 swept 94 live-doc files keyed on "a numeral immediately qualifying a countable repo noun", and its own verification log proves it examined CODEBASE_ANALYSIS.md (it rejected that file's "all 7 categories" as a false positive, with a reason). Both files were in scope and both claims match the key. THE ACTUAL CAUSE IS THE DATE. .1/.2 applied an explicit exclusion BY KIND — recorded in .2's log as "banked-run citations are dated measurements of a specific run and are exempt by the same reasoning .1 used for append-only history" — and the discriminator is exact: the five per-file counts .1 DELETED were undated bare numbers; the two that SURVIVED are both parenthetically dated ("(current HEAD, cargo test on 2026-04-30)", "(cargo test, 2026-05-02)"). .1 deleted every undated count in that file and left every dated one. REUSABLE FINDING: a date attached to a standing claim disguises it as history and exempts it from the sweep built to catch it; the exemption must be keyed on "is the enclosing record about the past or the present?", not on "is it dated?". Both survivors answer THE PRESENT IN THEIR OWN WORDS ("Current executed counts", "current HEAD") while carrying a 2-3 month old date — self-refuting on their own terms, the same signature OVERFLOW-DESTINATION-INSTRUMENTATION.1 independently measured in PGEN (Last updated: 2026-06-02 over 2026-07-31 content), whose candidate instrument (b) generalises to exactly this class. SWEPT FROM THE AUTHORITATIVE SET (147 tracked *.md, excluding CHANGES.md / DEVELOPMENT_NOTES.md / docs/tasks/ / docs/evidence/ by kind), keyed on a dated parenthetical qualifying a countable repo noun: the class is EXACTLY these two live-doc sites. The other hits are docs/decisions/*.md dated measurements, which are CORRECTLY exempt — a decision record states what was true when it was decided, which is its job, and that is the distinction the corrected rule captures. Also noted, NOT flagged: USER_GUIDE.md:466 "Exactly three knobs stay config-file-only" and book/src/knobs.md:1081 were already measured and deliberately KEPT by BOOK-TEST-COUNT-SHADOWS.2 as an authoritative enumeration of an exception; re-flagging them would be that tree's rejected false positive returning. NO REPAIR ATTEMPTED, deliberately: .2 deletes, .3 makes the corrected exemption rule durable. Checks: cargo check --all-targets clean; scripts/check_doctrines.sh 8/8 after git add. Docs-only => DUT byte-identical.`
  Commit: `pending`

- ID: `DATED-COUNT-SWEEP-EXEMPTION.2`
  Status: `pending`
  Goal: `Delete the two dated totals and CODEBASE_ANALYSIS.md's stale per-target claims at rung R1, replacing each with the derivation a reader can run; then re-sweep the live docs keyed on the CORRECTED rule and record what it finds.`
  Acceptance: `book/src/architecture.md:612 and CODEBASE_ANALYSIS.md:2423-2425 no longer assert a test count. architecture.md needs no replacement text — the three runnable derivations already sit 57 lines above at :545-550 — so the repair there is pure deletion; CODEBASE_ANALYSIS.md gets a pointer to those derivations rather than a second copy of them, since a second copy is the very shadow being removed (0033). The book_examples "54 runnable / 9 skip-sentineled" figures are resolved the same way OR the measurement is recorded explaining why they are not shadows. Re-sweep keyed on "is the enclosing record about the past or the present?" over the live docs, from the authoritative set, and record every hit AND every rejected false positive with its reason. mdbook build clean; check_doctrines.sh 8/8; docs-only.`
  Verification: `pending`
  Commit: `pending`

- ID: `DATED-COUNT-SWEEP-EXEMPTION.3`
  Status: `pending`
  Goal: `Make the corrected exemption rule durable, so the next sweep inherits it instead of re-deriving it — and decide whether it belongs in decision 0033, in a Knowledge Map card, or in both.`
  Acceptance: `The rule — a dated standing claim is not history, and an exemption must be keyed on whether the enclosing record speaks about the past or the present — is written where a future sweep will actually read it. Weigh: amending 0033 (it is the classification decision this refines, and 0033 already carries the three-part shadow test that both survivors PASS), versus a Knowledge Map fact card (question-keyed, so a sweep author retrieves it by asking "what is exempt from a shadow sweep?"), versus both. Note the precedent constraint: 0033 is CURRENT, so refining it is an amendment or a superseding record, not an edit — the same boundary 0038 hit. Also decide whether OVERFLOW-DESTINATION-INSTRUMENTATION's candidate instrument (b) — a self-declared date disagreeing with the file's newest content — should be adopted here as a shared mechanism rather than built twice; if so, coordinate rather than duplicate, since two trees building the same instrument is itself a 0033 shadow.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DATED-COUNT-SWEEP-EXEMPTION.1` | `done` | Audited, measured and registered; no repair attempted. The claims are **31 %** and **32 %** of reality. The first root-cause hypothesis (*"the sweep skipped the file it was editing"*) was **tested and refuted** — both files were in scope and both claims match the sweep's key. The real cause is the **date**: the sweep exempted dated measurements as honest history, and `.1` deleted every *undated* count in that file while leaving every *dated* one. |
| 2 | `DATED-COUNT-SWEEP-EXEMPTION.2` | `pending` | **Next.** Delete at R1 and re-sweep under the corrected rule. Bounded: the class is exactly two live-doc sites, and `architecture.md` needs no replacement text because the runnable derivations already sit 57 lines above the defect. |
| 3 | `DATED-COUNT-SWEEP-EXEMPTION.3` | `pending` | Make the corrected rule durable. This is the leaf that stops the class recurring — deleting two lines does not. Must also decide whether to share `OVERFLOW-DESTINATION-INSTRUMENTATION`'s self-declared-date instrument rather than build a second one. |

## Decisions

- `2026-07-31`: Registered as a **new tree** rather than by reopening the closed
  `BOOK-TEST-COUNT-SHADOWS`. That tree set the precedent itself, over
  `SHADOW-ENUMERATION-SWEEP`: *"history stays raw (decision `0031`), and a closed tree's
  claim is repaired by a new tree that measures it, not by rewriting the old one."*
- `2026-07-31` (`.1`): **The first root-cause hypothesis was written down, tested, and
  refuted before the tree was registered.** *"The sweep did not re-scan the file it was
  editing"* is the intuitive explanation and it is **wrong** — `BOOK-TEST-COUNT-SHADOWS.1`
  demonstrably examined both files. Recorded as refuted rather than silently replaced,
  because the project's standing rule is that a plausible-but-unmeasured cause is the
  thing that makes a repair miss its target.
- `2026-07-31` (`.1`): **The repair rung is fixed before the work starts** — `0033` **R1,
  deletion** — so the cheap wrong fix (*"just update the numbers"*) is not available later
  under time pressure. It is especially tempting here because the correct numbers are now
  measured and sitting in this file.
- `2026-07-31` (`.1`): **The tree's real subject is the exemption rule, not the two
  lines.** Deleting the survivors closes two instances; restating the rule as *past vs
  present* rather than *dated vs undated* closes the class. `.3` exists so `.2` cannot be
  mistaken for completion.

## Blockers

- None. `.2` is fully specified and bounded.

## Changelog

- `2026-07-31`: Created during session bootstrap, reading `book/src/architecture.md` as
  `SESSION_BOOTSTRAP.md` step 1 requires. The finding is not that two numbers are stale.
  It is that **the file contains the lesson and a live violation of it 57 lines apart** —
  `:543-555` explains that these counts *"had decayed — one by 72 %"* and that deriving
  them *"is the repair that cannot rot"*, while `:612` carries a claim decayed by **69 %**
  — and that the sweep which wrote that lesson had an **exemption** that the violation
  slipped through by being **dated**.
