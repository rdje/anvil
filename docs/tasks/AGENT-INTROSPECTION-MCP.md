# AGENT-INTROSPECTION-MCP: Agent-Drivable Introspection + MCP Interface

## Metadata

- Tree ID: `AGENT-INTROSPECTION-MCP`
- Status: `active`
- Roadmap lane: `Capability — agent-drivable introspection + MCP interface`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow (owner-directed lane)

## Goal

Make ANVIL agent-drivable for the downstream-bug-hunting loop by exposing
deep *construction-truth* introspection and control through a stable,
versioned, default-off API, with a thin read-mostly MCP adapter **beside**
the generator core. The closing capability is the autonomous loop:
generate → validate (Verilator / Yosys / iverilog / `--diff-sim`) → on a
tool failure shrink `(seed, knobs)` to a minimal reproducer → emit it.

Architecture and the transferred-vs-dropped reference-advice analysis are
recorded in
[`docs/decisions/0004-agent-introspection-mcp-lane.md`](../decisions/0004-agent-introspection-mcp-lane.md).

## Non-Goals

- **No stateful simulator-style session API** (`run_until`, `force_signal`,
  waveform DB, signal-over-time, `explain_x`, sensitivity trees,
  interactive stepping). ANVIL is a pure `(seed, knobs) -> artifact`
  function plus pure post-hoc analysis; it has no temporal session.
- No MCP logic inside the generator kernel; no new generation path; no
  generate-then-filter or output mutation/repair via the API.
- The AI agent is never a signoff oracle — ANVIL's manifests/metrics/tool
  results remain the source of truth.
- No second source of introspection truth (the schema is derived from
  existing `metrics`/`manifest`/`config`).
- No arbitrary-shell tool; no change to the default `--artifact dut`
  byte-identical contract.

## Acceptance Criteria

- A stable, versioned introspection schema exists, derived from existing
  `metrics`/`manifest`/`config` (no parallel re-implementation).
- A default-off MCP adapter (separate target) exposes resources
  (`.sv`/manifest/metrics/coverage/config + knob & motif catalogs) and a
  safe tool set (`generate`/`introspect`/`dump_config`/`coverage_gaps`)
  with deterministic run ids.
- A controlled `validate`/`minimize` tool set runs external tools only
  through the existing hardened `tool_matrix` invocations, sandboxed and
  resource-guarded.
- Agent workflows (find downstream bug, close coverage gap, minimize
  reproducer, triage failures, explain artifact) are exercised end-to-end.
- DUT lane stays byte-identical (snapshots 6/6); book + USER_GUIDE document
  the lane; downstream/self-checks clean.

## Task Tree

- ID: `AGENT-INTROSPECTION-MCP`
  Status: `active`
  Goal: `Agent-drivable introspection + MCP interface beside the core.`
  Children: `.1`, `.2`, `.3`, `.4`, `.5`, `.6`, `.7`

- ID: `AGENT-INTROSPECTION-MCP.1`
  Status: `done`
  Goal: `Design the lane + land decision record 0004 (architecture, transferred-vs-dropped reference advice, guardrails, phasing).`
  Acceptance: `docs/decisions/0004 + DEVELOPMENT_NOTES design note + this tree landed; no code; doctrine guardrails explicit.`
  Verification: `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check`.
  Commit: `AGENT-INTROSPECTION-MCP.1 - design + decision record 0004`

- ID: `AGENT-INTROSPECTION-MCP.2`
  Status: `pending`
  Goal: `Specify the stable, versioned introspection JSON schema, derived strictly from existing metrics/manifest/config; map each field to its existing source. Docs-only.`
  Acceptance: `Schema spec doc lists every field + provenance; confirms zero new computed truth; versioning policy stated.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-INTROSPECTION-MCP.3`
  Status: `pending`
  Goal: `Implement the introspection emission surface (anvil introspect / structured JSON dump) over the .2 schema. Additive, default-off-equivalent, DUT byte-identical.`
  Acceptance: `New surface emits the .2 schema from existing facts; snapshots 6/6; no change to existing stdout/manifest/CLI defaults.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-INTROSPECTION-MCP.4`
  Status: `pending`
  Goal: `Implement the read-only in-process MCP server (separate target): resources + pure/safe tools (generate/introspect/dump_config/coverage_gaps); deterministic run ids; content-addressed artifact cache; no external-tool exec.`
  Acceptance: `Server lists resources + safe tools; round-trips a generate+introspect; default anvil build unaffected; DUT byte-identical.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-INTROSPECTION-MCP.5`
  Status: `pending`
  Goal: `Add the controlled validate + minimize tools: external tools only via existing tool_matrix invocations, sandboxed + ram-guarded; minimize shrinks (seed, knobs); audit log + reproducible command line per call.`
  Acceptance: `validate returns structured tool reports; minimize produces a smaller failing (seed, knobs); security guardrails enforced + tested.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-INTROSPECTION-MCP.6`
  Status: `pending`
  Goal: `Package the agent-workflow prompts (find_downstream_bug, close_coverage_gap, minimize_reproducer, triage_tool_failures, explain_artifact).`
  Acceptance: `Each prompt drives its tool chain end-to-end on a sample.`
  Verification: `pending`
  Commit: `pending`

- ID: `AGENT-INTROSPECTION-MCP.7`
  Status: `pending`
  Goal: `Book chapter + USER_GUIDE section + CODEBASE_ANALYSIS update + closeout.`
  Acceptance: `mdBook documents the lane; USER_GUIDE shows invocation; live docs synced; tree closed.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `AGENT-INTROSPECTION-MCP.1` | `done` | Design + decision record landed this commit. |
| 2 | `AGENT-INTROSPECTION-MCP.2` | `pending` | Schema spec (docs) is the contract the code leaves stand on; design-first. |

`.3`–`.7` stay `pending`; `.1`/`.2` design may re-split `.3`–`.5` once the
schema and transport are pinned. Implementation leaves (`.3`+) are **code**
and require the design (`.1`/`.2`) to be accepted by the owner first.

## Decisions

- `2026-06-14`: Architecture, the transferred-vs-dropped reference-advice
  analysis, the security model, and the determinism→content-addressed-cache
  simplification are recorded in
  [`docs/decisions/0004`](../decisions/0004-agent-introspection-mcp-lane.md)
  and `DEVELOPMENT_NOTES.md`. Summary: MCP is a thin read-mostly adapter
  beside a deterministic core; the introspection schema is derived from
  existing facts; ANVIL needs no stateful simulator-style session.
- `2026-06-14`: Design-first cadence — `.1`/`.2` are docs; no code until
  the schema/architecture is accepted.

## Open Questions

- Transport: stdio MCP server first vs HTTP/service later (recommend stdio).
- Crate layout: separate `anvil-mcp` target vs feature-gated module
  (recommend separate target).
- Whether `validate` ships in the first MCP cut or stays CLI-only initially
  (recommend read-only introspection + `generate` first; guarded `validate`
  at `.5`).
- Introspection-schema versioning policy.

## Blockers

- None. Implementation leaves (`.3`+) are gated on owner acceptance of the
  `.1`/`.2` design, not on a technical blocker.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `AGENT-INTROSPECTION-MCP.1` | `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `AGENT-INTROSPECTION-MCP.1` | `AGENT-INTROSPECTION-MCP.1 - design + decision record 0004` | Pending hash; opens the tree. |

## Changelog

- `2026-06-14`: Created the tree; landed `.1` design + decision record 0004;
  frontier advanced to `.2` (schema spec).
