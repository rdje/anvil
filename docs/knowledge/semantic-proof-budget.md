---
id: semantic-proof-budget
title: Bounded semantic proofs use support, node, and work budgets
answers:
  - "how many endpoint bits can semantic gate merge prove"
  - "why does the semantic proof stop at 12 bits"
  - "what is the egraph truth table budget"
  - "why do larger semantic cones fall back to structural proof"
date: 2026-06-05
status: current
tags: [identity, factorization, egraph, performance]
evidence: src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

ANVIL's bounded semantic proof is limited by endpoint support, cone
size, and total truth-table work. Merge proofs allow up to 12
endpoint-support bits and 128 cone nodes only when
`assignment_count * cone_node_count <= 131072`; cleanup exact proofs use
the same 12-bit support ceiling with a stricter 64-node / 65536-work
budget and no more than three canonical endpoints. Wider or deeper
candidates fall back to structural identity rather than running an
unbounded semantic solver.
