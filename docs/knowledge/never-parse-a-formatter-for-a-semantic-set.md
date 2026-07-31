---
id: never-parse-a-formatter-for-a-semantic-set
title: Never parse a **formatter's** output for a semantic set — an extractor read 7 of 8 steering categories for its whole life because `rustfmt` wrapped the one match arm whose pattern got long enough, and it degrades exactly when the set **grows**
answers:
  - "why did my extractor silently miss one enum variant"
  - "is it safe to grep rustfmt output for a match arm"
  - "why did a doctrine check pass while a category was missing"
  - "what count floor should an extractor have"
  - "why is a one-directional coverage check a silent exemption"
  - "does grep -E want {0,1} or escaped braces"
  - "difference between ERE and BRE interval syntax in this repo"
date: 2026-07-31
status: current
tags: [extractor, doctrine, enforcement, parsing, regex, gate-quality, gotcha]
reverify: "bash scripts/check_enumeration_parity.sh"
evidence: docs/tasks/PARITY-EXTRACTOR-ARM-SHAPE-GAP.md (the 7-of-8 measurement across all six extractors); docs/decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md
---

`ENUMERATION-PARITY`'s steering extractor read **7 of 8** categories for its entire life. It used
`grep -oE '=> "[a-z]+"'`, which assumes a one-line match arm — and **`rustfmt` wrapped the one arm
whose pattern got long enough**, putting `=>` and the string on different lines.

Measured across all six extractors: the five that parse structure a *tool* controls (a bash array,
a Markdown column, a directory listing, link syntax) are **exact**. The only one parsing `rustfmt`
output was the only one wrong — **and it degrades precisely when a category grows**, which is when
the guard is needed.

**Three corollaries.**

**(a) A count floor catches *"matched nothing"*, not *"matched most"*.** The floor was 6 and the
extractor found 7. When an extraction can be off by one, the floor must be the **real** count —
safe to pin, because a floor is *shrink-coupled*, not growth-coupled, and so is not a shadow.

**(b) A one-directional coverage predicate turns a missing extraction into a SILENT EXEMPTION** —
an id that is never extracted is checked nowhere. Bias extractors toward **over**-matching and
strip only provable noise: over-matching cries wolf and gets noticed, under-matching is invisible.

**(c) Measure whether the guarded thing actually drifted, separately from whether the guard
works.** All four doc sites did name the missing category, so this was a *latent hole*, not a live
inconsistency — and saying otherwise would have been as wrong as missing it.

**And the dialect trap:** `grep -E` uses **ERE**, where the interval is `{0,1}`. `sed` is **BRE**
and wants the backslashes. This repo's scripts mix both, and the escaped form inside `grep -E`
matches a literal brace and **silently never fires**. **An extractor must die on a missing field,
never fall through to something plausible.**

See also [[coverage-check-vacuity]] for the fenced predicate that replaced whole-file matching.
