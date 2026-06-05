---
id: reset-defined-self-hold-flop-identity
title: Exact reset-defined self-hold flops can merge
answers:
  - "can self-holding flops merge"
  - "what sequential coinductive flop class does ANVIL support"
  - "does ANVIL merge resetless self-hold flops"
  - "why does exact D equals own Q prove flop equality"
date: 2026-06-05
status: current
tags: [identity, sequential, factorization, coinduction]
evidence: src/ir/compact.rs; book/src/factorization.md; book/src/sequential.md; DEVELOPMENT_NOTES.md
---

ANVIL merges exact reset-defined self-hold flops only when they have the
same width, reset kind, reset value, and `Module::flop_domain`, and each
D input is exactly its own Q. Reset establishes equality; `D == Q`
preserves it on every clock. Reset-less self-hold, reset/domain/width
mismatches, mutually-recursive registers, retimed state, and non-exact
feedback forms remain no-merge boundaries.
