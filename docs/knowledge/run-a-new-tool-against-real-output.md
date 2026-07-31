---
id: run-a-new-tool-against-real-output
title: The fixture agrees with you; the tool does not — run a new instrument against **real** output before trusting it, because a hand-made fixture is shaped by the same assumption as the parser
answers:
  - "why did my new checker pass on fixtures and fail on real data"
  - "should I trust a tool that only ran against a fixture"
  - "why did a deriver report thousands of gaps for an empty array"
  - "how do I validate a new analysis tool"
date: 2026-07-30
status: current
tags: [testing, fixtures, tooling, method, gotcha]
evidence: docs/tasks/DIFFERENTIAL-SIMULATION.md (`.3b.2` — two-space `input  logic`); docs/tasks/PHASE-7-ORACLE-MICRODESIGN.md (`.2c.2b.1` — `rem_euclid` vs `%`); docs/tasks/EVIDENCE-BANK-DURABILITY.md (`.5` — a deriver reporting 2907 gaps for an empty array)
---

Three independent instances in this repository:

- `DIFFERENTIAL-SIMULATION.3b.2` — a parser that assumed one space in `input logic`, against real
  output carrying two;
- `PHASE-7 .2c.2b.1` — `rem_euclid` versus `%` on negative operands;
- `EVIDENCE-BANK-DURABILITY.5` — a deriver reporting **2907 gaps** for an **empty** array.

Every one passed its fixtures. **A hand-made fixture is shaped by the same assumption as the
parser it feeds**, so it cannot detect that the assumption is wrong — it can only confirm the code
matches the belief.

**Run a new instrument against real output before trusting a single number it produces.** If the
instrument is a checker, also run [[coverage-check-vacuity]]'s probe: delete the subject and
confirm it fails.
