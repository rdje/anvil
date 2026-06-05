---
id: hierarchy-dedup-prune
title: Hierarchy module dedup prunes definitions made unreachable by a merge
answers:
  - "does hierarchy module dedup remove unreachable modules"
  - "does hierarchy_module_dedup change under-instantiation"
  - "when are unused module definitions pruned"
  - "what happens after module dedup rewrites instances"
date: 2026-06-05
status: current
tags: [identity, hierarchy, dedup, roadmap]
evidence: src/ir/dedup.rs; book/src/knobs.md; book/src/hierarchy.md; DEVELOPMENT_NOTES.md
---

When `Config::hierarchy_module_dedup` performs at least one structural
module merge, ANVIL then prunes module definitions that were reachable
before dedup but are no longer reachable from `Design::top` after
instance references have been rewritten to canonical module names.

If no structural merge occurs, the pass preserves existing
under-instantiated library definitions. Pre-existing unreferenced
modules are not removed by the reachability cleanup; they can still be
collapsed if they share duplicate structural signatures. Dedup-off
behavior is unchanged.
