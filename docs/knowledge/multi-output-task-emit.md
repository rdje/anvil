---
id: multi-output-task-emit
title: How ANVIL emits a multi-output `task automatic` — the `multi_output_task_emit_prob` co-supported-pair emit-projection
answers:
  - "how do I make ANVIL emit a multi-output task"
  - "how do I turn on multi_output_task_emit_prob"
  - "how do I get ANVIL to co-emit two gates in one task with multiple outputs"
  - "how is the multi-output task different from the single-gate task_emit surface"
  - "which gate pairs get co-emitted as one task"
  - "why must the two gates share a non-constant operand"
  - "why must the two gates be fan-in-independent"
  - "what is the co-supported sink"
  - "what is num_emitted_multi_output_tasks"
  - "what does tool_matrix --multi-output-task-gate prove"
  - "how is the multi-output task emit surface proven downstream-clean"
  - "where is multi-output task emission implemented"
  - "does multi-output task emission change the emitted RTL behaviour"
  - "what is the __mt detection token"
date: 2026-06-22
status: current
tags: [structured-emission, task, multi-output, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/multi_output_task_emit.rs (annotate_multi_output_task_groups, the one-roll-per-leader pairing, shares_nonconst_operand, in_fanin bounded backward DFS); src/config.rs (multi_output_task_emit_prob + the --multi-output-task-emit-prob CLI flag); src/gen/mod.rs (generate_module + generate_design rolls, run after task_emit before cone_function); src/ir/cone_function_emit.rs (sibling_marked extended to exclude members); src/emit/sv.rs (multi-output task decl/call section + multi_output_task_params/render_multi_output_task_decl/render_multi_output_task_call reusing render_cone_gate_expr with an empty interior_set + the per-gate passthrough); src/metrics.rs (num_emitted_multi_output_tasks); src/introspect/mod.rs (SCHEMA_VERSION 1.14); src/bin/tool_matrix.rs (--multi-output-task-gate, ScenarioSet::MultiOutputTaskSweep, ModuleReport.emitted_multi_output_task, saw_multi_output_task_emit); book/src/structured-emission.md; docs/decisions/0025-structured-emission-sixth-surface-multi-output-task.md; /tmp/anvil-multi-output-task-gate-r1/tool_matrix_report.json
reverify: 'cargo run --quiet -- --seed 3 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"multi_output_task_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"terminal_reuse_prob\":0.9,\"min_inputs\":3,\"max_inputs\":3,\"min_outputs\":2,\"max_outputs\":2,\"min_width\":4,\"max_width\":4,\"max_depth\":1});json.dump(c,open(\"/tmp/mt.json\",\"w\"))" && cargo run --quiet -- --seed 3 --config /tmp/mt.json | tee /tmp/mt.sv | grep -c "__mt(" && verilator --lint-only /tmp/mt.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.12b` — the multi-output `task automatic` emit-projection

ANVIL's **sixth richer-structured emission surface**
(decision [`0025`](../decisions/0025-structured-emission-sixth-surface-multi-output-task.md))
co-emits a **co-supported pair** of combinational gates as **one** `task
automatic` with several `output` arguments and a **deduplicated** `input` list,
instead of two inline `assign`s. It is a **generalization of the
[[combinational-task-emit]] single-gate surface** (decision `0014`, which wrapped
one gate with one `output`) from one output to several.

- **Turn it on:** `Config::multi_output_task_emit_prob` (the
  `--multi-output-task-emit-prob` CLI flag, or `--config` JSON, like
  `task_emit_prob` / `cone_function_emit_prob`; default `0.0` ⇒ byte-identical;
  validated `0.0..=1.0`). It is a **separate** knob from `task_emit_prob` so the
  shipped single-gate surface stays byte-identical (reusing it was rejected). A
  small comb-only shape (`flop_prob = 0.0`, high `terminal_reuse_prob`, shallow
  `max_depth`, `min_outputs ≥ 2`) makes the one pair easy to read.
- **What qualifies (`src/ir/multi_output_task_emit.rs`):** the gen-time pass
  `annotate_multi_output_task_groups` scans admissible, non-sibling-marked gates
  (the single-gate `task_emit` candidate set — non-structured, non-`Slice`, `≥ 1`
  operand) in ascending `NodeId`. For each ungrouped **leader** it rolls the
  probability **once** on the seeded RNG; on a hit it pairs the leader with the
  next ungrouped candidate that (a) **shares a non-constant direct operand** and
  (b) is **mutually fan-in-independent**. The pair lands in
  `Module.multi_output_task_groups` (`BTreeMap<NodeId, Vec<NodeId>>`, leader →
  partner members; an emitter-surface annotation — flat IR / validators / CSE /
  `canonical_module_signature` untouched). The pass runs **after** `task_emit` and
  **before** `cone_function` (whose `sibling_marked` excludes members), so the six
  emit-projections are mutually exclusive on a gate; `param_env` modules are
  skipped. The first cut groups a **pair**; wider co-supported groups are a
  recorded follow-up.
- **The shared-non-constant-operand rule (the co-supported sink):** the two gates
  must share at least one non-constant operand, so the deduplicated task genuinely
  has a **shared input formal feeding both outputs**. A shared *constant* folds
  inline as a literal (never a formal), so it does not count — without a real
  shared formal the task would be merely two unrelated tasks fused, with no new
  elaboration interaction.
- **The fan-in-independence rule (soundness):** neither member may lie in the
  other's transitive fan-in (`in_fanin`, a bounded backward DFS over `Node::Gate`
  operands). If it did, the member's net — driven by the shared task's `<wire>__mtv`
  passthrough — would feed, through gates *outside* the task, into a direct operand
  the task reads, closing a combinational cycle through the single `always_comb`
  call (a Verilator `UNOPTFLAT`). Independence makes the co-emitted task cycle-free
  by construction. The IR's operand-topological `NodeId` invariant
  (`Module::intern_gate` appends after its operands) makes one direction automatic,
  but both are checked for robustness.
- **Rendering (`src/emit/sv.rs`):** for a group keyed by `<leader>`,
  `render_multi_output_task_decl` emits `task automatic <leader>__mt(output logic
  […] o0, output logic […] o1, input logic […] a0, …); o0 = …; o1 = …; endtask`
  where the inputs are the **deduplicated** non-constant operands of both members
  (ascending `NodeId`, `multi_output_task_params`), and each `oj` is the member's
  operation over those formals. `render_multi_output_task_call` emits one `logic
  <wire>__mtv;` per member + one `always_comb <leader>__mt(<m0>__mtv, …, <param
  refs>);`. The body **reuses `render_cone_gate_expr` with an empty `interior_set`**,
  so each operand resolves to a folded `Constant` literal or its boundary parameter
  `a{i}` — exactly the shared-formal semantics. Each member's net then becomes the
  passthrough `assign <wire> = <wire>__mtv;` — members **keep** their module wires
  (co-equal roots, not absorbed, so no use-count rule; DAG-shared members are fine).
- **Behaviour-preserving / combinational only:** each output is the member gate's
  exact operation, so the module's behaviour is unchanged. Each member is a
  combinational gate; a flop `Q` is a leaf formal.
- **Introspection:** `Metrics::num_emitted_multi_output_tasks`
  (`= m.multi_output_task_groups.len()`) surfaces in the `--introspect`
  `module_metrics` (schema `1.14`); default-off reads `0`. Separate from
  `num_emitted_combinational_tasks` (the single-gate surface).
- **Downstream gate:** `tool_matrix --multi-output-task-gate`
  (`ScenarioSet::MultiOutputTaskSweep`) forces `multi_output_task_emit_prob = 1.0`
  over comb-only DUTs across all three construction strategies (with a high
  `terminal_reuse_prob = 0.6` + shallow `max_depth = 2` + `min_outputs ≥ 2` so
  co-supported, fan-in-independent pairs exist), detects an emitted task via
  `ModuleReport.emitted_multi_output_task` (`sv_text.contains("__mt(")`, distinct
  from the single-gate `"__t("` and the cone `"__cf("`), and lights
  `saw_multi_output_task_emit` only when that module is accepted by Verilator
  **and** a clean Yosys (a multi-output task is universally synthesizable like a
  single-gate task, so the gate runs the full plan: Verilator + both Yosys modes +
  Icarus). Banked clean `/tmp/anvil-multi-output-task-gate-r1` (3 scenarios / 12
  modules / 6 emitting a multi-output task / `coverage_gaps = []` / `12/0`
  Verilator + both Yosys + Icarus compile).

See [[structured-emission-sixth-surface-multi-output-task]] for the decision (why a
multi-output task sixth — the deferred runner-up from the fifth-surface probe,
chosen for the genuinely-new "multiple `output` formals + a shared input formal"
elaboration interaction), [[combinational-task-emit]] for the single-gate surface
it generalizes, [[multi-gate-cone-function-emit]] for the cone primitives its body
rendering reuses, and `book/src/structured-emission.md` for the user-facing
walk-through.
