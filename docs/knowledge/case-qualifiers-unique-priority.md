---
id: case-qualifiers-unique-priority
title: How ANVIL emits `unique` / `priority` case qualifiers — a decoration, not a tenth emit-projection
answers:
  - "how do I make ANVIL emit a unique case statement"
  - "how do I make ANVIL emit a priority case statement"
  - "how do I turn on unique_case_prob"
  - "how do I turn on priority_case_prob"
  - "which knob adds the unique keyword to a case statement"
  - "is the case qualifier one of the structured emission surfaces"
  - "why is unique_case_prob not in the emission steering category"
  - "what is num_emitted_unique_cases"
  - "what is num_emitted_priority_cases"
  - "what does the case-qualifier-gate prove"
  - "why does the case-qualifier gate skip Icarus for unique"
  - "what does vvp.tgt sorry: Case unique/unique0 qualities are ignored mean"
  - "is it safe for ANVIL to emit a unique case"
  - "how does ANVIL know the case statement is full and parallel"
  - "why does ANVIL always emit a default arm"
date: 2026-08-01
status: current
tags: [case-qualifier, unique, priority, emission, knob, assertion, downstream, valid-by-construction, matrix-gate, introspection, gotcha]
evidence: src/ir/case_qualifier.rs (CaseQualifier + gate_qualifies + annotate_case_qualifiers + the FULL/PARALLEL property test and its firing negative controls); src/ir/types.rs (Module.case_qualifiers); src/emit/sv.rs (the qualifier prefix on the two UNPROJECTED case/casez branches); src/config.rs (unique_case_prob / priority_case_prob + the "case_qualifier" knob_group arm); src/ir/knob_id.rs (the `qualifiers` steering category); src/metrics.rs (num_emitted_unique_cases / num_emitted_priority_cases); src/bin/tool_matrix.rs (ScenarioSet::CaseQualifierSweep, --case-qualifier-gate, scenario_emits_unique_case_qualifier); docs/evidence/anvil-case-qualifier-gate-r1.md; docs/decisions/0044-capability-breadth-unique-priority-case-qualifiers.md; book/src/structured-emission.md "A different kind of thing: case qualifiers"
reverify: 'cargo run --quiet --release -- --seed 7 --flop-prob 0.0 --comb-mux-prob 0.0 --case-mux-prob 0.5 --casez-mux-prob 0.5 --unique-case-prob 1.0 | grep -c "unique case\|unique casez"'
---

# A qualifier decorates a rendering; it does not replace one

`unique_case_prob` / `priority_case_prob` (both default `0.0`) prefix an IEEE 1800-2017
§12.5.3 keyword onto the `case` / `casez` statement a dynamic-selector `CaseMux` /
`CasezMux` **already** emits. Strip the keyword and the output is byte-identical.

That one sentence explains every design choice that looks surprising:

- **It is not a tenth emit-projection.** The nine `*_emit_prob` surfaces each *replace* how
  a gate renders and compete under mutual exclusion. This one claims nothing they want.
- **Its `KnobId`s live in a `qualifiers` steering category, not `emission`.** A qualifier
  claims only the gates the projections *declined*, so the two families are
  **anti-correlated** — one `--steer emission=2.0` over both would average a self-cancelling
  mixture. (`.4a` proposed `emission`; a `tests/pipeline.rs` guard pinning
  `emission.len() == 9` caught it, and the guard was right.)
- **Its knobs deliberately do not end in `_emit_prob`.** That name shape would have routed
  them into the `structured_emission` `knob_group`, and the decision-`0032` preset drift test
  derives `--profile structured-emission-max`'s expected set from that group — silently
  conscripting the qualifiers into the preset and costing it its Icarus column.

## Why emitting an assertion is safe here

A qualifier is a **claim a simulator checks at runtime**. Emitting one that could be false
would inject the exact sim/synth divergence class ANVIL exists to *find in tools*. It is
admissible only because both asserted properties are free from the generator — no analysis
pass, no generate-then-filter:

| property | asserted by | why it holds |
| --- | --- | --- |
| **FULL** (some `case_item` matches) | `priority` and `unique` | `emit/sv.rs` writes a `default:` arm for **every** `CaseMux`/`CasezMux` that renders as a statement, and `default` is itself a `case_item` |
| **PARALLEL** (no two match) | `unique` | `CaseMux` labels arm `i` with the integer `i`; `build_casez_patterns` is the **sole** `casez` pattern source and gives arm `i` the care-value `i` with one don't-care bit |

**Therefore the `default:` arm is never made conditional on the qualifier** — it is the
thing that makes FULL true. Both halves are asserted over real generated output by the
property test in `src/ir/case_qualifier.rs`, each with a firing negative control.

## Two operational gotchas

1. **`unique` is rolled first, and a gate it claims is *not* rolled for `priority`.** So
   each knob's `knob_roll_fires` equals its metric exactly. With both knobs at `1.0`,
   `priority` reports **no attempts at all** — not a `0/0` rate. That is why the gate runs
   one scenario per qualifier rather than one with both on.
2. **Icarus is a recorded no-op for `unique`, and `first_tool_warning` would have missed
   it.** `iverilog`/`vvp` exits `0` but prints `vvp.tgt sorry: Case unique/unique0 qualities
   are ignored.` once per qualified block — which does **not** contain the substring
   `warning:`, so the shared classifier passes it by lexical accident. The gate splits the
   tool plan (a second boolean gating the Icarus column alone) instead of re-classifying a
   mechanism every other gate depends on. It is **not** the `verilator_only` reduction: that
   drops Yosys too, and Yosys is this construct's strongest evidence — **identical
   synthesized cell counts** with and without the qualifier.

## Proving it yourself

The `reverify` above prints the number of qualified statements on a selector-biased shape.
The behaviour-preservation proof is one `sed` away — generate the same seed with the knob
off, strip the keyword from the ON corpus, and `diff`: over 8 modules and 176 qualified
statements the diff is **empty**.

Banked: [`anvil-case-qualifier-gate-r1`](../evidence/anvil-case-qualifier-gate-r1.md) — 52
modules, 359 qualified statements, `coverage_gaps = []`, Verilator 52/0, both Yosys modes
52/0, Icarus 28/0 (exactly the non-`unique` modules).

Related: [[introspection-schema-bump-classification]] · [[doctrine-enforcement]]
