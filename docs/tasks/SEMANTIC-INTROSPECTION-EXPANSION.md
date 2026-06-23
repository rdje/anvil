# SEMANTIC-INTROSPECTION-EXPANSION: behavioral query surface beyond structural projection

## Metadata

- Tree ID: `SEMANTIC-INTROSPECTION-EXPANSION`
- Status: `active` (**four named query kinds from decision `0011` delivered** at schema `1.7`: `output_support` `.1`/`.2` + `input_reach` `.3` + `flop_reset_provenance` `.4` + `module_reachability` `.5`; **a fifth query — `flop_dependencies` (the register→register dependency graph) — is now in progress**: `.6a` design-detail landed (this commit), `.6b.1`/`.6b.2` impl pending, schema `1.17 → 1.18`. The fifth query exercises the lane's documented "further derived-query kinds are open-ended breadth" clause; nothing retired)
- Roadmap lane: `Capability — deeper agent/introspection surface (extends AGENT-INTROSPECTION-MCP / AGENT-MCP-EXPANSION)`
- Created: `2026-06-15`
- Last updated: `2026-06-23` (**`.6a` landed — design-detail (no source) for the fifth derived query, `flop_dependencies`**: the register-to-register (flop→flop) dependency graph — per flop its direct register predecessors (`depends_on_flops` = its D-cone `support_flops`), direct successors (`driven_flops` = its Q's `input_reach` `reaches_flops`), and `self_dependent` (self-feedback). A `DEVELOPMENT_NOTES.md` design-detail entry resolves the five points (result shape = a FIFTH `skip_serializing_if`-omitted parallel `flop_dependencies` vec; derivation = reuse the support/reach machinery in one inversion pass; `"flop:<id>"` addressing; module-vs-design = top-module like `flop_reset_provenance`; schema `1.17 → 1.18`) grounded in a fresh read of `src/introspect/analyze.rs`; pre-split `.6b` → `.6b.1` (pure core) + `.6b.2` (surface). No new numbered decision (the `.3a`/`.4a`/`.5a` per-query precedent; decision `0011` governs the surface). Docs/design only — no source ⇒ DUT byte-identical. Prior: `.5b.2` closed `.5`/`.5b` at schema `1.7`.)
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
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.1`, `SEMANTIC-INTROSPECTION-EXPANSION.2`, `SEMANTIC-INTROSPECTION-EXPANSION.3`, `SEMANTIC-INTROSPECTION-EXPANSION.4`, `SEMANTIC-INTROSPECTION-EXPANSION.5`, `SEMANTIC-INTROSPECTION-EXPANSION.6`

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

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.4`
  Status: `done`
  Goal: `The third derived query — flop_reset_provenance: per-flop reset/data provenance (is each flop reset-defined vs data-driven, and how is its next state built — reset_kind/reset_value, ZeroDefault-vs-QFeedback default behavior, mux kind/arms, has_d). A pure projection of Module.flops (no graph walk), same SCHEMA-DERIVED / default-off / DUT-byte-identical contract; a new "flop_reset_provenance" query kind in the analyze registry.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.4a`, `SEMANTIC-INTROSPECTION-EXPANSION.4b`
  Result: `Done — all children done. flop_reset_provenance is delivered end-to-end: .4a design + .4b.1 pure core + .4b.2 surface (registry + run_analyze dispatch + schema 1.5 → 1.6 + analyze_schema enum + schema-doc §6.7/changelog + book/USER_GUIDE/README + KM card). The MCP analyze tool answers query=flop_reset_provenance with a FlopProvenance per flop; output_support/input_reach stay byte-identical (flop_provenance omitted); unknown "flop:<id>" ⇒ -32602; flopless ⇒ empty. E2e anvil-mcp smoke: seed 3 → schema 1.6, 31 flops; unknown flop:99999 → -32602. DUT byte-identical (snapshots 6/6).`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.4a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground flop_reset_provenance in the real Flop type + analyze.rs/mod.rs/mcp.rs. Pin the result shape (a third parallel flop_provenance vec + a FlopProvenance struct), the derivation (a direct projection of Module.flops), target addressing ("flop:<id>", None ⇒ all flops, unknown ⇒ -32602), and the schema bump (1.5 → 1.6). DEVELOPMENT_NOTES design-detail entry + the .4b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the points grounded in the real Flop type; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES entry grounded in Flop { reset_kind: ResetKind{None|Sync|Async}, reset_val: u128, kind: FlopKind{ZeroDefault|QFeedback}, mux: FlopMux{None|OneHot|Encoded} }. (1) Result shape: a THIRD parallel vec flop_provenance: Vec<FlopProvenance> on DerivedAnalysis (#[serde(default, skip_serializing_if = "Vec::is_empty")] ⇒ output_support/input_reach byte-identical); FlopProvenance { flop, width, has_reset, reset_kind (string), reset_value (DECIMAL STRING — u128-safe), default_behavior ("zero"|"hold"), mux_kind ("none"|"one_hot"|"encoded"), mux_arms, has_d } — enums mapped to strings for wire stability. (2) Derivation: a direct projection of Module.flops (no graph walk — the purest query yet), ascending-id order, pure, no IR/generator change. (3) Addressing: None ⇒ all flops; Some("flop:<id>") ⇒ one; unknown ⇒ -32602; flopless module + None ⇒ empty flop_provenance. (4) Schema: additive MINOR 1.5 → 1.6, envelope reused. Pre-split .4b → .4b.1 (pure core) + .4b.2 (surface); registry entry + dispatch land together in .4b.2.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.4b`
  Status: `done`
  Goal: `Implement flop_reset_provenance per the .4a design: the pure projection + types, the "flop_reset_provenance" query kind, the MCP analyze wiring, the schema 1.5 -> 1.6 bump, lib proofs (each ResetKind/FlopKind/FlopMux variant; None ⇒ all flops; flopless ⇒ empty; unknown ⇒ -32602; determinism), and book/USER_GUIDE/schema-doc/README/KM closeout. Default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.4b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.4b.2`
  Result: `Done — both children done. .4b.1 the pure core; .4b.2 the surface (registry + dispatch + schema 1.6 + analyze_schema enum + docs/KM). DUT byte-identical.`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.4b.1`
  Status: `done`
  Goal: `The pure flop_reset_provenance core in src/introspect/analyze.rs: QUERY_FLOP_RESET_PROVENANCE = "flop_reset_provenance", the FlopProvenance struct, the flop_provenance: Vec<FlopProvenance> field on DerivedAnalysis (#[serde(default, skip_serializing_if)]), and module_flop_provenance(&Module, Option<&str>) / design_flop_provenance(&Design, Option<&str>) — a direct projection of Module.flops (ascending id), enums → strings, reset_value a decimal string. Do NOT add to supported_query_kinds() yet (registry + run_analyze dispatch land together in .4b.2). Lib-tested only.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. each ResetKind/FlopKind/FlopMux variant mapped correctly + reset_value string + None ⇒ all flops ascending + flopless ⇒ empty + "flop:<id>" target + unknown ⇒ none + determinism; cargo test --test snapshots 6/6 byte-identical.`
  Result: `Done. src/introspect/analyze.rs gains QUERY_FLOP_RESET_PROVENANCE, FlopProvenance { flop, width, has_reset, reset_kind, reset_value (decimal string), default_behavior, mux_kind, mux_arms, has_d }, the third DerivedAnalysis.flop_provenance field (#[serde(default, skip_serializing_if)] ⇒ output_support/input_reach byte-identical), module_flop_provenance/design_flop_provenance + the flop_provenance_with/flop_provenance_of helpers (project m.flops ascending id; ResetKind→none/sync/async, FlopKind→zero/hold, FlopMux→none/one_hot/encoded; reset_val.to_string()). The 4 existing DerivedAnalysis literals gained flop_provenance: Vec::new(). supported_query_kinds() unchanged (joins with dispatch in .4b.2). 5 new in-crate proofs (each variant; "flop:<id>" + unknown target; flopless ⇒ empty; serialization omits the other vecs; design top-module variant). Validation: cargo test --lib 448/0/2 (20 analyze proofs); cargo test --test snapshots 6/6 byte-identical; clippy -D warnings + fmt clean. DUT byte-identical.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.4b.2`
  Status: `done`
  Goal: `Wire flop_reset_provenance to the surface: add the kind to supported_query_kinds() AND branch run_analyze by query kind (the empty-result -> -32602 guard checks flop_provenance for this kind) in the same commit; bump SCHEMA_VERSION 1.5 -> 1.6 (+ the "1.5" test-assertion updates); add the kind to the analyze_schema enum + refresh the tool/instructions descriptions; schema-doc §6.7 + a 1.5 -> 1.6 changelog + the row; book(agent-mcp) row + worked example; USER_GUIDE + README; a KM card. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the flop_reset_provenance relation (cached), unknown target ⇒ -32602; schema_version = 1.6 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. analyze.rs: flop_reset_provenance added to supported_query_kinds(); src/mcp/mod.rs run_analyze branches by query kind (module/design_flop_provenance) and the unknown-target → -32602 guard checks flop_provenance; analyze_schema enum gains the kind + the tool/instructions descriptions updated; SCHEMA_VERSION 1.5 → 1.6 + the doc comment; 6 "1.5" → "1.6" test assertions (2 introspect, 4 mcp). Docs: schema-doc §6.7 (third flop_provenance payload + FlopProvenance) + the 1.5 → 1.6 changelog + "defines 1.6"/checklist; book agent-mcp (analyze row + flop_reset_provenance worked example + the three JSON examples 1.5 → 1.6); USER_GUIDE (analyze description + --introspect schema 1.6); README (schema 1.6 in two spots + the analyze sentence); new KM card semantic-introspection-flop-reset-provenance (+ cross-link from semantic-introspection-analyze-tool; KNOWLEDGE_MAP regenerated). Validation: cargo test --lib 450/0/2 (incl. 2 new mcp flop_reset_provenance proofs); cargo test --test snapshots 6/6 byte-identical; clippy -D warnings + fmt clean; mdbook build clean; cargo test --test book_examples 3/3; KM + mem-arch self-checks clean; anvil-mcp stdio e2e smoke (seed 3 → schema 1.6, 31 flops; unknown flop:99999 → -32602). DUT byte-identical.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.5`
  Status: `done`
  Goal: `The fourth derived query — module_reachability: which modules in a Design are reachable from design.top via the instance graph (Module.instances[].module edges), and how each module sits in that graph (reachable, min depth from top, the distinct child module names it instantiates, its instance count). The last named query kind in decision 0011. Same SCHEMA-DERIVED / pure-post-hoc / default-off / DUT-byte-identical contract; a new "module_reachability" query kind in the analyze registry. A pure projection of Design.modules + the instance edges — relations over the construction graph, never behaviour.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.5a`, `SEMANTIC-INTROSPECTION-EXPANSION.5b`
  Result: `Done — all children done. module_reachability is delivered end-to-end: .5a design + .5b.1 pure core + .5b.2 surface (registry + run_analyze dispatch + schema 1.6 → 1.7 + analyze_schema enum + schema-doc §6.7/changelog + book/USER_GUIDE/README + KM card). The MCP analyze tool answers query=module_reachability with one ModuleReachability per module (reachable/depth/instantiates/instance_count); output_support/input_reach/flop_reset_provenance stay byte-identical (module_reachability omitted); unknown module name ⇒ -32602. E2e anvil-mcp smoke: hierarchy design seed 42 → schema 1.7, 3 modules (top mod_42_0002 depth 0 instantiating both leaves, all reachable); unknown module ⇒ -32602. DUT byte-identical (snapshots 6/6). All four named query kinds from decision 0011 now delivered.`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.5a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground module_reachability in the real Design { top, modules } / Module { instances } / Instance { module } IR + analyze.rs/mod.rs/mcp.rs. Pin (1) the result shape (a FOURTH parallel module_reachability: Vec<ModuleReachability> vec on DerivedAnalysis — prior documents byte-identical); (2) the derivation (a BFS from design.top over the Module.instances[].module edges of the design's module table — pure, no IR/generator change); (3) target addressing (a MODULE NAME; None ⇒ all modules; unknown ⇒ -32602 — distinct from the prior queries' port-name/"flop:<id>" targets); (4) module-vs-design semantics (a bare Module has no child defs, so the module variant degenerates to a trivial one-node graph rooted at itself); (5) the schema bump (1.6 → 1.7). DEVELOPMENT_NOTES design-detail entry + the .5b impl shape.`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in real code; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry resolves all five points, grounded in a fresh read of the Design/Module/Instance IR (src/ir/types.rs) + analyze.rs/mod.rs/mcp.rs. (1) Result shape: a new ModuleReachability { module, reachable, depth: Option<usize> (present iff reachable, skip_serializing_if None), instantiates: Vec<String> (distinct direct child module names, sorted+deduped), instance_count } + a FOURTH parallel vec module_reachability: Vec<ModuleReachability> on DerivedAnalysis with #[serde(default, skip_serializing_if = "Vec::is_empty")] (the established parallel-vec pattern; prior documents stay byte-identical). (2) Derivation: BFS from design.top over the Module.instances[].module edges of a name→Module index; min-depth, deterministic (sorted output by module name). Pure — no IR field, no generator change (the coverage_gaps/output_support project-don't-recompute precedent). (3) Addressing: target = a module NAME (None ⇒ all modules sorted by name; Some(name) ⇒ that one; unknown ⇒ no entry ⇒ -32602) — deliberately distinct from the port-name / "flop:<id>" targets of the prior three queries, because the natural identifier for this query is the module name. (4) Module-vs-design: design_module_reachability is the real query; module_module_reachability degenerates to one entry for the bare module itself (reachable, depth 0, its own instantiates/instance_count) since a bare Module carries no child defs to traverse — the same "no child defs" boundary the module variant of the other queries hits; a non-hierarchical DUT leaf has no instances ⇒ {module, reachable:true, depth:0, instantiates:[], instance_count:0}. (5) Schema: additive MINOR 1.6 → 1.7, DerivedAnalysisDocument envelope reused unchanged, DUT byte-identical. Pre-split .5b → .5b.1 (pure core) + .5b.2 (surface) per the .3b/.4b precedent; the registry entry + run_analyze dispatch land together in .5b.2 to keep every commit coherent.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.5b`
  Status: `done`
  Goal: `Implement module_reachability per the .5a design: the pure analysis (in analyze.rs), the "module_reachability" query kind, the MCP analyze wiring, the schema 1.6 -> 1.7 bump, lib proofs (BFS reachability + min depth + instantiates/instance_count + unreachable modules + module-name target + None ⇒ all modules sorted + unknown ⇒ -32602 + determinism + the bare-module degenerate case), and book/USER_GUIDE/schema-doc/README/KM closeout. Default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.5b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.5b.2`
  Result: `Done — both children done. .5b.1 the pure core; .5b.2 the surface (registry + dispatch + schema 1.7 + analyze_schema enum + docs/KM + e2e smoke). DUT byte-identical.`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.5b.1`
  Status: `done`
  Goal: `The pure module_reachability core in src/introspect/analyze.rs: QUERY_MODULE_REACHABILITY = "module_reachability", the ModuleReachability struct, the module_reachability: Vec<ModuleReachability> field on DerivedAnalysis (#[serde(default, skip_serializing_if = "Vec::is_empty")]), and the pure builders design_module_reachability(&Design, Option<&str>) (BFS from design.top over the Module.instances[].module edges of a name→Module index; min-depth; one entry per module sorted by name) / module_module_reachability(&Module, Option<&str>) (the bare-module degenerate one-node case). The 6 existing DerivedAnalysis literals gain module_reachability: Vec::new(). Do NOT add to supported_query_kinds() yet (registry + run_analyze dispatch land together in .5b.2). Lib-tested only; not wired to any emit path.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. BFS reachability over a multi-level design + min depth + instantiates (sorted/deduped) + instance_count + an unreachable module (no depth) + module-name target + None ⇒ all modules sorted + unknown name ⇒ none + the bare-module degenerate single entry + serialization omits the other query vecs + determinism; cargo test --test snapshots 6/6 byte-identical (analyze.rs is not in any output path; module_reachability omitted from prior documents ⇒ DUT byte-identical).`
  Result: `Done. src/introspect/analyze.rs gains QUERY_MODULE_REACHABILITY, ModuleReachability { module, reachable, depth: Option<usize> (#[serde(skip_serializing_if = "Option::is_none")] ⇒ present iff reachable), instantiates: Vec<String> (distinct child module names, sorted/deduped — both InstanceRole kinds count), instance_count }, the FOURTH DerivedAnalysis.module_reachability field (#[serde(default, skip_serializing_if = "Vec::is_empty")] ⇒ output_support/input_reach/flop_reset_provenance documents byte-identical), and design_module_reachability/module_module_reachability + the internal reachability_of/distinct_instantiated helpers. design_module_reachability: a min-depth BFS from design.top over a name→Module index of the Module.instances[].module edges (children visited in sorted order; one entry per module sorted by name; absent-top ⇒ every present module reachable:false — the honest whole-table enumeration, a documented divergence from the other design_* builders' top-absent early-return). module_module_reachability: the degenerate one-node case (a bare module = reachable depth 0, its own instantiates/instance_count). The 6 existing DerivedAnalysis literals gained module_reachability: Vec::new(). supported_query_kinds() unchanged (module_reachability joins it with the run_analyze dispatch in .5b.2). 6 new in-crate proofs (BFS depth/edges/multi-instance/unreachable/sorted; module-name target + unknown; bare-module degenerate; serialization omits the other 3 vecs; determinism; absent-top all-unreachable). Validation: cargo test --lib 456/0/2 (32 introspect::analyze proofs); cargo test --test snapshots 6/6 byte-identical; cargo clippy --all-targets -D warnings clean; cargo fmt --all --check clean. DUT byte-identical (no IR/generator change, not wired to any emit path). CODEBASE_ANALYSIS analyze.rs block amended.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.5b.2`
  Status: `pending`
  Goal: `Wire module_reachability to the surface: add "module_reachability" to analyze::supported_query_kinds() AND branch run_analyze by query kind (design_module_reachability vs module_module_reachability) in the same commit, updating the empty-result → -32602 guard to check module_reachability for this kind; bump SCHEMA_VERSION 1.6 -> 1.7 (+ the "1.6" test-assertion updates); add "module_reachability" to the analyze_schema enum + refresh the tool/instructions descriptions; schema-doc §6.7 + a 1.6 -> 1.7 changelog entry + the row; book(agent-mcp) module_reachability row + worked example; USER_GUIDE + README; a KM card. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the module_reachability relation (cached), unknown module name ⇒ -32602; schema_version = 1.7 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; snapshots 6/6 byte-identical; committed through COMMIT.md with the leaf id.`
  Result: `Done. analyze.rs: module_reachability added to supported_query_kinds(); src/mcp/mod.rs run_analyze branches by query kind (module/design_module_reachability) in both the design and module paths and the unknown-target → -32602 guard checks module_reachability; analyze_schema query enum + target description + the analyze tool description + the server instructions cover the fourth kind; SCHEMA_VERSION 1.6 → 1.7 in src/introspect/mod.rs + doc comment; 8 "1.6" → "1.7" test assertions (2 introspect, 6 mcp). Docs: schema-doc §6.7 (fourth module_reachability payload + ModuleReachability + the stale "two parallel vecs" intro fixed to four) + the 1.6 → 1.7 changelog + "defines 1.7"/checklist; book agent-mcp (analyze row + a module_reachability worked example + the four JSON examples 1.6 → 1.7 + the resource line); USER_GUIDE (analyze description + --introspect schema 1.7); README (schema 1.7 in two spots + the analyze sentence); new KM card semantic-introspection-module-reachability (+ cross-link from semantic-introspection-analyze-tool; KNOWLEDGE_MAP regenerated, 35 facts); CODEBASE_ANALYSIS (analyze + mcp blocks). Validation: cargo test --lib 458/0/2 (incl. 2 new mcp module_reachability proofs); cargo test --test snapshots 6/6 byte-identical; clippy -D warnings + fmt clean; mdbook build clean; cargo test --test book_examples 3/3; KM + mem-arch self-checks clean; e2e anvil-mcp stdio smoke (hierarchy seed 42 → schema 1.7, 3 modules all reachable, top depth 0; unknown module ⇒ -32602). DUT byte-identical.`
  Verification: `done`
  Commit: `done`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.6`
  Status: `active`
  Goal: `The fifth derived query — flop_dependencies: the register-to-register (flop→flop) dependency graph of a module. Per flop: depends_on_flops (direct register predecessors = its D-cone support_flops), driven_flops (direct register successors = its Q's input_reach reaches_flops), and self_dependent (a self-feedback register — counter/accumulator). The register-level analog of module_reachability, reusing the existing gate-graph support/reach machinery. The first query beyond decision 0011's four named kinds, under the lane's "open-ended breadth" clause; same SCHEMA-DERIVED / pure-post-hoc / default-off / DUT-byte-identical contract; a new "flop_dependencies" query kind in the analyze registry.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.6a`, `SEMANTIC-INTROSPECTION-EXPANSION.6b`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.6a`
  Status: `done`
  Goal: `Design-detail leaf (no source): ground flop_dependencies in the real src/introspect/analyze.rs (the DerivedAnalysis/SupportCone/ReachResult shapes + module_support_cones/module_input_reach builders + the "flop:<id>" target convention). Pin the result shape (a FIFTH parallel flop_dependencies: Vec<FlopDependencies> vec + the FlopDependencies struct), the derivation (reuse the support/reach machinery in one inversion pass — depends_on_flops = D-cone support_flops, driven_flops = the transpose, self_dependent = flop ∈ depends_on_flops), "flop:<id>" target addressing, module-vs-design (top-module like flop_reset_provenance), and the schema bump (1.17 → 1.18). DEVELOPMENT_NOTES design-detail entry + the .6b impl shape. No new numbered decision (the .3a/.4a/.5a per-query precedent; decision 0011 governs the surface).`
  Acceptance: `A DEVELOPMENT_NOTES design-detail entry resolving the five points grounded in the real analyze.rs; tree split recorded; no source change; docs/workflow self-checks clean.`
  Result: `Done. DEVELOPMENT_NOTES design-detail entry "flop_dependencies impl design-detail — .6a" resolves all five points, grounded in a fresh read of analyze.rs. (1) Result shape: a new FlopDependencies { flop: u32, depends_on_flops: Vec<u32>, driven_flops: Vec<u32>, self_dependent: bool } + a FIFTH parallel vec flop_dependencies: Vec<FlopDependencies> on DerivedAnalysis (#[serde(default, skip_serializing_if = "Vec::is_empty")] ⇒ the four prior documents byte-identical); both edge vecs present for every flop (sorted/deduped); no in/out-degree count fields (= vec len, a second source of truth); self_dependent is the one genuinely-extra derived boolean (the self-feedback marker). Deferred sub-kinds: transitive register-reachability closure, SCC/feedback-loop grouping, sequential-depth metric (nothing retired). (2) Derivation: reuse the support/reach machinery — depends_on_flops = the flop's D-cone support_flops; driven_flops = the transpose (one inversion pass, exactly input_reach_with restricted to flops); self_dependent = flop ∈ depends_on_flops; pure, no IR/generator change; dual-consistency a free test invariant; opaque MemRead/FsmOut terminate the cone for free. (3) Addressing: "flop:<id>" (consistent with output_support/input_reach/flop_reset_provenance); None ⇒ all flops ascending id; unknown/out-of-range ⇒ no entry ⇒ -32602; flopless + None ⇒ empty. (4) Module-vs-design: module_flop_dependencies + design_flop_dependencies on the top module (early-return empty when top absent), per-child-module a future extension (the flop_reset_provenance convention); no fmt closure needed (reports flop ids only). (5) Schema: additive MINOR 1.17 → 1.18, envelope reused, DUT byte-identical. Pre-split .6b → .6b.1 (pure core + lib proofs; NOT in supported_query_kinds yet) + .6b.2 (surface: registry + run_analyze dispatch in one commit + schema 1.17 → 1.18 + analyze_schema enum + schema-doc/book/USER_GUIDE/README/KM + e2e smoke). Docs/design only — no src/ ⇒ DUT byte-identical; self-checks green.`
  Verification: `bash scripts/check_doctrines.sh green (docs/design commit ⇒ code-scoped checks exempt); no src/ touched ⇒ cargo check/clippy/fmt/test unaffected; DUT byte-identical.`
  Commit: `this SEMANTIC-INTROSPECTION-EXPANSION.6a commit`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.6b`
  Status: `pending`
  Goal: `Implement flop_dependencies per the .6a design: the pure analysis (in analyze.rs), the "flop_dependencies" query kind, the MCP analyze wiring, the schema 1.17 -> 1.18 bump, lib proofs (predecessors/successors via the support/reach machinery + dual-consistency transpose + self_dependent on a counter + "flop:<id>" target + None ⇒ all flops ascending + flopless ⇒ empty + unknown ⇒ -32602 + determinism), and book/USER_GUIDE/schema-doc/README/KM closeout. Default-off / DUT byte-identical.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.6b.1`, `SEMANTIC-INTROSPECTION-EXPANSION.6b.2`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.6b.1`
  Status: `pending`
  Goal: `The pure flop_dependencies core in src/introspect/analyze.rs: QUERY_FLOP_DEPENDENCIES = "flop_dependencies", the FlopDependencies struct, the flop_dependencies: Vec<FlopDependencies> field on DerivedAnalysis (#[serde(default, skip_serializing_if = "Vec::is_empty")]), and module_flop_dependencies(&Module, Option<&str>) / design_flop_dependencies(&Design, Option<&str>) (reuse the support/reach cone machinery: depends_on_flops = the flop's D-cone support_flops, driven_flops = the transpose, self_dependent = flop ∈ depends_on_flops). The existing DerivedAnalysis literals gain flop_dependencies: Vec::new(). Do NOT add to supported_query_kinds() yet (registry + run_analyze dispatch land together in .6b.2). Lib-tested only; not wired to any emit path.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib green incl. exact predecessor/successor correctness on a hand-built pipeline + a self-feedback counter (self_dependent) + dual-consistency (M ∈ depends_on(N) ⇔ N ∈ driven(M)) + "flop:<id>" target + None ⇒ all flops ascending + flopless ⇒ empty + unknown ⇒ none + serialization omits the other four query vecs + determinism; cargo test --test snapshots 6/6 byte-identical (analyze.rs is not in any output path).`
  Result: `pending`
  Verification: `pending`
  Commit: `pending`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.6b.2`
  Status: `pending`
  Goal: `Wire flop_dependencies to the surface: add "flop_dependencies" to analyze::supported_query_kinds() AND branch run_analyze by query kind (module/design_flop_dependencies) in the same commit, updating the empty-result → -32602 guard to check flop_dependencies for this kind; bump SCHEMA_VERSION 1.17 -> 1.18 (+ the "1.17" test-assertion updates); add the kind to the analyze_schema enum + refresh the tool/instructions descriptions; schema-doc §6.7 + a 1.17 -> 1.18 changelog + the row; book(agent-mcp) row + worked example + the JSON examples 1.17 -> 1.18; USER_GUIDE + README; a KM card. Default-off / DUT byte-identical.`
  Acceptance: `cargo check/clippy(-D warnings)/fmt clean; cargo test --lib + introspect/mcp tests green; the pure MCP analyze tool returns the flop_dependencies relation (cached), unknown target ⇒ -32602; schema_version = 1.18 everywhere + schema doc updated; book/USER_GUIDE/schema-doc + a KM fact; snapshots 6/6 byte-identical; mdbook build + book_examples clean; an anvil-mcp stdio e2e smoke; committed through COMMIT.md with the leaf id.`
  Result: `pending`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

**Active frontier: `SEMANTIC-INTROSPECTION-EXPANSION.6b.1`** (the pure
`flop_dependencies` core in `analyze.rs`). The four named derived queries from
decision `0011` are delivered end-to-end — `output_support` (`.1`/`.2`),
`input_reach` (`.3`), `flop_reset_provenance` (`.4`), and `module_reachability`
(`.5`) — at introspection schema `1.7`, DUT byte-identical. A **fifth** query,
`flop_dependencies` (the register→register dependency graph — the first beyond
decision `0011`'s four named kinds, under the lane's "open-ended breadth" clause),
is now in progress: `.6a` design-detail landed; `.6b.1` (pure core) is next, then
`.6b.2` (surface, schema `1.17 → 1.18`). Nothing retired.

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEMANTIC-INTROSPECTION-EXPANSION.6b.1` | `pending` | **Next.** The pure `flop_dependencies` core in `analyze.rs`: `QUERY_FLOP_DEPENDENCIES` + `FlopDependencies` + the fifth `flop_dependencies` vec + `module_flop_dependencies`/`design_flop_dependencies` (reuse the support/reach machinery: predecessors = D-cone `support_flops`, successors = the transpose, `self_dependent`). Not in `supported_query_kinds()` yet (joins with dispatch in `.6b.2`). Lib-tested; snapshots 6/6 byte-identical. DUT byte-identical. |
| 2 | `SEMANTIC-INTROSPECTION-EXPANSION.6b.2` | `pending` | Surface: add `flop_dependencies` to `supported_query_kinds()` + branch `run_analyze` (same commit) + schema `1.17 → 1.18` + the `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM + e2e `anvil-mcp` smoke. Closes `.6b`/`.6`. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.6a` | `done` | Design-detail (no source) for `flop_dependencies`: pinned the result shape (a fifth `flop_dependencies: Vec<FlopDependencies>` vec — prior four documents byte-identical), the derivation (reuse the support/reach machinery in one inversion pass — predecessors = D-cone `support_flops`, successors = the transpose, `self_dependent` = self-feedback), `"flop:<id>"` addressing, the top-module module-vs-design convention, and the schema bump `1.17 → 1.18`. No new numbered decision (the `.3a`/`.4a`/`.5a` precedent). Pre-split `.6b` → `.6b.1`/`.6b.2`. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.5b.2` | `done` | Surface: added `module_reachability` to `supported_query_kinds()` + branched `run_analyze` (same commit) + schema `1.6 → 1.7` + the `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. 2 new MCP proofs; `cargo test --lib` 458/0/2; snapshots 6/6; book_examples 3/3; e2e `anvil-mcp` smoke (hierarchy → schema 1.7, 3 modules). Closes `.5b`/`.5`. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.5b.1` | `done` | The pure `module_reachability` core in `analyze.rs`: `QUERY_MODULE_REACHABILITY` + `ModuleReachability` + the fourth `module_reachability` vec + `design_module_reachability`/`module_module_reachability` (BFS over the design's instance edges). Not in `supported_query_kinds()` yet (joins with dispatch in `.5b.2`). 6 proofs; `cargo test --lib` 456/0/2; snapshots 6/6; clippy/fmt clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.5a` | `done` | Design-detail (no source) for `module_reachability`: pinned the result shape (a fourth `module_reachability: Vec<ModuleReachability>` vec — prior documents byte-identical), the derivation (BFS from `design.top` over the `Module.instances[].module` edges), **module-name** target addressing, the module-vs-design degenerate semantics, and the schema bump `1.6 → 1.7`. Pre-split `.5b` → `.5b.1`/`.5b.2`. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.4b.2` | `done` | Surface: added the kind to `supported_query_kinds()` + branched `run_analyze` (same commit) + schema `1.5 → 1.6` + the `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. 2 new MCP proofs; `cargo test --lib` 450/0/2; snapshots 6/6; book_examples 3/3; e2e `anvil-mcp` smoke (31 flops, schema 1.6). DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.4b.1` | `done` | The pure `flop_reset_provenance` core in `analyze.rs`: `QUERY_FLOP_RESET_PROVENANCE` + `FlopProvenance` + the third `flop_provenance` vec + `module_flop_provenance`/`design_flop_provenance` (a direct projection of `Module.flops`). 5 proofs; snapshots 6/6; clippy/fmt clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.4a` | `done` | Design-detail (no source) for `flop_reset_provenance`: pinned the result shape (a third `flop_provenance: Vec<FlopProvenance>` vec — prior documents byte-identical), the derivation (a direct projection of `Module.flops`, no graph walk), `"flop:<id>"` addressing, and the schema bump `1.5 → 1.6`. Pre-split `.4b` → `.4b.1`/`.4b.2`. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` | `done` | Surface: added `input_reach` to `supported_query_kinds()` + branched `run_analyze` by kind (same commit) + schema `1.4 → 1.5` + the `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. 2 new MCP `input_reach` proofs; `cargo test --lib` 443/0/2; snapshots 6/6; book_examples 3/3; e2e `anvil-mcp` smoke clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` | `done` | The pure `input_reach` core in `analyze.rs`: `QUERY_INPUT_REACH` + `ReachResult` + the second `reach_results` vec + `module_input_reach`/`design_input_reach` (invert the support relation). 7 reach proofs; snapshots 6/6; clippy/fmt clean. DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3a` | `done` | Design-detail (no source) for `input_reach`: pinned the result shape (a second `reach_results: Vec<ReachResult>` vec — `output_support` stays byte-identical), the derivation (invert the support relation, reusing `module_support_cones` ⇒ dual-consistency free + no IR change), `target`/source addressing (incl. the `"flop:<id>"` direction-by-query duality), and the schema bump `1.4 → 1.5`. Pre-split `.3b` → `.3b.1`/`.3b.2`. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.3` | `active` | Container: the second derived query `input_reach` (the dual fan-out of `output_support`). |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | `done` | Wired the `.2b.1` analysis to the surface: schema `1.2 → 1.3` + the `DerivedAnalysisDocument` + the pure MCP `analyze` tool (dispatch + `tools/list` + the `anvil://artifact/<run_id>/analysis/<query>` resource, unknown query/target → `-32602`) + book(`agent-mcp`)/USER_GUIDE/schema-doc + a KM fact. DUT byte-identical (snapshots 6/6). |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `done` | Landed the pure derived-relation analysis core (`src/introspect/analyze.rs`: the `DerivedAnalysis`/`SupportCone` types + `module_support_cones`/`design_support_cones`), a memoized combinational fan-in DFS over the existing IR graph; lib-tested for exact cone correctness + determinism + the flop/instance/mem-fsm boundaries + unknown-target. No IR/generator change → DUT byte-identical. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `done` | Resolved decision `0011`'s three open questions (the `DerivedAnalysis`/`SupportCone` shape; `query`-kind enum + `target` addressing; cone-stops-at-instance-boundary + default-introspect-stays-lean). Split `.2` → `.2a`/`.2b`. No source change. |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.1` | `done` | Landed decision `0011` — the first-class MCP-queryable SCHEMA-DERIVED derived-relation API + the no-shadow-simulator boundary + the first query (output support cone) + rejected alternatives. Split `.1`/`.2`/future. No source change. |

## Decisions

- `2026-06-23` (`.6a`, design-detail in `DEVELOPMENT_NOTES.md`): pinned
  `flop_dependencies`, the **fifth** derived query — *the register-to-register
  (flop→flop) dependency graph* — and the **first query beyond decision `0011`'s
  four named kinds**, exercising the lane's documented "further derived-query kinds
  are open-ended breadth" clause (under decision `0011`'s API + the `0004`/`0011`
  SCHEMA-DERIVED ceiling; **no new numbered decision**, the `.3a`/`.4a`/`.5a`
  per-query precedent). The register-level analog of `module_reachability` (a graph
  over a node class), but reusing the existing **gate-graph** support/reach
  machinery rather than the module table. (1) Result shape: a new `FlopDependencies
  { flop: u32, depends_on_flops: Vec<u32>, driven_flops: Vec<u32>, self_dependent:
  bool }` + a **fifth** parallel vec `flop_dependencies: Vec<FlopDependencies>` on
  `DerivedAnalysis` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]` ⇒
  the four prior documents stay byte-identical); both edge vecs present for every
  flop (sorted/deduped), no in/out-degree count fields (= vec `len`, a second source
  of truth), `self_dependent` the one genuinely-extra derived boolean (the
  self-feedback marker, parallel to `module_reachability.reachable`). (2) Derivation:
  reuse the support/reach machinery — `depends_on_flops` = the flop's D-cone
  `support_flops`; `driven_flops` = the transpose (one inversion pass, exactly
  `input_reach_with` restricted to flops); `self_dependent` = `flop ∈
  depends_on_flops`; pure, no IR/generator change; dual-consistency a free, provable
  test invariant; opaque `MemRead`/`FsmOut` terminate the underlying cone for free.
  Honest framing: each edge is individually derivable from `output_support` /
  `input_reach` on a `"flop:<id>"` target, but no single query returns the whole
  register graph — per the `2026-06-16` agent-API steering ("ask one query, get the
  complete machine-actionable answer") this is the register-graph **view**, not new
  computed truth. (3) Addressing: `"flop:<id>"` (consistent with `output_support` /
  `input_reach` / `flop_reset_provenance`); `None` ⇒ all flops ascending id;
  unknown/out-of-range ⇒ no entry ⇒ `-32602`; flopless + `None` ⇒ empty. (4)
  Module-vs-design: `module_flop_dependencies` + `design_flop_dependencies` on the
  **top** module (early-return empty when top absent), per-child-module a future
  extension — the `flop_reset_provenance` "operates on the top module" convention;
  no `format_instance_leaf_*` fmt closure needed (reports flop ids only). (5)
  Schema: additive MINOR `1.17 → 1.18`, `DerivedAnalysisDocument` envelope reused,
  DUT byte-identical. Deferred sub-kinds (nothing retired): a transitive
  register-reachability closure, SCC/feedback-loop grouping, a sequential-depth
  metric. Pre-split `.6b` → `.6b.1` (pure core) + `.6b.2` (surface); the registry
  entry + `run_analyze` dispatch land together in `.6b.2`.
- `2026-06-16` (`.5a`, design-detail in `DEVELOPMENT_NOTES.md`): pinned
  `module_reachability`, the **fourth** derived query (the last named kind in
  decision `0011`). (1) Result shape: a new `ModuleReachability { module,
  reachable, depth: Option<usize>, instantiates: Vec<String>, instance_count }` +
  a **fourth** parallel vec `module_reachability: Vec<ModuleReachability>` on
  `DerivedAnalysis` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]` ⇒
  the three prior documents stay byte-identical) — the established parallel-vec
  pattern (`.3a` rejected a tagged `results` enum precisely so each new kind is one
  more skip-if vec the `query` field discriminates). `depth` is `Option<usize>`
  with `skip_serializing_if = "Option::is_none"`: present (0 = top) iff reachable.
  (2) Derivation: a BFS from `design.top` over the `Module.instances[].module`
  edges of a name→`Module` index — pure, min-depth, deterministic (output sorted by
  module name); no IR field, no generator change (the `coverage_gaps` /
  `output_support` project-don't-recompute precedent). (3) Addressing: `target` = a
  **module name** (`None` ⇒ every module sorted; `Some(name)` ⇒ that one; unknown ⇒
  no entry ⇒ `-32602`) — deliberately distinct from the prior queries' port-name /
  `"flop:<id>"` targets, because the module name is this query's natural
  identifier. (4) Module-vs-design: `design_module_reachability` is the real query;
  `module_module_reachability` degenerates to a single entry for the bare module
  itself (a one-node graph rooted at itself) since a bare `Module` carries no child
  defs to traverse — the same "no child defs" boundary the module variant of the
  other three queries hits. (5) Schema: additive MINOR `1.6 → 1.7`,
  `DerivedAnalysisDocument` envelope reused unchanged, DUT byte-identical.
  Pre-split `.5b` → `.5b.1` (pure core) + `.5b.2` (surface); the registry entry +
  `run_analyze` dispatch land together in `.5b.2` to keep every commit coherent.
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
| `2026-06-23` | `SEMANTIC-INTROSPECTION-EXPANSION.6a` | Design-detail leaf, **no source change** (grounded in a fresh read of `src/introspect/analyze.rs` — the `DerivedAnalysis`/`SupportCone`/`ReachResult` parallel-vec shapes; `module_support_cones`/`design_support_cones` + `module_input_reach`/`design_input_reach` builders + their `support_cones_with`/`input_reach_with` internals; the `"flop:<id>"` target convention; the `design_*` top-module early-return pattern). `DEVELOPMENT_NOTES.md` design-detail entry "`flop_dependencies` impl design-detail — `.6a`" resolving the five points (a FIFTH `flop_dependencies` parallel vec + `FlopDependencies { flop, depends_on_flops, driven_flops, self_dependent }`; reuse the support/reach machinery in one inversion pass; `"flop:<id>"` addressing; top-module module-vs-design; schema `1.17 → 1.18`) + the `.6b` pre-split. Tree `.6`/`.6a`/`.6b` registered (+ root child `.6`) and `.6b` pre-split → `.6b.1`/`.6b.2`; frontier set to `.6b.1`. `bash scripts/check_doctrines.sh` green (docs/design commit ⇒ code-scoped `CODE-CHANGE-EVIDENCE` / `TASK-TREE-OWNERSHIP` exempt; `MEMORY-ARCH` + `KNOWLEDGE-MAP` pass). No `src/` touched ⇒ `cargo check/clippy/fmt/test` unaffected; **DUT byte-identical**. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.5b.2` | Surface wiring: `module_reachability` in `analyze::supported_query_kinds()` + the `run_analyze` query-kind dispatch (`module/design_module_reachability` in both the design and module paths) + the vec-aware `-32602` guard (`src/mcp/mod.rs`); `analyze_schema` `query` enum + `target` description + the `analyze` tool description + the server `instructions`; `SCHEMA_VERSION` `1.6 → 1.7` + doc comment (`src/introspect/mod.rs`); 8 `"1.6" → "1.7"` test assertions (2 introspect, 6 mcp). Docs: schema-doc §6.7 (fourth `module_reachability` payload + `ModuleReachability` + the stale "two parallel vecs" intro corrected to four) + `1.6 → 1.7` changelog + "defines 1.7"/checklist; book `agent-mcp` (analyze row + a `module_reachability` worked example + the four JSON examples `1.6 → 1.7` + the resource line); USER_GUIDE + README; new KM card `semantic-introspection-module-reachability` + cross-link; `CODEBASE_ANALYSIS` (analyze + mcp blocks). `cargo test --lib` **458 passed / 0 failed / 2 ignored** (incl. `mcp::tests::analyze_returns_module_reachability_and_caches_it` + `analyze_module_reachability_unknown_module_is_invalid_params`). `cargo test --test snapshots` **6/6 byte-identical**. `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `mdbook build book` clean; `cargo test --test book_examples` **3/3**; KM regenerated (35 facts / 262 keys) + `check_knowledge_map.sh` in sync; `check_memory_architecture.sh` clean. End-to-end `anvil-mcp` stdio smoke: `analyze {query:"module_reachability", seed:42, hierarchy config}` → schema `1.7`, 3 modules (top `mod_42_0002` depth 0 instantiating both leaves, all reachable); unknown module name → `-32602`. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.5a` | Design-detail leaf, **no source change** (grounded in a fresh read of the `Design { top, modules }` / `Module { instances }` / `Instance { module, role }` IR in `src/ir/types.rs`, plus `src/introspect/analyze.rs` — the `DerivedAnalysis` parallel-vec pattern + `design_*`/`module_*` builder split + `format_instance_leaf_design`; `src/introspect/mod.rs` — `SCHEMA_VERSION`/`DerivedAnalysisDocument`; `src/mcp/mod.rs` — `run_analyze` design-vs-module routing on `effective_hierarchy_depth_range` + the `analyze_schema` enum + the vec-aware `-32602` guard). `DEVELOPMENT_NOTES.md` design-detail entry (the five points + the `.5b` pre-split). Tree `.5`/`.5a`/`.5b` registered (+ root child) and `.5b` pre-split → `.5b.1`/`.5b.2`; frontier set to `.5b.1`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.5b.1` | Pure `module_reachability` core in `src/introspect/analyze.rs` (`QUERY_MODULE_REACHABILITY` + `ModuleReachability { module, reachable, depth: Option<usize>, instantiates, instance_count }` + the fourth `DerivedAnalysis.module_reachability` field + `design_module_reachability` (BFS over the instance graph) / `module_module_reachability` (degenerate one-node) + the `reachability_of`/`distinct_instantiated` helpers; the 6 existing `DerivedAnalysis` literals gained `module_reachability: Vec::new()`; `supported_query_kinds()` unchanged). `cargo test --lib` **456 passed / 0 failed / 2 ignored** (32 `introspect::analyze` proofs incl. 6 new: BFS depth/edges/multi-instance/unreachable/sorted; module-name target + unknown; bare-module degenerate; serialization omits the other 3 vecs; determinism; absent-top all-unreachable). `cargo test --test snapshots` **6/6 byte-identical** (DUT `.sv` unchanged; `module_reachability` omitted from prior documents). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean. `CODEBASE_ANALYSIS.md` `analyze.rs` block amended. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.4b.2` | Surface wiring: `flop_reset_provenance` in `supported_query_kinds()` + the `run_analyze` dispatch (`module/design_flop_provenance`) + the `flop_provenance` `-32602` guard (`src/mcp/mod.rs`); `analyze_schema` enum + tool/instructions text; `SCHEMA_VERSION` `1.5 → 1.6` + doc comment (`src/introspect/mod.rs`); 6 `"1.5" → "1.6"` test assertions (2 introspect, 4 mcp). Docs: schema-doc §6.7 (third payload + `FlopProvenance`) + `1.5 → 1.6` changelog + "defines 1.6"/checklist; book `agent-mcp` (analyze row + `flop_reset_provenance` worked example + the three JSON examples `1.5 → 1.6`); USER_GUIDE + README; new KM card `semantic-introspection-flop-reset-provenance` + cross-link; `CODEBASE_ANALYSIS` (both analyze blocks); `ROADMAP` lane status. `cargo test --lib` **450 passed / 0 failed / 2 ignored** (incl. `mcp::tests::analyze_returns_flop_reset_provenance_and_caches_it` + `analyze_flop_reset_provenance_unknown_target_is_invalid_params`). `cargo test --test snapshots` **6/6 byte-identical**. `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `mdbook build book` clean; `cargo test --test book_examples` **3/3**; KM regenerated + `check_knowledge_map.sh` in sync; `check_memory_architecture.sh` clean. End-to-end `anvil-mcp` stdio smoke: `analyze {query:"flop_reset_provenance", seed:3}` → schema `1.6`, 31 flops (flop 0 async/hold/encoded); unknown `flop:99999` → `-32602`. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.4b.1` | Pure `flop_reset_provenance` core in `src/introspect/analyze.rs` (`QUERY_FLOP_RESET_PROVENANCE` + `FlopProvenance` + the third `DerivedAnalysis.flop_provenance` field + `module_flop_provenance`/`design_flop_provenance` + `flop_provenance_with`/`flop_provenance_of`; the 4 existing `DerivedAnalysis` literals gained `flop_provenance: Vec::new()`; `supported_query_kinds()` unchanged). `cargo test --lib` **448 passed / 0 failed / 2 ignored** (20 `introspect::analyze` proofs incl. 5 new: each `ResetKind`/`FlopKind`/`FlopMux` variant + `reset_value` string + ascending-id ordering; `"flop:<id>"` + unknown target ⇒ none; flopless ⇒ empty; serialization omits the other vecs; design top-module variant). `cargo test --test snapshots` **6/6 byte-identical** (DUT `.sv` unchanged). `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean. CODEBASE_ANALYSIS `analyze.rs` block amended. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. | `done` |
| `2026-06-16` | `SEMANTIC-INTROSPECTION-EXPANSION.4a` | Design-detail leaf, **no source change** (grounded in the real `Flop` type in `src/ir/types.rs` — `ResetKind`/`FlopKind`/`FlopMux`/`reset_val` — plus `src/introspect/analyze.rs`/`mod.rs` + `src/mcp/mod.rs`). `DEVELOPMENT_NOTES.md` design-detail entry (the four points + the `.4b` pre-split: a third `flop_provenance` vec, a direct `Module.flops` projection, `"flop:<id>"` addressing, schema `1.5 → 1.6`). Tree `.4`/`.4a`/`.4b` registered + `.4b` pre-split → `.4b.1`/`.4b.2`. `bash scripts/check_memory_architecture.sh` + `bash knowledge-map/scripts/check_knowledge_map.sh` clean. Baseline `cargo check --all-targets` clean. | `done` |
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
| `SEMANTIC-INTROSPECTION-EXPANSION.6a` | `SEMANTIC-INTROSPECTION-EXPANSION.6a — flop_dependencies impl design-detail` | Design-detail (no source): opened `.6`, the **fifth** derived query `flop_dependencies` (the register→register dependency graph — first beyond decision `0011`'s four named kinds, under the lane's "open-ended breadth" clause). Pinned the result shape (a fifth `flop_dependencies: Vec<FlopDependencies>` vec — prior four documents byte-identical), the derivation (reuse the support/reach machinery in one inversion pass — predecessors = D-cone `support_flops`, successors = the transpose, `self_dependent` = self-feedback), `"flop:<id>"` addressing, the top-module module-vs-design convention, and the schema `1.17 → 1.18` bump. No new numbered decision (the `.3a`/`.4a`/`.5a` precedent). Registered `.6`/`.6a`/`.6b`; pre-split `.6b` → `.6b.1`/`.6b.2`. DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.5b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.5b.2 — module_reachability MCP surface + schema 1.7` | Registry + `run_analyze` dispatch (`module/design_module_reachability`) + schema `1.6 → 1.7` + `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. Closes `.5b`/`.5` — `module_reachability` delivered end-to-end (the **fourth** and last named query kind). 2 new MCP proofs + e2e `anvil-mcp` smoke; DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.5b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.5b.1 — pure module_reachability core` | `src/introspect/analyze.rs`: `QUERY_MODULE_REACHABILITY` + `ModuleReachability` + the fourth `module_reachability` vec + `design_module_reachability`/`module_module_reachability` (BFS over the instance graph + the degenerate one-node case). `supported_query_kinds()` unchanged (joins with dispatch in `.5b.2`). 6 proofs; `cargo test --lib` 456/0/2; snapshots 6/6 byte-identical. DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.5a` | `SEMANTIC-INTROSPECTION-EXPANSION.5a — module_reachability impl design-detail` | Design-detail (no source): opened `.5`, the **fourth** derived query `module_reachability`. Pinned the result shape (a fourth `module_reachability: Vec<ModuleReachability>` vec — prior documents byte-identical), the derivation (BFS from `design.top` over the `Module.instances[].module` edges), **module-name** target addressing, the module-vs-design degenerate semantics, and the schema `1.6 → 1.7` bump. Registered `.5`/`.5a`/`.5b`; pre-split `.5b` → `.5b.1`/`.5b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.4b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.4b.2 — flop_reset_provenance MCP surface + schema 1.6` | Registry + `run_analyze` dispatch + schema `1.5 → 1.6` + `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. Closes `.4b`/`.4` — `flop_reset_provenance` delivered end-to-end (third query). 2 new MCP proofs; DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.4b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.4b.1 — pure flop_reset_provenance core` | `src/introspect/analyze.rs`: `QUERY_FLOP_RESET_PROVENANCE` + `FlopProvenance` + the third `flop_provenance` vec + `module_flop_provenance`/`design_flop_provenance` (a direct `Module.flops` projection). `supported_query_kinds()` unchanged (joins with dispatch in `.4b.2`). 5 reach proofs; DUT byte-identical (snapshots 6/6). |
| `SEMANTIC-INTROSPECTION-EXPANSION.4a` | `SEMANTIC-INTROSPECTION-EXPANSION.4a — flop_reset_provenance impl design-detail` | Design-detail (no source): pinned the third query's result shape (a third `flop_provenance: Vec<FlopProvenance>` vec — prior documents byte-identical), the derivation (a direct `Module.flops` projection), `"flop:<id>"` addressing, and the schema `1.5 → 1.6` bump. Registered `.4`/`.4a`/`.4b`; pre-split `.4b` → `.4b.1`/`.4b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.3b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.2 — input_reach MCP surface + schema 1.5` | Registry + `run_analyze` dispatch + schema `1.4 → 1.5` + `analyze_schema` enum + schema-doc/book/USER_GUIDE/README/KM. Closes `.3b`/`.3` — `input_reach` delivered end-to-end. 2 new MCP proofs; DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.3b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.3b.1 — pure input_reach analysis core` | `src/introspect/analyze.rs`: `QUERY_INPUT_REACH` + `ReachResult` + the second `reach_results` vec + `module_input_reach`/`design_input_reach` (invert the support relation). `supported_query_kinds()` unchanged (joins with dispatch in `.3b.2`). 7 reach proofs; DUT byte-identical (snapshots 6/6). |
| `SEMANTIC-INTROSPECTION-EXPANSION.3a` | `SEMANTIC-INTROSPECTION-EXPANSION.3a — input_reach impl design-detail` | Design-detail (no source): pinned the `input_reach` result shape (a second `reach_results: Vec<ReachResult>` vec — `output_support` stays byte-identical), the derivation (invert the support relation, reusing `module_support_cones`), the source addressing + `"flop:<id>"` direction-by-query duality, and the schema `1.4 → 1.5` bump. Pre-split `.3b` → `.3b.1`/`.3b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2b.2` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.2 — the pure MCP analyze tool + schema 1.3` | Schema `1.2→1.3`; `DerivedAnalysisDocument` + the pure MCP `analyze` tool (DUT-only; unknown query/target → `-32602`; cached + served as `anvil://artifact/<run_id>/analysis/<query>`); schema-doc §6.7 + book + USER_GUIDE + KM fact. Closes `.2b`/`.2` — the first query is delivered end-to-end. DUT byte-identical. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2b.1` | `SEMANTIC-INTROSPECTION-EXPANSION.2b.1 — pure support-cone analysis core` | New `src/introspect/analyze.rs`: `DerivedAnalysis`/`SupportCone` + `module_support_cones`/`design_support_cones` (combinational fan-in DFS over the existing IR; FlopQ a register-boundary leaf, `"flop:<id>"` targets, instance-boundary stop, opaque mem/fsm termination). 9 in-crate proofs; DUT byte-identical (no IR/generator change). Split `.2b` → `.2b.1`/`.2b.2`. |
| `SEMANTIC-INTROSPECTION-EXPANSION.2a` | `SEMANTIC-INTROSPECTION-EXPANSION.2a — support-cone impl design-detail` | Resolved `0011`'s 3 open questions: the `DerivedAnalysis`/`SupportCone` shape; `output_support` query-kind + name `target`; default-introspect-stays-lean + cone-stops-at-instance-boundary; schema `1.2→1.3`. Split `.2` → `.2a`/`.2b`. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION.1` | `SEMANTIC-INTROSPECTION-EXPANSION.1 — activate lane + derived-query API design` | Decision `0011`: a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation API (`DerivedAnalysis` schema `1.3` + pure MCP `analyze` tool); first query = the output support cone. Activated the lane by owner directive; split `.1`/`.2`/future. No source change. |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-23`: **`.6a` landed — design-detail (no source) for the fifth derived
  query, `flop_dependencies`** (the register→register dependency graph; the first
  query beyond decision `0011`'s four named kinds, under the lane's "open-ended
  breadth" clause). A `DEVELOPMENT_NOTES.md` design-detail entry pins the result
  shape (a fifth `flop_dependencies: Vec<FlopDependencies>` parallel vec —
  `FlopDependencies { flop, depends_on_flops, driven_flops, self_dependent }`; the
  four prior documents byte-identical), the derivation (reuse the support/reach
  machinery in one inversion pass), `"flop:<id>"` addressing, the top-module
  module-vs-design convention, and the schema `1.17 → 1.18` bump, grounded in a
  fresh read of `src/introspect/analyze.rs`. No new numbered decision (the
  `.3a`/`.4a`/`.5a` per-query precedent; decision `0011` governs the surface).
  Registered `.6`/`.6a`/`.6b` (+ root child `.6`); pre-split `.6b` → `.6b.1` (pure
  core) + `.6b.2` (surface); frontier set to `.6b.1`. Docs/design only — no
  `src/` ⇒ DUT byte-identical; self-checks green.
- `2026-06-16`: **`.5b.2` landed — closes `.5b`/`.5`; `module_reachability`
  delivered end-to-end** (the **fourth** and last named derived query from decision
  `0011`; DUT byte-identical). Surface wiring: the kind added to
  `analyze::supported_query_kinds()` **together with** the `run_analyze` dispatch
  (`module_module_reachability` / `design_module_reachability` in both the module
  and design paths) so the registry and dispatch never disagree; the unknown-target
  → `-32602` guard checks `module_reachability`; the `analyze_schema` `query` enum +
  `target` description + the `analyze` tool description + the server `instructions`
  gained the kind; `SCHEMA_VERSION` `1.6 → 1.7` (+ 8 `"1.6" → "1.7"` test
  assertions). Docs: schema-doc §6.7 (the fourth `module_reachability` payload +
  `ModuleReachability`; the stale "two parallel vecs" intro corrected to four) + the
  `1.6 → 1.7` changelog; book `agent-mcp` (a worked example + the four JSON examples
  bumped + the table/resource rows); USER_GUIDE + README; a new KM card
  `semantic-introspection-module-reachability` (+ cross-link, KM regenerated to 35
  facts); `CODEBASE_ANALYSIS` (analyze + mcp blocks). `cargo test --lib` 458/0/2 (2
  new MCP proofs); snapshots 6/6 byte-identical; clippy/fmt clean; mdbook +
  book_examples 3/3; e2e `anvil-mcp` smoke (hierarchy seed 42 → schema `1.7`, 3
  modules all reachable, top depth 0; unknown module → `-32602`). The tree stays
  `active`; **no active frontier** — all four named query kinds are delivered;
  further kinds are open-ended breadth, none retired.
- `2026-06-16`: **`.5b.1` landed — the pure `module_reachability` core** (DUT
  byte-identical). `src/introspect/analyze.rs` gains `QUERY_MODULE_REACHABILITY`,
  the `ModuleReachability { module, reachable, depth: Option<usize> (skip-if-None
  ⇒ present iff reachable), instantiates (distinct child module names,
  sorted/deduped), instance_count }` struct, the **fourth**
  `DerivedAnalysis.module_reachability` field (`#[serde(default,
  skip_serializing_if = "Vec::is_empty")]` ⇒ the three prior documents stay
  byte-identical), and the pure `design_module_reachability` /
  `module_module_reachability` builders (+ the internal
  `reachability_of`/`distinct_instantiated` helpers). `design_module_reachability`
  is a min-depth BFS from `design.top` over a name→`Module` index of the
  `Module.instances[].module` edges (children visited in sorted order; one entry
  per module sorted by name; both `InstanceRole` kinds are edges; an absent top ⇒
  every present module `reachable: false` — the honest whole-table enumeration, a
  documented divergence from the other `design_*` builders' top-absent
  early-return). `module_module_reachability` is the degenerate one-node case (a
  bare module rooted at itself). The 6 existing `DerivedAnalysis` literals gained
  `module_reachability: Vec::new()`. `supported_query_kinds()` unchanged —
  `module_reachability` joins it with the `run_analyze` dispatch in `.5b.2`. 6 new
  proofs. `cargo test --lib` 456/0/2; snapshots 6/6 byte-identical; clippy/fmt
  clean; `CODEBASE_ANALYSIS` `analyze.rs` block amended. Frontier advances to
  `.5b.2` (surface).
- `2026-06-16`: **`.5a` design-detail landed** (no source change) — opened `.5`,
  the **fourth** derived query `module_reachability`: which modules in a `Design`
  are reachable from `design.top` via the instance graph. Grounded in the real
  `Design`/`Module`/`Instance` IR. Resolved the five points: (1) a **fourth**
  parallel `module_reachability: Vec<ModuleReachability>` vec on `DerivedAnalysis`
  (`#[serde(default, skip_serializing_if)]` ⇒ the three prior documents stay
  byte-identical) with `ModuleReachability { module, reachable, depth:
  Option<usize>, instantiates, instance_count }`; (2) a BFS from `design.top` over
  the `Module.instances[].module` edges of a name→`Module` index (min-depth, pure,
  deterministic — output sorted by module name); (3) **module-name** target
  addressing (`None` ⇒ all, unknown ⇒ `-32602`) — distinct from the prior queries'
  port-name / `"flop:<id>"` targets; (4) module-vs-design semantics (the bare
  module degenerates to a one-node graph rooted at itself); (5) additive MINOR
  schema `1.6 → 1.7`. Registered `.5`/`.5a`/`.5b` (+ root child) and pre-split
  `.5b` → `.5b.1` (pure core, **new frontier**) + `.5b.2` (surface). Baseline
  `cargo check` clean; self-checks clean.
- `2026-06-16`: **`.4b.2` landed — closes `.4b`/`.4`; `flop_reset_provenance`
  delivered end-to-end** (the third derived query; DUT byte-identical). Surface
  wiring: the kind added to `analyze::supported_query_kinds()` **together with**
  the `run_analyze` dispatch (`module_flop_provenance`/`design_flop_provenance`)
  so the registry and dispatch never disagree; the unknown-target → `-32602`
  guard checks `flop_provenance`; `analyze_schema` `enum` + the tool/`instructions`
  text gained the kind; `SCHEMA_VERSION` `1.5 → 1.6` (+ 6 `"1.5" → "1.6"` test
  assertions). Docs: schema-doc §6.7 (the third `flop_provenance` payload +
  `FlopProvenance`) + the `1.5 → 1.6` changelog; book `agent-mcp` (a worked
  example + the three JSON examples bumped); USER_GUIDE + README; a new KM card
  `semantic-introspection-flop-reset-provenance` (+ cross-link, KM regenerated);
  `CODEBASE_ANALYSIS` + `ROADMAP`. `cargo test --lib` 450/0/2 (2 new MCP proofs);
  snapshots 6/6 byte-identical; clippy/fmt clean; mdbook + book_examples 3/3; e2e
  `anvil-mcp` smoke (seed 3 → schema `1.6`, 31 flops; unknown `flop:99999` →
  `-32602`). The tree stays `active`; **no active frontier** — the last named kind
  `module_reachability` is open-ended `.5+`, none retired.
- `2026-06-16`: **`.4b.1` landed — the pure `flop_reset_provenance` core** (DUT
  byte-identical). `src/introspect/analyze.rs` gains `QUERY_FLOP_RESET_PROVENANCE`,
  the `FlopProvenance` struct, the **third** `DerivedAnalysis.flop_provenance`
  field (`#[serde(default, skip_serializing_if = "Vec::is_empty")]` ⇒
  `output_support`/`input_reach` documents stay byte-identical), and the pure
  `module_flop_provenance`/`design_flop_provenance` builders (+ the internal
  `flop_provenance_with`/`flop_provenance_of`) — a direct projection of
  `Module.flops` (ascending id) mapping `ResetKind`→`none/sync/async`,
  `FlopKind`→`zero/hold`, `FlopMux`→`none/one_hot/encoded`, `reset_val` →
  decimal string. The 4 existing `DerivedAnalysis` literals gained
  `flop_provenance: Vec::new()`. `supported_query_kinds()` unchanged —
  `flop_reset_provenance` joins it with the `run_analyze` dispatch in `.4b.2`. 5
  new proofs (each variant; `"flop:<id>"` + unknown target; flopless ⇒ empty;
  serialization omits the other vecs; design top-module variant). `cargo test
  --lib` 448/0/2; snapshots 6/6 byte-identical; clippy/fmt clean. Frontier
  advances to `.4b.2` (surface).
- `2026-06-16`: **`.4a` design-detail landed** (no source change) — opened `.4`,
  the **third** derived query `flop_reset_provenance` (per-flop reset/data
  provenance: reset_kind/reset_value, ZeroDefault-vs-QFeedback default behavior,
  mux kind/arms, has_d), grounded in the real `Flop` type. Resolved the four
  points: (1) a **third** parallel `flop_provenance: Vec<FlopProvenance>` vec on
  `DerivedAnalysis` (`#[serde(default, skip_serializing_if)]` ⇒ `output_support`
  and `input_reach` documents stay byte-identical), enums → strings, `reset_value`
  a u128-safe decimal string; (2) a **direct projection of `Module.flops`** (no
  graph walk — the purest query yet); (3) `"flop:<id>"` addressing, `None` ⇒ all
  flops ascending, unknown ⇒ `-32602`; (4) additive MINOR schema `1.5 → 1.6`.
  Registered `.4`/`.4a`/`.4b` (+ root children) and pre-split `.4b` → `.4b.1`
  (pure core, **new frontier**) + `.4b.2` (surface). Baseline `cargo check`
  clean; self-checks clean.
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
