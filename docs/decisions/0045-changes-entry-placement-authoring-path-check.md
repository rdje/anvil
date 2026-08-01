---
id: changes-entry-placement-authoring-path-check
title: Entry placement DOES warrant a mechanism, and it must key on the AUTHORING PATH — "if the staged CHANGES.md diff adds a `## ` heading, the file's first heading must be one of the added lines" — measured 3 fires / 0 false alarms over 766 commits, and it found a third misplaced entry nobody knew about
answers:
  - "should CHANGES.md entry placement be gated"
  - "how do I check that a CHANGES.md entry was added at the top"
  - "why is a date-keyed CHANGES.md ordering check a bad idea"
  - "why is a hash-keyed CHANGES.md ordering check vacuous"
  - "what is the authoring-path check for CHANGES.md"
  - "how many CHANGES.md entries are misplaced"
  - "should the 181 'this commit' provenance lines be backfilled"
  - "why does a Landed as line say this commit"
  - "why can the newest CHANGES.md entry never carry its own hash"
  - "how do I count CHANGES.md provenance lines correctly"
date: 2026-08-01
status: accepted
tags: [changes-md, live-docs, doctrine, mechanism, measurement, shadow, authoring-path, evidence]
reverify: "git --no-pager log --format=%h --follow -- CHANGES.md | while read c; do a=$(git --no-pager show $c -U0 -- CHANGES.md | grep -E '^\\+## ' | sed 's/^\\+//'); [ -z \"$a\" ] && continue; f=$(git --no-pager show $c:CHANGES.md | grep -m1 -E '^## '); printf '%s\\n' \"$a\" | grep -qxF \"$f\" || echo \"FIRE $c\"; done   # expect exactly 3: f9cf50a, 715019b, abf7090"
evidence: >
  Measured 2026-08-01 at 928817f, on the real repository history.
  (a) THE CANDIDATE, run over every commit that touches CHANGES.md — 766 commits: 664 ok,
  99 correctly skipped (they add no entry; hash backfills and typo fixes), and 3 fires.
  ALL THREE FIRES ARE TRUE POSITIVES. Two are the known offenders this tree opened on
  (715019b BOOK-TEST-COUNT-SHADOWS.2, abf7090 LIVE-DOC-REGISTRY-SHADOWS.1). The third,
  f9cf50a (RESOURCE-SAFE-TOOLING.2, 2026-06-14), was PREVIOUSLY UNKNOWN: its entry landed
  at heading 6 of 379 — five entries below the top — and no leaf of this tree had found it.
  Zero false alarms.
  (b) WHY THE EXISTING ORACLE MISSED IT, and why that is systematic rather than luck: all
  three misplaced entries are invisible to a hash-keyed ordering scan. f9cf50a's entry says
  `**Landed as:** this commit`; the other two carry no `Landed as:` line at all. The stale
  template that misplaces an entry is the SAME one that omits its provenance, so the hash
  oracle is blind to exactly the population it was built to detect — 0 of 3, measured.
  (c) NEGATIVE CONTROLS, both directions, on the staged path: the same new entry placed at
  the top => silent; appended at the bottom => FIRES; a provenance-only hash backfill with
  no new heading => correctly SKIPS.
  (d) THE PROVENANCE CENSUS, re-derived with ONE per-heading classifier run over two commit
  points, so instrument change is separated from corpus change:
  at 80edd42 (.3) — 652 headings: 392 hash, 182 "this commit", 77 none, 1 other.
  at 928817f (HEAD) — 678 headings: 412 hash, 181 "this commit", 77 none, 7 "pending", 1 other.
---

# The mechanism question, answered on measurement

`CHANGES-ENTRY-PLACEMENT.4` asked whether entry **placement** warrants a mechanism. It does,
and the design that works is the one decision [`0038`](0038-changes-md-position-repair-by-pointer.md)
named but could not yet evaluate: key the check on the **authoring path**, not on the file's
content.

## The check

> **If a commit's staged `CHANGES.md` diff adds at least one `## ` heading, then the first
> `## ` heading in the resulting file must be one of those added lines.**

That is the whole predicate. It needs **no date**, **no hash**, and no knowledge of the
file's ordering convention — it reads the diff and the resulting file, both of which `git`
already has.

## Why decision `0033`'s three-part test does not decide this

`.4`'s acceptance requires applying the three-question rule *first*. Applied honestly, **it
does not apply**: that rule classifies a **hand-maintained list `L` mirroring a set `S`**, and
asks whether `L` is derivable, growth-coupled, and silent. There is no list here. Entry
placement is a property of an *authoring act*, not a duplicated enumeration.

Recording that rather than forcing a fit, because a rule stretched past its subject is how a
framework starts producing confident wrong answers. The rule that *does* govern is
`DOCTRINE_ENFORCEMENT.md` §4's contract for any new check (deterministic, negative-controlled
both ways, non-vacuous), and the check above is evaluated against it below.

## The two candidates that were already dead, and why this one is not

`0038` and `.3` disqualified two designs **on measurement**, and both failures are instructive
because they are opposite:

| candidate | verdict | measured failure |
| --- | --- | --- |
| **date-keyed** ordering scan | cries wolf | 3 findings, **2 false** — mis-dated headings sitting above correctly-ordered entries |
| **hash-keyed** ordering scan | vacuous | **0 of 3** real offenders visible; the offending entries carry no resolvable hash |
| **authoring-path** (this one) | **ship it** | **3 fires / 766 commits, 3 true positives, 0 false alarms** |

The hash-keyed failure is the deep one, and the third instance sharpens it from an accident
into a rule: **the defect and the blind spot share a cause.** A stale entry template both
misplaces the entry *and* omits its `Landed as:` line, so any check that needs the provenance
line to work is guaranteed to miss precisely the entries that are broken. **A detector must
not depend on a field that the defect it detects also destroys.**

## What it found that nobody had

Running it over history surfaced **`f9cf50a`** (`RESOURCE-SAFE-TOOLING.2`, `2026-06-14`),
whose entry landed at heading **6 of 379** — five entries below the top. `.1`, `.2` and `.3`
all reported the class as **two** members; it is **three**. The tree's own git-order oracle
reported *zero* violations over 388 hash-bearing entries and was right about the population it
could see — which was, for this class, none of it.

The instance is milder than the two known ones (five entries down, not 43,000 lines down), and
that is the point: a check keyed on the authoring act catches the **mild** case too, and the
mild case is the one a human reviewer will never notice.

## The scope trap, handled

Both original offenders were **docs-only** commits, and
`scripts/check_diagnosis_evidence.sh` is scope-aware — it exempts them outright. This check is
**not** scope-aware by design: it triggers on the staged `CHANGES.md` diff regardless of what
else the commit touches, because the property has nothing to do with whether the commit
changed code.

## What this does NOT license

- **It does not check ordering.** It checks that a *newly added* entry is at the top. An edit
  to an old entry, a hash backfill, or a typo fix adds no heading and is correctly silent
  (99 of 766 commits took this path).
- **It does not require a `Landed as:` hash**, and must never be extended to. The newest entry
  **structurally cannot** carry its own hash — the commit does not exist while its message is
  being written — so a check demanding one would be permanently red by at least one entry.
  The 7 `pending` placeholders at HEAD are that tail plus its normal backlog.
- **It does not authorise moving or rewriting any landed entry.** `0038`'s ruling stands
  unchanged: position is itself a record, and the repair for a *past* misplacement is an
  additive pointer stub.

## Ruling on the 181 `this commit` lines: leave them

`.3` handed `.4` an explicit question — back-fill the unfinished provenance lines, or not.
**Do not.** Three reasons, in order of weight:

1. **They are frozen, not bleeding.** Measured with one classifier at two commit points, the
   count went **182 → 181** across 26 new entries. New work is not producing them; this is a
   legacy block from a retired template. A backfill would be a 181-entry history rewrite to
   fix a defect that stopped happening.
2. **`0038` §(d)(5) already refused the adjacent act** for the 73 oldest entries, and the
   argument transfers: `CHANGES.md` is append-only by owner directive (`0031`), and *"keep it
   raw, keep honest"* means a reader following the whole history must be able to see that the
   template was once incomplete.
3. **The value is low and the risk is not.** Each backfill requires resolving an entry to a
   commit, which for these entries means inference — precisely the guessing that produced two
   unreproducible counts in this tree's own history.

The forward-going half is already handled: new entries carry a hash or a `pending` that the
next commit backfills.

## A measurement discipline this tree keeps re-learning

Three counts in this tree did not reproduce when re-derived: `.2`'s `388`, `.3`'s `269`, and —
found here — the shape of the `181`. The last one is the clearest: **the same corpus yields
181, 202 or 233 depending on the extractor**, and only a *per-heading* classifier (walk `## `
headings; take the first `**Landed as:**` within each section; classify its first token) gives
the number that means anything. A line-anchored `grep` requiring backticks reports 0, because
the legacy form is unbackticked.

`.3` demanded that any proposed mechanism *"specify its extractor precisely enough to re-run"*.
This record does: the predicate is stated above in one sentence, and the front-matter
`reverify` is the whole scan as a runnable command, expected to print exactly three commits.

## Consequence

`CHANGES-ENTRY-PLACEMENT.5` implements this as a registered doctrine under
`DOCTRINE_ENFORCEMENT.md` §4: a script, a `DOCTRINES` registry row, negative controls both
ways, and the `ENUMERATION-PARITY`-gated doctrine-id list updated at every fenced site. The
prototype and its three controls in the Evidence block are what `.5` must reproduce, not
re-derive.
