# BOOK-PARAGRAPH-BLOBS: the rendered book has paragraphs that are walls of text — one is 22,908 characters

## Metadata

- Tree ID: `BOOK-PARAGRAPH-BLOBS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / book fidelity
- Created: `2026-08-02`
- Last updated: `2026-08-02` (registered from an owner finding; frontier `.1`)
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

- `.1` repairs the **6 paragraphs over 1,500 characters** by inserting blank lines at genuine topic
  boundaries, with **zero wording changes** — provable by a diff that adds only empty lines.
- The census is **re-run after the repair** and the new distribution reported, so the claim is
  measured rather than asserted.
- `mdbook build` and `mdbook test` stay green; `scripts/check_doctrines.sh` stays 11/11; book-only ⇒
  DUT byte-identical, `tests/snapshots.rs` untouched.
- `.2` decides whether anything should watch this, and states the by-product alternative it rejected
  before proposing a gate. **"No gate"** is a legitimate and fully acceptable outcome.

## Task Tree

- ID: `BOOK-PARAGRAPH-BLOBS`
  Status: `active`
  Goal: `The rendered book has no wall-of-text paragraphs, and the repair is whitespace-only.`
  Children: `.1` (repair the 6 blobs), `.2` (decide whether anything watches this)

- ID: `BOOK-PARAGRAPH-BLOBS.1`
  Status: `done`
  Goal: `Split the rendered prose blocks over 1,500 characters at genuine topic boundaries by inserting blank lines only.`
  Acceptance: `The diff adds blank lines and changes no words — ORIGINAL WORDING SUPERSEDED at execution, see Findings: "git diff --ignore-blank-lines shows no content hunks" is too narrow, because a sentence that begins mid-line cannot be split without moving the line break. Replaced by a stronger proof: whitespace-normalized word identity, plus a rendered list-structure proof. The post-repair census is re-run and reported against the pre-repair one. mdbook build green.`
  Verification: `Worst prose block 22,908 -> 8,704 chars (-62.0 %); architecture.html 22,908 -> 7,043 (-69.3 %); oversized mass 54,897 -> 43,092 (-21.5 %). Both proofs pass and both are negative-controlled. 24 paragraph breaks inserted across 5 chapters, zero words changed.`
  Commit: `pending`

- ID: `BOOK-PARAGRAPH-BLOBS.3`
  Status: `pending`
  Goal: `Decide what to do about the RUN-ON ENUMERATIONS — the residue .1 structurally cannot fix, because a single sentence has no paragraph boundary to insert a break at.`
  Acceptance: `Registered by .1 as a distinct defect, not a leftover. Every remaining oversized block is one sentence listing dozens of clauses; the natural repair is a markdown list, which changes structure and wording and therefore could not ride inside .1's whitespace-only constraint. States whether the repair is a list conversion, a move to a table, or a deletion of accreted evidence prose that ROADMAP/docs/evidence already own.`
  Verification: `pending`
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

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-PARAGRAPH-BLOBS.3` | `pending` | **Next.** The residue is one coherent defect class — four run-on enumerations that whitespace cannot touch — and it is now the whole of the remaining oversized mass. |
| 2 | `BOOK-PARAGRAPH-BLOBS.2` | `pending` | Decide what watches this, against a repaired baseline. Deliberately after `.3`: choosing a threshold while 43,092 characters of known-unfixed enumeration are still in the book would fit the number to the defect. |
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

## Open Questions

- **Is 1,500 characters the right threshold, or an artifact of this census?** It separates the 6
  outliers cleanly from a body whose per-chapter means sit at 250–430, but it was chosen by looking
  at the distribution. `.2` must derive a threshold or decline to set one — a fitted number in a gate
  is the failure decisions `0036` §(c) and `0040` §(c) both warn about.
- **Does `print.html` need its own verdict?** It concatenates every chapter, so it inherits each
  blob; it is excluded from the census as generated chrome, exactly as `BOOK-LINK-INTEGRITY.1`
  excluded it for double-counting. Repairing the sources repairs it. Worth stating, not measuring.
- **Are there blobs the `<p>` census cannot see?** **Answered at `.1`: yes, 5 of 11.** Measured, and
  the instrument was rebuilt — see Findings. Two `<td>` cases survive (`agent-mcp` 3,071,
  `knobs` 1,901) and are deliberately **out of scope**: a blank line cannot split a table cell, and
  `TABLE-RENDER-FIDELITY` already owns table well-formedness. `.3` gives them a verdict.
- **New — two blocks the guard declined to split.** `architecture.md:692-748` and
  `api-introspection.md:224-244` each hold exactly one sentence boundary, positioned so near an end
  that a cut would leave a sub-300-character orphan. The guard refused, deliberately: a shredded
  paragraph is worse reading than a long one. `.3` decides whether they are enumerations too.

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
| `2026-08-02` | `.1` | `24 paragraph breaks across 5 chapters. WORST PROSE BLOCK 22,908 -> 8,704 chars (-62.0 %); architecture.html 22,908 -> 7,043 (-69.3 %); oversized mass 54,897 -> 43,092 (-21.5 %). Two independent proofs, both negative-controlled: whitespace-normalized word identity (passes on the edit, FAILS on a one-character word change) and a rendered <li> census of 1,325 items across 31 chapters (identical after; FIRES on a landed sabotage that strips a list continuation's indent, which the word proof passes). The .0 census was CORRECTED: denominator 1,244 -> 3,467, over-threshold 6 -> 11, after rebuilding the instrument on html.parser with <pre> excluded. mdbook build exit 0; cargo test --test book_examples 4 passed / 0 failed; scripts/check_doctrines.sh 11/11. mdbook test fails identically on HEAD with the edits stashed - pre-existing, root-caused, surfaced.` | `repaired (whitespace-only); the run-on-enumeration residue is registered as .3 rather than claimed as fixed` |
| `2026-08-02` | `.0` | `registered from an owner finding, measured first at 9060993: mdbook build book, then over book/book-out/*.html (main only, print.html + 404.html excluded) — 1,244 rendered paragraphs across 30 chapters, 6 over 1,500 chars, worst 22,908 in architecture.html. Source confirmed: book/src/architecture.md lines 658-903 are 246 consecutive non-blank lines. Ownership search run against the six book/live-doc trees and TABLE-RENDER-FIDELITY; none owns paragraph structure.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `ebd7869` — `BOOK-PARAGRAPH-BLOBS.0 — register the owner finding on wall-of-text paragraphs` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |
| `.1` | `BOOK-PARAGRAPH-BLOBS.1 — split the wall-of-text paragraphs; worst block down 62 %` | Book-only ⇒ DUT byte-identical. Whitespace-only, proven twice. |

## Changelog

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
