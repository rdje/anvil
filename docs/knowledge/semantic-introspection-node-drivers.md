---
id: semantic-introspection-node-drivers
title: ANVIL's analyze tool answers node_drivers — per-node immediate (1-hop) driver adjacency + each node's GateOp (schema 1.21)
answers:
  - "what immediately drives an ANVIL IR node"
  - "what is the node_drivers query"
  - "how do I walk the ANVIL gate graph one hop at a time over MCP"
  - "does ANVIL expose a node's GateOp over introspection"
  - "what is a NodeDrivers / NodeRef"
  - "how is an IR node addressed in the analyze tool"
  - "which introspection schema version adds node_drivers"
  - "how is node_drivers different from output_support"
  - "does node_drivers keep the operand order"
  - "does node_drivers keep the other analyze queries byte-identical"
  - "the eighth analyze derived query kind"
  - "what is the atomic node-level primitive under the support cone"
date: 2026-06-23
status: current
tags: [introspection, mcp, analyze, node, gate, driver, adjacency, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_NODE_DRIVERS, NodeDrivers, NodeRef, DerivedAnalysis.node_drivers, module_node_drivers/design_node_drivers, node_drivers_with, node_kind_str/gate_op_str/node_ref_of); src/mcp/mod.rs (run_analyze node_drivers dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.21); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.20 -> 1.21 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.9` — the `node_drivers` derived query

`node_drivers` is the **eighth** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.21`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), and `fsm_provenance`
([[semantic-introspection-fsm-provenance]]). It is the **fourth query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth
clause. It answers *what immediately drives this node, and what op is it?* — a
relation over the IR by pure projection, never behaviour (the `0004`
no-shadow-simulator / structure-first ceiling).

It is the **atomic node-level primitive complementing the transitive
`output_support` cone**. Where a `SupportCone` collapses a whole fan-in to its
boundary leaves (primary inputs / flop `Q`s / instance outputs) and names neither
the interior `Node::Gate`s it crossed nor their ops, `node_drivers` exposes the
node-level fan-in graph **one hop at a time** *and* surfaces each node's `GateOp` —
genuinely new information no prior query carries. An agent can re-issue it for each
operand that is itself a gate, walking the DAG hop by hop and reconstructing any
cone itself.

- **Query / tool:** `analyze {query: "node_drivers", target?}` (DUT lane only).
  `target` is `"node:<id>"` (a new address vocabulary, parallel to `"flop:<id>"` /
  `"mem:<id>"` / `"fsm:<id>"`); omit ⇒ the **whole node-level adjacency** (every
  node, ascending id). Cached + served as
  `anvil://artifact/<run_id>/analysis/node_drivers`.
- **Result — a `NodeDrivers` per node:** `node` (id = its index in `Module.nodes`),
  `kind` (`"primary_input"` / `"constant"` / `"flop_q"` / `"mem_read"` / `"fsm_out"`
  / `"instance_output"` / `"gate"`), `op` (for a `Gate`, its `GateOp` as a stable
  base-op string e.g. `"and"` / `"mux"` / `"slice"`; omitted for a leaf), `width`,
  and `drivers` — the list of its direct operands **in operand order** (empty for a
  leaf). A `NodeRef` operand carries `node` (id), `kind`, and a resolved `name` (an
  input port name / `"flop:<id>"` / `"mem:<id>"` / `"fsm:<id>"` /
  `"<instance>.<port>"`, or `"node:<id>"` for an interior gate / constant).
- **Derivation — a single one-hop pass:** `node_drivers_with` iterates `Module.nodes`
  once, reading exactly **one level** of operands per node — no transitive walk, no
  DFS, no memoization (even more local than `output_support`; the `build_cone` walker
  is untouched). The design variant operates on the **top** module and resolves an
  instance-output operand to `"<instance>.<child-output-port>"`. Pure: no IR field,
  no generator change.
- **Schema shape (`1.20 → 1.21`, additive MINOR):** `DerivedAnalysis` gains an
  **eighth** parallel vec `node_drivers: Vec<NodeDrivers>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the seven prior query
  documents stay **byte-identical** (the key is omitted) and only a `node_drivers`
  document carries it (with `results: []`). Each query populates exactly one vec;
  `query` is the discriminator.
- **Operand order, not sorted:** `drivers` preserve the IR operand order (`a - b` ≠
  `b - a`; a `Mux`'s `[sel, a, b]`) — the one deliberate departure from the cone
  queries' sorted support lists; it stays deterministic because the operand `Vec` is.
- **Errors / contract:** an unknown `query`, or an unknown / out-of-range
  `"node:<id>"` (`id >= nodes.len()`), ⇒ JSON-RPC `-32602`; a **leaf** node is a
  *known-but-empty* entry (empty `drivers`, no `op`), not an error. SCHEMA-DERIVED /
  default-off: a pure post-hoc projection — the default `anvil` build and
  `--artifact dut` stay byte-identical.
- **Dual:** the immediate fan-**out** transpose `node_readers`
  ([[semantic-introspection-node-readers]], the ninth query, schema `1.22`) is now
  delivered — together they let an agent walk the construction DAG in either direction
  one hop at a time, with the duality `B ∈ node_drivers(A) ⇔ A ∈ node_readers(B)`.

See [[semantic-introspection-node-readers]], [[semantic-introspection-analyze-tool]], [[semantic-introspection-fsm-provenance]],
[[semantic-introspection-memory-provenance]],
[[semantic-introspection-flop-dependencies]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
