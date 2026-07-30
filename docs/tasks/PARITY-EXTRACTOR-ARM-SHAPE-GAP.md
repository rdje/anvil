# PARITY-EXTRACTOR-ARM-SHAPE-GAP: the steering-category extractor silently reads 7 of 8

## Metadata

- Tree ID: `PARITY-EXTRACTOR-ARM-SHAPE-GAP`
- Status: `active`
- Roadmap lane: Doctrine enforcement — a gate that under-verifies
- Created: `2026-07-31`
- Last updated: `2026-07-31` (registered; frontier `.1`)
- Owner: repo-local workflow

## Goal

`ENUMERATION-PARITY` pair 4 asserts that four live docs each name **every**
`--steer` category. Its extractor has been returning **7 of the 8** categories
since it landed: `datapath` is invisible to it. Close the gap, and close it at
the *class* level rather than the instance.

Found `2026-07-31` while repointing the extractor's path during
`IR-TYPES-DECOMPOSITION.2` — not by that move, which neither caused nor worsened
it. Measured against `git show HEAD:src/ir/types.rs`, the pre-move file, so the
defect is confirmed pre-existing.

## The defect

```
$ sed -n '/pub fn category(&self)/,/^    }$/p' <file> | grep -oE '=> "[a-z]+"' | ...
emission hierarchy motifs selectors sharing state terminals     # 7
$ …the authoritative set…
datapath emission hierarchy motifs selectors sharing state terminals   # 8
```

**Root cause: the regex encodes a source *formatting* assumption, not a source
*fact*.** `=> "[a-z]+"` assumes a match arm fits on one line. Seven arms do. The
`datapath` arm's pattern is three `|`-joined variants, which exceeds the line
width, so `rustfmt` renders it as a block:

```rust
KnobId::CoefficientProb | KnobId::ConstShiftAmountProb | KnobId::ConstComparandProb => {
    "datapath"
}
```

The `=>` and the string are now on different lines, and the regex never fires.
Nobody wrote it that way; **`rustfmt` did, because the pattern got long.** So the
trigger is *"a category gains enough knobs to wrap"* — the extractor is quietly
biased against exactly the categories that grow.

## Why it went undetected

The script's own header states the defence: *"Every extraction is COUNT-FLOORED.
An extractor that silently matches nothing would make this gate pass vacuously."*
That is precisely right, and precisely insufficient here:

- **A floor catches "matched nothing", not "matched most."** The floor is `6`; the
  extractor returns `7`. `7 ≥ 6`, so it passes.
- **`covers_set` is one-directional and per-category.** It asserts each
  *extracted* category is named at each doc site. A category the extractor never
  produces is never checked **anywhere**. So `datapath` could be deleted from
  `USER_GUIDE.md`, `book/src/knobs.md`, `book/src/algorithm.md` **and**
  `docs/AGENT_INTROSPECTION_SCHEMA.md` and the doctrine would stay green — which
  is the exact failure mode pair 4 exists to prevent.

This is the repo's own recorded gotcha firing again, twice over: *"The fixture
agrees with you; the tool does not"* and *"an extractor must die on a missing
field, never fall through to something plausible"* (`MEMORY.md`). The script
header even cites `EVIDENCE-BANK-DURABILITY.5` for this class — *a broken deriver
reports something plausible rather than dying*. **Plausible is the whole danger:
7 categories looks like a healthy result.**

### The docs were fine; the gate was blind

Measured `2026-07-31`, because *"the guard was not guarding"* and *"the guarded
thing has drifted"* are different claims and only measurement separates them:

| pair-4 doc site | names `datapath`? |
| --- | --- |
| `book/src/algorithm.md` | yes (1) |
| `book/src/knobs.md` | yes (2) |
| `USER_GUIDE.md` | yes (4) |
| `docs/AGENT_INTROSPECTION_SCHEMA.md` | yes (1) |

So **no drift had occurred** and no doc needs fixing. The severity is exactly and
only *"one of eight categories has been exempt from every parity check"* — a
latent hole, not a live inconsistency. Stating that plainly matters: overstating
it would be as dishonest as missing it, and the fix is the same either way.

## Blast radius — bounded by measurement, not assumed

All six declared extractors were re-derived and compared against their
authoritative sets (decision `0033` rule (2): search the **effect**, not the
shape you already found):

| extractor | extracted | authoritative | verdict |
| --- | ---: | ---: | --- |
| `extract_doctrine_registry_ids` | 8 | 8 | exact |
| `extract_doctrine_table_ids` | 8 | 8 | exact |
| `extract_book_chapter_files` | 29 | 29 | exact |
| `extract_book_summary_links` | 29 | 29 | exact |
| `extract_adapter_ids` | 5 | 5 | exact |
| `extract_steering_categories` | **7** | **8** | **GAP** |

Five of six parse *structure a tool controls* (a bash array, a Markdown table
column, a directory listing, a link syntax). Only this one parses **`rustfmt`
output**, and it is the only one that is wrong. That is the generalisable lesson,
and it is what `.1` must fix rather than the single missing string.

## Non-Goals

- **Not "widen the regex to also match the block form."** That fixes this
  instance and leaves the class: the next arm `rustfmt` reshapes breaks it again.
- **Not "hard-code the expected count as 8."** A number beside the list is one
  more copy of it (decision `0033`), and a repair may not introduce a new
  hand-maintained list.
- **No change to the taxonomy, the docs, or any `KnobId` behaviour.** This is a
  gate defect: the four doc sites are, as it happens, all currently correct —
  verified as part of `.1`, since "the gate was blind" and "the docs were wrong"
  are different claims and only measurement separates them.

## Acceptance Criteria

- The extractor is **format-independent**: it returns all 8 categories, and still
  returns 8 after the `datapath` arm is reformatted to single-line and back.
- Negative-controlled **both ways**: the pre-fix extractor demonstrably returns 7
  (recorded), and the post-fix extractor fails loudly on a wrong path (0 entries
  ⇒ floor trip) and on a category deleted from a doc site.
- The floor is re-derived from the measured count with a comment stating why a
  floor is safe to raise and is *not* a shadow: it is **shrink-coupled, not
  growth-coupled** — adding a 9th category never requires touching it, so
  decision `0033` rule (a) test (2) fails and it is not a shadow list.
- Whether the four doc sites were *actually* stale on `datapath` is **measured
  and recorded**, not assumed either way.

## Task Tree

- ID: `PARITY-EXTRACTOR-ARM-SHAPE-GAP`
  Status: `active`
  Goal: `Make the steering-category extractor read the taxonomy as a fact, not as a formatting pattern, and record the class lesson.`
  Children: `.1` (root-cause, fix, negative-control, close)

- ID: `PARITY-EXTRACTOR-ARM-SHAPE-GAP.1`
  Status: `pending`
  Goal: `Replace the arm-shape-dependent regex with a format-independent extraction of the category strings from KnobId::category's body (the set of string literals in that function IS the taxonomy; the doc comment sits above the sed start anchor and is excluded, verified). Re-derive the floor from the measured count. Measure whether the four doc sites were actually stale on datapath. Negative-control both ways. Record the class lesson in DEVELOPMENT_NOTES.md: five of six extractors parse structure a TOOL controls and are exact; the only one parsing rustfmt output is the only one wrong.`
  Acceptance: `scripts/check_enumeration_parity.sh extracts 8 categories and the driver stays 8/8; reformatting the datapath arm single-line and back does not change the result; a wrong path still trips the floor; deleting datapath from one doc site now FAILS (it did not before).`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PARITY-EXTRACTOR-ARM-SHAPE-GAP.1` | `pending` | **Next.** A gate that cannot fail is worse than no gate — it is banked as evidence that a property holds. This one has been silently exempting one of eight categories from every doc-parity check. |

## Decisions

- `2026-07-31`: Registered as its own tree rather than folded into
  `IR-TYPES-DECOMPOSITION.2`, which is where it was found. `.2` is a pure move
  and must stay reviewable as one; a real gate defect hidden inside a 460-line
  refactor is how defects get lost. The move only *repoints* the extractor's
  path — it neither caused the gap (proven against `git show HEAD:`) nor fixed
  it.
- `2026-07-31`: The fix targets the **class**, not the instance. Parsing
  `rustfmt`'s output for a semantic set is the defect; the missing `datapath`
  string is the symptom.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `PARITY-EXTRACTOR-ARM-SHAPE-GAP` | `measured the extractor against git show HEAD:src/ir/types.rs (pre-move) -> 7 categories, datapath absent; measured the authoritative set in KnobId::category -> 8; re-derived all six declared extractors against their authoritative sets -> five exact, this one short by one; confirmed the floor (6) cannot catch it since 7 >= 6, and that covers_set is per-category so an unextracted category is unverified at every site` | `defect confirmed, pre-existing, bounded to one extractor` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PARITY-EXTRACTOR-ARM-SHAPE-GAP.1` | `pending` | |

## Changelog

- `2026-07-31`: Created. Found while repointing the extractor's file path during
  `IR-TYPES-DECOMPOSITION.2`; the split surfaced it because moving `KnobId`
  required re-running the extractor and reading its output, which nothing had
  done since it was written. That is an argument for the split, not against it.
