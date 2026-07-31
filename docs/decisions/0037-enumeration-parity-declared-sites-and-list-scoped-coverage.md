---
id: enumeration-parity-declared-sites-and-list-scoped-coverage
title: `ENUMERATION-PARITY`'s declared-site list stays hand-written and is **authoritative** (a derived selector over- and under-matches *simultaneously*, and at every usable threshold it demands a rewrite of append-only history) — while `covers_set` is **measured vacuous at 3 of its 10 sites** and must be scoped to the enumeration, not the file
answers:
  - "can ENUMERATION-PARITY's declared documentation sites be derived instead of hand-listed"
  - "why is the ENUMERATION-PARITY site list not itself a shadow enumeration"
  - "why does covers_set grep the whole file and why is that wrong"
  - "which ENUMERATION-PARITY sites pass vacuously"
  - "does the adapter allow-list pair actually check anything"
  - "how do I tell whether a coverage check is vacuous"
  - "why is a proximity window not enough to fix covers_set"
  - "how should a docs-side enumeration be marked for checking"
  - "what happens if a derived site list includes CHANGES.md"
  - "why can a doctrine check never require append-only history to stay current"
date: 2026-07-31
status: accepted
tags: [doctrine, enumeration, enforcement, shadow-list, derivation, vacuity, gate-quality, history, book, north-star]
reverify: bash scripts/check_enumeration_parity.sh
evidence: scripts/check_enumeration_parity.sh:205-223 (`covers_set`), :243-262 (pair 1b), :276-289 (pair 3), :291-331 (pair 4); book/src/knobs.md:1595-1596, book/src/algorithm.md:375-377, USER_GUIDE.md:402-403 + :438, docs/AGENT_INTROSPECTION_SCHEMA.md:644, book/src/agent-mcp.md:186 + :1078-1079, book/src/api-introspection.md:111, book/src/api-tools.md:248, book/src/architecture.md:482-501, docs/knowledge/doctrine-enforcement.md:33-48, README.md:130-132, CODEBASE_ANALYSIS.md:2349 + :2669-2676; CHANGES.md:8400 and DEVELOPMENT_NOTES.md:3999 (the append-only historical six-name lists); docs/decisions/0031-ssd-volume-exclusivity.md (history stays raw); docs/decisions/0033-shadow-enumeration-classification.md (the three-question rule and the repair ladder). All counts measured `2026-07-31` at `abf7090`.
---

# 0037 - `ENUMERATION-PARITY`: the declared-site list is authoritative, and `covers_set` must read the list rather than the file

- Date: 2026-07-31
- Status: accepted
- Tree: `LIVE-DOC-REGISTRY-SHADOWS.2` (design leaf; answers both halves of the
  question `.1` left, and sets `.3`'s scope)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK**
  directive, from the frontier row set by `.1`.

## Context

`LIVE-DOC-REGISTRY-SHADOWS.1` repaired three live-doc sites that named a strict subset
of an authoritative registry. Two facts from that leaf define this one:

1. `ENUMERATION-PARITY` **has** a pair for the exact defect class that went unnoticed
   (pair 4, the `--steer` category taxonomy) and it was **green throughout**, because
   neither offending chapter was one of its four hand-declared sites. A gate whose
   coverage is a hand-written list of sites inherits the very failure it exists to
   prevent — decision `0033` rule (2) biting the mechanism built to enforce decision
   `0033`.
2. `.1`'s negative control, run in both directions, exposed a second and independent
   weakness: `covers_set` is a **whole-file substring grep**, so it scored
   `book/src/api-introspection.md` **7/8** while that file's actual enumeration held
   **6/8**. The word `emission` appeared elsewhere in the file, far from the list.

So `.2` is two questions, and they turn out to be coupled:

- **(i)** Can pair 4's declared-site list be **derived** rather than hand-maintained?
- **(ii)** Is `covers_set` the right **predicate**?

Both were answered by measurement against the real tree, not by reasoning about it.

## The measurements

### (i) The candidate selector — it over-matches and under-matches *at the same time*

The natural derivation, and the one the tree registered as the candidate: *every
tracked `*.md` that names ≥ **T** of the eight category words on one line.* Run over
`git ls-files '*.md'` at `abf7090`:

| threshold T | files selected |
| ---: | ---: |
| ≥ 2 | 56 |
| ≥ 3 | 20 |
| ≥ 4 | 16 |
| ≥ 5 | 14 |
| ≥ 6 | 13 |
| ≥ 7 | 6 |
| ≥ 8 | 5 |

**There is no threshold that yields the four declared sites.** The failure is not the
expected one (a loose selector that cries wolf); it is that the selector fails in *both*
directions and the two failures do not share a threshold:

**Loose (T ≤ 6) — it selects append-only history.** At every threshold from 2 through 6
the set contains `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/tasks/*.md` and
`docs/decisions/*.md`. These are not accidental prose matches. They are **the literal
six-name list, recorded correctly at the time it was written**:

```text
CHANGES.md:8400            (`state`/`selectors`/`datapath`/`terminals`/`sharing`/`hierarchy`). Lets a
DEVELOPMENT_NOTES.md:3999  (`state`/`selectors`/`datapath`/`terminals`/`sharing`/`hierarchy`), so a future
docs/tasks/COVERAGE-STEERED-GENERATION.md:75
                           KnobId::category() (exhaustive 21-variant match → state/selectors/datapath/terminals/sharing/hierarchy)
```

Under a derived site list the gate would demand that each of these name all eight
categories. There are exactly two ways to satisfy that, and **both are forbidden**:

- **Retro-edit the history.** `CHANGES.md` and `DEVELOPMENT_NOTES.md` are append-only
  and are *never* rewritten (decision `0031`, standing owner directive: *"Keep it raw,
  keep honest, so that people can follow the whole history."*). Task files are layer-B
  history (`MEMORY_ARCHITECTURE.md` §3) and are not retro-edited either. A doctrine check
  that can only be satisfied by rewriting history is a check that pressures an author
  into breaching an absolute doctrine — which is precisely the reasoning decision `0033`
  §1 already recorded for `check_no_boot_volume_refs.sh`'s allow-list.
- **Add an exclusion list.** That is the hand-maintained site list again, inverted, and
  now growth-coupled to the *history* rather than to the docs — strictly worse.

**Tight (T ≥ 7) — it drops sites the pair covers today.** At T = 7 the selector loses
**two of the four declared sites**, `book/src/algorithm.md` and `book/src/knobs.md`,
because their enumerations wrap across two lines:

```text
book/src/knobs.md:1595-1596   `state` / `selectors` / `datapath` / `terminals` / `motifs` / `emission` /
                              `sharing` / `hierarchy`)
book/src/algorithm.md:375-377 knob name (`flop_prob`) or one of the eight coarse categories — `state`,
                              `selectors`, `datapath`, `terminals`, `sharing`, `hierarchy`, `motifs`,
                              `emission` — so one entry can emphasise a whole family.
```

Nobody chose that line break for meaning; it is prose reflow. A tight selector therefore
**silently exempts a site because of where a line happened to wrap** — the same class of
defect as `PARITY-EXTRACTOR-ARM-SHAPE-GAP`, where an extractor encoded a `rustfmt`
formatting assumption and read 7 of 8 categories for its whole life. And even at T = 7
the set *still* contains two task files, so tightening does not buy purity, only loss.

**The selector's one genuine contribution.** Its output is not worthless — it surfaced
`CODEBASE_ANALYSIS.md:2349`, a live doc that names all eight categories on one line and
is **not** a declared pair-4 site, alongside the two chapters `.1` repaired. That is the
correct role for it: a **discovery instrument run at review time**, never the gate's
input. This is decision `0033` (c) restated with a measurement behind it — *the class is
discovered by review and held by derivation.*

### (ii) `covers_set` is vacuous at 3 of its 10 sites — measured

`.1` found a 7/8 miscount at one site. Applying decision `0033` rule (2) — *search from
the authoritative set, never from the shadow you found first* — the authoritative set
here is **every use of the predicate**, not the one instance that surfaced. There are 10:
4 in pair 1b, 2 in pair 3, 4 in pair 4.

The probe is the sharpest available: **delete the enumeration entirely and re-run the
predicate.** A check that still passes with the thing it checks removed is checking
nothing.

| pair | site | ids | enumeration deleted ⇒ |
| --- | --- | --- | --- |
| 3 | `book/src/api-tools.md` | 5 adapters | **VACUOUS — still passes** |
| 3 | `book/src/agent-mcp.md` | 5 adapters | **VACUOUS — still passes** |
| 4 | `book/src/knobs.md` | 8 categories | **VACUOUS — still passes** |
| 4 | `USER_GUIDE.md` | 8 categories | fails on **one** word (`terminals`) |
| 4 | `book/src/algorithm.md` | 8 categories | fails on 3 |
| 4 | `docs/AGENT_INTROSPECTION_SCHEMA.md` | 8 categories | fails on 3 |
| 4 | `book/src/api-introspection.md` | 8 categories | fails on 5 |
| 4 | `book/src/agent-mcp.md` | 8 categories | fails on 3 |
| 1b | `README.md` | 8 doctrines | fails on 7 |
| 1b | `book/src/architecture.md` | 8 doctrines | fails on 8 |
| 1b | `docs/knowledge/doctrine-enforcement.md` | 8 doctrines | fails on 5 |
| 1b | `CODEBASE_ANALYSIS.md` | 8 doctrines | fails on 7 |

**Pair 3 provides zero protection at both of its sites.** Delete the entire downstream
allow-list from `book/src/api-tools.md` and `ENUMERATION-PARITY` stays green, because
`verilator`, `yosys`, `iverilog`, `sv2v` and `slang` are ordinary vocabulary in a chapter
*about running those tools*. This is a live hole in a live gate, found only because the
sweep started from the predicate rather than from the reported symptom.

**The vacuity is predictable, and the predictor is measurable** — occurrences of each id
*outside* its enumeration:

| document | id | occurrences elsewhere |
| --- | --- | ---: |
| `book/src/api-tools.md` | `yosys` | 18 |
| `book/src/api-tools.md` | `verilator` | 12 |
| `book/src/api-tools.md` | `sv2v` / `slang` / `iverilog` | 2 / 2 / 1 |
| `book/src/architecture.md` | `MEMORY-ARCH` | 0 |
| `book/src/architecture.md` | `README-GROWTH` | 0 |
| `book/src/architecture.md` | `KNOWLEDGE-MAP` | 0 |

> **`covers_set`'s strength is inversely proportional to how ordinary its ids are as
> words in the document being checked — and the ids are most ordinary in exactly the
> document that documents them.**

Pair 1b survives on an accident of vocabulary: `MEMORY-ARCH` and `README-GROWTH` are
coined tokens that appear nowhere but the list. Pairs 3 and 4 name real things people
write sentences about. The predicate is therefore weakest precisely where the pair is
most needed, and no amount of adding *sites* fixes it — `.1`'s instinct to ask whether
the predicate itself was right was correct.

### Why a proximity window is not the repair — also measured

The obvious cheap fix is to require all ids inside **K** consecutive lines rather than
anywhere in the file. Every current site has a tight minimal spanning window (1–3 lines
for pairs 3 and 4; 3–20 for pair 1b), so K ≈ 24 would pass the tree today. It was
measured against the two controls that matter and it fails both:

- **Single-id omission** (the realistic drift: a ninth category ships and one site is
  not updated). Removing `motifs` from each pair-4 list moves the window to 159 lines
  (`knobs.md`), 445 (`AGENT_INTROSPECTION_SCHEMA.md`) and *absent* (`api-introspection.md`)
  — caught — but to only **6** lines at `book/src/algorithm.md` and **4** at
  `book/src/agent-mcp.md`, which K = 24 passes. The reason is pointed: the prose that
  *makes those two chapters good* — `.1`'s own sentence explaining that `motifs` rolls
  once per module while `emission` rolls once per candidate gate — sits three lines
  below the list. **Good documentation defeats a proximity heuristic.**
- **Pair 3 is untouched.** With the allow-list line deleted, the minimal window in
  `book/src/api-tools.md` is still **2 lines**, and in `book/src/agent-mcp.md` **2 lines**.
  A window predicate fixes one of the three vacuous sites and leaves both worst ones
  exactly as they are.

A heuristic that repairs a third of the known instances, and whose miss cases are
*written prose quality*, is not a signoff-level answer.

## Decision

### (a) The declared-site list is **authoritative**. It stays hand-written.

Applying decision `0033`'s own three-question rule to the site list itself:

1. **Derivable?** ✓ — a selector over `git ls-files` reaches candidate sites.
2. **Growth-coupled?** **✗ — and this is the whole answer.** The declared-site list is
   *supposed* to differ from "every tracked file that names the ids". The gap is not
   slack; it is the content of the rule: **append-only history must be allowed to record
   the set as it was**, exactly as `check_no_boot_volume_refs.sh`'s allow-list exists so
   a policy document may name the string it forbids and `CHANGES.md` may stay raw. A set
   growing from 8 to 9 categories must *not* require an entry anywhere in
   `CHANGES.md` — it must require the opposite.
3. — (not reached)

Test (2) fails ⇒ **authoritative, leave it hand-written.** It is the third instance of
the same hard case decision `0033` §1 records, and it arrives one level up: at the
enforcement layer rather than the content layer. This is recorded rather than assumed,
because the pair's own comment already *asserted* it (*"a grep for any file mentioning
two category words also matches ordinary prose"*) and an asserted premise is exactly what
this lane keeps finding to be false when measured. Here it measured true — and the real
reason is stronger and different from the asserted one: not "ordinary prose", but
**history that must stay raw**.

### (b) The site list is *reviewed and swept*, not *extended opportunistically*

The tree's Non-Goals forbid *"add the two chapters to pair 4 and stop"* — extending a
hand-maintained list by the sites we happened to find. That prohibition stands, and the
distinction that satisfies it is now available: the candidate list came from **a selector
run over the whole tree from the authoritative set**, and every one of its 20 candidates
was classified. That is a swept-and-reviewed list, not an opportunistic one. The
procedure — not the outcome — is what makes it legitimate, so it is written down as the
procedure for adding any future site:

> **To add or audit a declared site: run the discovery selector over the whole tree from
> the authoritative set, classify every candidate as *live doc* (declare it) or *history
> / incidental prose* (do not), and record the classification.** Never add a site because
> it turned up in a bug report.

Applying it now yields three live docs that name the taxonomy and are undeclared:
`book/src/agent-mcp.md`, `book/src/api-introspection.md` (both repaired by `.1`) and
`CODEBASE_ANALYSIS.md`. Pair 4 goes from 4 declared sites to 7 in `.3`.

### (c) `covers_set` must be scoped to the **enumeration**, by an explicit fence

The predicate becomes: *inside the region a document marks as its enumeration of set S,
the members named are exactly S.* Marked with an HTML-comment fence, invisible in
rendered Markdown and in the mdBook:

```markdown
<!-- enumeration: steer-categories -->
`state` / `selectors` / `datapath` / `terminals` / `sharing` /
`hierarchy` / `motifs` / `emission`
<!-- /enumeration -->
```

Four properties, each load-bearing:

- **Exact, not heuristic.** No thresholds, no windows, no line numbers, and no
  sensitivity to where prose reflows. It reads the thing the pair claims to check.
- **The fence is not a shadow** under decision `0033` (a). It names **no members**, so
  growing S from 8 to 9 never requires touching a fence — test (2) fails, as it must for
  any repair that is not to become the next defect.
- **Bidirectional parity becomes correct.** `covers_set` is one-directional today on the
  stated grounds that *"a chapter may legitimately name a subset in an example"*. With a
  fence that concern dissolves — examples live outside the fence — so the fenced region
  can be held to exact parity in both directions, catching an id that no longer exists
  as well as one that was never added.
- **A missing fence is a hard failure.** A declared site with no fence fails the check
  loudly. That is what stops the repair from degrading back into whole-file matching the
  first time someone edits a chapter.

Rung: this is **R4** (a registered doctrine check) sharpened, not a new rung. The check
already exists; what changes is that it stops passing vacuously.

### (d) Scope split

`.2` is this decision. **`.3`** implements it: fence the enumerations at every declared
site, rewrite `covers_set` to read the fenced region, declare the three reviewed sites,
and negative-control every pair in both directions — including re-running the vacuity
probe, which must now **fail** at all three sites where it currently passes.

## Decisive test applied

*"Delete the enumeration a pair exists to guard. Does the pair fail?"* Today: **no** at
`book/src/api-tools.md`, `book/src/agent-mcp.md` (pair 3) and `book/src/knobs.md` (pair
4). That question is the whole bar, it is cheap to ask of any coverage check, and it
should be asked of every one that is ever added. A check that passes with its subject
removed is manufacturing exactly the confidence decision `0033` §4 calls the uniquely
costly failure.

## Rejected alternatives

- **Derive the declared-site list from a threshold selector.** Rejected on measurement,
  not on taste: no threshold reproduces the declared sites; T ≤ 6 pulls in append-only
  history and would make the gate satisfiable only by rewriting it (absolutely forbidden,
  decision `0031`) or by a second hand-maintained exclusion list; T ≥ 7 drops
  `book/src/algorithm.md` and `book/src/knobs.md` because their lists wrap across two
  lines, silently exempting a site on a reflow accident.
- **Derive the site list and add an exclusion list for history.** Rejected — it is the
  hand-maintained list with its sign flipped, and it becomes growth-coupled to
  `CHANGES.md`, which grows on every commit.
- **A proximity window (all ids within K consecutive lines).** Rejected on measurement.
  It leaves both pair-3 sites exactly as vacuous (window = 2 lines with the allow-list
  deleted) and misses a single-id omission at two pair-4 sites (6 and 4 lines) because
  explanatory prose about the categories sits beside the list. Its miss cases are
  correlated with documentation *quality*, which is the wrong thing for a gate to punish
  or reward.
- **Require the whole enumeration on one line.** Rejected — 2 of the 4 current sites
  wrap, and forcing them not to would make a doctrine dictate prose line breaks. A gate
  that constrains rendering to remain checkable has inverted the relationship.
- **Add the two chapters to pair 4 and stop.** Rejected by the tree's own Non-Goals, and
  independently insufficient: `.1` measured that doing so would have caught
  `agent-mcp.md` outright but only partially diagnosed `api-introspection.md`, and would
  have left all three vacuous sites vacuous.
- **Machine-readable front-matter or a sidecar data file listing each site's members.**
  Rejected — that is a *copy of the set* in a new format, i.e. a fresh shadow, violating
  decision `0033`'s binding constraint that a repair may not introduce a new
  hand-maintained list. The fence deliberately carries no members.
- **Drop pair 3 as unfixable / low value.** Rejected — `feedback_never_retire_strategies`,
  and the measurement argues the opposite way: it is the pair with *zero* current
  protection, so it is the one with the most to gain.
- **Fix `covers_set` in `.2` alongside the decision.** Rejected — `.2`'s question was
  whether the predicate is right, and one leaf lands one thing (`COMMIT.md` task-tree
  rule 3). The fence touches 9 files and needs its own negative-control pass.

## Consequences

- `ENUMERATION-PARITY` gains a **stated, measured limit** rather than an assumed
  strength: 3 of its 10 coverage sites currently pass vacuously, and this is recorded in
  `DOCTRINE_ENFORCEMENT.md` §9 (honest limits) until `.3` removes it. A gate whose
  weakness is documented is worth more than one whose strength is assumed.
- The **declared-site list joins the repo's short list of authoritative hand-written
  lists**, for a reason that is now measured: history must stay raw. Decision `0033`
  §1's four hard cases become five, and the fifth is the enforcement layer itself.
- The **vacuity probe** — *delete the subject, re-run the check* — is adopted as the
  standard acceptance test for any coverage-shaped doctrine check, alongside the existing
  both-directions negative control.
- The **discovery selector** is adopted as a review instrument and explicitly barred from
  being a gate input, giving decision `0033` (c)'s *"discovered by review, held by
  derivation"* a concrete procedure.
- Pair 4 grows from 4 to 7 declared sites in `.3`; pair 3 keeps its 2 sites but gains a
  predicate that does something.
- Docs-only leaf: no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs`
  untouched.

## Open questions (for `.3`)

- Whether the fence marker carries the **set id** (`<!-- enumeration: steer-categories -->`)
  or is anonymous. Carrying the id lets one file fence two different enumerations —
  `book/src/agent-mcp.md` is a declared site for *both* pair 3 and pair 4 — so it is
  almost certainly required; `.3` confirms against the real files.
- Whether an unfenced *undeclared* file that contains a fence should be an error
  (a fence nobody checks) — the symmetric meta-check, and cheap.
- Whether pair 1b's four sites need fences too. Their ids are coined tokens so they are
  near-exact today, but "near-exact by accident of vocabulary" is not a property to rely
  on: `CODEBASE_ANALYSIS.md` already fails the probe by only 7 of 8, meaning one doctrine
  id does appear outside the list.

## Links

- Tree: [`docs/tasks/LIVE-DOC-REGISTRY-SHADOWS.md`](../tasks/LIVE-DOC-REGISTRY-SHADOWS.md)
  (`.1` repaired the three sites and sharpened the question; this leaf is `.2`; `.3`
  implements).
- Generalizes: decision [`0033`](0033-shadow-enumeration-classification.md) — the
  three-question rule (applied here to the *site list*), the repair ladder, rule (2)
  (*search from the authoritative set*, applied here to *every use of the predicate*),
  and (c)'s discovered-by-review verdict.
- Constrained by: decision [`0031`](0031-ssd-volume-exclusivity.md) — history stays raw,
  which is what makes a derived site list unsatisfiable.
- Precedent for the failure mode: `PARITY-EXTRACTOR-ARM-SHAPE-GAP.1` — an extractor that
  encoded a formatting assumption and read 7 of 8 for its whole life; a threshold
  selector would re-import that hazard on the doc side.
- Standards: `DOCTRINE_ENFORCEMENT.md` §3 (check archetypes), §4 (the check contract
  `.3` must obey), §9 (honest limits — where the vacuity is recorded), §11 (a doctrine
  with no working check is a suggestion).
- Touch points for `.3`: `scripts/check_enumeration_parity.sh`, `book/src/knobs.md`,
  `book/src/algorithm.md`, `book/src/agent-mcp.md`, `book/src/api-introspection.md`,
  `book/src/api-tools.md`, `USER_GUIDE.md`, `docs/AGENT_INTROSPECTION_SCHEMA.md`,
  `CODEBASE_ANALYSIS.md`.
