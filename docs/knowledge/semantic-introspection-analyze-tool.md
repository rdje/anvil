---
id: semantic-introspection-analyze-tool
title: ANVIL ships a pure MCP `analyze` tool that returns an output's combinational support cone (schema 1.3)
answers:
  - "how does an agent query a generated module's support cone over MCP"
  - "what does the anvil analyze MCP tool return"
  - "how do I ask what an ANVIL output depends on"
  - "what is the output_support query"
  - "what is a SupportCone"
  - "what is a DerivedAnalysisDocument"
  - "which introspection schema version adds the analyze surface"
  - "how does ANVIL address a flop D cone in analyze"
  - "what does analyze return for an unknown query or target"
  - "is the analyze MCP tool default-off and DUT byte-identical"
  - "where is the ANVIL support-cone analysis implemented"
  - "does ANVIL recurse through flops or child instances in a support cone"
  - "how does an agent ask what drives output Y over MCP"
date: 2026-06-16
status: current
tags: [introspection, mcp, analyze, support-cone, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (DerivedAnalysis/SupportCone, module_support_cones/design_support_cones); src/introspect/mod.rs (DerivedAnalysisDocument, derived_analysis_document, SCHEMA_VERSION = 1.3); src/mcp/mod.rs (run_analyze tool + analyze_schema + analysis resource); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7); book/src/agent-mcp.md; docs/decisions/0011-semantic-introspection-derived-query-surface.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.2b` — the `analyze` derived-relation tool

ANVIL exposes a first-class, pure, MCP-queryable derived-**relation** surface
(decision [`0011`](../decisions/0011-semantic-introspection-derived-query-surface.md),
introspection schema **`1.3`**). It answers *what does this output structurally
depend on?* — a relation, by pure IR-graph traversal, **never** a behavioural
simulation (the `0004` no-shadow-simulator / structure-first boundary is the
permanent ceiling).

- **Tool:** the pure MCP `analyze` tool (`src/mcp/mod.rs::run_analyze`) — DUT
  lane only (the non-DUT lanes carry no gate graph; a non-DUT `lane` is a clean
  tool error). Like `generate`/`introspect` it takes `(seed, config)`, plus a
  `query` kind and an optional `target`. Cached + served as the
  `anvil://artifact/<run_id>/analysis/<query>` resource.
- **First query — `output_support` (the default):** each target's transitive
  **combinational** fan-in support cone, a `SupportCone`:
  - `target` — an output **port name**, or a flop `D` addressed `"flop:<id>"`;
    omit ⇒ a cone for every output.
  - `support_inputs` (input port names) / `support_flops` (flop ids) /
    `support_instance_outputs` (`"<inst>.<port>"`) — the support **leaves**.
  - `cone_nodes` (distinct fan-in nodes) + `cone_depth` (max combinational gate
    depth).
- **Stopping rules (the cone is purely combinational):** a `FlopQ` is a
  **register boundary** (recorded in `support_flops`, not recursed — the cone
  feeding its `D` is the separate `"flop:<id>"` target); a child-instance output
  **stops at the instance boundary**; a `Constant` is no support source; opaque
  `MemRead`/`FsmOut` **terminate** the cone (counted, listed nowhere — the memory
  and FSM sides of that boundary are surfaced by the separate `memory_provenance`
  and `fsm_provenance` queries).
- **Document:** a `DerivedAnalysisDocument` (`src/introspect/mod.rs`) reuses the
  introspection envelope (`RequestEcho` + content `run_id`, the artifact
  pointers) with an `analysis: DerivedAnalysis` payload instead of the structural
  `introspection` payload. The **default `--introspect` document is unchanged**
  (only its `schema_version` string advances) — the cone is reached only via
  `analyze` (decision `0011` Q2).
- **Errors:** an unknown `query` kind or an unresolvable `target` ⇒ JSON-RPC
  `-32602` (the `prompts/get` validation precedent).
- **SCHEMA-DERIVED / default-off:** `DerivedAnalysis` is a pure post-hoc
  projection of the IR the generator already built — no new computed truth, no
  IR field, no generator change; the default `anvil` build and `--artifact dut`
  stay byte-identical.

The dual fan-out query, `input_reach` (schema `1.5`), is
[[semantic-introspection-input-reach]]; the per-flop reset/data query,
`flop_reset_provenance` (schema `1.6`), is
[[semantic-introspection-flop-reset-provenance]]; the design-level query,
`module_reachability` (schema `1.7`), is
[[semantic-introspection-module-reachability]]; the register-to-register
dependency graph, `flop_dependencies` (schema `1.18`), is
[[semantic-introspection-flop-dependencies]]; the per-inferrable-memory port
provenance, `memory_provenance` (schema `1.19`), is
[[semantic-introspection-memory-provenance]]; the per-generated-encoding-FSM
provenance, `fsm_provenance` (schema `1.20`), is
[[semantic-introspection-fsm-provenance]]; and the per-node immediate (1-hop)
driver adjacency, `node_drivers` (schema `1.21`), is
[[semantic-introspection-node-drivers]].

See [[semantic-introspection-derived-query-surface]],
[[agent-introspection-schema]], and [[agent-mcp-expansion-surface]].
