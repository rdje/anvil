---
id: semantic-introspection-flop-dependencies
title: ANVIL's analyze tool answers flop_dependencies — the register-to-register dependency graph (per flop its predecessors/successors + self-feedback flag, schema 1.18)
answers:
  - "how do an ANVIL module's registers feed each other"
  - "what is the flop_dependencies query"
  - "what is the register-to-register dependency graph in ANVIL"
  - "which flops does a flop depend on / drive"
  - "how do I find self-feedback registers (counters/accumulators) over MCP"
  - "what is a FlopDependencies"
  - "does ANVIL expose the flop / register dependency graph"
  - "which introspection schema version adds flop_dependencies"
  - "how does ANVIL compute flop dependencies"
  - "does flop_dependencies keep the other analyze queries byte-identical"
  - "how is a flop addressed in the analyze tool"
  - "what is depends_on_flops / driven_flops / self_dependent"
  - "the fifth analyze derived query kind"
date: 2026-06-23
status: current
tags: [introspection, mcp, analyze, flop, register, dependency-graph, sequential, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_FLOP_DEPENDENCIES, FlopDependencies, DerivedAnalysis.flop_dependencies, module_flop_dependencies/design_flop_dependencies, flop_dependencies_with); src/mcp/mod.rs (run_analyze flop_dependencies dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.18); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.17 -> 1.18 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.6` — the `flop_dependencies` derived query

`flop_dependencies` is the **fifth** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.18`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), and `module_reachability`
([[semantic-introspection-module-reachability]]). It is the **first query beyond
decision `0011`'s four named kinds**, added under the lane's open-ended-breadth
clause. It answers *how do this module's registers feed each other?* — the
register-to-register dependency graph, a relation over the IR by pure projection,
never behaviour (the `0004` no-shadow-simulator / structure-first ceiling).

It is the **register-level analog of `module_reachability`** (a graph over a node
class), but reuses the existing **gate-graph** support/reach machinery rather than
the module table.

- **Query / tool:** `analyze {query: "flop_dependencies", target?}` (DUT lane only).
  `target` is `"flop:<id>"` (consistent with the other flop queries); omit ⇒ every
  flop. Cached + served as `anvil://artifact/<run_id>/analysis/flop_dependencies`.
- **Result — a `FlopDependencies` per flop:** `flop` (id), `depends_on_flops` (direct
  register **predecessors** — flop ids whose `Q` feeds this flop's `D` cone, i.e. its
  D-cone `support_flops`), `driven_flops` (direct register **successors** — the
  transpose across the module), and `self_dependent` (whether `flop ∈
  depends_on_flops`: a self-feedback register — a counter/accumulator). Both edge
  vecs sorted/deduped; entries ascending by flop id.
- **Derivation — reuse the cone machinery:** each flop's D-cone `support_flops` are
  its predecessors; the transpose (`B ∈ depends_on(A)` ⇔ `A ∈ driven(B)`) gives
  successors — exactly the `input_reach` inversion restricted to flops, so the two
  directions cannot drift. A direct register-graph edge `A → B` (`B ∈
  depends_on_flops(A)`) means `B`'s `Q` feeds `A`'s `D` through pure combinational
  logic — one register-stage hop (the cone is transitive combinational and stops at
  every register boundary). The design variant operates on the **top** module (the
  `flop_reset_provenance` convention). Pure: no IR field, no generator change.
- **Schema shape (`1.17 → 1.18`, additive MINOR):** `DerivedAnalysis` gains a
  **fifth** parallel vec `flop_dependencies: Vec<FlopDependencies>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the four prior query
  documents stay **byte-identical** (the key is omitted) and only a
  `flop_dependencies` document carries it (with `results: []`). Each query populates
  exactly one vec; `query` is the discriminator.
- **Completeness vs redundancy:** each edge is individually derivable from
  `output_support` / `input_reach` on a `"flop:<id>"` target, but no single one of
  those returns the whole register graph; per the agent-audience completeness rule
  ([[api-first-everything-mcp-accessible]]) `flop_dependencies` returns the complete
  graph **view** (+ `self_dependent`) in one query — a relation, not new computed
  truth.
- **Errors / contract:** an unknown `query`, or an unknown / out-of-range
  `"flop:<id>"`, ⇒ JSON-RPC `-32602`; a flopless module ⇒ an empty result.
  SCHEMA-DERIVED / default-off: a pure post-hoc projection — the default `anvil`
  build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]], [[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
