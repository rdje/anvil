---
id: combinational-semantic-endpoint-fold
title: Bounded semantic gate proofs can fold to existing endpoints
answers:
  - "can ANVIL fold a gate to an input under egraph"
  - "does a and b or not b simplify to a"
  - "what happens to helper endpoints that cancel out"
  - "can semantic gate merge target non-gate nodes"
date: 2026-06-05
status: current
tags: [identity, factorization, egraph, roadmap]
evidence: src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

Under `identity_mode = node-id` and effective `factorization_level =
e-graph`, ANVIL's bounded semantic gate proof may rewire a gate to an
earlier non-gate canonical node. After truth-table enumeration, endpoints
that do not affect the output are minimized out of the proof, so
`a & (b | !b)` proves equal to the existing `a` endpoint. Different live
canonical roots remain distinct; the analogous cone over `c` rewires to
`c`, not to `a`.
