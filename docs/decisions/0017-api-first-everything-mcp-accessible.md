---
id: api-first-everything-mcp-accessible
title: API-first mandate — every meaningful ANVIL feature must be MCP-accessible, controllable, steerable, and queryable; deep semantic introspection is first-class
answers:
  - "must every ANVIL feature be exposed over MCP"
  - "is ANVIL fully automatable via API"
  - "what is the API-first mandate"
  - "what is the API-completeness gate for a new lane"
  - "do new knobs have to be settable via the MCP/config API"
  - "do new actions have to be invocable via an MCP tool"
  - "does every queryable fact have to be exposed via introspect or analyze"
  - "is deep semantic introspection first-class in ANVIL"
  - "can a feature ship CLI-only without an API"
  - "what cross-cutting acceptance criterion do the usability lanes share"
  - "is the human CLI a shim over the same API"
date: 2026-06-17
status: accepted
tags: [mcp, api, agent, introspection, steerable, queryable, doctrine, north-star, architecture, automation]
evidence: docs/decisions/0017-api-first-everything-mcp-accessible.md; docs/decisions/0004-agent-introspection-mcp-lane.md; docs/decisions/0005-agent-mcp-expansion-surface.md; docs/decisions/0011-semantic-introspection-derived-query-surface.md; docs/AGENT_INTROSPECTION_SCHEMA.md; src/mcp/mod.rs; src/introspect/mod.rs; book/src/agent-mcp.md; docs/tasks/SEMANTIC-INTROSPECTION-EXPANSION.md; docs/tasks/AGENT-MCP-EXPANSION.md
---

# 0017 - API-first: every meaningful ANVIL feature is MCP-accessible, controllable, steerable, and queryable

- Date: 2026-06-17
- Status: accepted
- Tree: registered by `USABILITY-LANE-OWNERSHIP.1`; a standing cross-cutting
  constraint on **all** current and future lanes.
- Extends: decisions [`0004`](0004-agent-introspection-mcp-lane.md) (the
  introspection/MCP lane + the `SCHEMA-DERIVED` / no-shadow-simulator boundary),
  [`0005`](0005-agent-mcp-expansion-surface.md) (read-mostly MCP breadth), and
  [`0011`](0011-semantic-introspection-derived-query-surface.md) (the
  derived-relation query surface). Builds on the `feedback_api_for_agents_not_humans`
  memory.

## Context

**Owner directive (`2026-06-17`):** *"ANVIL shall be fully automatable via API
using MCP. Everything meaningful in ANVIL shall be API-accessible, fully
controllable, steerable. Everything that can or should be queryable has to be
made queryable via API. Deep semantic introspection shall be first-class, more
than ever."*

ANVIL already has a strong agent surface — the `anvil-mcp` server (decisions
`0004`/`0005`): pure tools (`generate`/`introspect`/`analyze`/`dump_config`/
`coverage_gaps`), controlled tools (`validate`/`minimize`), resources, prompts,
and the SCHEMA-DERIVED derived-relation `analyze` API (decision `0011`). But that
surface grew **feature-by-feature**: each new capability decided, ad hoc, how
much of itself to expose. The owner is now elevating API-completeness from an
ad-hoc nicety to a **standing doctrine** that gates every lane: ANVIL's primary
consumer is an automating agent (`feedback_api_for_agents_not_humans`), so a
feature that is not fully reachable, controllable, steerable, and queryable over
MCP is **not done**.

This decision is registered alongside seven new owner-directed lanes (the six
"make it more usable" ideas plus the capability-breadth lane) — see
`USABILITY-LANE-OWNERSHIP` — and binds all of them, plus the existing
`SEMANTIC-INTROSPECTION-EXPANSION` and `AGENT-MCP-EXPANSION` lanes, which become
the cross-cutting homes for API-completeness deepening.

## Decision

**Every meaningful ANVIL feature must be fully exposed over the MCP API.** This
is a cross-cutting acceptance criterion — the **API-completeness gate** — that
applies to every current and future lane. A feature/lane leaf is not `done`
until all four hold:

1. **Controllable / steerable** — every knob, profile, target, or control the
   feature adds is settable via the MCP/config API (the `generate`/`analyze`
   tool inputs + `Config` serde), not CLI-only. The human CLI is a thin shim
   over the same API, never a superset of it.
2. **Invocable** — every *action* the feature adds (a hunt, a sweep, a
   minimize, a divergence check, a preset application) is reachable as an MCP
   tool, with the same default-off / reproducible / sandboxed discipline as the
   existing tools (decision `0004`).
3. **Queryable** — every fact, relation, metric, or result the feature produces
   that *should* be queryable is exposed via `--introspect` / the `analyze`
   registry, **SCHEMA-DERIVED** (a projection of an existing struct or a pure
   graph traversal — never a recomputed second source of truth, and never a
   behavioural oracle). Deep semantic introspection is treated as first-class:
   when in doubt, add the query kind.
4. **Documented** — the surface is reflected in `book/src/agent-mcp.md` + the
   schema doc (`docs/AGENT_INTROSPECTION_SCHEMA.md`) with the schema version
   bumped per its additive-MINOR policy.

### The hard boundary is unchanged

The `SCHEMA-DERIVED` / structure-first / **no-shadow-simulator** ceiling
(decisions `0004`/`0011`, ROADMAP steering gap 4) is **load-bearing and
preserved**. "Everything queryable" means every *derivable structural/relational
fact*, exposed as a vetted, named query kind in the `analyze` registry — **not** a
free-form query language and **not** behavioural truth (signal values, timing,
intended function). ANVIL has no whole-module semantics by design; this mandate
does not invent one.

### Lane invariants preserved

Default-off / byte-identical where output could change; rules-first / no
generate-then-filter; reproducible (seeded; no wall-clock / no `thread_rng`);
sandboxed + allow-listed + audit-logged controlled tools; no retirement of
existing surfaces. API-completeness is additive on top of these.

## Decisive test applied

For any new feature, ask: *"Could an agent, with only the MCP API and no shell
access to the ANVIL binary, fully drive this feature — set its controls, invoke
its actions, and read back everything it produces that matters?"* If the answer
is no, the feature is incomplete. A query is admissible iff it is a pure
projection of facts the artifact/IR already holds (the `SCHEMA-DERIVED` test).

## Rejected alternatives

- **CLI-only features (no API).** Rejected outright — it directly contradicts
  the automation mandate and `feedback_api_for_agents_not_humans`. The CLI is a
  shim over the API, not a parallel surface.
- **A free-form query language over the IR.** Rejected (per decision `0011`):
  keep the vetted, named-kind `analyze` registry — extensible by adding kinds,
  each reviewed and pure.
- **A behavioural / simulation query API** ("what is `y` when `x=…`",
  waveforms, timing). Forbidden by decision `0004` and ROADMAP gap 4. Still out
  of scope under this mandate.
- **A second source of truth** (precomputing/duplicating facts into a parallel
  store to "make them queryable"). Rejected: project, don't recompute-and-store
  (the `coverage_gaps` / `analyze` precedent).
- **One mega "API completeness" rewrite tree.** Rejected: API-completeness is a
  *standing gate on every lane*, enforced per-leaf, not a one-time project —
  `SEMANTIC-INTROSPECTION-EXPANSION` + `AGENT-MCP-EXPANSION` own the cross-cutting
  deepening as breadth.

## Consequences

- Every one of the seven new owner-directed lanes (`BUG-HUNT-ORCHESTRATION`,
  `ACCEPTANCE-DIVERGENCE-HUNTING`, `DOWNSTREAM-ADAPTER-EXPANSION`,
  `KNOB-ERGONOMICS-AND-PRESETS`, `CI-PACKAGING-DISTRIBUTION`,
  `COVERAGE-STEERED-GENERATION`, `CAPABILITY-BREADTH-EXPANSION`) carries the
  API-completeness gate in its acceptance criteria.
- `SEMANTIC-INTROSPECTION-EXPANSION` (deep introspection) and
  `AGENT-MCP-EXPANSION` (MCP breadth) are reaffirmed as first-class, ongoing
  lanes — the cross-cutting homes for new query kinds and tool surfaces.
- The introspection schema and the MCP tool/prompt registries will grow with
  every lane; the additive-MINOR schema policy and the pure/cached/sandboxed
  tool discipline absorb that growth without breaking determinism.
- The structure-first / no-shadow-simulator boundary is re-stated, in writing,
  as the permanent ceiling on "everything queryable."

## Links

- Owner directive: `2026-06-17` (full MCP automatability; everything
  accessible/controllable/steerable/queryable; deep semantic introspection
  first-class).
- Parent decisions: `0004` (MCP lane + `SCHEMA-DERIVED` + no-shadow-simulator),
  `0005` (read-mostly MCP breadth, project-don't-recompute), `0011`
  (derived-relation `analyze` registry).
- Memory: `feedback_api_for_agents_not_humans` (design the API for agents, not
  human-readable minimalism); `project_anvil_north_star` (surface downstream-tool
  bugs — full automation multiplies the bug-finding loop).
- Owning lanes: `USABILITY-LANE-OWNERSHIP` (registers the seven lanes),
  `SEMANTIC-INTROSPECTION-EXPANSION` + `AGENT-MCP-EXPANSION` (cross-cutting
  API-completeness deepening).
