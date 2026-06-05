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
is not enough. Semantically-dead helper endpoints may now be minimized
away, but live canonical roots remain part of identity.

`ENDPOINT-IDENTITY-BOUNDARY.1` adds a regression with two same-shaped
cones: `a & (b | !b)` and `c & (d | !d)`. They simplify to the first
endpoint in each pair. After `COMBINATIONAL-SEMANTIC-IDENTITY.1`, the
roots may fold to `a` and `c` respectively, but they must not collapse
to the same canonical node. This protects the doctrine that `NodeId`
identity means equality over the same canonical endpoints.
