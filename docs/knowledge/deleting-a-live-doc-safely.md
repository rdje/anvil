---
id: deleting-a-live-doc-safely
title: Before deleting from a live doc — grep the **doctrine checks** and `src/` for the file first, prove loss-freedom by sweeping **every** token in the deleted range rather than a phrase list, and remember a line cap and a byte cap are not redundant
answers:
  - "how do I delete a section from a live document without losing information"
  - "how do I prove a documentation deletion lost nothing"
  - "why did a docs-only deletion fail a doctrine check"
  - "can a source comment depend on a documentation file"
  - "why check both a line cap and a byte cap"
  - "what does the residue of a good deletion sweep look like"
date: 2026-07-30
status: current
tags: [docs, deletion, sweep, evidence, caps, gotcha, doctrine]
reverify: "bash scripts/check_doctrines.sh"
evidence: docs/decisions/0036-readme-landing-page-restoration.md (the 1141-line deletion and its 879-token sweep); docs/tasks/README-POLICY-ADOPTION.md; scripts/check_enumeration_parity.sh (the declared-site requirement that a deletion can breach)
---

Four rules, all earned deleting 1,141 lines from `README.md`
([`0036`](../decisions/0036-readme-landing-page-restoration.md)).

**(i) Grep the doctrine checks for the file first.** They encode invisible *content* requirements.
`README.md` was a declared `ENUMERATION-PARITY` site, so cutting one `--steer` bullet inside a
large deletion would have failed the hook. The repair is to **drop the site** (rung R1) — never to
re-add a list "to keep the gate green", because that list grows one line per future category,
which is exactly how the file reached 1,771 lines.

**(ii) Grep `src/` too.** A doc comment can name where a fact is *cited*: two `tool_matrix.rs`
comments said *"cited in `README.md`"*. **No docs-side check sees that lie.**

**(iii) Prove nothing was lost by sweeping EVERY token in the deleted range** — 879 of them here —
not a hand-picked phrase list. A working sweep's residue is made only of **composites of covered
parts**; an orphaned fact stands out in a residue of 32 where it would hide in 879. Report the
residue, not just the finds.

**(iv) A line cap and a byte cap are not redundant.** The surviving landing page was
**10,297 B across 141 lines**, so the projected file was already *over* the byte cap while at
**74 %** of the line cap. Prose density is the hidden variable: **118 B/line** for a numbered list
against **57** for a path list. **Trim to the cap; never raise the cap to fit the trim.**

See also [[overflow-destination-classification-and-the-unmeasured-axis]] — a cap on one axis bounds
a *projection* of a file, and content flows into the axis nobody measured.
