---
id: bounded-semantic-module-identity
title: Bounded pure-combinational module semantic identity can merge
answers:
  - "can ANVIL merge semantically equivalent modules"
  - "what does hierarchy_semantic_module_dedup do"
  - "why does semantic module dedup require matching port ids"
  - "does ANVIL merge stateful modules by semantic equivalence"
date: 2026-06-05
status: current
tags: [identity, hierarchy, factorization, module-dedup]
evidence: src/ir/dedup.rs; src/metrics.rs; book/src/hierarchy.md; book/src/knobs.md; DEVELOPMENT_NOTES.md
---

ANVIL has a default-off `hierarchy_semantic_module_dedup` pass for
non-top pure-combinational, state-free, concrete modules under
`identity_mode = node-id` and effective `factorization_level = e-graph`.
It merges only when `(PortId, width)` interfaces match and a bounded
whole-module truth-table proof matches: <= 12 emitted input bits, <= 128
reachable output-cone nodes, and <= 128-bit outputs. Supported proof
classes are instance-free modules and wrappers with <= 8 child instances
whose children are also inside the proof boundary. Stateful, memory/FSM,
parameterized, aggregate-projected, mismatched-interface, too-large,
too-many-instance, out-of-bound child, and ancestor/descendant wrapper
groups are skipped. Port IDs are part of the proof because instance
rewrites preserve parent-side port-id bindings.
