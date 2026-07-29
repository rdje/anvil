---
id: pgen-first-contact-parser-gap
title: "First-contact consumer result: ANVIL output exposed a missing case/endcase production in PGEN's parser within minutes"
answers:
  - "has ANVIL found a real bug in a downstream consumer"
  - "did ANVIL find a parser bug"
  - "what did PGEN find with ANVIL"
  - "is there evidence ANVIL exposes real tool gaps"
  - "case endcase missing from a consumer grammar"
  - "who consumes the microdesign and frontend lanes"
date: 2026-07-29
status: current
tags: [effectiveness, downstream, consumer, parser, pgen]
evidence: "Owner-relayed consumer report (2026-07-29), recorded in docs/tasks/BOOK-LANE-COVERAGE.md; the reproducer lives in PGEN's repository (grammars/rtl_frontend.ebnf), not in this one"
---

# First-contact consumer result: PGEN parser gap

**Consumer-reported, owner-relayed (`2026-07-29`); not re-verifiable
from this repository alone.**

On first contact with ANVIL-generated output, PGEN — the downstream
consumer project the microdesign and frontend artifact lanes were built
for — found a real, previously-unknown gap in its own parser within
minutes: the `case`/`endcase` construct was absent from its
`grammars/rtl_frontend.ebnf` grammar. In PGEN's words, "that's the tool
doing exactly what it's built to do."

Why this matters durably:

- It is the first recorded **live-consumer** confirmation of the product
  thesis (`README.md` "Project objective"): legal, reproducible, unusual
  RTL that stays inside the synthesizable envelope exposes real bugs in
  downstream HDL consumers.
- `case` emission is a DUT-lane structured surface (`CaseMux`, Phase 3);
  the report does not state which lane's artifact triggered the find, so
  this card does not claim one.
- Context of the report: the same message asked ANVIL to document the
  microdesign/frontend lanes in the mdBook — tracked as
  `BOOK-LANE-COVERAGE` ([[live-doc-path-portability]] discipline applies
  to how the book cites paths).
