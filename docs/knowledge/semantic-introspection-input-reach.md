---
id: semantic-introspection-input-reach
title: ANVIL's analyze tool answers input_reach — the dual fan-out of the support cone (schema 1.5)
answers:
  - "what does input X reach in an ANVIL module"
  - "what is the input_reach query"
  - "how do I ask which outputs an input drives over MCP"
  - "what does a flop Q reach in an ANVIL design"
  - "what is the dual of the output support cone"
  - "what is a ReachResult"
  - "how does ANVIL compute input_reach"
  - "which introspection schema version adds input_reach"
  - "why is reach_results a separate vec from results"
  - "does the input_reach surface keep output_support byte-identical"
  - "how is a child-instance output addressed as a reach source"
  - "what does analyze input_reach return for an unknown source"
date: 2026-06-16
status: current
tags: [introspection, mcp, analyze, input-reach, fan-out, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_INPUT_REACH, ReachResult, DerivedAnalysis.reach_results, module_input_reach/design_input_reach); src/mcp/mod.rs (run_analyze input_reach dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.5); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.4 -> 1.5 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.3` — the `input_reach` derived query

`input_reach` is the **second** derived-relation query of the MCP `analyze` tool
(introspection schema **`1.5`**), the exact **dual** of the `output_support`
support cone ([[semantic-introspection-analyze-tool]]). Where `output_support`
answers *what does this output depend on?*, `input_reach` answers *what does this
source reach?* — still a relation by pure IR-graph traversal, never a behavioural
simulation (the `0004` no-shadow-simulator / structure-first boundary is the
permanent ceiling).

- **Query / tool:** `analyze {query: "input_reach", target?}` (DUT lane only).
  `target` is a **source** — an input **port name**, a flop `Q` as `"flop:<id>"`,
  or a child-instance output `"<instance>.<port>"`; omit ⇒ every source. Cached +
  served as `anvil://artifact/<run_id>/analysis/input_reach`.
- **Result — a `ReachResult` per source:** `target` (the source), `reaches_outputs`
  (output port names it reaches), `reaches_flops` (flop ids whose `D`-cone it
  reaches), `fanout_targets` (their total).
- **Derivation — inversion, not a second walker:** `module_input_reach` /
  `design_input_reach` build every target's `SupportCone` with the existing
  support machinery (outputs + each `"flop:<id>"` D-cone) and bucket each target
  under the sources its cone lists. So a source `X` reaches a target `T` **iff**
  `T`'s support cone contains `X` — `output_support` and `input_reach` cannot
  drift, and the flop/instance/mem-fsm boundary rules live in exactly one place.
- **`"flop:<id>"` direction duality:** as an `input_reach` *source* it is the
  flop's **Q** (its fan-out); as an `output_support` *target* it is the flop's
  **D** cone (its fan-in). Same register boundary, opposite direction — the
  `query` kind chooses. Declared control ports (`clk`/`rst_n`) appear as sources
  with empty reach.
- **Schema shape (`1.4 → 1.5`, additive MINOR):** `DerivedAnalysis` gains a
  **second** parallel vec `reach_results: Vec<ReachResult>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so an
  `output_support` document is **byte-identical** to `1.4` (the key is omitted)
  and only an `input_reach` document carries it (with `results: []`). Each query
  populates exactly one vec; `query` is the discriminator. Rejected: a tagged
  enum (would break the existing `output_support` wire shape).
- **Errors / contract:** an unknown `query` or unresolvable `target` ⇒ JSON-RPC
  `-32602`. SCHEMA-DERIVED / default-off: a pure post-hoc projection of the IR the
  generator already built — no new computed truth, no IR field, no generator
  change; the default `anvil` build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
