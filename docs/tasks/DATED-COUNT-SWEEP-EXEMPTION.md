# DATED-COUNT-SWEEP-EXEMPTION: a date disguised a standing claim as history, and exempted it from the sweep built to catch it

## Metadata

- Tree ID: `DATED-COUNT-SWEEP-EXEMPTION`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` **done** — repaired at R1 in both files; the class turned out **6× larger** than `.1` registered and the root cause split into **two** failure modes; frontier `.3`)
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
  Commit: `ac7ffb6` — `DATED-COUNT-SWEEP-EXEMPTION.1 — the date is what carried it through`

- ID: `DATED-COUNT-SWEEP-EXEMPTION.2`
  Status: `done`
  Goal: `Delete the two dated totals and CODEBASE_ANALYSIS.md's stale per-target claims at rung R1, replacing each with the derivation a reader can run; then re-sweep the live docs keyed on the CORRECTED rule and record what it finds.`
  Acceptance: `book/src/architecture.md:612 and CODEBASE_ANALYSIS.md:2423-2425 no longer assert a test count. architecture.md needs no replacement text — the three runnable derivations already sit 57 lines above at :545-550 — so the repair there is pure deletion; CODEBASE_ANALYSIS.md gets a pointer to those derivations rather than a second copy of them, since a second copy is the very shadow being removed (0033). The book_examples "54 runnable / 9 skip-sentineled" figures are resolved the same way OR the measurement is recorded explaining why they are not shadows. Re-sweep keyed on "is the enclosing record about the past or the present?" over the live docs, from the authoritative set, and record every hit AND every rejected false positive with its reason. mdbook build clean; check_doctrines.sh 8/8; docs-only.`
  Verification: `done — REPAIRED, and .1's SCOPE WAS WRONG BY 6x, which measuring first is what caught. .1 registered "CODEBASE_ANALYSIS.md:2423-2425" — three lines. Enumerating that file against BOOK-TEST-COUNT-SHADOWS.1's OWN key returned FOURTEEN matches: the file carries a SECOND, LARGER COPY of the very per-file test-count list BTCS.1 deleted from book/src/architecture.md, and it was left completely untouched. Measured, all thirteen per-file claims: types.rs 40/40, validate.rs 26/26, cone.rs 42/43, gen/mod.rs 1/3, hierarchy.rs 6/6, module.rs 4/6, emit/sv.rs 17/26, metrics.rs 20/31, manifest.rs 3/3, microdesign 7/8, tool_matrix.rs 26/114 (+88, a 4.4x under-report and the worst instance in the repo), pipeline.rs 79/133, book_examples.rs 3/4 — NINE stale, all under-counts, plus the dated total 307 vs an actual 946 and "54 runnable / 9 skip-sentineled" vs 39 sentinels present. THE ROOT CAUSE IS REFINED, NOT REPLACED — .1's finding is exact WHERE IT WAS DERIVED and incomplete elsewhere, and both halves are recorded: (A) THE DATE, confirmed exactly for book/src/architecture.md — re-measured, ZERO undated counts survive in that file, only the dated total did, so within the file BTCS.1 actually repaired the date is the whole discriminator; (B) A SECOND, DISTINCT FAILURE MODE for CODEBASE_ANALYSIS.md, where BOTH dated (:2424, :2425) and UNDATED (:2422 26/114, :2423 79/133) counts survived, so the date explains nothing there. That file was SWEPT — BTCS.1's log proves it, rejecting "all 7 categories" with a reason — but it was JUDGED, not ENUMERATED: fourteen lines match its own key and none was surfaced. The reusable rule: A SWEEP THAT REPORTS ITS FINDS WITHOUT REPORTING ITS MATCH COUNT CANNOT BE AUDITED FOR RECALL. BTCS.1 recorded "found 2 more live shadows and three false positives" — precision reported, recall never — so a reader cannot tell whether it enumerated 5 candidates or 500. That is the exact sibling of CHANGES-ENTRY-PLACEMENT.3's finding that an extractor whose PATTERN is unrecorded is not reproducible; here it is an extractor whose MATCH SET is unrecorded. THE CLINCHING EVIDENCE THAT THESE WERE SHADOWS AND NOT FACTS, and it is better than any argument: THE TWO COPIES DISAGREED WITH EACH OTHER. For src/metrics.rs the book said 18, CODEBASE_ANALYSIS.md said 20, the truth is 31 — one derivable number, two copies, rotted to two DIFFERENT wrong values. And FOUR of the thirteen (types.rs, validate.rs, hierarchy.rs, manifest.rs) are ACCIDENTALLY CORRECT, including types.rs = 40, which is precisely the coincidence BOOK-TEST-COUNT-SHADOWS predicted and observed in the book copy when IR-TYPES-DECOMPOSITION.2 walked 42 down onto the stale number. A number that can become correct by coincidence carries no information (decision 0033). SCOPE WIDENED DELIBERATELY AND SAID SO: repairing 2 lines of a 14-line list would leave the file half-repaired and the class open, and it is ONE list, ONE shape, ONE rung — so .2 took all of it. This is the opposite of the /tmp sweep's error (widening a repair beyond its PROVEN defect); here every additional line was measured before it was touched. REPAIR AT RUNG R1, DELETION, in both files. book/src/architecture.md needed NO replacement text — the three runnable derivations already sat 57 lines above — so the total became a short paragraph explaining why no total is printed AND naming why this one survived (it was dated), which is the part a future editor needs. CODEBASE_ANALYSIS.md's thirteen bullets keep every word of their descriptions and lose only the numeral; its dated-total bullet is replaced by a pointer to the book's derivations plus the metrics.rs 18-vs-20-vs-31 divergence as the recorded evidence — a POINTER, never a second copy of the commands, since a second copy is the shadow being removed. ACCEPTANCE CHECK PROVEN NON-VACUOUS (0037): after the repair, the key returns ZERO hits in both files; re-inserting one synthetic count line makes it return 1, so green means clean rather than broken. RE-SWEEP under the CORRECTED rule ("is the enclosing record about the past or the present?") over 147 tracked *.md minus CHANGES/DEVELOPMENT_NOTES/docs-tasks/docs-evidence/docs-decisions by kind: NO live-doc site asserts a test count. Remaining hits are all past-tense records of a named leaf's completed action (MEMORY.md and docs/TASK_TREE.md rows narrating what THIS tree measured; TASK_TREE.md:131 "bumped 9 test assertions", :146 "13 test-fixture sites"), each checked individually and correctly exempt — which is the corrected rule working, since the OLD rule would have exempted them for the wrong reason (being dated) and the new one exempts them for the right one (being about the past). Checks: mdbook build clean; cargo test --test book_examples green; cargo check --all-targets clean; scripts/check_doctrines.sh 8/8 after git add (book/src/architecture.md is a declared ENUMERATION-PARITY doctrine-ids site, re-checked per the recorded gotcha about grepping the doctrine checks for a file before touching it — the edit is 100+ lines below the fence and parity holds). Docs-only => DUT byte-identical.`
  Commit: `a3c446d` — `DATED-COUNT-SWEEP-EXEMPTION.2 — the second copy was six times bigger`

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
| 2 | `DATED-COUNT-SWEEP-EXEMPTION.2` | `done` | Repaired at **R1** in both files — and **`.1`'s scope was wrong by 6×**, which measuring before editing is exactly what caught. `.1` registered three lines in `CODEBASE_ANALYSIS.md`; enumerating that file against `BOOK-TEST-COUNT-SHADOWS.1`'s **own key** returned **fourteen** — the file holds a **second, larger copy of the very list** that tree deleted from the book, left untouched. **Nine of thirteen** per-file claims stale, worst `tool_matrix.rs` **26 → 114**. Root cause **refined, not replaced**: the date is exact for `architecture.md` (zero undated counts survive there) but explains nothing in `CODEBASE_ANALYSIS.md`, where undated counts survived too — that file was **judged, not enumerated**. Clincher: the two copies **disagreed with each other** (`metrics.rs`: book 18, this file 20, truth 31). |
| 3 | `DATED-COUNT-SWEEP-EXEMPTION.3` | `pending` | **Next.** Make the corrected rule durable. This is the leaf that stops the class recurring — deleting fifteen lines does not. `.2` gives it **two** rules to land, not one: (i) an exemption must be keyed on *past vs present*, never on *dated vs undated*; (ii) **a sweep must record its match count, not only its finds**, or its recall cannot be audited — the sibling of `CHANGES-ENTRY-PLACEMENT.3`'s unrecorded-extractor finding. Must also decide whether to share `OVERFLOW-DESTINATION-INSTRUMENTATION`'s self-declared-date instrument rather than build a second one. |

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
- `2026-07-31` (`.2`): **`.1`'s root cause is REFINED, not replaced, and both halves are
  kept.** The date is the exact discriminator **in the file `BOOK-TEST-COUNT-SHADOWS.1`
  actually repaired** — re-measured, zero undated counts survive in `architecture.md`. It
  explains **nothing** in `CODEBASE_ANALYSIS.md`, where undated counts survived too. So
  there are **two** failure modes, not one, and the second is the more dangerous:
  **that file was judged, not enumerated.** Fourteen lines matched the sweep's own key and
  none was surfaced. Recorded here rather than edited into `.1`, which is layer-B history.
- `2026-07-31` (`.2`): **New rule, and it is `.3`'s second deliverable — a sweep must
  record its match count, not only its finds.** `BOOK-TEST-COUNT-SHADOWS.1` reported *"found
  2 more live shadows and three false positives"*: precision reported, **recall never**. A
  reader cannot tell whether it examined 5 candidates or 500, so its coverage claim was
  unfalsifiable. This is the exact sibling of `CHANGES-ENTRY-PLACEMENT.3`'s finding that an
  extractor whose *pattern* is unrecorded is not reproducible — here it is an extractor
  whose *match set* is unrecorded. Two trees, two days, one shape: **an instrument is only
  as trustworthy as what it records about itself.**
- `2026-07-31` (`.2`): **Scope widened from 3 lines to 15, deliberately, and said so.**
  Repairing 2 of a 14-line list would leave the file half-repaired and the class open, and
  it is one list, one shape, one rung. This is *not* the `/tmp` sweep's error of widening
  beyond a proven defect: every additional line was **measured before it was touched**, and
  the four accidentally-correct ones were identified as such rather than silently "fixed".
- `2026-07-31` (`.2`): **The decisive evidence is that the two copies disagreed with each
  other.** For `src/metrics.rs` the book said **18**, `CODEBASE_ANALYSIS.md` said **20**,
  the truth is **31**. One derivable number, two copies, two *different* wrong values. No
  argument about shadows is needed after that; and it is why the replacement text records
  the divergence rather than just deleting the numbers.

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DATED-COUNT-SWEEP-EXEMPTION.2` | `a3c446d` — `DATED-COUNT-SWEEP-EXEMPTION.2 — the second copy was six times bigger` | Repaired 15 lines across two files at **R1**. `.1`'s scope was wrong by 6×; the widened measurement, the refined two-part root cause, and the new *record-your-match-count* rule are all recorded in the Decisions and Verification Log above. |
| `DATED-COUNT-SWEEP-EXEMPTION.1` | `ac7ffb6` — `DATED-COUNT-SWEEP-EXEMPTION.1 — the date is what carried it through` | Registration only; **no repair attempted**, deliberately. The leaf's work product is the measurement plus the refuted first hypothesis. |

## Blockers

- None. `.3` needs no new input; `.2` handed it two measured rules and a coordination note.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `DATED-COUNT-SWEEP-EXEMPTION.2` | `Enumerated CODEBASE_ANALYSIS.md against BTCS.1's own key: 14 matches, not the 3 .1 registered. All 13 per-file claims measured: 9 stale, all under-counts (worst tool_matrix.rs 26 -> 114); 4 accidentally correct incl. types.rs 40. Dated total 307 vs actual 946 (751 lib + 195 integration). Repaired at R1 in both files; acceptance key returns 0 hits after, and 1 hit on a synthetic re-inserted count, so the check is proven non-vacuous. Re-swept under the corrected past-vs-present rule: no live-doc site asserts a test count; every remaining hit is a past-tense record of a named leaf's completed action, each checked individually. mdbook build clean; cargo test --test book_examples green (4 tests); cargo check --all-targets clean; check_doctrines.sh 8/8 after git add, with book/src/architecture.md re-checked as a declared ENUMERATION-PARITY site` | `class closed in both files; .1's root cause refined into two distinct failure modes; scope widened 3 -> 15 lines on measurement and recorded as such` |
| `2026-07-31` | `DATED-COUNT-SWEEP-EXEMPTION.1` | `measured at 349eeb6 with the three derivations book/src/architecture.md:545-550 publishes; first root-cause hypothesis tested and refuted; class swept from the authoritative set (147 tracked *.md)` | `defect confirmed, pre-existing, live; registered without repair` |

## Changelog

- `2026-07-31`: Created during session bootstrap, reading `book/src/architecture.md` as
  `SESSION_BOOTSTRAP.md` step 1 requires. The finding is not that two numbers are stale.
  It is that **the file contains the lesson and a live violation of it 57 lines apart** —
  `:543-555` explains that these counts *"had decayed — one by 72 %"* and that deriving
  them *"is the repair that cannot rot"*, while `:612` carries a claim decayed by **69 %**
  — and that the sweep which wrote that lesson had an **exemption** that the violation
  slipped through by being **dated**.
