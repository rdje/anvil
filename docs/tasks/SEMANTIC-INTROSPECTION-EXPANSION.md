# SEMANTIC-INTROSPECTION-EXPANSION: behavioral query surface beyond structural projection

## Metadata

- Tree ID: `SEMANTIC-INTROSPECTION-EXPANSION`
- Status: `proposed`
- Roadmap lane: `Capability — deeper agent/introspection surface (extends AGENT-INTROSPECTION-MCP / AGENT-MCP-EXPANSION)`
- Created: `2026-06-15`
- Last updated: `2026-06-15`
- Owner: repo-local workflow
- Note: registered `proposed` by owner roadmap steering (`2026-06-15`) as a named
  sibling of `SV-VERSION-TARGETING` (the activated lane). Captured here so it is
  not overlooked.

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
  Status: `proposed`
  Goal: `Behavioral / derived query surface over generated artifacts, derived from existing IR facts.`
  Children: `SEMANTIC-INTROSPECTION-EXPANSION.1`

- ID: `SEMANTIC-INTROSPECTION-EXPANSION.1`
  Status: `proposed`
  Goal: `(Future) Design/decision leaf: inventory candidate derived/behavioral queries that stay SCHEMA-DERIVED (no new oracle), pick the first, define its schema versioning + derivation + default-off contract, and split the tree — before any code.`
  Acceptance: `A decision record naming the first query, its derivation from existing IR facts, and its schema/versioning; no source change; self-checks clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SEMANTIC-INTROSPECTION-EXPANSION.1` | `proposed` | Not active. Eligible once this lane is selected; first leaf is a design/decision leaf (pick the first derived query, keep it SCHEMA-DERIVED). |

## Decisions

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
| `2026-06-15` | `SEMANTIC-INTROSPECTION-EXPANSION` | Tree registered `proposed` (ownership only, no leaf executed). | `done` (registration) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEMANTIC-INTROSPECTION-EXPANSION` | `SV-VERSION-TARGETING.1 — open SV-version lane + decision 0009` | Registered `proposed` alongside the activated `SV-VERSION-TARGETING` lane. |

## Changelog

- `2026-06-15`: Created and registered `proposed` (owner-directed sibling lane).
