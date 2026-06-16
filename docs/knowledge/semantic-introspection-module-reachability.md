---
id: semantic-introspection-module-reachability
title: ANVIL's analyze tool answers module_reachability — which modules in a design are reachable from the top via the instance graph (schema 1.7)
answers:
  - "which modules in an ANVIL design are reachable from the top"
  - "what is the module_reachability query"
  - "how do I ask which modules are dead or unreachable over MCP"
  - "what is a ModuleReachability"
  - "does ANVIL expose the design module / instance graph"
  - "how deep is a module in the hierarchy / what is its depth from the top"
  - "what modules does a module instantiate"
  - "which introspection schema version adds module_reachability"
  - "how does ANVIL compute module reachability"
  - "does module_reachability keep the other analyze queries byte-identical"
  - "how is a module addressed in the analyze tool"
  - "what does analyze module_reachability return for a combinational DUT"
date: 2026-06-16
status: current
tags: [introspection, mcp, analyze, hierarchy, module, reachability, instance-graph, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_MODULE_REACHABILITY, ModuleReachability, DerivedAnalysis.module_reachability, design_module_reachability/module_module_reachability); src/mcp/mod.rs (run_analyze module_reachability dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.7); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.6 -> 1.7 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.5` — the `module_reachability` derived query

`module_reachability` is the **fourth** (and last named) derived-relation query of
the MCP `analyze` tool (introspection schema **`1.7`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), and `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]). It answers *which modules in a
design are reachable from `design.top` via the instance graph, and how does each
one sit in it?* — a relation over the construction graph by pure projection, never
behaviour (the `0004` no-shadow-simulator / structure-first ceiling). It is the
first query whose home is the **whole design** rather than one module's node graph.

- **Query / tool:** `analyze {query: "module_reachability", target?}` (DUT lane
  only). Unlike the prior three, `target` is a **module name** (not a port name /
  `"flop:<id>"`); omit ⇒ every module. Cached + served as
  `anvil://artifact/<run_id>/analysis/module_reachability`.
- **Result — a `ModuleReachability` per module:** `module` (name), `reachable`
  (from `design.top`), `depth` (the minimum instance-graph distance from the top —
  `0` for the top; `Option<usize>`, present iff `reachable`), `instantiates` (the
  distinct child module names it directly instantiates, sorted/deduped — its local
  out-edges; both `PlannedChild` and `ParentCone` helper instances count), and
  `instance_count` (its direct-instance count, `>= instantiates.len()`). Entries
  sorted by module name.
- **Derivation — a min-depth BFS** from `design.top` over a name→`Module` index of
  the `Module.instances[].module` edges. A pure projection of `Design.modules` +
  the instance edges; no gate-graph walk. A combinational/sequential leaf DUT is a
  single module reported as a trivial root (`reachable`, `depth 0`); a malformed
  design whose top is absent reports every present module `reachable: false` (the
  honest whole-table enumeration).
- **Schema shape (`1.6 → 1.7`, additive MINOR):** `DerivedAnalysis` gains a
  **fourth** parallel vec `module_reachability: Vec<ModuleReachability>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so `output_support` /
  `input_reach` / `flop_reset_provenance` documents stay **byte-identical** (the key
  is omitted) and only a `module_reachability` document carries it (with
  `results: []`). Each query populates exactly one vec; `query` is the discriminator.
- **Errors / contract:** an unknown `query` or unknown module name ⇒ JSON-RPC
  `-32602`. SCHEMA-DERIVED / default-off: a pure post-hoc projection — no new
  computed truth, no IR field, no generator change; the default `anvil` build and
  `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]], [[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
