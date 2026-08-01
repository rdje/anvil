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
  - "how do I stop an extractor from silently skipping an item"
  - "what is total_or_fail"
  - "is widening a regex enough to fix a vacuous check"
date: 2026-08-01
status: current
tags: [doctrine, enforcement, enumeration, vacuity, gate-quality, extractor, regex, gotcha]
reverify: "grep -n 'total_or_fail' scripts/check_enumeration_parity.sh   # the standing guard: an extractor must account for every item it walks. To RE-EARN the claim rather than read it, reproduce the historic bug: narrow the category capture back to \"([a-z]+)\" AND rename a knob_ids! category to contain an underscore, then run bash scripts/check_enumeration_parity.sh — it must FAIL with 'walked N item(s) but produced N-2 — it SILENTLY SKIPPED 2' (before PARITY-EXTRACTOR-CHARSET-GAP it printed ok at all seven sites). Restore both files afterwards; verify with git diff --stat."
evidence: docs/tasks/PARITY-EXTRACTOR-CHARSET-GAP.md (the measurement — a `case_qualifier` steering category reported `ok` at all 7 fenced sites, 0 FAILs, and 7 FAILs after the widening); scripts/check_enumeration_parity.sh (`extract_steering_categories`, `extract_adapter_ids`, and the recorded charset lesson beside the arm-shape one); DEVELOPMENT_NOTES.md `2026-08-01` `PARITY-EXTRACTOR-CHARSET-GAP.1` and `.2` (the latter records the two rungs `.1` skipped and the four controls)
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

**Do not stop at widening — that is still a guess, and it is not enforced.** Two rungs beyond it,
both live in `scripts/check_enumeration_parity.sh`:

1. **Specify nothing.** The quotes already delimit the value, so capture `"([^"]+)"`. A capture
   bounded by the real delimiters cannot be too narrow for anything the source can express.
2. **Make a silent skip impossible** — `total_or_fail`: an extractor must **account for every item
   it walks**, comparing a deliberately *looser* candidate predicate against the extraction. A
   skipped item becomes a hard failure (*"walked 40 item(s) but produced 38 — it SILENTLY SKIPPED
   2"*) instead of an invisible one. This also subsumes the older reshaped-row failure, since a row
   the pattern cannot parse is just another skip.

The two predicates must differ in strictness. If the candidate side used the same pattern as the
extraction, the comparison would be circular and always pass — the very vacuity being repaired,
reproduced inside the guard.

**Honest limit:** the guard fires when a member outside the narrowed class **arrives**, not when the
narrowing is **introduced**. Re-narrow the capture today and every existing member still matches, so
nothing is skipped. That is acceptable because the moment a member outside the class arrives is
exactly the moment the defect would do harm — and that moment is now loud. Detecting the narrowing
itself would require knowing what the source *could* legally contain, a semantic fact no syntactic
guard has (the same reason decision `0033` (c) refuses to ship a shadow *detector*).

See [[negative-control-must-be-able-to-fail]] for why a control that passes first time deserves
suspicion, and [[coverage-check-vacuity]] for the document-side half of this failure.
