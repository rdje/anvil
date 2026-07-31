# BOOK-TEST-COUNT-SHADOWS: four of five per-file test counts in the book are stale

## Metadata

- Tree ID: `BOOK-TEST-COUNT-SHADOWS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — counts deleted + live docs swept; frontier `.2`, the one find `.1` declined to guess at)
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
  Children: `.1` (delete + sweep), `.2` (the one sweep find `.1` did not guess at)

- ID: `BOOK-TEST-COUNT-SHADOWS.1`
  Status: `done`
  Goal: `Replace book/src/architecture.md's "Current counts" list with the derivation instead of the snapshot; sweep the live docs for other prose counts of derivable sets, searching from the effect rather than from this instance's shape; record what the sweep finds.`
  Acceptance: `No per-file test count remains in book/src/architecture.md; the reader can still learn where tests live and how to count them; the sweep's results are recorded in this tree; mdbook build clean; docs-only.`
  Verification: `done — the five per-file counts are GONE from book/src/architecture.md, replaced by three runnable derivations (per-file grep, whole-crate grep+awk, `cargo test --lib -- --list`) plus the per-file prose describing what each cluster is FOR, which is the part that was actually carrying information. Re-measured at f335926 before deleting: types.rs 40 (claimed 40 — ACCIDENTALLY TRUE, exactly as this tree predicted: IR-TYPES-DECOMPOSITION.2 moved 2 tests out and walked 42 down onto the stale number), validate.rs 26 (claimed 26), cone.rs 43 (claimed 42), emit/sv.rs 26 (claimed 17), metrics.rs 31 (claimed 18). A knob_id.rs bullet was added for the cluster COVERAGE-STEERED-GENERATION.6 just reshaped. SWEEP (94 live-doc files; CHANGES.md / DEVELOPMENT_NOTES.md / docs/tasks/*.md EXCLUDED BY KIND — an append-only dated measurement is honest history, not a standing claim), keyed on the EFFECT (a numeral immediately qualifying a countable repo noun) rather than on this instance's shape, per decision 0033 rule (2). It found 2 more live shadows and, just as usefully, three FALSE POSITIVES that the three-part test correctly rejects. FOUND AND REPAIRED HERE: `book/src/api-tools.md` "exposes **10 tools**" and `book/src/api-reference.md` "the 10 tools: <all ten listed>" + "the 5 workflow prompts" — all three derivable (tools/list, prompts/list), growth-coupled and silent. MEASURED AGAINST THE RUNNING SERVER FIRST: both are CURRENTLY CORRECT (10 tools, 5 prompts), so these were LATENT, not live inconsistencies — stated precisely rather than overstated, per PARITY-EXTRACTOR-ARM-SHAPE-GAP.1's lesson that whether the guarded thing drifted must be measured separately from whether the guard works. Repaired by DELETION (R1): api-tools.md now tells the reader to ask the server (`tools/list`), and api-reference.md keeps the LIST and drops the NUMBER beside it — decision 0033's "a number beside a list is one more copy of it". FALSE POSITIVES, rejected with the reason: (a) structured-emission.md's "the six surfaces above" / "the other six surfaces" is POSITIONAL NARRATIVE about the moment the 7th surface landed and stays true as the set grows — test (2) fails; (b) its "nine surfaces" / ">= 8" mirror the REAL identifiers `saw_all_nine_emit_surfaces_in_one_module` / `saw_all_emit_surfaces_in_one_module`, so a 10th surface forces a rename in code — test (3) fails, it is not silent; (c) CODEBASE_ANALYSIS.md's "all 7 categories" is the Phase-7 microdesign MANIFEST FACT categories, not the steering taxonomy, and 7 is correct for that set — a shape-keyed sweep would have "fixed" it wrongly. SPLIT OUT as .2 rather than guessed at: USER_GUIDE.md:464 "The 16 knobs documented below as config-file knobs" — a real same-class candidate, but verifying it means reading the section it points at, and it carries a second count ("Three knobs stay config-file-only") whose list follows it. Checks: mdbook build clean; cargo test --test book_examples green (two new bash blocks carry REASONED skip sentinels — both read src/ or need anvil-mcp on PATH, and the harness runs blocks in a scratch CWD, so unsentineled they would fail on a missing path rather than on being wrong); scripts/check_doctrines.sh 8/8 (book/src/api-tools.md is a declared ENUMERATION-PARITY pair-3 site, so it was re-checked after editing, per the recorded gotcha about grepping the doctrine checks for a file before touching it). Docs-only => DUT byte-identical.`
  Commit: `pending`

- ID: `BOOK-TEST-COUNT-SHADOWS.2`
  Status: `pending`
  Goal: `Resolve the one sweep find .1 deliberately did not guess at: USER_GUIDE.md:464's "The 16 knobs documented below as 'config-file' knobs are now also first-class CLI flags", plus the "Three knobs stay config-file-only: library_prob, use_async_reset, max_nodes_per_module" beside it. MEASURE both against the real Config/CLI surface first — the count may be right, wrong, or referring to a set that no longer has crisp boundaries — then apply the rung the measurement earns.`
  Acceptance: `The claim is measured against src/config.rs + the CLI flag table BEFORE any edit, and the measurement is recorded whatever it says (decision 0033 rule (0): never write "currently correct" without measuring it). If it is a shadow, repaired at rung R1 by deletion — the sentence's real content is "config knobs are also CLI flags, except these three", which needs no total. The three-name exclusion list stays: it is an authoritative enumeration of an exception, not a shadow of a derivable set, and its own count ("Three") is verifiable against the list it introduces. mdbook/USER_GUIDE consistency preserved; docs-only.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-TEST-COUNT-SHADOWS.1` | `done` | The five per-file counts are replaced by three runnable derivations; the prose that was actually informative (what each test cluster is *for*) is kept. The predicted embarrassment was **live at the time of deletion**: `types.rs` claimed 40 and measured 40, because `IR-TYPES-DECOMPOSITION.2` had walked 42 down onto the stale number. The sweep found 2 more live shadows (**repaired**: the MCP `10 tools` / `5 workflow prompts` counts, both measured **currently correct** and therefore latent, not live) and correctly **rejected three false positives**. |
| 2 | `BOOK-TEST-COUNT-SHADOWS.2` | `pending` | **Next.** `USER_GUIDE.md`'s *"The 16 knobs documented below as config-file knobs"* — the one sweep find `.1` refused to guess at, because verifying it means reading the section it points at rather than pattern-matching the sentence. Measure before editing. |

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
| `2026-07-31` | `BOOK-TEST-COUNT-SHADOWS.1` | `re-measured the five claims at f335926 (types 40 vs claimed 40 — accidentally true; validate 26 vs 26; cone 43 vs 42; emit/sv 26 vs 17; metrics 31 vs 18), deleted all five and replaced them with runnable derivations; swept 94 live-doc files from the EFFECT (a numeral qualifying a countable repo noun), excluding append-only history by kind; measured the 2 finds against the RUNNING MCP server (tools/list = 10, prompts/list = 5) BEFORE editing and recorded them as latent-not-live; repaired 3 sites by deletion; rejected 3 false positives with reasons; split the 1 ambiguous find into .2. mdbook build clean; cargo test --test book_examples green; scripts/check_doctrines.sh 8/8 (re-run after touching pair-3 site book/src/api-tools.md)` | `done` |
| `2026-07-31` | `BOOK-TEST-COUNT-SHADOWS` | `measured all five "Current counts" claims in book/src/architecture.md against git show HEAD:<file> | grep -c '#[test]' — 4 of 5 stale (types.rs 40 vs 42, cone.rs 42 vs 43, emit/sv.rs 17 vs 26, metrics.rs 18 vs 31; only validate.rs at 26 correct); every error an UNDER-count; confirmed all three of decision 0033 rule (a)'s tests hold for the shape` | `defect confirmed, pre-existing` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-TEST-COUNT-SHADOWS.1` | `BOOK-TEST-COUNT-SHADOWS.1 — derive the counts, do not print them` | R1 deletion in `book/src/architecture.md` + the effect-keyed sweep. Also repaired the MCP `10 tools` / `5 workflow prompts` counts (measured correct first — latent, not live). Three false positives rejected with reasons; one ambiguous find split to `.2`. |

## Changelog

- `2026-07-31`: Created. Surfaced by `IR-TYPES-DECOMPOSITION.2`: moving two tests
  out of `src/ir/types.rs` meant reading the book's claim about that file, which
  nothing had done since it was written. The same split surfaced the sibling
  `ENUMERATION-PARITY` extractor gap. A refactor that forces someone to re-read
  the claims about the thing being refactored is worth something beyond the
  refactor.
