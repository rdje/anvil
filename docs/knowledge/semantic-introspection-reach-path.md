---
id: semantic-introspection-reach-path
title: ANVIL's analyze tool answers reach_path — a node's longest combinational fan-OUT path to a boundary sink, the forward complement to longest_path / the path-witness for node_reach (schema 1.27)
answers:
  - "what is the reach_path query"
  - "how do I see the longest forward path a node drives over MCP"
  - "what is the longest combinational fan-out path of an IR node in ANVIL"
  - "show me the deepest gate chain a node drives"
  - "what is a ReachPath"
  - "the forward complement to longest_path"
  - "the path-witness for node_reach"
  - "the fourteenth analyze derived query kind"
  - "which introspection schema version adds reach_path"
  - "how is a reach_path sink addressed"
  - "does reach_path keep the other analyze queries byte-identical"
  - "is reach_path consistent with node_reach"
  - "why does reach_path stop at the register boundary"
  - "is reach_path a timing critical path"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, reach-path, fan-out, longest-path, node-reach, witness, derived-relation, schema, structure-first, node-graph, register-boundary]
evidence: src/introspect/analyze.rs (QUERY_REACH_PATH, ReachPath, DerivedAnalysis.reach_path, module_reach_path/design_reach_path, reach_path_with, build_reach_path, gate_reach_height, drives_sink/pick_sink, reach_path_analysis, the consistency proof reach_path_realizes_node_reach_with_a_forward_chain + the register-boundary proof reach_path_records_the_flop_d_sink_and_stops_at_the_register_boundary + the longest-chain proof reach_path_picks_the_longest_chain_and_depth_equals_path_len + the tie-break proof reach_path_tie_break_is_smallest_reader_id); src/mcp/mod.rs (run_analyze reach_path dispatch + analyze_schema enum + the 2 mcp proofs analyze_returns_reach_path_and_caches_it / analyze_reach_path_unknown_target_is_invalid_params); src/introspect/mod.rs (SCHEMA_VERSION = 1.27); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.26 -> 1.27 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.15` — the `reach_path` derived query

`reach_path` is the **fourteenth** derived-relation query of the MCP `analyze` tool
(introspection schema **`1.27`**), beside `output_support`
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
([[semantic-introspection-instance-input-bindings]]), `longest_path`
([[semantic-introspection-longest-path]]), and `node_reach`
([[semantic-introspection-node-reach]]). It is the **tenth query beyond decision `0011`'s
four named kinds**, added under the lane's open-ended-breadth clause. It answers *what is the
longest thing this node drives, gate by gate?* — a relation over the IR by pure projection,
never behaviour (the `0004` no-shadow-simulator / structure-first ceiling).

It is the **forward-transitive witness**, completing the {set, witness} × {fan-in, fan-out}
square: `output_support` is the backward set, `longest_path` the backward witness (the chain
realizing `cone_depth`), `node_reach` the forward set, and `reach_path` the **forward
witness**. Where `node_reach` reports *which* boundary sinks a node reaches, `reach_path`
reports one representative **longest gate-chain to one of them** — so it is the **forward
complement to `longest_path`** and the **path-witness for `node_reach`**.

- **Query / tool:** `analyze {query: "reach_path", target?}`. `target` is `"node:<id>"` — the
  **same namespace as `node_drivers` / `node_readers` / `node_reach`** (single-endpoint ⇒ no
  MCP signature change); omit ⇒ one entry per IR node (ascending id). Cached + served as
  `anvil://artifact/<run_id>/analysis/reach_path`.
- **Result — a `ReachPath` per node:** `node` (id, `"node:<id>"`), `depth` (the gate count,
  `== path.len()`), `path` (the ordered interior-gate chain, each a `PathStep { node, op,
  width }` — **the same step type as `longest_path`**), and `sink` (`Option<String>` — the
  boundary sink in the `output_support`/`longest_path` target namespace: an output port name,
  or `"flop:<id>"`; `null` iff the node reaches no sink). So `reach_path(n).sink` chains
  straight into a follow-up `longest_path` / `output_support`.
- **Derivation — a greedy forward descent over a sink-aware height memo:** build the reader
  index by transposing operands (the **same** pass `node_reach_with` builds); a
  `gate_reach_height(g)` memo = the longest forward gate-chain from `g` to a sink-driving gate
  (`if cont>0 {1+cont} else if drives_sink(g) {1} else {0}`); then descend from the entry gate
  along the max-height reader, **ties broken by smallest reader id** ⇒ a unique byte-stable
  path; `pick_sink` = the smallest output-port name driven, else `"flop:<id>"` of the smallest
  flop whose `D` is the node. No IR field, no generator change.
- **Register boundary is automatic:** a flop is not a `Gate`, so it contributes no reader edge
  — the forward walk records a reached flop `D` sink and **never crosses into that flop's
  `Q`** downstream (the `node_reach` rule).
- **Structural gate-depth, NOT timing:** there is no delay model (the `longest_path` honesty
  boundary — named `reach_path`, not `critical_path`).
- **Provable consistency with `node_reach`:** `reach_path(n).sink.is_some() ==
  (node_reach(n).fanout_targets > 0)` (a node has a forward path to a sink iff it reaches ≥ 1
  sink), the chosen `sink ∈ node_reach(n)`'s sink set, and `depth == path.len()` — lib proofs
  assert all three. The forward analog of the `longest_path.depth == output_support.cone_depth`
  witness.
- **Module-vs-design — real in BOTH:** `reach_path` lives in one module's node graph, so
  `module_reach_path` and `design_reach_path` are both real (no fmt closure — `PathStep`
  carries ids + ops and sinks are plain names / flop ids); a missing design top ⇒ an empty
  analysis. Not Design-only like `instance_provenance`.
- **Schema shape (`1.26 → 1.27`, additive MINOR):** `DerivedAnalysis` gains a **fourteenth**
  parallel vec `reach_path: Vec<ReachPath>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the thirteen prior query
  documents stay **byte-identical** (the key is omitted) and only a `reach_path` document
  carries it (with `results: []`). No new leaf/step type — it reuses `PathStep`.
- **Errors / contract:** an unknown `query`, or a target that is not `"node:<id>"` / is
  out-of-range, ⇒ JSON-RPC `-32602`; a node that reaches no sink is a *known-but-empty* entry
  (empty `path`, `sink: null`), not an error. SCHEMA-DERIVED / default-off: a pure post-hoc
  projection — the default `anvil` build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-node-reach]] (the forward-transitive *set* this is the
path-witness for), [[semantic-introspection-longest-path]] (the backward-transitive witness
this is the forward complement of), [[semantic-introspection-node-readers]] (the 1-hop forward
view), [[semantic-introspection-analyze-tool]], [[semantic-introspection-derived-query-surface]],
and [[agent-introspection-schema]].
