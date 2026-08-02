# BOOK-PARAGRAPH-BLOBS: the rendered book has paragraphs that are walls of text — one is 22,908 characters

## Metadata

- Tree ID: `BOOK-PARAGRAPH-BLOBS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / book fidelity
- Created: `2026-08-02`
- Last updated: `2026-08-02` (`.4` instruments durable; `.3a` applied `0048` to the three verified duplications; frontier `.3b`)
- Owner: repo-local live-doc hygiene

## Goal

**Owner finding, `2026-08-02`:** *"Make sure that paragraphs in the book are not stitched together as
one big blob in sections or chapters… It is really painful to read like this. This visual defect
appears of course in the rendered HTML version of the book."*

The owner reviews the **book**, not the code (`COMMIT.md` §9), so a readability defect in the
rendered HTML is a defect in the only surface they see.

**Measured at `9060993` before registering**, over `mdbook build book` output, `<main>` only,
`print.html` and `404.html` excluded as generated chrome:

| Chapter | worst rendered `<p>` | mean `<p>` | `<p>` count |
| --- | --- | --- | --- |
| `architecture.html` | **22,908 chars** | 1,234 | 26 |
| `hierarchy.html` | 4,371 | 420 | 91 |
| `ir.html` | 3,211 | 402 | 45 |
| `knobs.html` | 1,655 | 432 | 102 |

**Denominator: 1,244 rendered paragraphs across 30 chapters. Exactly 6 exceed 1,500 characters** —
`architecture` ×1, `hierarchy` ×1, `ir` ×3, `knobs` ×1. The book is *not* broadly afflicted; the
defect is narrow, and one case is catastrophic. `architecture.md`'s blob is **lines 658–903 — 246
consecutive lines with no blank line between them**, which is roughly **71 % of that chapter's entire
prose in a single `<p>`**.

## Why it happened — and why it is the same mechanism `UNGATED-PRACTICE-AUDIT.1` just measured

The blob is an **accretion**, not one bad edit. Reading it, successive slices each appended one more
sentence to a paragraph that already existed — *"The dedicated Phase 2 sharing gate is now closed
too…"*, *"The dedicated Phase 3 structured-surface gate is now closed as well…"* — over months.

Every one of those edits was correct in isolation and its diff was three lines long. **Nothing an
author is forced to do reports how long the paragraph has become.** That is exactly the refined rule
landed at `UNGATED-PRACTICE-AUDIT.1` (`0e4654f`): *a practice survives where its output is a
by-product of work the author is already forced to do*. Paragraph length is a by-product of nothing —
not of the diff, not of `mdbook build` (which exits `0`), not of any doctrine. It is the first
independent instance of that rule since it was written, arriving from an owner rather than an audit.

## Non-Goals

- **Not a rewrite.** `.1` inserts blank lines at genuine topic boundaries and changes **no wording**.
  A whitespace-only repair is reviewable at a glance and trivially reversible; a rewrite is neither,
  and the owner asked for readability, not new content.
- **Not a line-length or hard-wrap change.** The source is hard-wrapped and that is fine; the defect
  is the absence of blank lines *between* paragraphs, not the presence of newlines *within* one.
- **Not a mandate to gate this.** Whether a check is warranted is `.2`'s question, and per decision
  `0047` plus `UNGATED-PRACTICE-AUDIT.1`, *removing the need* outranks *watching harder*.
- **No `src/` change.** Book-only ⇒ DUT byte-identical.

## Acceptance Criteria

- `.1` repairs every block over 1,500 characters that a blank line **can** fix, with **zero wording
  changes**. *Restated at execution against the corrected census (**11** over-threshold blocks, not
  6): 11 are splittable and were repaired; **4 are run-on enumerations** that no blank line can
  split, and they moved to `.3` rather than being counted as done.* The proof of "zero wording
  changes" was also strengthened — see Findings.
- The census is **re-run after the repair** and the new distribution reported, so the claim is
  measured rather than asserted.
- `mdbook build` stays green and `scripts/check_doctrines.sh` stays 11/11; book-only ⇒ DUT
  byte-identical, `tests/snapshots.rs` untouched. *`mdbook test` was dropped from this list at `.1`:
  it fails identically on a clean `HEAD` under local mdBook v0.5.2 while CI pins v0.4.40 — a
  pre-existing defect this tree does not own and must not be held to. See* **Surfaced by `.1`**.
- `.2` decides whether anything should watch this, and states the by-product alternative it rejected
  before proposing a gate. **"No gate"** is a legitimate and fully acceptable outcome.

## Task Tree

- ID: `BOOK-PARAGRAPH-BLOBS`
  Status: `active`
  Goal: `The rendered book has no wall-of-text paragraphs, and the repair is whitespace-only.`
  Children: `.1` (repair the splittable blobs, **done**), `.4` (make the instruments durable, **done**), `.3a` (link the verified duplications, **done**), `.3b` (the capability roll-calls, which are *not* duplications, **done**), `.3c` (the twice-carried fourteen-query `analyze` roll-call, **done**), `.2` (decide whether anything watches this)

- ID: `BOOK-PARAGRAPH-BLOBS.4`
  Status: `done`
  Goal: `Promote the measuring instruments out of gitignored target/tmp/book-blob/ into scripts/, so the census that judges this tree survives the next session.`
  Acceptance: `The promoted census reproduces the validated instrument's numbers EXACTLY (denominator, over-threshold count, and every block's size) — a promotion that changes the measurement is a new instrument, not a promotion. It additionally reports the source anchor and a REPAIRABILITY class per block, because .3 needs to know which blocks a blank line structurally cannot fix. Negative-controlled: a sabotage of the book must move the census. Documented in TOOLBOX.md Part 1. Deliberately NOT a doctrine check — whether anything gates paragraph size is .2's question.`
  Verification: `Exact reproduction: 3,501 prose blocks / 30 chapters / 12 over 1,500 / oversized mass 43,092 and all 12 block sizes identical to target/tmp/book-blob/census2.py; book_list_signature.py reproduces .1's 1,325 <li> across 31 chapters. All 12 source anchors resolve (the throwaway instrument left 4 unresolved). THREE controls, all through scripts/negative_control.sh so every mutation's count is asserted: (1) merge two paragraphs -> census FIRES, 3,501->3,500 blocks, 12->13 over threshold, mass 43,092->44,919; (2) re-wrap a line inside a paragraph -> census SILENT on the measurement fields, proving it measures RENDERED length not source lines; (3) a paragraph break inside a list item with no continuation indent -> list signature FIRES (knobs.html content SHA), word proof reports OK (90,542 -> 90,542) — the complementarity, re-proven with the promoted instruments. Restores cmp-verified; book rebuilt clean.`
  Commit: `d25bbe7`

- ID: `BOOK-PARAGRAPH-BLOBS.1`
  Status: `done`
  Goal: `Split the rendered prose blocks over 1,500 characters at genuine topic boundaries by inserting blank lines only.`
  Acceptance: `The diff adds blank lines and changes no words — ORIGINAL WORDING SUPERSEDED at execution, see Findings: "git diff --ignore-blank-lines shows no content hunks" is too narrow, because a sentence that begins mid-line cannot be split without moving the line break. Replaced by a stronger proof: whitespace-normalized word identity, plus a rendered list-structure proof. The post-repair census is re-run and reported against the pre-repair one. mdbook build green.`
  Verification: `Worst prose block 22,908 -> 8,704 chars (-62.0 %); architecture.html 22,908 -> 7,043 (-69.3 %); oversized mass 54,897 -> 43,092 (-21.5 %). Both proofs pass and both are negative-controlled. 24 paragraph breaks inserted across 5 chapters, zero words changed.`
  Commit: `df7bc6e`

- ID: `BOOK-PARAGRAPH-BLOBS.3`
  Status: `split at execution into .3a (done) and .3b (pending)`
  Goal: `Decide what to do about the RUN-ON ENUMERATIONS — the residue .1 structurally cannot fix, because a single sentence has no paragraph boundary to insert a break at.`
  Acceptance: `Registered by .1 as a distinct defect, not a leftover. Every remaining oversized block is one sentence listing dozens of clauses. OWNER DECIDED 2026-08-02: replace the duplicated prose with an ABSOLUTE GitHub URL to the .md that already owns it - not a list conversion, and not the deletion the agent proposed. The link MUST be absolute (https://github.com/rdje/anvil/blob/main/...); a relative ../../X.md renders as a dead .html and BOOK-LINK-TARGETS blocks it (decision 0046). Precondition VERIFIED before any content is removed - see Decisions.`
  Verification: `SPLIT, because the precondition was measured per block rather than assumed for the class — see Findings (.3a). 0048's route applies only where the block is a lossy COPY of an owning file, and that is true of 3 of the 9 blocks, not all of them. Executing one repair across all nine would have deleted book-only prose under a decision that never authorised it.`
  Commit: `see .3a / .3b`

- ID: `BOOK-PARAGRAPH-BLOBS.3a`
  Status: `done`
  Goal: `Apply 0048 to the blocks whose content is a VERIFIED lossy copy of a file that already owns it: the two per-bank rN registers and the saw_* coverage roll-call.`
  Acceptance: `Per-block precondition measured, not sampled, BEFORE any deletion: every id/flag removed must be reachable elsewhere, and the exact residue stated rather than discovered later. Links absolute per 0048; zero relative .md escapes; BOOK-LINK-TARGETS green; the target URL returns 200. The <li> count must be UNCHANGED — one of the three sites is inside a list item, and .1's regression was text escaping its <li>. Census re-run and reported.`
  Verification: `Oversized blocks 12 -> 9; worst rendered block 8,704 -> 4,419 chars (-49.2 %); oversized mass 43,092 -> 21,954 (-49.1 %). Precondition, measured exhaustively: 73 of 77 bank ids are glossed in ROADMAP.md (r7, r8, r11, r12 are not - r7 is still mentioned there, r8's lesson is KEPT in both chapters, r11 and r12 survive in CHANGES.md x25/x18 and CODEBASE_ANALYSIS.md); 77 of 77 saw_* flags remain defined in src/, and 62 of 77 are still named elsewhere in the book. Nothing left the repository. Structure: n_li 1,325 UNCHANGED across 31 chapters, only hierarchy.html's content SHA moved - the exact signature of text removed from inside a list item without escaping it. 3 absolute links render with .md intact; 0 relative .md escapes; https://github.com/rdje/anvil/blob/main/ROADMAP.md returns HTTP 200. mdbook build exit 0; scripts/check_doctrines.sh 11/11.`
  Commit: `a385b76`

- ID: `BOOK-PARAGRAPH-BLOBS.3b`
  Status: `done`
  Goal: `Repair the CAPABILITY ROLL-CALLS — the run-on sentences that are NOT duplications, so 0048's link route does not apply to them — and give the four non-prose blocks a recorded verdict.`
  Acceptance: `Measured first: clause-level overlap with ROADMAP/CODEBASE_ANALYSIS/USER_GUIDE/CHANGES is 53-84 %, so a link-and-delete would lose book-only prose. The candidate repair is a MARKDOWN LIST - it changes structure, not wording, and .1 named it as the natural repair while placing it outside its own whitespace-only scope. Clause preservation must be PROVEN (extract the clause set before and after and compare), not asserted, and the <li> census must account for the new items. The two <td> blocks and the two guard-declined splittables get a stated verdict, not silence. STRENGTHENED at execution: the clause SET comparison named here is blind to REORDERING, and a reordered capability register is a changed claim about what a gate proved - replaced by an order-sensitive word-SEQUENCE identity modulo separators and connectives. VERDICT SCOPE CORRECTED: the acceptance named 7 distinct blocks of the 9 that existed; all 9 got a verdict.`
  Verification: `Oversized blocks 9 -> 4; worst rendered block 4,419 -> 3,071 chars (-30.5 %); oversized mass 21,954 -> 8,210 (-62.6 %). All FOUR capability roll-calls are gone from the over-threshold set: architecture.html's worst block is now 1,362 (was 4,419) and hierarchy.html's is 1,618 (a metric-name <li>, not a roll-call). CLAUSE PRESERVATION PROVEN, not asserted, by the new scripts/prove_clauses_unchanged.py: architecture.md 43,542 -> 43,542 and hierarchy.md 101,051 -> 101,051 normalized chars, byte-identical sequences. Four negative controls through scripts/negative_control.sh, every substitution count asserted: drop a clause -> FIRES; reword a clause -> FIRES; REORDER two adjacent clauses -> FIRES (the property a set comparison would have missed); insert a connective -> SILENT, the stated blind spot, with the carrier verified by reading the mutated file rather than by a second instrument (the word proof is saturated by the conversion itself and cannot discriminate). STRUCTURE FULLY RECONCILED: prose blocks 3,501 -> 3,671 (+170) = +168 <li> +2 <p>, and every term is accounted (57+26+56+29 = 168 new list items; +1 <p> for the sandwich tail that keeps "remain useful targeted evidence." verbatim, +1 <p> for the ir.md split; the four mid-sentence repairs are block-count neutral because three MOVE a break and the fourth merges two blocks that the list lead-in re-creates). book_list_signature.py moved in exactly the 2 edited chapters and no others; ir.md's split left <li> UNCHANGED, proving the continuation did not escape its item. ir.md whitespace-only, prove_words_unchanged.py OK (30,768 -> 30,768). mdbook build exit 0; scripts/check_doctrines.sh 11/11.`
  Commit: `afb9847`

- ID: `BOOK-PARAGRAPH-BLOBS.3c`
  Status: `done`
  Goal: `Repair the FOURTEEN-QUERY analyze roll-call, which the book carries TWICE and which is now the whole of the book's remaining oversized mass above the recorded acceptances.`
  Acceptance: `Measured at .3b before registering: all fourteen query names appear in agent-mcp.md, api-introspection.md, TOOLBOX.md and docs/AGENT_INTROSPECTION_SCHEMA.md, and the book itself already names the SCHEMA file as canonical - so this is 0033's shadow shape. But the two book copies carry book-only editorial glosses ("what does this output depend on?") that the canonical file does not, so 0048's link-and-delete would lose prose, exactly as .3a measured for the capability roll-calls. The repair is therefore STRUCTURAL: lift the fourteen descriptions out of the <td> (a GFM pipe-table cell cannot hold a block-level list, so no list conversion is possible in place). Whether the third copy should link rather than duplicate is part of this leaf, not assumed. STRENGTHENED at execution: the book-only precondition was re-measured PER GLOSS rather than inherited from .3b's per-class claim, and the first measurement of it was WRONG - a line-wise grep over a hard-wrapped book undercounts, so it was redone whole-file and whitespace-normalized.`
  Verification: `Oversized blocks 4 -> 2; worst rendered block 3,071 -> 1,901 chars (-38.2 %); oversized mass 8,210 -> 3,519 (-57.1 %). BOTH remaining blocks are .3b's RECORDED ACCEPTANCES, so the book now holds zero unaccepted oversized blocks. Precondition measured per gloss, whole-file and whitespace-normalized: 10 of the <td>'s 14 editorial glosses exist NOWHERE else in the repository (4 recur once each, in agent-mcp.md's own per-query sections), so 0048's link-and-delete would have destroyed 10 pieces of book-only prose - the route was declined on measurement, not on inheritance. THE LIFT IS PROVEN IN TWO HALVES because no single existing instrument can express it: (1) CONTENT - the fourteen items extracted from the before-<td> and from the after-list are byte-identical in sequence, 14/14; (2) REMAINDER - the file with that run excised on both sides is identical, 54,791 = 54,791 normalized chars, modulo two stated additions (136 chars). api-introspection.md's 11-bullet conversion needs no such split and passes the plain clause proof OUTRIGHT: 11,593 normalized chars, byte-identical sequence. New --allow-move mode on scripts/prove_clauses_unchanged.py reports agent-mcp.md +15 words and ZERO REMOVED, every added word named, and api-introspection.md word-multiset IDENTICAL (1,648). FOUR negative controls, each with its substitution count asserted: drop a query -> FIRES; reword -> FIRES; invent a word -> FIRES; REORDER two adjacent bullets -> SILENT under --allow-move and FIRES under the default mode, the complementarity proven on ONE unsaturated carrier. STRUCTURE RECONCILED TERM BY TERM: prose blocks 3,671 -> 3,698 (+27) = +14 <li> (the lifted items) +11 <li> (the schema bullets) +1 <p> (the list lead-in) +1 <p> (the section opening split); book_list_signature.py 1,493 -> 1,518 <li> = +25 = 14 + 11, moving in exactly the 2 edited chapters of 31 and no others. Zero mid-sentence paragraph breaks book-wide (0 of 2,190), so .1's defect was not reintroduced. Anchor #derived-relation-queries-analyze verified present in the rendered HTML. mdbook build exit 0; scripts/check_doctrines.sh 11/11; cargo test --test book_examples 4 passed / 0 failed.`
  Commit: `pending`

- ID: `BOOK-PARAGRAPH-BLOBS.2`
  Status: `pending`
  Goal: `Decide whether paragraph size should be watched, and by what — a gate, a by-product route, or a recorded acceptance.`
  Acceptance: `States the by-product alternative considered before any gate is proposed. Over-gating is a defect in its own right (DOCTRINE_ENFORCEMENT.md §9, decision 0033 test (2)). "Recorded acceptance" is a legitimate outcome; a threshold, if any, must be derived rather than fitted.`
  Verification: `pending`
  Commit: `pending`

## Findings (`.1`, measured `2026-08-02`)

### The registered census was wrong, and the correction is larger than the finding

`.0` reported **6 of 1,244**. Both numbers were produced by a regex that matched `<p>…</p>`
non-greedily across the rendered HTML, which **spans code blocks** — so it merged and inflated
elements and never looked outside `<p>` at all. Rebuilt on `html.parser`, walking block containers
under `<main>` and **excluding `<pre>`** (code is meant to be long; counting it as prose is what made
the first instrument report a 7,913-character "paragraph" that was a Rust struct definition):

| | `.0` (wrong) | `.1` (corrected) |
| --- | --- | --- |
| denominator | 1,244 | **3,467** prose blocks, 30 chapters |
| over 1,500 chars | 6 | **11** |
| elements counted | `<p>` only | `<p>`, `<li>`, `<blockquote>`, `<td>` |

The `<p>`-only count of **6 was coincidentally right**; the denominator was wrong by 2.8×, and **5
oversized blocks were invisible** — including a **10,781-character `<li>`** in `hierarchy.html`, the
second-worst blob in the book. `.0`'s Open Question 3 had named this exact limit (*"long list items
and table cells render outside `<p>` and are not counted… a stated limit, not a measured absence"*),
which is why it was measured rather than inherited.

### The blob is two defects, not one — and only one is whitespace-fixable

Classifying every oversized block by *how many sentence boundaries it contains*:

- **11 splittable** — genuinely several sentences run together with no blank line. `.1` repairs these.
- **4 run-on enumerations** — **one sentence** listing dozens of clauses (*"The old `r7` report is
  now the historical wrapper-baseline artifact, `r10` is …, `r11` is …"* for 7,159 characters).
  **Whitespace cannot split a sentence.** The natural repair is a markdown list, which changes
  structure and wording — outside `.1`'s stated constraint, so it is registered as `.3` rather than
  smuggled in. Saying *"I fixed the paragraphs"* while these remain would be false.

### Result

| Metric | before | after | Δ |
| --- | --- | --- | --- |
| worst prose block | **22,908** | **8,704** | **−62.0 %** |
| `architecture.html` worst | 22,908 | 7,043 | −69.3 % |
| total mass in oversized blocks | 54,897 | 43,092 | −21.5 % |

24 paragraph breaks across 5 chapters (`architecture`, `hierarchy`, `ir`, `knobs`,
`api-introspection`), **zero words changed**. The mass figure moves far less than the worst-case
figure, and that is the honest shape of the result: what remains is almost entirely the four run-on
enumerations `.3` owns. The *count* of oversized blocks went **11 → 12**, which is not a regression —
splitting one 10,781-character block yields two blocks that are both still over the threshold. A
count is the wrong metric here; worst-case and mass are the right ones.

### The acceptance criterion was wrong and was corrected in flight

`.0` promised *"a diff that adds only empty lines"*. That is unachievable and would have produced a
worse book: the sentences that begin a new topic mostly begin **mid-line** in hard-wrapped source, so
splitting there requires moving a line break. Held to the letter, the criterion would have forced
breaks only at the three places a sentence happened to start a line.

Replaced with a **stronger** proof, not a weaker one — `scripts`-free, in
`target/tmp/book-blob/prove_words_unchanged.py`: collapse each file's whitespace to single spaces and
require the result to be **byte-identical** before and after. That permits arbitrary re-wrapping and
forbids any word change. Negative-controlled: it passes on the real edit and **fails** on a
one-character change (`tests` → `testz`), with the divergence located and printed.

### A real regression, caught only because a second proof was added

The first pass dropped the **two-space indentation** when splitting inside a markdown list item. The
words were identical, the word proof passed — and the continuation had silently **escaped its `<li>`**
and been promoted to a top-level paragraph, ending the list. That is a structural change to the
rendered document, and **the word-identity proof is blind to it by construction**, because it
collapses exactly the whitespace that carries list nesting.

Repaired two ways: the splitter now preserves the block's continuation indent, and a **second,
independent proof** compares the rendered `<li>` census — count plus a SHA of every item's
whitespace-normalized text — before and after.

**Both proofs are negative-controlled, and the control had to be run twice.** The first attempt's
sabotage **refused to apply** (`0 matches`, because the targeted cut was at indent 0, not 2) and
printed `REFUSED` rather than a verdict — the `NEGATIVE-CONTROL-HARNESS.1` trap avoided by a guard
that asserts its substitution count. Retargeted at a genuinely indented cut, the sabotage **landed**
(1 substitution, asserted) and the result is decisive:

| Proof | on the sabotage | verdict |
| --- | --- | --- |
| rendered `<li>` signature | **FIRES** — `hierarchy.html`, `n_li` unchanged at 476, content SHA changed | catches text escaping a list item |
| whitespace-normalized words | **passes** | structurally blind to it — alone it would have shipped the regression |

Restored with `cmp`-verified identity and rebuilt clean.

## Findings (`.4`, measured `2026-08-02`)

### The promotion had to be checked as a promotion, not accepted as a rewrite

The first draft of `scripts/book_prose_census.py` joined a block's text chunks with `" "` instead of
`""`. The denominator (**3,501**) and the over-threshold count (**12**) matched the validated
instrument exactly, and **nine of twelve block sizes did not** — every block containing an inline
`<code>` was inflated by one character per inline element (5,391 → 5,468; 3,071 → 3,123). A browser
renders `<code>x</code>,` with no space, so the joined-with-`""` reading is the true one.

**The count matching is what makes this dangerous.** The two numbers a reader checks first agreed, and
the measurement underneath was wrong. `.4`'s acceptance criterion demanded *every block's size*
precisely so this could not pass, and it is the only reason the defect was caught.

### A negative control's count assertion proves the experiment RAN — not that it ran the INTENDED experiment

The first list-structure control dedented a list-item continuation and expected the text to escape its
`<li>`. `scripts/negative_control.sh` asserted the substitution count, reported `applied`, and the
proof stayed **silent** — which reads exactly like a blind instrument.

Root-caused rather than classified: the regex captured `(\n)` before the indented line, so it deleted
the **blank line** as well as the indent. mdBook then treated the dedented text as a *lazy
continuation* of the preceding paragraph — still inside the same `<li>`. **List structure genuinely
did not change, so the instrument was right and the carrier was wrong.**

This refines decision [`0047`](../decisions/0047-negative-control-carrier-is-the-mutation.md) rather
than contradicting it. `0047`'s guard catches *"the substitution matched nothing"*. It cannot catch
*"the substitution matched, and mutated something other than what you meant"* — the count is 1 either
way. The second failure mode is quieter, because it produces a verdict that looks like a finding
about the check. **A control's expectation must be checked against the rendered artifact, not against
the author's intent for the regex.** Corrected carrier (preserve the blank line, strip only the
indent) → **FIRES**, as `.1` recorded.

### `.1`'s complementarity claim is carrier-sensitive, and the carrier is now recorded

Re-proving *"the word proof is blind to a list escape"* with the promoted instruments failed on the
first carrier: the dedent also removed the single space between `construct.` and `**Default`, and the
word proof **fired** (90,542 → 90,541 normalized chars). That is correct behaviour — a changed
inter-word space *is* a change it should see.

The blindness needs the carrier `.1` actually used: a **paragraph break inserted inside a list item
with no continuation indent**. Both a blank line and a single newline normalize to one space, so the
word count is untouched:

| Carrier | word-identity proof | list-structure proof |
| --- | --- | --- |
| dedent an existing continuation (also eats an inter-word space) | **FIRES** (90,542 → 90,541) | silent (lazy continuation — nothing escaped) |
| **break inside `<li>`, no continuation indent** (`.1`'s regression) | **OK** (90,542 → 90,542) | **FIRES** (`knobs.html` content SHA) |

The claim *"these two proofs are complementary"* is only true of the second row. Recorded here with
its carrier, because a reproducible claim needs the mutation, not just the verdict.

### The census reports two different kinds of fact, and only one is invariant

A control that re-wrapped a source line inside a paragraph — which cannot change the rendered book —
**fired** against the full census JSON. Not a defect: the JSON carries a *measurement* (sizes, counts,
mass) and a *source locator* (`chapter.md:line`), and re-wrapping legitimately moves line numbers.
Compared on the measurement fields alone, the same mutation is correctly **silent** while the
paragraph-merge mutation still **fires** — so the instrument discriminates rather than merely being
able to fire (`DOCTRINE_ENFORCEMENT.md` §9). Anyone diffing two census runs must compare the
measurement, not the whole document.

## Findings (`.3a`, measured `2026-08-02`)

### `.1` called the residue one defect class. Measured per block, it is two — and `0048` authorises the repair for only one of them

`0048` admits an absolute GitHub link **only where the link is the sole route to content the book
deliberately removed** — content that demonstrably lives elsewhere. `.1` labelled every surviving
block a *run-on enumeration* and `0048` was written expecting the whole class to be a lossy copy of
`ROADMAP.md`. Checking each block instead of the class:

| Block | what it is | lives elsewhere? | route |
| --- | --- | --- | --- |
| `hierarchy.md` 8,704 · `architecture.md` 7,043 | the per-bank `rN` **register** | **73 / 77** ids glossed in `ROADMAP.md` | `0048` link ✔ |
| `architecture.md` 5,391 | the `saw_*` **coverage roll-call** | **77 / 77** flags defined in `src/` | `0048` link ✔ |
| `architecture.md` 4,419 · `hierarchy.md` 3,763 · 2,076 · `architecture.md` 1,934 | **capability roll-calls** — *what a gate proved* | only **53–84 %** of clauses appear verbatim in any other live doc | **link would delete book-only prose ✘** |

**That 53–84 % is the whole finding.** A link-and-delete across the class would have removed prose
that exists nowhere else, under a decision whose stated precondition is that it exists somewhere
else. The first three are genuine shadows and `0033`'s R1 applies; the rest are not shadows, they are
badly-formatted original content, and their repair is structural. Split into `.3a` / `.3b` on that
measurement rather than executed as one sweep.

### The copy was not merely redundant — it was already wrong

`architecture.md`'s register closed with *"`r85` is the current hierarchy full bank"* while the same
chapter names **`r87`** as the closing gate a few lines above; `hierarchy.md` did the same. So the
duplicate had drifted from its own chapter, not just from `ROADMAP.md`. That is
[`0033`](../decisions/0033-shadow-enumeration-classification.md)'s failure mode caught in the act,
and it makes the removal a **correctness** repair, not only a readability one.

### What the book actually lost, stated rather than discovered later

Measured exhaustively after the edit, not sampled before it:

- **Bank ids: 73 of 77 are glossed in `ROADMAP.md`.** The four that are not — `r7`, `r8`, `r11`,
  `r12` — are not lost: `r7` is still mentioned in `ROADMAP.md`; **`r8`'s gloss was deliberately
  kept in both chapters**, because it records a *lesson* (the Phase 4 gate must use a
  hierarchy-focused sequential leaf profile) rather than a breadcrumb; `r11` and `r12` survive in
  `CHANGES.md` (×25 / ×18) and `CODEBASE_ANALYSIS.md`.
- **Coverage flags: 77 of 77 remain defined in `src/`**, and 62 of 77 are still named elsewhere in
  the book.

**Nothing left the repository.** Four one-line glosses no longer appear in `ROADMAP.md` specifically,
which is the stated cost.

### The `<li>` census is what makes this edit safe to claim

One of the three sites sits **inside a list item**, and `.1`'s shipped regression was a continuation
losing its indent and escaping its `<li>`. After the edit `n_li` is **1,325 across 31 chapters,
unchanged**, with only `hierarchy.html`'s content SHA moving — precisely the signature of *text
removed from inside a list item without escaping it*. Had the indent been dropped, the count would
have moved and the word proof would not have noticed
([[matched-mutation-is-not-the-intended-mutation]]).

### `CODEBASE_ANALYSIS.md` carries a third copy of the register, and it is the most current one

Surfaced, not repaired: the same `rN` roll-call sits at `CODEBASE_ANALYSIS.md` as a single wall of
text, and it runs to **`r87`** — further than either book copy did. Out of scope here (this tree owns
the **book**), and recorded under *Surfaced by `.3a`* so it is not rediscovered.

## Findings (`.3b`, measured `2026-08-02`)

### `.1` shipped four paragraph breaks **inside a sentence**, and every instrument in the kit was
### either blind to them or **rewarded** them

Opening the 4,419-character block to convert it, its rendered text began `modes). That report
banks…` — mid-clause. `git show df7bc6e` confirms `.1` inserted the blank line **before** the
sentence-final token rather than after it, four times:

```markdown
840/0 pass-fail in Verilator plus both repo-owned Yosys      <- paragraph ended here

modes). That report banks wrapper exact / reuse / …          <- next paragraph began here
```

`Yosys` / `modes).` once, and `downstream tool` / `bank.` three times. **Measured with a
denominator rather than reported as a sighting:** over every blank line in `book/src`, **4 of 2,181
paragraph breaks** fall inside a sentence, all four in `architecture.md`, all four from `df7bc6e`.

**Why `.1`'s two proofs could not have caught it, and why the census is worse than blind:**

| Instrument | Verdict | Why |
| --- | --- | --- |
| `prove_words_unchanged.py` | **passes** | a newline and a blank line both normalize to one space |
| `book_list_signature.py` | **passes** | it watches `<li>`; these are `<p>` |
| `mdbook build` | **exit 0** | two paragraphs are legal markdown |
| `book_prose_census.py` | **improves** | the block got *smaller* — the number `.1` was moving |

The last row is the finding. **A splitter's own success metric goes up when it cuts a sentence in
half.** An instrument that rewards the defect it should catch is worse than one merely blind to it,
and this is the first measured instance in this repo. Recorded as a Knowledge Map card,
[[paragraph-split-can-cut-a-sentence-in-half]], with the source-level predicate that does see it.

**Repaired by moving the break, not deleting it** — deleting it restores the wall of text. Moving it
to the true boundary (`…downstream tool bank.` ⟶ blank ⟶ `It also proves…`) keeps the word sequence
intact, so the word proof still passes, *and* keeps the split.

### The clause-**set** proof the acceptance asked for is blind to reordering, so a stronger one was built

`.3b`'s acceptance said *"extract the clause set before and after and compare"*. Implemented
literally that is a multiset comparison, and a multiset is blind to **order** — a capability register
whose clauses are shuffled makes a different claim about what a gate proved while comparing equal.

Replaced with a strictly stronger proof in `scripts/prove_clauses_unchanged.py`: remove exactly what
a list conversion is permitted to change — list markers, clause separators (`,` `;` `:` and a
word-final `.`), and standalone connectives — then require the remaining word **sequence** to be
byte-identical. Both files came back **identical to the character** (43,542 and 101,051), which is a
much sharper result than "the sets matched".

Two design defects were caught while building it, and both are recorded because both produced a
*passing* proof:

- **Adjacent list items re-merged into one clause.** Stripping the `- ` marker and collapsing
  whitespace turns `- A` / `- B` into `A B`, so a conversion that dropped a separating comma
  would have compared equal. Fixed by treating the marker itself as a clause boundary.
- **Splitting on every `.` shredded `manifest.json` and `1.28`.** Symmetric, so not unsound, but it
  buried the real signal in noise; a `.` now separates only when it ends a word, and inline code
  spans are lifted out before any splitting so `` `0=4:4,1=2:2` `` cannot manufacture a boundary.

### The stated blind spot is measured, and its carrier had to be verified by hand

Dropping standalone `and`/`plus`/`or` from both sides means an edit consisting *only* of adding one
is invisible. Probed rather than assumed: inserting `and` into a list item leaves the clause proof
**silent** (exit `0`), as declared.

Per [[matched-mutation-is-not-the-intended-mutation]] a `silent` verdict must be checked against the
artifact. It could not be checked with the usual partner instrument: `prove_words_unchanged.py` is
**saturated** — it is already firing on the list conversion itself — so it cannot discriminate the
connective. The carrier was verified **directly**, by reading the mutated file (`- and budgeted
multi-helper allocation`, line 1141) with the substitution count asserted at 1. **Recorded as a
limit on `.1`'s complementarity claim: a proof pair only complements while one of the pair is
expected to be silent.**

### The elided-head enumeration is the one shape a bullet split cannot make read well

`hierarchy.md`'s fourth roll-call enumerates *"the direct sibling helper, direct registered sibling
helper, …, stateful parent-output helper **routes**"* — the head noun appears once, at the end, and
governs everything before it. Split into bullets, the early items are noun fragments. Supplying the
elided head would read better and **would invent words**, which is the one thing the proof forbids.
Wording preserved, and the shape stated here rather than discovered by a reader.

### The leaf's own verdict list was two blocks short

`.3b`'s acceptance named *"the two `<td>` blocks and the two guard-declined splittables"* — with the
roll-call set that is **7 distinct blocks** of the **9** that existed. `hierarchy.md:456` (a
1,618-char `<li>`) and `ir.md:579` (a 1,552-char `<p>`) appear in no list anywhere in this tree. They
were found by enumerating the census instead of the plan. All nine have a verdict below.

### Verdicts — all nine over-threshold blocks as they stood at `.3a`

| Block | class | verdict |
| --- | --- | --- |
| `architecture.md` 4,419 · 1,934 · `hierarchy.md` 3,763 · 2,076 | capability roll-calls | **repaired** — converted to markdown lists, clause sequence proven identical |
| `ir.md` 1,552 | splittable | **repaired** — one blank line at its only top-level sentence boundary that leaves both halves under threshold (1,254 / 364). A `.1` residue: its other boundary is 63 characters in, and `.1`'s guard rightly declined that one. Inside a list item, so the continuation indent was preserved and the `<li>` count proves it did not escape |
| `agent-mcp.md` 3,071 (`<td>`) | table cell | **owned by new leaf `.3c`** — not an acceptance. It is fourteen sentences of prose in a pipe-table cell, now the book's worst block, and a GFM cell cannot hold a block-level list, so no in-place repair exists |
| `api-introspection.md` 1,620 | guard-declined splittable | **owned by `.3c`** — the *same* fourteen-query enumeration; repairing the two separately would land the same content twice |
| `knobs.md` 1,901 (`<td>`) | table cell | **recorded acceptance** — a knob → metrics lookup row. Every peer cell in that column has the same shape; it is a scan target, not prose, and repairing it means restructuring a reference table that works |
| `hierarchy.md` 1,618 (`<li>`) | list item | **recorded acceptance** — same shape: a parenthesised metric-name list supporting a one-line concept in an outline. A nested list of 22 identifiers would bury the outline it exists to support |

### The fourteen-query roll-call is carried three times, and the book already names its owner

Measured before registering `.3c`, per name rather than per class: **all fourteen** `analyze` query
names appear in `TOOLBOX.md`, `book/src/agent-mcp.md`, `book/src/api-introspection.md` **and**
`docs/AGENT_INTROSPECTION_SCHEMA.md` (and 25–166× each in `src/`). `api-introspection.md` already
calls the schema file *"the canonical §7"* — so the book names its own authority and then duplicates
it twice. That is [`0033`](../decisions/0033-shadow-enumeration-classification.md)'s shape, but
**not** `0048`'s route: the two book copies carry editorial glosses (*"what does this output depend
on?"*) the canonical file does not, so a link-and-delete would lose book-only prose — the identical
finding `.3a` measured for the capability roll-calls, arriving a second time.

## Findings (`.3c`, measured `2026-08-02`)

### The book-only precondition was inherited from a per-class claim, and the first attempt to re-measure it was **wrong**

`.3b` established for the *capability roll-calls* that a `0048` link-and-delete would lose book-only
prose, and registered `.3c` expecting the same. That is a claim about a different block, so it was
re-measured per gloss. **The first measurement was line-wise `grep`, and it was wrong**: the book is
hard-wrapped, so `show me the deepest thing this node drives, gate by gate` reported *"appears only
in the `<td>`"* while it also sits at `agent-mcp.md:937` — **split across two source lines**. Redone
whole-file and whitespace-normalized:

| | count |
| --- | --- |
| glosses that exist **nowhere else in the repository** | **10 of 14** |
| glosses that recur once, in `agent-mcp.md`'s own per-query sections | 4 of 14 |

So `0048`'s route is declined **on measurement**, not on inheritance, and the cost of taking it is
stated as a number: 10 pieces of prose destroyed. This is the third time in one session that a
line-wise pattern lost to the hard wrap — the same failure `BOOK-LINK-TARGETS` records for wrapped
link text, and it cost a negative-control carrier here too (control 3 refused to apply until it was
retargeted at an unwrapped span).

### No instrument in the kit can express a LIFT, and the first one built for it **cried wolf**

A lift moves a run of clauses to a different part of the file. `prove_clauses_unchanged.py` is
**order-sensitive by construction** — that is the property `.3b` built it for — so it fires on any
lift and reports nothing useful. The obvious fix, a move-tolerant `--allow-move` mode comparing
clause **multisets**, was built and **failed on the real edit**:

```
REMOVED: `memory_provenance` (`1.18 → 1.19`)          <- 10 clauses reported destroyed
ADDED:   `module_reachability` (…) `flop_dependencies` (…) `memory_provenance` (…) …   <- 1 created
```

**Root-caused rather than reclassified: a list conversion deletes the separators that *define* a
clause boundary.** `A, B, C` is three clauses; `- A` / `- B` / `- C` is one, because the commas are
gone — which is the entire point of the conversion. So the clause multiset is **not invariant under
the very edit the mode exists to permit**, and the mode does not merely miss things, it manufactures
findings. The stable unit is the **word**: `normalize()` already strips separators before splitting,
so the word multiset survives both the separator removal and the move. Rebuilt on words, and the
result is sharp — `+15 words, 0 removed`, with every added word printed.

This is the same family as `.3b`'s *"a clause **set** is blind to reordering"*, arriving from the
other side: there the unit was too coarse to see a real change, here it is destroyed by a permitted
one. Recorded as [[clause-boundary-is-destroyed-by-the-list-conversion]].

### The lift needed two proofs, because one instrument answers only half the question

| Half | Proof | Result |
| --- | --- | --- |
| **content** — did the moved run change? | the 14 items extracted from the before-`<td>` and the after-list, compared in order | **14/14 byte-identical** |
| **remainder** — did anything else change? | the file with that run excised on both sides | **54,791 = 54,791** normalized chars, modulo 2 stated additions (136 chars) |

Neither alone is sufficient: the first is blind to everything outside the run, the second to
everything inside it. `--allow-move` is the cheap re-runnable summary of both, and is **opt-in and
never the default** precisely because it is order-blind — the property the default mode exists to
have. Proven complementary on **one unsaturated carrier**: reordering two `api-introspection.md`
bullets is **SILENT** under `--allow-move` and **FIRES** under the default mode.

**The first attempt at that control was saturated and had to be moved.** Run on `agent-mcp.md`, it
fired — but that file already diverges by the 15 added words, so it fires on *anything*. A saturated
instrument cannot discriminate, exactly as `.3b` recorded for `prove_words_unchanged.py`; the carrier
was moved to the file that still passes cleanly.

### The third copy should **not** link — measured, not assumed

`.3c`'s acceptance left open whether `TOOLBOX.md`'s row should link rather than duplicate. Measured:
**0 of its 15 parenthesised glosses are shared verbatim with the book's**, because the two are
written in different registers — `TOOLBOX.md` gives a terse capability descriptor (*"an output's
transitive support cone"*), the book gives the question the query answers (*"what does this output
depend on?"*). It is therefore **not** a lossy copy of an authoritative set, so `0033`'s first test
fails and R1 does not apply: replacing it with a link would substitute a pointer to prose that says
something *different*. Verdict: **not a shadow of the book**; its 1,594-character cell is a live-doc
readability question, which this tree does not own — the same boundary `.3a` drew for
`CODEBASE_ANALYSIS.md`. Recorded under *Surfaced by `.3c`*.

**`.3b`'s "carried three times" is right for the prose and needs one qualification**, measured here:
the fourteen *names* appear in **six** files, but only **three** carry descriptive prose
(`agent-mcp.md`'s `<td>`, `api-introspection.md`'s bullet, `TOOLBOX.md`'s row). The other three carry
them in framings that are **not duplicate prose and must not be swept in** — a JSON-schema `enum` row
(`api-tools.md`), a `query → producer function` mapping (`docs/AGENT_INTROSPECTION_SCHEMA.md`), and a
resource-URI line (`agent-mcp.md:255`).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-PARAGRAPH-BLOBS.2` | `pending` | **Next, and now against a fully repaired baseline.** Every oversized block that any repair can reach is gone: the **2** that remain are both `.3b` **recorded acceptances**. `.3a`→`.3b`→`.3c` cut the mass a threshold would be fitted to from 43,092 → 21,954 → 8,210 → **3,519**. `.2` must still derive a threshold rather than fit one, and must not treat the census as a quality metric — [[paragraph-split-can-cut-a-sentence-in-half]] is the companion predicate. |
| — | `BOOK-PARAGRAPH-BLOBS.3c` | `done` | Lifted the fourteen-query roll-call out of its table cell `2026-08-02`; oversized blocks 4 → 2, mass −57.1 %, and **no unaccepted oversized block remains in the book**. |
| — | `BOOK-PARAGRAPH-BLOBS.3b` | `done` | Converted the four capability roll-calls to lists `2026-08-02`; oversized blocks 9 → 4, mass −62.6 %. Repaired four **mid-sentence** breaks `.1` had shipped, and gave all nine blocks a verdict. |
| — | `BOOK-PARAGRAPH-BLOBS.3a` | `done` | Applied `0048` to the three verified duplications `2026-08-02`; worst block −49.2 %, mass −49.1 %. |
| — | `BOOK-PARAGRAPH-BLOBS.4` | `done` | Instruments promoted `2026-08-02`; the census reproduces `.1`'s numbers exactly and is now durable. |
| — | `BOOK-PARAGRAPH-BLOBS.1` | `done` | Repaired `2026-08-02`; worst block −62 %. |

## Decisions

- `2026-08-02` (registration): **Registered rather than fixed in passing**, though the owner called it
  *not urgent*. The standing directive is that a defect is handled only when a tree owns it
  ([`0041`](../decisions/0041-owner-standing-directives-recorded-in-layer-c.md)); an unregistered
  "I'll tidy that later" does not survive a session boundary, which is the failure the whole
  task-tree system exists to prevent.
- `2026-08-02` (registration): **Measured before registering, with a denominator.** *"Some mdBooks do
  this"* is an impression; **6 of 1,244** is a scope. Without the denominator the tree would have
  invited a sweep of all 30 chapters, and 1,238 paragraphs are already fine.
- `2026-08-02` (registration): **Whitespace-only repair, fixed as a constraint up front.** The
  temptation on opening a 246-line paragraph is to rewrite it. That would bury a readability fix
  inside a content change the owner did not ask for and cannot review at a glance.
- `2026-08-02` (registration): **Ownership search run, not assumed.** `BOOK-LINK-INTEGRITY` (closed)
  held link targets; `BOOK-EXAMPLES-RUNNABLE` holds runnable examples; `BOOK-LANE-COVERAGE` holds
  lane coverage; `BOOK-TEST-COUNT-SHADOWS` and `LIVE-DOC-BOOK-ALIGNMENT` hold counts and code
  alignment; `TABLE-RENDER-FIDELITY` is the doctrine for table well-formedness. **No tree or doctrine
  owns paragraph structure or rendered readability**, so this is a first mechanism, not a second
  (`feedback_full_factorization`).

## Decisions (continued — `.3`)

- `2026-08-02` (**owner decision**, in answer to `.1`'s surfaced question): **link, do not duplicate
  and do not delete.** *"There is no need to have the content of an already existing `.md` file in
  the book if you can just provide a link to that `.md` file from the book."* This overrides the
  agent's stated instinct (deletion) and is the better call: deletion removes the content from the
  owner's review surface, whereas a link keeps it one click away.

- `2026-08-02` (**mechanism, corrected against the owner's framing**): **the link must be an
  ABSOLUTE GitHub URL.** The owner observed that such links work when the book is on GitHub. The
  precise mechanism, and it matters: mdBook **rewrites a relative `.md` target to `.html`**, so
  `[ROADMAP.md](../../ROADMAP.md)` renders as `../../ROADMAP.html` — alive on GitHub, **dead in the
  rendered book**, with `mdbook build` exiting `0`
  ([`0046`](../decisions/0046-book-never-links-outside-book-src.md),
  [[mdbook-md-to-html-rewrite-trap]]), and `BOOK-LINK-TARGETS` blocks it at pre-commit. An
  **absolute** `https://` target is passed through untouched. Verified three ways rather than
  recalled: the gate skips absolute schemes (`check_book_link_targets.sh:149`); the rendered HTML
  keeps `href="https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md"` with its
  `.md` intact; and that URL returns **HTTP 200** on the public repo.

- `2026-08-02` (**not a new convention — an existing one**): the book already carries **6** absolute
  links, **5** of them GitHub blob links to repo `.md` files (`docs/AGENT_INTROSPECTION_SCHEMA.md`
  ×4, decision `0019`). `.3` extends the established pattern rather than inventing one.

- `2026-08-02` (**precondition verified BEFORE removing anything**, because linking to content that
  is not actually elsewhere would lose it): of the **111** distinct `saw_*` coverage flags the book
  names, **0 are book-only** — every one is in `ROADMAP.md` or `src/`. Every sampled bank id
  (`r51`, `r73`, `r78`, `r83`) appears **2–3×** in `ROADMAP.md` against **1×** in the book, and
  `ROADMAP.md`'s prose is *strictly richer* (it records `r83`'s **198-scenario** count, which the
  book omits). The book's register is therefore a **lossy copy** of an authoritative set — decision
  [`0033`](../decisions/0033-shadow-enumeration-classification.md)'s shadow, whose repair is R1,
  removal of the copy. Replacing it with a link is that repair, done without losing the content.

- `2026-08-02` (**stated limit, not discovered later**): `BOOK-LINK-TARGETS` **skips** absolute URLs
  by design, so these links are **not verified by any gate** — a renamed or moved target 404s
  silently. Pinning to `main` keeps them current at the cost of rot on rename; pinning to a SHA
  would freeze content that is meant to stay live. `main` is chosen deliberately; whether the gap
  deserves a checker is `.2`'s question, and per `0047` the by-product route is preferred to a new
  gate.

### ⚠ CONFLICT found `2026-08-02` — read this before executing `.3`

- **Owner directive `2026-08-02`: do NOT use `{{#include ...}}`.** *"I prefer having a link pointing
  the `.md` files than using `{{#include ...}}`."* This **agrees with** decision
  [`0046`](../decisions/0046-book-never-links-outside-book-src.md) candidate **D**, which already
  rejected `{{#include}}` on four counts (a 163 KB chapter injection, anchor collisions worsened in
  `print.html`, a build dependency outside `src`, …). Settled, no conflict.

- **But the LINK form the owner chose is the one `0046` explicitly REJECTED.** `0046` candidate
  **B — absolute GitHub URL — is marked ❌**, for two stated reasons: (i) `book/book.toml` sets
  `git-repository-url = ""` and `edit-url-template = ""`, so the book is **deliberately built
  host-agnostic**, and hard-coding a host contradicts a setting the project already made
  (**verified**: both lines are live at `book/book.toml:14-15`); and (ii) it breaks on a fork, an
  org rename, a mirror, **or an offline reader**, and must pin either `main` (a moving target) or a
  SHA (stale on landing). `0046`'s accepted form **A** is to *name* the file in backticks and not
  link at all — measured unanimous at **31 of 32** sites.

- **An error in this tree's own record, corrected here rather than left standing.** The entry
  committed at `a5645c1` said the absolute-link form was *"not a new convention — an existing one"*
  and that `.3` *"extends the established pattern"*. **That is wrong.** The 6 absolute links were
  introduced by `AGENT-INTROSPECTION-MCP.7`, `ACCEPTANCE-DIVERGENCE-HUNTING.2f` and
  `BOOK-API-REFERENCE.1`, all of which **predate `0046`** (`9ad7385`, `2026-08-01`). They are
  **residue that predates the decision rejecting the form**, not an endorsed convention. The claim
  was made from a count without checking the dates, and it materially understated the conflict.

- **The owner's directive governs — but the supersession must be EXPLICIT, not silent.**
  `docs/decisions/INDEX.md` is unambiguous: *"To change a fact, add a new record or mark the old one
  superseded; do not silently rewrite the old decision."* So `.3` **may not** simply start adding
  absolute links: it must first land a decision record that supersedes `0046`'s candidate-B
  rejection, states the owner directive and date, and answers `0046`'s two live objections — above
  all the **offline reader**, which is the very case the owner raised. Whether `book.toml`'s
  host-agnostic setting should also change is part of that record.

- **RESOLVED `2026-08-02` by decision [`0048`](../decisions/0048-book-links-to-the-file-it-stopped-duplicating.md)**,
  which supersedes `0046` candidate **B** *narrowly* — absolute links are admitted **only** where the
  link is the sole route to content the book deliberately removed; form **A** still governs every
  incidental reference and the 31/32 convention is not reopened.
- **The owner's preferred "works on both surfaces" option was tested and does not exist.** `book/src`
  and `book/book-out` sit at the same depth, so a relative `../../ROADMAP.md` *would* resolve from
  either — but a probe chapter proved mdBook rewrites the target to `.html` in **both** the markdown
  form **and the raw-HTML `<a href>` form**. No relative form survives into the rendered book, so the
  absolute URL is the only one live on GitHub *and* locally. Probe removed, book rebuilt clean.

## Open Questions

- **Is 1,500 characters the right threshold, or an artifact of this census?** It separates the 6
  outliers cleanly from a body whose per-chapter means sit at 250–430, but it was chosen by looking
  at the distribution. `.2` must derive a threshold or decline to set one — a fitted number in a gate
  is the failure decisions `0036` §(c) and `0040` §(c) both warn about.
- **Does `print.html` need its own verdict?** It concatenates every chapter, so it inherits each
  blob; it is excluded from the census as generated chrome, exactly as `BOOK-LINK-INTEGRITY.1`
  excluded it for double-counting. Repairing the sources repairs it. Worth stating, not measuring.
- **Are there blobs the `<p>` census cannot see?** **Answered at `.1`: yes, 5 of 11.** Measured, and
  the instrument was rebuilt — see Findings. **Verdicts given at `.3b`:** `knobs` 1,901 is a
  **recorded acceptance** (a knob → metrics lookup row; a scan target, and a GFM pipe-table cell
  cannot hold a block-level list), while `agent-mcp` 3,071 moved to **`.3c`** rather than being
  accepted — it is fourteen sentences of *prose* in a table cell, not a name list, and it is now the
  book's worst block.
- **New — two blocks the guard declined to split.** `architecture.md:692-748` and
  `api-introspection.md:224-244` each hold exactly one sentence boundary, positioned so near an end
  that a cut would leave a sub-300-character orphan. The guard refused, deliberately: a shredded
  paragraph is worse reading than a long one. **Answered at `.3b`:** the first was a capability
  roll-call and is now a list; the second is the second copy of the fourteen-query enumeration and
  moved to `.3c`. **Neither needed the split the guard declined** — in both cases the sentence
  boundary was never the repair, which is the guard being right for the wrong reason.
- **New at `.3b` — is a smaller block always an improvement?** **No, and the census cannot tell.**
  Cutting a sentence in half lowers worst-block size and oversized mass, so `.1`'s four mid-sentence
  breaks *improved* every number this tree reports while making the prose worse. `.2` must not treat
  the census as a quality metric without a companion predicate; the source-level one is recorded in
  [[paragraph-split-can-cut-a-sentence-in-half]].

## Surfaced by `.3c`, owned by nobody yet

- **`TOOLBOX.md`'s `analyze` row is a 1,594-character table cell.** Measured **not** to be a copy of
  the book's (0 of 15 glosses shared verbatim — see Findings), so it is original prose with the same
  wall-of-text shape the owner objected to, in a live doc rather than the book. This tree owns the
  **book**; the same boundary `.3a` drew for `CODEBASE_ANALYSIS.md`. **The census cannot even see
  it** — `scripts/book_prose_census.py` reads rendered `book/book-out/*.html`, so every live doc
  outside the book is unmeasured by construction. Whether the live docs deserve their own census is
  a question for a live-doc hygiene tree, not for `.2`, whose subject is the book.

## Surfaced by `.3b`, owned by nobody yet

- **`MEMORY.md` is saturated against its own hard cap — it was `6,138` of `6,144` bytes at `cd83e3c`,
  six bytes of headroom.** `MEMORY-ARCH` **FAILED** on this leaf's first `MEMORY.md` update and had to
  be satisfied by cutting prior content to pay for the new pointer. That is the cap working as
  designed (*"move content down, do not trim prose"*), but at six bytes of slack the resume pointer
  has become **zero-sum**: every future slice must delete a previous session's pointer to record its
  own, and the cheapest thing to delete is always the oldest — which is the opposite of what a
  recovery surface should preserve. The failure mode is silent, because the gate cannot tell routing
  from deletion.
- **A hand-kept count inside `MEMORY.md` has no derivation.** The `blockers` line asserts *"SEVEN
  instrument notes"* while citing three cards; nothing derives seven (`grep -l 'gotcha'
  docs/knowledge/*.md` gives **24**, now 25). This leaf **declined to increment it** rather than
  guess, and cited its new card from `next_action` instead — but the number is now one behind reality
  by construction. It is the same shadow shape
  [`0033`](../decisions/0033-shadow-enumeration-classification.md) classifies, sitting inside the
  file that warns against exactly this two lines above (*"a hand-kept list of card names is the exact
  shadow `0033` repairs by deletion"*).

Both belong to the resume-pointer contract, not to book fidelity. Not opened here because the repo
may not pivot dirty; `RESUME-POINTER-CONTRACT` is the nearest existing owner.

## Surfaced by `.3a`, owned by nobody yet

- **`CODEBASE_ANALYSIS.md` carries a third copy of the per-bank register, and it is the most current
  one.** The same `rN` roll-call the book just stopped duplicating sits there as a single wall of
  text running to **`r87`** — further than either book copy reached. This tree owns the **book**, so
  it is out of scope here, but the shadow it forms is the same one
  [`0033`](../decisions/0033-shadow-enumeration-classification.md) classifies, and the two book
  copies proved the failure mode is real by going stale at `r85`. Needs its own tree; not opened
  here because the repo may not pivot dirty.

## Surfaced by `.1`, owned by nobody yet

- **`mdbook test book` FAILS on a clean tree, and CI cannot see it.** Root-caused, not classified:
  local **mdBook v0.5.2** treats an **unlabelled ``` fence** as a Rust doctest and compiles it, so
  `book/src/agent-mcp.md` lines 174 and 245 fail with `E0425`. **CI pins v0.4.40**
  (`.github/workflows/ci.yml`), which does not, so the CI step is green while a current local run is
  red. Established as **pre-existing**, not caused by this leaf, by stashing the `book/src/` edits
  and re-running on `HEAD` — **identical two failures, identical exit 101**. The book holds ~100
  unlabelled fences, so bumping the pinned version turns this red in CI. **This is a third instance
  of the by-product rule** ([[practice-survives-as-a-by-product-not-by-a-gate]]): nothing forces
  anyone to run `mdbook test` locally, and the pin means CI's green is not evidence about any other
  version. Needs its own tree; not opened here because this one was mid-leaf and the repo may not
  pivot dirty.

## Blockers

- None. The owner explicitly called this **not urgent**.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-02` | `.3c` | `Oversized blocks 4 -> 2; worst rendered block 3,071 -> 1,901 chars (-38.2 %); oversized mass 8,210 -> 3,519 (-57.1 %). BOTH survivors are .3b's recorded acceptances, so no UNACCEPTED oversized block remains. PRECONDITION RE-MEASURED PER GLOSS, and the first attempt was WRONG: a line-wise grep over a hard-wrapped book reported a gloss as unique that sits at agent-mcp.md:937 split across two lines. Redone whole-file and whitespace-normalized: 10 of 14 glosses exist NOWHERE else in the repository, so 0048's link-and-delete would have destroyed 10 pieces of book-only prose - the route declined on measurement, not inheritance. THE LIFT IS PROVEN IN TWO HALVES because no single instrument expresses it: content (the 14 items extracted from the before-<td> and the after-list, byte-identical in sequence, 14/14) and remainder (the file with that run excised on both sides, 54,791 = 54,791 normalized chars, modulo two stated additions of 136 chars). api-introspection.md needs no split and passes the PLAIN clause proof outright: 11,593 normalized chars, byte-identical sequence. NEW --allow-move MODE, and its first design CRIED WOLF: a clause multiset reported 10 clauses destroyed and 1 created on a correct edit, because a list conversion deletes the separators that DEFINE a clause boundary. Rebuilt on the word multiset, which is invariant under both the separator removal and the move: agent-mcp.md +15 words / ZERO REMOVED with every added word printed, api-introspection.md multiset IDENTICAL (1,648 words). FOUR controls, every substitution count asserted: drop a query -> FIRES; reword -> FIRES; invent a word -> FIRES (after its first carrier REFUSED to apply, defeated by the same hard wrap); REORDER two adjacent bullets -> SILENT under --allow-move and FIRES under the default, the complementarity proven on ONE UNSATURATED carrier after the first attempt was saturated by this leaf's own additions. STRUCTURE RECONCILED TERM BY TERM: prose blocks 3,671 -> 3,698 (+27) = 14 lifted <li> + 11 schema <li> + 1 lead-in <p> + 1 section-split <p>; book_list_signature.py 1,493 -> 1,518 (+25 = 14 + 11) moving in exactly the 2 edited chapters of 31. Zero mid-sentence breaks book-wide (0 of 2,190). Anchor #derived-relation-queries-analyze present in the rendered HTML. mdbook build exit 0; scripts/check_doctrines.sh 11/11; cargo test --test book_examples 4 passed / 0 failed.` | `lifted, not deleted; the third copy measured NOT to be a shadow and left alone with a recorded verdict` (book + `scripts/` + docs only; DUT byte-identical) |
| `2026-08-02` | `.3b` | `Oversized blocks 9 -> 4; worst rendered block 4,419 -> 3,071 chars (-30.5 %); oversized mass 21,954 -> 8,210 (-62.6 %). All four capability roll-calls left the over-threshold set. CLAUSE PRESERVATION PROVEN by the new scripts/prove_clauses_unchanged.py: architecture.md 43,542 -> 43,542 and hierarchy.md 101,051 -> 101,051 normalized chars, byte-identical SEQUENCES (order-sensitive, so a reorder cannot pass - the clause SET the acceptance named would have been blind to it). FOUR controls via scripts/negative_control.sh, every substitution count asserted: drop a clause -> FIRES; reword -> FIRES; reorder two adjacent clauses -> FIRES; add a connective -> SILENT (the stated blind spot), its carrier verified by READING the mutated file because prove_words_unchanged.py is saturated by the conversion and cannot discriminate. STRUCTURE RECONCILED TERM BY TERM: prose blocks 3,501 -> 3,671 (+170) = 168 new <li> + 2 new <p>; book_list_signature.py moved in exactly the 2 edited chapters of 31 and no others; the ir.md split left <li> UNCHANGED, proving its continuation stayed inside its item. NEW FINDING, measured with a denominator: .1 shipped FOUR paragraph breaks INSIDE a sentence (4 of 2,181 breaks book-wide, all in architecture.md, all from df7bc6e) - invisible to both of .1's proofs and REWARDED by the census, which reports the smaller block as progress. Repaired by moving each break to the true sentence boundary; ir.md whitespace-only, prove_words_unchanged.py OK (30,768 -> 30,768). mdbook build exit 0; scripts/check_doctrines.sh 11/11; knowledge-map check OK.` | `repaired; all 9 blocks carry a verdict (2 repaired classes, 2 recorded acceptances, 2 moved to the new .3c) and the acceptance's own list was 2 blocks short` (book + scripts + docs only; DUT byte-identical) |
| `2026-08-02` | `.3a` | `Oversized blocks 12 -> 9; worst rendered block 8,704 -> 4,419 chars (-49.2 %); oversized mass 43,092 -> 21,954 (-49.1 %), measured with the promoted scripts/book_prose_census.py. PRECONDITION MEASURED EXHAUSTIVELY BEFORE DELETION, not sampled: 73/77 bank ids glossed in ROADMAP.md (r7 still mentioned there; r8's gloss deliberately KEPT in both chapters as a lesson; r11/r12 survive in CHANGES.md x25/x18 and CODEBASE_ANALYSIS.md), 77/77 saw_* flags still defined in src/ and 62/77 still named elsewhere in the book - nothing left the repository. STRUCTURE: n_li 1,325 across 31 chapters UNCHANGED, only hierarchy.html's content SHA moved, the exact signature of text removed from inside an <li> without escaping it. LINKS: 3 absolute targets render with .md intact, 0 relative .md escapes, target returns HTTP 200, BOOK-LINK-TARGETS ok (186 local of 231). mdbook build exit 0; scripts/check_doctrines.sh 11/11.` | `repaired by link per 0048; the 53-84 %-book-only capability roll-calls moved to .3b rather than being deleted under a decision that does not cover them` (book-only; DUT byte-identical) |
| `2026-08-02` | `.4` | `Promoted census reproduces the validated instrument EXACTLY: 3,501 prose blocks / 30 chapters / 12 over 1,500 / mass 43,092, all 12 sizes identical — after a first draft that matched both headline numbers while inflating 9 of 12 sizes. book_list_signature.py reproduces .1's 1,325 <li> across 31 chapters. All 12 source anchors resolve. Three controls via scripts/negative_control.sh (every substitution count asserted): paragraph merge -> census FIRES (3,501->3,500 blocks, 12->13, mass 43,092->44,919); source re-wrap -> census SILENT on measurement fields; break inside <li> without continuation indent -> list signature FIRES, word proof OK (90,542 -> 90,542). One control FAILED first and was root-caused to the carrier, not the instrument (it deleted the blank line, so mdBook lazily continued the paragraph inside the same <li>). Restores cmp-verified; mdbook build exit 0; scripts/check_doctrines.sh 11/11.` | `promoted; instruments durable in scripts/ and documented in TOOLBOX.md §7` (docs+scripts only; DUT byte-identical) |
| `2026-08-02` | `.1` | `24 paragraph breaks across 5 chapters. WORST PROSE BLOCK 22,908 -> 8,704 chars (-62.0 %); architecture.html 22,908 -> 7,043 (-69.3 %); oversized mass 54,897 -> 43,092 (-21.5 %). Two independent proofs, both negative-controlled: whitespace-normalized word identity (passes on the edit, FAILS on a one-character word change) and a rendered <li> census of 1,325 items across 31 chapters (identical after; FIRES on a landed sabotage that strips a list continuation's indent, which the word proof passes). The .0 census was CORRECTED: denominator 1,244 -> 3,467, over-threshold 6 -> 11, after rebuilding the instrument on html.parser with <pre> excluded. mdbook build exit 0; cargo test --test book_examples 4 passed / 0 failed; scripts/check_doctrines.sh 11/11. mdbook test fails identically on HEAD with the edits stashed - pre-existing, root-caused, surfaced.` | `repaired (whitespace-only); the run-on-enumeration residue is registered as .3 rather than claimed as fixed` |
| `2026-08-02` | `.0` | `registered from an owner finding, measured first at 9060993: mdbook build book, then over book/book-out/*.html (main only, print.html + 404.html excluded) — 1,244 rendered paragraphs across 30 chapters, 6 over 1,500 chars, worst 22,908 in architecture.html. Source confirmed: book/src/architecture.md lines 658-903 are 246 consecutive non-blank lines. Ownership search run against the six book/live-doc trees and TABLE-RENDER-FIDELITY; none owns paragraph structure.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `ebd7869` — `BOOK-PARAGRAPH-BLOBS.0 — register the owner finding on wall-of-text paragraphs` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |
| `.1` | `df7bc6e` — `BOOK-PARAGRAPH-BLOBS.1 — split the wall-of-text paragraphs; worst block down 62 %` | Book-only ⇒ DUT byte-identical. Whitespace-only, proven twice. |
| `.3a` | `a385b76` — `BOOK-PARAGRAPH-BLOBS.3a — link the registers the book stopped duplicating` | Book-only ⇒ DUT byte-identical. Three `0048` links; worst block −49.2 %, mass −49.1 %. |
| `.4` | `d25bbe7` — `BOOK-PARAGRAPH-BLOBS.4 — promote the book-census instruments into scripts/` | Docs + `scripts/` only ⇒ DUT byte-identical. Reproduces the validated census exactly; three controls, one of which failed with the instrument innocent. |
| `.3b` | `afb9847` — `BOOK-PARAGRAPH-BLOBS.3b — convert the capability roll-calls to lists; oversized mass down 63 %` | Book + `scripts/` + docs only ⇒ DUT byte-identical. Clause sequence proven identical; four controls. Also repaired four **mid-sentence** breaks `.1` shipped. |
| `.3c` | `pending` — `BOOK-PARAGRAPH-BLOBS.3c — lift the fourteen-query roll-call out of its table cell` | Book + `scripts/` + docs only ⇒ DUT byte-identical. Lift proven in two halves; `--allow-move` added after its clause-unit first draft cried wolf. |

## Changelog

- `2026-08-02`: `.3c` **lifted** the fourteen-query `analyze` roll-call out of the GFM table cell it
  could not fit in, and converted its second copy — the schema-bump enumeration — into eleven
  bullets. **Oversized blocks 4 → 2, worst rendered block 3,071 → 1,901 characters (−38.2 %),
  oversized mass 8,210 → 3,519 (−57.1 %)**, and the two survivors are both `.3b`'s **recorded
  acceptances**, so the book now holds **no unaccepted oversized block**. Four things are recorded
  rather than smoothed over. **(1) The book-only precondition was inherited, and re-measuring it
  went wrong first.** `.3b` established it for a *different* block; measured per gloss, **10 of 14**
  exist nowhere else in the repository, so `0048`'s link-and-delete would have destroyed ten pieces
  of prose. The first measurement said otherwise because it was **line-wise over a hard-wrapped
  book** — the third time in one session a line-wise pattern lost to the wrap, and it cost a
  negative-control carrier as well. **(2) The move-tolerant proof built for this leaf CRIED WOLF**,
  reporting ten clauses destroyed and one created on a correct edit. Root-caused, not reclassified:
  **a list conversion deletes the separators that define a clause boundary**, so the clause multiset
  is not invariant under the very edit the mode exists to permit. Rebuilt on the **word** multiset —
  `+15 words, 0 removed`, every added word printed. **(3) A lift needs two proofs**, because content
  and remainder are separate questions and each instrument is blind to the other's half: the
  fourteen items moved **byte-identically in sequence**, and the file with that run excised is
  identical on both sides (54,791 = 54,791) modulo two stated additions. **(4) The third copy was
  measured and left alone** — `TOOLBOX.md` shares **0 of 15** glosses with the book's, so it is
  original prose in a different register, not a shadow, and linking would point at prose that says
  something else.

- `2026-08-02`: `.3b` converted the **four capability roll-calls** — the run-on sentences `.3a`
  measured as **53–84 % book-only**, so `0048`'s link would have deleted prose that exists nowhere
  else — into markdown lists. **Oversized blocks 9 → 4, worst rendered block 4,419 → 3,071
  characters (−30.5 %), oversized mass 21,954 → 8,210 (−62.6 %)**, and all four roll-calls left the
  over-threshold set entirely. Four things are recorded rather than smoothed over. **(1) `.1`
  shipped four paragraph breaks *inside a sentence*** — `…repo-owned Yosys` ⟶ blank ⟶ `modes). That
  report banks…` and three of `…downstream tool` ⟶ blank ⟶ `bank.` — measured book-wide as **4 of
  2,181** breaks, and the reason nothing reported them is the sharp part: both of `.1`'s proofs are
  structurally blind to it and **the census actively rewards it**, because cutting a sentence in
  half makes the block *smaller*. Repaired by moving each break to the true boundary, and recorded as
  [[paragraph-split-can-cut-a-sentence-in-half]]. **(2) The acceptance's own proof was too weak** —
  a clause *set* is blind to reordering, and a reordered capability register is a changed claim
  about what a gate proved; replaced by an order-sensitive word-**sequence** identity modulo
  separators and connectives (`scripts/prove_clauses_unchanged.py`), which returned **byte-identical
  sequences** (43,542 and 101,051 normalized characters). Two defects in that instrument were caught
  while building it, both of which produced a *passing* proof: adjacent list items re-merging into
  one clause, and splitting inside `manifest.json`. **(3) The instrument's blind spot is measured,
  and its carrier had to be verified by hand** — a lone added `and` is silent by design, and the
  usual partner proof could not confirm the carrier because it is *saturated* by the conversion
  itself, which is a stated limit on `.1`'s complementarity claim. **(4) The leaf's own verdict list
  was two blocks short** — it named 7 of the 9 over-threshold blocks; all nine now carry a verdict
  (four roll-calls and `ir.md` repaired, `knobs.md`'s and `hierarchy.md`'s metric-name cells recorded
  as **accepted** because they are scan targets rather than prose, and the two copies of the
  fourteen-query `analyze` roll-call moved to a new **`.3c`**).
- `2026-08-02`: `.3a` applied [`0048`](../decisions/0048-book-links-to-the-file-it-stopped-duplicating.md)
  to the three blocks whose content is a **verified** lossy copy of a file that already owns it — the
  two per-bank `rN` registers and the `saw_*` coverage roll-call. **Oversized blocks 12 → 9, worst
  rendered block 8,704 → 4,419 characters (−49.2 %), oversized mass 43,092 → 21,954 (−49.1 %).** The
  leaf was **split at execution**, and the reason is the finding: `0048` authorises a link only where
  it is the *sole route* to content that lives elsewhere, and measured per block rather than per
  class, that is true of **three** of the nine survivors. The four **capability roll-calls** are only
  **53–84 %** clause-verbatim in any other live doc — deleting them under `0048` would have removed
  book-only prose, so they moved to `.3b` for a structural repair instead. Two things are recorded
  rather than smoothed over: **(1)** the removed copy was not merely redundant, it was **already
  wrong** — both registers still called `r85` "the current bank" a few lines below the sentence
  naming `r87`, so this is a correctness repair as well as a readability one; **(2)** the exact
  residue is stated, not discovered later — **73/77** bank ids are glossed in `ROADMAP.md` (`r7` is
  still mentioned there, **`r8`'s gloss was deliberately kept** in both chapters because it records a
  lesson, `r11`/`r12` survive in `CHANGES.md` and `CODEBASE_ANALYSIS.md`) and **77/77** coverage
  flags remain defined in `src/`. **Nothing left the repository.** One of the three sites is inside a
  list item, so the `<li>` census carries the safety claim: `n_li` **1,325 unchanged** across 31
  chapters with only `hierarchy.html`'s content SHA moving.
- `2026-08-02`: `.4` made this tree's instruments durable. The census, the list-structure proof and
  the word-identity proof lived in **gitignored `target/tmp/book-blob/`** — so the apparatus that
  judges `.1`, `.3` and `.2` would have vanished with the next `cargo clean`, and `.2` cannot derive
  a threshold from a measurement nobody can re-run. Promoted to `scripts/book_prose_census.py`,
  `scripts/book_list_signature.py` and `scripts/prove_words_unchanged.py`, documented in
  `TOOLBOX.md` **§7**, and **deliberately not registered as a doctrine check** — `.2` owns that
  question and `0047` prefers removing the need to watching harder. The census gained a **source
  anchor** and a **repairability class** per block (`SPLITTABLE` / `RUN-ON` / `LIST-ITEM` /
  `TABLE-CELL`), which is what `.3` needs to give each surviving block a verdict. Three things are
  recorded rather than smoothed over: **(1)** the first draft matched the denominator and the
  over-threshold count while inflating **9 of 12** block sizes — caught only because `.4`'s
  acceptance demanded every size; **(2)** a control **failed and the instrument was innocent** — the
  carrier deleted a blank line as well as an indent, so mdBook lazily continued the paragraph and
  nothing escaped its `<li>`, which refines `0047`: an asserted substitution count proves the
  experiment *ran*, not that it ran the *intended* experiment; **(3)** `.1`'s complementarity claim
  is **carrier-sensitive** — the word proof is blind only to a break inserted inside a list item
  without a continuation indent, and that exact carrier is now recorded with both verdicts.
- `2026-08-02`: `.1` repaired the splittable blobs. **Worst prose block 22,908 → 8,704 characters
  (−62.0 %)**, `architecture.html` **−69.3 %**, oversized mass **−21.5 %**; 24 paragraph breaks
  across 5 chapters with **zero words changed**. Three things went differently from the plan and all
  three are recorded rather than smoothed over. **(1) `.0`'s census was wrong** — its `<p>`-only
  regex spanned code blocks; rebuilt on `html.parser` with `<pre>` excluded, the denominator is
  **3,467** not 1,244 and **11** blocks exceed the threshold, not 6, including a **10,781-character
  `<li>`** that the first instrument could not see at all. **(2) `.0`'s acceptance criterion was
  unachievable** — *"a diff that adds only empty lines"* cannot split a sentence that begins
  mid-line, so it was replaced by a **stronger** proof (whitespace-normalized word identity),
  negative-controlled both ways. **(3) The first pass shipped a real regression** — dropping a list
  item's indent silently promoted its continuation out of the `<li>`, which the word proof is blind
  to by construction; fixed in the splitter and caught by a **second** proof over the rendered `<li>`
  census, whose control had to be run **twice** because the first sabotage refused to apply. The
  residue is a **distinct defect** — four **run-on enumerations**, single sentences of dozens of
  clauses that no blank line can split — registered as **`.3`** rather than claimed as repaired.
  Separately surfaced and root-caused: **`mdbook test book` fails on a clean tree** under local
  mdBook v0.5.2 (unlabelled fences compiled as Rust doctests) while CI pins v0.4.40 and stays green.
- `2026-08-02`: Created from an owner finding — the rendered book stitches paragraphs into blobs, and
  the owner reviews the book rather than the code. Measured before registering: **6 of 1,244**
  rendered paragraphs exceed 1,500 characters, the worst being a single **22,908-character** `<p>` in
  `architecture.html` that is **246 consecutive blank-line-free source lines** and roughly **71 %** of
  that chapter's prose. The cause is **accretion** — successive slices each appended one sentence to
  an existing paragraph, every diff three lines long — which makes this the first independent
  instance of the rule landed one commit earlier at `UNGATED-PRACTICE-AUDIT.1`: **paragraph length is
  a by-product of nothing**, so nothing reported it. Scoped to a **whitespace-only** repair with no
  wording change, and explicitly not a mandate to gate.
