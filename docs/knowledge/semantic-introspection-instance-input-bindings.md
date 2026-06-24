---
id: semantic-introspection-instance-input-bindings
title: ANVIL's analyze tool answers instance_input_bindings — the parent node driving each child input port, the parent-side dual of instance_provenance (schema 1.24)
answers:
  - "what drives a child instance's input ports from the parent"
  - "what is the instance_input_bindings query"
  - "how do I see what the parent feeds each child input over MCP"
  - "what is an InstanceInputBindings"
  - "the parent-side dual of instance_provenance"
  - "which introspection schema version adds instance_input_bindings"
  - "the eleventh analyze derived query kind"
  - "how do I map a child input back to the parent node in ANVIL"
  - "how do I trace a signal across the module boundary with analyze"
  - "how is a child instance addressed in instance_input_bindings"
  - "does instance_input_bindings keep the other analyze queries byte-identical"
  - "why is the instance_input_bindings module variant non-degenerate"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, instance, hierarchy, child-module, input-bindings, parent-side-dual, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_INSTANCE_INPUT_BINDINGS, InstanceInputBindings, InstanceInputBinding, DerivedAnalysis.instance_input_bindings, module_instance_input_bindings/design_instance_input_bindings, instance_input_bindings_analysis, the cross-boundary binding proof instance_input_bindings_resolve_the_parent_driver_across_the_boundary + the non-degenerate module-variant proof); src/mcp/mod.rs (run_analyze instance_input_bindings dispatch + analyze_schema enum + the 2 mcp proofs); src/introspect/mod.rs (SCHEMA_VERSION = 1.24); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.23 -> 1.24 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.12` — the `instance_input_bindings` derived query

`instance_input_bindings` is the **eleventh** derived-relation query of the MCP
`analyze` tool (introspection schema **`1.24`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), `fsm_provenance`
([[semantic-introspection-fsm-provenance]]), `node_drivers`
([[semantic-introspection-node-drivers]]), `node_readers`
([[semantic-introspection-node-readers]]), and `instance_provenance`
([[semantic-introspection-instance-provenance]]). It is the **seventh query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth
clause. It answers *what, in the parent, drives each of this child instance's
inputs?* — a relation over the IR by pure projection, never behaviour (the `0004`
no-shadow-simulator / structure-first ceiling).

It is the **parent-side dual of `instance_provenance`** — the exact future
extension `instance_provenance`'s design (`.11a`) named and deferred. Where
`instance_provenance` **descends into the child** to report the support cone of each
child **output** (in the child's terms), `instance_input_bindings` **ascends to the
parent** to report what drives each child **input** (in the parent's terms), read
straight from the instance's binding table `Instance.inputs: Vec<(PortId, NodeId)>`.
The two queries **bracket the instance boundary on both sides** — chaining
`instance_input_bindings → instance_provenance` traces a parent signal across the
module boundary through a child output and back, closing the loop the opaque
`Node::InstanceOutput` leaf hides.

- **Query / tool:** `analyze {query: "instance_input_bindings", target?}` on a
  **hierarchy** (design) config. `target` is a child instance **name**; omit ⇒ every
  child instance in the top (ascending name). Cached + served as
  `anvil://artifact/<run_id>/analysis/instance_input_bindings`.
- **Result — an `InstanceInputBindings` per child instance:** `instance` (its name,
  the entity), `module` (the child module it instantiates), `role`
  (`"planned_child"` | `"parent_cone"`), and `input_bindings` — one
  `InstanceInputBinding { port, driver }` per **bound** child input port (ascending
  child PortId). `port` is the child input **port name** in a design (or the
  `"port<id>"` fallback in a bare module); `driver` is the parent node as a `NodeRef`
  (`node` / `kind` / resolved `name` — a parent input name, a parent `"flop:<id>"`, a
  sibling `"<sibling>.<port>"`, or a parent `"node:<id>"`).
- **Derivation — a pure read of `Instance.inputs`:** `design_instance_input_bindings`
  walks each top instance (sorted by name); for each `(child_port_id,
  parent_node_id)` binding (sorted by child PortId) it resolves `driver =
  node_ref_of(top, parent_node_id, top_fmt)` **in the parent's graph** (a child input
  bound to a sibling instance output resolves to `"<sibling>.<port>"`) and names the
  child input port via the child def (fallback `"port<id>"`). No graph walk, no IR
  field, no generator change.
- **Module variant — NON-degenerate (the contrast with `instance_provenance`):**
  `module_instance_input_bindings` carries **real bindings** — the parent driver lives
  in the bare module's own graph, so it needs no child definitions. Only the child
  input port *name* degrades to `"port<id>"`. This is the load-bearing asymmetry:
  `instance_provenance`'s value (the child output cone) lives *inside* the child ⇒ its
  module variant is degenerate-empty; `instance_input_bindings`'s value (the parent
  driver) lives *in the parent* ⇒ its module variant is full.
- **Schema shape (`1.23 → 1.24`, additive MINOR):** `DerivedAnalysis` gains an
  **eleventh** parallel vec `instance_input_bindings: Vec<InstanceInputBindings>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the ten prior query
  documents stay **byte-identical** (the key is omitted) and only an
  `instance_input_bindings` document carries it (with `results: []`). Each query
  populates exactly one vec; `query` is the discriminator. `InstanceInputBinding`
  **reuses `NodeRef`** for the parent driver (the `node_drivers`/`node_readers`
  operand-handle precedent).
- **Scope boundary (the `node_drivers` one-hop precedent):** `driver` is the
  **immediate (1-hop)** parent node — chain `output_support` / `node_drivers` on
  `driver.node` for the transitive parent cone. The query reports **every** entry of
  the instance's `inputs` table faithfully — for a clocked child the `clk`/`rst_n`
  bindings (driven by the parent's `clk`/`rst_n`) are reported alongside the data
  inputs; the query does not filter control ports.
- **Errors / contract:** an unknown `query`, or a child instance name not in the top,
  ⇒ JSON-RPC `-32602`; an instance with no data bindings is a *known-but-empty* entry
  (empty `input_bindings`), not an error. SCHEMA-DERIVED / default-off: a pure
  post-hoc projection — the default `anvil` build and `--artifact dut` stay
  byte-identical.

See [[semantic-introspection-instance-provenance]] (its descent-side dual),
[[semantic-introspection-node-drivers]],
[[semantic-introspection-node-readers]],
[[semantic-introspection-analyze-tool]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
