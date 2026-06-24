---
id: semantic-introspection-instance-provenance
title: ANVIL's analyze tool answers instance_provenance — per-child-instance descent into the child module's graph, the third opaque-leaf boundary-opener (schema 1.23, design-only)
answers:
  - "what drives a child instance's outputs inside the child module"
  - "what is the instance_provenance query"
  - "how do I open the opaque InstanceOutput leaf in ANVIL introspection"
  - "what is an InstanceProvenance"
  - "how do I see inside a child instance over MCP"
  - "which introspection schema version adds instance_provenance"
  - "why is instance_provenance design-only"
  - "which analyze query crosses the module boundary"
  - "the tenth analyze derived query kind"
  - "how is a child instance addressed in the analyze tool"
  - "does instance_provenance keep the other analyze queries byte-identical"
  - "the third opaque-leaf boundary-opener after memory_provenance and fsm_provenance"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, instance, hierarchy, child-module, provenance, support-cone, boundary-opener, derived-relation, schema, design-only, structure-first]
evidence: src/introspect/analyze.rs (QUERY_INSTANCE_PROVENANCE, InstanceProvenance, DerivedAnalysis.instance_provenance, module_instance_provenance/design_instance_provenance, instance_provenance_analysis, instance_role_str, the cross-boundary descent proof instance_provenance_descends_into_the_child_module); src/mcp/mod.rs (run_analyze instance_provenance dispatch + analyze_schema enum + the 2 mcp proofs); src/introspect/mod.rs (SCHEMA_VERSION = 1.23); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.22 -> 1.23 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.11` — the `instance_provenance` derived query

`instance_provenance` is the **tenth** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.23`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), `fsm_provenance`
([[semantic-introspection-fsm-provenance]]), `node_drivers`
([[semantic-introspection-node-drivers]]), and `node_readers`
([[semantic-introspection-node-readers]]). It is the **sixth query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth
clause. It answers *what, inside this child instance, drives each of its
outputs?* — a relation over the IR by pure projection, never behaviour (the
`0004` no-shadow-simulator / structure-first ceiling).

It is the **third opaque-leaf boundary-opener**, completing the trilogy with
`memory_provenance` (opens `Node::MemRead`) and `fsm_provenance` (opens
`Node::FsmOut`): those are the three opaque leaves a support cone terminates at,
and `instance_provenance` opens the third (`Node::InstanceOutput`). Where every
other query records a child-instance output as an opaque `"<instance>.<port>"`
leaf and stops, `instance_provenance` descends into the child module and reports
the support cone of each child output port **inside the child module's own node
graph**. It is the **first and only query that crosses the module boundary**,
which is exactly why it is **design-only**: a bare `Module` carries the
`InstanceOutput` leaves but not the child module definitions needed to descend.

- **Query / tool:** `analyze {query: "instance_provenance", target?}` on a
  **hierarchy** (design) config. `target` is a child instance **name**; omit ⇒
  every child instance in the top (ascending name). Cached + served as
  `anvil://artifact/<run_id>/analysis/instance_provenance`.
- **Result — an `InstanceProvenance` per child instance:** `instance` (its name,
  the entity), `module` (the child module it instantiates), `role`
  (`"planned_child"` | `"parent_cone"`), and `output_support` — one `SupportCone`
  per child **output port** (in the child's declaration order), each with
  `target = "<instance>.<child-output-port-name>"` (the exact name the other
  queries give that leaf) and **child-internal** support leaves (the child's input
  ports, flops, and grand-child instance outputs).
- **Derivation — reuse `build_cone` with the child as its walked module:**
  `design_instance_provenance` indexes `design.modules` by name; for each top
  instance (sorted by name) it looks up the child `Module` and `build_cone`s per
  child output port **inside the child** with the child's own
  `format_instance_leaf_design` fmt. `build_cone` is reused **unchanged** — only
  the `Module` it walks changes (the child, not the analyzed top — the first query
  whose `build_cone` module is other than the top). Pure: no IR field, no generator
  change.
- **Module variant — the degenerate no-child-defs case:**
  `module_instance_provenance` lists a bare module's instances (name/module/role)
  with **empty** `output_support` (it has no child bodies to descend into) — the
  `format_instance_leaf_module` / `module_module_reachability` degenerate
  precedent. A leaf DUT module has no instances ⇒ an empty analysis.
- **Schema shape (`1.22 → 1.23`, additive MINOR):** `DerivedAnalysis` gains a
  **tenth** parallel vec `instance_provenance: Vec<InstanceProvenance>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the nine prior
  query documents stay **byte-identical** (the key is omitted) and only an
  `instance_provenance` document carries it (with `results: []`). Each query
  populates exactly one vec; `query` is the discriminator. `InstanceProvenance`
  **reuses `SupportCone`** (one cone per child output port — the
  `memory_provenance`/`fsm_provenance` cone-reuse precedent).
- **Scope boundary (future, nothing retired):** input-binding chaining
  (`Instance.inputs`) is the **parent-side dual** query
  [[semantic-introspection-instance-input-bindings]] (delivered as `.12`); this query
  descends exactly one level (a grand-child instance output is a cone leaf, not
  recursed — chain by re-querying with the child as a new top).
- **Errors / contract:** an unknown `query`, or a child instance name not in the
  top, ⇒ JSON-RPC `-32602`; a child with no output ports is a *known-but-empty*
  entry (empty `output_support`), not an error. SCHEMA-DERIVED / default-off: a pure
  post-hoc projection — the default `anvil` build and `--artifact dut` stay
  byte-identical.

See [[semantic-introspection-memory-provenance]],
[[semantic-introspection-fsm-provenance]],
[[semantic-introspection-node-drivers]],
[[semantic-introspection-node-readers]],
[[semantic-introspection-analyze-tool]],
[[semantic-introspection-flop-dependencies]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
