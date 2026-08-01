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
  Status: `pending`
  Goal: `Split the 6 rendered paragraphs over 1,500 characters at genuine topic boundaries by inserting blank lines only.`
  Acceptance: `The diff adds blank lines and changes no words — verifiable with git diff --ignore-blank-lines showing no content hunks. The post-repair census is re-run and reported against the pre-repair one. mdbook build + mdbook test green.`
  Verification: `pending`
  Commit: `pending`

- ID: `BOOK-PARAGRAPH-BLOBS.2`
  Status: `pending`
  Goal: `Decide whether paragraph size should be watched, and by what — a gate, a by-product route, or a recorded acceptance.`
  Acceptance: `States the by-product alternative considered before any gate is proposed. Over-gating is a defect in its own right (DOCTRINE_ENFORCEMENT.md §9, decision 0033 test (2)). "Recorded acceptance" is a legitimate outcome; a threshold, if any, must be derived rather than fitted.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-PARAGRAPH-BLOBS.1` | `pending` | **Next.** The owner named a concrete reading defect and it is measured; repair before deciding what, if anything, should watch it. |
| 2 | `BOOK-PARAGRAPH-BLOBS.2` | `pending` | Decide against a repaired baseline, not a broken one. |

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
- **Are there blobs the `<p>` census cannot see?** Long list items and table cells render outside
  `<p>` and are not counted. The owner's complaint was about paragraphs, so this is scoped
  deliberately — but it is a stated limit, not a measured absence.

## Blockers

- None. The owner explicitly called this **not urgent**.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-02` | `.0` | `registered from an owner finding, measured first at 9060993: mdbook build book, then over book/book-out/*.html (main only, print.html + 404.html excluded) — 1,244 rendered paragraphs across 30 chapters, 6 over 1,500 chars, worst 22,908 in architecture.html. Source confirmed: book/src/architecture.md lines 658-903 are 246 consecutive non-blank lines. Ownership search run against the six book/live-doc trees and TABLE-RENDER-FIDELITY; none owns paragraph structure.` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.0` (registration) | `BOOK-PARAGRAPH-BLOBS.0 — register the owner finding on wall-of-text paragraphs` | Docs-only; no work leaf executed yet. `.0` is this repo's registration-commit convention, required because `.githooks/commit-msg` rejects a subject naming no leaf. |

## Changelog

- `2026-08-02`: Created from an owner finding — the rendered book stitches paragraphs into blobs, and
  the owner reviews the book rather than the code. Measured before registering: **6 of 1,244**
  rendered paragraphs exceed 1,500 characters, the worst being a single **22,908-character** `<p>` in
  `architecture.html` that is **246 consecutive blank-line-free source lines** and roughly **71 %** of
  that chapter's prose. The cause is **accretion** — successive slices each appended one sentence to
  an existing paragraph, every diff three lines long — which makes this the first independent
  instance of the rule landed one commit earlier at `UNGATED-PRACTICE-AUDIT.1`: **paragraph length is
  a by-product of nothing**, so nothing reported it. Scoped to a **whitespace-only** repair with no
  wording change, and explicitly not a mandate to gate.
