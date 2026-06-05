---
id: endpoint-identity-boundary
title: Semantic gate merging preserves canonical leaf endpoints
answers:
  - "can same-shape cones over different inputs merge"
  - "does semantic gate merge ignore endpoint identity"
  - "what does endpoint-preserving identity mean"
  - "why don't identical truth-table shapes always share NodeIds"
date: 2026-06-05
status: current
tags: [identity, factorization, egraph, roadmap]
evidence: src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

ANVIL's bounded semantic gate merge keys a cone by both its proven
function and its canonical leaf endpoints. Equal local truth-table shape
is not enough.

`ENDPOINT-IDENTITY-BOUNDARY.1` adds a regression with two same-shaped
cones: `a & (b | !b)` and `c & (d | !d)`. They simplify to the first
endpoint in each pair, but their endpoint sets differ, so
`merge_equivalent_gates` removes zero gates. This protects the doctrine
that `NodeId` identity means equality over the same canonical endpoints.
