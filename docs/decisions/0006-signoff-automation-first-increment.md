---
id: signoff-automation-first-increment
title: The first SIGNOFF-AUTOMATION-EXPANSION increment promotes unswept generator knobs into explicit matrix axes + coverage facts
answers:
  - "what is the first SIGNOFF-AUTOMATION-EXPANSION increment"
  - "why not add a formal or techmapping acceptance column first"
  - "which generator knobs are not yet swept by tool_matrix"
  - "how does ANVIL remove hidden bias from the adversarial knob sweep"
  - "are microdesign and frontend lanes run through the tool_matrix acceptance columns"
  - "what acceptance columns does tool_matrix run today"
  - "why does ANVIL not add a new downstream tool column as the first signoff increment"
  - "what is ROADMAP steering gap 3 about adversarial axis coverage"
date: 2026-06-15
status: current
tags: [signoff, tool-matrix, coverage, adversarial, sweep, quality]
evidence: docs/decisions/0006-signoff-automation-first-increment.md; docs/tasks/SIGNOFF-AUTOMATION-EXPANSION.md; src/bin/tool_matrix.rs; src/downstream/mod.rs; ROADMAP.md
---

# 0006 - SIGNOFF-AUTOMATION-EXPANSION first increment: promote unswept knobs into explicit matrix axes + coverage facts

- Date: 2026-06-15
- Status: accepted
- Tags: signoff, tool-matrix, coverage, adversarial, sweep, quality

## Context

`SIGNOFF-AUTOMATION-EXPANSION` (Lane 3 of the three owner-directed post-phase
capability lanes) is opened to broaden downstream signoff-acceptance automation
in service of `project_anvil_north_star` (surface downstream-tool bugs via
valid-by-construction, downstream-acceptance-quality output). Its `.1` leaf is a
design/decision leaf that must pick **one** concrete, evidenced first increment
before any code edit, on the deciding factor recorded in the tree: **expected
bug-surfacing value per unit of implementation + validation cost.**

### Current signoff surface (inventory, current code)

- **Acceptance columns** (all **DUT-lane only**), each warning-as-failure:
  Verilator lint (`run_verilator`/`run_verilator_design`), Yosys
  `without-abc` (`synth -noabc`), Yosys `with-abc` (the repo-owned
  warning-clean `abc -fast; opt -fast` path), Icarus `iverilog -g2012` compile
  (`--iverilog-compile`), and the opt-in `--diff-sim` iverilog↔verilator
  semantic-agreement column. Tool invocations and verdicts live in
  `ToolInvocation`/`ModuleReport`/`DesignReport`/`DiffSimReport`
  (`src/bin/tool_matrix.rs`); the hardened invocation primitives live in
  `src/downstream/mod.rs` (fixed `verilator`/`yosys`/`iverilog` allow-list).
- **Scenario axes** swept by the built-in sets: construction strategy
  (sequential / shuffled / interleaved), identity mode (relaxed / node-id),
  factorization level (8 rungs), share probability (phase-2: `0.0/0.3/0.9`),
  structured mux/fold focus (phase-3), and 30+ hierarchy profiles (phase-4).
- **Coverage:** `CoverageSummary` (~112 `saw_*` facts + axis sets) +
  `compute_coverage_gaps`; phase gates `--phase1-gate` …
  `--phase4-hierarchy-gate`, `--phase2-share-gate`, `--phase3-structured-gate`,
  and `--fail-on-coverage-gap`.
- **Artifact families:** the matrix runs the acceptance columns on the **DUT
  lane only** (`artifact_kind = "module" | "design"`). The microdesign /
  frontend lanes have **separate** parity gates (`tests/microdesign_parity.rs`,
  `tests/frontend_parity.rs`) that extract facts; they are **not** run through
  the matrix's lint/synth acceptance columns.

### Gaps the inventory surfaced

1. **Hidden bias (ROADMAP steering gap 3).** Several generator knobs exist in
   `Config` but are **not swept as explicit axes** — they fire only by random
   chance inside the general motif-heavy profiles: operand / mux-arm
   duplication rates, `width_parameterization_prob`, `aggregate_prob` /
   `aggregate_array_prob`, and the memory×fsm interplay (`memory_prob` /
   `fsm_prob` together). ROADMAP gap 3 explicitly warns that "the adversarial
   space must be modeled as an explicit axis matrix, not as one vague notion of
   randomness … exercised without hidden bias from whichever implementation
   path is currently easiest."
2. The non-DUT lanes are not under the lint/synth acceptance columns.
3. No formal/SVA, CDC-lint, or FPGA-techmapping columns.

## Decision

**The first increment is "richer adversarial knob-sweep coverage": promote the
currently-unswept-but-existing generator knobs into explicit, first-class
`tool_matrix` scenario axes + `saw_*` coverage facts (and a focused gate), so
they fire by construction rather than by chance.** This directly closes ROADMAP
steering gap 3's hidden-bias hole, with repo-owned banked evidence (a clean
`tool_matrix` report whose new facts are all `true`), at low implementation +
validation cost (it reuses the existing scenario/coverage/gate machinery and the
hardened acceptance columns — **no new tool, no new dependency**), default-off /
byte-identical where a knob changes emitted RTL.

**Decisive test applied — "a day-one failure must be a real signal, not noise."**
A newly-swept **legal** knob combination that makes Verilator/Yosys
warn or reject is exactly the north-star signal (a downstream tool tripping on
valid-by-construction RTL it had not been forced to see). Contrast the rejected
alternatives, whose day-one failures are mostly noise:

- **Rejected (as first) — a new aggressive-synthesis / techmapping column**
  (e.g. `synth_ice40` / `synth_xilinx`, or raw ABC). The repo already had to
  *soften* `with-abc` because raw ABC "was tripping non-actionable
  combinational-network warnings on valid generated designs"; a techmap target
  would produce **more** such non-actionable noise (target-capacity limits ≠
  tool bugs), fighting the warning-as-failure discipline. High ceiling, high
  noise, high curation cost — not a clean first increment.
- **Rejected (as first) — a formal / SVA column.** ANVIL is structure-first and
  generates **no spec/properties** (whole-module functionality is an explicit
  non-goal); a formal flow would have nothing to prove and brushes the
  spec/oracle non-goal. Poor fit.
- **Rejected (as first) — run the non-DUT lanes through the acceptance
  columns.** Genuinely valuable (new artifact families under signoff) but
  higher-cost and nuanced: the **frontend** lane's stub-child elaboration
  corpora are not end-to-end *synthesizable* (undefined child modules), so a
  Yosys-synth column would fail by design, not by bug. This needs its own
  per-lane design sub-leaf and is **kept as a future leaf**, not retired.

No mode/strategy/gate is retired (`feedback_never_retire_strategies`); the
higher-ceiling new-column and non-DUT-acceptance paths remain named future
leaves of this lane.

### Tree split

`.1` (this leaf) splits the tree forward:

- **`.2`** — implement the first knob-sweep batch: add explicit scenarios that
  force the highest-bias unswept knobs (lead candidate: the
  duplication-rate / aggregate / width-parameterization / memory×fsm knobs),
  plus the matching `saw_*` coverage facts and a focused gate, default-off /
  byte-identical where a knob changes RTL, banked clean across Verilator +
  both Yosys modes. To be split into design + impl if it proves broad (the
  `.3a`/`.3b` precedent).

## Consequences

- The adversarial matrix exercises previously-implicit knobs **by
  construction**, removing the hidden-bias blind spot ROADMAP gap 3 warns about,
  with banked coverage evidence rather than narrative.
- The single source of downstream truth stays `tool_matrix` + `downstream`; the
  increment adds scenarios/facts, not a second runtime path.
- Warning-as-failure, rules-first, no-retirement, and default-off /
  byte-identical invariants are all preserved.
- The higher-ceiling increments (new tool columns; non-DUT lanes under
  acceptance) are explicitly preserved as future leaves of this lane.

## Open questions

- `.2` decides the exact first knob batch and the scenario shapes (one focused
  scenario per knob vs a small combined-stress scenario), and which new `saw_*`
  facts + gate assertions to add.

## Links

- Task-tree: `SIGNOFF-AUTOMATION-EXPANSION.1` (this leaf); frontier advances to `.2`
- Predecessor lane: decision [`0005`](0005-agent-mcp-expansion-surface.md)
  (`AGENT-MCP-EXPANSION`, closed)
- North star: `project_anvil_north_star` (auto-memory)
- Doctrine: `feedback_rules_first_generation`, `feedback_never_retire_strategies`
- Reuse: `src/bin/tool_matrix.rs` (scenarios / `CoverageSummary` /
  `compute_coverage_gaps`), `src/downstream/mod.rs` (acceptance columns),
  `ROADMAP.md` steering gap 3
