# LIVE-DOC-REGISTRY-SHADOWS: live docs that name a subset of an authoritative registry

## Metadata

- Tree ID: `LIVE-DOC-REGISTRY-SHADOWS`
- Status: `active`
- Roadmap lane: Live-doc hygiene / shadow-enumeration residue
- Created: `2026-07-31`
- Last updated: `2026-07-31` (registered; frontier `.1`)
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
  Status: `pending`
  Goal: `Repair the three measured sites: add the missing coverage tool to the KM card's list and delete the 9 beside it; re-derive that card's schema_version; add motifs + emission to both book chapters' category lists. Regenerate KNOWLEDGE_MAP.md from the cards.`
  Acceptance: `A live tools/list, the knob_ids! table, and introspect::SCHEMA_VERSION are each re-measured at repair time and the doc matches; the count beside the tool list is gone (R1) while the list itself is complete; check_knowledge_map.sh clean; mdbook build clean; check_doctrines.sh 8/8; docs-only.`
  Verification: `pending`
  Commit: `pending`

- ID: `LIVE-DOC-REGISTRY-SHADOWS.2`
  Status: `pending`
  Goal: `Answer whether ENUMERATION-PARITY pair 4's declared-site list can be DERIVED (e.g. every tracked doc that names >= 3 category words on one line) rather than hand-maintained, and either replace it or record precisely why a hand-maintained list is correct here.`
  Acceptance: `The candidate derived selector is RUN against the tree and its output compared to the current four sites; if it over-matches, the false positives are named. Decision 0033's own middle test (growth-coupled) is applied explicitly, since the pair's comment already argues a discovered list would cry wolf on ordinary prose about "state and sharing" — that argument is re-tested with a real selector rather than assumed. Whatever the outcome, it lands as a decision record, because both answers are load-bearing for every future pair.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-REGISTRY-SHADOWS.1` | `pending` | **Next.** Three measured, live, user-visible inaccuracies — two of them in the book, which is the owner's only window into the project. Repair is mechanical once measured; the measurement is already done and recorded above. Re-measure at repair time anyway (rule 0). |
| 2 | `LIVE-DOC-REGISTRY-SHADOWS.2` | `pending` | The general question. `.1` fixes three sites; `.2` decides whether the *gate* can stop depending on someone having found them. Deliberately second: repair the live inaccuracy first, then reason about the mechanism with the tree clean. |

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
| `2026-07-31` | `LIVE-DOC-REGISTRY-SHADOWS` | `measured a live tools/list (10 tools; docs/knowledge/api-reference.md lists 9, coverage absent) and the knob_ids! category column (8; book/src/agent-mcp.md and book/src/api-introspection.md each name 6, omitting motifs + emission); confirmed the other five list sites name all eight; confirmed ENUMERATION-PARITY pair 4 passes because neither offending chapter is one of its four declared sites; confirmed prompts/list = 5 so the card's "5 prompts" is correct` | `defect confirmed, pre-existing, live` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| — | — | `.1` pending |

## Changelog

- `2026-07-31`: Created by `BOOK-TEST-COUNT-SHADOWS.2`'s effect-keyed sweep. The
  finding that makes the tree worth opening is not any one stale list — it is
  that **the gate for this exact defect class was green throughout**, because a
  gate whose coverage is a hand-written list of sites inherits the very failure
  it exists to prevent. Decision `0033` rule (2) applied to `0033`'s own
  enforcement.
