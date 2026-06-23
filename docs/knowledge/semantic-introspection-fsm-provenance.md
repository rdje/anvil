---
id: semantic-introspection-fsm-provenance
title: ANVIL's analyze tool answers fsm_provenance — per-generated-encoding-FSM provenance (shape + the support cone of its transition-select sel input, schema 1.20)
answers:
  - "what drives an ANVIL FSM's state transitions"
  - "what is the fsm_provenance query"
  - "how do I see what feeds an FSM's sel input over MCP"
  - "does ANVIL expose FSM provenance"
  - "what is an FsmProvenance"
  - "how does ANVIL open the opaque FsmOut leaf boundary"
  - "which introspection schema version adds fsm_provenance"
  - "how is an FSM addressed in the analyze tool"
  - "what is sel_support"
  - "does fsm_provenance report num_states / encoding / state_width / is_mealy"
  - "does fsm_provenance keep the other analyze queries byte-identical"
  - "does ANVIL report whether an FSM is Moore or Mealy over introspection"
  - "the seventh analyze derived query kind"
date: 2026-06-23
status: current
tags: [introspection, mcp, analyze, fsm, provenance, support-cone, derived-relation, schema, structure-first]
evidence: src/introspect/analyze.rs (QUERY_FSM_PROVENANCE, FsmProvenance, DerivedAnalysis.fsm_provenance, module_fsm_provenance/design_fsm_provenance, fsm_provenance_with); src/mcp/mod.rs (run_analyze fsm_provenance dispatch + analyze_schema enum); src/introspect/mod.rs (SCHEMA_VERSION = 1.20); docs/AGENT_INTROSPECTION_SCHEMA.md (section 6.7 + the 1.19 -> 1.20 changelog); book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md
reverify: cargo test --lib analyze
---

# `SEMANTIC-INTROSPECTION-EXPANSION.8` — the `fsm_provenance` derived query

`fsm_provenance` is the **seventh** derived-relation query of the MCP `analyze`
tool (introspection schema **`1.20`**), beside `output_support`
([[semantic-introspection-analyze-tool]]), `input_reach`
([[semantic-introspection-input-reach]]), `flop_reset_provenance`
([[semantic-introspection-flop-reset-provenance]]), `module_reachability`
([[semantic-introspection-module-reachability]]), `flop_dependencies`
([[semantic-introspection-flop-dependencies]]), and `memory_provenance`
([[semantic-introspection-memory-provenance]]). It is the **third query beyond
decision `0011`'s four named kinds** (after `flop_dependencies` and
`memory_provenance`), added under the lane's open-ended-breadth clause, and the
**direct sibling of `memory_provenance`** — the **second to open a documented
opaque-leaf boundary**, the `Node::FsmOut` analog of the `MemRead` one. It answers
*what drives this FSM's state machine?* — a relation over the IR by pure projection,
never behaviour (the `0004` no-shadow-simulator / structure-first ceiling).

The six prior queries treat a `Node::FsmOut` as an opaque registered leaf that
*terminates* a support cone (counted, listed nowhere). `fsm_provenance` instead
reports the cone feeding the FSM's one generated **input** port — its
transition-select cone `sel` — *without* recursing *through* the FSM's registered
state (a register boundary, like a flop `Q`) and *without* surfacing the
transition/output table **values** (the construction-time-resolved state-machine
behaviour — deliberately out of scope, a relation never behaviour).

- **Query / tool:** `analyze {query: "fsm_provenance", target?}` (DUT lane only).
  `target` is `"fsm:<id>"` (a new address vocabulary, parallel to `"mem:<id>"` /
  `"flop:<id>"`); omit ⇒ every FSM. Cached + served as
  `anvil://artifact/<run_id>/analysis/fsm_provenance`.
- **Result — an `FsmProvenance` per FSM:** `fsm` (id), the structural shape
  `num_states` / `encoding` (`"binary"` / `"one_hot"` / `"gray"`) / `state_width`
  (the encoded `state_q` register width) / `sel_width` / `out_width` / `is_mealy`
  (Mealy output decode over `(state_q, sel)` vs Moore over state only), and
  `sel_support` — the full `SupportCone` of the transition-select input (inputs /
  flop `Q`s / child-instance outputs + cone size/depth), with `target`
  `"fsm:<id>.sel"`. Entries ascending by FSM id.
- **Derivation — reuse the cone machinery:** the FSM's `sel` `NodeId` (always
  present) is fed to the **same** `build_cone` the support cone uses (one walker —
  full-factorization), so the `sel` cone classifies leaves exactly like an output
  cone (opaque `MemRead`/`FsmOut` terminate it ⇒ finite/acyclic). The structural
  fields are a direct projection of `Fsm` (`FsmEncoding` → a stable string +
  `FsmEncoding::state_width`; `is_mealy` = `Fsm::is_mealy()`). The design variant
  operates on the **top** module and resolves instance-output leaves to
  `"<instance>.<child-output-port>"`. Pure: no IR field, no generator change.
- **Schema shape (`1.19 → 1.20`, additive MINOR):** `DerivedAnalysis` gains a
  **seventh** parallel vec `fsm_provenance: Vec<FsmProvenance>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so the six prior query
  documents stay **byte-identical** (the key is omitted) and only a `fsm_provenance`
  document carries it (with `results: []`). Each query populates exactly one vec;
  `query` is the discriminator.
- **One generated input cone:** an FSM has exactly **one** generated input cone
  (`sel`) — unlike a memory's four ports — because its transition/output tables are
  construction-time constants (`Vec<u32>` / `Vec<u128>`), not `NodeId` cones. The
  query reports the FSM's shape, not its behaviour.
- **Errors / contract:** an unknown `query`, or an unknown / out-of-range
  `"fsm:<id>"`, ⇒ JSON-RPC `-32602`; an FSM-less module ⇒ an empty result (the
  default-off `fsm_prob` case). SCHEMA-DERIVED / default-off: a pure post-hoc
  projection — the default `anvil` build and `--artifact dut` stay byte-identical.

See [[semantic-introspection-analyze-tool]], [[semantic-introspection-memory-provenance]],
[[semantic-introspection-flop-dependencies]],
[[semantic-introspection-input-reach]],
[[semantic-introspection-flop-reset-provenance]],
[[semantic-introspection-module-reachability]],
[[semantic-introspection-derived-query-surface]], and [[agent-introspection-schema]].
