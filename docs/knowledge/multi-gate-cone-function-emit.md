---
id: multi-gate-cone-function-emit
title: How ANVIL emits a multi-gate-cone `function automatic` — the `cone_function_emit_prob` gate emit-projection
answers:
  - "how do I make ANVIL emit a multi-gate cone function"
  - "how do I turn on cone_function_emit_prob"
  - "how do I get ANVIL to wrap a whole cone in one function automatic"
  - "how is the cone function different from the single-gate function_emit surface"
  - "which gates become a cone root and which interior gates get absorbed"
  - "why is an interior gate only absorbed when used once"
  - "how does ANVIL render a multi-statement cone function body"
  - "what is num_emitted_cone_functions"
  - "what does tool_matrix --cone-function-gate prove"
  - "how is the cone function emit surface proven downstream-clean"
  - "where is cone function emission implemented"
  - "does cone function emission change the emitted RTL behaviour"
  - "what is the __cf detection token"
date: 2026-06-17
status: current
tags: [structured-emission, function, cone, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/cone_function_emit.rs (annotate_cone_function_gates, compute_use_counts, absorb_children single-use cone-walk); src/config.rs (cone_function_emit_prob); src/gen/mod.rs (generate_module + generate_design rolls, run last after the task_emit pass); src/emit/sv.rs (cone-decl section + interior-suppression + render_cone_function_decl/render_cone_function_call/render_cone_gate_expr/cone_function_params/cone_operand_ref); src/metrics.rs (num_emitted_cone_functions); src/introspect/mod.rs (SCHEMA_VERSION 1.11); src/bin/tool_matrix.rs (--cone-function-gate, ScenarioSet::ConeFunctionSweep, ModuleReport.emitted_cone_function, saw_cone_function_emit); book/src/structured-emission.md; docs/decisions/0016-structured-emission-fifth-surface-cone-function.md; /tmp/anvil-cone-function-gate-r1/tool_matrix_report.json
reverify: 'cargo run --quiet -- --seed 4 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"cone_function_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"gate_struct_weight\":0,\"terminal_reuse_prob\":0.1,\"min_width\":4,\"max_width\":4,\"min_inputs\":3,\"max_inputs\":4,\"min_outputs\":1,\"max_outputs\":1,\"max_depth\":2});json.dump(c,open(\"/tmp/cf.json\",\"w\"))" && cargo run --quiet -- --seed 4 --config /tmp/cf.json | tee /tmp/cf.sv | grep -c "__cf(" && verilator --lint-only /tmp/cf.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.10b` — the multi-gate-cone `function automatic` emit-projection

ANVIL's **fifth richer-structured emission surface**
(decision [`0016`](../decisions/0016-structured-emission-fifth-surface-cone-function.md))
re-renders a whole combinational **cone** — a root gate plus the chain of
interior gates feeding it — as one multi-statement `function automatic` over the
cone's boundary leaves, instead of the inline per-gate `assign` chain. It is a
**deepening of the [[combinational-function-emit]] single-gate surface**
(decision `0012`, which took one gate over its direct operands) from one gate to
an entire cone.

- **Turn it on:** `Config::cone_function_emit_prob` (serde/config-file only — no
  CLI flag, like `function_emit_prob` / `generate_loop_emit_prob` /
  `task_emit_prob`; default `0.0` ⇒ byte-identical; validated `0.0..=1.0`). It is
  a **separate** knob from `function_emit_prob` so the shipped single-gate
  surface stays byte-identical (reusing it was rejected). Set it in a `--config`
  JSON; a small comb-only shape (`flop_prob = 0.0`, low `terminal_reuse_prob`)
  makes the one cone easy to read.
- **What qualifies (`src/ir/cone_function_emit.rs`):** the gen-time pass
  `annotate_cone_function_gates` walks every admissible gate as a candidate
  **root**, rolls the probability on the seeded RNG, and for a winner collects
  its cone into `Module.cone_function_gates` (`BTreeMap<NodeId, Vec<NodeId>>`,
  root → topo-ordered absorbed interior gate ids; an emitter-surface annotation —
  flat IR / validators / CSE / `canonical_module_signature` untouched). A root
  is an admissible gate (the `function_emit` candidate set — non-structured,
  non-`Slice`, `≥ 1` operand) whose cone has **at least one** absorbable interior
  gate (else it is left to the single-gate surface). The pass runs **last**
  (after `function_emit` / `generate_loop` / `task_emit` / `soft_union`), so the
  five emit-projections are mutually exclusive on a gate; `param_env` modules are
  skipped.
- **The single-use absorption rule (soundness):** an interior gate is absorbed
  only when `compute_use_counts` shows it is **used exactly once** in the whole
  module (gate operands + output drives + flop D/mux + instance inputs). Then its
  sole consumer is the cone edge that reached it, so the emitter can suppress
  **both** its module wire declaration and its inline `assign` (it now lives only
  as a function-local) provably safely. A multi-use (DAG-shared) gate stays a
  **boundary parameter** — keeping its own wire + assign — so the function reads
  it by name. This keeps the emission `-Wall` clean: every parameter is used,
  nothing is left undriven.
- **Rendering (`src/emit/sv.rs`):** for a marked root `<root>`,
  `render_cone_function_decl` emits `function automatic logic [W-1:0]
  <root>__cf(<one param per distinct boundary leaf, ascending NodeId>); <one
  `logic` local per absorbed interior gate, in topological order>; <root>__cf =
  <root expr>; endfunction`, constants folded inline as literals; the root's
  assign becomes `assign <root> = <root>__cf(<boundary-leaf refs>);`. A dedicated
  `render_cone_gate_expr` resolves each operand to its in-function name (interior
  → local wire name; boundary leaf → `a{i}`; constant → literal) — `render_gate`
  cannot be reused because `node_ref` resolves inputs/constants/flops
  intrinsically, ignoring a `names` override.
- **Behaviour-preserving / combinational only:** the function computes exactly
  the cone's value, so the module's behaviour is unchanged. The cone walk stops
  at the support-leaf boundary (primary inputs, flop `Q`s, instance outputs) — it
  never crosses a register edge or instance boundary.
- **Introspection:** `Metrics::num_emitted_cone_functions`
  (`= m.cone_function_gates.len()`) surfaces in the `--introspect`
  `module_metrics` (schema `1.11`); default-off reads `0`. Separate from
  `num_emitted_combinational_functions` (the single-gate surface).
- **Downstream gate:** `tool_matrix --cone-function-gate`
  (`ScenarioSet::ConeFunctionSweep`) forces `cone_function_emit_prob = 1.0` over
  comb-only DUTs across all three construction strategies (with
  `terminal_reuse_prob = 0.3` to keep single-use interiors plentiful), detects an
  emitted cone via `ModuleReport.emitted_cone_function`
  (`sv_text.contains("__cf(")`, distinct from the single-gate `"__f("`), and
  lights `saw_cone_function_emit` only when that module is accepted by Verilator
  **and** a clean Yosys (a cone function is universally synthesizable like a
  single-gate function, so the gate runs the full plan: Verilator + both Yosys
  modes + Icarus). Banked clean `/tmp/anvil-cone-function-gate-r1` (3 scenarios /
  12 modules / 12 emitting a cone function / 148 cone functions / `coverage_gaps
  = []` / `12/0` Verilator + both Yosys + Icarus compile).

See [[structured-emission-fifth-surface-cone-function]] for the decision (why a
multi-gate cone fifth, over the deferred multi-output `task` and the source-less
nested `generate`, with `interface` / `modport` still disqualified),
[[combinational-function-emit]] for the single-gate surface it deepens, and
`book/src/structured-emission.md` for the user-facing walk-through.
