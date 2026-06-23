---
id: casez-mux-if-emit
title: How ANVIL emits a procedural masked `if`/`else if` priority chain — the `casez_mux_if_emit_prob` projection of a dynamic-selector `CasezMux`
answers:
  - "how do I make ANVIL emit a masked if/else if priority chain"
  - "how do I turn on casez_mux_if_emit_prob"
  - "how do I get ANVIL to emit an always_comb masked if else if chain instead of a casez"
  - "how do I render a CasezMux as a masked if/else if priority chain instead of a parallel casez"
  - "can ANVIL emit a wildcard casez as an if else if chain with a mask"
  - "which gates become a masked procedural if/else if priority chain"
  - "how is the masked if/else if chain different from the casez render or the bare-equality case-mux chain"
  - "what is the ninth structured emission surface"
  - "what is num_emitted_casez_mux_if_chains"
  - "what does tool_matrix --casez-mux-if-gate prove"
  - "how is the casez-mux-if emit surface proven downstream-clean"
  - "where is masked if/else if priority-chain emission implemented"
  - "does masked if/else if priority-chain emission change the emitted RTL behaviour"
  - "why is the casez-mux-if gate metric-keyed instead of text-keyed"
  - "why does the casez-mux-if gate bias casez_mux_prob and zero both comb_mux_prob and case_mux_prob"
  - "why does ANVIL ship the masked-AND form instead of sel ==? pattern"
  - "what is the care_mask / value_masked idiom in the casez chain"
date: 2026-06-23
status: current
tags: [structured-emission, if-else-if, masked-priority-chain, casez-mux, always-comb, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection, metric-keyed]
evidence: src/ir/casez_mux_if_emit.rs (annotate_casez_mux_if_gates — the dynamic-selector GateOp::CasezMux candidate predicate excluding constant selectors + CaseMux + all eight sibling marks, run last; gate_qualifies = non-Constant selector + operands.len() >= 4; collect-then-roll one gen_bool per candidate; skips param_env modules); src/config.rs (casez_mux_if_emit_prob + the --casez-mux-if-emit-prob CLI flag, validated 0.0..=1.0); src/main.rs (the flag overlay); src/gen/mod.rs (generate_module + generate_design rolls, run after case_mux_if); src/ir/types.rs (Module.casez_mux_if_gates: BTreeSet<NodeId>); src/emit/sv.rs (the structured-case always_comb loop — the GateOp::CasezMux arm branches casez…endcase → masked if…else if, per arm care_mask = ~wildcard_mask & sel_mask + value_masked = pattern & care_mask via the existing constant_value/bitmask helpers, (sel & SW'h{care}) == SW'h{val}, no __cv passthrough); src/metrics.rs (num_emitted_casez_mux_if_chains = m.casez_mux_if_gates.len()); src/introspect/mod.rs (SCHEMA_VERSION 1.17); src/bin/tool_matrix.rs (--casez-mux-if-gate, ScenarioSet::CasezMuxIfSweep, casez_mux_if_focus_config, ModuleReport.emitted_casez_mux_if = num_emitted_casez_mux_if_chains > 0 [metric-keyed], saw_casez_mux_if_emit); book/src/structured-emission.md; docs/decisions/0029-structured-emission-ninth-surface-casez-mux-masked-priority-chain.md; /tmp/anvil-casez-mux-if-gate-r1/tool_matrix_report.json (12/12 modules emit a chain / 108 chains)
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/cz.json && python3 -c "import json;c=json.load(open(\"/tmp/cz.json\"));c.update({\"casez_mux_if_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"comb_mux_prob\":0.0,\"case_mux_prob\":0.0,\"casez_mux_prob\":1.0,\"min_inputs\":3,\"max_inputs\":3,\"min_outputs\":1,\"max_outputs\":1,\"min_width\":4,\"max_width\":4,\"max_depth\":1,\"min_mux_arms\":2,\"max_mux_arms\":2});json.dump(c,open(\"/tmp/czi.json\",\"w\"))" && cargo run --quiet -- --seed 1 --config /tmp/czi.json | tee /tmp/czi.sv | grep -c "else if ((" && iverilog -g2012 -o /tmp/czi.vvp /tmp/czi.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.19b` — the procedural masked `if`/`else if` priority-chain emit-projection

ANVIL's **ninth richer-structured emission surface**
(decision [`0029`](../decisions/0029-structured-emission-ninth-surface-casez-mux-masked-priority-chain.md))
re-expresses a **dynamic-selector** `CasezMux` gate — rendered today as the parallel
`always_comb casez (sel) … default` statement — as a **masked `if`/`else if` priority chain**
over the same operand refs. It **generalizes the [[case-mux-if-emit]] eighth surface** from the
bare-equality `CaseMux` to the wildcard `CasezMux` (`GateOp::CasezMux` vs `GateOp::CaseMux`;
masked equalities vs plain equalities — the wildcard `?` bits force the mask).

It is **simpler than the seventh surface** (and like the eighth): a `CasezMux` is **already**
declared as an `always_comb`-written `logic` var, so this surface needs **no** `<wire>__cv`
output var + passthrough — only the `always_comb` *body* swaps `casez … endcase` → masked
`if … else if`.

- **Turn it on:** `Config::casez_mux_if_emit_prob` (the `--casez-mux-if-emit-prob` CLI flag, or
  `--config` JSON, like `case_mux_if_emit_prob`; default `0.0` ⇒ byte-identical; validated
  `0.0..=1.0`). It is a **separate** knob from `case_mux_if_emit_prob` so the shipped surfaces
  stay byte-identical (reusing one was rejected). A small comb-only shape with the `casez`-mux
  block forced (`casez_mux_prob = 1.0`, and **both** `comb_mux_prob = 0.0` and
  `case_mux_prob = 0.0` so the earlier-rolling comb-mux and case-mux blocks never pre-empt it)
  makes the masked chain easy to read.
- **What qualifies (`src/ir/casez_mux_if_emit.rs`):** the gen-time pass
  `annotate_casez_mux_if_gates` scans every `GateOp::CasezMux` gate whose **selector operand is
  not a `Node::Constant`** (a constant selector is statically collapsed by the emitter to a
  continuous `assign` and never emits an `always_comb` block — excluding it keeps the chain count
  exact) with at least one arm (`operands.len() >= 4`), and rolls the probability **once** per
  candidate on the seeded RNG. The bare-equality `CaseMux` (owned by the eighth surface) is
  excluded. Because the pass runs **last** (after `case_mux_if` and the seven earlier
  projections), `gate_qualifies` also excludes any gate already claimed by a sibling projection,
  so a gate is projected by **at most one** of the nine surfaces. Marked gates land in
  `Module.casez_mux_if_gates` (`BTreeSet<NodeId>`). An emitter-surface annotation — the flat IR /
  validators / CSE / `canonical_module_signature` are untouched; `param_env` modules are skipped.
- **Rendering (`src/emit/sv.rs`):** in the structured-case `always_comb` loop, the
  `GateOp::CasezMux` arm branches on `m.casez_mux_if_gates.contains(&idx)`: a marked gate emits
  `if ((sel & SW'h{care}) == SW'h{val}) <g> = arm_0; else if … else <g> = W'h0;` (per arm
  `care_mask = ~wildcard_mask & sel_mask` and `value_masked = pattern & care_mask`, computed from
  the `(value, mask)` constants via the existing `constant_value` / `bitmask` helpers — the
  established `metrics.rs` / `compact.rs` care-mask idiom; `endcase` omitted); an unmarked gate
  emits the `casez … endcase` verbatim. **No** `<g>__cv` var — the `CasezMux` is already an
  `always_comb` var.
- **Behaviour-preserving / combinational only:** anvil builds `casez` patterns with **exactly one
  wildcard bit per arm** and **non-overlapping** care patterns (`build_casez_patterns`,
  `wildcard_bits = 1`), so at most one masked equality is true ⇒ the masked priority chain selects
  the same arm as the parallel `casez`, the trailing `else` covers exactly the `default`, and no
  arm's condition is constant-true. The chain reads only the selector + arm refs the `casez`
  already reads and writes only `<g>` — exactly the parallel `casez`'s read/write set, so there is
  no cycle risk.
- **Why the masked-AND form (not `sel ==? pattern`):** a fresh probe **disqualified** the concise
  `sel ==? pattern` wildcard-equality operator — Yosys `0.64` `read_verilog -sv` rejects `==?`
  (`syntax error, unexpected '?'`) in **both** repo modes — so the lowered masked-AND form ships.
  It is Verilator `5.046` `-Wall` 2012/2017/2023 + both Yosys modes + Icarus clean and iverilog
  `vvp` **exhaustively** sim-equivalent to the parallel `casez` (128/128 disjoint + 128/128 a
  hand-built overlapping-priority probe).
- **Introspection:** `Metrics::num_emitted_casez_mux_if_chains` (`= m.casez_mux_if_gates.len()`)
  surfaces in the `--introspect` `module_metrics` (schema `1.17`); default-off reads `0`. It is
  **exact** because constant-selector `CasezMux` is excluded.
- **Downstream gate:** `tool_matrix --casez-mux-if-gate` (`ScenarioSet::CasezMuxIfSweep`) forces
  `casez_mux_if_emit_prob = 1.0` over comb-only DUTs across all three construction strategies. Its
  `casez_mux_if_focus_config` is **`casez_mux_prob`-biased**: `casez_mux_prob = 0.9` with **both**
  `comb_mux_prob = 0.0` and `case_mux_prob = 0.0` (the comb-mux and case-mux rolls both fire before
  the casez-mux roll in `cone.rs` and would otherwise starve it — the eighth surface's single-zero
  generalized to a double-zero). Detection is **metric-keyed**:
  `ModuleReport.emitted_casez_mux_if = module_metrics.num_emitted_casez_mux_if_chains > 0`, **not**
  a text token — this surface emits no new identifier (only the `always_comb` body changes form),
  and an `if ((… & …) == …)` scan would also match the eighth surface's chain. It lights
  `saw_casez_mux_if_emit` only when that module is accepted by Verilator **and** a clean Yosys (a
  procedural `always_comb if/else if` chain is universally synthesizable, so the gate runs the full
  plan: Verilator + both Yosys modes + Icarus). Banked clean `/tmp/anvil-casez-mux-if-gate-r1`
  (3 scenarios / 12 modules / 12 emitting a chain / 108 chains / `coverage_gaps = []` / `12/0`
  Verilator + both Yosys + Icarus compile). Across the bank the masked chain adds **zero** new
  Verilator `-Wall` warnings versus the knob-off parallel `casez`.

See [[structured-emission-ninth-surface-casez-mux-masked-priority-chain]] for the decision (why a
masked `if`/`else if` priority chain ninth — generalize the eighth's bare-equality chain to the
wildcard `CasezMux`, chosen over the Yosys-rejected `==?` operator and nested/multi-level
`generate` / `interface` / `modport`), [[case-mux-if-emit]] for the sibling bare-equality
`CaseMux` → `if`/`else if` surface, and `book/src/structured-emission.md` for the user-facing
walk-through. Nested/multi-level `generate` and `interface` / `modport` remain the recorded future
surfaces.
