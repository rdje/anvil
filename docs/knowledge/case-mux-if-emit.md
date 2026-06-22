---
id: case-mux-if-emit
title: How ANVIL emits a procedural `if`/`else if` priority chain — the `case_mux_if_emit_prob` projection of a dynamic-selector `CaseMux`
answers:
  - "how do I make ANVIL emit a procedural if/else if priority chain"
  - "how do I turn on case_mux_if_emit_prob"
  - "how do I get ANVIL to emit an always_comb if else if chain instead of a case"
  - "how do I render a CaseMux as an if/else if priority chain instead of a parallel case"
  - "can ANVIL emit an N-way mux as an if else if chain"
  - "which gates become a procedural if/else if priority chain"
  - "how is the if/else if priority chain different from the case render or the 2:1 mux if/else"
  - "what is the eighth structured emission surface"
  - "what is num_emitted_case_mux_if_chains"
  - "what does tool_matrix --case-mux-if-gate prove"
  - "how is the case-mux-if emit surface proven downstream-clean"
  - "where is if/else if priority-chain emission implemented"
  - "does if/else if priority-chain emission change the emitted RTL behaviour"
  - "why is the case-mux-if gate metric-keyed instead of text-keyed"
  - "why does the case-mux-if gate bias case_mux_prob and zero comb_mux_prob"
  - "why is there no __cv passthrough for the CaseMux priority chain"
date: 2026-06-23
status: current
tags: [structured-emission, if-else-if, priority-chain, case-mux, always-comb, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection, metric-keyed]
evidence: src/ir/case_mux_if_emit.rs (annotate_case_mux_if_gates — the dynamic-selector GateOp::CaseMux candidate predicate excluding constant selectors + CasezMux + all seven sibling marks, run last; gate_qualifies; collect-then-roll one gen_bool per candidate); src/config.rs (case_mux_if_emit_prob + the --case-mux-if-emit-prob CLI flag); src/main.rs (the flag overlay); src/gen/mod.rs (generate_module + generate_design rolls, run after mux_if); src/ir/types.rs (Module.case_mux_if_gates: BTreeSet<NodeId>); src/emit/sv.rs (the structured-case always_comb loop — the GateOp::CaseMux arm branches case…endcase → if…else if over the same operand refs + W'h0 default, no __cv passthrough); src/metrics.rs (num_emitted_case_mux_if_chains = m.case_mux_if_gates.len()); src/introspect/mod.rs (SCHEMA_VERSION 1.16); src/bin/tool_matrix.rs (--case-mux-if-gate, ScenarioSet::CaseMuxIfSweep, case_mux_if_focus_config, ModuleReport.emitted_case_mux_if = num_emitted_case_mux_if_chains > 0 [metric-keyed], saw_case_mux_if_emit); book/src/structured-emission.md; docs/decisions/0028-structured-emission-eighth-surface-case-mux-priority-chain.md; /tmp/anvil-case-mux-if-gate-r1/tool_matrix_report.json (12/12 modules emit a chain / 83 chains)
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"case_mux_if_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"comb_mux_prob\":0.0,\"case_mux_prob\":1.0,\"casez_mux_prob\":0.0,\"min_inputs\":3,\"max_inputs\":3,\"min_outputs\":1,\"max_outputs\":1,\"min_width\":4,\"max_width\":4,\"max_depth\":1,\"min_mux_arms\":2,\"max_mux_arms\":2});json.dump(c,open(\"/tmp/cmi.json\",\"w\"))" && cargo run --quiet -- --seed 1 --config /tmp/cmi.json | tee /tmp/cmi.sv | grep -c "else if (" && iverilog -g2012 -o /tmp/cmi.vvp /tmp/cmi.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.17b` — the procedural `if`/`else if` priority-chain emit-projection

ANVIL's **eighth richer-structured emission surface**
(decision [`0028`](../decisions/0028-structured-emission-eighth-surface-case-mux-priority-chain.md))
re-expresses a **dynamic-selector** `CaseMux` gate — rendered today as the parallel
`always_comb case (sel) … default` statement — as an **`if`/`else if` priority chain**
over the same operand refs. It is the lane's **first N-way procedural priority chain**
and a direct sibling of the [[mux-if-emit]] seventh surface (which projects a single 2:1
`Mux`); this one projects the N-way `CaseMux` (`GateOp::CaseMux` vs `GateOp::Mux`;
sequential-priority `if`/`else if` vs parallel-match `case` — a different frontend/synth
code path).

It is **simpler than the seventh surface**: a `CaseMux` is **already** declared as an
`always_comb`-written `logic` var, so this surface needs **no** `<wire>__cv` output var +
passthrough — only the `always_comb` *body* swaps `case … endcase` → `if … else if`.

- **Turn it on:** `Config::case_mux_if_emit_prob` (the `--case-mux-if-emit-prob` CLI flag,
  or `--config` JSON, like `mux_if_emit_prob`; default `0.0` ⇒ byte-identical; validated
  `0.0..=1.0`). It is a **separate** knob from `mux_if_emit_prob` so the shipped surfaces
  stay byte-identical (reusing one was rejected). A small comb-only shape with the
  `case`-mux block forced (`case_mux_prob = 1.0`, `comb_mux_prob = 0.0` so the
  earlier-rolling comb-mux block never preempts it) makes the chain easy to read.
- **What qualifies (`src/ir/case_mux_if_emit.rs`):** the gen-time pass
  `annotate_case_mux_if_gates` scans every `GateOp::CaseMux` gate whose **selector operand
  is not a `Node::Constant`** (a constant selector is statically collapsed by the emitter to
  a continuous `assign` and never emits an `always_comb` block — excluding it keeps the
  chain count exact) with at least one arm, and rolls the probability **once** per candidate
  on the seeded RNG. A `CasezMux` (masked `casez ?` wildcards — the recorded follow-up) is
  excluded. Because the pass runs **last** (after `mux_if` and the six earlier projections),
  `gate_qualifies` also excludes any gate already claimed by a sibling projection (vacuous in
  practice — no other pass marks a `CaseMux` — but kept for robustness), so a gate is
  projected by **at most one** of the eight surfaces. Marked gates land in
  `Module.case_mux_if_gates` (`BTreeSet<NodeId>`). An emitter-surface annotation — the flat
  IR / validators / CSE / `canonical_module_signature` are untouched; `param_env` modules are
  skipped.
- **Rendering (`src/emit/sv.rs`):** in the structured-case `always_comb` loop, the
  `GateOp::CaseMux` arm branches on `m.case_mux_if_gates.contains(&idx)`: a marked gate emits
  `if (sel == SW'd0) <g> = arm_0; else if (sel == SW'd1) <g> = arm_1; … else <g> = W'h0;`
  (reusing the **same** `sel` / selector width / `node_ref(arm)` the `case` arm computes, and
  the same `W'h0` default as the trailing `else`; `endcase` omitted); an unmarked gate emits
  the `case … endcase` verbatim. **No** `<g>__cv` var — the `CaseMux` is already an
  `always_comb` var (the simpler-than-seventh case).
- **Behaviour-preserving / combinational only:** the `case` labels `SW'd0..SW'd{k-1}` are
  **distinct constants by construction** (arm `i` ⇒ label `SW'd{i}`), so at most one equality
  is true ⇒ the priority chain selects the same arm as the parallel `case`, and the trailing
  `else` covers exactly the `default`. The chain reads only the selector + arm refs the `case`
  already reads and writes only `<g>` — exactly the parallel `case`'s read/write set, so there
  is no cycle risk.
- **Introspection:** `Metrics::num_emitted_case_mux_if_chains` (`= m.case_mux_if_gates.len()`)
  surfaces in the `--introspect` `module_metrics` (schema `1.16`); default-off reads `0`. It
  is **exact** because constant-selector `CaseMux` is excluded.
- **Downstream gate:** `tool_matrix --case-mux-if-gate` (`ScenarioSet::CaseMuxIfSweep`) forces
  `case_mux_if_emit_prob = 1.0` over comb-only DUTs across all three construction strategies.
  Its `case_mux_if_focus_config` is **`case_mux_prob`-biased**: `case_mux_prob = 0.9` with
  `comb_mux_prob = 0.0` (the comb-mux roll fires before the case-mux roll in `cone.rs` and
  would otherwise starve it); **no** `comb_mux_encoding_prob` steering is needed because a
  `CaseMux` selector is a generated dynamic cone by construction (no encoding-path trap — the
  inverse of the [[mux-if-emit]] gate's situation). Detection is **metric-keyed**:
  `ModuleReport.emitted_case_mux_if = module_metrics.num_emitted_case_mux_if_chains > 0`,
  **not** a text token — this surface emits no new identifier (only the `always_comb` body
  changes form), and an `if (… == …)` scan would also match FSM decode blocks. It lights
  `saw_case_mux_if_emit` only when that module is accepted by Verilator **and** a clean Yosys
  (a procedural `always_comb if/else if` chain is universally synthesizable, so the gate runs
  the full plan: Verilator + both Yosys modes + Icarus). Banked clean
  `/tmp/anvil-case-mux-if-gate-r1` (3 scenarios / 12 modules / 12 emitting a chain / 83 chains
  / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus compile). Across the bank
  the chain adds **zero** new Verilator `-Wall` warnings versus the knob-off parallel `case`.

See [[structured-emission-eighth-surface-case-mux-priority-chain]] for the decision (why an
N-way `if`/`else if` priority chain eighth — the first N-way procedural priority chain, the
recorded decision-`0027` follow-up, chosen over the `CasezMux` masked chain and
nested/multi-level `generate` / `interface` / `modport`), [[mux-if-emit]] for the sibling 2:1
`Mux` → `if`/`else` surface, and `book/src/structured-emission.md` for the user-facing
walk-through. The `CasezMux` masked priority chain is the recorded follow-up.
