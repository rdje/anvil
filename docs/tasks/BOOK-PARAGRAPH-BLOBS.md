# BOOK-PARAGRAPH-BLOBS: the rendered book has paragraphs that are walls of text — one is 22,908 characters

## Metadata

- Tree ID: `BOOK-PARAGRAPH-BLOBS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / book fidelity
- Created: `2026-08-02`
- Last updated: `2026-08-02` (owner decided `.3`'s repair: link, don't duplicate; frontier `.3`)
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
  Children: `.1` (repair the splittable blobs, **done**), `.4` (make the instruments durable), `.3` (the run-on enumerations `.1` cannot fix), `.2` (decide whether anything watches this)

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
  Status: `pending`
  Goal: `Decide what to do about the RUN-ON ENUMERATIONS — the residue .1 structurally cannot fix, because a single sentence has no paragraph boundary to insert a break at.`
  Acceptance: `Registered by .1 as a distinct defect, not a leftover. Every remaining oversized block is one sentence listing dozens of clauses. OWNER DECIDED 2026-08-02: replace the duplicated prose with an ABSOLUTE GitHub URL to the .md that already owns it - not a list conversion, and not the deletion the agent proposed. The link MUST be absolute (https://github.com/rdje/anvil/blob/main/...); a relative ../../X.md renders as a dead .html and BOOK-LINK-TARGETS blocks it (decision 0046). Precondition VERIFIED before any content is removed - see Decisions.`
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

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-PARAGRAPH-BLOBS.3` | `pending` | **Next.** The residue is one coherent defect class — run-on enumerations that whitespace cannot touch — and it is now the whole of the remaining oversized mass. |
| 2 | `BOOK-PARAGRAPH-BLOBS.2` | `pending` | Decide what watches this, against a repaired baseline. Deliberately after `.3`: choosing a threshold while 43,092 characters of known-unfixed enumeration are still in the book would fit the number to the defect. |
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
| `2026-08-02` | `.4` | `Promoted census reproduces the validated instrument EXACTLY: 3,501 prose blocks / 30 chapters / 12 over 1,500 / mass 43,092, all 12 sizes identical — after a first draft that matched both headline numbers while inflating 9 of 12 sizes. book_list_signature.py reproduces .1's 1,325 <li> across 31 chapters. All 12 source anchors resolve. Three controls via scripts/negative_control.sh (every substitution count asserted): paragraph merge -> census FIRES (3,501->3,500 blocks, 12->13, mass 43,092->44,919); source re-wrap -> census SILENT on measurement fields; break inside <li> without continuation indent -> list signature FIRES, word proof OK (90,542 -> 90,542). One control FAILED first and was root-caused to the carrier, not the instrument (it deleted the blank line, so mdBook lazily continued the paragraph inside the same <li>). Restores cmp-verified; mdbook build exit 0; scripts/check_doctrines.sh 11/11.` | `promoted; instruments durable in scripts/ and documented in TOOLBOX.md §7` (docs+scripts only; DUT byte-identical) |
| `2026-08-02` | `.1` | `24 paragraph breaks across 5 chapters. WORST PROSE BLOCK 22,908 -> 8,704 chars (-62.0 %); architecture.html 22,908 -> 7,043 (-69.3 %); oversized mass 54,897 -> 43,092 (-21.5 %). Two independent proofs, both negative-controlled: whitespace-normalized word identity (passes on the edit, FAILS on a one-character word change) and a rendered <li> census of 1,325 items across 31 chapters (identical after; FIRES on a landed sabotage that strips a list continuation's indent, which the word proof passes). The .0 census was CORRECTED: denominator 1,244 -> 3,467, over-threshold 6 -> 11, after rebuilding the instrument on html.parser with <pre> excluded. mdbook build exit 0; cargo test --test book_examples 4 passed / 0 failed; scripts/check_doctrines.sh 11/11. mdbook test fails identically on HEAD with the edits stashed - pre-existing, root-caused, surfaced.` | `repaired (whitespace-only); the run-on-enumeration residue is registered as .3 rather than claimed as fixed` |
| `2026-08-02` | `.0` | `registered from an owner finding, measured first at 9060993: mdbook build book, then over book/book-out/*.html (main only, print.html + 404.html excluded) — 1,244 rendered paragraphs across 30 chapters, 6 over 1,500 chars, worst 22,908 in architecture.html. Source confirmed: book/src/architecture.md lines 658-903 are 246 consecutive non-blank lines. Ownership search run against the six book/live-doc trees and TABLE-RENDER-FIDELITY; none owns paragraph structure.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `ebd7869` — `BOOK-PARAGRAPH-BLOBS.0 — register the owner finding on wall-of-text paragraphs` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |
| `.1` | `df7bc6e` — `BOOK-PARAGRAPH-BLOBS.1 — split the wall-of-text paragraphs; worst block down 62 %` | Book-only ⇒ DUT byte-identical. Whitespace-only, proven twice. |
| `.4` | `d25bbe7` — `BOOK-PARAGRAPH-BLOBS.4 — promote the book-census instruments into scripts/` | Docs + `scripts/` only ⇒ DUT byte-identical. Reproduces the validated census exactly; three controls, one of which failed with the instrument innocent. |

## Changelog

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
