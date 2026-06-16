---
id: semantic-introspection-derived-query-surface
title: ANVIL gains a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation introspection API; the first query is the transitive support cone of an output
answers:
  - "can ANVIL answer derived queries about a generated artifact"
  - "what depends on input X in an ANVIL module"
  - "what is the support of output Y / what drives output Y"
  - "does ANVIL expose dependency cones over MCP"
  - "is there a deep semantic introspection query API"
  - "what is the SEMANTIC-INTROSPECTION-EXPANSION first query"
  - "does ANVIL have a behavioral oracle or shadow simulator"
  - "how does an agent query a generated module's structure semantically"
  - "what is the derived-analysis introspection section"
  - "how is deep semantic introspection kept SCHEMA-DERIVED"
  - "what is the anvil analyze MCP tool"
date: 2026-06-16
status: accepted
tags: [introspection, mcp, semantic, derived-query, schema-derived, agent, api, north-star]
evidence: docs/decisions/0011-semantic-introspection-derived-query-surface.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md; docs/decisions/0004-agent-introspection-mcp-lane.md; docs/decisions/0005-agent-mcp-expansion-surface.md; docs/AGENT_INTROSPECTION_SCHEMA.md; src/introspect/mod.rs; src/mcp/mod.rs; src/metrics.rs; src/ir/types.rs
---

# 0011 - SEMANTIC-INTROSPECTION-EXPANSION: a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation introspection API

- Date: 2026-06-16
- Status: accepted
- Tree: `SEMANTIC-INTROSPECTION-EXPANSION.1` (design leaf; activates the lane, splits `.1` + `.2` + future)
- Extends: decisions [`0004`](0004-agent-introspection-mcp-lane.md) (introspection/MCP lane) and
  [`0005`](0005-agent-mcp-expansion-surface.md) (read-mostly MCP breadth)

## Context

**Owner directive (`2026-06-16`):** *"ANVIL shall have deep semantic
introspection exposed through a clean API — everything shall be queryable via
MCP through a top-notch API; deep semantic introspection shall be first-class."*

Today ANVIL's agent surface (decisions `0004`/`0005`) is a complete
**SCHEMA-DERIVED structural / metric projection**:

- `--introspect` / the MCP `introspect` tool emit a versioned envelope (schema
  `1.2`) whose payload is a serde projection of existing `Config` / `Metrics` /
  `DesignMetrics` (`src/introspect/mod.rs`), with content-addressed `run_id`
  caching and the artifact `.sv`/manifest served as `ResourceRef`s.
- The MCP server (`src/mcp/mod.rs`) exposes pure tools
  (`generate`/`introspect`/`dump_config`/`coverage_gaps`), controlled tools
  (`validate`/`minimize`), resources, and five workflow prompts.
- `Metrics`/`DesignMetrics` (`src/metrics.rs`) are rich on **counts and
  distributions** (gate kinds, fanout stats, flop kinds, depth histograms,
  factorization telemetry, ~200 hierarchy-composition fields).

What is conspicuously **absent** is any **derived *relation*** over a specific
artifact: the surface answers *"how many / how deep / how shared"* but never
*"what depends on what."* An agent cannot ask:

- *"What is the support of output `y` — which primary inputs / flops / child
  instances does it structurally depend on?"*
- *"What does input `x` reach?"*
- *"Which flops are reset-defined vs data-driven?"* / *"which modules are
  reachable from the top?"*

These are exactly the questions an agent triaging a downstream-tool failure
needs in order to localize and minimize — and they are **all derivable from the
IR graph that already exists post-construction** (`Module.nodes` operands +
`drives` + flop D-cones + the instance graph), with **no new oracle**.

## Decision

**ANVIL gains a first-class, versioned, MCP-queryable, SCHEMA-DERIVED
*derived-relation* introspection API**: a new `DerivedAnalysis` payload section
in the introspection envelope plus a new **pure** MCP tool `analyze`, both
answering *derived structural/relational* questions over the already-emitted
`Module` / `Design` by pure post-hoc graph traversal. It is "deep semantic
introspection" in ANVIL's **structure-first** sense — it exposes the *meaning of
the construction graph* (what depends on what, what reaches what, what is
reset-defined) — explicitly **not** a behavioral oracle.

### What "deep semantic" means here (and the hard boundary)

`SCHEMA-DERIVED` (decision `0004`) and ROADMAP steering gap 4 (structure-first;
no shadow simulator) are **load-bearing and unchanged**. A derived query is
admissible **iff** it is a pure function of facts the IR *already holds* after
construction — a graph walk, not a re-derivation of behaviour. It therefore
exposes **relations** (support, reach, reachability, provenance), never
**behavioural truth** (signal values, timing, intended function). ANVIL has no
whole-module semantics by design; this API does not invent one.

### The API shape (the "top-notch, everything-queryable" surface)

A small, **extensible, vetted registry of derived-query kinds** — mirroring the
fixed `PROMPTS` registry pattern (decision `0005`), not a free-form query
language with arbitrary compute. Each kind is a named, pure analysis with a
typed result:

- **Introspection envelope:** a new optional `analysis: DerivedAnalysis` payload
  section (serde-projected from a new pure `analyze::*` result struct — the
  struct stays the single source of truth, preserving `SCHEMA-DERIVED`).
  Additive ⇒ introspection schema **MINOR bump `1.2 → 1.3`**.
- **MCP:** a new **pure** tool `analyze` with input
  `{ seed, lane, config?/n_params?/n_children?, query: <kind>, target?: <port-or-node> }`
  returning the derived relation, content-addressed and cached exactly like the
  existing pure tools (no FS, no spawn, no mutation). Large results (whole-cone
  dumps) are served as a `ResourceRef`, not inlined (decision `0004`
  "structured queries, not bulk dumps").

This is the clean API the owner asked for: one queryable entry point, an
extensible set of derived-fact kinds, every result a pure projection of an
existing struct.

### First query (`.2` impl leaf): the transitive support cone of an output

The foundational relation everything else composes from: for each output port
(and each flop D-cone), the **transitive fan-in support** — the set of primary
inputs / flop Qs / child-instance outputs it structurally depends on, plus cone
size and combinational depth. Symmetrically, the fanout reach of an input. The
derivation is a pure BFS/DFS over the existing node-operand graph + `drives` (+
the instance-output → child-input edges for designs); it reuses the same graph
the emitter and `metrics` already walk. It directly serves the north star: an
agent localizing a downstream-tool failure asks *"what feeds output `y`?"* to
minimize the reproducer.

## Decisive test applied

"Every introspection field is either an envelope field the schema owns or a
serde projection of an existing struct — and nothing re-derives behaviour."
A support cone is a pure projection of the IR graph that already exists; it adds
zero new computed *truth* about behaviour, only makes an **existing structural
relation** explicit. It passes the `SCHEMA-DERIVED` test and the structure-first
boundary.

## Rejected alternatives

- **A behavioural / simulation query surface** ("what is `y` when `x=…`",
  waveforms, timing paths). Forbidden by decision `0004` (no stateful
  simulator-style API, no shadow simulator) and ROADMAP gap 4. ANVIL is
  structure-first; intended behaviour is out of scope.
- **A free-form query language** (arbitrary predicates/compute over the IR).
  Rejected for the same reason raw-shell tools were (decision `0004`): unvetted
  surface. The API is a **fixed registry of named derived-query kinds**, each
  reviewed and pure — extensible by adding kinds, like the prompts registry.
- **A second source of truth** (precomputing relations into a new IR field or a
  parallel store). Rejected: the analysis is **pure post-hoc** over the existing
  IR; no generator change, no IR field, DUT byte-identical (the `coverage_gaps`
  precedent — project, don't recompute-and-store).
- **Inlining whole cones into every introspection doc.** Rejected: large derived
  results are `ResourceRef`s (structured queries, not bulk dumps); the default
  `introspect` doc stays lean.
- **Computing relations at generation time.** Rejected: it is read-only
  analysis; binding it into generation would risk the byte-identical contract.

## Consequences

- Agents gain *relational* introspection — the missing half of the surface —
  through one clean, extensible, MCP-queryable API, all SCHEMA-DERIVED.
- The introspection schema gains its first `analysis` payload section (MINOR
  `1.3`); the MCP server gains its first derived-query tool, pure and cached.
- The default `anvil` build, `--artifact dut`, and every existing gate stay
  byte-identical; `introspect`/MCP purity (decisions `0004`/`0005`) is preserved.
- The structure-first / no-shadow-simulator boundary is reaffirmed, in writing,
  as the lane's permanent ceiling.

## Open questions (resolved at `.2` / `.2a`)

- The exact `DerivedAnalysis` struct shape + the `analyze` tool's `query`-kind
  enum spelling + the `target` addressing (port id vs name vs node id).
- Whether the support cone ships in the default `introspect` payload (lean) or
  only via the explicit `analyze` tool (likely the latter, ResourceRef for big
  cones).
- Design-vs-module cone semantics across the instance boundary (how a child
  instance output participates in the parent cone).

## Tree split

`SEMANTIC-INTROSPECTION-EXPANSION` is activated:

- **`.1`** (this leaf, design) — decision `0011`: the derived-relation API, the
  SCHEMA-DERIVED boundary, the first query, the rejected alternatives. Docs-only.
- **`.2`** (impl, `proposed`) — the support-cone query: a pure `src/analyze`
  (or `src/introspect/analyze`) module + the `DerivedAnalysis` schema `1.3`
  section + the pure MCP `analyze` tool + book/USER_GUIDE/KM; default-off / DUT
  byte-identical. Pre-split into `.2a` (design-detail) + `.2b` (impl) when picked
  if broad.
- **future (`.3`+)** — additional derived-query kinds (flop reset provenance,
  module reachability from top, per-module hierarchy depth), each a new vetted
  kind in the same registry.

## Links

- Owner directive: `2026-06-16` (deep semantic introspection first-class +
  MCP-queryable top-notch API).
- Parent decisions: `0004` (introspection/MCP lane, `SCHEMA-DERIVED`,
  no-shadow-simulator), `0005` (read-mostly MCP breadth, project-don't-recompute).
- North star: `project_anvil_north_star` (relational introspection helps agents
  localize/minimize downstream-tool failures); ROADMAP steering gap 4
  (structure-first — the permanent ceiling).
- Reuse / touch points: `src/introspect/mod.rs` (payload section + builder),
  `docs/AGENT_INTROSPECTION_SCHEMA.md` (schema `1.3`), `src/mcp/mod.rs` (`analyze`
  tool + resource), `src/metrics.rs` / `src/ir/types.rs` (the IR graph the
  analysis walks), `book/src/agent-mcp.md` (user-facing).
