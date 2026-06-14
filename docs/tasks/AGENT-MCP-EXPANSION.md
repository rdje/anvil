# AGENT-MCP-EXPANSION: Broaden the Read-Mostly Agent/MCP Interface

## Metadata

- Tree ID: `AGENT-MCP-EXPANSION`
- Status: `active`
- Roadmap lane: `Capability — agent/MCP interface breadth (post-AGENT-INTROSPECTION-MCP)`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
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
  Status: `pending`
  Goal: `Design/decision leaf: scope the expansion, re-confirm every lane invariant, locate where coverage gaps are currently computed (matrix-side CoverageSummary) and how to surface them read-only, and finalize/split the .2-.5 decomposition. Record a decision (and a Knowledge Map card if a durable fact emerges).`
  Acceptance: `A decision record + this tree's confirmed leaf plan; no source change; docs/workflow validation clean.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-MCP-EXPANSION.2`
  Status: `pending`
  Goal: `Expose coverage gaps as an MCP tool — surface the downstream CoverageSummary gap list read-only so an agent can target unexercised surfaces. Provisional pending .1.`
  Acceptance: `A new MCP tool returns the current coverage-gap set; in-process protocol test; no new computed truth beyond projecting existing coverage facts; DUT byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-MCP-EXPANSION.3`
  Status: `pending`
  Goal: `Drive the non-DUT lanes (microdesign, frontend) over MCP — generate/introspect for --artifact microdesign|frontend through the umbrella ArtifactLane plumbing. Provisional pending .1.`
  Acceptance: `MCP generate/introspect work for the non-DUT lanes via existing lane impls; in-process tests; default dut path unchanged.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-MCP-EXPANSION.4`
  Status: `pending`
  Goal: `Optional HTTP transport for anvil-mcp beside stdio (stdio remains the default). Provisional pending .1.`
  Acceptance: `An HTTP transport drives the same McpServer::handle dispatcher; stdio default unchanged; transport-level test.`
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
| 1 | `AGENT-MCP-EXPANSION.1` | `pending` | Design leaf must confirm invariants, locate the coverage-gap source, and finalize the decomposition before any code edit. |

## Decisions

- `2026-06-15`: Open this lane `active` as the first of the three
  owner-directed post-phase capability lanes (order `2 → 3 → 1`). The
  first leaf is a design/decision leaf because the coverage-gap source is
  currently matrix-only and the read-only exposure path needs to be
  decided before implementation (mirrors how the original MCP lane led
  with `.1` decision + `.2` schema spec).

## Open Questions

- `.1` decides: is coverage best surfaced as a pure tool over a recorded
  `CoverageSummary`, or as a controlled tool that runs a small matrix
  subset? The read-mostly + default-off + no-new-truth invariants bias
  toward projecting an existing/recorded summary rather than computing
  new state on demand.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-15` | `AGENT-MCP-EXPANSION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGENT-MCP-EXPANSION.1` | `pending` | `pending` |

## Changelog

- `2026-06-15`: Created task tree (Lane 2), opened `active`, frontier at
  `.1`, via `CAPABILITY-LANE-OWNERSHIP.1`.
