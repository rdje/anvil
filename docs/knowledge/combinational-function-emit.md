---
id: combinational-function-emit
title: How ANVIL emits a combinational `function automatic` — the `function_emit_prob` single-gate emit-projection
answers:
  - "how do I make ANVIL emit a combinational function"
  - "how do I turn on function_emit_prob"
  - "how do I get ANVIL to print a function automatic"
  - "which gates qualify for function emission"
  - "why is Slice excluded from function emission"
  - "are case casez or for-fold gates function-emitted"
  - "how does ANVIL render an emitted function automatic"
  - "how does ANVIL handle duplicate operands in an emitted function"
  - "is ANVIL function emission combinational only"
  - "what is num_emitted_combinational_functions"
  - "what does tool_matrix --function-emit-gate prove"
  - "how is the combinational function emit surface proven downstream-clean"
  - "where is combinational function emission implemented"
  - "does function emission change the emitted RTL behaviour"
date: 2026-06-16
status: current
tags: [structured-emission, function, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/function_emit.rs (annotate_function_emit_gates); src/config.rs (function_emit_prob); src/gen/mod.rs (generate_module + generate_design rolls, after the soft_union pass); src/emit/sv.rs (function_emit_gate, render_gate_function_decl, render_gate_function_body, render_gate_function_call); src/metrics.rs (num_emitted_combinational_functions); src/bin/tool_matrix.rs (--function-emit-gate, ScenarioSet::FunctionEmitSweep, ModuleReport.emitted_combinational_function, saw_combinational_function_emit); book/src/structured-emission.md; docs/decisions/0012-structured-emission-first-surface-combinational-function.md; /tmp/anvil-function-emit-gate-r1/tool_matrix_report.json
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"function_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"gate_struct_weight\":0,\"min_width\":4,\"max_width\":4,\"min_inputs\":3,\"max_inputs\":4});json.dump(c,open(\"/tmp/fe.json\",\"w\"))" && cargo run --quiet -- --seed 11 --config /tmp/fe.json | tee /tmp/fe.sv | grep -c "function automatic" && verilator --lint-only /tmp/fe.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.2b` — the combinational `function automatic` emit-projection

ANVIL's **first richer-structured emission surface**
(decision [`0012`](../decisions/0012-structured-emission-first-surface-combinational-function.md))
re-renders a selected combinational gate as a `function automatic` of its
direct operands instead of an inline `assign`.

- **Turn it on:** `Config::function_emit_prob` (serde/config-file only — no
  CLI flag, like `soft_union_slice_prob` / `aggregate_prob`; default `0.0` ⇒
  byte-identical; validated `0.0..=1.0`). Set it in a `--config` JSON. A
  comb-only shape (`flop_prob = 0.0`, `gate_struct_weight = 0`) makes the
  functions easy to read.
- **What qualifies:** the gen-time pass
  `crate::ir::function_emit::annotate_function_emit_gates` rolls the
  probability on the seeded RNG per *qualifying* candidate and marks the
  winners in `Module.function_emit_gates` (`BTreeSet<NodeId>`, an
  emitter-surface annotation — flat IR / validators / CSE /
  `canonical_module_signature` untouched, disjoint from
  `soft_union_slice_gates`). A candidate is an ordinary combinational `Gate`
  with ≥1 operand. It runs **after** the `soft_union` pass (so `union soft`
  marks are excluded) and skips Phase-5 `param_env` modules.
- **Excluded (still emitted inline — nothing retired):** structured
  selectors (`CaseMux` / `CasezMux` / `ForFold`) are their own richer
  surface, and **`Slice`** is excluded because a bit-select reads only a
  sub-range of its operand, so a full-width function parameter leaves bits
  unused and a forced `function_emit_prob = 1.0` `verilator -Wall` sweep
  flags `UNUSEDSIGNAL`. A slice-aware projection (`src[hi:lo]`) is a
  recorded follow-up.
- **Rendering (`src/emit/sv.rs`):** for a marked gate `<wire>`, a
  `function automatic logic [W-1:0] <wire>__f(input logic [Wi-1:0] a0, …)`
  declaration whose body re-expresses the op over **positional** params
  (`render_gate_function_body`, the positional counterpart of
  `render_gate`), and the call site becomes
  `assign <wire> = <wire>__f(<operand refs>);`. Positional params handle a
  gate whose operands repeat:
  `assign concat_0 = concat_0__f(case_mux_0, case_mux_0);`.
- **Behaviour-preserving / combinational only:** the call evaluates to
  exactly the inline expression, so the module's behaviour is unchanged
  (a flop `Q` is a leaf parameter — the projection never recurses through a
  register edge or instance boundary).
- **Introspection:** `Metrics::num_emitted_combinational_functions`
  (`= m.function_emit_gates.len()`) surfaces in the `--introspect`
  `module_metrics` (schema `1.8`); default-off reads `0`.
- **Downstream gate:** `tool_matrix --function-emit-gate`
  (`ScenarioSet::FunctionEmitSweep`) forces `function_emit_prob = 1.0` over
  comb-only DUTs across all three construction strategies, detects an
  emitted function via `ModuleReport.emitted_combinational_function`
  (`sv_text.contains("function automatic")`), and lights
  `saw_combinational_function_emit` only when that module is accepted by
  Verilator **and** a clean Yosys (a synthesizable function is universally
  accepted, so — unlike the Verilator-only `union soft` up-opt — the gate
  runs the full tool plan: Verilator + both Yosys modes + Icarus). Banked
  clean `/tmp/anvil-function-emit-gate-r1` (3 scenarios / 12 modules / 608
  emitted functions / `coverage_gaps = []` / `12/0` Verilator + both Yosys +
  Icarus compile).

See [[structured-emission-first-surface-combinational-function]] for the
decision (why a function first, over interface/modport + nested generate),
[[sv-version-soft-union-upopt]] for the sibling default-off emit-projection,
and `book/src/structured-emission.md` for the user-facing walk-through.
