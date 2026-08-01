# PARITY-EXTRACTOR-CHARSET-GAP: an id containing `_` or `-` is invisible to two `ENUMERATION-PARITY` extractors, so the gate passes vacuously

## Metadata

- Tree ID: `PARITY-EXTRACTOR-CHARSET-GAP`
- Status: `closed` (`.1` landed — both extractors widened, both controls fail-before/pass-after)
- Roadmap lane: Doctrine enforcement — a gate that under-verifies
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.1` landed; **tree CLOSED**)
- Owner: repo-local workflow

## Goal

Two of `ENUMERATION-PARITY`'s extractors capture an id with a character class
narrower than the ids the authoritative set can legally contain. A value outside
that class is not *mis-read* — it is **not read at all**, so it never enters the
authoritative set, and every doc-parity assertion about it passes **vacuously**.

Close it at the **class** level, not the instance: widen both extractors to the
charset their sources actually permit, and prove each one *sees* such an id with
a control that fails before the fix and passes after.

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
- `scripts/check_doctrines.sh` green; no `src/` change.

## Task Tree

- ID: `PARITY-EXTRACTOR-CHARSET-GAP`
  Status: `done`
  Goal: `Widen both ENUMERATION-PARITY extractors to the charset their sources permit, with a per-extractor control proving the gate now sees such an id.`
  Children: `PARITY-EXTRACTOR-CHARSET-GAP.1`

- ID: `PARITY-EXTRACTOR-CHARSET-GAP.1`
  Status: `done`
  Goal: `Repair both extractors: extract_steering_categories' category capture ([a-z]+ -> [a-z0-9_]+, matching the name column on the same row) and extract_adapter_ids' id capture ([a-z0-9]+ -> [a-z0-9_-]+). Record the charset lesson beside the arm-shape lesson in the extractor's history comment, since the comment already names the coincidence it now relies on. Verify each with a delete-the-subject control: temporarily rename a category / adapter id to contain a separator and confirm the gate goes RED (it currently reports ok), then restore.`
  Acceptance: `Both controls fail-before / pass-after; count floors still hold; check_doctrines.sh green; no src/ change.`
  Verification: `done — extract_steering_categories' category capture "([a-z]+)" -> "([a-z0-9_]+)" (now matching the name column on the same row) and extract_adapter_ids' "[a-z0-9]+" -> "[a-z0-9_-]+". BOTH CONTROLS MEASURED FAIL-BEFORE / PASS-AFTER on the real source, then restored to a zero diff: (A) a category renamed `qualifiers` -> `case_qualifier` produced 0 FAILs before the widening (the vacuous pass being repaired) and 7 FAILs after, each naming case_qualifier at its fenced site; (B) an adapter id renamed `iverilog` -> `iverilog-compile` produced 1 FAIL before — but the WRONG one, the count floor reporting "produced 4 entries (floor 5)", which points at the extractor rather than at the missing doc entry — and 2 correct parity FAILs after, each naming iverilog-compile. So the repair improves DIAGNOSIS at the adapter site and adds COVERAGE at the category site. Extractors still read the full authoritative sets (9 categories incl. the new `qualifiers`; 5 adapter ids), so both count floors hold. The script's history comment gains the charset lesson beside the arm-shape one, including WHY a floor covers a rename but not an addition (shrink-coupled, not growth-coupled — the script's own note) and the transferable rule: capture the charset the SOURCE permits, not the charset its current members happen to use. scripts/check_doctrines.sh green, all 9 doctrines; no src/ change.`
  Commit: `PARITY-EXTRACTOR-CHARSET-GAP.1`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none — tree complete) | `done` | `.1` landed `2026-08-01`. It was the only leaf: the defect was already measured and the repair is two character classes, so the work that mattered was the **two controls** — a vacuous pass is precisely what was being fixed, and only a failing control distinguishes the repair from the coincidence that was hiding it. |

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
| `2026-08-01` | `PARITY-EXTRACTOR-CHARSET-GAP.1` | `both extractors widened; CONTROL A (category with an underscore) 0 FAILs before -> 7 after, each naming case_qualifier; CONTROL B (adapter id with a hyphen) 1 WRONG failure before (the count floor reporting "produced 4 entries (floor 5)", pointing at the extractor rather than the missing doc entry) -> 2 correct parity FAILs after, each naming iverilog-compile; both sources restored to a zero diff; extractors still read the full sets (9 categories, 5 adapter ids) so both floors hold; check_doctrines.sh green (all 9)` | `done` (repair + both controls; no src/ change) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PARITY-EXTRACTOR-CHARSET-GAP.1` | `PARITY-EXTRACTOR-CHARSET-GAP.1 — an extractor must capture the charset its source permits` | Registered and repaired in one commit; the tree file exists before the edit, per the ownership doctrine. Both extractors widened; both controls measured fail-before / pass-after on the real source, then restored to a zero diff. |

## Changelog

- `2026-08-01`: Created, from a defect measured at
  `CAPABILITY-BREADTH-EXPANSION.4b.1` when a new steering category containing an
  underscore passed all seven doc-parity sites vacuously.
- `2026-08-01`: `.1` done — both extractors widened to the charset their sources
  permit, each held by a control measured fail-before / pass-after on the real
  source. The adapter site gained **diagnosis** rather than coverage: its floor
  did fire on the probe, but reported *"produced 4 entries (floor 5)"* — pointing
  at the extractor instead of at the two docs missing the id. The durable rule,
  recorded in the script beside the arm-shape lesson: **capture the charset the
  SOURCE permits, not the charset its current members happen to use**, because a
  count floor is shrink-coupled and therefore blind to an id that is born
  invisible. **Tree CLOSED.**
