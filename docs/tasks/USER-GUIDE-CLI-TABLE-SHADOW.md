# USER-GUIDE-CLI-TABLE-SHADOW: the CLI flag table is a silent shadow of clap's registry, and it has drifted in both directions

## Metadata

- Tree ID: `USER-GUIDE-CLI-TABLE-SHADOW`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.2` **done** — contract recorded, table repaired 75 → 93, `.1`'s
  measurement corrected, `.4` registered; frontier `.3`)
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

### Correction (`.2`, `2026-08-01`) — two of those five numbers were wrong, from one root cause

The table above is left as `.1` measured it (layer-B history is not retro-edited); this block
records what `.2` re-measured at `f2d282e` and why they differ. **`.1` measured a document that
documents TWO clap commands against ONE command's `--help`.**

| bucket | `.1` said | actually | why |
| --- | ---: | ---: | --- |
| clap flags — the authoritative set `S` | **108** | **107** | `.1` counted every `--x` **string** in `anvil --help`. One of them, `--diff-sim`, is not an option of `anvil`: it is quoted in prose inside the `hunt` subcommand's one-line description in the `Commands:` block. |
| in the CLI flag table | 75 | 75 | ✓ confirmed |
| mentioned elsewhere, not tabled | **20** | **19** | the 20th was the same phantom `--diff-sim`. |
| absent from `USER_GUIDE.md` entirely | 13 | 13 | ✓ confirmed, member for member |
| table rows naming a flag that no longer exists | **2** | **0** | `--divergence` and `--no-minimize` **exist** — as `anvil hunt` options (`src/main.rs` `HuntCommand`), and both are correctly tabled in the **`anvil hunt` table**, which is a different table for a different command. Measured against `anvil hunt --help` that table is **10/10 complete** (only `--help` itself is untabled, as in every table here). |

**One root cause, and it is the transferable part:** `USER_GUIDE.md` holds two flag tables against
two **disjoint clap namespaces** — `anvil` and `anvil hunt` — and `.1`'s extractor pooled the
document's flag rows into one population and compared them to one command's registry. An
extractor over a multi-command CLI must be **command-scoped**; and `--help` output quotes *other*
commands' flags in prose, so "looks like a flag in `--help`" is not "is a flag of this command".

This is the **fifth** instrument error in this repo's recent history — decision `0045` catalogues
three, `.1` recorded a fourth (the backtick-anchored matcher) *in the very leaf this block
corrects*. Filed as a fact card rather than only as prose, per the standing rule that a finding is
not closed until something mechanical fails if it recurs; the mechanism question is `.3`'s, and
this correction is the single most important input to it.

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
  Goal: `Close the gap between the live docs' hand-maintained flag lists and clap's flag registry, after deciding what each list is meant to contain.`
  Children: `.1` (audit + register), `.2` (decide the table's contract, then repair), `.3` (the mechanism question), `.4` (the book's second copy of the same shadow)

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.1`
  Status: `done`
  Goal: `Audit + register (docs-only). Measure the gap in BOTH directions against clap's registry, classify it against decision 0033's three-part test, and register the tree before anything is written. No repair in this leaf.`
  Acceptance: `A measured three-bucket table (in-table / mentioned-elsewhere / absent) plus the stale-row count, the absent set LISTED rather than counted, the 0033 classification stated test by test, and no edit to USER_GUIDE.md.`
  Verification: `done — MEASURED at 1df0071 against ./target/release/anvil --help: 108 clap flags, 75 in the CLI flag table, 20 mentioned elsewhere in USER_GUIDE.md but not in the table, 13 absent from the file entirely, and 2 table rows (--divergence, --no-minimize) naming flags that NO LONGER EXIST — so the shadow has drifted in BOTH directions. Eight of the 13 absent are hierarchy knobs, i.e. one lane's flags added without a docs pass rather than thirteen independent slips. Caveat recorded rather than dropped: --version is a clap built-in, making the actionable absent set 12. 0033 CLASSIFICATION, test by test: (1) derivable — S is clap's registry in src/main.rs, reachable via anvil --help; (2) growth-coupled — README.md calls USER_GUIDE "the live CLI reference: EVERY flag", so a new flag requires a new row for the document to be what it claims; (3) silent — 13 flags are absent and every gate in the repo is green. All three hold ⇒ silent shadow, repair by the 0033 ladder. THE INSTRUMENT WAS CORRECTED MID-AUDIT AND THE FIRST NUMBER WAS WRONG: a backtick-anchored matcher reported 22 absent, because `--artifact dut` puts the flag and its value inside ONE code span, so a `\`--artifact\`` pattern never matches it. Re-measured with a word-boundary matcher: 13. That is the FOURTH extractor error in this repo's recent history (decision 0045 catalogues three others), and it is why the count above is published with the matcher that produced it. NO REPAIR ATTEMPTED, deliberately: whether the table is meant to be exhaustive or a curated core is .2's decision, and the 20 mentioned-but-not-tabled flags make that a real question rather than a formality. Docs-only ⇒ DUT byte-identical.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.1`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.2`
  Status: `done`
  Goal: `Decide what the CLI flag table is — the exhaustive flag reference, or a curated core with prose owning the rest — record it, then repair to match: add the missing rows the decision requires and resolve the 2 stale rows (--divergence, --no-minimize).`
  Acceptance: `The contract is recorded BEFORE any row is written. If the table is exhaustive, all 12 actionable absent flags gain rows and the 20 mentioned-elsewhere flags are reconciled; if curated, the criterion for inclusion is stated and the table says so in a line above it, so the next author knows which it is. Either way the 2 stale rows go. Docs-only.`
  Verification: `done — CONTRACT DECIDED AND WRITTEN FIRST, and it is NEITHER of the two options the leaf was framed around. "Exhaustive over anvil --help" would have forced rows for --seed / --out / --count / --trace / --help / --version, which the document already covers in its own sections; "curated core" would have left the criterion to taste and given .3 nothing to gate. The contract landed is EXHAUSTIVE OVER A DERIVED SET: every flag that sets a KNOB has a row — directly (its value is a Config field, so it is equally settable in --config knobs.json) or as a convenience flag that sets several at once — and flags that select a MODE rather than a knob VALUE are documented by their own sections and deliberately not tabled. THE SET IS DERIVABLE, WHICH IS THE WHOLE POINT: the knob side is the CLI projection of config.rs::Overrides (90 fields, 89 name-identical + child_instances_per_module_by_depth exposed as --child-instances-per-depth) plus the three convenience flags --profile / --full-factorization / --no-full-factorization. MEASURED AT f2d282e, and the partition is TOTAL AND DISJOINT: 93 knob + 14 mode = 107 = |anvil --help|, zero flags in neither, zero in both — a total_or_fail property, not a spot check (PARITY-EXTRACTOR-CHARSET-GAP.2's rule applied to a docs audit). REPAIR: the table went 75 -> 93 rows, +18, each inserted beside its topical neighbours rather than appended — 9 hierarchy (8 of the absent cluster + --max-parent-cone-instances-per-module), 4 emit-projections (--multi-output-task-emit-prob / --mux-if-emit-prob / --case-mux-if-emit-prob / --casez-mux-if-emit-prob), 3 uniqueness (--max-ast-instances / --operand-duplication-rate / --mux-arm-duplication-rate, the two SIGNOFF-AUTOMATION-EXPANSION.2b promoted-unswept knobs among them), 2 memory-governor (--max-rss-mb / --ram-abort-pct). NOTE THE CRITERION EARNED SIX ROWS THE .1 FRAMING WOULD HAVE MISSED: 12 of the 18 are the "absent entirely" set, but 6 more were mentioned in prose and are genuine knobs — the "add the 12 absent flags" repair would have left the table incomplete on its own new contract. It also EXCLUDES --version on principle (a mode flag) rather than as .1s "honest caveat", so the awkward member stops being awkward. RE-MEASURED AFTER THE EDIT: 93 rows, 0 knob flags missing, 0 rows naming a non-knob. THE 2 STALE ROWS WERE NOT STALE — see the Correction block above; --divergence and --no-minimize are live anvil hunt options correctly tabled in the anvil hunt table, which is itself 10/10 complete against anvil hunt --help. The contract paragraph therefore also states that anvil hunt is a separate command with a separate namespace and links to its table, so the next reader cannot repeat .1s conflation. DELIBERATELY NO COUNT AND NO MEMBER LIST in the contract paragraph: decision 0033 — a number beside a list is one more copy of it, and naming the 14 mode flags would have re-imported the exact shadow this leaf removes, inside the sentence that removes it. TWO GATES FIRED ON THIS LEAF'S OWN PROSE AND BOTH WERE RIGHT, recorded because it is the enforcement working rather than a footnote: (1) TABLE-RENDER-FIDELITY caught the docs/TASK_TREE.md index row writing the cardinality as `|anvil --help|` and DROPPING 1,242 rendered characters to two unescaped pipes — inside a code span, which is exactly the protection that does not exist — in a row about being rigorous with sets; escaped as \\|. (2) EVIDENCE-CITATIONS refused the new in-page link [..](#anvil-hunt-turnkey-cli-bug-hunt) because the anchor slug matches the anvil-<name> bank-citation shape; classified in docs/evidence/INVENTORY.md section 2 (the sanctioned growable list) as a THIRD collision class after binaries and directories — a Markdown heading anchor; any heading beginning "anvil ..." slugifies into the citation shape. Section 2's own heading count was stale at (19) against 20 entries and is corrected to (21). Checks: scripts/check_doctrines.sh — ALL 10 registered doctrines hold, TABLE-RENDER-FIDELITY ok at 2,505 data rows / 434 tables / 256 tracked *.md; knowledge-map check OK, in sync (116 -> 117 facts); cargo check --all-targets / clippy -D warnings / fmt --check green; cargo test 1,087 passed / 0 failed / 19 ignored across 17 targets incl. tests/snapshots.rs, run UNPIPED because `cargo test | tail` reports tail's status (the repo's own recorded gotcha, and the first run here was piped); docs-only => DUT byte-identical.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.2`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.3`
  Status: `pending`
  Goal: `Decide whether the gap warrants a mechanism. The set is derivable from clap, so a derived check is CHEAP here in a way it was not for CHANGES-ENTRY-PLACEMENT — but it is only meaningful if .2 rules the table exhaustive; a curated table is authoritative under 0033 test (2) and must NOT be gated.`
  Acceptance: `The decision follows .2's contract, not the other way round. If a check is written it obeys DOCTRINE_ENFORCEMENT.md section 4, is negative-controlled both ways (delete a row => fails; add a flag without a row => fails), and states its extractor precisely enough to re-run — the matcher correction recorded in .1 is exactly the trap it must avoid.`
  Verification: `pending`
  Commit: `pending`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.4`
  Status: `pending`
  Goal: `The SECOND site: book/src/knobs.md's "## CLI coverage" section is the same shadow of the same set. It says in its own words "The canonical list comes from anvil --help; the snapshot below is accurate as of this commit" — and measured at f2d282e it is missing 11 of the 107 top-level flags. Decide and repair by the same 0033 ladder .2 used.`
  Acceptance: `The 11 are named, not counted. The section either gains them, or is replaced by a pointer to USER_GUIDE.md's now-contract-bearing table (0033's R1 repair-by-deletion — a second copy of a set that already has an authoritative home is the anti-pattern, and the book chapter's own prose already documents every knob by config-field name, so nothing is lost). Whichever, the "accurate as of this commit" claim must stop being a promise nothing keeps. Docs-only; mdbook build clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `done` | Audited and registered. The measurement is the work product: **108** clap flags vs **75** table rows, **13** absent entirely, **2** rows naming flags that no longer exist. Passes all three of decision `0033`'s tests ⇒ a genuine silent shadow. |
| 2 | `USER-GUIDE-CLI-TABLE-SHADOW.2` | `done` | Contract decided and written **first**, and it is neither option the leaf offered: **exhaustive over a derived set** — every flag that sets a *knob* (a `Config` field, or a convenience flag setting several), with *mode* flags owned by their own sections. The partition is **total and disjoint**: 93 + 14 = 107 = `anvil --help`. Table repaired 75 → **93** rows. The "2 stale rows" were **not stale** — see the Correction block. |
| 3 | `USER-GUIDE-CLI-TABLE-SHADOW.3` | `pending` | **Next.** The mechanism question, and `.2` has now answered its precondition *and* handed it the trap to avoid: the table is exhaustive over a **derivable** set (`config.rs::Overrides` + 3 aliases), so a check is cheap and meaningful — and it must be **command-scoped**, because `.1`'s two wrong numbers both came from measuring a two-command document against one command's `--help`. |
| 4 | `USER-GUIDE-CLI-TABLE-SHADOW.4` | `pending` | The second site, found by `.2`'s measurement: `book/src/knobs.md` §*CLI coverage* is the same shadow of the same set, **missing 11 of 107**, under a sentence promising it is "accurate as of this commit". Ordered after `.3` because if `.3` writes a check, `.4`'s repair should be the shape that check can hold. |

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
- `2026-08-01` (`.2`): **The contract is "exhaustive over the knobs", not over `anvil --help`.**
  Both framings the leaf offered were wrong. *Exhaustive over `--help`* forces rows for `--seed`,
  `--out`, `--count`, `--trace`, `--help`, `--version` — flags the document already covers in
  dedicated sections, so the table would duplicate them and the duplication would drift.
  *Curated core* leaves inclusion to taste, which is unmaintainable and ungateable. The third
  option is better than both because it is **derived**: a knob is a flag whose value is a
  `Config` field (equivalently: settable in `--config knobs.json`), plus the convenience flags
  that set several at once. `README.md`'s *"every flag"* promise is kept by the **document**, and
  this table keeps the part of it that is a set with an authority.
- `2026-08-01` (`.2`): **The partition was required to be total, not merely non-overlapping.**
  93 knob + 14 mode = 107 = `|anvil --help|`, checked both ways. A criterion that leaves a
  residue is not a contract, it is a preference — and `PARITY-EXTRACTOR-CHARSET-GAP.2` already
  paid for the lesson that an audit which does not account for every item it walks can skip one
  silently.
- `2026-08-01` (`.2`): **No count and no member list in the contract paragraph.** Naming the 14
  mode flags would have created a hand-maintained list mirroring an authoritative set that grows
  — decision `0033`'s three tests, all passing — i.e. the shadow this leaf exists to remove,
  reintroduced inside the sentence that removes it. The paragraph states the *criterion* only.
- `2026-08-01` (`.2`): **`.4` extends this tree rather than opening a new one.** The book's
  §*CLI coverage* is a shadow of the **same** authoritative set (clap's top-level registry). One
  tree owning "shadows of clap's flag registry" is `feedback_full_factorization` — one mechanism,
  never two; two trees over one set would each hold half a picture of the same drift.

## Open Questions

- ~~Is the CLI flag table meant to be exhaustive, or a curated core?~~ **Answered by `.2`:**
  neither as posed — it is exhaustive over the *knobs*, a derived set. See Decisions.
- ~~Were `--divergence` / `--no-minimize` renamed or removed?~~ **Answered by `.2`: neither —
  they were never gone.** They are live `anvil hunt` options, correctly tabled in the `anvil
  hunt` table. `.1` measured them against the wrong command's registry. See the Correction block.
- Should `.3`'s check, if written, also hold the **`anvil hunt`** table? It is 10/10 today and
  the same one-line extractor would cover it — but a check over two tables must not repeat `.1`'s
  error of pooling them, so each table needs its own command-scoped authority. **`.3`'s call.**

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `measured at 1df0071 against ./target/release/anvil --help: 108 flags / 75 table rows / 20 mentioned-not-tabled / 13 absent / 2 stale rows; 0033 three-part test applied and all three hold; matcher corrected mid-audit (backtick-anchored reported 22 absent because \`--artifact dut\` is one code span; word-boundary gives 13); tree registered; no USER_GUIDE.md edit` | `registered` (docs-only; DUT byte-identical) |
| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.2` | `re-measured at f2d282e, command-scoped: anvil --help = 107 OPTIONS (not 108 — --diff-sim is prose inside the hunt subcommand's Commands: description, not an option of anvil); knob set derived from config.rs::Overrides (90 fields) + 3 convenience flags = 93; mode set = 14; 93+14=107 with zero residue and zero overlap (total_or_fail, checked both directions). anvil hunt --help = 11 options; the anvil hunt table = 10/10 (only --help untabled) => the 2 "stale rows" are live flags, correctly placed. Contract paragraph written BEFORE any row. Table repaired 75 -> 93; re-measured after the edit: 0 knob flags missing, 0 non-knob rows. scripts/check_markdown_tables.sh ok (2490 rows / 431 tables / 256 files); scripts/check_doctrines.sh green; cargo test green` | `done` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `USER-GUIDE-CLI-TABLE-SHADOW.1` | `USER-GUIDE-CLI-TABLE-SHADOW.1 — audit + register the CLI-table shadow` | Docs-only; no `USER_GUIDE.md` edit. |
| `USER-GUIDE-CLI-TABLE-SHADOW.2` | `USER-GUIDE-CLI-TABLE-SHADOW.2 — the table is exhaustive over the knobs, and 18 were missing` | Docs-only. Also corrects two of `.1`'s five numbers and registers `.4` (the book's second copy of the same shadow). |

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
- `2026-08-01` (`.2`): Contract decided — **exhaustive over the knobs**, a set derived from
  `config.rs::Overrides` plus three convenience flags, with *mode* flags owned by their own
  sections. Written into `USER_GUIDE.md` above the table before a single row was added, then the
  table repaired **75 → 93**. The partition is total and disjoint (93 + 14 = 107). Two of `.1`'s
  five numbers turned out to be wrong from **one** root cause — the document tables **two** clap
  commands and `.1` measured both against one command's `--help`; `--divergence` and
  `--no-minimize` were never stale, and `S` was 107 not 108. That is the **fifth** instrument
  error in this repo's recent history and the second in this tree, so it is filed as the fact
  card [[cli-flag-audit-must-be-command-scoped]] rather than only as prose. `.4` registered: the
  book's `## CLI coverage` section is the same shadow of the same set, missing **11** of 107.
