---
id: fsm-mealy-outputs
title: How ANVIL emits a Mealy FSM output — the `fsm_mealy_prob` knob
answers:
  - "how do I make ANVIL emit a Mealy FSM"
  - "how do I turn on fsm_mealy_prob"
  - "how do I get a Mealy state machine output from ANVIL"
  - "which knob makes the FSM output depend on the input"
  - "what command emits a Mealy FSM"
  - "how is an ANVIL FSM output Moore vs Mealy"
  - "is the ANVIL FSM output Moore or Mealy by default"
  - "what is num_mealy_fsm_modules"
  - "what does the phase6_mealy_fsm tool_matrix scenario prove"
  - "what is saw_mealy_fsm_design"
  - "is the Mealy FSM extension default-off and byte-identical"
  - "where is Mealy FSM output emission implemented"
date: 2026-06-22
status: current
tags: [fsm, mealy, moore, sequential, knob, emission, downstream, valid-by-construction, rules-first, matrix-gate, introspection]
evidence: src/config.rs (fsm_mealy_prob + --fsm-mealy-prob + config_category "fsm"); src/gen/module.rs (build_fsm_block rolls mealy_outputs); src/ir/types.rs (Fsm.mealy_outputs + is_mealy); src/ir/validate.rs (mealy_outputs shape/mask check); src/ir/compact.rs (Mealy FSMs excluded from merge_equivalent_fsms); src/emit/sv.rs (nested case(state)->case(sel) Mealy output decode); src/metrics.rs (num_mealy_fsm_modules); src/bin/tool_matrix.rs (phase6_mealy_fsm scenario, saw_mealy_fsm_design, --phase4-hierarchy-gate); book/src/sequential.md "FSM outputs: Moore vs Mealy"; docs/decisions/0024-mealy-fsm-outputs.md
reverify: 'cargo run --quiet -- --seed 3 --fsm-prob 1.0 --fsm-mealy-prob 1.0 --min-width 2 --max-width 4 --flop-prob 0.0 --constant-prob 0.0 --max-depth 1 | tee /tmp/mealy.sv | grep -c "case (sel" && verilator --lint-only /tmp/mealy.sv && echo CLEAN'
---

# `CAPABILITY-BREADTH-EXPANSION.2b` — the Mealy FSM output extension

ANVIL's generated FSM (the Phase-6 `fsm_prob` motif) is a **Moore** machine by
default: its output is decoded from the current state alone. The opt-in
`fsm_mealy_prob` knob makes the output **Mealy** — it depends on the current
*input* (`sel`) as well as the current state
(decision [`0024`](../decisions/0024-mealy-fsm-outputs.md)).

- **Turn it on:** `Config::fsm_mealy_prob` — the `--fsm-mealy-prob` CLI flag, or
  `--config` JSON / MCP (`config_category` `"fsm"`; default `0.0` ⇒ Moore,
  byte-identical; validated `0.0..=1.0`). It only has an effect when an FSM block
  is built, so pair it with `fsm_prob > 0`. A single-module comb-scaffold shape
  (`--flop-prob 0.0 --max-depth 1`) makes the decode easy to read:

  ```bash
  cargo run --release -- --seed 3 --fsm-prob 1.0 --fsm-mealy-prob 1.0 \
        --min-width 2 --max-width 4 --flop-prob 0.0 --constant-prob 0.0 --max-depth 1
  ```

- **What it builds:** `src/gen/module.rs`'s `build_fsm_block` rolls the
  probability on the seeded RNG; when it fires it fills
  `Fsm.mealy_outputs: Option<Vec<Vec<u128>>>` — a per-`(state, sel_value)`
  constant output table mirroring `transitions` (`None` ⇒ Moore, the
  byte-identical path). `Fsm::is_mealy()` is `mealy_outputs.is_some()`.
- **What it emits (`src/emit/sv.rs`):** a second nested decode —
  `case (state)` → `case (sel)` — driving the **opaque** `Node::FsmOut` leaf, so
  the output reads the input-dependent `sel` cone. The state register stays
  Moore-clocked (async reset to state 0, next-state from the transition table);
  only the output decode is new.
- **Valid-by-construction / nothing retired:** no new IR node (the output is
  still `Node::FsmOut`), rules-first (selection at construction time, no
  generate-then-filter), `validate.rs` checks the table shape (one row per state,
  `1<<sel_width` entries, masked to the output width), and a Mealy FSM is
  conservatively excluded from `merge_equivalent_fsms` (sound — the Moore-only
  dedup keying does not cover the Mealy table yet).
- **Introspection:** `DesignMetrics::num_mealy_fsm_modules` (a filter over
  `Module::fsms` for `is_mealy()`; `<= num_fsm_modules`) surfaces in the
  `--introspect` `design_metrics` at schema **`1.13`**; default-off reads `0`.
- **Downstream gate:** `tool_matrix`'s `phase6_mealy_fsm` scenario
  (`fsm_mealy_prob = 1.0`, folded into the `Phase4Hierarchy` set under
  `--phase4-hierarchy-gate`) lights `saw_mealy_fsm_design` only when a Mealy FSM
  design is downstream-clean. Mealy is universally synthesizable, so it takes the
  full tool plan (Verilator + both Yosys modes + Icarus) — no Verilator-only
  carve-out like the `union soft` up-opt.

See [[mealy-fsm-outputs]] for the design decision (the `(state_q, sel)` output
model, why `.2` advanced ahead of the SV-up-opt strand) and
`book/src/sequential.md` "FSM outputs: Moore vs Mealy" for the user-facing
walk-through.
