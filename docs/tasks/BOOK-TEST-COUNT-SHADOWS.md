# BOOK-TEST-COUNT-SHADOWS: four of five per-file test counts in the book are stale

## Metadata

- Tree ID: `BOOK-TEST-COUNT-SHADOWS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (registered; frontier `.1`)
- Owner: repo-local workflow

## Goal

`book/src/architecture.md` publishes a *"Current counts"* list of per-file unit
test totals. **Four of the five are wrong**, one by 72 %. Repair by the rung
decision [`0033`](../decisions/0033-shadow-enumeration-classification.md) already
mandates for this exact shape — **R1, deletion** — not by correcting the numbers
and not by gating them.

Found `2026-07-31` during `IR-TYPES-DECOMPOSITION.2`, when moving two tests out
of `src/ir/types.rs` required checking the book's claim about that file.

## The measurement (at `bd7dba2`, before the move)

| claim in `book/src/architecture.md` | claimed | actual `#[test]` | error |
| --- | ---: | ---: | ---: |
| `src/ir/types.rs` | 40 | **42** | −2 |
| `src/ir/validate.rs` | 26 | 26 | ✓ |
| `src/gen/cone.rs` | 42 | **43** | −1 |
| `src/emit/sv.rs` | 17 | **26** | −9 |
| `src/metrics.rs` | 18 | **31** | −13 (**72 % understated**) |

Every error is an **under**-count, which is the signature of the defect: tests get
added, the prose does not. Nothing in the repo forces the update, so the numbers
decay monotonically from the day they are written.

**A sharp illustration of why correcting them is the wrong repair:**
`IR-TYPES-DECOMPOSITION.2` moved exactly 2 tests out of `types.rs`, taking it from
42 to 40 — and thereby made the book's stale "40" *accidentally true again*. A
number that can become correct by coincidence is not carrying information.

## The doctrinal position (why this is residue, not a new discovery)

`SHADOW-ENUMERATION-SWEEP` closed on `2026-07-30` asserting the class was fully
handled: *"either derived away (R1), compiler-maintained (R2), guarded by a
derived `#[test]` (R3), or held by the registered `ENUMERATION-PARITY` doctrine
(R4)."* **These five counts are in none of those four buckets**, and they satisfy
all three of decision `0033` rule (a)'s tests:

1. **derivable** — `grep -c '#\[test\]' <file>`;
2. **growth-coupled** — every added test requires a matching prose edit;
3. **silent** — no compile error, no failing test, no gate.

So the closing claim was **too strong**. That is recorded here rather than edited
into the closed tree: history stays raw (decision `0031`), and a closed tree's
claim is repaired by a *new* tree that measures it, not by rewriting the old one.
This is also decision `0033`'s own rule (0) biting its author: *never write
"currently correct" without measuring it.*

The sibling residue found the same day — `ENUMERATION-PARITY`'s own steering
extractor silently reading 7 of 8 categories, i.e. bucket **R4** being partly
hollow — is owned by
[`PARITY-EXTRACTOR-ARM-SHAPE-GAP`](PARITY-EXTRACTOR-ARM-SHAPE-GAP.md). Two
independent residues of one closing claim, found within an hour of each other,
is itself the finding: the claim was not measured when it was made.

## Non-Goals

- **Not "update the numbers."** That restores accuracy for exactly as long as it
  takes someone to add a test, and re-arms the same trap. Decision `0033`:
  a count beside a derivable list is one more copy of it.
- **Not "add a doctrine to gate the counts."** Explicitly the wrong rung — `0033`
  records that gating a redundant count *"keeps the shadow alive and spends a
  mechanism on it forever."* Deletion costs nothing and closes it permanently.
- **No test is added, removed, or renamed.** Docs-only ⇒ DUT byte-identical.

## Acceptance Criteria

- The per-file counts are gone from `book/src/architecture.md`, replaced by the
  *derivation* (how a reader gets the current number themselves) rather than a
  snapshot of it.
- The surrounding prose still tells the reader what it was really for — that unit
  tests live inline in `#[cfg(test)] mod tests` per module, and roughly where the
  test mass sits — without asserting a number that decays.
- A repo-wide sweep for the same shape in the live docs, from the **effect**
  (any prose asserting a count of a derivable set) rather than from the shape of
  this instance — decision `0033` rule (2). Whatever it finds is recorded, and
  either repaired here or split into its own leaf.
- `mdbook build` clean.

## Task Tree

- ID: `BOOK-TEST-COUNT-SHADOWS`
  Status: `active`
  Goal: `Delete the per-file test-count shadows from the book and sweep the live docs for the same shape.`
  Children: `.1` (delete + sweep + close)

- ID: `BOOK-TEST-COUNT-SHADOWS.1`
  Status: `pending`
  Goal: `Replace book/src/architecture.md's "Current counts" list with the derivation instead of the snapshot; sweep the live docs for other prose counts of derivable sets, searching from the effect rather than from this instance's shape; record what the sweep finds.`
  Acceptance: `No per-file test count remains in book/src/architecture.md; the reader can still learn where tests live and how to count them; the sweep's results are recorded in this tree; mdbook build clean; docs-only.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-TEST-COUNT-SHADOWS.1` | `pending` | **Next.** The book is the owner's only window into the project, so a stale number there misinforms rather than merely omits — the same reason decision `0033` ranked a stale Knowledge Map card as the worst case of this class. |

## Decisions

- `2026-07-31`: Registered as its own tree, separate from
  `PARITY-EXTRACTOR-ARM-SHAPE-GAP`. Both are residue of the same closed-tree
  claim, but one is a **gate that under-verifies** and the other is **prose that
  drifted**; the repairs share no code and bundling them would make each harder
  to review. Two focused trees beat one muddled one.
- `2026-07-31`: Repair rung fixed **before** the work starts, so the cheap wrong
  fix is not available later under time pressure: **R1, delete**. Not correct,
  not gate.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `BOOK-TEST-COUNT-SHADOWS` | `measured all five "Current counts" claims in book/src/architecture.md against git show HEAD:<file> | grep -c '#[test]' — 4 of 5 stale (types.rs 40 vs 42, cone.rs 42 vs 43, emit/sv.rs 17 vs 26, metrics.rs 18 vs 31; only validate.rs at 26 correct); every error an UNDER-count; confirmed all three of decision 0033 rule (a)'s tests hold for the shape` | `defect confirmed, pre-existing` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |

## Changelog

- `2026-07-31`: Created. Surfaced by `IR-TYPES-DECOMPOSITION.2`: moving two tests
  out of `src/ir/types.rs` meant reading the book's claim about that file, which
  nothing had done since it was written. The same split surfaced the sibling
  `ENUMERATION-PARITY` extractor gap. A refactor that forces someone to re-read
  the claims about the thing being refactored is worth something beyond the
  refactor.
