---
id: domain-aware-flop-identity
title: Flop identity includes the clock/reset domain
answers:
  - "can equivalent flops merge across clock domains"
  - "does flop merge key on Module::flop_domain"
  - "why are cross-domain duplicate flops kept distinct"
  - "what happens to flop_domains when flops are merged or compacted"
date: 2026-06-05
status: current
tags: [identity, sequential, multi-clock, factorization]
evidence: src/ir/compact.rs; book/src/factorization.md; book/src/sequential.md; DEVELOPMENT_NOTES.md
---

ANVIL's post-drain flop identity signature includes
`Module::flop_domain(flop.id)` in addition to width, reset kind, reset
value, and the bounded D-cone proof. Equal-looking flops in different
clock/reset domains do not merge. When flops are merged or compacted,
explicit `Module.flop_domains` entries are remapped to the new dense
`FlopId` space so later passes and library callers observe the same
domain ownership.
