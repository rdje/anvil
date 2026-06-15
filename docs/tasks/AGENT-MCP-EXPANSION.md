# AGENT-MCP-EXPANSION: Broaden the Read-Mostly Agent/MCP Interface

## Metadata

- Tree ID: `AGENT-MCP-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability — agent/MCP interface breadth (post-AGENT-INTROSPECTION-MCP)`
- Created: `2026-06-15`
- Last updated: `2026-06-15` (`.2` coverage_gaps pure-projection tool done; frontier → `.3a`; `.1` design/decision leaf done with decision `0005`; `.3` split into `.3a`/`.3b`)
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
  Status: `active`
  Goal: `Drive the non-DUT lanes (microdesign, frontend) over MCP — generate/introspect for --artifact microdesign|frontend through the umbrella ArtifactLane plumbing, keyed by a lane arg defaulting to dut. Per decision 0005, split into design + impl because the non-DUT introspection projection is an unresolved choice.`
  Children: `AGENT-MCP-EXPANSION.3a`, `AGENT-MCP-EXPANSION.3b`

- ID: `AGENT-MCP-EXPANSION.3a`
  Status: `pending`
  Goal: `Design leaf: decide whether each non-DUT lane (microdesign, frontend) already exposes a manifest the introspection layer can project verbatim, or whether a thin per-lane projection must be defined — keeping the introspection document a serde projection of the existing manifest (no new computed truth). Record the chosen shape.`
  Acceptance: `A recorded decision (task-tree note and/or decision-record addendum) on the non-DUT introspection projection; no source change; docs/workflow validation clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-MCP-EXPANSION.3b`
  Status: `pending`
  Goal: `Implementation leaf: route MCP generate/introspect through the umbrella ArtifactLane dispatch keyed by a lane arg (default dut), per the .3a-decided projection.`
  Acceptance: `MCP generate/introspect work for microdesign + frontend via existing lane impls; in-process tests; default dut path unchanged and byte-identical.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `AGENT-MCP-EXPANSION.3a` | `pending` | Non-DUT introspection projection design; precedes `.3b` impl. |
| 2 | `AGENT-MCP-EXPANSION.3b` | `pending` | Route MCP generate/introspect through the umbrella lane dispatch, per `.3a`. |

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

## Open Questions

- (`.1` resolved) Coverage is surfaced as a **pure projection of a
  recorded report**, not an on-demand recompute — see decision `0005`.
- `.3a` decides: does each non-DUT lane already expose a manifest the
  introspection layer can project verbatim, or must a thin per-lane
  projection be defined? Bias: reuse the existing manifest; define nothing
  new that can drift.
- `.4` decides: a sub-flag on `anvil-mcp` (`--http <addr>`) vs a separate
  bin. Bias: a flag on the existing bin, loopback default.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `AGENT-MCP-EXPANSION.1` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/gen_knowledge_map.sh` regen + `knowledge-map/scripts/check_knowledge_map.sh`; docs/decision + task-tree edits; no source change (design/decision leaf) | `clean` |
| `2026-06-15` | `AGENT-MCP-EXPANSION.2` | `cargo fmt --all --check`; `cargo check --all-targets`; `cargo test --lib mcp::` (30 pass, incl. 6 new); `cargo test --test snapshots` (6 pass, byte-identical); `cargo clippy --all-targets -- -D warnings` | `clean` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGENT-MCP-EXPANSION.1` | `AGENT-MCP-EXPANSION.1 — design/decision leaf + decision 0005` | Decision `0005`; `.2` sharpened; `.3` split `.3a`/`.3b`; `.4` loopback note; frontier → `.2`. |
| `AGENT-MCP-EXPANSION.2` | `AGENT-MCP-EXPANSION.2 — coverage_gaps pure-projection MCP tool` | Pure tool projecting a recorded `tool_matrix_report.json`; DUT byte-identical; frontier → `.3a`. |

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
