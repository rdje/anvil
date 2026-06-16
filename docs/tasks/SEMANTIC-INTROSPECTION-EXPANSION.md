# SEMANTIC-INTROSPECTION-EXPANSION: behavioral query surface beyond structural projection

## Metadata

- Tree ID: `SEMANTIC-INTROSPECTION-EXPANSION`
- Status: `active` (two queries delivered: `output_support` `.1`/`.2` + `input_reach` `.3` — both end-to-end, schema `1.5`; **no active frontier**, future kinds `.4+`)
- Roadmap lane: `Capability — deeper agent/introspection surface (extends AGENT-INTROSPECTION-MCP / AGENT-MCP-EXPANSION)`
- Created: `2026-06-15`
- Last updated: `2026-06-16` (**activated by explicit owner directive**; `.1` design — decision `0011`; `.2a` design-detail; `.2b.1` the pure analysis core; `.2b.2` the agent-facing surface — schema `1.3` + the pure MCP `analyze` tool + the `DerivedAnalysisDocument` + docs/KM. **`.2` done — the first query (output support cone) is delivered end-to-end, DUT byte-identical.** `.3` (`input_reach`) opened: `.3a` design-detail **done** (DEVELOPMENT_NOTES entry: result shape = second `reach_results` vec, derivation = invert the support relation, source addressing + `"flop:<id>"` direction-by-query duality, schema `1.4 → 1.5`); `.3b` pre-split → `.3b.1` (pure core, **frontier**) + `.3b.2` (surface).)
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
  Status: `done`
  Goal: `Implement the first derived query (the output support cone) as a pure post-hoc analysis + the DerivedAnalysis schema 1.3 + the pure MCP analyze tool, default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.2a`, `SEMANTIC-INTROSPECTION-EXPANSION.2b`
  Result: `Done — both children done. The first query (output support cone) is delivered end-to-end: the pure analysis core (.2b.1) + the schema 1.3 / DerivedAnalysisDocument / pure MCP analyze tool / docs / KM (.2b.2). DUT byte-identical (snapshots 6/6).`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2a`
  Status: `done`
  Goal: `Design-detail leaf: resolve decision 0011's three open questions (the DerivedAnalysis/SupportCone struct shape; the query-kind enum + target addressing; design-vs-module cone semantics + whether the support cone ships in the default introspect payload) against the real src/introspect/mod.rs + src/mcp/mod.rs code, and fix the .2b impl shape. Split .2 into .2a + .2b.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving all three questions grounded in real code; the tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `DerivedAnalysis { query: String, results: Vec<SupportCone> } + SupportCone { target, support_inputs[], support_flops[], support_instance_outputs[], cone_nodes, cone_depth } (serde + Default, sorted Vecs ⇒ deterministic bytes) in a new pure src/introspect/analyze.rs; module_support_cones(m, target: Option<&str>) + design variant do a memoized DFS over the existing Module.nodes operands + drives + flop D-cones (NO IR field / NO generator change — coverage_gaps project-don't-recompute precedent). query-kind enum: output_support first (future: input_reach, flop_reset_provenance, module_reachability); unknown query/target → -32602 (prompts/get precedent). target = output port NAME (absent ⇒ all outputs); flop D-cones as "flop:<id>". The DEFAULT introspect payload is UNTOUCHED (no analysis field) — reached only via a new PURE MCP analyze tool returning a standalone DerivedAnalysisDocument reusing the envelope (schema 1.3 + RequestEcho/run_id); big cones inline first-cut (ResourceRef spill-over a noted .2b option). Cone STOPS at the instance boundary (child-instance outputs are support leaves; recursion is a future kind). Schema 1.2 -> 1.3 MINOR bump (SCHEMA_VERSION + schema-doc + ~5 "1.2" test assertions, the .2b.1 procedure); DUT .sv byte-identical (introspect not in snapshots). Pre-split .2b -> .2b.1 (analyze module + types, lib-tested) + .2b.2 (MCP tool + schema + docs) if broad.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b`
  Status: `done`
  Goal: `Implement the .2a design: the pure support-cone analysis + types, the schema 1.2 -> 1.3 bump, and the pure MCP analyze tool. Split (it spans two reviewable ownership areas) into .2b.1 (pure analyze module + types, lib-tested) + .2b.2 (schema bump + MCP analyze tool + docs + KM).`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.2b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`
  Result: `Done — both children done. The pure analysis core + the agent-facing surface (schema 1.3, the DerivedAnalysisDocument, the pure MCP analyze tool, the analysis resource) + docs + KM, DUT byte-identical.`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b.1`
  Status: `done`
  Goal: `The pure derived-relation analysis core: a new src/introspect/analyze.rs carrying DerivedAnalysis { query, results: Vec<SupportCone> } + SupportCone { target, support_inputs[], support_flops[], support_instance_outputs[], cone_nodes, cone_depth } (serde + Default, BTreeSet -> sorted Vec => deterministic bytes) + the pure builders module_support_cones(&Module, Option<&str>) / design_support_cones(&Design, Option<&str>) doing a memoized combinational fan-in DFS over the existing IR graph. FlopQ is a register-boundary support leaf (recorded, not recursed); child-instance outputs are leaves (the cone stops at the instance boundary); a flop D input is addressable as "flop:<id>"; opaque MemRead/FsmOut terminate the cone (documented boundary, a future kind). No IR field / no generator change; not wired to any emit path => DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. exact cone-correctness on hand-built modules (combinational support; flop-boundary leaf not recursed; "flop:<id>" target; child-instance-output leaf name resolution in a design; constant-not-support; mem/fsm-read termination; unknown target => no cone; determinism + sorted vecs); snapshots 6/6 byte-identical (analyze.rs is not in any output path).`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.2b.2`
  Status: `done`
  Goal: `Wire the analysis to the surface: schema SCHEMA_VERSION 1.2 -> 1.3 (+ the ~5 "1.2" test-assertion bumps + a schema-doc section/changelog) + a DerivedAnalysisDocument envelope (RequestEcho/run_id reuse + an analysis: DerivedAnalysis payload) + the pure MCP analyze tool (dispatch + tools/list + the anvil://artifact/<run_id>/analysis/<query> resource), unknown query/target => -32602; + book(agent-mcp)/USER_GUIDE/schema-doc + a KM fact. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the support cone (cached), unknown query/target -> -32602; schema_version = 1.3 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; committed through COMMIT.md with the leaf id.`
  Result: `Done. SCHEMA_VERSION 1.2->1.3 + 6 "1.2" test-assertion bumps; DerivedAnalysisDocument + derived_analysis_document in src/introspect/mod.rs; the pure run_analyze MCP tool in src/mcp/mod.rs (DUT-only; query validated against analyze::supported_query_kinds(); unknown query/target -> -32602; cached in CachedArtifact.analyses; served at anvil://artifact/<run_id>/analysis/<query>); analyze in tools/list + instructions. Docs: schema-doc 6.7 + 1.3 changelog, book agent-mcp (analyze row + worked example + the stale 1.0 example fixed), USER_GUIDE MCP tool/resource lists + 1.2->1.3, KM fact semantic-introspection-analyze-tool. Validation: cargo test --lib 427/0/2 (incl. 5 mcp analyze proofs + the derived-document proof); snapshots 6/6; clippy -D warnings + fmt clean; mdbook build clean; book_examples 3/3; KM in sync; anvil-mcp stdio e2e smoke (schema 1.3 cone + -32602).`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.3`
  Status: `done`
  Goal: `The second derived query — input_reach: the dual fan-OUT of the delivered output_support cone (which outputs / flop-D cones a given input port / flop Q / child-instance output structurally reaches). Owner-directed (2026-06-16) as the next lane. Same SCHEMA-DERIVED / pure-post-hoc / default-off / DUT-byte-identical contract; same first-class MCP analyze registry (a new "input_reach" query kind added to analyze::supported_query_kinds()).`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.3a`, `SEMANTIC-INTROSPECTION-EXPANSION.3b`
  Result: `Done — all children done. input_reach is delivered end-to-end: .3a design + .3b.1 pure core + .3b.2 surface (registry + run_analyze dispatch + schema 1.4 → 1.5 + analyze_schema enum + schema-doc §6.7/changelog + book/USER_GUIDE/README + KM card). The MCP analyze tool answers query=input_reach with the dual fan-out (reaches_outputs/reaches_flops/fanout_targets per source); output_support stays byte-identical (reach_results omitted); unknown source ⇒ -32602; e2e anvil-mcp stdio smoke confirms schema 1.5 + 37 reach results + the -32602 path. DUT byte-identical (snapshots 6/6).`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.3a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground input_reach in the real src/introspect/analyze.rs + mod.rs + mcp.rs. Pin: (1) the result shape — likely a new ReachResult { target, reaches_outputs[], reaches_flops[], ... } reusing DerivedAnalysis (decide whether DerivedAnalysis.results stays Vec<SupportCone> or generalizes to an enum/second vec — a schema-shape choice that drives the MINOR bump 1.4 -> 1.5 vs reuse); (2) the derivation — invert per-output/per-flop-D support (reuse the existing module_support_cones builder: input X reaches output Y iff X in support(Y)) vs a forward consumers BFS, choosing the pure/cheap one with no IR/generator change; (3) target addressing — input port NAME (absent => all inputs), plus "flop:<id>" (Q as a reach source) and child-instance-output sources, unknown target => -32602; (4) the schema-version decision (new query kind alone may not need a bump if the document shape is reused; a new result struct does). No source change; DEVELOPMENT_NOTES design-detail entry + the .3b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the four points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all four points, grounded in a fresh read of analyze.rs/mod.rs/mcp.rs. (1) Result shape: a new ReachResult { target (the SOURCE), reaches_outputs[], reaches_flops[], fanout_targets } (dual of SupportCone, serde + Default + sorted vecs); DerivedAnalysis gains a SECOND parallel vec reach_results: Vec<ReachResult> with #[serde(default, skip_serializing_if = "Vec::is_empty")] (rejected: a tagged enum that would break the existing output_support wire shape; shoehorning reach into SupportCone). output_support documents stay byte-identical (reach_results omitted). (2) Derivation: INVERT the support relation — enumerate all targets (outputs + "flop:<id>" D-cones), build each via the existing module_support_cones machinery, bucket target T under each X in support(T); dual-consistency (X reaches Y iff Y's support ∋ X) is then free and provable, no boundary-rule re-implementation, no IR/generator change (rejected: a forward consumers BFS). (3) Addressing: target=None ⇒ all sources (inputs decl-order, then flop Qs ascending id, then instance outputs sorted) incl. empty results; Some(input name) / Some("flop:<id>" = the Q's fan-out) / Some("<inst>.<port>"); "flop:<id>" duality documented (same boundary, direction set by query kind); unknown source ⇒ no result ⇒ -32602. (4) Schema: additive MINOR 1.4 → 1.5 (new #[serde(default)] field + query kind), DerivedAnalysisDocument envelope reused unchanged, DUT byte-identical. Pre-split .3b → .3b.1 (pure core) + .3b.2 (surface) per the .2b precedent; the registry entry + dispatch land together in .3b.2 to keep every commit coherent.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.3b`
  Status: `done`
  Goal: `Implement input_reach per the .3a design: the pure analysis (reusing analyze.rs), the "input_reach" query kind, the MCP analyze tool wiring, the schema 1.4 -> 1.5 bump, lib tests for exact reach correctness (dual of the output_support proofs) + determinism + unknown-target, and book/USER_GUIDE/schema-doc/KM closeout. Default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.3b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.3b.2`
  Result: `Done — both children done. .3b.1 the pure core; .3b.2 the surface (registry + dispatch + schema 1.5 + analyze_schema enum + docs/KM). DUT byte-identical.`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.3b.1`
  Status: `done`
  Goal: `The pure input_reach core in src/introspect/analyze.rs: add QUERY_INPUT_REACH = "input_reach", the ReachResult struct, the reach_results: Vec<ReachResult> field on DerivedAnalysis (#[serde(default, skip_serializing_if = "Vec::is_empty")]), and the pure builders module_input_reach(&Module, Option<&str>) / design_input_reach(&Design, Option<&str>) (enumerate all targets = outputs + "flop:<id>" D-cones → build each support cone via the existing machinery → invert: bucket target T under each X in support(T) → resolve the requested source). Do NOT add input_reach to supported_query_kinds() yet (that registry entry + run_analyze dispatch land together in .3b.2 to keep the intermediate commit coherent). Lib-tested only; not wired to any emit path.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. exact reach proofs (the transpose of the support-cone proofs: X reaches Y iff Y's support ∋ X) + flop-Q-as-source reach + design instance-output-as-source reach + target=None ⇒ one ReachResult per source incl. an empty one + determinism/sorted + unknown-source ⇒ no result; cargo test --test snapshots 6/6 byte-identical (analyze.rs is not in any output path; reach_results omitted from output_support docs ⇒ DUT byte-identical).`
  Result: `Done. src/introspect/analyze.rs gains QUERY_INPUT_REACH, ReachResult { target, reaches_outputs[], reaches_flops[], fanout_targets }, the second DerivedAnalysis.reach_results field (#[serde(default, skip_serializing_if = "Vec::is_empty")] ⇒ output_support docs byte-identical), and module_input_reach/design_input_reach with the internal input_reach_with/cone_support_keys/source_universe/make_reach_result helpers (invert the support relation; "flop:<id>" source = the Q's fan-out; source universe = inputs decl-order + flop Qs ascending + instance outputs sorted; control ports show empty reach). supported_query_kinds() unchanged (input_reach joins with dispatch in .3b.2). 7 new in-crate reach proofs (transpose of the cone proofs; flop-Q + flop-D-side duals; design instance-output source; None-all-sources incl. empty clk/rst_n; unknown-source ⇒ none; determinism/sorted; output_support omits reach_results). Validation: cargo test --lib 441/0/2 (15 analyze proofs); cargo test --test snapshots 6/6 byte-identical; cargo clippy --all-targets -D warnings clean; cargo fmt --all --check clean. DUT byte-identical (no IR/generator change, not wired to any emit path).`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.3b.2`
  Status: `done`
  Goal: `Wire input_reach to the surface: add "input_reach" to analyze::supported_query_kinds() AND branch run_analyze by query kind (support builders vs reach builders) in the same commit, updating the empty-result → -32602 guard to check the vec the query populates; bump SCHEMA_VERSION 1.4 -> 1.5 (+ the "1.4" test-assertion updates); add "input_reach" to the analyze_schema enum + refresh the tool description; schema-doc §6.7 + a 1.4 -> 1.5 changelog entry + the input_reach row; book(agent-mcp) input_reach row + worked example; USER_GUIDE tool enum + 1.4 -> 1.5; a KM fact. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the input_reach relation (cached), unknown source ⇒ -32602; schema_version = 1.5 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. analyze.rs: input_reach added to supported_query_kinds(); src/mcp/mod.rs run_analyze branches by query kind (module/design_input_reach vs the support builders) and the unknown-target → -32602 guard checks the query's vec; analyze_schema enum gains "input_reach" + the tool/instructions descriptions updated; SCHEMA_VERSION 1.4 → 1.5 in src/introspect/mod.rs + the doc comment; 6 "1.4" → "1.5" test assertions (2 introspect, 4 mcp); the stale MCP introspect "schema 1.0" description made version-agnostic. Docs: schema-doc §6.7 (results vs reach_results split + ReachResult) + the 1.4 → 1.5 changelog + the "defines 1.5"/checklist; book agent-mcp (analyze row + input_reach worked example + both JSON examples 1.4 → 1.5); USER_GUIDE (analyze description + --introspect schema 1.5); README (schema 1.5 in two spots + the analyze sentence); new KM card semantic-introspection-input-reach (+ cross-link from semantic-introspection-analyze-tool; KNOWLEDGE_MAP regenerated). Validation: cargo test --lib 443/0/2 (incl. 2 new mcp input_reach proofs); cargo test --test snapshots 6/6 byte-identical; clippy -D warnings + fmt clean; mdbook build clean; cargo test --test book_examples 3/3; KM + mem-arch self-checks clean; anvil-mcp stdio e2e smoke (schema 1.5, 37 reach results, unknown source → -32602). DUT byte-identical.`
  Verification: `done`
  Commit: `done`

## Current Frontier

**No active frontier.** Two derived queries are delivered end-to-end —
`output_support` (`.1`/`.2`) and `input_reach` (`.3`, all of `.3a`/`.3b.1`/`.3b.2`
done), at introspection schema `1.5`, DUT byte-identical. The tree stays `active`
(nothing retired): the remaining query kinds (`flop_reset_provenance`,
`module_reachability`) are open-ended `.4+` breadth — not yet registered, not a
blocker. The next lane is owner-directed.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` | `done` | Surface: added `input_reach` to `supported_query_kinds()` + branched `run_analyze` by kind (same commit) + schema `1.4 → 1.5` + the `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. 2 new MCP `input_reach` proofs; `cargo test --lib` 443/0/2; snapshots 6/6; book_examples 3/3; e2e `anvil-mcp` smoke clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` | `done` | The pure `input_reach` core in `analyze.rs`: `QUERY_INPUT_REACH` + `ReachResult` + the second `reach_results` vec + `module_input_reach`/`design_input_reach` (invert the support relation). 7 reach proofs; snapshots 6/6; clippy/fmt clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3a` | `done` | Design-detail (no source) for `input_reach`: pinned the result shape (a second `reach_results: Vec<ReachResult>` vec — `output_support` stays byte-identical), the derivation (invert the support relation, reusing `module_support_cones` ⇒ dual-consistency free + no IR change), `target`/source addressing (incl. the `"flop:<id>"` direction-by-query duality), and the schema bump `1.4 → 1.5`. Pre-split `.3b` → `.3b.1`/`.3b.2`. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3` | `active` | Container: the second derived query `input_reach` (the dual fan-out of `output_support`). |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | `done` | Wired the `.2b.1` analysis to the surface: schema `1.2 → 1.3` + the `DerivedAnalysisDocument` + the pure MCP `analyze` tool (dispatch + `tools/list` + the `anvil://artifact/<run_id>/analysis/<query>` resource, unknown query/target → `-32602`) + book(`agent-mcp`)/USER_GUIDE/schema-doc + a KM fact. DUT byte-identical (snapshots 6/6). |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `done` | Landed the pure derived-relation analysis core (`src/introspect/analyze.rs`: the `DerivedAnalysis`/`SupportCone` types + `module_support_cones`/`design_support_cones`), a memoized combinational fan-in DFS over the existing IR graph; lib-tested for exact cone correctness + determinism + the flop/instance/mem-fsm boundaries + unknown-target. No IR/generator change → DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `done` | Resolved decision `0011`'s three open questions (the `DerivedAnalysis`/`SupportCone` shape; `query`-kind enum + `target` addressing; cone-stops-at-instance-boundary + default-introspect-stays-lean). Split `.2` → `.2a`/`.2b`. No source change. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.1` | `done` | Landed decision `0011` — the first-class MCP-queryable SCHEMA-DERIVED derived-relation API + the no-shadow-simulator boundary + the first query (output support cone) + rejected alternatives. Split `.1`/`.2`/future. No source change. |

## Decisions

- `2026-06-16` (owner steering, audience): **the introspection / MCP query API is
  for AI agents, not human consumption.** Agents can ingest and act on a lot
  of structured data very fast, so the API should optimize for **machine-friendly
  completeness, structured/queryable shape, batch breadth, and speed** — not
  human-readable minimalism or terse summaries. Design implication for every query
  kind (incl. `input_reach`, `.3`): prefer returning the full structured relation
  (all targets / complete reach sets / explicit ids) over abridged human digests;
  keep results JSON-structured and deterministic; lean into "ask one query, get the
  complete machine-actionable answer" rather than paginating for human eyes. This
  does **not** relax the SCHEMA-DERIVED / no-shadow-simulator ceiling — it is about
  *shape and completeness for the agent consumer*, still pure relations over the
  emitted IR. (Big results still spill to `ResourceRef` per `0011` to avoid
  unbounded inline payloads — a transport choice, not a completeness cut.)
- `2026-06-16` (owner steering, lane order): after the cross-module sequential
  equivalence sub-tree (`IDENTITY-DEEPENING.3b.2b`) closed, the owner directed PNT
  into this lane's next derived query, **`input_reach`** (`.3`).
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
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` | Surface wiring: `input_reach` in `analyze::supported_query_kinds()` + the `run_analyze` query-kind dispatch + the vec-aware `-32602` guard (`src/mcp/mod.rs`); `analyze_schema` enum + tool/instructions descriptions; `SCHEMA_VERSION` `1.4 → 1.5` + doc comment (`src/introspect/mod.rs`); 6 `"1.4" → "1.5"` test assertions (2 `introspect`, 4 `mcp`); the stale MCP `introspect` "schema 1.0" description made version-agnostic. Docs: schema-doc §6.7 + `1.4 → 1.5` changelog + "defines 1.5"/checklist; book `agent-mcp` (analyze row + `input_reach` worked example + both JSON examples `1.4 → 1.5`); USER_GUIDE + README; new KM card `semantic-introspection-input-reach` + cross-link; `CODEBASE_ANALYSIS` (both analyze blocks); `ROADMAP` lane status. `cargo test --lib` **443 passed / 0 failed / 2 ignored** (incl. `mcp::tests::analyze_returns_input_reach_relation_and_caches_it` + `analyze_input_reach_unknown_source_is_invalid_params`). `cargo test --test snapshots` **6/6 byte-identical**. `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `mdbook build book` clean; `cargo test --test book_examples` **3/3**; KM regenerated + `check_knowledge_map.sh` in sync; `check_memory_architecture.sh` clean. End-to-end `anvil-mcp` stdio smoke: `analyze {query:"input_reach", seed:7}` → schema `1.5`, 37 `reach_results`, `results` empty; unknown source → `-32602`. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` | Pure `input_reach` core in `src/introspect/analyze.rs` (`QUERY_INPUT_REACH` + `ReachResult` + the second `DerivedAnalysis.reach_results` field + `module_input_reach`/`design_input_reach` + the `input_reach_with`/`cone_support_keys`/`source_universe`/`make_reach_result` helpers; `supported_query_kinds()` unchanged). `cargo test --lib` **441 passed / 0 failed / 2 ignored** (15 `introspect::analyze` proofs incl. 7 new: transpose-of-support; flop-Q + flop-D-side duals; design instance-output source; `None`-all-sources incl. empty clk/rst_n; unknown-source ⇒ none; determinism/sorted; `output_support` omits `reach_results`). `cargo test --test snapshots` **6/6 byte-identical** (DUT `.sv` unchanged; `reach_results` omitted from `output_support` docs). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean. CODEBASE_ANALYSIS `analyze.rs` block amended. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.3a` | Design-detail leaf, **no source change** (grounded in a fresh read of `src/introspect/analyze.rs` — the `DerivedAnalysis`/`SupportCone` types, `module_support_cones`/`design_support_cones`, the `visit` fan-in DFS, `resolve_target`; `src/introspect/mod.rs` — `DerivedAnalysisDocument`/`derived_analysis_document`/`SCHEMA_VERSION`; `src/mcp/mod.rs` — `run_analyze` dispatch + `analyze_schema` enum + the `-32602` guard). `DEVELOPMENT_NOTES.md` design-detail entry (the four points + the `.3b` pre-split). `bash scripts/check_memory_architecture.sh` clean; `bash knowledge-map/scripts/check_knowledge_map.sh` in sync. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | Schema `1.2→1.3` (`src/introspect/mod.rs` `SCHEMA_VERSION` + the `DerivedAnalysisDocument`/`derived_analysis_document`) + the pure MCP `analyze` tool (`src/mcp/mod.rs` `run_analyze` + `analyze_schema` + `CachedArtifact.analyses` + the analysis resource in `resources_list`/`resources_read` + `tools/list` + `instructions`). `cargo test --lib` **427 passed / 0 failed / 2 ignored** (incl. `introspect::derived_analysis_document_reuses_envelope_and_carries_analysis` + the 5 `mcp::tests::analyze_*` proofs). `cargo test --test snapshots` **6/6 byte-identical** (default introspection-document shape unchanged ⇒ DUT `.sv` untouched). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `mdbook build book` clean; `cargo test --test book_examples` **3/3**; Knowledge Map regenerated + `check_knowledge_map.sh` in sync; `check_memory_architecture.sh` clean. End-to-end `anvil-mcp` stdio smoke: `analyze {seed:7}` → schema `1.3` `output_support` cone, unknown query → `-32602`. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | New pure module `src/introspect/analyze.rs` + `pub mod analyze;` in `src/introspect/mod.rs`. `cargo test --lib` **421 passed / 0 failed / 2 ignored** (incl. 9 new `introspect::analyze` proofs: exact combinational support; flop-boundary leaf not recursed + `"flop:<id>"` target; constant-not-support; opaque mem-read termination; absent-target ⇒ per-output cones; unknown-target ⇒ no cone; design child-instance-output name resolution; determinism + sorted; shared-fan-in counted once). `cargo test --test snapshots` **6/6 byte-identical** (DUT `.sv` unchanged). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; baseline `cargo check --all-targets` clean. `bash scripts/check_memory_architecture.sh` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.2a` | Design-detail leaf, no source change (grounded in a fresh read of `src/introspect/mod.rs` `IntrospectionPayload`/`IntrospectionDocument`/`RequestEcho`/`content_run_id_for_knobs` + `src/mcp/mod.rs` pure-tool dispatch + `CachedArtifact`). `DEVELOPMENT_NOTES.md` design-detail entry + tree split. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.1` | Design/decision leaf, no source change (grounded in a fresh survey of `docs/AGENT_INTROSPECTION_SCHEMA.md`, `src/introspect/mod.rs`, `src/mcp/mod.rs`, `src/metrics.rs`, `src/ir/types.rs`, decisions `0004`/`0005`). Decision `0011` + `INDEX.md` + tree activation/split; `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean; `KNOWLEDGE_MAP.md` regenerated. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-15` | `SEMANTIC-INTROSPECTION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.2 — input_reach MCP surface + schema 1.5` | Registry + `run_analyze` dispatch + schema `1.4 → 1.5` + `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. Closes `.3b`/`.3` — `input_reach` delivered end-to-end. 2 new MCP proofs; DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.1 — pure input_reach analysis core` | `src/introspect/analyze.rs`: `QUERY_INPUT_REACH` + `ReachResult` + the second `reach_results` vec + `module_input_reach`/`design_input_reach` (invert the support relation). `supported_query_kinds()` unchanged (joins with dispatch in `.3b.2`). 7 reach proofs; DUT byte-identical (snapshots 6/6). |
| `SEMANTIC-INTROSPECTION-EXPANSION.3a` | `SEMANTIC-INTROSPECTION-EXPANSION.3a — input_reach impl design-detail` | Design-detail (no source): pinned the `input_reach` result shape (a second `reach_results: Vec<ReachResult>` vec — `output_support` stays byte-identical), the derivation (invert the support relation, reusing `module_support_cones`), the source addressing + `"flop:<id>"` direction-by-query duality, and the schema `1.4 → 1.5` bump. Pre-split `.3b` → `.3b.1`/`.3b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2 — the pure MCP analyze tool + schema 1.3` | Schema `1.2→1.3`; `DerivedAnalysisDocument` + the pure MCP `analyze` tool (DUT-only; unknown query/target → `-32602`; cached + served as `anvil://artifact/<run_id>/analysis/<query>`); schema-doc §6.7 + book + USER_GUIDE + KM fact. Closes `.2b`/`.2` — the first query is delivered end-to-end. DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1 — pure support-cone analysis core` | New `src/introspect/analyze.rs`: `DerivedAnalysis`/`SupportCone` + `module_support_cones`/`design_support_cones` (combinational fan-in DFS over the existing IR; FlopQ a register-boundary leaf, `"flop:<id>"` targets, instance-boundary stop, opaque mem/fsm termination). 9 in-crate proofs; DUT byte-identical (no IR/generator change). Split `.2b` → `.2b.1`/`.2b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `SEMANTIC-INTROSPECTION-EXPANSION.2a — support-cone impl design-detail` | Resolved `0011`'s 3 open questions: the `DerivedAnalysis`/`SupportCone` shape; `output_support` query-kind + name `target`; default-introspect-stays-lean + cone-stops-at-instance-boundary; schema `1.2→1.3`. Split `.2` → `.2a`/`.2b`. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION.1` | `SEMANTIC-INTROSPECTION-EXPANSION.1 — activate lane + derived-query API design` | Decision `0011`: a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation API (`DerivedAnalysis` schema `1.3` + pure MCP `analyze` tool); first query = the output support cone. Activated the lane by owner directive; split `.1`/`.2`/future. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-16`: **`.3b.2` landed — closes `.3b`/`.3`; `input_reach` delivered
  end-to-end** (DUT byte-identical). Surface wiring: `input_reach` added to
  `analyze::supported_query_kinds()` **together with** the `run_analyze`
  query-kind dispatch (`module_input_reach`/`design_input_reach` vs the support
  builders) so the registry and dispatch never disagree; the unknown-target →
  `-32602` guard now checks the result vec the query populates; `analyze_schema`
  `enum` + the tool/`instructions` descriptions gained `input_reach`;
  `SCHEMA_VERSION` `1.4 → 1.5` (+ 6 `"1.4" → "1.5"` test assertions); the stale
  MCP `introspect` "schema 1.0" description made version-agnostic. Docs: schema-doc
  §6.7 (split into `results` vs `reach_results` + `ReachResult`) + the `1.4 → 1.5`
  changelog + "defines 1.5"/checklist; book `agent-mcp` (analyze row + an
  `input_reach` worked example + both JSON examples `1.4 → 1.5`); USER_GUIDE +
  README; a new KM card `semantic-introspection-input-reach` (+ cross-link from
  `semantic-introspection-analyze-tool`, KM regenerated); `CODEBASE_ANALYSIS` (both
  analyze blocks) + `ROADMAP` lane status. `cargo test --lib` 443/0/2 (2 new MCP
  `input_reach` proofs); snapshots 6/6 byte-identical; clippy/fmt clean; mdbook +
  book_examples 3/3; e2e `anvil-mcp` smoke (schema `1.5`, 37 reach results,
  unknown source → `-32602`). The tree stays `active`; no active frontier — the
  remaining kinds (`flop_reset_provenance`, `module_reachability`) are open-ended
  `.4+`, none retired.
- `2026-06-16`: **`.3b.1` landed — the pure `input_reach` core** (DUT
  byte-identical). `src/introspect/analyze.rs` gains `QUERY_INPUT_REACH`, the
  `ReachResult` struct, the **second** `DerivedAnalysis.reach_results` field
  (`#[serde(default, skip_serializing_if = "Vec::is_empty")]` ⇒ `output_support`
  documents stay byte-identical), and the pure `module_input_reach` /
  `design_input_reach` builders (+ the internal `input_reach_with` /
  `cone_support_keys` / `source_universe` / `make_reach_result` helpers) that
  **invert** the support relation: enumerate every target (outputs + `"flop:<id>"`
  D-cones), build each cone via the existing machinery, bucket `T` under each
  `X ∈ support(T)`. `"flop:<id>"` as a source = the Q's fan-out; the source
  universe is inputs (decl-order) + flop Qs (ascending) + instance outputs
  (sorted); control ports show empty reach. `supported_query_kinds()` is
  **unchanged** — `input_reach` joins it together with the `run_analyze` dispatch
  in `.3b.2`, so no intermediate commit mislabels. 7 new reach proofs (the
  transpose of the cone proofs). `cargo test --lib` 441/0/2; snapshots 6/6
  byte-identical; clippy/fmt clean. Frontier advances to `.3b.2` (surface).
- `2026-06-16`: **`.3a` design-detail landed** (no source change): resolved the
  four `input_reach` design points grounded in real code and pre-split `.3b` →
  `.3b.1` (pure core, **new frontier**) + `.3b.2` (surface). (1) Result shape: a
  new `ReachResult` (the dual of `SupportCone`) + a **second parallel vec**
  `reach_results: Vec<ReachResult>` on `DerivedAnalysis` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so `output_support`
  documents stay byte-identical (rejected: a tagged enum that would break the
  existing wire shape; shoehorning reach into `SupportCone`). (2) Derivation:
  **invert the support relation** (enumerate all targets = outputs + `"flop:<id>"`
  D-cones, build each via the existing `module_support_cones` machinery, bucket
  target `T` under each `X ∈ support(T)`) ⇒ dual-consistency is free and provable,
  no boundary-rule re-implementation, no IR/generator change (rejected: a forward
  consumers BFS). (3) Addressing: `None` ⇒ all sources (inputs decl-order, then
  flop Qs, then instance outputs) incl. empty results; `Some(input)` /
  `Some("flop:<id>")` = the Q's fan-out / `Some("<inst>.<port>")`; the
  `"flop:<id>"` direction-by-query duality documented; unknown source ⇒ `-32602`.
  (4) Schema: additive MINOR `1.4 → 1.5`, `DerivedAnalysisDocument` envelope
  reused unchanged, DUT byte-identical. Frontier advances to `.3b.1`.
- `2026-06-16`: **Re-entered `active` with a frontier** — owner directed PNT into
  the next derived query, **`input_reach`** (the dual fan-out of the delivered
  `output_support` cone), after `IDENTITY-DEEPENING.3b.2b` closed. Registered `.3`
  (container) + `.3a` (design-detail, **frontier**) + `.3b` (impl); the `.3a` goal
  is grounded in a fresh read of `src/introspect/analyze.rs`. Also recorded the
  owner's **API-audience** steering (the API targets AI agents, not humans ⇒
  optimize for machine-friendly completeness / structured breadth / speed, within
  the unchanged SCHEMA-DERIVED ceiling). No source change (design registration +
  durable decision capture only); handoff for a fresh session.
- `2026-06-16`: `.2b.2` landed, closing `.2b`/`.2` — the **first query is
  delivered end-to-end**. Schema `1.2 → 1.3` (`SCHEMA_VERSION` + 6 `"1.2"`
  test-assertion bumps); the `DerivedAnalysisDocument` envelope +
  `derived_analysis_document` builder; the pure MCP `analyze` tool (`run_analyze`,
  DUT-only, `query` validated against `analyze::supported_query_kinds()`, unknown
  query/target ⇒ `-32602`, cached in `CachedArtifact.analyses` and served as
  `anvil://artifact/<run_id>/analysis/<query>`, registered in `tools/list` +
  `instructions`). Docs: schema-doc §6.7 + `1.3` changelog, book `agent-mcp`
  (`analyze` row + worked example + the stale `1.0` example fixed), USER_GUIDE MCP
  tool/resource lists + `1.2→1.3`, KM fact `semantic-introspection-analyze-tool`.
  DUT byte-identical (snapshots 6/6). No active frontier; future query kinds are
  open-ended `.3+`.
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
