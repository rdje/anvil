---
id: sweep-exemption-past-vs-present-and-recorded-recall
title: A sweep's exemption is keyed on whether the enclosing record speaks about the **past or the present** — never on whether it is **dated** — and a sweep must record its **match count**, not only its finds, or its recall is unauditable
answers:
  - "what is exempt from a shadow-enumeration sweep"
  - "is a dated claim history and therefore exempt from a sweep"
  - "why did a dated test count survive the sweep built to catch it"
  - "how do I decide whether a doc claim is a standing claim or a historical record"
  - "how should a sweep record its result so its coverage can be audited"
  - "why is reporting finds without reporting candidates not enough"
  - "how do I tell whether a sweep had good recall or just got lucky"
  - "what must a verification log say about a sweep"
  - "does a self-declared Last updated field detect a stale standing claim"
  - "should a staleness instrument be shared between an inline dated claim and a file freshness field"
  - "why can a staleness check not be applied to a closed task tree"
  - "why is the past-versus-present test not mechanizable as a gate"
date: 2026-07-31
status: accepted
tags: [doctrine, sweep, audit, recall, exemption, shadow-list, staleness, docs, gate-quality, method, north-star]
reverify: "git ls-files '*.md' | grep -vE '^(CHANGES\\.md|DEVELOPMENT_NOTES\\.md|docs/tasks/|docs/TASK_TREE\\.md|docs/evidence/|docs/decisions/)' | xargs grep -lEi '^[*_> -]*(last[ -]updated|updated on|as of|snapshot date|generated on)[*_ ]*:' | wc -l  -> 0 live-doc self-declared freshness fields; the same pattern over docs/tasks/*.md returns 75 files (60 done/closed + 14 active + 1 TEMPLATE), which is the negative control proving the 0 is real"
evidence: docs/tasks/DATED-COUNT-SWEEP-EXEMPTION.md (.1 the measurement and the refuted first hypothesis; .2 the 14-match enumeration and the two-copies-disagreed clincher); docs/tasks/BOOK-TEST-COUNT-SHADOWS.md (.1's verification log — the sweep that reported finds without candidates); docs/decisions/0033-shadow-enumeration-classification.md (the classification rule this record leaves untouched and the "discovered by review" verdict it supplies conduct rules for); docs/decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md (delete-the-subject vacuity test; sweep from the authoritative set); docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md §6 finding 2 (the self-declared-date candidate instrument this record measures against). All counts re-derived 2026-07-31 at e954fe8.
---

# 0039 - Sweep conduct: exemption is keyed on past-vs-present, and recall must be recorded

- Date: 2026-07-31
- Status: accepted
- Tree: `DATED-COUNT-SWEEP-EXEMPTION.3` (the leaf that makes `.1`/`.2`'s two measured rules durable)
- Activated by: autonomous PNT selection under the owner's standing **DECIDE, DON'T ASK** directive
- **Refines the practice of, and does not amend or supersede,**
  [`0033`](0033-shadow-enumeration-classification.md)

## Context

[`0033`](0033-shadow-enumeration-classification.md) settled **what a shadow enumeration
is** — the three-question rule — and closed with an honest verdict: *"this class is
discovered by review and held by derivation."* It wrote down the derivation half in full
(the four-rung repair ladder, the `ENUMERATION-PARITY` doctrine). It wrote down **nothing
about how the review half is conducted**, and two sweeps later that omission has now
produced two measured misses of different shapes.

Both were measured by `DATED-COUNT-SWEEP-EXEMPTION`, whose `.1` and `.2` leaves hold the
full numbers. Restated here only as far as this record's rules need:

**Failure mode A — an exemption keyed on the wrong property.**
`BOOK-TEST-COUNT-SHADOWS.1` swept 94 live-doc files, deleted five per-file test counts from
`book/src/architecture.md`, and **exempted every dated one**. Its recorded reasoning was
that *"banked-run citations are dated measurements of a specific run and are exempt by the
same reasoning `.1` used for append-only history."* That reasoning is correct for a
`CHANGES.md` entry or a banked evidence citation. Applied as *"is it dated?"* it exempted
two claims that were **standing claims wearing a date**:

| site | text | claimed | actual | |
| --- | --- | ---: | ---: | --- |
| `book/src/architecture.md:612` | *"294 passing tests (**current HEAD**, `cargo test` on 2026-04-30)"* | 294 | **946** | 31 % of reality |
| `CODEBASE_ANALYSIS.md:2425` | *"**Current** executed counts … 307 (`cargo test`, 2026-05-02)"* | 307 | **946** | 32 % of reality |

Both answer *the present* **in their own words** while carrying a date two and three months
stale. They are self-refuting on their own terms, and the date is the whole discriminator:
re-measured after the repair, **zero** undated counts survived in the file `BOOK-TEST-COUNT-SHADOWS.1`
actually repaired.

**Failure mode B — a sweep that reported precision and never recall.**
The date explains **nothing** in `CODEBASE_ANALYSIS.md`, where *undated* counts survived
too. That file was demonstrably **in scope** — the sweep's own log rejects its *"all 7
categories"* as a false positive, with a reason — so it was **judged, not enumerated**.
Enumerating it against that sweep's **own key** returns **14** matches where `.1` had
registered **3**; all 13 per-file claims were then measured, **9 stale, every error an
under-count**, the worst being `src/bin/tool_matrix.rs` at **26 → 114**. The sweep's log
recorded *"found 2 more live shadows and three false positives"* — **precision reported,
recall never** — so a reader cannot tell whether it examined 5 candidates or 500. Its
coverage claim was **unfalsifiable**.

**The evidence that settles it is not an argument.** For `src/metrics.rs` the book said
**18**, `CODEBASE_ANALYSIS.md` said **20**, the truth is **31**. One derivable number, two
copies, rotted to two *different* wrong values. And **4 of the 13 were accidentally
correct** — the coincidence `0033` predicts, and the reason a number that can become right
by accident carries no information.

## Decision

### (a) The exemption rule

> **A sweep exempts a hit when the enclosing record is a statement about the PAST. It never
> exempts a hit for carrying a DATE.** A date is not evidence of pastness — a standing claim
> with a date on it is still a standing claim, and the date makes it *look* like history
> precisely to the sweep built to catch it.
>
> The test is one question asked of the **enclosing record**, not of the numeral:
> *does this record describe what was true at a named moment, or what is true now?*
>
> - **About the past ⇒ exempt.** A `CHANGES.md` entry; a `DEVELOPMENT_NOTES.md` note; a
>   task-tree verification log; a decision record stating what was decided; a banked
>   evidence citation naming a run. Each is *correct* precisely because it is frozen, and
>   editing it would breach [`0031`](0031-ssd-volume-exclusivity.md).
> - **About the present ⇒ in scope, dated or not.** A live doc's snapshot section; a book
>   chapter's description of the code as it is; anything the record itself calls *current*.
>
> Where the record says which it is, **believe the record over the date.** Both survivors
> said *"current"*.

The rule is stated as a property of the **enclosing record** rather than of the line,
because that is the level at which the answer is stable: a numeral is ambiguous in
isolation, whereas *"what is this file for"* is answerable once per file and reusable across
every hit in it. It also makes the by-kind exclusion list derivable from the rule instead of
from habit — `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/tasks/`, `docs/TASK_TREE.md`,
`docs/evidence/` and `docs/decisions/` are exempt **because they are records of the past**,
not because someone listed them.

### (b) The recall rule

> **A sweep records its MATCH COUNT, not only its finds.** The verification log must state:
> the **authoritative set** swept (and its size), the **key** used verbatim, the number of
> **candidates the key matched**, and the disposition of every candidate — repaired,
> rejected with a reason, or classified out by (a). Reporting finds alone reports
> *precision* and leaves *recall* unmeasurable, which makes the coverage claim unfalsifiable.

The denominator is the whole point. *"Found 2 shadows and 3 false positives"* is compatible
with a key that matched 5 candidates and with one that matched 500; only the second is a
miss, and nothing in the log distinguishes them. Recording the match count costs one line
and converts an unfalsifiable claim into a checkable one.

This is the exact sibling of `CHANGES-ENTRY-PLACEMENT.3`'s finding two commits earlier — an
extractor whose *pattern* is unrecorded is not reproducible; here it is an extractor whose
*match set* is unrecorded. Two trees, two days, one shape: **an instrument is only as
trustworthy as what it records about itself; a number without its method is a rumour.**

**Corollary, earned inside this leaf.** *Print the bucket you could not classify.* Writing
this record's own measurement, a status extractor whose `sed` script began with a greedy
``.*`` before the capture group returned `surelog` as a tree's status, because that `.*`
took the **last** backticked token on the line rather than the first after `Status:`.
It was caught only because the sweep **printed its unclassified bucket** instead of silently
binning it — rule (b) in miniature, and the third instance of this repo's standing gotcha
that *an extractor must die on a missing field, never fall through to something plausible.*

### (c) These are review rules, not a gate — and that is deliberate

No check is registered for either rule, for two independent reasons, both already settled in
this repo:

1. `0033` §(c) established that test (1) of the classification rule is a **semantic**
   relation, so the class is not mechanizable as discovery; *"is this record about the past
   or the present?"* is the same kind of question one level up. A guessing gate has only two
   failure modes — miss (false confidence) or cry wolf — and *a gate that cries wolf gets
   deleted, taking its real coverage with it.*
2. `DATED-COUNT-SWEEP-EXEMPTION`'s Non-Goals already bind this leaf: `0033` records that
   gating a redundant count *"keeps the shadow alive and spends a mechanism on it forever."*
   Rule (b) in particular is a rule about **what a human or agent writes in a log**; a gate
   that demanded a match-count line would check the line's presence, never its truth.

What is mechanized instead is the thing that *is* structural: `ENUMERATION-PARITY` holds the
declared pairs, and `0037`'s *delete-the-subject* probe proves a coverage check non-vacuous.
Rules (a) and (b) govern the review that finds the pairs in the first place.

### (d) Coordination with `OVERFLOW-DESTINATION-INSTRUMENTATION`'s candidate instrument (b) — measured, and the answer is "precondition", not "duplicate"

`.2` flagged a coordination risk: `OVERFLOW-DESTINATION-INSTRUMENTATION.1` §6 finding 2
proposes *a self-declared date that disagrees with the file's newest content* as a staleness
instrument, and two trees building one instrument would itself be a `0033` shadow. Measured
rather than argued, at `e954fe8`:

| measurement | result |
| --- | ---: |
| file-level self-declared freshness field in either survivor file | **0** |
| the same field across the 108-file live-doc set | **0** |
| the same field across `docs/tasks/*.md` (the excluded set — negative control) | **75 files / 76 lines** |
| of those 75: tree status `done` / `closed` | **60** |
| of those 75: tree status `active` | **14** |
| of those 75: `TEMPLATE.md` (`proposed`) | **1** |
| the same pattern on a synthetic line appended to a live-doc copy | **1** |

Three conclusions, none of which was available without measuring:

1. **Instrument (b) could not have caught this class.** Its subject is a *file-level
   freshness field* — one date claiming to describe a whole file, against which the file's
   newest content is a derivable baseline. This class's dates are *inline parentheticals
   inside one sentence*, for which there is no "newest content" to compare against. Neither
   survivor carried a freshness field at all. There is therefore **no duplicated mechanism
   to avoid**, and neither tree blocks the other.
2. **Instrument (b) has zero subjects in ANVIL's live docs.** Every one of its 75 real
   subjects is a task-tree file. The zero is not vacuous: the same pattern returns 75 files
   over the excluded set and fires on a synthetic insert.
3. **Rule (a) is a precondition for instrument (b), not a competitor to it.** **60 of the 74
   real trees are `done` or `closed`**, where the `Last updated:` field is a statement about
   the past and is *correctly* frozen; only the **14 active** ones make a claim about the
   present. Applied without rule (a), a mechanical staleness comparison would cry wolf on
   **60 of 74** — which is `0033` **test (2)** exactly, and the failure mode that gets a gate
   deleted. `OVERFLOW-DESTINATION-INSTRUMENTATION.2` inherits this measurement: if it adopts
   instrument (b), it must scope it to records that speak about the present.

### (e) Where this rule lives, and why there is no separate fact card

This record is the single canonical home. `docs/decisions/` is a Knowledge Map scan
directory, so this record's `answers:` keys are indexed directly and a sweep author asking
*"what is exempt from a shadow sweep?"* is routed here in one lookup. Writing an additional
`docs/knowledge/` card would create a second copy of a rule whose entire subject is that
second copies rot — `0033` R1 applied to this leaf's own output. `0033` gains a **dated
pointer**, not a restatement, so the classification decision routes a reader to the conduct
rules without holding a copy of them.

## Decisive test applied

*"Would these two rules, applied by `BOOK-TEST-COUNT-SHADOWS.1`, have caught what it
missed?"*

Rule (a) catches failure mode A directly: both survivors say *"current"* in their own text,
so the enclosing record speaks about the present and the date buys no exemption. Rule (b)
catches failure mode B without needing anyone to have been more careful: a log required to
state *"key matched 14 candidates in `CODEBASE_ANALYSIS.md`; 2 repaired"* is self-evidently
incomplete on its face, whereas *"found 2 more live shadows"* reads like success. That is the
bar — the second rule has to work when the sweeper's judgement is the thing that failed.

## What this decision does NOT license

Stated explicitly so it cannot be cited later to justify a sweep:

1. **It does not license editing any record that speaks about the past.** `CHANGES.md`,
   `DEVELOPMENT_NOTES.md`, task-tree logs and landed decision records stay raw
   ([`0031`](0031-ssd-volume-exclusivity.md), [`0038`](0038-changes-md-position-repair-by-pointer.md)).
   Rule (a) *widens* what is in scope for a sweep of **live** docs; it narrows nothing about
   history and grants no new permission over it.
2. **It does not amend `0033`.** The three-question classification rule, the four-rung repair
   ladder, and the four hard cases are unchanged. This record governs the **conduct** of the
   review `0033` §(c) says the class requires; it does not change what a shadow *is*.
3. **It does not license adding a gate for either rule.** §(c).
4. **It does not license re-auditing `BOOK-TEST-COUNT-SHADOWS`'s closed leaves.** They are
   correct history; a closed tree's too-strong claim is repaired by a new tree that measures
   it — the precedent that tree itself set over `SHADOW-ENUMERATION-SWEEP`.
5. **It does not license treating a recorded match count as proof of correctness.** It makes
   recall *auditable*; it does not make it *good*. A key that matches nothing still reports
   zero, which is why `0037`'s delete-the-subject probe stays the separate acceptance test
   for any check, and why a count floor must be the real count rather than a safe lower one.

## Rejected alternatives

- **Amend `0033` in place with the two rules.** Rejected. `0033`'s subject is *what a list
  is*; these are rules for *how a sweep is run and recorded*, and they apply to sweeps that
  have nothing to do with enumerations (the `/tmp` sweep that damaged `0030`'s `reverify`
  failed rule (a)'s test too — it rewrote the policy documents whose *subject* was the string
  being rewritten). Folding conduct into classification would bury a general rule inside a
  specific one, and `0033` is `accepted`, so a substantive addition is a new record or a
  supersession, never an edit ([`0038`](0038-changes-md-position-repair-by-pointer.md)'s
  boundary).
- **Write a `docs/knowledge/` fact card instead of, or in addition to, this record.**
  Rejected — §(e). The Knowledge Map indexes `docs/decisions/` directly, so a card would be a
  second copy with no retrieval gain. (This differs from `coverage-check-vacuity`, which
  carries a *portable method* generalised out of a project-specific enforcement decision;
  here the record **is** the portable method, with no project-specific residue to leave
  behind.)
- **Key the exemption on "does the record name a specific run?"** Rejected — it is the same
  mistake one step further out. `book/src/architecture.md:612` named a specific `cargo test`
  run *and* called its result *"current HEAD"*. Naming a run is what a standing claim does
  when it wants to sound like evidence.
- **Require a sweep to enumerate every candidate in its log, not just count them.**
  Rejected — a 94-file sweep would produce a log longer than the repair, and a log nobody
  reads is not an audit trail. The count is the falsifiable part; the dispositions of the
  *rejected* candidates are already required by rule (b) and are few by construction.
- **Adopt `OVERFLOW-DESTINATION-INSTRUMENTATION`'s instrument (b) as this tree's mechanism.**
  Rejected on measurement — §(d). It has zero subjects in the live-doc set and could not have
  fired on either survivor, so adopting it would have been a mechanism aimed at a class it
  does not cover.
- **Do nothing, on the grounds that `.2` already deleted the offending lines.** Rejected —
  `.2` closed two instances of a class whose *cause* was a recorded, reusable, and wrong
  heuristic. The next sweep inherits the heuristic, not the deletions. This is the leaf that
  stops the class recurring, and the tree was split at `.1` precisely so `.2` could not be
  mistaken for completion.

## Consequences

- The **review half** of `0033` §(c) now has written conduct rules, so a sweep is
  reproducible and its coverage claim is falsifiable. Before this record, `0033` specified
  the derivation half completely and the review half not at all — which is where both
  measured misses landed.
- **Every verification log for a sweep gains a required field** (the match count). It costs
  one line and it is the line that would have exposed both failures.
- `OVERFLOW-DESTINATION-INSTRUMENTATION.2` inherits a measurement it would otherwise have had
  to take: instrument (b) has **0** subjects in the live-doc set and **75** in `docs/tasks/`,
  **60** of which are correctly-frozen closed trees. If that tree adopts the instrument, rule
  (a) is what keeps it from crying wolf on 60 of 74 files.
- The by-kind exclusion list used by live-doc sweeps is now **derived from a stated rule**
  rather than carried by habit, and it gains `docs/TASK_TREE.md` — the task-tree *index* is
  layer-B history of the same kind as `docs/tasks/`, and `.2` swept it and hand-checked its
  two hits rather than exempting them by kind.
- `0033` is **refined in practice and unchanged in substance**; it gains a dated pointer
  Amendment only.
- Docs-only record: no `src/` change ⇒ **DUT byte-identical**, `tests/snapshots.rs` untouched.

## Links

- Tree: [`docs/tasks/DATED-COUNT-SWEEP-EXEMPTION.md`](../tasks/DATED-COUNT-SWEEP-EXEMPTION.md)
  (`.1` measured and registered; `.2` repaired at R1 and found the class 5× larger than
  registered; this leaf is `.3`).
- Refines the practice of: [`0033`](0033-shadow-enumeration-classification.md) — the
  classification rule, the repair ladder, and the *discovered by review, held by derivation*
  verdict this record supplies the review rules for.
- Method precedents: [`0037`](0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md)
  (*delete the subject and re-run the check*; *sweep from the authoritative set and classify
  every candidate — never add a site because it turned up in a bug report*),
  [`0038`](0038-changes-md-position-repair-by-pointer.md) (an instrument whose method is
  unrecorded is not reproducible — the sibling finding two commits earlier).
- Governing doctrine: [`0031`](0031-ssd-volume-exclusivity.md) — history stays raw, which is
  *why* records about the past are exempt and why rule (a) must not be read as licence to
  tidy them.
- Coordination: [`docs/tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md`](../tasks/OVERFLOW-DESTINATION-INSTRUMENTATION.md)
  §6 finding 2 — the self-declared-date candidate instrument, measured against this class in
  §(d).
- Standards: `MEMORY_ARCHITECTURE.md` §3 (layer C: append once, supersede, never silently
  rewrite), `DOCTRINE_ENFORCEMENT.md` §9 (honest limits — the basis for the §(c) no-gate
  verdict) and §6.1 (a box is earned, not ticked).
