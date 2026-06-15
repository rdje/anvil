---
id: signoff-knob-sweep-gate
title: tool_matrix --signoff-knob-sweep-gate promotes four previously-unswept knobs into explicit axes with provable coverage facts
answers:
  - "what does tool_matrix --signoff-knob-sweep-gate do"
  - "which generator knobs does the signoff knob-sweep gate cover"
  - "how does ANVIL prove operand_duplication_rate fired"
  - "which metric proves mux_arm_duplication_rate fired"
  - "how does ANVIL prove aggregate_array_prob selected an array-packed aggregate"
  - "how does ANVIL prove a memory module and an FSM module in one design"
  - "what is num_operator_gates_with_duplicate_operands"
  - "which saw_* facts does the signoff knob-sweep gate require"
  - "why is the mux-arm-duplication scenario a single-module DUT not a wrapper design"
  - "where is the banked signoff knob-sweep report"
date: 2026-06-15
status: current
tags: [signoff, tool-matrix, coverage, adversarial, sweep, duplication, aggregate, memory, fsm]
evidence: src/bin/tool_matrix.rs (ScenarioSet::SignoffKnobSweep, build_signoff_knob_sweep_scenarios, compute_coverage_gaps); src/metrics.rs (num_operator_gates_with_duplicate_operands); DEVELOPMENT_NOTES.md (SIGNOFF-AUTOMATION-EXPANSION.2b); /tmp/anvil-signoff-knob-sweep-r1/tool_matrix_report.json
reverify: cargo run --release --bin tool_matrix -- --signoff-knob-sweep-gate --yosys-mode both --out /tmp/anvil-signoff-knob-sweep-check
---

# `tool_matrix --signoff-knob-sweep-gate` (SIGNOFF-AUTOMATION-EXPANSION.2b)

The first richer-knob-sweep increment of the signoff-automation lane.
It promotes four generator knobs that previously fired only by chance
inside motif-heavy profiles into explicit first-class `tool_matrix`
scenario axes, so each fires **by construction** and is proved from one
realized metric (ROADMAP steering gap 3 — remove hidden bias). Opt-in,
mutually exclusive with the phase gates, auto-enables coverage-gap
failure. Scenario set `ScenarioSet::SignoffKnobSweep`: four focused
scenarios across all three construction strategies (12 total).

| Knob | Scenario shape | Coverage fact | Proving metric |
|---|---|---|---|
| `operand_duplication_rate` | single-module DUT, arith-only tiny pool | `saw_operand_duplication` | `num_operator_gates_with_duplicate_operands` (new; counts `Add`/`Mul` gates with a repeated operand slot; RTL byte-identical) |
| `mux_arm_duplication_rate` | single-module DUT, 2-arm comb-mux tiny pool, **default `flop_prob`** | `saw_mux_arm_duplication` | `num_muxes_degenerate` |
| `aggregate_array_prob` | depth-1 wrapper, `aggregate_prob=1.0`+`aggregate_array_prob=1.0`, **uniform width** | `saw_array_packed_aggregate_design` | `num_array_packed_aggregate_modules` |
| memory×fsm interplay | depth-1 wrapper, `memory_prob=0.5`+`fsm_prob=1.0`, 6 leaves | `saw_memory_fsm_interplay_design` | `num_memory_modules > 0 && num_fsm_modules > 0` |

Gotchas (empirically established, see `DEVELOPMENT_NOTES.md` `.2b`):
the two duplication scenarios are **single-module DUTs**, not wrapper
designs — the wrapper-lane leaf builder does not hit the degenerate-mux
path; the mux-dup scenario keeps the **default `flop_prob`** because
forcing it to `0.0` collapses `num_muxes_degenerate` to ~0; memory×fsm
needs `memory_prob` strictly in `(0,1)` because per-leaf memory-vs-FSM
selection is mutually exclusive (`memory_prob` is rolled first and
returns early, `src/gen/module.rs`). Banked downstream-clean at
`/tmp/anvil-signoff-knob-sweep-r1` (12 scenarios, 48 modules,
`coverage_gaps = []`, `48/0` Verilator + both Yosys). Default-off /
byte-identical; nothing retired. See [[signoff-automation-first-increment]]
(decision `0006`).
