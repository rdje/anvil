---
id: semantic-introspection-node-readers
title: ANVIL's analyze tool answers node_readers — per-node immediate (1-hop) reader adjacency, the exact transpose of node_drivers (schema 1.22)
answers:
  - "what immediately reads an ANVIL IR node"
  - "what is the node_readers query"
  - "what nodes consume an ANVIL IR node"
  - "how do I walk the ANVIL gate graph fan-out one hop at a time over MCP"
  - "what is a NodeReaders"
  - "how is node_readers different from node_drivers"
  - "is node_readers the transpose of node_drivers"
  - "which introspection schema version adds node_readers"
  - "how is an IR node's fan-out addressed in the analyze tool"
  - "does node_readers keep the other analyze queries byte-identical"
  - "the ninth analyze derived query kind"
  - "how do I walk the ANVIL construction DAG in either direction"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, node, gate, reader, fan-out, adjacency, transpose, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_NODE_READERS, NodeReaders, DerivedAnalysis.node_readers, module_node_readers/design_node_readers, node_readers_with, the transpose proof node_readers_is_the_exact_transpose_of_node_drivers); src/mcp/mod.rs (run_analyze node_readers dispatch + analyze_schema enum + the 2 mcp proofs); src/introspect/mod.rs (SCHEMA_VERSION = 1.22); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.21 -> 1.22 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.10` — the `node_readers` derived query

`node_readers` is the **ninth** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.22`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), `fsm_provenance`
([[semantic-introspection-fsm-provenance]]), and `node_drivers`
([[semantic-introspection-node-drivers]]). It is the **fifth query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth
clause. It answers *which nodes immediately read this node?* — a relation over
the IR by pure projection, never behaviour (the `0004` no-shadow-simulator /
structure-first ceiling).

It is the **exact transpose of `node_drivers`**. Where `node_drivers` reports a
node's fan-**in** (its direct operands), `node_readers` reports its fan-**out**
(the nodes that list it as a direct operand). It is the node-level analog of
`input_reach` ↔ `output_support`: one walks operand edges forward, the other
inverts the same edge set. With both queries an agent can walk the construction
DAG in **either direction** one hop at a time, with the provable duality
`B ∈ node_drivers(A).drivers` ⇔ `A ∈ node_readers(B).readers` — so the two
cannot drift.

- **Query / tool:** `analyze {query: "node_readers", target?}` (DUT lane only).
  `target` is `"node:<id>"` (the same address vocabulary `node_drivers`
  introduced); omit ⇒ the **whole node-level fan-out adjacency** (every node,
  ascending id). Cached + served as
  `anvil://artifact/<run_id>/analysis/node_readers`.
- **Result — a `NodeReaders` per node:** `node` (id = its index in `Module.nodes`),
  `kind` / `op` / `width` describing the **subject** node (mirroring `NodeDrivers`
  field-for-field; `op` omitted for a leaf), and `readers` — the nodes that read it,
  each a `NodeRef` (the same struct `node_drivers` uses), in **ascending node-id
  order** (sorted + deduplicated). Readers are always gates (only a gate has
  operands), so each reader's `NodeRef.kind` is `"gate"` and `name` is `"node:<id>"`.
- **Derivation — one pass transposing the operand relation:** `node_readers_with`
  iterates `Module.nodes` once, building a `BTreeMap<u32, BTreeSet<u32>>` reader
  index (for each `Gate` `r`, for each operand `o`, insert `r` into `readers[o]`),
  then resolves each reader via the shared `node_ref_of`. The `BTreeSet` keeps readers
  sorted + deduplicated + deterministic and collapses the `x & x` double-operand case
  to one reader. Pure: no IR field, no generator change; the design variant operates
  on the **top** module.
- **Boundary (deliberate, symmetric with `node_drivers`):** only node-to-node operand
  fan-out is reported. A node that drives a module output port or a flop `D` — but is
  no gate's operand — has an **empty `readers`** (those are not operand edges; use
  `input_reach` for the cone-level fan-out). This is what makes the transpose exact.
- **Schema shape (`1.21 → 1.22`, additive MINOR):** `DerivedAnalysis` gains a **ninth**
  parallel vec `node_readers: Vec<NodeReaders>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the eight prior query
  documents stay **byte-identical** (the key is omitted) and only a `node_readers`
  document carries it (with `results: []`). Each query populates exactly one vec;
  `query` is the discriminator.
- **Sorted, not operand order:** `readers` are sorted + deduplicated (a node's readers
  are a *set*), the opposite of `node_drivers`' operand-order `drivers` (an operand
  *list*).
- **Errors / contract:** an unknown `query`, or an unknown / out-of-range
  `"node:<id>"` (`id >= nodes.len()`), ⇒ JSON-RPC `-32602`; a node **no gate reads** is
  a *known-but-empty* entry (empty `readers`), not an error. SCHEMA-DERIVED /
  default-off: a pure post-hoc projection — the default `anvil` build and
  `--artifact dut` stay byte-identical.

See [[semantic-introspection-node-drivers]], [[semantic-introspection-analyze-tool]],
[[semantic-introspection-fsm-provenance]],
[[semantic-introspection-memory-provenance]],
[[semantic-introspection-flop-dependencies]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
