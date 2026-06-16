---
id: generate-loop-emit
title: How ANVIL emits a `generate for` loop — the `generate_loop_emit_prob` replication emit-projection
answers:
  - "how do I make ANVIL emit a generate for loop"
  - "how do I turn on generate_loop_emit_prob"
  - "how do I get ANVIL to print a generate block or genvar"
  - "which replications qualify for generate-loop emission"
  - "why is a wider lane excluded from generate-loop emission"
  - "how does ANVIL render an emitted generate for loop"
  - "what is the genvar increment form in an emitted generate loop"
  - "is ANVIL generate-loop emission combinational only"
  - "what is num_emitted_generate_loops"
  - "what does tool_matrix --generate-loop-gate prove"
  - "how is the generate for loop emit surface proven downstream-clean"
  - "where is generate-loop emission implemented"
  - "does generate-loop emission change the emitted RTL behaviour"
date: 2026-06-16
status: current
tags: [structured-emission, generate, genvar, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/generate_loop.rs (annotate_generate_loop_gates); src/config.rs (generate_loop_emit_prob); src/gen/mod.rs (generate_module + generate_design rolls, after the function_emit pass); src/emit/sv.rs (generate_loop_gate, render_generate_loop_block, the generate-block section + assign-loop suppression); src/metrics.rs (num_emitted_generate_loops); src/bin/tool_matrix.rs (--generate-loop-gate, ScenarioSet::GenerateLoopSweep, ModuleReport.emitted_generate_loop, saw_generate_loop_emit); book/src/structured-emission.md; docs/decisions/0013-structured-emission-second-surface-generate-loop.md; /tmp/anvil-generate-loop-gate-r1/tool_matrix_report.json
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"generate_loop_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"min_width\":4,\"max_width\":8,\"min_inputs\":3,\"max_inputs\":5,\"min_outputs\":1,\"max_outputs\":2,\"max_depth\":3});json.dump(c,open(\"/tmp/gl.json\",\"w\"))" && cargo run --quiet -- --seed 12 --config /tmp/gl.json | tee /tmp/gl.sv | grep -c "generate" && verilator --lint-only /tmp/gl.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.4b` — the `generate for` loop emit-projection

ANVIL's **second richer-structured emission surface**
(decision [`0013`](../decisions/0013-structured-emission-second-surface-generate-loop.md))
re-renders a selected `{N{x}}` replication as a single-level `generate for`
loop instead of an inline `assign <wire> = {N{x}};`.

- **Turn it on:** `Config::generate_loop_emit_prob` (serde/config-file only — no
  CLI flag, like `function_emit_prob` / `soft_union_slice_prob`; default `0.0` ⇒
  byte-identical; validated `0.0..=1.0`). Set it in a `--config` JSON. A small
  comb-only shape (`flop_prob = 0.0`) makes the one loop easy to read.
- **What qualifies:** the gen-time pass
  `crate::ir::generate_loop::annotate_generate_loop_gates` rolls the probability
  on the seeded RNG per *qualifying* candidate and marks the winners in
  `Module.generate_loop_gates` (`BTreeSet<NodeId>`, an emitter-surface
  annotation — flat IR / validators / CSE / `canonical_module_signature`
  untouched, disjoint from `function_emit_gates`). A candidate is a
  `GateOp::Concat` of the `{N{x}}` form — `N ≥ 2` operands that are all the
  **same** `NodeId` — with a **1-bit lane** (so result width `== N` and
  `<wire>[gi] = x` is bit-faithful). This is the common one-hot `{W{sel}}`
  mux-mask broadcast idiom. It runs **after** the `function_emit` pass (so a
  function-emit-marked replication is excluded — the two projections are
  mutually exclusive on a gate) and skips Phase-5 `param_env` modules.
- **Excluded (still emitted inline — nothing retired):** a **wider lane** (e.g.
  `{4{byte}}` with an 8-bit lane) is still index-regular but would need a
  part-select body (`<wire>[gi*LW +: LW] = x`); a part-select projection is a
  recorded follow-up. Until then a wider replication emits the inline `{N{x}}`.
- **Rendering (`src/emit/sv.rs`):** for a marked replication `<wire>`, a
  generate-block section (after the function-decl section) emits
  `genvar <wire>__gi; generate for (<wire>__gi = 0; <wire>__gi < N; <wire>__gi =
  <wire>__gi + 1) begin : <wire>__gen assign <wire>[<wire>__gi] = <x>; end
  endgenerate` (`render_generate_loop_block` + the `generate_loop_gate`
  defensive accessor), and the per-gate assign loop `continue`s past the marked
  gate so the inline `assign <wire> = {N{x}};` is suppressed. The increment is
  the maximally-portable `gi = gi + 1` (`gi++` is equally valid, not foreclosed).
- **Behaviour-preserving / combinational only:** the unrolled loop is exactly
  `{N{x}}`, so the module's behaviour is unchanged.
- **Introspection:** `Metrics::num_emitted_generate_loops`
  (`= m.generate_loop_gates.len()`) surfaces in the `--introspect`
  `module_metrics` (schema `1.9`); default-off reads `0`.
- **Downstream gate:** `tool_matrix --generate-loop-gate`
  (`ScenarioSet::GenerateLoopSweep`) forces `generate_loop_emit_prob = 1.0` over
  comb-only DUTs across all three construction strategies, detects an emitted
  loop via `ModuleReport.emitted_generate_loop`
  (`sv_text.contains("generate")`), and lights `saw_generate_loop_emit` only
  when that module is accepted by Verilator **and** a clean Yosys (a `generate
  for` is universally synthesizable like a function, so — unlike the
  Verilator-only `union soft` up-opt — the gate runs the full tool plan:
  Verilator + both Yosys modes + Icarus). Banked clean
  `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8 emitting a
  loop / `coverage_gaps = []` / `12/0` Verilator + both Yosys + Icarus compile).

See [[structured-emission-second-surface-generate-loop]] for the decision (why a
`generate for` second, over `task` / `interface` / a constant-predicate
`generate if`), [[combinational-function-emit]] for the sibling first surface,
and `book/src/structured-emission.md` for the user-facing walk-through.
