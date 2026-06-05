---
id: memory-identity-boundary
title: Inferrable memories stay instance-local under full factorization
answers:
  - "why doesn't ANVIL merge duplicate memories"
  - "does full factorization merge memories"
  - "are memory blocks state by instance"
  - "why can FSMs merge but memories stay separate"
date: 2026-06-05
status: current
tags: [identity, memory, factorization, roadmap]
evidence: src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

ANVIL deliberately does not merge current inferrable `Memory` blocks
under `identity_mode = node-id`, even when two memories share identical
write/read source cones. The memory template has no reset-defined array
contents, so equal cones do not prove equal stored state.

`MEMORY-IDENTITY-BOUNDARY.1` adds a regression that drives two
independent memories through the node-id/e-graph state-sharing boundary
and compaction. Both `Memory` blocks and both `MemRead` leaves must
remain. Generated FSMs can merge because they reset to state 0 and have
explicit transition/output tables; memories remain state-by-instance
until a future task adds stronger reset/init or equivalence evidence.
