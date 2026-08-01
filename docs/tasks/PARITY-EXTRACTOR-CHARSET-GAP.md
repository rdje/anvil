# PARITY-EXTRACTOR-CHARSET-GAP: an id containing `_` or `-` is invisible to two `ENUMERATION-PARITY` extractors, so the gate passes vacuously

## Metadata

- Tree ID: `PARITY-EXTRACTOR-CHARSET-GAP`
- Status: `closed` (`.2` landed — the class is now held by a STANDING guard, not by two corrected values)
- Roadmap lane: Doctrine enforcement — a gate that under-verifies
- Created: `2026-08-01`
- Last updated: `2026-08-01` (**REOPENED then CLOSED at `.2`** — `.1` closed the instances, not the class its own Goal names)
- Owner: repo-local workflow

## Goal

Two of `ENUMERATION-PARITY`'s extractors capture an id with a character class
narrower than the ids the authoritative set can legally contain. A value outside
that class is not *mis-read* — it is **not read at all**, so it never enters the
authoritative set, and every doc-parity assertion about it passes **vacuously**.

Close it at the **class** level, not the instance. Concretely that means three
things, and `.1` delivered only the first:

1. widen both extractors so the ids in play are read (`.1`);
2. **stop specifying a charset at all** — the quotes already delimit the value, so
   a *widened guess* is still a guess (`.2`, rung R1);
3. **make a silent skip impossible** — a standing check that an extractor accounts
   for every item it walks, so the next narrowing (of any kind, by anyone) fails
   loudly instead of passing vacuously (`.2`, rung R2).

Without (3) the repair is two corrected values plus prose, which is the bottom of
the repair ladder and the anti-pattern `DOCTRINE_ENFORCEMENT.md` §11 names: *a
rule nothing checks is a rule nothing follows.*

## How it was found (measured, not theorised)

`CAPABILITY-BREADTH-EXPANSION.4b.1` added a new `--steer` steering category. The
first name tried was **`case_qualifier`**. It appears in **zero** of the seven
fenced doc sites, so `ENUMERATION-PARITY` had to fail on all seven.

```
$ bash scripts/check_enumeration_parity.sh
[enumeration-parity] ok: steer categories <-> the knob_ids! table — book/src/algorithm.md …
[enumeration-parity] ok: … book/src/knobs.md …
[enumeration-parity] ok: … USER_GUIDE.md …
[enumeration-parity] ok: … docs/AGENT_INTROSPECTION_SCHEMA.md …
[enumeration-parity] ok: … book/src/agent-mcp.md …
[enumeration-parity] ok: … book/src/api-introspection.md …
[enumeration-parity] ok: … CODEBASE_ANALYSIS.md …
[enumeration-parity] ok: all declared enumeration pairs are in parity   # 7/7 VACUOUS
```

Renaming the category to `qualifiers` (no underscore) made the same check fail on
all seven, as it always should have. The rename was the right call on its own
merits — it matches the taxonomy's naming convention — but it **masks** the
defect rather than fixing it, which is why this tree exists.

## The defect

`scripts/check_enumeration_parity.sh`:

```sh
# (1) steering categories — the third column of the `knob_ids!` table
sed -nE 's/… "[a-z0-9_]+",[[:space:]]*"([a-z]+)";…/\1/p'
#          ^^^^^^^^^^^ name: underscore OK   ^^^^^^^ category: letters ONLY

# (2) adapter ids — every `Adapter::id()` return
grep -oE '"[a-z0-9]+"'
#          ^^^^^^^^^ no `_`, no `-`
```

**Root cause: the charset encodes a coincidence of the current membership, not a
rule about the source.** Nothing declares that a steering category must be a
single lowercase word, and the *name* column on the very same row already
permits `_` — so the asymmetry is accidental. Every category that exists today
happens to be one word, so the narrowing has never been exercised.

**The repository had already written the coincidence down as load-bearing, in
this very file, and then carried it forward anyway.** The comment above the
extractor explains why a *previous* version returned the right answer for the
wrong reason:

> *"(b) `grep -oE '"[a-z]+"'` over that over-run then skips every knob NAME only
> because each one happens to contain `_`, and no category does."*

That sentence identifies "no category contains `_`" as an accident being relied
upon. The rewrite fixed the *over-run* (a) and kept the *charset* (b), turning a
noticed accident into a live silent filter.

### Why the count floor covers one site and not the other — the general rule

Both extractors have a count floor, and probing them showed the floors behave
**differently**, for a reason worth stating as a rule.

| probe | floor before repair | what fired |
| --- | --- | --- |
| **rename** an adapter id `iverilog` → `iverilog-compile` | `>= 5`, extracts 4 | floor trips — but reports *"produced 4 entries (floor 5)"*, which points at the extractor, not at the missing doc entry |
| **add** a category `case_qualifier` | `>= 8`, extracts 8 | **nothing** — 7/7 vacuous `ok` |

The floor is **shrink-coupled, not growth-coupled** — a property the script's own
notes already state. So a charset gap that hides a **renamed** id surfaces as a
shrink and the floor catches it, while a gap that hides a **new** id is *born
invisible*: the pre-existing members still extract, the floor is satisfied, and
nothing fires.

> **Ids are added far more often than they are renamed, so the case a floor
> cannot see is the common one.**

That is why this is a real defect rather than a redundancy, and why the repair is
a control per extractor rather than a tighter floor.

### Second site, latent not live

Extractor (2) excludes `_` **and** `-`. Measured `2026-08-01`: the five current
`Adapter::id()` values (`verilator`, `yosys`, `iverilog`, `sv2v`, `slang`) are
all within `[a-z0-9]+`, so nothing is missed **today**. It is repaired in the
same pass because the failure mode is identical, because `iverilog-compile` is
already the *tool-column* spelling elsewhere in the codebase (so the shape is not
hypothetical), and because — per the table above — the floor that currently
covers it would report the wrong diagnosis even when it fires.

## Non-Goals

- **Not** a constraint on what ids may be called. The repair widens the reader; it
  does not narrow the source.
- **Not** a rewrite of `ENUMERATION-PARITY`'s pair table or its scope. Only the
  two extractors' character classes and their controls.

## Acceptance Criteria

- Both extractors read an id containing `_` (and, for the adapter extractor, `-`)
  rather than silently dropping it.
- **A control per extractor, failing before and passing after** — per
  `DOCTRINE_ENFORCEMENT.md` §9, temporarily introducing such an id must make the
  gate go **red**; that is the whole assertion, since a vacuous pass is exactly
  what is being repaired.
- The extractors' existing count floors still hold, and the recorded history
  comment gains the charset lesson beside the arm-shape one.
- **A STANDING guard, not a one-off probe** (`.2`): re-running the exact historic
  scenario — a narrowed capture plus a member outside it — must produce a **hard
  failure**, with no human remembering to re-probe. This is the criterion `.1`
  did not meet, and closing the tree without it was premature.
- `scripts/check_doctrines.sh` green; no `src/` change.

## Task Tree

- ID: `PARITY-EXTRACTOR-CHARSET-GAP`
  Status: `done`
  Goal: `Widen both ENUMERATION-PARITY extractors to the charset their sources permit, with a per-extractor control proving the gate now sees such an id.`
  Children: `PARITY-EXTRACTOR-CHARSET-GAP.1`, `PARITY-EXTRACTOR-CHARSET-GAP.2`

- ID: `PARITY-EXTRACTOR-CHARSET-GAP.1`
  Status: `done`
  Goal: `Repair both extractors: extract_steering_categories' category capture ([a-z]+ -> [a-z0-9_]+, matching the name column on the same row) and extract_adapter_ids' id capture ([a-z0-9]+ -> [a-z0-9_-]+). Record the charset lesson beside the arm-shape lesson in the extractor's history comment, since the comment already names the coincidence it now relies on. Verify each with a delete-the-subject control: temporarily rename a category / adapter id to contain a separator and confirm the gate goes RED (it currently reports ok), then restore.`
  Acceptance: `Both controls fail-before / pass-after; count floors still hold; check_doctrines.sh green; no src/ change.`
  Verification: `done — extract_steering_categories' category capture "([a-z]+)" -> "([a-z0-9_]+)" (now matching the name column on the same row) and extract_adapter_ids' "[a-z0-9]+" -> "[a-z0-9_-]+". BOTH CONTROLS MEASURED FAIL-BEFORE / PASS-AFTER on the real source, then restored to a zero diff: (A) a category renamed `qualifiers` -> `case_qualifier` produced 0 FAILs before the widening (the vacuous pass being repaired) and 7 FAILs after, each naming case_qualifier at its fenced site; (B) an adapter id renamed `iverilog` -> `iverilog-compile` produced 1 FAIL before — but the WRONG one, the count floor reporting "produced 4 entries (floor 5)", which points at the extractor rather than at the missing doc entry — and 2 correct parity FAILs after, each naming iverilog-compile. So the repair improves DIAGNOSIS at the adapter site and adds COVERAGE at the category site. Extractors still read the full authoritative sets (9 categories incl. the new `qualifiers`; 5 adapter ids), so both count floors hold. The script's history comment gains the charset lesson beside the arm-shape one, including WHY a floor covers a rename but not an addition (shrink-coupled, not growth-coupled — the script's own note) and the transferable rule: capture the charset the SOURCE permits, not the charset its current members happen to use. scripts/check_doctrines.sh green, all 9 doctrines; no src/ change.`
  Commit: `PARITY-EXTRACTOR-CHARSET-GAP.1`

- ID: `PARITY-EXTRACTOR-CHARSET-GAP.2`
  Status: `done`
  Goal: `Close the class MECHANICALLY, which .1 did not. .1 corrected two charsets and wrote prose; nothing standing would catch the next narrowing, so the repair sat at the bottom of the repair ladder while this tree's Goal says "close it at the class level, not the instance". Two rungs: (R1, derive) stop guessing a charset at all — the quotes already delimit the value, so capture "([^"]+)"; a widened guess is still a guess. (R2, standing guard) add total_or_fail: an extractor must ACCOUNT FOR every item it walks, comparing a deliberately LOOSER candidate predicate against the extraction, so a SILENTLY SKIPPED item becomes a hard failure instead of an invisible one.`
  Acceptance: `The exact historic scenario (narrowed charset + a member outside it) must produce a hard FAIL where it previously produced 7 vacuous oks; a reshaped row must fail; both extractors' outputs unchanged on a healthy tree; the guard's own circularity avoided (candidate predicate strictly looser than the extraction); honest limit stated.`
  Verification: `done — R1: both captures are now "([^"]+)" — the delimiters ARE the specification, so the capture cannot be too narrow for any value the source can express. Outputs on a healthy tree unchanged (9 categories, 5 adapter ids). R2: total_or_fail compares candidate_steering_rows (loose: an identifier followed by =>) against extract_steering_categories_raw (strict, pre-dedup, since categories legitimately repeat), and candidate_adapter_impls (a count of `fn id` impls) against extract_adapter_ids_raw. FOUR CONTROLS run on the real source, then everything restored to a zero diff: (1) re-narrowing the category charset to the ORIGINAL [a-z]+ alone => SILENT — the guard's stated limit, because every current category still fits, so nothing is skipped; (2) reshaping one knob_ids! row onto two lines (the ARM-SHAPE class, which this guard also now covers) => FAIL "walked 40 item(s) but produced 39 — it SILENTLY SKIPPED 1"; (3) an adapter id `slang.v2`, unreadable under the old charset => 2 correct parity FAILs naming it; (4) THE DECISIVE ONE — the exact historic bug reproduced, narrowed charset AND a case_qualifier category => FAIL "walked 40 item(s) but produced 38 — it SILENTLY SKIPPED 2", where before this tree the identical scenario produced 0 FAILs and 7 vacuous oks. So the class is closed in the form that matters: a member the extractor cannot read is now ALWAYS loud, never silent. check_doctrines.sh green (all 9); no src/ change.`
  Commit: `PARITY-EXTRACTOR-CHARSET-GAP.2`

## What `.2` does NOT claim (the honest limit)

Control (1) is the limit, and it is stated rather than papered over: **the guard does not detect a
narrowing at the moment it is introduced** — only at the moment a member outside the narrowed class
arrives. Re-narrow the category capture today and every existing category still matches, so nothing
is skipped and nothing fires.

That is acceptable, and it is worth being precise about why: **the moment a member outside the class
arrives is exactly the moment the defect would do harm.** Before this tree, that moment produced
seven vacuous `ok`s. After it, that moment produces a hard failure naming the skip. The window in
which a narrowed regex is *both present and harmless* is a window in which it is, by construction,
harming nothing.

Detecting the narrowing itself would need the check to know what its sources *could* legally
contain — a semantic fact no syntactic guard has, and the same reason decision `0033` (c) refuses to
ship a shadow **detector**. Guessing it would reintroduce cry-wolf, which this file records as the
failure mode that gets a gate deleted.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none — tree complete) | `done` | `.1` + `.2` landed `2026-08-01`. `.1` was **wrongly** treated as the only leaf: the defect was already measured and the repair is two character classes, so the work that mattered was the **two controls** — a vacuous pass is precisely what was being fixed, and only a failing control distinguishes the repair from the coincidence that was hiding it. |

## Decisions

- `2026-08-01`: Registered as its own tree rather than folded into
  `CAPABILITY-BREADTH-EXPANSION.4b.1`, which found it. The defect is
  **pre-existing** and independent of the case-qualifier construct — it would
  bite any future underscored category or separator-bearing adapter id — and
  [`0041`](../decisions/0041-owner-standing-directives-recorded-in-layer-c.md)
  §(a) records that a defect is only handled once a task-tree owns it. Bundling a
  silent regex widening into a feature commit is also how a repair stops being
  reviewable.
- `2026-08-01`: **Not** reopening `PARITY-EXTRACTOR-ARM-SHAPE-GAP` (`closed`).
  That tree fixed a *format* assumption (a match arm fits on one line); this is a
  *charset* assumption about the captured value. Same file, same check, different
  class of wrongness — and `0033`'s reasoning against burying a specific question
  inside a general one applies.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `PARITY-EXTRACTOR-CHARSET-GAP` | `tree registered; defect measured — a case_qualifier steering category reported ok at all 7 fenced sites; the adapter extractor's second site confirmed latent-not-live against the 5 current Adapter::id() values` | `registered` |
| `2026-08-01` | `PARITY-EXTRACTOR-CHARSET-GAP.2` | `R1 — both captures become "([^"]+)": the delimiters are the specification, so no guessed charset remains. R2 — total_or_fail added: an extractor must account for every item it walks (loose candidate predicate vs strict extraction, pre-dedup), so a silently skipped item is a hard failure. FOUR controls on the real source, all restored to a zero diff: (1) re-narrowing the charset alone is SILENT (the stated limit — nothing is skipped while every member still fits); (2) a reshaped knob_ids! row => "walked 40 but produced 39 — SILENTLY SKIPPED 1"; (3) an adapter id slang.v2 => 2 correct parity FAILs; (4) DECISIVE — the exact historic bug (narrow charset + case_qualifier) => "walked 40 but produced 38 — SILENTLY SKIPPED 2", vs 0 FAILs / 7 vacuous oks before this tree` | `done` (class closed by a STANDING guard; no src/ change) |
| `2026-08-01` | `PARITY-EXTRACTOR-CHARSET-GAP.1` | `both extractors widened; CONTROL A (category with an underscore) 0 FAILs before -> 7 after, each naming case_qualifier; CONTROL B (adapter id with a hyphen) 1 WRONG failure before (the count floor reporting "produced 4 entries (floor 5)", pointing at the extractor rather than the missing doc entry) -> 2 correct parity FAILs after, each naming iverilog-compile; both sources restored to a zero diff; extractors still read the full sets (9 categories, 5 adapter ids) so both floors hold; check_doctrines.sh green (all 9)` | `done` (repair + both controls; no src/ change) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PARITY-EXTRACTOR-CHARSET-GAP.1` | `PARITY-EXTRACTOR-CHARSET-GAP.1 — an extractor must capture the charset its source permits` | Registered and repaired in one commit; the tree file exists before the edit, per the ownership doctrine. Both extractors widened; both controls measured fail-before / pass-after on the real source, then restored to a zero diff. |

## Changelog

- `2026-08-01`: Created, from a defect measured at
  `CAPABILITY-BREADTH-EXPANSION.4b.1` when a new steering category containing an
  underscore passed all seven doc-parity sites vacuously.
- `2026-08-01`: **`.2` done — and the tree should not have been closed at `.1`.** `.1` corrected
  two charsets and wrote prose; nothing standing would have caught the next narrowing, so the
  repair sat at the **bottom** of the repair ladder (*derive → compile error → derived test →
  registered doctrine*) while this tree's own Goal says *"close it at the class level, not the
  instance"*. `.2` takes the two rungs `.1` skipped: **(R1)** stop guessing a charset — the quotes
  already delimit the value, so both captures become `"([^"]+)"`, since a *widened* guess is still
  a guess; **(R2)** `total_or_fail`, which requires an extractor to **account for every item it
  walks**, turning a silent skip into a hard failure. The decisive control: the exact historic
  scenario now reports *"walked 40 item(s) but produced 38 — it SILENTLY SKIPPED 2"* where it
  previously produced **7 vacuous `ok`s**. The guard also subsumes the older
  `PARITY-EXTRACTOR-ARM-SHAPE-GAP` class (a reshaped row is a skip). Limit stated, not hidden: it
  fires when a member outside the class *arrives*, not when the narrowing is *introduced* — which
  is the moment the defect would otherwise do harm. **Tree CLOSED, properly this time.**
- `2026-08-01`: `.1` done — both extractors widened to the charset their sources
  permit, each held by a control measured fail-before / pass-after on the real
  source. The adapter site gained **diagnosis** rather than coverage: its floor
  did fire on the probe, but reported *"produced 4 entries (floor 5)"* — pointing
  at the extractor instead of at the two docs missing the id. The durable rule,
  recorded in the script beside the arm-shape lesson: **capture the charset the
  SOURCE permits, not the charset its current members happen to use**, because a
  count floor is shrink-coupled and therefore blind to an id that is born
  invisible. **Tree CLOSED.**
