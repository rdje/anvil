---
id: semantic-introspection-node-reach
title: ANVIL's analyze tool answers node_reach — a node's transitive combinational fan-OUT (the output ports + flop D cones it reaches), the transitive complement to node_readers (schema 1.26)
answers:
  - "what is the node_reach query"
  - "how do I see what a node ultimately drives over MCP"
  - "what is the transitive combinational fan-out of an IR node in ANVIL"
  - "which outputs and flops does a node reach"
  - "what is a NodeReach"
  - "the transitive complement to node_readers"
  - "the node-addressed generalization of input_reach"
  - "the thirteenth analyze derived query kind"
  - "which introspection schema version adds node_reach"
  - "how do I get the per-FSM or per-memory reach in ANVIL"
  - "does node_reach keep the other analyze queries byte-identical"
  - "how is a node_reach target addressed"
  - "why does node_reach stop at the register boundary"
  - "is node_reach consistent with input_reach"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, node-reach, fan-out, reach, input-reach, node-readers, derived-relation, schema, structure-first, node-graph, register-boundary]
evidence: src/introspect/analyze.rs (QUERY_NODE_REACH, NodeReach, DerivedAnalysis.node_reach, module_node_reach/design_node_reach, node_reach_with, node_reach_analysis, the consistency proof node_reach_matches_input_reach_for_each_input_node + the register-boundary proof node_reach_records_the_flop_d_sink_and_stops_at_the_register_boundary + the instance-input-only boundary proof node_reach_instance_input_only_node_reaches_nothing); src/mcp/mod.rs (run_analyze node_reach dispatch + analyze_schema enum + the 2 mcp proofs analyze_returns_node_reach_and_caches_it / analyze_node_reach_unknown_target_is_invalid_params); src/introspect/mod.rs (SCHEMA_VERSION = 1.26); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.25 -> 1.26 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.14` — the `node_reach` derived query

`node_reach` is the **thirteenth** derived-relation query of the MCP `analyze` tool
(introspection schema **`1.26`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), `fsm_provenance`
([[semantic-introspection-fsm-provenance]]), `node_drivers`
([[semantic-introspection-node-drivers]]), `node_readers`
([[semantic-introspection-node-readers]]), `instance_provenance`
([[semantic-introspection-instance-provenance]]), `instance_input_bindings`
([[semantic-introspection-instance-input-bindings]]), and `longest_path`
([[semantic-introspection-longest-path]]). It is the **ninth query beyond decision `0011`'s
four named kinds**, added under the lane's open-ended-breadth clause. It answers *what does
this node ultimately reach?* — a relation over the IR by pure projection, never behaviour
(the `0004` no-shadow-simulator / structure-first ceiling).

It is the **transitive complement to `node_readers`**, completing the node-addressed
driver/reader × 1-hop/transitive 2×2 matrix: `node_drivers` (backward 1-hop), `node_readers`
(forward 1-hop), `longest_path` (backward transitive — the ordered fan-in chain), and
`node_reach` (the forward-transitive corner). It is also the **node-addressed generalization
of `input_reach`** — `input_reach` accepts only cone-leaf *sources* (inputs / flop `Q`s /
instance outputs) and is computed by inverting cones; `node_reach` accepts *any* node via a
forward graph walk, so it **subsumes the per-FSM/per-memory reach** (address a `MemRead` /
`FsmOut` node by its `kind`; nothing retired).

- **Query / tool:** `analyze {query: "node_reach", target?}`. `target` is `"node:<id>"` — the
  **same namespace as `node_drivers` / `node_readers`** (single-endpoint ⇒ no MCP signature
  change); omit ⇒ one entry per IR node (ascending id). Cached + served as
  `anvil://artifact/<run_id>/analysis/node_reach`.
- **Result — a `NodeReach` per node:** `node` (id, `"node:<id>"`), `kind` / `op` / `width`
  (the node-family header, mirroring `NodeReaders`), `reaches_outputs` (the output port names
  whose driving node is in the node's forward closure, sorted), `reaches_flops` (the flop ids
  whose `D` node is in the closure, sorted), and `fanout_targets` (their total).
- **Derivation — a forward-closure walk over the reader index:** transpose the operand
  relation into a reader index (the **same** pass `node_readers_with` builds), walk the
  forward closure of the target node over that index, then classify the boundary sinks =
  output ports whose driver ∈ closure (`m.drives` / `driver_of_port`) + flops whose `D` ∈
  closure (`m.flops`). No IR field, no generator change.
- **Register boundary is automatic:** a flop is not a `Gate`, so it contributes no reader
  edge — the forward walk records a reached flop `D` sink and **never crosses into that
  flop's `Q`** downstream (the symmetric dual of `output_support`/`visit` stopping at a flop
  `Q`).
- **Sink boundary (symmetric with `input_reach`):** the sinks are output ports + flop `D`
  cones only — *not* child-instance inputs. A node that only feeds an instance input reaches
  nothing here; chain `instance_input_bindings` → `instance_provenance` to cross the module
  boundary.
- **Provable consistency:** `node_reach("node:<the PrimaryInput node of input i>").{reaches_outputs,
  reaches_flops} == input_reach(i).{reaches_outputs, reaches_flops}` by construction (the
  forward walk and the inversion-based reach agree on every cone-leaf source) — a lib proof
  asserts it. The forward-reach analog of the `longest_path.depth == cone_depth` /
  `node_drivers ↔ node_readers` duality proofs.
- **Module-vs-design — real in BOTH:** `node_reach` lives in one module's node graph, so
  `module_node_reach` and `design_node_reach` are both real (no fmt closure — sinks are plain
  port names + flop ids); a missing design top ⇒ an empty analysis. Not Design-only like
  `instance_provenance`.
- **Schema shape (`1.25 → 1.26`, additive MINOR):** `DerivedAnalysis` gains a **thirteenth**
  parallel vec `node_reach: Vec<NodeReach>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the twelve prior query
  documents stay **byte-identical** (the key is omitted) and only a `node_reach` document
  carries it (with `results: []`). No new leaf-handle type (the sinks are an output port name
  + a flop id, the `ReachResult` payload).
- **Errors / contract:** an unknown `query`, or a target that is not `"node:<id>"` / is
  out-of-range, ⇒ JSON-RPC `-32602`; a node that reaches no sink is a *known-but-empty* entry,
  not an error. SCHEMA-DERIVED / default-off: a pure post-hoc projection — the default `anvil`
  build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-node-readers]] (the 1-hop view this extends transitively),
[[semantic-introspection-input-reach]] (the cone-leaf-sourced reach this generalizes to any
node), [[semantic-introspection-longest-path]] (the backward-transitive sibling),
[[semantic-introspection-analyze-tool]], [[semantic-introspection-derived-query-surface]],
and [[agent-introspection-schema]].
