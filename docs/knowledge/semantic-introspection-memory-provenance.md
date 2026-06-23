---
id: semantic-introspection-memory-provenance
title: ANVIL's analyze tool answers memory_provenance — per-inferrable-memory port provenance (shape + the support cone of each of its read/write address, write-data, write-enable ports, schema 1.19)
answers:
  - "what drives an ANVIL memory's read or write address"
  - "what is the memory_provenance query"
  - "how do I see what feeds a memory's write data or write enable over MCP"
  - "does ANVIL expose memory port provenance"
  - "what is a MemoryProvenance"
  - "how does ANVIL open the opaque MemRead leaf boundary"
  - "which introspection schema version adds memory_provenance"
  - "how is a memory addressed in the analyze tool"
  - "what is read_addr_support / write_addr_support / write_data_support / write_enable_support"
  - "does memory_provenance keep the other analyze queries byte-identical"
  - "does ANVIL report a memory's addr_width / data_width / kind / single_port"
  - "the sixth analyze derived query kind"
date: 2026-06-23
status: current
tags: [introspection, mcp, analyze, memory, port-provenance, support-cone, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_MEMORY_PROVENANCE, MemoryProvenance, DerivedAnalysis.memory_provenance, module_memory_provenance/design_memory_provenance, memory_provenance_with); src/mcp/mod.rs (run_analyze memory_provenance dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.19); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.18 -> 1.19 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.7` — the `memory_provenance` derived query

`memory_provenance` is the **sixth** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.19`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), and `flop_dependencies`
([[semantic-introspection-flop-dependencies]]). It is the **second query beyond
decision `0011`'s four named kinds** (after `flop_dependencies`), added under the
lane's open-ended-breadth clause, and the **first to open the documented
opaque-`MemRead`-leaf boundary**. It answers *what drives this memory's ports?* — a
relation over the IR by pure projection, never behaviour (the `0004`
no-shadow-simulator / structure-first ceiling).

The five prior queries treat a `Node::MemRead` as an opaque registered leaf that
*terminates* a support cone (counted, listed nowhere). `memory_provenance` instead
reports the cones feeding a memory's **input** ports — *without* recursing *through*
the memory's stored contents (still a register boundary, like a flop `Q`).

- **Query / tool:** `analyze {query: "memory_provenance", target?}` (DUT lane only).
  `target` is `"mem:<id>"` (a new address vocabulary, parallel to `"flop:<id>"`);
  omit ⇒ every memory. Cached + served as
  `anvil://artifact/<run_id>/analysis/memory_provenance`.
- **Result — a `MemoryProvenance` per memory:** `mem` (id), the structural shape
  `addr_width` / `data_width` / `kind` (`"single_port"` / `"simple_dual_port"`) /
  `single_port`, and the support cone of each of its four driving ports —
  `read_addr_support` (`raddr`), `write_addr_support` (`waddr`),
  `write_data_support` (`wdata`), `write_enable_support` (`we`). Each is a full
  `SupportCone` (inputs / flop `Q`s / child-instance outputs + cone size/depth),
  with `target` `"mem:<id>.<port>"`. Entries ascending by memory id.
- **Derivation — reuse the cone machinery:** each port's `NodeId` (always present)
  is fed to the **same** `build_cone` the support cone uses (one walker —
  full-factorization), so a port cone classifies leaves exactly like an output cone
  (opaque `MemRead`/`FsmOut` terminate it ⇒ finite/acyclic). The design variant
  operates on the **top** module and resolves instance-output leaves to
  `"<instance>.<child-output-port>"`. Pure: no IR field, no generator change.
- **Schema shape (`1.18 → 1.19`, additive MINOR):** `DerivedAnalysis` gains a
  **sixth** parallel vec `memory_provenance: Vec<MemoryProvenance>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the five prior query
  documents stay **byte-identical** (the key is omitted) and only a
  `memory_provenance` document carries it (with `results: []`). Each query populates
  exactly one vec; `query` is the discriminator.
- **`SinglePort` note:** a single-port memory shares one address node, so its read
  and write address cones carry **identical support** (only their `target` labels
  `.raddr` vs `.waddr` differ); `single_port` flags this.
- **Errors / contract:** an unknown `query`, or an unknown / out-of-range
  `"mem:<id>"`, ⇒ JSON-RPC `-32602`; a memoryless module ⇒ an empty result (the
  default-off `memory_prob` case). SCHEMA-DERIVED / default-off: a pure post-hoc
  projection — the default `anvil` build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]], [[semantic-introspection-flop-dependencies]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
