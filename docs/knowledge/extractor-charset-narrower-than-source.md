---
id: extractor-charset-narrower-than-source
title: A parity check also passes vacuously from the OTHER side — when the extractor's character class is narrower than its source, an id is never read, so it is never missing; and a count floor cannot see an id that is born invisible
answers:
  - "my new steering category passed every doc parity check and it should not have"
  - "ENUMERATION-PARITY says ok but the docs do not name my new id"
  - "why did adding a knob category not fail the enumeration gate"
  - "can a doctrine check pass because it never read the value"
  - "what charset should an extractor capture"
  - "does the count floor catch a missing id"
  - "why is a count floor blind to a new id"
  - "shrink-coupled vs growth-coupled guard"
  - "is renaming my id to fit the regex a valid fix"
  - "can an id contain an underscore or a hyphen in check_enumeration_parity"
date: 2026-08-01
status: current
tags: [doctrine, enforcement, enumeration, vacuity, gate-quality, extractor, regex, gotcha]
reverify: "grep -n 'a-z0-9_-' -n scripts/check_enumeration_parity.sh   # the adapter capture; then grep -n '(\\[a-z0-9_\\]+)' scripts/check_enumeration_parity.sh   # the category capture. To re-earn the claim rather than read it: temporarily give a `knob_ids!` row a category containing `_` and re-run `bash scripts/check_enumeration_parity.sh` — it must go RED naming that category at each fenced site (before PARITY-EXTRACTOR-CHARSET-GAP.1 it printed `ok` at all seven), then restore."
evidence: docs/tasks/PARITY-EXTRACTOR-CHARSET-GAP.md (the measurement — a `case_qualifier` steering category reported `ok` at all 7 fenced sites, 0 FAILs, and 7 FAILs after the widening); scripts/check_enumeration_parity.sh (`extract_steering_categories`, `extract_adapter_ids`, and the recorded charset lesson beside the arm-shape one); DEVELOPMENT_NOTES.md `2026-08-01` `PARITY-EXTRACTOR-CHARSET-GAP.1`
---

A "does every doc name every member of set S" check has **two** halves that can go vacuous, and
[[coverage-check-vacuity]] covers only one of them.

- **Document side** — the predicate is too loose, so a doc "names" an id it does not really list.
  Repaired by fenced regions.
- **Authoritative-set side** — the **extractor** never produced the id, so `S` is missing it, so
  *no site can be missing it*. **Every site passes, and the fence is irrelevant.**

The second one is the nastier failure, because a perfect predicate does not help: you cannot be
missing what was never in the set.

**The concrete cause, measured in ANVIL (`2026-08-01`).** `extract_steering_categories` captured the
category column with `"([a-z]+)"` — letters only — while the **name** column on the very same row
already allowed `[a-z0-9_]+`. A new category `case_qualifier` was therefore invisible, and
`check_enumeration_parity.sh` printed **`ok` at all seven fenced sites** when it should have failed
at all seven.

**The rule:** *capture the charset the **source** permits, not the charset its current members happen
to use.* Every category that existed was one lowercase word, so the narrowing had never been
exercised — a coincidence, not a rule, and nothing declared it.

**A count floor does not save you, and it is important to know why.** A floor is **shrink-coupled,
not growth-coupled**:

| the hidden id is… | floor | what fires |
| --- | --- | --- |
| a **renamed** existing id | drops below the floor | trips — but its message names the *entry count*, pointing at the extractor rather than at the docs missing the id |
| a **newly added** id | the old members still extract, so the count is unchanged | **nothing** |

**Ids are added far more often than they are renamed, so the case a floor cannot see is the common
one.** When the floor *does* fire it also gives the wrong diagnosis, sending you to debug the
extractor instead of the two docs.

**Do not accept the escape hatch.** Renaming the id to something the broken reader accepts makes the
gate green immediately, and it **masks** the defect so the next id is born invisible again. A fix
that works only because you picked a value the reader tolerates is not a fix. (In ANVIL the rename
to `qualifiers` was independently correct — it matches `selectors` / `terminals` / `motifs` /
`emission` — and was still not treated as the repair.)

**Hold each extractor with a control, per `DOCTRINE_ENFORCEMENT.md` §9:** give a member a character
from the widened class and confirm the check goes **red naming it**. Since a vacuous pass is exactly
what is being repaired, only a failing control distinguishes the fix from the coincidence that was
hiding the bug. See [[negative-control-must-be-able-to-fail]] for why a control that passes first
time deserves suspicion.
