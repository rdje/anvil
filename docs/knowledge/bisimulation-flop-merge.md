---
id: bisimulation-flop-merge
title: Opt-in bounded bisimulation merges mutually-recursive flops (default-off)
answers:
  - "can ANVIL merge mutually-recursive registers"
  - "does ANVIL merge swapped-feedback flops"
  - "what does bisimulation_flop_merge do"
  - "how does ANVIL merge sequentially equivalent flops beyond exact self-hold"
  - "is the bisimulation flop merge on by default"
  - "what is merge_bisimilar_flops"
  - "why are resetless flops excluded from bisimulation merge"
  - "what metric counts bisimulation flop merges"
  - "how is the bisimulation flop merge proven sound and downstream-clean"
  - "what is the bisimulation flop merge bucket cap"
date: 2026-06-15
status: current
tags: [identity, sequential, factorization, bisimulation, coinduction, flop-merge]
evidence: src/ir/compact.rs (merge_bisimilar_flops, finalize_flop_merge, canonical_flop_endpoint); book/src/factorization.md; book/src/knobs.md; DEVELOPMENT_NOTES.md; docs/decisions/0007-identity-deepening-first-extension.md
reverify: "ANVIL_DUMP_BISIM_SV=1 cargo test --lib merge_bisimilar_flops_merges_mutual_swap_registers, then lint /tmp/anvil-bisim-merged.sv with verilator --lint-only -Wall + yosys (both modes) + iverilog -g2012"
---

The opt-in `Config::bisimulation_flop_merge` knob (default `false`,
`IDENTITY-DEEPENING.2b`) runs `merge_bisimilar_flops` after the exact
`merge_equivalent_flops` pass and before the FSM merge. It is a bounded
greatest-fixpoint partition refinement (Kanellakis–Smolka): bucket flops by
`(width, reset_kind, reset_val, clock_domain)`, then keep two flops in one
class iff their D-cones — with every `FlopQ` endpoint rewritten to its current
class representative (the quotient signature) — are proven equal by the same
bounded 12-bit / 128-node / 131072-work endpoint proof, until the partition is
stable. At the fixpoint the partition is a bisimulation, sound by
reset-base-case coinduction.

This lifts the mutually-recursive-register / swapped-feedback class the exact
pass provably cannot prove (each exact D-cone keys a *different* concrete
`FlopQ` endpoint). It strictly generalizes — and does not retire — the exact
self-hold and same-endpoint classes
([[reset-defined-self-hold-flop-identity]]). Active only under
`identity_mode = node-id` with effective `factorization_level = e-graph`;
`identity_mode = relaxed` is the real off-switch. **Resetless flops are
excluded** (no reset ⇒ no provable equal initial state ⇒ no base case), which
preserves the resetless-self-hold boundary. Over-budget cones take the
structural fallback. The count is surfaced as
`Metrics::bisimulation_flops_merged`. Default-off ⇒ emitted RTL is
byte-identical. Worked example: the mutual swap of two equal-reset registers
collapses to one self-holding register, downstream-clean across Verilator,
both Yosys modes, and Icarus. Whole-module sequential equivalence and
retimed-state equivalence stay open ([[identity-deepening-first-extension]]).
