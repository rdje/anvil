# SEMANTIC-INTROSPECTION-EXPANSION: behavioral query surface beyond structural projection

## Metadata

- Tree ID: `SEMANTIC-INTROSPECTION-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability — deeper agent/introspection surface (extends AGENT-INTROSPECTION-MCP / AGENT-MCP-EXPANSION)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**activated by explicit owner directive**; `.1` design landed — decision `0011`; `.2a` design-detail landed — resolved `0011`'s open questions + the `.2b` impl shape; split `.2` → `.2a` done + `.2b`; split `.2b` → `.2b.1` + `.2b.2`; `.2b.1` landed — the pure `src/introspect/analyze.rs` support-cone analysis core, DUT byte-identical; frontier → `.2b.2`)
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
  Status: `active`
  Goal: `Implement the first derived query (the output support cone) as a pure post-hoc analysis + the DerivedAnalysis schema 1.3 + the pure MCP analyze tool, default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.2a`, `SEMANTIC-INTROSPECTION-EXPANSION.2b`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2a`
  Status: `done`
  Goal: `Design-detail leaf: resolve decision 0011's three open questions (the DerivedAnalysis/SupportCone struct shape; the query-kind enum + target addressing; design-vs-module cone semantics + whether the support cone ships in the default introspect payload) against the real src/introspect/mod.rs + src/mcp/mod.rs code, and fix the .2b impl shape. Split .2 into .2a + .2b.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving all three questions grounded in real code; the tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `DerivedAnalysis { query: String, results: Vec<SupportCone> } + SupportCone { target, support_inputs[], support_flops[], support_instance_outputs[], cone_nodes, cone_depth } (serde + Default, sorted Vecs ⇒ deterministic bytes) in a new pure src/introspect/analyze.rs; module_support_cones(m, target: Option<&str>) + design variant do a memoized DFS over the existing Module.nodes operands + drives + flop D-cones (NO IR field / NO generator change — coverage_gaps project-don't-recompute precedent). query-kind enum: output_support first (future: input_reach, flop_reset_provenance, module_reachability); unknown query/target → -32602 (prompts/get precedent). target = output port NAME (absent ⇒ all outputs); flop D-cones as "flop:<id>". The DEFAULT introspect payload is UNTOUCHED (no analysis field) — reached only via a new PURE MCP analyze tool returning a standalone DerivedAnalysisDocument reusing the envelope (schema 1.3 + RequestEcho/run_id); big cones inline first-cut (ResourceRef spill-over a noted .2b option). Cone STOPS at the instance boundary (child-instance outputs are support leaves; recursion is a future kind). Schema 1.2 -> 1.3 MINOR bump (SCHEMA_VERSION + schema-doc + ~5 "1.2" test assertions, the .2b.1 procedure); DUT .sv byte-identical (introspect not in snapshots). Pre-split .2b -> .2b.1 (analyze module + types, lib-tested) + .2b.2 (MCP tool + schema + docs) if broad.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b`
  Status: `active`
  Goal: `Implement the .2a design: the pure support-cone analysis + types, the schema 1.2 -> 1.3 bump, and the pure MCP analyze tool. Split (it spans two reviewable ownership areas) into .2b.1 (pure analyze module + types, lib-tested) + .2b.2 (schema bump + MCP analyze tool + docs + KM).`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.2b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b.1`
  Status: `done`
  Goal: `The pure derived-relation analysis core: a new src/introspect/analyze.rs carrying DerivedAnalysis { query, results: Vec<SupportCone> } + SupportCone { target, support_inputs[], support_flops[], support_instance_outputs[], cone_nodes, cone_depth } (serde + Default, BTreeSet -> sorted Vec => deterministic bytes) + the pure builders module_support_cones(&Module, Option<&str>) / design_support_cones(&Design, Option<&str>) doing a memoized combinational fan-in DFS over the existing IR graph. FlopQ is a register-boundary support leaf (recorded, not recursed); child-instance outputs are leaves (the cone stops at the instance boundary); a flop D input is addressable as "flop:<id>"; opaque MemRead/FsmOut terminate the cone (documented boundary, a future kind). No IR field / no generator change; not wired to any emit path => DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. exact cone-correctness on hand-built modules (combinational support; flop-boundary leaf not recursed; "flop:<id>" target; child-instance-output leaf name resolution in a design; constant-not-support; mem/fsm-read termination; unknown target => no cone; determinism + sorted vecs); snapshots 6/6 byte-identical (analyze.rs is not in any output path).`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`
  Status: `proposed`
  Goal: `Wire the analysis to the surface: schema SCHEMA_VERSION 1.2 -> 1.3 (+ the ~5 "1.2" test-assertion bumps + a schema-doc section/changelog) + a DerivedAnalysisDocument envelope (RequestEcho/run_id reuse + an analysis: DerivedAnalysis payload) + the pure MCP analyze tool (dispatch + tools/list + the anvil://artifact/<run_id>/analysis/<query> resource), unknown query/target => -32602; + book(agent-mcp)/USER_GUIDE/schema-doc + a KM fact. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the support cone (cached), unknown query/target -> -32602; schema_version = 1.3 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; committed through COMMIT.md with the leaf id.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | `proposed` | Wire the `.2b.1` analysis to the surface: schema `1.2 → 1.3` bump + `DerivedAnalysisDocument` envelope + the pure MCP `analyze` tool (dispatch + `tools/list` + analysis resource, unknown query/target → `-32602`) + book(`agent-mcp`)/USER_GUIDE/schema-doc + a KM fact. Default-off / DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `done` | Landed the pure derived-relation analysis core (`src/introspect/analyze.rs`: the `DerivedAnalysis`/`SupportCone` types + `module_support_cones`/`design_support_cones`), a memoized combinational fan-in DFS over the existing IR graph; lib-tested for exact cone correctness + determinism + the flop/instance/mem-fsm boundaries + unknown-target. No IR/generator change → DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `done` | Resolved decision `0011`'s three open questions (the `DerivedAnalysis`/`SupportCone` shape; `query`-kind enum + `target` addressing; cone-stops-at-instance-boundary + default-introspect-stays-lean). Split `.2` → `.2a`/`.2b`. No source change. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.1` | `done` | Landed decision `0011` — the first-class MCP-queryable SCHEMA-DERIVED derived-relation API + the no-shadow-simulator boundary + the first query (output support cone) + rejected alternatives. Split `.1`/`.2`/future. No source change. |

## Decisions

- `2026-06-16` (`.2a`, design-detail in `DEVELOPMENT_NOTES.md`): resolved decision
  `0011`'s three open questions. (1) `DerivedAnalysis { query, results:
  Vec<SupportCone> }` + `SupportCone { target, support_inputs[], support_flops[],
  support_instance_outputs[], cone_nodes, cone_depth }` (serde + `Default`, sorted
  Vecs ⇒ deterministic) in a new pure `src/introspect/analyze.rs`;
  `module_support_cones(m, target: Option<&str>)` does a memoized DFS over the
  existing IR graph (no IR field / no generator change). `query`-kind:
  `output_support` first; `target` = output port name (absent ⇒ all outputs);
  unknown query/target ⇒ `-32602`. (2) The **default `introspect` payload stays
  lean** (no `analysis` field) — the cone is reached only via a new **pure** MCP
  `analyze` tool returning a standalone `DerivedAnalysisDocument` (envelope reuse,
  schema `1.3`); big cones inline first-cut (ResourceRef spill-over a noted `.2b`
  option). (3) The cone **stops at the instance boundary** (child-instance
  outputs are support leaves; recursion is a future kind). Schema `1.2 → 1.3`
  MINOR bump (DUT `.sv` byte-identical — introspect not in snapshots). Split `.2`
  → `.2a` (done) + `.2b` (impl); pre-split `.2b` → `.2b.1` (analyze module) /
  `.2b.2` (MCP tool + schema + docs) if broad.
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
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | New pure module `src/introspect/analyze.rs` + `pub mod analyze;` in `src/introspect/mod.rs`. `cargo test --lib` **421 passed / 0 failed / 2 ignored** (incl. 9 new `introspect::analyze` proofs: exact combinational support; flop-boundary leaf not recursed + `"flop:<id>"` target; constant-not-support; opaque mem-read termination; absent-target ⇒ per-output cones; unknown-target ⇒ no cone; design child-instance-output name resolution; determinism + sorted; shared-fan-in counted once). `cargo test --test snapshots` **6/6 byte-identical** (DUT `.sv` unchanged). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; baseline `cargo check --all-targets` clean. `bash scripts/check_memory_architecture.sh` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.2a` | Design-detail leaf, no source change (grounded in a fresh read of `src/introspect/mod.rs` `IntrospectionPayload`/`IntrospectionDocument`/`RequestEcho`/`content_run_id_for_knobs` + `src/mcp/mod.rs` pure-tool dispatch + `CachedArtifact`). `DEVELOPMENT_NOTES.md` design-detail entry + tree split. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.1` | Design/decision leaf, no source change (grounded in a fresh survey of `docs/AGENT_INTROSPECTION_SCHEMA.md`, `src/introspect/mod.rs`, `src/mcp/mod.rs`, `src/metrics.rs`, `src/ir/types.rs`, decisions `0004`/`0005`). Decision `0011` + `INDEX.md` + tree activation/split; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-15` | `SEMANTIC-INTROSPECTION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1 — pure support-cone analysis core` | New `src/introspect/analyze.rs`: `DerivedAnalysis`/`SupportCone` + `module_support_cones`/`design_support_cones` (combinational fan-in DFS over the existing IR; FlopQ a register-boundary leaf, `"flop:<id>"` targets, instance-boundary stop, opaque mem/fsm termination). 9 in-crate proofs; DUT byte-identical (no IR/generator change). Split `.2b` → `.2b.1`/`.2b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `SEMANTIC-INTROSPECTION-EXPANSION.2a — support-cone impl design-detail` | Resolved `0011`'s 3 open questions: the `DerivedAnalysis`/`SupportCone` shape; `output_support` query-kind + name `target`; default-introspect-stays-lean + cone-stops-at-instance-boundary; schema `1.2→1.3`. Split `.2` → `.2a`/`.2b`. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION.1` | `SEMANTIC-INTROSPECTION-EXPANSION.1 — activate lane + derived-query API design` | Decision `0011`: a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation API (`DerivedAnalysis` schema `1.3` + pure MCP `analyze` tool); first query = the output support cone. Activated the lane by owner directive; split `.1`/`.2`/future. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-16`: split `.2b` → `.2b.1` (pure analysis core) + `.2b.2` (surface
  wiring), and `.2b.1` landed: the pure `src/introspect/analyze.rs` support-cone
  analysis core — `DerivedAnalysis`/`SupportCone` types + the
  `module_support_cones`/`design_support_cones` combinational fan-in DFS over the
  already-emitted IR. Resolved the `.2a` "+ flop D-cones" wording into a clean
  rule: the cone is purely combinational (`FlopQ` is a register-boundary support
  leaf; a flop's `D` cone is the separate target `"flop:<id>"`), child-instance
  outputs stop at the boundary, and opaque `MemRead`/`FsmOut` terminate the cone
  (a documented boundary + future kind). 9 in-crate proofs; DUT byte-identical
  (no IR field / no generator change; not wired to any emit path). Frontier
  advances to `.2b.2` (schema `1.2→1.3` + the pure MCP `analyze` tool + docs/KM).
- `2026-06-16`: `.2a` design-detail landed (no source change): resolved decision
  `0011`'s three open questions and fixed the `.2b` impl shape — the
  `DerivedAnalysis`/`SupportCone` struct in a pure `src/introspect/analyze.rs`;
  `output_support` first query-kind + name-`target` addressing; the default
  `introspect` payload stays lean (cone reached only via a new pure MCP `analyze`
  tool, schema `1.3`); the cone stops at the instance boundary. Split `.2` →
  `.2a` (done) + `.2b` (impl). Frontier advances to `.2b`.
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
