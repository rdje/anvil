# LIVE-DOC-REGISTRY-SHADOWS: live docs that name a subset of an authoritative registry

## Metadata

- Tree ID: `LIVE-DOC-REGISTRY-SHADOWS`
- Status: `done`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.3` **done** — the fence landed; **tree CLOSED**)
- Owner: repo-local workflow

## Goal

Three live-doc sites name a **strict subset** of a registry that is
authoritative elsewhere in the repo, and no mechanism notices:

| site | names | registry | authoritative source |
| --- | ---: | ---: | --- |
| `docs/knowledge/api-reference.md` | **9** MCP tools | **10** | `tools/list` (`src/mcp/mod.rs`) |
| `book/src/agent-mcp.md` | **6** steering categories | **8** | the `knob_ids!` table (`src/ir/knob_id.rs`) |
| `book/src/api-introspection.md` | **6** steering categories | **8** | the same table |

All three were **measured against the running system**, not inferred, during
[`BOOK-TEST-COUNT-SHADOWS.2`](BOOK-TEST-COUNT-SHADOWS.md)'s effect-keyed sweep.

## The measurement

**MCP tools.** A live `tools/list` against `target/release/anvil-mcp` returns
**10**: `generate`, `introspect`, `dump_config`, `validate`, `minimize`,
`coverage_gaps`, `analyze`, `coverage`, `hunt`, `divergence`. The Knowledge Map
card `docs/knowledge/api-reference.md` says *"the **9 tools**"* and then **lists
nine** — `coverage` is absent from the enumeration, not merely uncounted. The
same card cites `schema_version 1.11`; the live value is **`1.27`**.

**Steering categories.** The `knob_ids!` table's third column yields **8**:
`datapath`, `emission`, `hierarchy`, `motifs`, `selectors`, `sharing`, `state`,
`terminals`. Two book chapters name six of them, omitting `motifs` and
`emission`:

- `book/src/agent-mcp.md:1078` — *"the **fixed set** `state` / `selectors` /
  `datapath` / `terminals` / `sharing` / `hierarchy`"*. The words "fixed set"
  make this a positive false claim rather than a partial example.
- `book/src/api-introspection.md:111` — *"pooled over each coarse category
  (`state` / `selectors` / `datapath` / `terminals` / `sharing` /
  `hierarchy`)"*.

Correct at every other list site, measured: `USER_GUIDE.md`, `book/src/knobs.md`,
`book/src/algorithm.md`, `docs/AGENT_INTROSPECTION_SCHEMA.md`,
`CODEBASE_ANALYSIS.md` all name all eight.

## Why no mechanism caught it

`ENUMERATION-PARITY` **has** a pair for exactly this set — pair 4, *"the live
docs that enumerate the `--steer` category taxonomy"* — and it passes, because
it checks **four declared sites** and neither offending chapter is among them:

```text
book/src/algorithm.md  book/src/knobs.md  USER_GUIDE.md  docs/AGENT_INTROSPECTION_SCHEMA.md
```

The pair's own comment records that it was written at
`COVERAGE-STEERED-GENERATION.4c`, which *"found the six-name list copied into
five live docs plus the book"* — it enumerated the sites **it happened to find**
and never re-swept from the authoritative set. That is decision
[`0033`](../decisions/0033-shadow-enumeration-classification.md) rule (2)
biting the mechanism built to enforce decision `0033`: *search from the
authoritative set, never from the shadow you found first.* A declared-site list
is itself a hand-maintained set — the third copy of the problem.

This is the sharpest available restatement of that rule, and the reason this
tree exists rather than a one-line doc fix.

## Non-Goals

- **Not "correct the numbers and move on."** `agent-mcp.md`'s and
  `api-introspection.md`'s counts are incidental; the **membership** is the
  defect. Same for the KM card, where the count `9` is R1-by-deletion but the
  missing `coverage` entry is a repair.
- **Not "add the two chapters to pair 4 and stop."** That repeats the exact
  failure — extending a hand-maintained site list by the two sites we happened
  to find this time. `.2` must decide whether the declared-site list can be
  *derived* instead, and record the answer either way.
- **No code change.** `src/` is untouched; the registries are already correct.
  Any change lands in docs, `docs/knowledge/`, and possibly
  `scripts/check_enumeration_parity.sh`.

## Acceptance Criteria

- Every site above names the full registry, or drops the enumeration entirely
  (decision `0033` rung **R1** — a list that must not be maintained is deleted,
  not gated).
- The KM card's stale `schema_version` is re-derived from
  `introspect::SCHEMA_VERSION`, not hand-copied.
- `KNOWLEDGE_MAP.md` regenerated from its cards and `check_knowledge_map.sh`
  clean — the card is the source, the map is derived.
- The declared-site question in `.2` is answered with a measurement and a
  recorded decision, not a guess.
- `mdbook build` clean; `scripts/check_doctrines.sh` 8/8; docs-only ⇒ DUT
  byte-identical.

## Task Tree

- ID: `LIVE-DOC-REGISTRY-SHADOWS`
  Status: `done`
  Goal: `Repair three live-doc sites that name a strict subset of an authoritative registry, and decide whether ENUMERATION-PARITY's declared-site list can stop being hand-maintained.`
  Children: `.1` (repair the three sites), `.2` (the declared-site question), `.3` (implement the list-scoped predicate `.2` decided on)

- ID: `LIVE-DOC-REGISTRY-SHADOWS.1`
  Status: `done`
  Goal: `Repair the three measured sites: add the missing coverage tool to the KM card's list and delete the 9 beside it; re-derive that card's schema_version; add motifs + emission to both book chapters' category lists. Regenerate KNOWLEDGE_MAP.md from the cards.`
  Acceptance: `A live tools/list, the knob_ids! table, and introspect::SCHEMA_VERSION are each re-measured at repair time and the doc matches; the count beside the tool list is gone (R1) while the list itself is complete; check_knowledge_map.sh clean; mdbook build clean; check_doctrines.sh 8/8; docs-only.`
  Verification: `done — RE-MEASURED at repair time (rule 0), all three confirming the registration measurement: a live tools/list returns 10 (generate, introspect, dump_config, validate, minimize, coverage_gaps, analyze, coverage, hunt, divergence); introspect::SCHEMA_VERSION is "1.27"; the knob_ids! category column yields 8. REPAIRED. (a) docs/knowledge/api-reference.md: the tool list gained the missing `coverage` entry and the count "9" beside it was DELETED (rung R1 — a number beside a list is one more copy of it, the same repair .1 of BOOK-TEST-COUNT-SHADOWS applied to api-reference.md); the pure/controlled split was taken from book/src/api-tools.md rather than guessed, putting `coverage` in the pure group. The card's `schema_version 1.11` was replaced by a POINTER to introspect::SCHEMA_VERSION plus an explicit note that copying the number is how the line came to say 1.11 against a live 1.27 — the derivation-not-snapshot rung again. The `evidence:` frontmatter line carried the same three counts ("the 9 tools", "the 5 prompts", "the 4 analyze query schemas"); all three numbers dropped there too, since a count in a retrieval card is read INSTEAD of re-deriving. (b) book/src/agent-mcp.md and (c) book/src/api-introspection.md: both category lists completed to all eight, and both gained the one sentence that makes the two new members non-obvious — `motifs` rolls at most once per module while `emission` rolls once per candidate gate, which is WHY they are separate categories and not one. KNOWLEDGE_MAP.md regenerated from the cards (88 facts / 889 question keys) and check_knowledge_map.sh reports in-sync; verified the stale count is absent from the generated map, i.e. the fix went into the SOURCE card and propagated, not into the derived file. NEGATIVE CONTROL, and it genuinely fails: a probe copy of check_enumeration_parity.sh with the two chapters added to pair 4 was run against `git show HEAD:<file>` (pre-repair) and against the worktree (post-repair) — agent-mcp.md FAILS at HEAD missing {emission, motifs} and passes after; api-introspection.md FAILS at HEAD missing {motifs} and passes after. Per the recorded gotcha, a control that passes on the first try is worthless, so this was checked in both directions before being believed. THE CONTROL ALSO SURFACED A REAL WEAKNESS IN THE PROPOSED MECHANISM, recorded here as input to .2: covers_set is a WHOLE-FILE SUBSTRING grep, so api-introspection.md scored 7/8 rather than 6/8 — the word "emission" appears elsewhere in that file, far from the list. Extending pair 4 to these sites would therefore have caught agent-mcp.md outright but only partially diagnosed api-introspection.md, and a file whose list is short can pass outright if every missing word happens to appear somewhere else in it. That is the same shape as the documented datapath/covers_set over-match hazard, and it means .2 cannot simply add sites — it has to decide whether covers_set should check the LIST rather than the FILE. Also corrected in passing: the BOOK-TEST-COUNT-SHADOWS container node still read Status: active after its metadata was flipped to done in the previous commit. Checks: mdbook build clean; check_knowledge_map.sh in sync; scripts/check_doctrines.sh 8/8 after git add. Docs-only => DUT byte-identical.`
  Commit: `abf7090` — `LIVE-DOC-REGISTRY-SHADOWS.1 — name the whole registry, not the part we found`

- ID: `LIVE-DOC-REGISTRY-SHADOWS.2`
  Status: `pending`
  Goal: `Answer whether ENUMERATION-PARITY pair 4's declared-site list can be DERIVED (e.g. every tracked doc that names >= 3 category words on one line) rather than hand-maintained, and either replace it or record precisely why a hand-maintained list is correct here.`
  Acceptance: `The candidate derived selector is RUN against the tree and its output compared to the current four sites; if it over-matches, the false positives are named. Decision 0033's own middle test (growth-coupled) is applied explicitly, since the pair's comment already argues a discovered list would cry wolf on ordinary prose about "state and sharing" — that argument is re-tested with a real selector rather than assumed. Whatever the outcome, it lands as a decision record, because both answers are load-bearing for every future pair.`
  Verification: `done — decision 0037. BOTH HALVES ANSWERED BY MEASUREMENT, and the two turned out to be coupled. (i) THE SITE LIST CANNOT BE DERIVED — and not for the predicted reason. The candidate selector ("every tracked *.md naming >= T of the eight category words on one line") was RUN over git ls-files '*.md' at abf7090: it selects 56/20/16/14/13/6/5 files at T=2..8, and NO threshold yields the four declared sites. It fails in BOTH directions and the two failures share no threshold. Loose (T<=6): it selects append-only history — CHANGES.md:8400 and DEVELOPMENT_NOTES.md:3999 hold the LITERAL six-name list ((state)/(selectors)/(datapath)/(terminals)/(sharing)/(hierarchy)), correct when written, and docs/tasks/COVERAGE-STEERED-GENERATION.md:75 records the historical 21-variant/6-category state. Under a derived site list the gate would demand these name all eight, satisfiable ONLY by retro-editing history (absolutely forbidden, decision 0031: "keep it raw, keep honest") or by an exclusion list — the same hand-maintained list, sign-flipped, and now growth-coupled to a file that grows every commit. Tight (T>=7): it DROPS book/src/algorithm.md and book/src/knobs.md, 2 of the 4 declared sites, purely because their enumerations wrap across two lines — a prose-reflow accident silently exempting a site, which is PARITY-EXTRACTOR-ARM-SHAPE-GAP's exact hazard re-imported on the doc side — and it still retains two task files, so tightening buys loss, not purity. Under decision 0033 rule (a) the declared-site list therefore FAILS TEST (2): it is SUPPOSED to differ from "every file naming the ids", and the gap IS the content of the rule — structurally identical to check_no_boot_volume_refs.sh's allow-list, for the same underlying reason (history must stay raw). AUTHORITATIVE, leave it hand-written; the fifth hard case on 0033's list and the first at the ENFORCEMENT layer rather than the content layer. The pair's own comment asserted this ("a grep for any file mentioning two category words also matches ordinary prose") and it measured TRUE — but the real reason is different and stronger than the asserted one. The selector is NOT discarded: it correctly surfaced CODEBASE_ANALYSIS.md:2349 (all eight on one line, undeclared) alongside the two chapters .1 repaired, so it is adopted as a REVIEW-TIME DISCOVERY INSTRUMENT and explicitly barred from being a gate input — decision 0033 (c)'s "discovered by review, held by derivation" given a written procedure. (ii) covers_set IS THE WRONG PREDICATE, and the defect is larger than .1's 7/8 miscount. Applying 0033 rule (2) — sweep from the AUTHORITATIVE SET, which here is EVERY USE OF THE PREDICATE (10 sites: 4+2+4), not the one instance reported — and probing each by DELETING THE ENUMERATION AND RE-RUNNING: 3 of 10 STILL PASS. book/src/api-tools.md and book/src/agent-mcp.md (pair 3, the downstream allow-list) are BOTH vacuous, so pair 3 provides ZERO protection at either site; book/src/knobs.md (pair 4) likewise; USER_GUIDE.md misses by ONE word (terminals). Pair 1b survives on an accident of vocabulary, and only just — CODEBASE_ANALYSIS.md and README.md each fail by 7 of 8, so one doctrine id already appears outside their lists. The predictor is measurable: occurrences of an id OUTSIDE its enumeration are yosys 18 / verilator 12 / sv2v 2 / slang 2 / iverilog 1 in api-tools.md, versus MEMORY-ARCH 0 / README-GROWTH 0 / KNOWLEDGE-MAP 0 in architecture.md. THE RULE: a coverage check is strong in inverse proportion to how ordinary its ids are as words in the checked document — and they are most ordinary in exactly the document that documents them, so it is weakest where the pair matters most. A PROXIMITY WINDOW WAS EVALUATED AND REJECTED ON MEASUREMENT, not on taste: with the enumeration deleted the minimal spanning window in api-tools.md is STILL 2 lines and in agent-mcp.md 2 lines, so a window fixes neither pair-3 site; and a single-id omission (motifs removed) leaves a 6-line window at algorithm.md and a 4-line window at agent-mcp.md, both of which any usable K passes — because .1's OWN explanatory prose (motifs rolls once per module, emission once per candidate gate) sits three lines below the list. Its miss cases correlate with documentation QUALITY, which is the wrong thing for a gate to punish. CHOSEN REPAIR: an HTML-comment fence delimiting each declared enumeration, with covers_set reading only inside the fence, exact parity in BOTH directions (the one-directional design existed because "a chapter may name a subset in an example" — with a fence, examples live outside it), and a missing fence a hard failure. The fence NAMES NO MEMBERS, so it fails 0033 test (2) and cannot become the next shadow. Adopted repo-wide: "delete the subject and re-run the check" is now the standard acceptance test for any coverage-shaped doctrine check, recorded in DOCTRINE_ENFORCEMENT.md 9 (honest limits) alongside the live 3-of-10 vacuity, which stays stated until .3 removes it. Scope split per COMMIT.md task-tree rule 3: this leaf is the decision; .3 implements. Checks: check_doctrines.sh 8/8 after git add; mdbook build clean; check_knowledge_map.sh in sync. Docs-only => DUT byte-identical.`
  Commit: `e873a6e` — `LIVE-DOC-REGISTRY-SHADOWS.2 — the site list is authoritative; the predicate is not`

- ID: `LIVE-DOC-REGISTRY-SHADOWS.3`
  Status: `done`
  Goal: `Implement decision 0037: fence every declared enumeration with an HTML comment carrying its set id, rewrite covers_set to read only the fenced region and to require exact parity in both directions, fail hard on a declared site with no fence, and declare the three reviewed pair-4 sites (book/src/agent-mcp.md, book/src/api-introspection.md, CODEBASE_ANALYSIS.md) so pair 4 goes from 4 sites to 7.`
  Acceptance: `The vacuity probe — delete the enumeration, re-run the check — must now FAIL at all three sites where it currently passes (book/src/api-tools.md, book/src/agent-mcp.md pair 3, book/src/knobs.md pair 4); it is the acceptance test, so it is run before and after. Every pair negative-controlled in both directions: drop one id from a fenced list => FAIL, restore => PASS, at every declared site, not a sampled one (a proof sampling one member of a set cannot detect the set is partitioned). A declared site with no fence FAILS loudly. Fences are invisible in the rendered book (mdbook build clean, spot-checked in the generated HTML). No new hand-maintained list is introduced: the fence carries a set id and no members. check_doctrines.sh 8/8; docs + scripts only => DUT byte-identical.`
  Verification: `done — IMPLEMENTED. covers_set is replaced by covers_fenced_set, which reads only the text between <!--enum:ID--> and <!--/enum:ID-->; a declared site with no fence (or an empty one) is a HARD FAILURE, so the predicate cannot silently decay back to whole-file matching. 13 fences added across 11 files: doctrine-ids at README.md + book/src/architecture.md + docs/knowledge/doctrine-enforcement.md + CODEBASE_ANALYSIS.md; adapter-ids at book/src/api-tools.md + book/src/agent-mcp.md; steer-categories at book/src/algorithm.md + book/src/knobs.md + USER_GUIDE.md + docs/AGENT_INTROSPECTION_SCHEMA.md + book/src/agent-mcp.md + book/src/api-introspection.md + CODEBASE_ANALYSIS.md. Pair 4 declared over 7 sites (was 4). MARKERS ARE INLINE, NOT ON THEIR OWN LINE, and that is load-bearing rather than stylistic: measured against the real sites, most enumerations live mid-paragraph, inside a Markdown TABLE ROW (agent-mcp.md:186, api-introspection.md:111) or inside a bulleted list, and an HTML comment on its own line is a CommonMark HTML *block* that interrupts a paragraph and splits a table or list — it would change the rendered book. The marker carries the SET ID because book/src/agent-mcp.md and CODEBASE_ANALYSIS.md are each declared sites for TWO different sets, which settles 0037's open question by measurement rather than guess. USER_GUIDE.md has two enumerations; the prose one (:402) is fenced and the one inside the ``` error-message code block (:438) deliberately is not — a marker there would be VISIBLE. ACCEPTANCE TEST PASSES: the vacuity probe (delete the enumeration, re-run) now FAILS at all 13 sites, including the 3 that previously passed with it deleted — both pair-3 sites and knobs.md. NEGATIVE CONTROLS, EVERY SITE NOT A SAMPLE (a proof sampling one member of a set cannot detect the set is partitioned): 98 single-id drops — 8 categories x 7 sites + 5 adapters x 2 sites + 8 doctrine ids x 4 sites — every one FAILS, plus 2 empty-fence controls FAIL and the unmutated tree PASSES. Controls were run by mutating the worktree and restoring from the INDEX (git checkout-index -f), never `git checkout --`, per the recorded gotcha that a checkout during a control sweep discards unrelated edits — here, all 13 fences. THE CONTROL CAUGHT A REAL DEFECT IN THIS LEAF'S OWN WORK, and it is the finding worth keeping: the first 1b run had 1 wrong pass of 32 — dropping MEMORY-ARCH from book/src/architecture.md left the gate GREEN, because the honest-limit paragraph .2 added to that chapter sat INSIDE what became the doctrine-ids fence and named MEMORY-ARCH a second time to illustrate the coined-token point. The vacuity defect, reproduced at fence scale, inside the fix for it, by the sentence explaining it. Repaired by moving the commentary out of the fence (where it reads better anyway), and a duplicate-id audit across all 13 fences now reports none. Two limits recorded as a dated Correction on decision 0037 rather than edited away: (i) BIDIRECTIONAL PARITY IS WITHDRAWN — 4 fences must enclose bulleted list items that carry prose, so harvesting the ids a region names would also harvest every other backticked token in it and cry wolf; the reverse direction is near-empty by policy anyway since nothing is ever retired; (ii) a fence must contain the enumeration and NOT the discussion of it. RENDERING VERIFIED, not assumed: mdbook build clean, and the generated HTML checked — markers survive as HTML comments (invisible), no marker leaks as visible text at any of 6 spot-checked chapters, and algorithm.md's category sentence is still one paragraph with the fence inside it. Checks: check_doctrines.sh 8/8 after git add; mdbook build clean; check_knowledge_map.sh in sync. src/ untouched => DUT byte-identical.`
  Commit: `ff2406c` — `LIVE-DOC-REGISTRY-SHADOWS.3 — check the list, not the file (tree CLOSED)`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-REGISTRY-SHADOWS.1` | `done` | All three sites repaired against re-measured registries: the KM card gained the missing `coverage` tool and lost its `9`, its `schema_version 1.11` became a pointer to `introspect::SCHEMA_VERSION`, and both book chapters now name all eight categories with the `motifs`-vs-`emission` resolution difference spelled out. Negative control **fails pre-repair, passes post** — and in failing, exposed that `covers_set` greps the whole *file*, not the *list*, which is now `.2`'s central question rather than an afterthought. |
| 2 | `LIVE-DOC-REGISTRY-SHADOWS.2` | `done` | Both halves answered by measurement and landed as decision `0037`. **(i) No** — the derived selector over- *and* under-matches, with no threshold in between: at `T ≤ 6` it selects append-only history holding the literal six-name list, so the gate would be satisfiable only by rewriting history (decision `0031`, forbidden) or by an inverted hand-maintained list; at `T ≥ 7` it drops 2 of the 4 declared sites on a prose-reflow accident. The site list **fails `0033` test (2)** and is authoritative — history must stay raw. The selector is kept as a **review-time discovery instrument** (it found `CODEBASE_ANALYSIS.md:2349` undeclared). **(ii) Yes, and worse than reported** — sweeping all 10 uses of the predicate, **3 pass with the enumeration deleted outright**, including *both* pair-3 sites, so pair 3 checks nothing today. A proximity window was measured and rejected. Repair: fence the enumeration. |
| 3 | `LIVE-DOC-REGISTRY-SHADOWS.3` | `done` | **Tree complete.** 13 fences across 11 files; `covers_fenced_set` reads only inside the fence and a missing fence is a hard failure. **The acceptance test passes**: the vacuity probe now fails at all 13 sites, including the 3 that previously passed with the enumeration deleted. 98 single-id controls — every id at every site, not a sample — all fail; 2 empty-fence controls fail; unmutated passes. The control caught **this leaf's own defect**: `.2`'s honest-limit paragraph sat inside `architecture.md`'s fence and named `MEMORY-ARCH` twice, so dropping it from the registry list stayed green — the vacuity reproduced at fence scale by the sentence explaining vacuity. Bidirectional parity **withdrawn** (a fence enclosing prose-bearing list items is not losslessly extractable); recorded as a dated Correction on `0037`. |

## Decisions

- `2026-07-31`: Registered as its own tree rather than folded into
  `BOOK-TEST-COUNT-SHADOWS`. Both are live-doc enumeration defects, but that
  tree's shape is *a count of a derivable set* and this one's is *the membership
  of an authoritative registry* — different repair (delete the number vs. add the
  missing member), different mechanism (no gate vs. a gate that passes because
  its site list is short), and one of these needs a `scripts/` change. Same
  reasoning that split `BOOK-TEST-COUNT-SHADOWS` from
  `PARITY-EXTRACTOR-ARM-SHAPE-GAP`: two focused trees beat one muddled one.
- `2026-07-31` (`.2`, decision [`0037`](../decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md)):
  the declared-site list is **authoritative and stays hand-written**, because it must be
  allowed to differ from "every tracked file naming the ids" — append-only history holds
  the *old* list correctly and may never be retro-edited (decision `0031`). Measured, not
  assumed: no threshold of the candidate selector reproduces the declared sites, and the
  two ways it fails do not share a threshold.
- `2026-07-31` (`.2`): the discovery selector is **kept and demoted** rather than
  discarded — a review-time instrument, never a gate input. It earned that by finding a
  real undeclared site (`CODEBASE_ANALYSIS.md:2349`) that no bug report had surfaced.
- `2026-07-31` (`.2`): `covers_set` moves from **file-scoped to fence-scoped**, and
  from one-directional coverage to **exact parity inside the fence**. The
  one-directional design existed to permit a chapter naming a subset in an example; a
  fence makes that concern disappear, because examples sit outside it.
- `2026-07-31` (`.2`): the implementation is **split into `.3`** rather than folded in
  here. `COMMIT.md` task-tree rule 3 (one leaf per commit), and the fence touches nine
  files and needs its own negative-control pass — including re-running the vacuity probe
  in the failing direction.
- `2026-07-31`: Repair rung fixed **before** the work starts, per this lane's
  convention. The **count** `9` beside the tool list is **R1, delete** — a number
  beside a list is one more copy of it. The **list** is **repaired**, not deleted:
  it is the API surface a reader needs enumerated, and `ENUMERATION-PARITY`
  already holds two other chapters to exactly that standard.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS.3` | `13 fences added across 11 files and covers_set replaced by covers_fenced_set (fence-scoped, missing fence = hard failure); pair 4 declared over 7 sites. ACCEPTANCE TEST: the vacuity probe (delete the enumeration, re-run) now FAILS at all 13 sites, including the 3 that previously passed. CONTROLS AT EVERY SITE, NOT A SAMPLE: 98 single-id drops (8 categories x 7 sites, 5 adapters x 2, 8 doctrine ids x 4) all FAIL; 2 empty-fence controls FAIL; unmutated PASSES; mutations restored from the INDEX via git checkout-index, never git checkout --. The first 1b run had 1 WRONG PASS of 32 — .2's own honest-limit paragraph sat inside architecture.md's fence and named MEMORY-ARCH twice — repaired by moving the commentary out, and a duplicate-id audit over all 13 fences now reports none. Rendering verified in the generated HTML: markers survive as invisible HTML comments, none leaks as text at 6 spot-checked chapters, paragraphs/tables intact. mdbook build clean; check_doctrines.sh 8/8 after git add; check_knowledge_map.sh in sync` | `done — tree CLOSED` |
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS.2` | `ran the candidate derived selector over git ls-files '*.md' at seven thresholds (T=2..8 -> 56/20/16/14/13/6/5 files) and classified every candidate: no threshold yields the four declared sites; T<=6 selects CHANGES.md:8400 + DEVELOPMENT_NOTES.md:3999 + docs/tasks/* holding the literal historical six-name list (unsatisfiable without a forbidden history rewrite), T>=7 drops book/src/algorithm.md + book/src/knobs.md because their lists wrap across two lines. Ran the VACUITY PROBE (delete the enumeration, re-run covers_set) against ALL TEN uses of the predicate, not the one .1 reported: 3 still pass — book/src/api-tools.md and book/src/agent-mcp.md (pair 3, both sites, so pair 3 protects nothing) and book/src/knobs.md (pair 4); USER_GUIDE.md misses by one word. Measured the predictor (id occurrences outside the enumeration: yosys 18 / verilator 12 vs MEMORY-ARCH 0 / README-GROWTH 0). Evaluated and rejected a proximity window by measurement: window is still 2 lines at both pair-3 sites with the allow-list deleted, and a single-id omission leaves 6 lines at algorithm.md / 4 at agent-mcp.md. Landed decision 0037 + the honest limit in DOCTRINE_ENFORCEMENT.md 9. check_doctrines.sh 8/8 after git add; mdbook build clean; check_knowledge_map.sh in sync` | `done — decision 0037; .3 registered` |
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS.1` | `re-measured all three registries at repair time (tools/list = 10; SCHEMA_VERSION = 1.27; knob_ids! categories = 8) and repaired every site: coverage added to the KM card's tool list + the "9" deleted (R1), schema_version 1.11 replaced by a pointer to introspect::SCHEMA_VERSION, the card's evidence frontmatter stripped of its three counts, and motifs + emission added to both book chapters with the once-per-module vs once-per-gate distinction spelled out. KNOWLEDGE_MAP.md regenerated from the cards and verified in sync, with the stale count confirmed absent from the generated map. Negative control run in BOTH directions against git show HEAD: — agent-mcp.md fails pre-repair missing {emission, motifs}, api-introspection.md fails missing {motifs}, both pass after — and it exposed that covers_set greps the whole file rather than the list, scoring api-introspection.md 7/8 when its list held 6/8. mdbook build clean; check_doctrines.sh 8/8 after git add` | `done` |
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS` | `measured a live tools/list (10 tools; docs/knowledge/api-reference.md lists 9, coverage absent) and the knob_ids! category column (8; book/src/agent-mcp.md and book/src/api-introspection.md each name 6, omitting motifs + emission); confirmed the other five list sites name all eight; confirmed ENUMERATION-PARITY pair 4 passes because neither offending chapter is one of its four declared sites; confirmed prompts/list = 5 so the card's "5 prompts" is correct` | `defect confirmed, pre-existing, live` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-REGISTRY-SHADOWS.1` | `LIVE-DOC-REGISTRY-SHADOWS.1 — name the whole registry, not the part we found` | `coverage` added to the KM card's tool list + the count R1-deleted; `schema_version` de-snapshotted; both book chapters completed to all eight steering categories. Negative control fails pre-repair. |
| `LIVE-DOC-REGISTRY-SHADOWS.2` | `LIVE-DOC-REGISTRY-SHADOWS.2 — the site list is authoritative; the predicate is not` | Decision `0037`. The site list cannot be derived (history must stay raw); `covers_set` is vacuous at 3 of 10 sites, both pair-3 sites among them. Window predicate measured and rejected. `.3` registered to implement the fence. |
| `LIVE-DOC-REGISTRY-SHADOWS.3` | `LIVE-DOC-REGISTRY-SHADOWS.3 — check the list, not the file` | 13 fences, `covers_fenced_set`, missing fence = hard failure, pair 4 over 7 sites. Vacuity probe now fails at all 13; 98 single-id controls all fail. Caught and fixed its own prose-inside-the-fence defect. Tree **CLOSED**. |

## Changelog

- `2026-07-31` (`.2`): the tree's central question turned out to have a **larger second
  half than its first**. "Can the site list be derived?" resolved to a clean *no* with a
  reason worth keeping (history must stay raw). "Is `covers_set` the right predicate?"
  resolved to a *no* that found **a live gate protecting nothing at two sites** — pair 3,
  the downstream allow-list, passes with its enumeration deleted outright. That was
  reachable only by sweeping from the authoritative set (every use of the predicate)
  rather than from the site `.1` reported: decision `0033` rule (2), third application,
  and the second time it has turned a one-site report into a class.
- `2026-07-31`: Created by `BOOK-TEST-COUNT-SHADOWS.2`'s effect-keyed sweep. The
  finding that makes the tree worth opening is not any one stale list — it is
  that **the gate for this exact defect class was green throughout**, because a
  gate whose coverage is a hand-written list of sites inherits the very failure
  it exists to prevent. Decision `0033` rule (2) applied to `0033`'s own
  enforcement.
