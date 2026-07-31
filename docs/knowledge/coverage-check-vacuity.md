---
id: coverage-check-vacuity
title: A "does the doc name every id" check can pass VACUOUSLY — test it by deleting the subject and re-running, and scope the match to a marked region rather than to the whole file
answers:
  - "how do I know a doctrine check actually checks anything"
  - "how do I test whether a coverage check is vacuous"
  - "why did ENUMERATION-PARITY pass while a doc was missing an entry"
  - "why is grepping the whole file for every id a weak check"
  - "which ENUMERATION-PARITY pairs protect nothing"
  - "does the downstream adapter allow-list pair check anything"
  - "what is the acceptance test for a new coverage-shaped check"
  - "why is a proximity window not enough to scope a doc check"
  - "how should an enumeration be marked so a check can read it"
  - "why is a check weakest in the document that documents its ids"
date: 2026-07-31
status: current
tags: [doctrine, enforcement, enumeration, vacuity, gate-quality, testing, docs]
reverify: bash scripts/check_enumeration_parity.sh
evidence: docs/decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md; scripts/check_enumeration_parity.sh:205-223 (`covers_set`); DOCTRINE_ENFORCEMENT.md §9; docs/tasks/LIVE-DOC-REGISTRY-SHADOWS.md
---

A check of the shape *"this document names every member of set S"*, implemented as a
whole-file substring grep, can pass **with the enumeration it guards deleted outright**.

**The acceptance test — ask it of every coverage-shaped check, new or existing:**
**delete the subject and re-run the check.** If it still passes, it is checking nothing.
This sits alongside the both-directions negative control (decision `0033` (b)), not
instead of it: a control proves the check *can* fire, the vacuity probe proves it fires
on the *right input*.

**The predictor is structural, so you can see it before writing the check:** a coverage
check's strength is inversely proportional to how ordinary its ids are as *words* in the
document being checked — and ids are most ordinary in exactly the document that
documents them. Coined tokens (`MEMORY-ARCH`, `README-GROWTH`) appear nowhere but their
list, so the file-scoped predicate is near-exact; real vocabulary (`verilator`, `yosys`,
`state`, `sharing`) appears throughout the chapter that lists it, so the predicate
degrades to nothing. **The check is therefore weakest precisely where the enumeration is
most worth guarding.**

**Measured in ANVIL at `abf7090` (`2026-07-31`): 3 of `ENUMERATION-PARITY`'s 10 coverage
sites passed the probe** — both downstream-allow-list sites (so that pair protected
nothing at either site) and one steering-taxonomy site. Adding more *sites* cannot fix
this; the *predicate* is what is wrong.

**Repaired** at `LIVE-DOC-REGISTRY-SHADOWS.3`: every declared site marks its enumeration
with an **inline HTML-comment fence** carrying the set id
(`<!--enum:steer-categories-->` … `<!--/enum:steer-categories-->`), the check reads only
inside the fence, and a **missing fence is a hard failure** so the predicate cannot decay
back to whole-file matching. Markers are inline, not on their own line, because an HTML
comment on its own line is a CommonMark *block* that would split the paragraph, table or
list it sits in — the fence has to be invisible in the rendered book. It carries a set id
because a single file can be a declared site for two different sets. It names **no
members**, so it is not itself a shadow under
[[shadow-enumeration-classification]]'s test (2).

A proximity window was measured and **rejected**: it leaves an id-rich chapter exactly as
vacuous, and it misses a single-id omission wherever good explanatory prose sits beside
the list — its blind spots correlate with documentation *quality*.

**Two limits of the fenced predicate, both earned in implementation.** It is **coverage,
not exact parity** — a fence enclosing prose-bearing list items cannot be losslessly
harvested, and the reverse direction is near-empty here anyway since nothing is ever
retired. And **a fence must contain the enumeration, not the discussion of it**:
commentary inside a fence that names an id a second time re-imports the vacuity at fence
scale. That produced the single wrong pass in a 98-control sweep, and the offender was the
paragraph explaining the defect. Audit each fence for duplicate ids, and drop **every** id
at **every** site rather than sampling.

Separately, the check's own **declared-site list is authoritative and stays
hand-written**: it must be allowed to differ from "every tracked file naming the ids",
because append-only history records the *old* set correctly and may never be retro-edited
(decision `0031`). Sites are added by sweeping the whole tree from the authoritative set
and classifying every candidate — never because one turned up in a bug report.

Full measurement, the rejected alternatives, and the fence design:
`docs/decisions/0037-enumeration-parity-declared-sites-and-list-scoped-coverage.md`.
See also [[doctrine-enforcement]] for the registry+driver this check runs under.
