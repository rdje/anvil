---
id: semantic-introspection-longest-path
title: ANVIL's analyze tool answers longest_path — one representative longest combinational fan-in path per target, the witness for output_support's cone_depth (schema 1.25)
answers:
  - "what is the longest_path query"
  - "how do I see the deepest dependency chain of an output over MCP"
  - "what is the longest combinational fan-in path of a target in ANVIL"
  - "what realizes output_support's cone_depth"
  - "what is a LongestPath / PathStep"
  - "how do I get the ordered chain of gates between an output and a leaf"
  - "the twelfth analyze derived query kind"
  - "which introspection schema version adds longest_path"
  - "is longest_path a timing critical path"
  - "how is longest_path's path made deterministic / unique"
  - "does longest_path keep the other analyze queries byte-identical"
  - "how is a longest_path target addressed"
  - "the witness for cone_depth / the transitive complement to node_drivers"
date: 2026-06-24
status: current
tags: [introspection, mcp, analyze, longest-path, cone-depth, witness, gate-depth, derived-relation, schema, structure-first, node-graph]
evidence: src/introspect/analyze.rs (QUERY_LONGEST_PATH, LongestPath, PathStep, DerivedAnalysis.longest_path, module_longest_path/design_longest_path, longest_path_with, build_longest_path, node_depth, longest_path_analysis, the depth==cone_depth consistency proof longest_path_realizes_the_output_support_cone_depth + the deterministic tie-break proof longest_path_tie_break_picks_the_smallest_operand_id); src/mcp/mod.rs (run_analyze longest_path dispatch + analyze_schema enum + the 2 mcp proofs analyze_returns_longest_path_and_caches_it / analyze_longest_path_unknown_target_is_invalid_params); src/introspect/mod.rs (SCHEMA_VERSION = 1.25); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.24 -> 1.25 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.13` — the `longest_path` derived query

`longest_path` is the **twelfth** derived-relation query of the MCP `analyze` tool
(introspection schema **`1.25`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), `memory_provenance`
([[semantic-introspection-memory-provenance]]), `fsm_provenance`
([[semantic-introspection-fsm-provenance]]), `node_drivers`
([[semantic-introspection-node-drivers]]), `node_readers`
([[semantic-introspection-node-readers]]), `instance_provenance`
([[semantic-introspection-instance-provenance]]), and `instance_input_bindings`
([[semantic-introspection-instance-input-bindings]]). It is the **eighth query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth clause.
It answers *show me the deepest dependency chain of this target, gate by gate* — a
relation over the IR by pure projection, never behaviour (the `0004` no-shadow-simulator
/ structure-first ceiling).

It is the **witness for `output_support`'s scalar `cone_depth`** and the **transitive
complement to `node_drivers`**. Where `output_support` collapses the whole fan-in to its
boundary leaves (and reports `cone_depth` only as a number) and `node_drivers` exposes a
single node's *immediate (1-hop)* operands, `longest_path` returns the **ordered chain of
interior gates** — each with its op — realizing the deepest path. No prior query carries
an ordered transitive path.

- **Query / tool:** `analyze {query: "longest_path", target?}`. `target` is an output
  port name or `"flop:<id>"` — the **same namespace as `output_support`** (single-endpoint
  ⇒ no MCP signature change); omit ⇒ one path per output port (declaration order). Cached +
  served as `anvil://artifact/<run_id>/analysis/longest_path`.
- **Result — a `LongestPath` per resolved target:** `target` (the output / `"flop:<id>"`),
  `depth` (the gate-depth, `== path.len() == output_support(target).cone_depth`), `path`
  (the interior-gate chain ordered from the target's driver toward the leaf — one
  `PathStep { node, op, width }` per gate; every step is a `Gate`, so `op` is always
  present), and `leaf` (the terminal boundary leaf as a `NodeRef` — a primary input / flop
  `"flop:<id>"` / instance output / `"mem:<id>"` / `"fsm:<id>"` / constant; omitted only
  when the target is **undriven**).
- **Derivation — two pure passes over the existing node graph:** (1) a `node_depth` memo
  computing each node's max gate-depth — **the same recurrence `visit` returns as
  `cone_depth`**; (2) a greedy descent from the root that, at each gate, descends into the
  operand of **maximum depth, ties broken by smallest operand node id** (a total order ⇒ a
  unique, byte-stable path), stopping at the first non-gate (the leaf, via `node_ref_of`).
  No IR field, no generator change; `O(cone_nodes) + O(depth)`.
- **Provable consistency:** `longest_path(t).depth == output_support(t).cone_depth` by
  construction (the path length *is* the cone depth) — the two cannot drift; a lib proof
  asserts it on a hand-built module. The depth analog of the `node_drivers ↔ node_readers`
  / `output_support ↔ input_reach` duality proofs.
- **Honesty boundary:** it is **structural gate-depth, NOT a timing critical path** —
  ANVIL has no delay model, so "longest" means the maximum count of `Gate` nodes on any
  fan-in chain (no per-gate delay / wire-load / tech-mapping notion). Named `longest_path`,
  not `critical_path`, to keep that honest.
- **Module-vs-design — real in BOTH:** `longest_path` lives in one module's node graph (the
  `output_support` pattern), so `module_longest_path` and `design_longest_path` are both
  real; only the instance-leaf naming fmt differs (`format_instance_leaf_module` vs
  `_design`). Not Design-only like `instance_provenance`.
- **Schema shape (`1.24 → 1.25`, additive MINOR):** `DerivedAnalysis` gains a **twelfth**
  parallel vec `longest_path: Vec<LongestPath>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the eleven prior query
  documents stay **byte-identical** (the key is omitted) and only a `longest_path` document
  carries it (with `results: []`). `LongestPath.leaf` **reuses `NodeRef`** for the terminal
  leaf (the `node_drivers`/`node_readers` operand-handle precedent).
- **Errors / contract:** an unknown `query`, or a target that is not an output /
  `"flop:<id>"`, ⇒ JSON-RPC `-32602`; a resolvable-but-undriven target is a
  *known-but-empty* entry (empty `path`, no `leaf`), not an error. SCHEMA-DERIVED /
  default-off: a pure post-hoc projection — the default `anvil` build and `--artifact dut`
  stay byte-identical.

See [[semantic-introspection-analyze-tool]] (the `output_support` cone whose scalar
`cone_depth` this witnesses), [[semantic-introspection-reach-path]] (the **forward complement**
— the longest fan-OUT path, the witness this query mirrors),
[[semantic-introspection-node-drivers]] (the 1-hop view this
extends transitively), [[semantic-introspection-input-reach]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
