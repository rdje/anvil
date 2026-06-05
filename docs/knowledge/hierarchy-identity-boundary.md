---
id: hierarchy-identity-boundary
title: Hierarchy module dedup is structural, not semantic
answers:
  - "does hierarchy module dedup prove semantic equivalence"
  - "can hierarchy_module_dedup merge structurally different modules"
  - "why do semantically equal modules stay separate"
  - "what is the module dedup proof boundary"
date: 2026-06-05
status: current
tags: [identity, hierarchy, dedup, roadmap]
evidence: src/ir/dedup.rs; book/src/hierarchy.md; book/src/factorization.md; DEVELOPMENT_NOTES.md
---

`hierarchy_module_dedup` is a structural module-template pass. It groups
module definitions by `canonical_module_signature`, rewrites instances to
the canonical survivor when the signatures match, and performs the
conservative reachability cleanup described by `hierarchy-dedup-prune`.

It does not prove whole-module semantic equivalence. A regression covers
two one-bit modules that compute the same function (`input` and
`Not(Not(input))`) but have different IR structure: their signatures
differ, `dedup_modules` removes zero modules, and the top-level
instances keep their original module names. Deeper hierarchy equivalence
remains future work requiring a real module-level proof.
