# BOOK-TEST-COUNT-SHADOWS: four of five per-file test counts in the book are stale

## Metadata

- Tree ID: `BOOK-TEST-COUNT-SHADOWS`
- Status: `done`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.2` **done** ⇒ tree **CLOSED**; residue split to `LIVE-DOC-REGISTRY-SHADOWS`)
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
  Status: `done`
  Goal: `Delete the per-file test-count shadows from the book and sweep the live docs for the same shape.`
  Children: `.1` (delete + sweep), `.2` (the one sweep find `.1` did not guess at) — both `done`

- ID: `BOOK-TEST-COUNT-SHADOWS.1`
  Status: `done`
  Goal: `Replace book/src/architecture.md's "Current counts" list with the derivation instead of the snapshot; sweep the live docs for other prose counts of derivable sets, searching from the effect rather than from this instance's shape; record what the sweep finds.`
  Acceptance: `No per-file test count remains in book/src/architecture.md; the reader can still learn where tests live and how to count them; the sweep's results are recorded in this tree; mdbook build clean; docs-only.`
  Verification: `done — the five per-file counts are GONE from book/src/architecture.md, replaced by three runnable derivations (per-file grep, whole-crate grep+awk, `cargo test --lib -- --list`) plus the per-file prose describing what each cluster is FOR, which is the part that was actually carrying information. Re-measured at f335926 before deleting: types.rs 40 (claimed 40 — ACCIDENTALLY TRUE, exactly as this tree predicted: IR-TYPES-DECOMPOSITION.2 moved 2 tests out and walked 42 down onto the stale number), validate.rs 26 (claimed 26), cone.rs 43 (claimed 42), emit/sv.rs 26 (claimed 17), metrics.rs 31 (claimed 18). A knob_id.rs bullet was added for the cluster COVERAGE-STEERED-GENERATION.6 just reshaped. SWEEP (94 live-doc files; CHANGES.md / DEVELOPMENT_NOTES.md / docs/tasks/*.md EXCLUDED BY KIND — an append-only dated measurement is honest history, not a standing claim), keyed on the EFFECT (a numeral immediately qualifying a countable repo noun) rather than on this instance's shape, per decision 0033 rule (2). It found 2 more live shadows and, just as usefully, three FALSE POSITIVES that the three-part test correctly rejects. FOUND AND REPAIRED HERE: `book/src/api-tools.md` "exposes **10 tools**" and `book/src/api-reference.md` "the 10 tools: <all ten listed>" + "the 5 workflow prompts" — all three derivable (tools/list, prompts/list), growth-coupled and silent. MEASURED AGAINST THE RUNNING SERVER FIRST: both are CURRENTLY CORRECT (10 tools, 5 prompts), so these were LATENT, not live inconsistencies — stated precisely rather than overstated, per PARITY-EXTRACTOR-ARM-SHAPE-GAP.1's lesson that whether the guarded thing drifted must be measured separately from whether the guard works. Repaired by DELETION (R1): api-tools.md now tells the reader to ask the server (`tools/list`), and api-reference.md keeps the LIST and drops the NUMBER beside it — decision 0033's "a number beside a list is one more copy of it". FALSE POSITIVES, rejected with the reason: (a) structured-emission.md's "the six surfaces above" / "the other six surfaces" is POSITIONAL NARRATIVE about the moment the 7th surface landed and stays true as the set grows — test (2) fails; (b) its "nine surfaces" / ">= 8" mirror the REAL identifiers `saw_all_nine_emit_surfaces_in_one_module` / `saw_all_emit_surfaces_in_one_module`, so a 10th surface forces a rename in code — test (3) fails, it is not silent; (c) CODEBASE_ANALYSIS.md's "all 7 categories" is the Phase-7 microdesign MANIFEST FACT categories, not the steering taxonomy, and 7 is correct for that set — a shape-keyed sweep would have "fixed" it wrongly. SPLIT OUT as .2 rather than guessed at: USER_GUIDE.md:464 "The 16 knobs documented below as config-file knobs" — a real same-class candidate, but verifying it means reading the section it points at, and it carries a second count ("Three knobs stay config-file-only") whose list follows it. Checks: mdbook build clean; cargo test --test book_examples green (two new bash blocks carry REASONED skip sentinels — both read src/ or need anvil-mcp on PATH, and the harness runs blocks in a scratch CWD, so unsentineled they would fail on a missing path rather than on being wrong); scripts/check_doctrines.sh 8/8 (book/src/api-tools.md is a declared ENUMERATION-PARITY pair-3 site, so it was re-checked after editing, per the recorded gotcha about grepping the doctrine checks for a file before touching it). Docs-only => DUT byte-identical.`
  Commit: `1a6f276` — `BOOK-TEST-COUNT-SHADOWS.1 — derive the counts, do not print them`

- ID: `BOOK-TEST-COUNT-SHADOWS.2`
  Status: `done`
  Goal: `Resolve the one sweep find .1 deliberately did not guess at: USER_GUIDE.md:464's "The 16 knobs documented below as 'config-file' knobs are now also first-class CLI flags", plus the "Three knobs stay config-file-only: library_prob, use_async_reset, max_nodes_per_module" beside it. MEASURE both against the real Config/CLI surface first — the count may be right, wrong, or referring to a set that no longer has crisp boundaries — then apply the rung the measurement earns.`
  Acceptance: `The claim is measured against src/config.rs + the CLI flag table BEFORE any edit, and the measurement is recorded whatever it says (decision 0033 rule (0): never write "currently correct" without measuring it). If it is a shadow, repaired at rung R1 by deletion — the sentence's real content is "config knobs are also CLI flags, except these three", which needs no total. The three-name exclusion list stays: it is an authoritative enumeration of an exception, not a shadow of a derivable set, and its own count ("Three") is verifiable against the list it introduces. mdbook/USER_GUIDE consistency preserved; docs-only.`
  Verification: `done — MEASURED FIRST (rule 0), and the claim is wrong under its own referent. The sentence read "The 16 knobs documented below as 'config-file' knobs are now also first-class CLI flags". Measured against USER_GUIDE.md itself: only FIVE of that sixteen have a bullet below that sentence (soft_union_slice_prob, function_emit_prob, generate_loop_emit_prob, task_emit_prob, cone_function_emit_prob); the other ELEVEN have no bullet anywhere in USER_GUIDE.md; and NINE knobs are documented below, four of which (multi_output_task_emit_prob, mux_if_emit_prob, case_mux_if_emit_prob, casez_mux_if_emit_prob) shipped AFTER KNOB-ERGONOMICS-AND-PRESETS.2b.1 and were never among the sixteen. So "16" was never the count of anything at that location — it is the 2b.1 commit prose carried over onto a referent it never matched, and the set below has since grown 5 -> 9. All three of decision 0033 rule (a)'s tests hold: derivable (count the bullets; diff Config against Overrides), growth-coupled (every new emission surface adds a bullet), silent (no compile error, no test, no gate — ENUMERATION-PARITY holds the CATEGORY taxonomy at this file, not this count). Repaired at rung R1 by DELETION: the number is gone, replaced by "Every knob documented below ... is now also a first-class CLI flag" plus a RUNNABLE derivation (comm -23 of Config's fields against --help's flags) so a reader re-derives rather than trusts the paragraph. THE THREE-NAME EXCLUSION LIST WAS MEASURED AND KEPT: Config has 92 fields, Overrides 88, and the difference is exactly {library_prob, max_nodes_per_module, use_async_reset} + {seed, steering}; --help confirms no flag exists for the three. It is an authoritative enumeration of an EXCEPTION, not a shadow of a derivable set, and its own "three" is verifiable against the list it introduces. TWO SELF-CAUGHT ERRORS, both before landing: (i) the first draft of the derivation wrote to /tmp, which decision 0031 / NO-BOOT-VOLUME-REFS forbids in any live doc — rewritten to process substitution, no temp path at all; (ii) RUNNING the published derivation (rather than trusting it) showed it prints FIVE names, not three: seed does NOT appear (--seed matches its field name) and child_instances_per_module_by_depth DOES (its flag is the differently-named repeatable --child-instances-per-depth) — the prose was corrected to name the two real exceptions instead of the two guessed ones. That is the recorded "the fixture agrees with you; the tool does not" gotcha, caught by running the artifact. A book-test skip sentinel added by the first draft was also removed: tests/book_examples.rs scans book/src/ only, so in USER_GUIDE.md the sentinel implies a gate that does not exist. SWEEP (effect-keyed, over all tracked *.md minus CHANGES/DEVELOPMENT_NOTES/docs-tasks/docs-evidence by kind): banked-run citations ("3 scenarios / 12 modules") are dated measurements of a specific run and are exempt by the same reasoning .1 used for append-only history. It found TWO live registry shadows at sites no gate declares, both SPLIT OUT to LIVE-DOC-REGISTRY-SHADOWS rather than repaired here (different shape — membership of an authoritative REGISTRY, not a count of a derivable set — and one needs a scripts/ change): (a) docs/knowledge/api-reference.md names "the 9 tools" and LISTS NINE, omitting the coverage tool entirely — measured against a live tools/list, which returns TEN — and also cites schema_version 1.11 against a live 1.27; a stale KM card is the worst case per the recorded gotcha, because it is read INSTEAD of re-deriving; (b) book/src/agent-mcp.md and book/src/api-introspection.md each name SIX of the EIGHT steering categories, omitting motifs and emission, and agent-mcp.md calls its six "the fixed set". REJECTED as a false positive with the reason: KNOWLEDGE_MAP.md's "the 4 analyze query schemas" describes how many schemas that CHAPTER details (api-introspection.md has 5 ### sections, four of them query schemas, and says so), not how many queries the registry has — the chapter itself lists all fourteen in its stability section, so this mirrors the page accurately and a shape-keyed sweep would have "fixed" it wrongly. Checks: cargo check --all-targets clean; the published derivation re-run and its output matched the corrected prose exactly; scripts/check_doctrines.sh 8/8 after git add (USER_GUIDE.md is a declared ENUMERATION-PARITY pair-4 site, so it was re-checked per the recorded gotcha about grepping the doctrine checks for a file before touching it). Docs-only => DUT byte-identical.`
  Commit: `715019b` — `BOOK-TEST-COUNT-SHADOWS.2 — the count matched neither referent`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BOOK-TEST-COUNT-SHADOWS.1` | `done` | The five per-file counts are replaced by three runnable derivations; the prose that was actually informative (what each test cluster is *for*) is kept. The predicted embarrassment was **live at the time of deletion**: `types.rs` claimed 40 and measured 40, because `IR-TYPES-DECOMPOSITION.2` had walked 42 down onto the stale number. The sweep found 2 more live shadows (**repaired**: the MCP `10 tools` / `5 workflow prompts` counts, both measured **currently correct** and therefore latent, not live) and correctly **rejected three false positives**. |
| 2 | `BOOK-TEST-COUNT-SHADOWS.2` | `done` | `USER_GUIDE.md`'s *"The 16 knobs documented below as config-file knobs"* — measured before editing, and **wrong under its own referent**: only 5 of that 16 are documented below, 11 appear nowhere in the file, and 9 knobs are documented below (4 of them post-dating the 16). Repaired at **R1 by deletion** + a runnable derivation; the three-name exclusion list measured correct and kept. Two self-caught errors before landing (a `/tmp` path that breaches `NO-BOOT-VOLUME-REFS`; a derivation whose real output named two different exceptions than the prose guessed). Sweep split 2 live registry shadows to `LIVE-DOC-REGISTRY-SHADOWS` and rejected 1 false positive with a reason. |

**Tree complete.** Both leaves are `done`; the per-file test counts and the
`USER_GUIDE` knob count are gone, each replaced by a derivation a reader can
run. The residue this tree's own sweep turned up is owned by
[`LIVE-DOC-REGISTRY-SHADOWS`](LIVE-DOC-REGISTRY-SHADOWS.md) — a different
defect shape (membership of an authoritative *registry*, at sites the
`ENUMERATION-PARITY` doctrine does not declare), deliberately not folded in
here.

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
| `2026-07-31` | `BOOK-TEST-COUNT-SHADOWS.2` | `measured USER_GUIDE.md's "16 knobs documented below" against BOTH candidate referents before editing: 5 of the 2b.1 sixteen have a bullet below the sentence, 11 have none anywhere in the file, and 9 knobs are documented below (4 post-dating the sixteen) — so 16 matched neither. Measured Config (92 fields) minus Overrides (88) = {library_prob, max_nodes_per_module, use_async_reset} + {seed, steering}, and confirmed via --help that the three named knobs really have no flag, so the exclusion list is correct and stays. Deleted the count (R1) and published a runnable derivation in its place; then RAN that derivation, which printed two exceptions the draft prose had guessed wrong (child_instances_per_module_by_depth in, seed out) and one /tmp path that breaches NO-BOOT-VOLUME-REFS — both fixed before staging. Effect-keyed sweep over all tracked *.md minus append-only/dated-evidence kinds: 2 live registry shadows split to LIVE-DOC-REGISTRY-SHADOWS, 1 false positive rejected with a reason. cargo check --all-targets clean; scripts/check_doctrines.sh 8/8 after git add` | `done` |
| `2026-07-31` | `BOOK-TEST-COUNT-SHADOWS` | `measured all five "Current counts" claims in book/src/architecture.md against git show HEAD:<file> \| grep -c '#[test]' — 4 of 5 stale (types.rs 40 vs 42, cone.rs 42 vs 43, emit/sv.rs 17 vs 26, metrics.rs 18 vs 31; only validate.rs at 26 correct); every error an UNDER-count; confirmed all three of decision 0033 rule (a)'s tests hold for the shape` | `defect confirmed, pre-existing` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-TEST-COUNT-SHADOWS.1` | `BOOK-TEST-COUNT-SHADOWS.1 — derive the counts, do not print them` | R1 deletion in `book/src/architecture.md` + the effect-keyed sweep. Also repaired the MCP `10 tools` / `5 workflow prompts` counts (measured correct first — latent, not live). Three false positives rejected with reasons; one ambiguous find split to `.2`. |
| `BOOK-TEST-COUNT-SHADOWS.2` | `BOOK-TEST-COUNT-SHADOWS.2 — the count matched neither referent` | R1 deletion of `USER_GUIDE.md`'s `16 knobs` + a runnable `comm -23` derivation; the measured-correct three-name exclusion list kept. Closes the tree. Two live finds split to `LIVE-DOC-REGISTRY-SHADOWS`. |

## Changelog

- `2026-07-31`: **CLOSED at `.2`.** The count did not merely decay — it never
  matched. Written as *"the 16 knobs documented below"*, only 5 of that 16 were
  ever documented below it, and the section it points at has since grown to 9.
  That is a sharper version of the tree's own thesis: `.1` deleted numbers that
  *became* wrong, `.2` deleted one that was wrong the day it was written, because
  a count is asserted about a referent and nothing checks that the referent is the
  set the author had in mind. Both leaves landed the same rung (**R1, delete**),
  and in both the replacement is a command rather than a number. Also earned:
  **run the derivation you publish.** The `.2` snippet was correct-looking and
  wrong — it printed two exception names the prose had guessed differently — and
  only executing it surfaced that, exactly as the recorded *"the fixture agrees
  with you; the tool does not"* gotcha predicts. Residue (registry-membership
  shadows at undeclared gate sites) went to `LIVE-DOC-REGISTRY-SHADOWS` rather
  than being folded in, on the same "two focused trees beat one muddled one"
  reasoning that separated this tree from `PARITY-EXTRACTOR-ARM-SHAPE-GAP`.
- `2026-07-31`: Created. Surfaced by `IR-TYPES-DECOMPOSITION.2`: moving two tests
  out of `src/ir/types.rs` meant reading the book's claim about that file, which
  nothing had done since it was written. The same split surfaced the sibling
  `ENUMERATION-PARITY` extractor gap. A refactor that forces someone to re-read
  the claims about the thing being refactored is worth something beyond the
  refactor.
