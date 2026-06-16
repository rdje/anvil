---
id: combinational-task-emit
title: How ANVIL emits a combinational `task automatic` — the `task_emit_prob` gate emit-projection
answers:
  - "how do I make ANVIL emit a task automatic"
  - "how do I turn on task_emit_prob"
  - "how do I get ANVIL to print a task or always_comb task call"
  - "which gates qualify for task emission"
  - "how does ANVIL render an emitted combinational task"
  - "what is the output-var passthrough form in an emitted task"
  - "is ANVIL task emission combinational only"
  - "what is num_emitted_combinational_tasks"
  - "what does tool_matrix --task-emit-gate prove"
  - "how is the combinational task emit surface proven downstream-clean"
  - "where is task emission implemented"
  - "does task emission change the emitted RTL behaviour"
  - "how is task emission different from function emission"
date: 2026-06-16
status: current
tags: [structured-emission, task, always_comb, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/task_emit.rs (annotate_task_emit_gates); src/config.rs (task_emit_prob); src/gen/mod.rs (generate_module + generate_design rolls, after the generate_loop pass); src/emit/sv.rs (task_emit_gate, render_gate_task_decl, render_gate_task_call reusing render_gate_function_body, the task section + assign-loop passthrough); src/metrics.rs (num_emitted_combinational_tasks); src/bin/tool_matrix.rs (--task-emit-gate, ScenarioSet::TaskEmitSweep, ModuleReport.emitted_combinational_task, saw_combinational_task_emit); book/src/structured-emission.md; docs/decisions/0014-structured-emission-third-surface-combinational-task.md; /tmp/anvil-task-emit-gate-r1/tool_matrix_report.json
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"task_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"gate_struct_weight\":0,\"min_width\":4,\"max_width\":4,\"min_inputs\":2,\"max_inputs\":3,\"min_outputs\":1,\"max_outputs\":1,\"max_depth\":2});json.dump(c,open(\"/tmp/te.json\",\"w\"))" && cargo run --quiet -- --seed 1 --config /tmp/te.json | tee /tmp/te.sv | grep -c "task automatic" && verilator --lint-only /tmp/te.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.6b` — the combinational `task automatic` emit-projection

ANVIL's **third richer-structured emission surface**
(decision [`0014`](../decisions/0014-structured-emission-third-surface-combinational-task.md))
re-renders a selected combinational gate as a `task automatic` called from
`always_comb` instead of an inline `assign <wire> = <op>;`. It is the
decision `0012` single-gate `function automatic` projection expressed as a
*procedural* `task` (an `output` arg written from `always_comb`) rather than a
value-returning `function` — a genuinely distinct elaboration surface.

- **Turn it on:** `Config::task_emit_prob` (serde/config-file only — no CLI
  flag, like `function_emit_prob` / `generate_loop_emit_prob` /
  `soft_union_slice_prob`; default `0.0` ⇒ byte-identical; validated
  `0.0..=1.0`). Set it in a `--config` JSON. A small comb-only shape
  (`flop_prob = 0.0`) makes the one task easy to read.
- **What qualifies:** the gen-time pass
  `crate::ir::task_emit::annotate_task_emit_gates` rolls the probability on the
  seeded RNG per *qualifying* candidate and marks the winners in
  `Module.task_emit_gates` (`BTreeSet<NodeId>`, an emitter-surface annotation —
  flat IR / validators / CSE / `canonical_module_signature` untouched, disjoint
  from the sibling gate sets). The candidate set is **identical to
  `function_emit`**: an ordinary combinational `Gate` with `≥ 1` operand,
  excluding structured selectors (`CaseMux` / `CasezMux` / `ForFold`) and
  `Slice` bit-selects. It runs **after** the `function_emit` and
  `generate_loop` passes (so an already-marked gate is excluded — the four
  emit-projections are mutually exclusive on a gate) and skips Phase-5
  `param_env` modules.
- **Excluded (still emitted inline — nothing retired):** structured selectors
  (their own procedural rendering) and `Slice` (a full-width param would trip
  `-Wall UNUSEDSIGNAL`, the `function_emit` reason).
- **Rendering (`src/emit/sv.rs`):** for a marked gate `<wire>`, a task section
  (after the generate-loop section) emits
  `task automatic <wire>__t(output logic [W-1:0] o, input logic [Wi-1:0] a0, …);
  o = <op over a0..a{n-1}>; endtask` (`render_gate_task_decl`, whose body
  **reuses `render_gate_function_body` verbatim**) + `logic [W-1:0] <wire>__tv;`
  + `always_comb <wire>__t(<wire>__tv, <operand refs>);` (`render_gate_task_call`),
  and the per-gate assign loop rewrites the marked gate's assign to the
  passthrough `assign <wire> = <wire>__tv;` (the `task_emit_gate` defensive
  accessor gates it). This is the **output-var + passthrough** integration:
  `<wire>` stays a continuous-assign net, only the gate's own drive changes
  (the `function_emit` parallel).
- **Behaviour-preserving / combinational only:** the task writes exactly the
  gate's value into `<wire>__tv`, so the module's behaviour is unchanged. A
  flop's `Q` is a leaf parameter — the task never recurses through a register
  edge or instance boundary.
- **Introspection:** `Metrics::num_emitted_combinational_tasks`
  (`= m.task_emit_gates.len()`) surfaces in the `--introspect` `module_metrics`
  (schema `1.10`); default-off reads `0`.
- **Downstream gate:** `tool_matrix --task-emit-gate`
  (`ScenarioSet::TaskEmitSweep`) forces `task_emit_prob = 1.0` over comb-only
  DUTs across all three construction strategies, detects an emitted task via
  `ModuleReport.emitted_combinational_task` (`sv_text.contains("task
  automatic")`), and lights `saw_combinational_task_emit` only when that module
  is accepted by Verilator **and** a clean Yosys (a combinational `task` is
  universally synthesizable like a function, so — unlike the Verilator-only
  `union soft` up-opt — the gate runs the full tool plan: Verilator + both
  Yosys modes + Icarus). Banked clean `/tmp/anvil-task-emit-gate-r1` (3
  scenarios / 12 modules / 12 emitting a task / `coverage_gaps = []` / `12/0`
  Verilator + both Yosys + Icarus compile).

See [[structured-emission-third-surface-combinational-task]] for the decision
(why a `task` third, over nested `generate` / `interface` / `modport`),
[[combinational-function-emit]] and [[generate-loop-emit]] for the sibling
surfaces, and `book/src/structured-emission.md` for the user-facing
walk-through.
