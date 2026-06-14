---
id: agent-introspection-schema
title: ANVIL's agent-introspection schema is a versioned envelope derived from existing metrics/manifest/config
answers:
  - "what fields does the ANVIL introspection schema expose"
  - "where is the ANVIL introspection schema spec"
  - "is the ANVIL introspection schema versioned"
  - "what is anvil schema_version"
  - "does the ANVIL introspection adapter compute new truth"
  - "what is the ANVIL introspection envelope"
  - "how is the ANVIL introspection schema kept from drifting"
  - "what is invariant SCHEMA-DERIVED"
date: 2026-06-14
status: current
tags: [mcp, agent, api, introspection, schema, versioning]
evidence: docs/AGENT_INTROSPECTION_SCHEMA.md; src/metrics.rs; src/config.rs; src/bin/tool_matrix.rs; docs/decisions/0004-agent-introspection-mcp-lane.md
---

The agent-introspection schema (`AGENT-INTROSPECTION-MCP.2`, spec at
`docs/AGENT_INTROSPECTION_SCHEMA.md`) is a thin **versioned envelope** around
facts ANVIL already records, not a new computation. The envelope carries
`schema_version` (`"1.0"`), `anvil_version`, `lane`, a `request` echo of the
`(seed, knobs, lane)` determinism tuple with a content-addressed `run_id`, an
`artifact` descriptor (`.sv`/manifest as fetch-on-demand `ResourceRef`s), the
`introspection` payload, and `warnings`.

**Invariant SCHEMA-DERIVED:** every payload section is the exact `serde`
projection of an existing struct — `config` ← `Config`, `module_metrics` ←
`Metrics`, `design_metrics` ← `DesignMetrics` (`metrics::compute` /
`compute_design`), `coverage` ← `tool_matrix::CoverageSummary` (incl.
`coverage_gaps`), and the `microdesign`/`frontend` lane `Manifest`s. The
adapter computes zero new truth; struct field lists stay owned by the code, so
the schema cannot become a second source of truth that drifts. `coverage` is a
matrix-run property and is absent (with a `warnings[]` note) for a
single-artifact generate.

**Versioning:** `MAJOR.MINOR`. Additive `#[serde(default)]` growth is
MINOR/compatible; rename/retype/semantic change or section removal is MAJOR;
`anvil_version` travels alongside; determinism is preserved across versions.
This is the contract the code leaves (`.3`+) must conform to; see
[[agent-introspection-mcp-lane]] for the lane architecture.
