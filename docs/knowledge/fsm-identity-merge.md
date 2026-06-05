---
id: fsm-identity-merge
title: Deterministic generated FSM blocks can merge under node-id identity
answers:
  - "can ANVIL merge duplicate FSM blocks"
  - "why can FSMs merge but memories stay opaque"
  - "what does fsms_merged measure"
  - "does full factorization include FSM state"
date: 2026-06-05
status: current
tags: [identity, factorization, fsm, metrics]
evidence: src/ir/compact.rs; src/gen/module.rs; src/metrics.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

Under `identity_mode = node-id`, ANVIL merges duplicate generated FSM
blocks when their selector proof, selector width, encoding, state count,
transition table, Moore-output table, and output width match. This is
safe because generated FSMs reset to state 0 and are fully table-defined.
The metric is `fsms_merged`.

Generated memories deliberately remain opaque because their array
contents are not reset-defined by the current template.
