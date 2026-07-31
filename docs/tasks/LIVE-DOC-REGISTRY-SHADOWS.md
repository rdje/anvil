# LIVE-DOC-REGISTRY-SHADOWS: live docs that name a subset of an authoritative registry

## Metadata

- Tree ID: `LIVE-DOC-REGISTRY-SHADOWS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (`.1` **done** — three sites repaired, KM regenerated; frontier `.2`)
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
  Status: `active`
  Goal: `Repair three live-doc sites that name a strict subset of an authoritative registry, and decide whether ENUMERATION-PARITY's declared-site list can stop being hand-maintained.`
  Children: `.1` (repair the three sites), `.2` (the declared-site question)

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
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-REGISTRY-SHADOWS.1` | `done` | All three sites repaired against re-measured registries: the KM card gained the missing `coverage` tool and lost its `9`, its `schema_version 1.11` became a pointer to `introspect::SCHEMA_VERSION`, and both book chapters now name all eight categories with the `motifs`-vs-`emission` resolution difference spelled out. Negative control **fails pre-repair, passes post** — and in failing, exposed that `covers_set` greps the whole *file*, not the *list*, which is now `.2`'s central question rather than an afterthought. |
| 2 | `LIVE-DOC-REGISTRY-SHADOWS.2` | `pending` | **Next.** The general question, and `.1` sharpened it: it is no longer only *"can the declared-site list be derived?"* but also *"is `covers_set` even the right predicate?"* — `.1` measured it scoring `api-introspection.md` 7/8 when its list was 6/8, because a missing word appeared elsewhere in the file. A gate that reads the file instead of the list can pass on a doc whose enumeration is short. Both halves land as one decision record. |

## Decisions

- `2026-07-31`: Registered as its own tree rather than folded into
  `BOOK-TEST-COUNT-SHADOWS`. Both are live-doc enumeration defects, but that
  tree's shape is *a count of a derivable set* and this one's is *the membership
  of an authoritative registry* — different repair (delete the number vs. add the
  missing member), different mechanism (no gate vs. a gate that passes because
  its site list is short), and one of these needs a `scripts/` change. Same
  reasoning that split `BOOK-TEST-COUNT-SHADOWS` from
  `PARITY-EXTRACTOR-ARM-SHAPE-GAP`: two focused trees beat one muddled one.
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
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS.1` | `re-measured all three registries at repair time (tools/list = 10; SCHEMA_VERSION = 1.27; knob_ids! categories = 8) and repaired every site: coverage added to the KM card's tool list + the "9" deleted (R1), schema_version 1.11 replaced by a pointer to introspect::SCHEMA_VERSION, the card's evidence frontmatter stripped of its three counts, and motifs + emission added to both book chapters with the once-per-module vs once-per-gate distinction spelled out. KNOWLEDGE_MAP.md regenerated from the cards and verified in sync, with the stale count confirmed absent from the generated map. Negative control run in BOTH directions against git show HEAD: — agent-mcp.md fails pre-repair missing {emission, motifs}, api-introspection.md fails missing {motifs}, both pass after — and it exposed that covers_set greps the whole file rather than the list, scoring api-introspection.md 7/8 when its list held 6/8. mdbook build clean; check_doctrines.sh 8/8 after git add` | `done` |
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS` | `measured a live tools/list (10 tools; docs/knowledge/api-reference.md lists 9, coverage absent) and the knob_ids! category column (8; book/src/agent-mcp.md and book/src/api-introspection.md each name 6, omitting motifs + emission); confirmed the other five list sites name all eight; confirmed ENUMERATION-PARITY pair 4 passes because neither offending chapter is one of its four declared sites; confirmed prompts/list = 5 so the card's "5 prompts" is correct` | `defect confirmed, pre-existing, live` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-REGISTRY-SHADOWS.1` | `LIVE-DOC-REGISTRY-SHADOWS.1 — name the whole registry, not the part we found` | `coverage` added to the KM card's tool list + the count R1-deleted; `schema_version` de-snapshotted; both book chapters completed to all eight steering categories. Negative control fails pre-repair. |

## Changelog

- `2026-07-31`: Created by `BOOK-TEST-COUNT-SHADOWS.2`'s effect-keyed sweep. The
  finding that makes the tree worth opening is not any one stale list — it is
  that **the gate for this exact defect class was green throughout**, because a
  gate whose coverage is a hand-written list of sites inherits the very failure
  it exists to prevent. Decision `0033` rule (2) applied to `0033`'s own
  enforcement.
