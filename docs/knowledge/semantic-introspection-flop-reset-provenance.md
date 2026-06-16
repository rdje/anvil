---
id: semantic-introspection-flop-reset-provenance
title: ANVIL's analyze tool answers flop_reset_provenance — per-flop reset/data provenance (schema 1.6)
answers:
  - "which flops are reset-defined vs data-driven in an ANVIL module"
  - "what is the flop_reset_provenance query"
  - "how do I ask a flop's reset kind or reset value over MCP"
  - "what is a FlopProvenance"
  - "does ANVIL expose per-flop reset provenance"
  - "how does ANVIL report a flop's mux kind or default behavior"
  - "which introspection schema version adds flop_reset_provenance"
  - "why is reset_value a string not a number"
  - "what does default_behavior zero vs hold mean for a flop"
  - "does the flop_reset_provenance surface keep the other queries byte-identical"
  - "how is a flop addressed in the analyze tool"
  - "what does analyze flop_reset_provenance return for a flopless module"
date: 2026-06-16
status: current
tags: [introspection, mcp, analyze, flop, reset, provenance, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_FLOP_RESET_PROVENANCE, FlopProvenance, DerivedAnalysis.flop_provenance, module_flop_provenance/design_flop_provenance); src/mcp/mod.rs (run_analyze flop_reset_provenance dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.6); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.5 -> 1.6 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.4` — the `flop_reset_provenance` derived query

`flop_reset_provenance` is the **third** derived-relation query of the MCP
`analyze` tool (introspection schema **`1.6`**), beside `output_support`
([[semantic-introspection-analyze-tool]]) and `input_reach`
([[semantic-introspection-input-reach]]). It answers *is each flop reset-defined
or data-driven, and how is its next state built?* — still a relation by pure
projection, never behaviour (the `0004` no-shadow-simulator / structure-first
ceiling).

- **Query / tool:** `analyze {query: "flop_reset_provenance", target?}` (DUT lane
  only). `target` is `"flop:<id>"`; omit ⇒ every flop. Cached + served as
  `anvil://artifact/<run_id>/analysis/flop_reset_provenance`.
- **Result — a `FlopProvenance` per flop:** `flop` (id), `width`, `has_reset`,
  `reset_kind` (`"none"`/`"sync"`/`"async"`), `reset_value` (the `u128` reset value
  as a **decimal string** — exact on any JSON consumer), `default_behavior`
  (`"zero"` = `FlopKind::ZeroDefault`, load 0 when no select asserted; `"hold"` =
  `FlopKind::QFeedback`, keep `Q`), `mux_kind` (`"none"`/`"one_hot"`/`"encoded"`),
  `mux_arms` (arm/data-slot count), `has_d` (`Flop::d.is_some()`).
- **Derivation — a direct projection of `Module.flops`** (ascending id), the
  purest derived query: every field already lives on the `Flop`, so there is no
  graph walk (unlike `output_support`/`input_reach`). Enum fields are mapped to
  stable strings so the wire survives an internal enum gaining variants.
- **Schema shape (`1.5 → 1.6`, additive MINOR):** `DerivedAnalysis` gains a
  **third** parallel vec `flop_provenance: Vec<FlopProvenance>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so `output_support`
  and `input_reach` documents stay **byte-identical** (the key is omitted) and
  only a `flop_reset_provenance` document carries it (with `results: []`). Each
  query populates exactly one vec; `query` is the discriminator.
- **Errors / contract:** an unknown `query` or unresolvable `"flop:<id>"` ⇒
  JSON-RPC `-32602`; a flopless module + `target = None` ⇒ an empty (not errored)
  result. SCHEMA-DERIVED / default-off: a pure post-hoc projection — no new
  computed truth, no IR field, no generator change; the default `anvil` build and
  `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]], [[semantic-introspection-input-reach]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
