# SEMANTIC-INTROSPECTION-EXPANSION: behavioral query surface beyond structural projection

## Metadata

- Tree ID: `SEMANTIC-INTROSPECTION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability — deeper agent/introspection surface (extends AGENT-INTROSPECTION-MCP / AGENT-MCP-EXPANSION)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**activated by explicit owner directive**: deep semantic introspection first-class + everything MCP-queryable via a top-notch API; `.1` design landed — decision `0011`; frontier → `.2`)
- Owner: repo-local workflow
- Note: registered `proposed` by owner roadmap steering (`2026-06-15`); **activated
  `2026-06-16` by explicit owner directive** ("deep semantic introspection shall
  be first-class … everything shall be queryable via MCP through a top-notch
  API"), taking priority over the table order per the PNT owner-names-a-lane rule.

## Goal

Deepen ANVIL's introspection surface from today's **structural / metric
projection** (the versioned `--introspect` envelope + MCP read-mostly tools,
`AGENT-INTROSPECTION-MCP` / `AGENT-MCP-EXPANSION`, decisions `0004`/`0005`) toward
a **behavioral query surface** — letting an agent ask derived/behavioral
questions about a generated artifact (e.g. "what cones depend on input X",
"which flops are reset-defined", "what is the support of output Y") beyond the
raw serde projection of `Config`/`Metrics`/`DesignMetrics`.

## Non-Goals

- No stateful simulator-style session API and no shadow simulator (the
  `agent-introspection-mcp-lane` boundary, decision `0004`; ROADMAP steering gap
  4 — structure-first, a full shadow simulator stays out of scope).
- No new computed truth that drifts from the generator's own facts: any
  behavioral query must be derived from existing IR / construction-time facts,
  not a second source of truth (the `SCHEMA-DERIVED` invariant, decision `0004`).
- No change to the default-off / DUT-byte-identical contract of the MCP/introspect
  lanes.

## Acceptance Criteria

- Each landed query is derived from existing IR/metrics (no drift, no second
  oracle), versioned in the introspection schema, and default-off / DUT
  byte-identical.
- Live docs + book (`agent-mcp.md`) + schema doc + a Knowledge Map fact per
  durable query surface.
- Every leaf committed through `COMMIT.md` with its leaf id.

## Task Tree

- ID: `SEMANTIC-INTROSPECTION-EXPANSION`
  Status: `active`
  Goal: `A first-class, MCP-queryable, SCHEMA-DERIVED derived-RELATION query surface over generated artifacts (what depends on what), derived from existing IR facts — never a behavioral oracle.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.1`, `SEMANTIC-INTROSPECTION-EXPANSION.2`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf: inventory candidate derived queries that stay SCHEMA-DERIVED (no new oracle, no shadow simulator), define the first-class MCP-queryable API shape, pick the first query, fix its schema versioning + derivation + default-off contract, and split the tree — before any code.`
  Acceptance: `A decision record naming the API surface, the SCHEMA-DERIVED boundary, the first query, and its schema/versioning; no source change; self-checks clean.`
  Result: `Decision 0011. The lane delivers a first-class, versioned, MCP-queryable, SCHEMA-DERIVED derived-RELATION query API: a new optional DerivedAnalysis introspection payload section (schema MINOR 1.2 -> 1.3) + a new PURE MCP analyze tool, both answering derived structural/relational questions over the already-emitted Module/Design by pure post-hoc graph traversal — relations (support, reach, reachability, provenance), NOT behavioral truth (the decision 0004 no-shadow-simulator / structure-first boundary is the permanent ceiling). API = a fixed, extensible registry of named derived-query KINDS (the prompts-registry pattern), each pure + typed; large results served as ResourceRefs (structured queries, not bulk dumps). First query (.2) = the transitive fan-in SUPPORT CONE of each output (+ symmetric input fanout reach): the set of primary inputs / flop Qs / child-instance outputs an output structurally depends on, + cone size/depth, by pure BFS/DFS over the existing node-operand graph + drives. Default-off / DUT byte-identical (pure post-hoc, no IR change, no generator change — the coverage_gaps project-don't-recompute precedent). Rejected: behavioral/simulation queries (0004), a free-form query language, a second source of truth, inlining whole cones, computing relations at gen time. Split into .1 (done) + .2 (impl) + future kinds (.3+: reset provenance, module reachability, per-module depth).`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2`
  Status: `proposed`
  Goal: `Implement the first derived query: the transitive support cone of an output (+ input fanout reach) as a pure post-hoc analysis + the DerivedAnalysis schema 1.3 payload section + the pure MCP analyze tool, default-off / DUT byte-identical. Pre-split into .2a (design-detail) + .2b (impl) when picked if broad.`
  Acceptance: `A pure analyze module computing the support cone from the IR graph (no IR/generator change); the DerivedAnalysis section + introspection schema 1.2 -> 1.3 (+ schema doc + test-assertion bumps); the pure MCP analyze tool (cached, ResourceRef for big cones); cargo check/clippy/fmt/test clean; snapshots 6/6 byte-identical; introspect/MCP purity preserved; book(agent-mcp)/USER_GUIDE/schema doc + KM fact; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEMANTIC-INTROSPECTION-EXPANSION.2` | `proposed` | Implement the `.1` design: the transitive output **support cone** query as a pure post-hoc IR-graph analysis + the `DerivedAnalysis` schema `1.3` payload section + the pure MCP `analyze` tool, default-off / DUT byte-identical. Pre-split into `.2a`/`.2b` if broad. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.1` | `done` | Landed decision `0011` — the first-class MCP-queryable SCHEMA-DERIVED derived-relation API + the no-shadow-simulator boundary + the first query (output support cone) + rejected alternatives. Split `.1`/`.2`/future. No source change. |

## Decisions

- `2026-06-16` (`.1`, decision [`0011`](../decisions/0011-semantic-introspection-derived-query-surface.md)):
  activated the lane by explicit owner directive. The surface is a first-class,
  versioned, MCP-queryable, **SCHEMA-DERIVED derived-relation** API (a
  `DerivedAnalysis` introspection section, schema `1.3`, + a pure MCP `analyze`
  tool) answering *what depends on what* over the already-emitted IR by pure
  graph traversal — **relations, not behaviour** (the `0004` no-shadow-simulator
  / structure-first boundary is the permanent ceiling). First query = the output
  **support cone**. API = a fixed, extensible registry of named query kinds
  (prompts-registry pattern); big results are `ResourceRef`s. Default-off / DUT
  byte-identical (pure post-hoc, the `coverage_gaps` project-don't-recompute
  precedent). Rejected: behavioral/simulation queries, a free-form query
  language, a second source of truth, inlining whole cones, gen-time computation.
- `2026-06-15`: Registered `proposed` by owner roadmap steering as a named future
  capability lane. Not started; `SV-VERSION-TARGETING` was activated first.

## Open Questions

- Which derived/behavioral query is highest-leverage first while staying
  `SCHEMA-DERIVED` — resolved by `.1` when activated.

## Blockers

- None (not active by choice, not dependency).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.1` | Design/decision leaf, no source change (grounded in a fresh survey of `docs/AGENT_INTROSPECTION_SCHEMA.md`, `src/introspect/mod.rs`, `src/mcp/mod.rs`, `src/metrics.rs`, `src/ir/types.rs`, decisions `0004`/`0005`). Decision `0011` + `INDEX.md` + tree activation/split; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-15` | `SEMANTIC-INTROSPECTION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEMANTIC-INTROSPECTION-EXPANSION.1` | `SEMANTIC-INTROSPECTION-EXPANSION.1 — activate lane + derived-query API design` | Decision `0011`: a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation API (`DerivedAnalysis` schema `1.3` + pure MCP `analyze` tool); first query = the output support cone. Activated the lane by owner directive; split `.1`/`.2`/future. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-16`: **Activated by explicit owner directive** ("deep semantic
  introspection shall be first-class … everything queryable via MCP through a
  top-notch API"). `.1` design landed — decision `0011`: a first-class,
  versioned, MCP-queryable, SCHEMA-DERIVED derived-relation API (`DerivedAnalysis`
  introspection section, schema `1.3`, + a pure MCP `analyze` tool) answering
  *what depends on what* by pure IR-graph traversal — relations, not behaviour
  (the `0004` no-shadow-simulator boundary is the permanent ceiling). First query
  = the output support cone. Split `.1` (done) + `.2` (impl) + future kinds.
  Frontier advances to `.2`.
- `2026-06-15`: Created and registered `proposed` (owner-directed sibling lane).
