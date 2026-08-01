# USER-GUIDE-CLI-TABLE-SHADOW: the CLI flag table is a silent shadow of clap's registry, and it has drifted in both directions

## Metadata

- Tree ID: `USER-GUIDE-CLI-TABLE-SHADOW`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.1` **done** — audited and registered; frontier `.2`)
- Owner: repo-local workflow

## Goal

`README.md` names `USER_GUIDE.md` as **"the live CLI reference: every flag, knob, preset,
steering category, `tool_matrix` gate, and downstream-verification workflow."** Its CLI flag
table is therefore a hand-maintained list that mirrors a set which already exists — clap's
flag registry in `src/main.rs`, reachable by `anvil --help`.

Measured `2026-08-01` at `1df0071`, it has **fallen behind in both directions**.

## The measurement (at `1df0071`)

| bucket | count | meaning |
| ---: | ---: | --- |
| clap flags on `anvil --help` | **108** | the authoritative set `S` |
| in the CLI flag table | **75** | the shadow `L` |
| mentioned elsewhere in `USER_GUIDE.md` but **not** in the table | **20** | documented, but not where the reference promises |
| **absent from `USER_GUIDE.md` entirely** | **13** | undocumented |
| table rows naming a flag that **no longer exists** | **2** | `--divergence`, `--no-minimize` |

**The 13 absent flags**, listed rather than counted so the repair has a work-list:

`--hierarchy-child-input-cone-prob`, `--hierarchy-child-source-mode`,
`--hierarchy-parent-cone-instance-prob`, `--hierarchy-parent-flop-prob`,
`--hierarchy-registered-child-input-cone-prob`,
`--hierarchy-registered-sibling-mixed-support-prob`,
`--hierarchy-registered-sibling-route-prob`, `--hierarchy-sibling-route-prob`,
`--max-ast-instances`, `--max-parent-cone-instances-per-module`,
`--mux-arm-duplication-rate`, `--operand-duplication-rate`, `--version`.

**Honest caveat on that 13:** `--version` is a clap built-in and is arguably not a flag this
document owes the reader, which makes the actionable set **12**. Recorded rather than quietly
dropped, because a count that silently excludes its awkward member is the kind of number this
repo has now had four of (decision `0045`).

**Eight of the 13 are hierarchy knobs**, which is a cluster rather than a scatter — it suggests
one lane's flags were added without a documentation pass, not that thirteen authors each forgot
once.

## It is a shadow enumeration under decision `0033` — all three tests pass

1. **Derivable ✓.** `S` is clap's registry in `src/main.rs`, reachable by ordinary script
   means (`anvil --help`, or the derive attributes themselves).
2. **Growth-coupled ✓.** Every new flag *requires* a matching row for the document to be what
   `README.md` says it is. This is not an allow-list that is *supposed* to differ from `S` —
   the gap is the defect, not the content.
3. **Silent ✓.** Omitting a row produces no compile error, no failing test, and nothing at
   runtime. Measured: 13 flags are absent and every gate in the repo is green.

All three ⇒ **silent shadow: repair it by decision `0033`'s ladder.** Note the contrast with
`check_no_boot_volume_refs.sh`'s allow-list, which passes test (1) and **fails test (2)** and
is therefore authoritative — that is the case the rule was written to protect, and this is not
it.

## Why it matters

A missing row is not a cosmetic gap. `--steer` *errors* on an unknown key and a knob absent
from the reference is a **delivered and invisible** capability — the exact argument decision
`0033`'s pair 4 makes about the steering-category taxonomy. Two of the absent flags
(`--operand-duplication-rate`, `--mux-arm-duplication-rate`) are the *promoted unswept knobs*
that `SIGNOFF-AUTOMATION-EXPANSION.2b` built a whole gate to prove fire by construction; they
are gated, banked, and undocumented.

The two **stale** rows are worse than the omissions: a reader who types `--divergence` gets an
error from a document that told them it exists.

## Non-Goals

- **Not a `USER_GUIDE.md` reformat.** The prose knob sections are good and stay.
- **Not "move the 20 mentioned-but-not-tabled flags into the table"** without deciding first
  whether the table is meant to be exhaustive or a curated core. That is `.2`'s decision, and
  it must be made before any row is written — the same discipline `CHANGES-ENTRY-PLACEMENT.2`
  applied before moving anything.
- **No code change** unless `.3` concludes a mechanism is warranted.

## Acceptance Criteria

- `.2` records an explicit decision on **what the table is**: the exhaustive flag reference, or
  a curated core with prose owning the rest. The 20 mentioned-but-not-tabled flags make this a
  real question, not a formality.
- The 2 stale rows are resolved (removed, or corrected if the flags were renamed).
- If `.3` proposes a check, it obeys `DOCTRINE_ENFORCEMENT.md` §4 and is negative-controlled
  **both ways** — a removed row must fail it, and a legitimately-absent flag (if `.2` rules the
  table curated) must not.
- `scripts/check_doctrines.sh` stays green; docs-only ⇒ DUT byte-identical.

## Task Tree

- ID: `USER-GUIDE-CLI-TABLE-SHADOW`
  Status: `active`
  Goal: `Close the gap between USER_GUIDE.md's CLI flag table and clap's flag registry, after deciding what that table is meant to contain.`
  Children: `.1` (audit + register), `.2` (decide the table's contract, then repair), `.3` (the mechanism question)

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.1`
  Status: `done`
  Goal: `Audit + register (docs-only). Measure the gap in BOTH directions against clap's registry, classify it against decision 0033's three-part test, and register the tree before anything is written. No repair in this leaf.`
  Acceptance: `A measured three-bucket table (in-table / mentioned-elsewhere / absent) plus the stale-row count, the absent set LISTED rather than counted, the 0033 classification stated test by test, and no edit to USER_GUIDE.md.`
  Verification: `done — MEASURED at 1df0071 against ./target/release/anvil --help: 108 clap flags, 75 in the CLI flag table, 20 mentioned elsewhere in USER_GUIDE.md but not in the table, 13 absent from the file entirely, and 2 table rows (--divergence, --no-minimize) naming flags that NO LONGER EXIST — so the shadow has drifted in BOTH directions. Eight of the 13 absent are hierarchy knobs, i.e. one lane's flags added without a docs pass rather than thirteen independent slips. Caveat recorded rather than dropped: --version is a clap built-in, making the actionable absent set 12. 0033 CLASSIFICATION, test by test: (1) derivable — S is clap's registry in src/main.rs, reachable via anvil --help; (2) growth-coupled — README.md calls USER_GUIDE "the live CLI reference: EVERY flag", so a new flag requires a new row for the document to be what it claims; (3) silent — 13 flags are absent and every gate in the repo is green. All three hold ⇒ silent shadow, repair by the 0033 ladder. THE INSTRUMENT WAS CORRECTED MID-AUDIT AND THE FIRST NUMBER WAS WRONG: a backtick-anchored matcher reported 22 absent, because `--artifact dut` puts the flag and its value inside ONE code span, so a `\`--artifact\`` pattern never matches it. Re-measured with a word-boundary matcher: 13. That is the FOURTH extractor error in this repo's recent history (decision 0045 catalogues three others), and it is why the count above is published with the matcher that produced it. NO REPAIR ATTEMPTED, deliberately: whether the table is meant to be exhaustive or a curated core is .2's decision, and the 20 mentioned-but-not-tabled flags make that a real question rather than a formality. Docs-only ⇒ DUT byte-identical.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.1`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.2`
  Status: `pending`
  Goal: `Decide what the CLI flag table is — the exhaustive flag reference, or a curated core with prose owning the rest — record it, then repair to match: add the missing rows the decision requires and resolve the 2 stale rows (--divergence, --no-minimize).`
  Acceptance: `The contract is recorded BEFORE any row is written. If the table is exhaustive, all 12 actionable absent flags gain rows and the 20 mentioned-elsewhere flags are reconciled; if curated, the criterion for inclusion is stated and the table says so in a line above it, so the next author knows which it is. Either way the 2 stale rows go. Docs-only.`
  Verification: `pending`
  Commit: `pending`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.3`
  Status: `pending`
  Goal: `Decide whether the gap warrants a mechanism. The set is derivable from clap, so a derived check is CHEAP here in a way it was not for CHANGES-ENTRY-PLACEMENT — but it is only meaningful if .2 rules the table exhaustive; a curated table is authoritative under 0033 test (2) and must NOT be gated.`
  Acceptance: `The decision follows .2's contract, not the other way round. If a check is written it obeys DOCTRINE_ENFORCEMENT.md section 4, is negative-controlled both ways (delete a row => fails; add a flag without a row => fails), and states its extractor precisely enough to re-run — the matcher correction recorded in .1 is exactly the trap it must avoid.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `done` | Audited and registered. The measurement is the work product: **108** clap flags vs **75** table rows, **13** absent entirely, **2** rows naming flags that no longer exist. Passes all three of decision `0033`'s tests ⇒ a genuine silent shadow. |
| 2 | `USER-GUIDE-CLI-TABLE-SHADOW.2` | `pending` | **Next.** Decide the table's contract *before* writing a row — the 20 mentioned-but-not-tabled flags mean "exhaustive vs curated" is a real question, and repairing without answering it produces a table nobody can maintain. |
| 3 | `USER-GUIDE-CLI-TABLE-SHADOW.3` | `pending` | The mechanism question, and it depends entirely on `.2`: a derived check is cheap here (the set comes from clap), but gating a **curated** table would destroy the property it exists to hold — decision `0033` test (2). |

## Decisions

- `2026-08-01` (`.1`): **Registered before any edit**, per the task-tree ownership doctrine and
  the standing directive that a defect is only handled if a tree owns it. Found while adding
  `--unique-case-prob` / `--priority-case-prob` rows at
  `CAPABILITY-BREADTH-EXPANSION.4b.3`: adding two rows to a table that was already missing
  thirteen sharpened an inconsistency rather than creating one, and the honest response was to
  measure the whole table rather than quietly match the local style.
- `2026-08-01` (`.1`): **Repair deferred to `.2` deliberately.** The obvious move — add the
  missing rows — presumes the table is exhaustive, and 20 flags are documented in prose
  instead. Writing rows first would answer that question by accident.

## Open Questions

- Is the CLI flag table meant to be exhaustive, or a curated core? `README.md` says
  *"every flag"* of `USER_GUIDE.md` as a whole, which the document arguably satisfies via prose
  for 20 of them. **Deliberately unanswered until `.2`.**
- Were `--divergence` / `--no-minimize` **renamed** or **removed**? The repair differs: a rename
  needs a corrected row, a removal needs the row deleted.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `measured at 1df0071 against ./target/release/anvil --help: 108 flags / 75 table rows / 20 mentioned-not-tabled / 13 absent / 2 stale rows; 0033 three-part test applied and all three hold; matcher corrected mid-audit (backtick-anchored reported 22 absent because \`--artifact dut\` is one code span; word-boundary gives 13); tree registered; no USER_GUIDE.md edit` | `registered` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `USER-GUIDE-CLI-TABLE-SHADOW.1` | `USER-GUIDE-CLI-TABLE-SHADOW.1 — audit + register the CLI-table shadow` | Docs-only; no `USER_GUIDE.md` edit. |

## Changelog

- `2026-08-01`: Created. Found while `CAPABILITY-BREADTH-EXPANSION.4b.3` added two CLI-table
  rows and the surrounding table turned out to be missing thirteen. Measured in **both**
  directions — 13 flags absent, 2 rows stale — and classified as a genuine decision-`0033`
  silent shadow (derivable from clap, growth-coupled by `README.md`'s own promise, silent on
  omission). The instrument was corrected mid-audit, which is recorded because it is the
  fourth such correction in this repo's recent history: a backtick-anchored matcher reported
  **22** absent, missing `` `--artifact dut` `` because the flag and its value share one code
  span; the word-boundary matcher gives **13**. Repair deliberately deferred to `.2`, which
  must first decide whether the table is exhaustive or curated.
