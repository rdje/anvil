# AGENT-MCP-EXPANSION: Broaden the Read-Mostly Agent/MCP Interface

## Metadata

- Tree ID: `AGENT-MCP-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability — agent/MCP interface breadth (post-AGENT-INTROSPECTION-MCP)`
- Created: `2026-06-15`
- Last updated: `2026-06-15` (`.3b` non-DUT lanes over MCP done; `.3` container closed; frontier → `.4`; `.1`/`.2`/`.3a` done; decision `0005`)
- Owner: repo-local workflow

## Goal

Extend the read-mostly agent/MCP interface delivered by the closed
`AGENT-INTROSPECTION-MCP` tree with the owner-gated breadth recorded in
decision `0004` / `MEMORY.md`, **without** touching the deterministic
generator core and **without** weakening any lane invariant:

- expose downstream **coverage gaps** as an MCP tool so an agent can ask
  "what is not yet exercised?" and drive generation toward it;
- drive the **non-DUT lanes** (`microdesign`, `frontend`) over MCP, so
  the agent can generate/introspect all three artifact families, not
  only DUT;
- add an optional **HTTP transport** for `anvil-mcp` beside the existing
  stdio transport.

The destination is a strictly richer, still read-mostly, still
default-off agent interface that helps an agent find downstream-tool
bugs — the `project_anvil_north_star` purpose — while the default
`anvil` build and `--artifact dut` stay byte-identical.

## Non-Goals

- No change to the deterministic generator core, the IR, or emitted RTL.
  Default `--artifact dut` stays byte-identical (the load-bearing
  contract enforced since `AGENT-INTROSPECTION-MCP.2a`).
- No new **computed truth** in the introspection schema — invariant
  SCHEMA-DERIVED holds (every payload field stays a serde projection of
  an existing `Config`/`Metrics`/`DesignMetrics`/coverage struct).
- No weakening of the controlled-tool guardrails: external tools run only
  through the hardened `downstream` allow-list (fixed binary names,
  sandboxed temp dir, RAM-guarded, audit-logged); no arbitrary
  shell/path.
- The AI agent is **never** a signoff oracle; ANVIL remains the source of
  truth. `minimize` continues to search the input `(seed, knobs)` space
  and never mutates/repairs RTL.
- No second source of truth: reuse `tool_matrix` / `downstream` /
  `diff_sim` / `metrics` / `introspect` rather than forking logic.

## Acceptance Criteria

- Each landed leaf is rules-first, default-off where it could change
  bytes, and proven against focused tests plus a downstream-clean smoke
  where a tool boundary is crossed.
- The MCP protocol surface stays unit-tested in-process (the
  `mcp::McpServer::handle` pure dispatcher pattern).
- DUT byte-identical contract preserved throughout (snapshots 6/6).
- `book/src/agent-mcp.md`, `USER_GUIDE.md`, and `README.md` are updated
  for any new user-visible MCP surface (closeout leaf).
- Every leaf is committed through `COMMIT.md` with its leaf ID in the
  subject.

## Task Tree

- ID: `AGENT-MCP-EXPANSION`
  Status: `active`
  Goal: `Broaden the read-mostly agent/MCP interface with owner-gated breadth.`
  Children: `AGENT-MCP-EXPANSION.1`, `AGENT-MCP-EXPANSION.2`, `AGENT-MCP-EXPANSION.3`, `AGENT-MCP-EXPANSION.4`, `AGENT-MCP-EXPANSION.5`

- ID: `AGENT-MCP-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf: scope the expansion, re-confirm every lane invariant, locate where coverage gaps are currently computed (matrix-side CoverageSummary) and how to surface them read-only, and finalize/split the .2-.5 decomposition. Record a decision (and a Knowledge Map card if a durable fact emerges).`
  Acceptance: `A decision record + this tree's confirmed leaf plan; no source change; docs/workflow validation clean.`
  Result: `Decision 0005 records the read-only exposure path for all three items; lane invariants re-confirmed against current code; coverage-gap source located as bin-private CoverageSummary/compute_coverage_gaps with already-recorded coverage_gaps in tool_matrix_report.json; .2 sharpened to a pure projection of a recorded report; .3 split into .3a (design) + .3b (impl); .4 gets a loopback-default security note. Decision 0005 carries Knowledge Map answers: front-matter (folds into KNOWLEDGE_MAP.md). No source change.`
  Verification: `docs/decision + task-tree + KM regen; check_memory_architecture.sh + check_knowledge_map.sh green (see Verification Log)`
  Commit: `AGENT-MCP-EXPANSION.1 — design/decision leaf + decision 0005`

- ID: `AGENT-MCP-EXPANSION.2`
  Status: `done`
  Goal: `Expose coverage gaps as a PURE MCP tool that projects a recorded tool_matrix_report.json (inline report OR report_path), returning the already-computed coverage_gaps + selected dark coverage facts + tool pass/fail. No recompute, no tool spawn. Per decision 0005.`
  Acceptance: `A new pure MCP tool returns the recorded coverage-gap set via a serde_json::Value key projection (NOT a mirror of the bin-private CoverageSummary struct); in-process protocol test (McpServer::handle); no new computed truth (gaps are relayed, not re-derived); read-only (no generation/tool spawn); DUT byte-identical.`
  Result: `Landed the pure coverage_gaps tool in src/mcp/mod.rs: project_coverage_gaps / load_coverage_report / coverage_gaps_projection. Accepts inline report OR report_path; relays the recorded coverage_gaps array + gap_count + clean flag + run metadata + tool_summary + the dark saw_* facts (recorded false booleans, sorted). serde_json::Value key projection — does NOT mirror the bin-private CoverageSummary. Dispatched before the seed/config parse (takes neither). 6 new in-process McpServer::handle tests (inline, path, clean, missing, both-args, not-a-report). DUT byte-identical (snapshots 6/6); no src/gen|emit|ir touched. User-facing book/USER_GUIDE/README sync deferred to .5 closeout per tree acceptance (the AGENT-INTROSPECTION-MCP .7 precedent).`
  Verification: `cargo fmt --check; cargo check --all-targets; cargo test --lib mcp:: (30 pass) + cargo test --test snapshots (6 pass, byte-identical); cargo clippy --all-targets -D warnings (all clean)`
  Commit: `AGENT-MCP-EXPANSION.2 — coverage_gaps pure-projection MCP tool`

- ID: `AGENT-MCP-EXPANSION.3`
  Status: `done`
  Goal: `Drive the non-DUT lanes (microdesign, frontend) over MCP — generate/introspect for --artifact microdesign|frontend through the umbrella ArtifactLane plumbing, keyed by a lane arg defaulting to dut. Per decision 0005, split into design + impl because the non-DUT introspection projection is an unresolved choice.`
  Children: `AGENT-MCP-EXPANSION.3a`, `AGENT-MCP-EXPANSION.3b`
  Result: `Container complete — .3a (design) and .3b (impl) both done. MCP generate/introspect now drive microdesign + frontend through the umbrella ArtifactLane; the manifest is inlined per schema §5/§6.5 and also served as a resource. DUT default byte-identical.`

- ID: `AGENT-MCP-EXPANSION.3a`
  Status: `done`
  Goal: `Design leaf: decide whether each non-DUT lane (microdesign, frontend) already exposes a manifest the introspection layer can project verbatim, or whether a thin per-lane projection must be defined — keeping the introspection document a serde projection of the existing manifest (no new computed truth). Record the chosen shape.`
  Acceptance: `A recorded decision (task-tree note and/or decision-record addendum) on the non-DUT introspection projection; no source change; docs/workflow validation clean.`
  Result: `Decided that both non-DUT lanes ALREADY emit a deterministic expected-facts manifest (microdesign::emit_manifest over microdesign::Manifest {seed,top,params,localparams,widths,generate,package_constants,const_exprs}; frontend::emit_manifest over frontend::Manifest {seed,top,packages,top_params,top_localparams,instances,generate_branches}), carried on the umbrella LaneArtifact.manifest: Option<String>, so introspection adds no new computed truth. **PARTIALLY SUPERSEDED BY .3b** — the .3a stance that the manifest goes ONLY via the ArtifactDescriptor.manifest ResourceRef (not inlined, "to avoid a schema bump / per §6.6") misread the schema contract: AGENT_INTROSPECTION_SCHEMA.md §5/§6.5 ALREADY define inlined microdesign_manifest/frontend_manifest payload sections at v1.0 (§6.6's "resource not inlined" applies only to the bulk .sv). .3b conforms to the schema: it inlines the manifest in the payload AND sets the artifact.manifest ResourceRef. Still NO schema-version bump (the sections were defined at v1.0). See Decisions 2026-06-15 .3b. No source change in .3a.`
  Verification: `scripts/check_memory_architecture.sh + knowledge-map check; design/decision leaf, no source change`
  Commit: `AGENT-MCP-EXPANSION.3a — non-DUT introspection projection design`

- ID: `AGENT-MCP-EXPANSION.3b`
  Status: `done`
  Goal: `Implementation leaf (per .3a): (1) add a manifest-carrying introspection builder reusing ArtifactDescriptor.manifest: Option<ResourceRef>; (2) extend MCP CachedArtifact with manifest: Option<String> and serve anvil://artifact/<run_id>/manifest; (3) generalize build_artifact to dispatch on a lane arg (default dut) through the umbrella MicrodesignLane/FrontendLane; (4) non-DUT MCP tool args carry lane + the lane's scoped knobs (n_params, n_children), not the DUT Config; (5) feed a deterministic canonical encoding of the lane knobs into content_run_id so non-DUT run_ids stay content-addressed.`
  Acceptance: `MCP generate/introspect work for microdesign + frontend via existing lane impls (manifest served as a resource); in-process McpServer::handle tests; NO schema-version bump (document shape unchanged); default dut path unchanged and byte-identical (snapshots 6/6).`
  Result: `Landed in src/introspect/mod.rs + src/mcp/mod.rs. introspect: content_run_id refactored to content_run_id_for_knobs (DUT output byte-identical) so non-DUT lanes feed scoped-knob JSON ({"n_params":N} / {"n_params":N,"n_children":M}) into the content address; new manifest_lane_document builds the non-DUT envelope as a serde_json::Value (NOT the typed IntrospectionDocument — RequestEcho.knobs is a Config, and a Value object would re-sort DUT keys), INLINING the manifest under microdesign_manifest/frontend_manifest per schema §5/§6.5 AND setting the artifact.manifest ResourceRef (§4). mcp: CachedArtifact gains manifest: Option<String>; build_and_cache_lane routes through umbrella MicrodesignLane/FrontendLane; lane arg (default dut) branches generate/introspect before the DUT config parse; generate/introspect get a generate_schema with lane/config/n_params/n_children; resources_read serves .../manifest, resources_list advertises it. 5 new in-process tests. SCHEMA-CONFORMANCE FIX during impl: .3a wrongly planned ResourceRef-only/no-inline — the schema already mandates inlined payload sections at v1.0, so .3b inlines (see Decisions). DUT byte-identical (snapshots 6/6; introspect 6/6).`
  Verification: `cargo fmt --check; cargo check --all-targets; cargo test --lib mcp:: (35 pass, incl. 5 new) + cargo test --lib introspect:: (6 pass) + cargo test --test snapshots (6 pass, byte-identical); cargo clippy --all-targets -- -D warnings`
  Commit: `AGENT-MCP-EXPANSION.3b — non-DUT lanes (microdesign/frontend) over MCP`

- ID: `AGENT-MCP-EXPANSION.4`
  Status: `pending`
  Goal: `Optional HTTP transport for anvil-mcp beside stdio (stdio remains the default), driving the same McpServer::handle dispatcher behind an explicit opt-in flag; bind loopback-only by default. Per decision 0005.`
  Acceptance: `An HTTP transport drives the same McpServer::handle dispatcher; stdio default unchanged; loopback-only default bind; per-call downstream guardrails (allow-list/sandbox/RAM-guard/audit) unchanged; transport-level test.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-MCP-EXPANSION.5`
  Status: `pending`
  Goal: `Closeout — sync book/src/agent-mcp.md + USER_GUIDE.md + README.md to the expanded MCP surface; close the tree.`
  Acceptance: `mdBook builds clean; book_examples gate green; user-facing surfaces reflect the new tools/transport.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `AGENT-MCP-EXPANSION.4` | `pending` | Optional loopback-default HTTP transport over the same `McpServer::handle`. |
| 2 | `AGENT-MCP-EXPANSION.5` | `pending` | Closeout: sync book/USER_GUIDE/README to the expanded MCP surface (coverage_gaps + non-DUT lanes + HTTP). |

## Decisions

- `2026-06-15`: Open this lane `active` as the first of the three
  owner-directed post-phase capability lanes (order `2 → 3 → 1`). The
  first leaf is a design/decision leaf because the coverage-gap source is
  currently matrix-only and the read-only exposure path needs to be
  decided before implementation (mirrors how the original MCP lane led
  with `.1` decision + `.2` schema spec).
- `2026-06-15` (`.1`): Recorded decision
  [`0005`](../decisions/0005-agent-mcp-expansion-surface.md). (a) Coverage
  gaps are surfaced as a **pure tool that projects a recorded
  `tool_matrix_report.json`** (inline or by path) — the report already
  carries the `compute_coverage_gaps` output, so the MCP tool relays it
  via a `serde_json::Value` key projection (never mirroring the
  bin-private `CoverageSummary`), keeping the single gap computation in
  `tool_matrix`. A recompute-on-demand controlled tool was rejected
  (second source of truth, heavy, against read-mostly/no-new-truth).
  (b) Non-DUT lanes route through the umbrella `ArtifactLane` dispatch
  keyed by a `lane` arg (default `dut`); `.3` split into `.3a` design +
  `.3b` impl because the non-DUT introspection projection is unresolved.
  (c) HTTP transport drives the same `McpServer::handle` dispatcher behind
  an opt-in flag, loopback-only by default, stdio still default. All five
  `0004` lane invariants re-confirmed against current code.
- `2026-06-15` (`.3a`): Non-DUT introspection projection decided. Both
  non-DUT lanes already emit a deterministic expected-facts manifest
  (`microdesign::emit_manifest` / `frontend::emit_manifest`, serde
  projections of their `Manifest` structs), carried on the umbrella
  `LaneArtifact.manifest: Option<String>`. The non-DUT introspection
  document **reuses the existing, currently-unused
  `ArtifactDescriptor.manifest: Option<ResourceRef>` slot** to point at
  `anvil://artifact/<run_id>/manifest`, and the manifest content is served
  as an MCP **resource** (schema §6.6: full manifests are fetched, not
  inlined). This means **no new per-lane projection logic and no
  schema-version bump** — only previously-`None` fields populate and a new
  lane/`kind` value appears. The DUT-only payload
  (`module_metrics`/`design_metrics`/`modules`) stays absent for non-DUT
  lanes; their structured facts live in the manifest resource. Invariant
  SCHEMA-DERIVED holds (the manifest is an existing serde projection;
  introspect computes nothing). Rejected: inlining the manifest into the
  introspection payload (a new `lane_manifest` field) — it would bump the
  schema and violate §6.6's "structured queries, not bulk dumps".
  **[Corrected by `.3b` — see below.]**
- `2026-06-15` (`.3b`, **corrects `.3a`**): Implementation against the schema
  contract showed the `.3a` "ResourceRef-only / no inline" stance was wrong.
  `docs/AGENT_INTROSPECTION_SCHEMA.md` §5/§6.5 **already define** inlined
  `microdesign_manifest` / `frontend_manifest` payload sections **at v1.0**
  (they are "small and stable", an exact serde projection of each lane's
  `Manifest`); §6.6's "resource, not inlined" rule applies only to the **bulk
  `.sv`**, not the manifest. So `.3b` **conforms to the schema**: it inlines
  the manifest in the `introspection` payload under the schema key **and**
  also sets the `artifact.manifest` ResourceRef (§4) + serves it as a
  resource — both derive from the one `emit_manifest` output, so they cannot
  drift. Net: still **no schema-version bump** (the sections existed at v1.0;
  populating them is conformance), and the agent gets both the inlined facts
  and a raw-bytes resource. Lesson recorded: a design leaf must check the
  schema *spec*, not only the code, before deciding the wire shape.

## Open Questions

- (`.1` resolved) Coverage is surfaced as a **pure projection of a
  recorded report**, not an on-demand recompute — see decision `0005`.
- (`.3a` resolved) Each non-DUT lane already emits a verbatim-projectable
  manifest; the introspection document reuses the existing
  `ArtifactDescriptor.manifest` ResourceRef slot and serves the manifest
  as a resource — no new projection, no schema bump. See Decisions
  (`2026-06-15` `.3a`).
- (`.3b` resolved) `content_run_id` was refactored to
  `content_run_id_for_knobs(lane, seed, knobs_json)` (DUT output
  byte-identical); non-DUT lanes feed a deterministic scoped-knob JSON
  (`{"n_params":N}` / `{"n_params":N,"n_children":M}`) into the address, so
  same `(seed, lane, knobs)` ⇒ same `run_id` and differing scoped knobs ⇒
  distinct `run_id` (proven by `non_dut_lane_run_id_is_deterministic_and_knob_sensitive`).
- `.4` decides: a sub-flag on `anvil-mcp` (`--http <addr>`) vs a separate
  bin. Bias: a flag on the existing bin, loopback default.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `AGENT-MCP-EXPANSION.1` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/gen_knowledge_map.sh` regen + `knowledge-map/scripts/check_knowledge_map.sh`; docs/decision + task-tree edits; no source change (design/decision leaf) | `clean` |
| `2026-06-15` | `AGENT-MCP-EXPANSION.2` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo test --lib mcp::` (30 pass, incl. 6 new); `cargo test --test snapshots` (6 pass, byte-identical); `cargo clippy --all-targets -- -D warnings` | `clean` |
| `2026-06-15` | `AGENT-MCP-EXPANSION.3a` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; design/decision leaf, no source change | `clean` |
| `2026-06-15` | `AGENT-MCP-EXPANSION.3b` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo test --lib mcp::` (35 pass, incl. 5 new) + `cargo test --lib introspect::` (6 pass) + `cargo test --test snapshots` (6 pass, byte-identical); `cargo clippy --all-targets -- -D warnings` | `clean` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGENT-MCP-EXPANSION.1` | `AGENT-MCP-EXPANSION.1 — design/decision leaf + decision 0005` | Decision `0005`; `.2` sharpened; `.3` split `.3a`/`.3b`; `.4` loopback note; frontier → `.2`. |
| `AGENT-MCP-EXPANSION.2` | `AGENT-MCP-EXPANSION.2 — coverage_gaps pure-projection MCP tool` | Pure tool projecting a recorded `tool_matrix_report.json`; DUT byte-identical; frontier → `.3a`. |
| `AGENT-MCP-EXPANSION.3a` | `AGENT-MCP-EXPANSION.3a — non-DUT introspection projection design` | Design leaf; partially superseded by `.3b` (schema mandates inlined payload sections). |
| `AGENT-MCP-EXPANSION.3b` | `AGENT-MCP-EXPANSION.3b — non-DUT lanes (microdesign/frontend) over MCP` | Umbrella-routed generate/introspect; manifest inlined (§6.5) + resource (§4); DUT byte-identical; `.3` container closed; frontier → `.4`. |

## Changelog

- `2026-06-15`: Created task tree (Lane 2), opened `active`, frontier at
  `.1`, via `CAPABILITY-LANE-OWNERSHIP.1`.
- `2026-06-15`: `.1` done — decision `0005` recorded; `.2` sharpened to a
  pure recorded-report projection; `.3` split into `.3a` (design) +
  `.3b` (impl); `.4` gets a loopback-default security note; frontier
  advanced to `.2` then `.3a`.
- `2026-06-15`: `.2` done — pure `coverage_gaps` MCP tool landed in
  `src/mcp/mod.rs` (projects a recorded `tool_matrix_report.json`; 6 new
  in-process tests; DUT byte-identical); frontier advanced to `.3a`.
- `2026-06-15`: `.3a` done — non-DUT introspection projection decided
  (reuse `ArtifactDescriptor.manifest` ResourceRef + serve manifest as a
  resource; no schema bump; manifest stays an existing serde projection).
  No source change; frontier advanced to `.3b`.
- `2026-06-15`: `.3b` done — non-DUT lanes (microdesign/frontend) routed
  over MCP through the umbrella `ArtifactLane`; manifest inlined per schema
  §5/§6.5 **and** served as a resource (§4). Corrected `.3a`'s
  ResourceRef-only stance to conform to the schema spec. `.3` container
  closed. DUT byte-identical (snapshots 6/6). Frontier advanced to `.4`.
