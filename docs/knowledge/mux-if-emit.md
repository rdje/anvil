---
id: mux-if-emit
title: How ANVIL emits a procedural `if`/`else` — the `mux_if_emit_prob` `always_comb` projection of a 2:1 `Mux`
answers:
  - "how do I make ANVIL emit a procedural if/else"
  - "how do I turn on mux_if_emit_prob"
  - "how do I get ANVIL to emit an always_comb if else block"
  - "how do I render a mux as a procedural conditional instead of a ternary"
  - "can ANVIL emit a 2:1 mux as an if/else statement"
  - "which gates become a procedural if/else"
  - "how is the procedural if/else different from CaseMux / casez"
  - "what is the seventh structured emission surface"
  - "what is num_emitted_mux_if_blocks"
  - "what does tool_matrix --mux-if-gate prove"
  - "how is the mux-if emit surface proven downstream-clean"
  - "where is procedural if/else emission implemented"
  - "does procedural if/else emission change the emitted RTL behaviour"
  - "what is the __cv detection token"
  - "why does the mux-if gate force comb_mux_encoding_prob"
date: 2026-06-22
status: current
tags: [structured-emission, if-else, procedural, mux, always-comb, emission, knob, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/ir/mux_if_emit.rs (annotate_mux_if_gates — the GateOp::Mux candidate predicate excluding all six sibling marks, run last; gate_qualifies; collect-then-roll one gen_bool per candidate); src/config.rs (mux_if_emit_prob + the --mux-if-emit-prob CLI flag); src/main.rs (the flag overlay); src/gen/mod.rs (generate_module + generate_design rolls, run after cone_function); src/ir/types.rs (Module.mux_if_gates: BTreeSet<NodeId>); src/emit/sv.rs (the procedural-block section — logic [w-1:0] <wire>__cv; + always_comb if/else over node_ref operand refs + the gate-assign-loop <wire>__cv passthrough); src/metrics.rs (num_emitted_mux_if_blocks = m.mux_if_gates.len()); src/introspect/mod.rs (SCHEMA_VERSION 1.15); src/bin/tool_matrix.rs (--mux-if-gate, ScenarioSet::MuxIfSweep, mux_if_focus_config, ModuleReport.emitted_mux_if, saw_mux_if_emit); book/src/structured-emission.md; docs/decisions/0027-structured-emission-seventh-surface-procedural-if-else.md; /tmp/anvil-mux-if-gate-r1/tool_matrix_report.json (12/12 modules emit a __cv block / 215 blocks)
reverify: 'cargo run --quiet -- --seed 1 --dump-config > /tmp/c.json && python3 -c "import json;c=json.load(open(\"/tmp/c.json\"));c.update({\"mux_if_emit_prob\":1.0,\"flop_prob\":0.0,\"constant_prob\":0.0,\"comb_mux_prob\":1.0,\"comb_mux_encoding_prob\":1.0,\"min_inputs\":3,\"max_inputs\":3,\"min_outputs\":1,\"max_outputs\":1,\"min_width\":4,\"max_width\":4,\"max_depth\":1,\"min_mux_arms\":2,\"max_mux_arms\":2});json.dump(c,open(\"/tmp/mi.json\",\"w\"))" && cargo run --quiet -- --seed 1 --config /tmp/mi.json | tee /tmp/mi.sv | grep -c "__cv" && iverilog -g2012 -o /tmp/mi.vvp /tmp/mi.sv && echo CLEAN'
---

# `STRUCTURED-EMISSION-EXPANSION.15b` — the procedural `if`/`else` emit-projection

ANVIL's **seventh richer-structured emission surface**
(decision [`0027`](../decisions/0027-structured-emission-seventh-surface-procedural-if-else.md))
re-expresses a 2:1 `Mux` gate — rendered today as the continuous-assign ternary
`assign <wire> = (sel) ? (a) : (b);` — as a **procedural `always_comb` `if`/`else`**
block writing a per-gate `<wire>__cv` output var, the net driven from it by a
passthrough `assign`. It is the lane's **first procedural-conditional** shape (the six
prior surfaces are `function` / `task` / `generate` projections; the `Mux` is a
continuous-assign ternary; `CaseMux` / `CasezMux` are `case` / `casez`). It reuses the
[[combinational-task-emit]] single-gate-task **output-var + passthrough** mechanism,
but emits a bare `always_comb` `if`/`else` rather than a `task` call.

- **Turn it on:** `Config::mux_if_emit_prob` (the `--mux-if-emit-prob` CLI flag, or
  `--config` JSON, like `task_emit_prob` / `cone_function_emit_prob`; default `0.0` ⇒
  byte-identical; validated `0.0..=1.0`). It is a **separate** knob from
  `task_emit_prob` / `function_emit_prob` so the shipped surfaces stay byte-identical
  (reusing one was rejected). A small comb-only shape with the comb-mux block forced
  down its **encoded chained-ternary** path (`comb_mux_prob = 1.0` +
  `comb_mux_encoding_prob = 1.0`, which is what builds plain `GateOp::Mux` gates; the
  one-hot path emits `AND`/`OR` and yields **no** `Mux`) makes the blocks easy to read.
- **What qualifies (`src/ir/mux_if_emit.rs`):** the gen-time pass
  `annotate_mux_if_gates` scans every `GateOp::Mux` gate with exactly three operands (a
  one-bit selector by IR invariant) and rolls the probability **once** per candidate on
  the seeded RNG. Because the pass runs **last** (after `function_emit` / `generate_loop`
  / `task_emit` / `multi_output_task` / `cone_function` / `soft_union`), `gate_qualifies`
  also **excludes** any gate already claimed by one of those six sibling projections, so
  a gate is projected by **at most one** of the seven surfaces. Marked gates land in
  `Module.mux_if_gates` (`BTreeSet<NodeId>` — a marked `Mux` carries no payload; its
  operands are read straight from the node). An emitter-surface annotation — the flat IR
  / validators / CSE / `canonical_module_signature` are untouched; `param_env` modules
  are skipped.
- **Rendering (`src/emit/sv.rs`):** for each marked `Mux` `g = Mux[sel, a, b]` of width
  `W`, a procedural-block section emits `logic [W-1:0] <g>__cv;` + `always_comb begin if
  (<sel>) <g>__cv = <a>; else <g>__cv = <b>; end` (operand refs via the same `node_ref`
  resolver the inline ternary uses; a 1-bit gate drops the `[W-1:0]`), and the
  gate-assign loop emits the passthrough `assign <g> = <g>__cv;` instead of the inline
  ternary. The `<g>` net **stays a net** — only its drive changes (minimal blast radius;
  every downstream consumer of `<g>` is unchanged).
- **Behaviour-preserving / combinational only:** the `if`/`else` writes the gate's exact
  value (`sel == 1 ⇒ a` operand 1, `sel == 0 ⇒ b` operand 2 — the ternary's operand
  mapping), so the module's behaviour is unchanged. The `Mux` is combinational; its
  operand refs are leaves of the block, which reads only those refs and writes only
  `<g>__cv` — exactly the inline ternary's read/write set, so there is no cycle risk.
- **Introspection:** `Metrics::num_emitted_mux_if_blocks` (`= m.mux_if_gates.len()`)
  surfaces in the `--introspect` `module_metrics` (schema `1.15`); default-off reads `0`.
- **Downstream gate:** `tool_matrix --mux-if-gate` (`ScenarioSet::MuxIfSweep`) forces
  `mux_if_emit_prob = 1.0` over comb-only DUTs across all three construction strategies.
  Its `mux_if_focus_config` is **Mux-biased**: it forces `comb_mux_prob = 0.9` **and**
  `comb_mux_encoding_prob = 1.0` so the comb-mux block takes the chained-ternary path
  that builds plain `GateOp::Mux` gates (the one-hot path would emit no `Mux`). It
  detects an emitted block via `ModuleReport.emitted_mux_if` (`sv_text.contains("__cv")`,
  distinct from the call tokens `"__f("` / `"__t("` / `"__mt("` / `"__cf("` and the var
  tokens `"__tv"` / `"__mtv"`), and lights `saw_mux_if_emit` only when that module is
  accepted by Verilator **and** a clean Yosys (a procedural `always_comb if/else` is
  universally synthesizable, so the gate runs the full plan: Verilator + both Yosys modes
  + Icarus). Banked clean `/tmp/anvil-mux-if-gate-r1` (3 scenarios / 12 modules / 12
  emitting a block / 215 blocks / `coverage_gaps = []` / `12/0` Verilator + both Yosys +
  Icarus compile). Across the bank the projection adds **zero** new Verilator `-Wall`
  warnings versus the knob-off build, and an `iverilog` simulation proves it
  bit-identical to the inline ternaries it replaces.

See [[structured-emission-seventh-surface-procedural-if-else]] for the decision (why a
procedural `if`/`else` seventh — the first procedural-conditional shape, chosen over
nested/multi-level `generate`, which has no routine by-construction source, and
`interface` / `modport`, empirically disqualified), [[combinational-task-emit]] for the
single-gate task surface whose output-var + passthrough mechanism it reuses, and
`book/src/structured-emission.md` for the user-facing walk-through. The N-way `CaseMux`
→ `if`/`else if` priority chain is the recorded follow-up.
