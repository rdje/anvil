---
id: sequential-module-dedup
title: Opt-in hierarchy_sequential_module_dedup merges whole sequentially-equivalent stateful leaf modules (default-off)
answers:
  - "what does hierarchy_sequential_module_dedup do"
  - "what does dedup_sequential_modules do"
  - "how do I merge sequentially-equivalent stateful modules"
  - "is hierarchy_sequential_module_dedup default-off"
  - "what does modules_sequentially_equivalent do"
  - "how does ANVIL prove two whole stateful leaf modules sequentially equivalent in code"
  - "what metric counts sequentially-duplicate module pairs"
  - "what is sequential_module_proof_signatures"
  - "what is num_sequentially_duplicate_module_pairs"
  - "which modules are excluded from sequential module dedup"
  - "what introspection schema version adds the sequential module proof metric"
date: 2026-06-16
status: current
tags: [identity, sequential, factorization, bisimulation, coinduction, module-dedup, hierarchy, metrics]
evidence: src/ir/dedup.rs (dedup_sequential_modules, group_sequentially_equivalent_modules); src/ir/compact.rs (modules_sequentially_equivalent, build_combined_module, bisimulation_partition); src/metrics.rs (sequential_module_proof_signatures, num_sequentially_duplicate_module_pairs); src/gen/mod.rs; book/src/factorization.md (§9b); book/src/hierarchy.md; docs/decisions/0008-identity-deepening-whole-module-sequential-equivalence.md
reverify: "cargo test --lib sequential   (the proof + metric + bank tests); downstream bank: ANVIL_DUMP_SEQ_MODULE_SV=1 cargo test --lib sequential_dedup_merged_design_is_downstream_clean, split the dump per module, then lint with verilator --lint-only -Wall + yosys (both modes) + iverilog -g2012"
---

The opt-in `Config::hierarchy_sequential_module_dedup` knob (default `false`,
`IDENTITY-DEEPENING.3b.2b`) runs `dedup_sequential_modules` in `generate_design`
after the structural (`dedup_modules`) and combinational
(`dedup_semantic_modules`) module-dedup passes, gated on `identity_mode = node-id`
with effective `factorization_level = e-graph`. It is the **sequential
generalization** of the combinational module dedup — the zero-flop case *is* the
combinational one — and is added beside it, retiring nothing.

The equivalence verdict is `compact::modules_sequentially_equivalent(a, b)`: it
materializes a combined module `a.nodes ++ b.nodes` / `a.flops ++ b.flops` (B's
ids offset, B's `PrimaryInput {port, width}` kept so A/B inputs unify for free in
the shared endpoint vocabulary), reuses the [[bisimulation-flop-merge]]
`bisimulation_partition` on the **union** state, then proves every output drive
cone equal under the final quotient by the same bounded 12-bit / 128-node /
131072-work proof. Sound by coinduction (interface base case + reset base case +
stable quotient transition + equal output cones), the same discipline as the
flop-level merge, now across two machines. No flop bijection is required.

Candidate grouping = a cheap structural pre-filter (interface + flop multiset)
bucket + greedy-by-representative grouping (sound because sequential equivalence
is transitive), factored into the non-mutating
`group_sequentially_equivalent_modules` shared by the pass and the metric.
**First cut excludes** modules with memories, FSMs, child instances, width
parameters, packed aggregates, multiple clock domains, or any resetless flop (no
reset ⇒ no base case) — each a named, excluded boundary. Over-budget / interface
mismatch / unprovable pairs conservatively fail to merge (never a guess).
Default-off ⇒ emitted RTL is byte-identical; `--identity-mode relaxed` is the real
off-switch.

The merge is RTL-invisible-instrumented by
`DesignMetrics::sequential_module_proof_signatures` (one class id per in-boundary
module — equal ids were proven sequentially equivalent) and
`num_sequentially_duplicate_module_pairs` (reducible to 0 by the pass), surfaced
in `--introspect` design documents at **schema `1.4`** (the additive MINOR bump
that added these two `DesignMetrics` fields). Banked downstream-clean: two
two-cycle delay lines — one built with a redundant `~~in` cone so they are
structurally distinct yet sequentially equivalent — collapse to one definition,
clean across Verilator, both Yosys modes, and Icarus. Deeper cases (modules with
memories / FSMs / instances / params / aggregates / multi-clock, retimed-state
equivalence) stay open ([[identity-deepening-whole-module-sequential-equivalence]]).
