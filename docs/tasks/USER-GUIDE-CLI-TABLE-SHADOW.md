# USER-GUIDE-CLI-TABLE-SHADOW: the CLI flag table is a silent shadow of clap's registry, and it has drifted in both directions

## Metadata

- Tree ID: `USER-GUIDE-CLI-TABLE-SHADOW`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-08-01`
- Last updated: `2026-08-01` (`.5` **done** — the `tool_matrix` block deleted by `0033` rung R1
  after its destination was completed and losslessness **proven first**; `.2`'s knob/mode
  criterion settled the contract by *failing* — `tool_matrix` has no knobs; `.6` registered for
  the mechanism question; frontier `.6`)
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
  Children: `.1` (audit + register), `.2` (decide the table's contract, then repair), `.3` (the mechanism question), `.4` (the book's second copy of the same shadow), `.5` (the book's `tool_matrix` block — a different clap registry), `.6` (the mechanism question for that second set)

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
  Status: `done`
  Goal: `Decide whether the gap warrants a mechanism. The set is derivable from clap, so a derived check is CHEAP here in a way it was not for CHANGES-ENTRY-PLACEMENT — but it is only meaningful if .2 rules the table exhaustive; a curated table is authoritative under 0033 test (2) and must NOT be gated.`
  Acceptance: `The decision follows .2's contract, not the other way round. If a check is written it obeys DOCTRINE_ENFORCEMENT.md section 4, is negative-controlled both ways (delete a row => fails; add a flag without a row => fails), and states its extractor precisely enough to re-run — the matcher correction recorded in .1 is exactly the trap it must avoid.`
  Verification: `done — A MECHANISM IS WARRANTED, AND IT IS PAIR 5 OF check_enumeration_parity.sh, NOT AN ELEVENTH DOCTRINE. The feedback_full_factorization test was applied rather than waved through, and it points the opposite way from TABLE-RENDER-FIDELITY's: that one registered separately because NO doctrine owned markdown well-formedness, whereas ENUMERATION-PARITY already owns "a hand-maintained docs list mirroring an authoritative set" and this is one. 0033 rule (a), test by test: derivable (cli_overrides in src/main.rs enumerates the flags that set a knob, and clap derives each flag spelling from its field name); growth-coupled (.2's recorded contract makes the table exhaustive over the knobs, so every new knob REQUIRES a row); silent (13 flags were absent at 1df0071 with every gate green). THE AUTHORITATIVE SET COMES FROM SOURCE, NEVER FROM anvil --help, for two reasons and the first is decisive: a doctrine check reads the REPOSITORY (DOCTRINE_ENFORCEMENT.md section 4(4)) and --help needs a built binary, which a fresh clone and a pre-build CI step do not have — the gate would be skipped exactly where it is the backstop; and --help is prose that quotes OTHER commands flags, which is what made .1 count 108 where 107 exist. THE WHITESPACE STRIP IS LOAD-BEARING AND IT WAS MEASURED, NOT ASSUMED: PARITY-EXTRACTOR-ARM-SHAPE-GAP is ALREADY LIVE inside cli_overrides — rustfmt split `cli` from `.hierarchy_registered_sibling_mixed_support_prob` across two lines (95 chars of field name), so a naive line-wise scan reads 91 of 92 and the 92nd is INVISIBLE rather than wrong. Stripping all whitespace first removes the hazard at its root instead of widening a regex around one wrap. FOUR NEGATIVE CONTROLS, ALL FIRED, each restored from an ON-VOLUME backup and each verified against git afterwards: (1) delete the --operand-duplication-rate row => FAIL naming exactly that flag; (2) add a knob flag with no row (cli.probe_control_knob into the Overrides literal) => FAIL naming --probe-control-knob, i.e. the growth direction, which is the one that actually happens; (3) remove the fence => hard FAIL, not a silent skip, so the repair cannot degrade back to whole-file matching; (4) neuter the tr -d [:space:] => the extractor drops 92 -> 91 and the TIGHT FLOOR CATCHES IT with a message pointing at the extractor rather than at the docs — which is the evidence for setting the floor at the exact measured count. HONEST LIMIT STATED, NOT PAPERED OVER: control (4) is the SHRINK case; had the wrap-hidden flag been newly added the floor would be satisfied and nothing would fire, exactly as the file already records about floors being shrink-coupled. SUBSTRING VACUITY MEASURED AND ABSENT: no derived flag is a proper substring of another, so covers_fenced_set grep cannot pass a deleted row on another row content — checked rather than assumed, since that is precisely how decision 0037 found 3 of 10 sites vacuous. ONE SITE, DELIBERATELY: book/src/knobs.md CLI coverage is a second copy of this set and is .4 work, to be repaired by 0033 rung R1 (delete the copy, point at the reference) — gating it first would FREEZE the copy in place, the outcome R1 exists to avoid. THE MODE HALF IS DELIBERATELY UNGATED: those flags are authoritative under test (2), and writing a list of them so a gate could check it would create the shadow this doctrine removes. Checks: scripts/check_doctrines.sh all 10 hold; cargo check/clippy/fmt/test green; src/main.rs byte-identical to HEAD after the controls; docs+script only => DUT byte-identical.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.3`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.4`
  Status: `done`
  Goal: `The SECOND site: book/src/knobs.md's "## CLI coverage" section is the same shadow of the same set. It says in its own words "The canonical list comes from anvil --help; the snapshot below is accurate as of this commit" — and measured at f2d282e it is missing 11 of the 107 top-level flags. Decide and repair by the same 0033 ladder .2 used.`
  Acceptance: `The 11 are named, not counted. The section either gains them, or is replaced by a pointer to USER_GUIDE.md's now-contract-bearing table (0033's R1 repair-by-deletion — a second copy of a set that already has an authoritative home is the anti-pattern, and the book chapter's own prose already documents every knob by config-field name, so nothing is lost). Whichever, the "accurate as of this commit" claim must stop being a promise nothing keeps. Docs-only; mdbook build clean.`
  Verification: `done — REPAIRED BY R1 (delete the copy, point at the reference), and the case for R1 turned out to be far stronger than "the copy is behind". THE REGISTERED COUNT OF 11 WAS WRONG; IT IS 14, and the correction is the SIXTH instrument note in this repo's recent history. Measured at 5ce2dd3 with a COMMAND-SCOPED, SECTION-SCOPED instrument, the 14 absent flags NAMED not counted: --artifact, --case-mux-if-emit-prob, --casez-mux-if-emit-prob, --fsm-mealy-prob, --hierarchy-registered-sibling-mixed-support-prob, --introspect, --lane-n-children, --lane-n-params, --multi-output-task-emit-prob, --mux-if-emit-prob, --priority-case-prob, --steer, --sv-version, --unique-case-prob. WHY .2 GOT 11 — REPRODUCED RATHER THAN WAVED AWAY: 11 comes out only if the scope is the WHOLE "## CLI coverage" section (so the "not yet exposed" prose and the tool_matrix block, neither of which is the snapshot, both count as coverage) AND S is 105 rather than 107 (dropping --lane-n-children/--lane-n-params). book/src/knobs.md is byte-identical between f2d282e and 5ce2dd3, so the file did not move; the instrument did. THE AUTHORITATIVE SET WAS DERIVED TWICE AND THE TWO AGREE EXACTLY, which is the control: (a) the Cli struct in src/main.rs — 105 declared fields minus `command` plus clap built-ins --help/--version; (b) the Options: block of anvil --help, command-scoped so the Commands: prose cannot contribute. diff = empty, 107 both ways. THE FAR WORSE DEFECT WAS FOUND WHILE MEASURING, AND IT IS NOT AN OMISSION: the "### Not yet exposed via CLI (reachable via --config FILE)" subsection listed 12 knobs, and 9 OF THE 12 HAVE CLI FLAGS. That is .1s own severity judgement about stale rows — a reader who believes it reaches for --config to set something a flag already sets. EIGHT OF THE NINE WERE CONTRADICTED INSIDE THE SAME SECTION (they are in its own "### Presets and capability knobs" block, ~60 lines earlier) and all nine by line 1128 of the same chapter, which states the truth. THE TRUTH WAS DERIVED, NOT TRUSTED: --dump-config's serde projection of Config::default() walked TOTALLY (93 keys at all levels; 1 dict-valued, child_instances_per_module_by_depth, which IS CLI-settable as the renamed --child-instances-per-depth) leaves exactly THREE fields with no flag — library_prob, use_async_reset, max_nodes_per_module — matching the independently-authored sentence at line 1128 member for member. DELETION PROVEN LOSSLESS BEFORE IT WAS MADE, not asserted after: all 107 flags remain documented across USER_GUIDE.md + book/src/*.md (comm against the derived set: 0 undocumented); the 3 true bullets were strict SUBSETS of richer entries already at knobs.md:120 (max_nodes_per_module), :198 (use_async_reset) and :1109 (library_prob); the 9 long capability-knob paragraphs were RETAINED, not deleted, because they are prose and not a shadow (0033 test (2) fails — a new capability knob does not REQUIRE a bullet for correctness) — only their false heading changed. A TOTAL SWEEP FOR OTHER SITES, count recorded per 0039: 30 book/src/*.md files walked, distinct top-level flags named per file — knobs.md 101 is the lone outlier; recipes.md 44 / structured-emission.md 19 / tutorial.md 17 are usage commands, not enumerations. ONE SITE, CONFIRMED. A THIRD SHADOW WAS MEASURED AND DELIBERATELY NOT REPAIRED HERE: the "### tool_matrix auxiliary binary" block names 22 of tool_matrix --help's 37 options. It shadows a DIFFERENT clap registry (src/bin/tool_matrix.rs), so it needs its own extractor and is registered as .5 rather than folded in — one leaf per commit, and .3 already recorded that a command without an Overrides projection does not transfer the one-line extractor. It carries a dated measured-incomplete note in the meantime rather than silence. The inbound link at book/src/architecture.md:917 promised "the full categorised list lives in" the deleted section and was repointed at USER_GUIDE.md; the #cli-coverage anchor is preserved so the link does not break. Checks: scripts/check_doctrines.sh — all 10 registered doctrines hold (ENUMERATION-PARITY incl. the SUMMARY.md<->chapters pair, TABLE-RENDER-FIDELITY 2,510 rows / 435 tables / 257 files); mdbook build exit 0; cargo check --all-targets / clippy -D warnings / fmt --check all exit 0; cargo test under ram_guard exit 0 — 1,087 passed / 0 failed / 19 ignored across 17 targets incl. tests/snapshots.rs, run UNPIPED; docs-only => DUT byte-identical, tests/snapshots.rs untouched.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.4`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.5`
  Status: `done`
  Goal: `The THIRD site, measured by .4: book/src/knobs.md's "### tool_matrix auxiliary binary" block names 22 of tool_matrix --help's 37 options. It is the same DEFECT CLASS as .2/.4 but a DIFFERENT authoritative set — src/bin/tool_matrix.rs's clap Cli, not src/main.rs's — so it needs its own extractor and its own contract decision.`
  Acceptance: `The 15 absent options are NAMED, not counted, and the instrument is stated precisely enough to re-run (command-scoped: tool_matrix --help's Options: block, never anvil's). Decide the block's contract FIRST, as .2 did — exhaustive over tool_matrix's flags, exhaustive over its GATE flags only, or deleted in favour of USER_GUIDE.md's "Tool matrix sweeps" section (0033 rung R1) — then repair to match. If a gate is warranted it is an ENUMERATION-PARITY pair, not a doctrine (feedback_full_factorization), derived from SOURCE not from a built binary, and negative-controlled both ways. The dated measured-incomplete note .4 left must be removed by whatever lands, not left beside a repaired list. Docs-only unless the pair needs a script change; mdbook build clean.`
  Verification: `done — REPAIRED BY R1, AND THE CONTRACT DECISION IS THE PART THAT TRANSFERS: .2's criterion does not merely fail to apply here, it settles the question by failing. .2 partitioned anvil's flags into KNOB (a Config field, equivalently settable in --config knobs.json) and MODE (its own section, not tabled). tool_matrix HAS NO Overrides PROJECTION AND NO KNOBS — not one of its 37 options sets a Config field — so under .2's own partition every one of them is a mode flag and a chapter about knobs owes ZERO rows. The block was a flag list, for a binary with no knobs, inside the knob chapter, one screen below the anvil flag list .4 deleted the same day. THE AUTHORITATIVE SET WAS DERIVED TWICE AND THE TWO AGREE EXACTLY, which is the control the standing rule requires: (a) the Cli struct in src/bin/tool_matrix.rs — 35 #[arg(long)] fields, whitespace-stripped so no rustfmt wrap can hide one — plus clap's --help/--version, which exist because the binary sets #[command(version)]; (b) the Options: block of tool_matrix --help, command-scoped so the harness's own prose cannot contribute. diff = empty, 37 both ways. THE REGISTERED "22 of 37" IS SCOPE-DEPENDENT AND THE SNAPSHOT ITSELF NAMED 21 — the SEVENTH instrument note in this repo's recent history, recorded rather than waved through. The ```text fence names 21; the whole subsection names 22. The extra is --diff-sim, which appears ONLY in the closing prose as a cross-reference ("It does not run a testbench; use --diff-sim for cross-simulator trace agreement"), never as an entry in the list. So 22 reproduces only if a cross-reference counts as a list member. Unlike .2 -> .4 (11 vs 14) neither number is wrong; they measure different regions, and the leaf publishes both with their scopes rather than picking the one that flatters. R1 WAS NOT LOSSLESS AS FOUND, AND THAT WAS MEASURED BEFORE A WORD WAS WRITTEN. .4 could delete outright because 0 of 107 flags lost a home. Here FOUR declared options — --base-seed, --verilator-bin, --yosys-bin, --iverilog-bin — were documented NOWHERE in the live docs except this snapshot, so a straight delete would have destroyed the only documentation of four options while claiming to repair a documentation defect. The repair order is therefore: complete the destination, PROVE losslessness, then cut. THE DESTINATION GOT THE CONTRACT, and it is derived rather than chosen: USER_GUIDE.md §Tool matrix sweeps gains a "### Options" heading and a paragraph recording that the list is EXHAUSTIVE OVER THE OPTIONS tool_matrix DECLARES — every field of the clap Cli — with clap's two built-ins outside the set because the framework supplies them rather than the binary declaring them. That exclusion is DERIVABLE (a built-in is exactly an option with no Cli field), not the "honest caveat" .1 had to make for --version, so the awkward member stops being awkward for a second time. The partition is total and disjoint: 35 declared + 2 built-in = 37 = |tool_matrix --help|. REPAIR: the options region went 32 -> 35 declared options named, +7 entries (--out, --base-seed, --resume, --verilator-bin, --yosys-bin, --iverilog-bin, --divergence), each placed beside its topical neighbour rather than appended. Re-measured after the edit: 0 declared options missing. --divergence WAS THE .1 TRAP IN REVERSE and is worth the sentence: it is not absent from USER_GUIDE.md — it is present under anvil hunt, a DIFFERENT command's namespace — so a document-wide grep calls it documented while the matrix's own column has no entry in the reference. .1 measured two commands against one registry; this is one spelling read across two registries. The contract paragraph states the namespace boundary explicitly (--divergence, --out) so the next author cannot repeat either direction. DELETION PROVEN LOSSLESS BEFORE IT WAS MADE, then re-proven after: all 35 declared options are named across USER_GUIDE.md + book/src/*.md + TOOLBOX.md (comm against the derived set: 0 without a home, both pre- and post-cut). The two built-ins are excluded on principle and said out loud rather than dropped silently. A FINDING FOR .6, MEASURED NOT ASSUMED: the options region also names --ast-json, --binary and --language, which are slang's and verilator's flags quoted in prose. A pair over this region must therefore be ONE-DIRECTIONAL coverage (covers_fenced_set), never exact parity — exact parity would cry wolf on three tokens that are correct prose, and DOCTRINE_ENFORCEMENT.md §9 records that a gate which cries wolf gets deleted, taking its real coverage with it. .6 REGISTERED for the mechanism question, mirroring .2 -> .3 exactly: after R1 there is no copy left to gate, so the question is no longer "gate the copy" (which .3 refused because gating freezes) but "gate the canonical home", which is precisely what .3 did for the anvil table. Checks: scripts/check_doctrines.sh — all 10 registered doctrines hold (ENUMERATION-PARITY incl. all 5 pairs, TABLE-RENDER-FIDELITY 2,526 rows / 442 tables / 259 files, README-GROWTH 161 lines / 10,653 B); mdbook build exit 0; cargo check --all-targets / clippy -D warnings / fmt --check all exit 0; cargo test under ram_guard exit 0 — 1,087 passed / 0 failed / 19 ignored across 17 targets incl. tests/snapshots.rs, run UNPIPED; docs-only => DUT byte-identical, tests/snapshots.rs untouched.`
  Commit: `USER-GUIDE-CLI-TABLE-SHADOW.5`

- ID: `USER-GUIDE-CLI-TABLE-SHADOW.6`
  Status: `pending`
  Goal: `The mechanism question for the tool_matrix option set, standing to .5 exactly as .3 stood to .2. .5 recorded a contract making USER_GUIDE.md §Tool matrix sweeps -> Options exhaustive over the options src/bin/tool_matrix.rs declares, so the list is now derivable AND growth-coupled AND silent — decision 0033's three tests, all passing. Decide whether it becomes ENUMERATION-PARITY pair 6.`
  Acceptance: `The decision follows .5's contract, not the other way round. If a pair is written it obeys DOCTRINE_ENFORCEMENT.md section 4, derives the set from SOURCE (the Cli struct in src/bin/tool_matrix.rs) and never from a built binary — .3's reason applies unchanged: a doctrine check reads the repository, and a fresh clone has no binary. It is whitespace-stripped before matching (PARITY-EXTRACTOR-ARM-SHAPE-GAP), charset-captured to what the SOURCE permits rather than to what today's members happen to use (PARITY-EXTRACTOR-CHARSET-GAP), total_or_fail'd against a looser candidate scan, count-floored, and negative-controlled BOTH ways (delete an entry => fails naming it; add a Cli field with no entry => fails naming it). It must be ONE-DIRECTIONAL coverage: .5 measured three foreign tokens inside the region (--ast-json, --binary, --language — other tools' flags in prose), so exact parity would cry wolf. The clap built-ins stay OUTSIDE the checked set, matching the contract. A substring-vacuity probe is required before the pair is trusted, as .3 ran: --sv2v is a proper substring of --sv2v-bin and --slang of --slang-bin, so unlike .3's set this one is NOT substring-free and a naive covers_ grep can pass a deleted entry on another entry's content. Docs + one enforcement script at most; DUT byte-identical.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `done` | Audited and registered. The measurement is the work product: **108** clap flags vs **75** table rows, **13** absent entirely, **2** rows naming flags that no longer exist. Passes all three of decision `0033`'s tests ⇒ a genuine silent shadow. |
| 2 | `USER-GUIDE-CLI-TABLE-SHADOW.2` | `done` | Contract decided and written **first**, and it is neither option the leaf offered: **exhaustive over a derived set** — every flag that sets a *knob* (a `Config` field, or a convenience flag setting several), with *mode* flags owned by their own sections. The partition is **total and disjoint**: 93 + 14 = 107 = `anvil --help`. Table repaired 75 → **93** rows. The "2 stale rows" were **not stale** — see the Correction block. |
| 3 | `USER-GUIDE-CLI-TABLE-SHADOW.3` | `done` | A mechanism **is** warranted, and it is **pair 5 of `check_enumeration_parity.sh`** — not an eleventh doctrine, because `ENUMERATION-PARITY` already owns this subject (`feedback_full_factorization`). The set is derived from `cli_overrides` in **source**, never from `anvil --help`. **Four negative controls fired**, including one that caught the `rustfmt`-wrap defect **live** in `cli_overrides` before it shipped. |
| 4 | `USER-GUIDE-CLI-TABLE-SHADOW.4` | `done` | The copy is **deleted** (`0033` rung R1) and the section now points at the gated table. The registered **11** was **14** — an instrument difference, not a file change. And the copy was not merely behind: **9 of the 12** knobs in its *"not yet exposed via CLI"* list **have CLI flags**, 8 of them contradicted inside the same section. Deletion proven lossless first: **0** of the 107 flags lost documentation. |
| 5 | `USER-GUIDE-CLI-TABLE-SHADOW.5` | `done` | The copy is **deleted** (`0033` rung R1) and the destination now carries the contract. `.2`'s knob/mode criterion **settled it by failing**: `tool_matrix` has no `Overrides` projection and no knobs, so under that partition all 37 options are *mode* flags and a knob chapter owes **zero** rows. Unlike `.4`, R1 was **not lossless as found** — four options were documented nowhere else — so the destination was completed and losslessness **proven first**. The registered **22** is section-scope; the snapshot itself named **21**. |
| 6 | `USER-GUIDE-CLI-TABLE-SHADOW.6` | `pending` | **Next.** The mechanism question, standing to `.5` as `.3` stood to `.2`. `.5`'s recorded contract makes the destination derivable ∧ growth-coupled ∧ silent, so `0033`'s three tests all pass. Two constraints `.3` did not face, both **measured** by `.5`: the region contains three foreign tokens (other tools' flags in prose) ⇒ **coverage, not exact parity**; and the set is **not substring-free** (`--slang` ⊂ `--slang-bin`, `--sv2v` ⊂ `--sv2v-bin`) ⇒ a naive `covers_` grep can pass a deleted entry. |

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

- `2026-08-01` (`.3`): **A pair, not a doctrine.** `feedback_full_factorization` forbids a second
  mechanism for a job that already has one. `ENUMERATION-PARITY` owns "a hand-maintained docs list
  mirroring an authoritative set"; this is one. The test was *applied*, not waved through — and it
  points the opposite way from `TABLE-RENDER-FIDELITY`'s, which registered separately precisely
  because no doctrine owned markdown well-formedness.
- `2026-08-01` (`.3`): **The set is read from source, not from `anvil --help`.** A doctrine check
  reads the repository (`DOCTRINE_ENFORCEMENT.md` §4(4)); `--help` needs a built binary a fresh
  clone and a pre-build CI step do not have, so the gate would be absent exactly where it is the
  backstop. And `--help` is prose that quotes other commands' flags — the thing that produced
  `.1`'s two wrong numbers.
- `2026-08-01` (`.3`): **`book/src/knobs.md` §*CLI coverage* was deliberately NOT declared as a
  second gated site.** Gating a copy freezes it in place; decision `0033`'s preferred rung is R1 —
  delete the copy, point at the reference. That is `.4`'s decision to make, and pre-gating it
  would have made it for them.
- `2026-08-01` (`.3`): **The mode half of the partition stays ungated.** Those flags are
  authoritative under `0033` test (2): no list of them exists, and writing one so a gate could
  check it would create the shadow this doctrine removes.

- `2026-08-01` (`.4`): **R1, and the deciding evidence was not the omissions.** A copy that is
  merely *behind* argues for R1 only on principle. This one had gone **false**: 9 of the 12
  knobs it called *"not yet exposed via CLI"* have CLI flags, and 8 of those 9 were listed as
  CLI flags **in the same section**. A refreshed copy would drift back to behind; a *gated* copy
  freezes (`.3`'s reason for leaving it). Deletion is the only rung that removes the failure
  mode instead of relocating it.
- `2026-08-01` (`.4`): **Lossless was proven before the deletion, not asserted after.** Every
  one of the 107 flags was checked to survive in `USER_GUIDE.md` + `book/src/*.md` (0 lost), and
  the 3 truthful bullets were shown to be strict *subsets* of richer entries already in the same
  chapter. A repair that cannot show what it kept is a deletion with a rationale attached.
- `2026-08-01` (`.4`): **The 9 capability-knob paragraphs were RETAINED, only their heading was
  false.** They fail `0033` test (2) — a new capability knob does not *require* a bullet for the
  chapter to be correct — so they are prose, not a shadow, and `feedback_never_retire_strategies`
  applies. The repair is scoped to the shadow and the false claim; it does not use the occasion
  to prune.
- `2026-08-01` (`.4`): **The three config-file-only knobs are named ONCE and not re-listed.**
  That set *is* growth-coupled and derivable (`Config` fields minus `cli_overrides`), so writing
  it a second time under the new heading would have created a fresh shadow inside the repair —
  the mistake `.2` avoided by refusing to enumerate the 14 mode flags.
- `2026-08-01` (`.4`): **`.5` is a new leaf, not an extension of `.4`.** The `tool_matrix` block
  is the same defect *class* over a **different** authoritative set. `.2` put the book's `anvil`
  snapshot in this tree because it shadowed the *same* registry; that reasoning does not reach a
  second binary's registry, and `COMMIT.md`'s one-leaf-per-commit rule forbids bundling it.

- `2026-08-01` (`.5`): **`.2`'s criterion decided this leaf by failing to apply.** The obvious
  move was to pick one of the three options the leaf offered. Instead the first question asked
  was the one `.2` answered for `anvil`: *which half of the knob/mode partition are these?*
  `tool_matrix` has **no `Overrides` projection and no knobs** — not one of its options sets a
  `Config` field — so every one of them is a *mode* flag, and a chapter about knobs owes **zero**
  rows. That is a derivation, not a preference, and it is why the answer is R1 rather than a
  refreshed list: the block was not merely a stale copy, it was a flag list for a binary with no
  knobs inside the knob chapter.
- `2026-08-01` (`.5`): **Losslessness was measured BEFORE the contract was written, and it
  changed the plan.** `.4` could cut immediately because 0 of 107 flags lost a home. Here four
  declared options had no documented home anywhere in the live docs outside the snapshot, so the
  same rung applied the same way would have destroyed the only documentation of four options
  while claiming to repair a documentation defect. The repair is ordered — complete the
  destination, prove losslessness, then cut — and the proof was re-run after the cut.
- `2026-08-01` (`.5`): **The built-in exclusion is derived, not caveated.** `.1` had to publish
  `--version` as an "honest caveat" that made its actionable set 12 rather than 13. Here the rule
  is mechanical: a clap built-in is exactly an option with **no `Cli` field**, so "the options
  `tool_matrix` declares" separates them without judgement. `35 + 2 = 37`, total and disjoint.
- `2026-08-01` (`.5`): **No count and no member list in the contract paragraph** — `.2`'s rule,
  applied unchanged. The paragraph states the criterion; naming the two built-ins, or writing
  "35 options", would be one more copy of a list inside the sentence that removes one.
- `2026-08-01` (`.5`): **`--divergence` is `.1`'s trap read backwards, and the namespace boundary
  is now written down.** `.1` measured two commands against one registry. This is one *spelling*
  living in two registries: `--divergence` is documented in `USER_GUIDE.md` under `anvil hunt`,
  so a document-wide grep reports it documented while the matrix's own column has no entry in
  the reference. A file-scoped instrument cannot see that; only a command-scoped one can.
- `2026-08-01` (`.5`): **`.6` is a new leaf, not part of this one.** `.2` decided the contract and
  repaired; `.3` decided the mechanism. This tree's own cadence, and `COMMIT.md`'s
  one-leaf-per-commit rule, put the gate in its own leaf. The question also *changed shape* at
  R1: `.3` refused to gate the book copy because gating freezes a copy in place, and after the
  cut there is no copy — so `.6` asks whether the **canonical home** is gated, which is exactly
  what `.3` decided for the `anvil` table.

## Open Questions

- ~~Is the CLI flag table meant to be exhaustive, or a curated core?~~ **Answered by `.2`:**
  neither as posed — it is exhaustive over the *knobs*, a derived set. See Decisions.
- ~~Were `--divergence` / `--no-minimize` renamed or removed?~~ **Answered by `.2`: neither —
  they were never gone.** They are live `anvil hunt` options, correctly tabled in the `anvil
  hunt` table. `.1` measured them against the wrong command's registry. See the Correction block.
- ~~Is the book's §*CLI coverage* missing 11?~~ **Answered by `.4`: no — 14, and the number was
  never the point.** The registered 11 is reproducible only under a scope that counts the
  *"not yet exposed"* prose and the `tool_matrix` block as coverage, against a 105-flag `S`.
  `book/src/knobs.md` is byte-identical between `f2d282e` and `5ce2dd3`, so the instrument moved,
  not the file. The section's **false** half — 9 of 12 — mattered more than either count.
- ~~Should the **`tool_matrix`** block be deleted (R1) or gated as a sixth `ENUMERATION-PARITY`
  pair?~~ **Answered by `.5`: deleted — and the two were never alternatives.** Gating the *copy*
  is the option `.3` had already refused on principle (a gated copy freezes in place). The real
  fork was *where the set lives*, and `.2`'s knob/mode criterion settled it: `tool_matrix` has no
  knobs, so the knob chapter owes zero rows. Whether the **canonical home** is then gated is a
  separate question with a different answer available, and it is `.6`'s.
- Was the registered **22 of 37** right? **Both numbers are, at different scopes, and `.5`
  publishes both.** The `text` fence named **21**; the whole subsection named **22**. The extra
  is `--diff-sim`, which the closing prose mentions as a *cross-reference* rather than listing.
  Unlike `.2` → `.4`'s 11-vs-14 this is not a correction — neither reading is wrong — but it is
  the **seventh** instrument note here, and the rule it repeats is the tree's oldest: an
  enumeration audit must state the region it measured, because "the block" is not a scope.
- Should `.3`'s check also hold the **`anvil hunt`** table? **`.3` says not yet, and states why:**
  `HuntCommand` has no `Overrides` projection to derive from — its flags map to a `HuntRequest`
  built by `build_hunt_request`, which mixes CLI fields with defaults and derived values, so the
  same one-line extractor does not transfer. The table is 10/10 today and the namespace is small
  and stable. Left open rather than closed: the honest state is *unmechanized*, not *fine*.

## Blockers

- None.

## Surfaced by `.4`, owned elsewhere

- **`mdbook` rewrites `.md` links to `.html`, so a link from the book to a repo-root file renders
  dead** — and this repair nearly shipped one. `[USER_GUIDE.md](../../USER_GUIDE.md#knobs)` (the
  form `book/src/recipes.md:857` already uses) becomes `../../USER_GUIDE.html`, which does not
  exist. It resolves correctly when the Markdown *source* is read on GitHub, and is dead in the
  **rendered book** — the surface the owner reviews. `mdbook build` exits `0` on it. Both of
  `.4`'s pointers were switched to the plain-backtick form; the book went **6 dead local links →
  2**, the remaining two being the pre-existing `recipes.md` link and its `print.html` copy.
  **Not repaired here** — `.4` does not own `recipes.md`, and no tree or doctrine owns *book link
  integrity* (`TABLE-RENDER-FIDELITY` owns table well-formedness; nothing owns links). Registered
  as its own tree, `BOOK-LINK-INTEGRITY`, in the commit following `.4` — the repo must be
  handoff-ready before a pivot, and one leaf lands per commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.1` | `measured at 1df0071 against ./target/release/anvil --help: 108 flags / 75 table rows / 20 mentioned-not-tabled / 13 absent / 2 stale rows; 0033 three-part test applied and all three hold; matcher corrected mid-audit (backtick-anchored reported 22 absent because \`--artifact dut\` is one code span; word-boundary gives 13); tree registered; no USER_GUIDE.md edit` | `registered` (docs-only; DUT byte-identical) |
| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.2` | `re-measured at f2d282e, command-scoped: anvil --help = 107 OPTIONS (not 108 — --diff-sim is prose inside the hunt subcommand's Commands: description, not an option of anvil); knob set derived from config.rs::Overrides (90 fields) + 3 convenience flags = 93; mode set = 14; 93+14=107 with zero residue and zero overlap (total_or_fail, checked both directions). anvil hunt --help = 11 options; the anvil hunt table = 10/10 (only --help untabled) => the 2 "stale rows" are live flags, correctly placed. Contract paragraph written BEFORE any row. Table repaired 75 -> 93; re-measured after the edit: 0 knob flags missing, 0 non-knob rows. scripts/check_markdown_tables.sh ok (2490 rows / 431 tables / 256 files); scripts/check_doctrines.sh green; cargo test green` | `done` (docs-only; DUT byte-identical) |

| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.3` | `pair 5 added to scripts/check_enumeration_parity.sh: extract_cli_knob_flags reads the cli.<field> references in cli_overrides (src/main.rs), whitespace-stripped, kebab-cased => 92 knob flags; total_or_fail (95 loose cli. occurrences vs 95 extracted); floor 92; covers_fenced_set over USER_GUIDE.md's new <!--enum:knob-flags--> fence. FOUR NEGATIVE CONTROLS, all fired: delete a row => FAIL naming --operand-duplication-rate; add cli.probe_control_knob => FAIL naming --probe-control-knob; remove the fence => hard FAIL; neuter the whitespace strip => extractor 92 -> 91, floor catches it and blames the extractor not the docs. Substring-vacuity probe: no derived flag is a proper substring of another. src/main.rs verified byte-identical to HEAD after the controls; scripts/check_doctrines.sh 10/10; cargo check/clippy/fmt/test green` | `done` (docs + enforcement script; DUT byte-identical) |

| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.4` | `S derived TWICE and identical: the Cli struct in src/main.rs (105 fields - `command` + clap --help/--version) and the Options: block of anvil --help, command-scoped => 107 both ways, empty diff. Book section (command- AND section-scoped, i.e. excluding the tool_matrix block and the "not yet exposed" prose, neither of which is the snapshot) = 93 => 14 absent, NAMED in the leaf. The registered 11 reproduced only at whole-section scope against a 105-flag S; book/src/knobs.md byte-identical f2d282e..5ce2dd3, so the instrument moved not the file. SEPARATE AND WORSE: 9 of the 12 "not yet exposed via CLI" bullets name knobs that HAVE flags, 8 contradicted inside the same section; the true set is 3 (library_prob, use_async_reset, max_nodes_per_module), derived by walking --dump-config TOTALLY (93 keys, 1 dict-valued and CLI-settable under a renamed flag) and matching line 1128 member for member. LOSSLESS PROVEN PRE-DELETION: 0 of 107 flags lost documentation across USER_GUIDE.md + book/src/*.md; the 3 truthful bullets are subsets of knobs.md:120 / :198 / :1109. Sweep for other sites, count recorded per 0039: 30 book files walked, knobs.md 101 the lone outlier. tool_matrix block measured 22 of 37 => registered as .5, note left. scripts/check_doctrines.sh 10/10 (TABLE-RENDER-FIDELITY 2,510 rows / 435 tables / 257 files); mdbook build exit 0; cargo test green, UNPIPED` | `done` (book-only; DUT byte-identical) |

| `2026-08-01` | `USER-GUIDE-CLI-TABLE-SHADOW.5` | `S derived TWICE and identical: the Cli struct in src/bin/tool_matrix.rs (35 #[arg(long)] fields, whitespace-stripped) + clap --help/--version, and the Options: block of tool_matrix --help, command-scoped => 37 both ways, empty diff. Book block, all numbers on the 37-option scope: fence-scoped 21 named / 16 absent; section-scoped 22 / 15, the extra being --diff-sim as a prose cross-reference. On the 35-DECLARED scope the fence named 19. Every number is published with the region AND the denominator it was taken against, because mixing a 37-scope numerator with a 35-scope denominator is this leaf's own defect class and the first draft of the book prose did exactly that before it was caught. CONTRACT DECIDED FIRST and derived, not chosen: .2's knob/mode partition settles it by FAILING — tool_matrix has no Overrides projection and no knobs, so all 37 are mode flags and the knob chapter owes zero rows => R1. LOSSLESSNESS MEASURED BEFORE ANY EDIT and it changed the plan: 4 declared options (--base-seed, --verilator-bin, --yosys-bin, --iverilog-bin) had NO documented home outside the snapshot, unlike .4's 0 of 107. Destination completed first: USER_GUIDE.md §Tool matrix sweeps gains "### Options" + a contract paragraph (exhaustive over the options tool_matrix DECLARES; clap built-ins outside the set, a derivable exclusion; 35 + 2 = 37 total and disjoint) and 7 entries (--out, --base-seed, --resume, --verilator-bin, --yosys-bin, --iverilog-bin, --divergence); re-measured after: 32 -> 35 declared named, 0 missing. Post-cut re-proof over USER_GUIDE.md + book/src/*.md + TOOLBOX.md: 0 of 35 lost a home. --divergence recorded as .1's trap in reverse (one spelling, two registries — documented under anvil hunt, absent as the matrix column). FOR .6, MEASURED: 3 foreign tokens inside the region (--ast-json, --binary, --language) => coverage not exact parity; and the set is NOT substring-free (--slang ⊂ --slang-bin, --sv2v ⊂ --sv2v-bin) => a naive covers_ grep can pass a deleted entry, which .3's set could not. scripts/check_doctrines.sh 10/10 (TABLE-RENDER-FIDELITY 2,528 rows / 442 tables / 259 files; MEMORY-ARCH 30 lines / 6,099 B against caps 50 / 6,144 — and the BYTE cap FIRED on the first MEMORY.md draft at 6,377 B, repaired by routing the next_action detail down into .6's Acceptance rather than trimming prose, which is the routing the failure message asks for); mdbook build exit 0; cargo check --all-targets / clippy -D warnings / fmt --check all exit 0; cargo test under ram_guard exit 0 — 1,087 passed / 0 failed / 19 ignored across 17 targets, UNPIPED` | `done` (docs-only; DUT byte-identical) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `USER-GUIDE-CLI-TABLE-SHADOW.1` | `USER-GUIDE-CLI-TABLE-SHADOW.1 — audit + register the CLI-table shadow` | Docs-only; no `USER_GUIDE.md` edit. |
| `USER-GUIDE-CLI-TABLE-SHADOW.2` | `USER-GUIDE-CLI-TABLE-SHADOW.2 — the table is exhaustive over the knobs, and 18 were missing` | Docs-only. Also corrects two of `.1`'s five numbers and registers `.4` (the book's second copy of the same shadow). Landed `cd7f24b` + `c793cf3` (hash backfill). |
| `USER-GUIDE-CLI-TABLE-SHADOW.3` | `USER-GUIDE-CLI-TABLE-SHADOW.3 — gate the table as ENUMERATION-PARITY pair 5` | Docs + one enforcement script. No `src/` change (the control's edit to `src/main.rs` was reverted and verified byte-identical to HEAD). |
| `USER-GUIDE-CLI-TABLE-SHADOW.4` | `USER-GUIDE-CLI-TABLE-SHADOW.4 — delete the book's copy of the flag list` | Landed `ccfbc23`. Book + docs + one script comment; no `src/` change. Registers `.5` (the `tool_matrix` block, 22 of 37) and surfaces the mdBook dead-link class. |
| `USER-GUIDE-CLI-TABLE-SHADOW.5` | `USER-GUIDE-CLI-TABLE-SHADOW.5 — give tool_matrix's options one home` | Book + `USER_GUIDE.md` + docs; no `src/` change. Registers `.6` (the mechanism question) with two constraints `.3` did not face, both measured here. |

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
- `2026-08-01` (`.3`): The table is now **mechanically gated** — `ENUMERATION-PARITY` **pair 5**,
  derived from `cli_overrides` in source rather than from `anvil --help`, over a new
  `<!--enum:knob-flags-->` fence in `USER_GUIDE.md`. Four negative controls fired, and one of them
  found the `PARITY-EXTRACTOR-ARM-SHAPE-GAP` defect **live** in `cli_overrides`: `rustfmt` had
  split `cli` from `.hierarchy_registered_sibling_mixed_support_prob`, so a line-wise extractor
  reads **91 of 92** and the 92nd is invisible. The whitespace strip removes the hazard at its
  root; the control is the two numbers.
- `2026-08-01` (`.4`): The book's copy is **deleted** by `0033` rung **R1**, and §*CLI coverage*
  now points at the gated `USER_GUIDE.md` table. Two corrections to the registered picture, both
  found by measuring rather than by reading. **(i) The count was 14, not 11** — `S` was derived
  twice (the `Cli` struct; `anvil --help`'s `Options:` block) and the two agree exactly at 107,
  and `book/src/knobs.md` is byte-identical `f2d282e..5ce2dd3`, so the file never moved: the
  registered 11 needs whole-section scope against a 105-flag `S`. The **sixth** instrument note
  in this repo's recent history, and the second in this tree. **(ii) The section was not merely
  behind — it was false.** Its *"not yet exposed via CLI"* list named 12 knobs of which **9 have
  CLI flags**, 8 of them contradicted ~60 lines earlier *inside the same section* and all 9 by
  line 1128 of the same chapter. That is the failure mode `.1` called worse than an omission,
  and it is what settled R1: refreshing a copy returns it to *behind*, gating one *freezes* it,
  and only deleting it removes the mode. Lossless was **proven before** the cut (0 of 107 flags
  lost a home; the 3 truthful bullets are strict subsets of richer entries in the same chapter),
  and the 9 capability-knob paragraphs were **kept** — they are prose, failing `0033` test (2),
  so only their heading was wrong. `.5` registered: the §*`tool_matrix`* block names **22 of
  37**, a different binary's registry needing a different extractor.
- `2026-08-01` (`.5`): The `tool_matrix` block is **deleted** by `0033` rung **R1**, and
  `USER_GUIDE.md` §*Tool matrix sweeps* now carries a `### Options` list under a recorded
  contract — *exhaustive over the options `tool_matrix` declares*, with clap's built-ins outside
  the set by a **derivable** rule (a built-in is an option with no `Cli` field), `35 + 2 = 37`.
  **The contract decision came from `.2`'s criterion failing, not from picking one of the three
  options the leaf offered:** `tool_matrix` has no `Overrides` projection and no knobs, so under
  the knob/mode partition every option is a *mode* flag and a knob chapter owes **zero** rows —
  the block was a flag list, for a binary with no knobs, inside the knob chapter. **R1 was not
  lossless as found, and measuring that first changed the order of the repair:** four declared
  options (`--base-seed`, `--verilator-bin`, `--yosys-bin`, `--iverilog-bin`) had no documented
  home anywhere in the live docs outside the snapshot, against `.4`'s 0 of 107, so the
  destination was completed (**32 → 35** declared options named, +7 entries) and losslessness
  proven **before** the cut and again after. Two smaller findings, both recorded because they
  are instrument facts rather than content: the registered **22** is *section* scope and the
  snapshot itself named **21** (the extra is `--diff-sim` as a prose cross-reference — the
  **seventh** instrument note here, and unlike `.2` → `.4` neither number is wrong); and
  `--divergence` is `.1`'s trap read backwards — one *spelling* in two registries, documented
  under `anvil hunt` while the matrix's own column had no entry in the reference. `.6`
  registered for the mechanism question with two constraints `.3` did not face, both measured:
  the region carries three foreign tokens (other tools' flags in prose) ⇒ **coverage, not exact
  parity**; and the set is **not substring-free** ⇒ a naive `covers_` grep can pass a deleted
  entry on another entry's text.
