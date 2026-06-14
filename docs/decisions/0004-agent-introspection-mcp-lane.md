---
id: agent-introspection-mcp-lane
title: ANVIL exposes agent control + deep introspection as a default-off MCP adapter beside the generator core
answers:
  - "should ANVIL expose an MCP server for AI agents"
  - "what is ANVIL's agent / MCP interface architecture"
  - "is the ANVIL MCP server inside the generator core"
  - "why doesn't ANVIL need a stateful simulator-style session API"
  - "can an AI agent drive ANVIL to find downstream tool bugs"
  - "what RTL-simulator MCP advice applies to ANVIL"
  - "what is the ANVIL introspection API"
date: 2026-06-14
status: current
tags: [mcp, agent, api, architecture, introspection]
evidence: docs/decisions/0004-agent-introspection-mcp-lane.md; docs/tasks/AGENT-INTROSPECTION-MCP.md; DEVELOPMENT_NOTES.md
---

# 0004 - Agent control + deep introspection exposed as a default-off MCP adapter beside the generator core

- Date: 2026-06-14
- Status: accepted
- Tags: mcp, agent, api, architecture, introspection

## Context

The owner asked whether ANVIL should build deep semantic introspection
exposed through a clean API so an AI agent can automate ANVIL via MCP
(Model Context Protocol), and shared reference advice originally written
for an *RTL simulator* with MCP support. This record captures the decision
to pursue that lane and the architecture/guardrails it must follow. The
owning task tree is [`AGENT-INTROSPECTION-MCP`](../tasks/AGENT-INTROSPECTION-MCP.md).

ANVIL's north star (`project_anvil_north_star`): surface downstream-tool
bugs via valid-by-construction, downstream-acceptance-quality output. The
highest-value agent use is therefore closing the bug-hunting loop —
generate, validate against Verilator/Yosys/iverilog/`--diff-sim`, and on a
tool failure shrink `(seed, knobs)` to a minimal reproducer — autonomously.

ANVIL is already "machine-controllable": a Rust library API (`Generator`,
`Config`, `metrics::compute`/`compute_design`, `manifest`), JSON
expected-facts manifests, `DesignMetrics`/`Metrics`, `--dump-config`, and
the `tool_matrix` harness with `coverage_gaps`. The gap is a *stable,
versioned, agent-shaped* surface and an MCP bridge.

## Decision

Build an **agent-introspection + MCP lane** as a thin, read-mostly adapter
*beside* the generator core, not inside it. The simulator reference advice
is adopted where it transfers and explicitly rejected where it is
simulator-specific.

### Architecture (what transfers from the reference advice)

- **Machine-controllable first, MCP-exposed second.** The stable library
  API is the contract; MCP is one adapter over it. The generator kernel
  stays deterministic and untouched.
- **MCP beside the core.** A separate, default-off target (the default
  `anvil` build and the byte-identical `--artifact dut` contract are
  unaffected). Nothing in `src/gen/` learns about MCP.
- **Resources / Tools / Prompts** map to ANVIL:
  - *Resources* (read-only): the emitted `.sv`, the manifest, the
    `metrics`/`DesignMetrics`, the coverage facts, the effective config
    echo, and static catalogs (knob taxonomy, motif catalogue).
  - *Tools* (actions, mostly pure): `generate(seed, knobs, lane)`,
    `introspect(artifact)`, `dump_config`, `coverage_gaps`; then the
    controlled `validate(artifact, tools)` and `minimize(failing_seed,
    knobs)`.
  - *Prompts* (workflows): `find_downstream_bug`, `close_coverage_gap`,
    `minimize_reproducer`, `triage_tool_failures`, `explain_artifact`.
- **Structured queries, not bulk dumps.** Tools return structured
  metrics/coverage; large `.sv` and full manifests are exposed as
  *resources* the agent fetches deliberately.
- **Security model.** Read-only by default; `generate`/`introspect` are
  pure and side-effect-free (no FS writes without an explicit out-dir
  tool); `validate` shells external tools only through the **existing
  hardened `tool_matrix` invocations**, sandboxed to a project-root/tmp
  scope, resource-guarded (reuse `scripts/ram_guard.sh` + the
  `--max-rss-mb`/`--ram-abort-pct` governor), with no arbitrary shell
  exposed; every tool call carries a deterministic run id and the exact
  reproducible `(seed, knobs)` + command line, and is audit-logged.

### ANVIL-specific simplifications (where ANVIL beats the simulator case)

- **Determinism collapses the "service session" into a content-addressed
  cache.** A simulator needs an in-memory/service layer for stateful, fast,
  fine-grained queries. ANVIL does not: artifacts are pure functions of
  `(seed, knobs, lane, version)`, so caching is trivially sound and a
  simple in-process MCP server (stdio first) suffices — no gRPC service is
  required for correctness or performance.
- **ANVIL *is* the oracle, so introspection is construction-truth, not
  inference.** `DepSet`, motif/rule provenance, child-input binding
  provenance, and coverage facts are recorded by construction, so the agent
  reads ground truth instead of parsing emitted SV. This is the
  anti-archaeology principle already in `KNOWLEDGE_MAP_ARCHITECTURE.md`.

### Explicitly rejected / dropped (simulator-specific, do not copy)

- **A stateful simulator-style session API** (`run_until`, `force_signal`,
  waveform DB, signal-value-over-time, `explain_x`, sensitivity trees,
  interactive stepping). ANVIL has no temporal session; copying this would
  invent state ANVIL does not have.
- **MCP inside the kernel** — rejected; it would couple the deterministic
  core to an integration concern.
- **A second source of introspection truth** — rejected; the schema is
  *derived* from the existing `metrics`/`manifest`/`config`, never a
  parallel re-implementation that can drift.
- **The AI agent as a signoff oracle, or any API path that mutates/repairs
  output** — rejected; that violates rules-first / valid-by-construction.
  The agent drives experiments and explains; ANVIL stays the source of
  truth.
- **A raw-shell tool** — rejected; only fixed, vetted tool invocations.

### Phasing (the task tree leaves)

1. Design + this record (`.1`, docs-only).
2. Stable, versioned introspection schema spec, derived from existing
   facts (`.2`, docs).
3. Introspection emission surface (`anvil introspect` / structured JSON)
   (`.3`, code; additive, DUT byte-identical).
4. Read-only in-process MCP server: resources + pure/safe tools (`.4`,
   code; separate target).
5. Controlled `validate` + `minimize` tools (`.5`, code; sandboxed).
6. Agent-workflow prompts (`.6`, docs/config).
7. Book + USER_GUIDE + closeout (`.7`).

## Consequences

- A new capability lane exists with the same signoff bar as every other
  ANVIL lane; no code lands until a tree leaf owns it, and the design
  (`.1`/`.2`) is reviewable before implementation.
- The default `anvil` build, the `--artifact dut` byte-identical contract,
  rules-first/valid-by-construction, lane separation, and reproducibility
  are all invariants this lane must preserve.
- Honest scope: the lane exposes structural / provenance / coverage /
  resolved-facts (which ANVIL knows), never claimed whole-module "intended
  behavior" (which ANVIL by doctrine does not have).

## Open questions

- Transport: stdio MCP server first (local, matches Claude Code / Cursor);
  HTTP/service later if multi-client demand appears. Recommended: stdio.
- Crate layout: a separate `anvil-mcp` target vs a feature-gated module.
  Recommended: separate target so the default build stays unaffected.
- Whether `validate` belongs in the first MCP cut or stays CLI-only at
  first (agent shells `tool_matrix`). Recommended: ship read-only
  introspection + `generate` first; add the guarded `validate` as `.5`.
- Schema versioning policy for the introspection contract.

## Links

- Task-tree: `AGENT-INTROSPECTION-MCP.1`
- North star: `project_anvil_north_star` (auto-memory)
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter)
- Reuse: `src/bin/tool_matrix.rs`, `src/diff_sim/mod.rs`, `src/metrics.rs`,
  `src/manifest.rs`, `scripts/ram_guard.sh`, `src/mem_guard.rs`
