---
id: agent-mcp-expansion-surface
title: Broaden the read-mostly agent/MCP surface by projecting recorded facts, routing non-DUT lanes, and adding an optional HTTP transport
answers:
  - "how does an ANVIL agent ask what coverage is not yet exercised"
  - "how are downstream coverage gaps surfaced over MCP"
  - "does the coverage_gaps MCP tool recompute coverage"
  - "where does the coverage_gaps MCP tool get its data"
  - "can the ANVIL MCP server generate microdesign or frontend artifacts"
  - "how do non-DUT lanes reach the MCP interface"
  - "does anvil-mcp support an HTTP transport"
  - "is the anvil-mcp HTTP transport on by default"
  - "what invariants constrain the AGENT-MCP-EXPANSION lane"
date: 2026-06-15
status: current
tags: [mcp, agent, coverage, transport, architecture, introspection]
evidence: docs/decisions/0005-agent-mcp-expansion-surface.md; docs/tasks/AGENT-MCP-EXPANSION.md; src/mcp/mod.rs; src/bin/tool_matrix.rs; src/umbrella/mod.rs
---

# 0005 - Broaden the read-mostly agent/MCP surface: project recorded facts, route non-DUT lanes, add optional HTTP transport

- Date: 2026-06-15
- Status: accepted
- Tags: mcp, agent, coverage, transport, architecture, introspection

## Context

Decision [`0004`](0004-agent-introspection-mcp-lane.md) landed the
read-mostly agent/MCP lane (closed tree `AGENT-INTROSPECTION-MCP`,
`2026-06-15`): a default-off `anvil-mcp` stdio JSON-RPC server exposing
pure tools (`generate`/`introspect`/`dump_config`), controlled tools
(`validate`/`minimize` over the hardened `src/downstream/` allow-list),
resources, and five workflow prompts. `0004` deliberately deferred three
breadth items as owner-gated follow-ups, now owned by the
[`AGENT-MCP-EXPANSION`](../tasks/AGENT-MCP-EXPANSION.md) tree:

1. expose downstream **coverage gaps** as an MCP tool (an agent asks
   "what is not yet exercised?" and drives generation at it — the
   `close_coverage_gap` prompt already references the dark surfaces);
2. drive the **non-DUT lanes** (`microdesign`, `frontend`) over MCP, so
   the agent can generate/introspect all three artifact families;
3. add an optional **HTTP transport** beside stdio.

This record is the `AGENT-MCP-EXPANSION.1` design/decision leaf: it
re-confirms the lane invariants, locates where coverage gaps are computed,
decides the read-only exposure path for each item, and finalizes the
`.2`–`.5` decomposition (no source change).

### Where coverage gaps live today

`CoverageSummary` (`src/bin/tool_matrix.rs:286`) and
`compute_coverage_gaps` (`src/bin/tool_matrix.rs:6552`) are **private to
the `tool_matrix` binary**; they are not in the library, so neither
`src/mcp/` nor `src/lib.rs` can call them. The crucial fact is that the
serialized `MatrixReport` (the `tool_matrix_report.json` artifact) already
carries `coverage: CoverageSummary` and the **already-computed**
`coverage_gaps: Vec<String>` (both `Serialize`,
`src/bin/tool_matrix.rs:488-489`). The gap list is therefore a recorded
fact, not something the MCP server must (or should) re-derive.

### Lane invariants re-confirmed against current code

- **Adapter beside the core.** `src/mcp/mod.rs` is a library module whose
  `McpServer::handle` is a pure `Value -> Option<Value>` dispatcher;
  `src/bin/anvil_mcp.rs` is the stdio shell. Nothing in `src/gen/` knows
  about MCP. ✔
- **SCHEMA-DERIVED / no new computed truth.** Every tool/resource payload
  is a serde projection of an existing `Config`/`Metrics`/`DesignMetrics`/
  manifest/recorded-report struct. ✔
- **Controlled tools only via hardened `downstream`.** `validate`/
  `minimize` run `verilator`/`yosys`/`iverilog` only through the fixed
  allow-list, sandboxed temp dir, RAM-guarded, audit-logged; no arbitrary
  shell/path. ✔
- **Agent is never a signoff oracle; `minimize` searches the input
  `(seed, knobs)` space**, never mutating/repairing RTL. ✔
- **Default `--artifact dut` byte-identical**; reuse `tool_matrix` /
  `downstream` / `diff_sim` / `metrics` / `mem_guard` rather than forking
  logic. ✔

## Decision

Each expansion item is implemented as a strictly read-mostly, default-off
addition that preserves every invariant above.

### `.2` — coverage gaps as a PURE tool that PROJECTS a recorded report

Surface coverage gaps as a **pure MCP tool over a recorded
`tool_matrix_report.json`**, not as a controlled tool that recomputes a
matrix subset on demand. The tool accepts the recorded report **inline**
(`report`: object — the zero-filesystem default) **or by path**
(`report_path`: string), and returns a thin projection of the recorded
facts: the `scenario_set`, scenario/module counts, `artifact_kind`, the
`coverage_gaps` array (+ a `gap_count`), the downstream `tool_summary`
pass/fail, and the selected dark coverage facts.

The projection reads known JSON keys via `serde_json::Value` rather than
mirroring the bin-private `CoverageSummary` struct field-by-field. That
struct grows on nearly every hierarchy slice (it already has ~150 fields);
mirroring it into `src/mcp/` would be a maintenance landmine and a second
source of truth. A loose key projection is robust to that growth and keeps
the MCP side decoupled.

This preserves: **read-only** (a file/inline read — no generation, no tool
spawn, no recompute), **no new computed truth** (`compute_coverage_gaps`
already ran at record time inside `tool_matrix`; the tool only relays its
output), **DUT byte-identical** (no generator-core touch), and **single
source of truth** (the one gap computation stays in `tool_matrix`).

**Rejected — a controlled tool that runs a matrix subset on demand.** It
would compute coverage state on demand (a second runtime path to the gap
list that can drift from `compute_coverage_gaps`), turn a read-only query
into a heavy tool-spawning controlled action, and pull matrix logic toward
`src/mcp/`. It loses on every invariant the pure-projection path keeps.
(The reading of an agent-supplied report path is a plain file read with no
process execution, categorically distinct from the controlled tools'
"no arbitrary path/shell" guardrail, which is about not spawning arbitrary
binaries; the inline form needs no filesystem at all.)

### `.3` — non-DUT lanes over MCP, routed through the umbrella

Route `generate`/`introspect` through the umbrella `ArtifactLane`
dispatch (`src/umbrella/`, which already carries `DutLane`/
`MicrodesignLane`/`FrontendLane`) keyed by a `lane` argument that defaults
to `dut`. Today `build_artifact` (`src/mcp/mod.rs:538`) is DUT-only and the
`generate` summary hardcodes `"lane": "dut"`; `.3` generalizes that path.

Because each non-DUT lane already has its own expected-facts manifest, the
non-DUT introspection document must remain a **serde projection of that
existing manifest** — not a new computed document. Whether each lane has a
ready projection or one must be defined is itself an unresolved design
choice, so **`.3` splits into `.3a` (design) + `.3b` (implementation)**,
mirroring how the original lane split `.5` into `.5.1/.5.2/.5.3`.

### `.4` — optional HTTP transport beside stdio (stdio remains default)

`McpServer::handle` is already transport-agnostic (`handle_line` does only
the JSON string round-trip). `.4` adds an HTTP transport that drives the
**same** `handle` dispatcher behind an explicit opt-in flag; **stdio stays
the default**, so no existing invocation changes. Security note for the
`.4` design: HTTP would expose the controlled `validate`/`minimize` tools
over a socket, so the transport must **bind loopback-only by default** and
the per-call `downstream` guardrails (allow-list, sandbox, RAM guard,
audit log) continue to apply unchanged.

### `.5` — closeout

Sync `book/src/agent-mcp.md` + `USER_GUIDE.md` + `README.md` to the
expanded surface; close the tree.

## Consequences

- The agent gains a "what is unexercised?" query and full
  three-lane generate/introspect, plus an optional networked transport —
  strictly richer, still read-mostly, still default-off.
- The single coverage-gap computation stays in `tool_matrix`; the MCP tool
  is a relay, so the two cannot drift.
- The default `anvil` build and `--artifact dut` byte-identical contract,
  rules-first/valid-by-construction, lane separation, and reproducibility
  remain invariants every leaf preserves.
- Honest scope unchanged from `0004`: the lane exposes structural /
  provenance / coverage / resolved-facts (which ANVIL knows), never
  claimed whole-module "intended behavior".

## Open questions

- `.3a` decides: does each non-DUT lane already expose a manifest the
  introspection layer can project verbatim, or must a thin projection be
  defined per lane? (Bias: reuse the existing manifest; define nothing new
  that can drift.)
- `.4` decides: a sub-flag on `anvil-mcp` (`--http <addr>`) vs a separate
  bin. Bias: a flag on the existing bin, loopback default, so the default
  build/stdio path is untouched.
- Whether `.2` returns the dark coverage facts as a fixed projection or
  echoes the whole recorded `coverage` object. Bias: a thin, named
  projection plus the raw `coverage_gaps` array.

## Correction (2026-06-15, during `.3b` implementation)

The `.3a` design (recorded in the task tree) initially planned to surface
the non-DUT lane manifest **only** via the `artifact.manifest` ResourceRef
(not inlined), reasoning that inlining "would bump the schema / violate
§6.6". Implementing `.3b` against the schema *spec* showed this misread the
contract: `docs/AGENT_INTROSPECTION_SCHEMA.md` §5/§6.5 **already define**
inlined `microdesign_manifest` / `frontend_manifest` payload sections **at
v1.0** (they are "small and stable", an exact serde projection of each
lane's `Manifest`); §6.6's "resource, not inlined" rule applies only to the
**bulk `.sv`**. So `.3b` conforms to the schema: it **inlines** the manifest
in the `introspection` payload under the schema key **and** sets the
`artifact.manifest` ResourceRef (§4), both derived from the one
`emit_manifest` output (no drift). This is still **no `schema_version`
bump** — the sections existed at v1.0; populating them is conformance, not
extension. Lesson: a design leaf must check the schema *spec*, not only the
code, before deciding the wire shape. The high-level decision above
(route through the umbrella; manifest stays a serde projection; default
`dut` byte-identical) is unchanged.

## Links

- Task-tree: `AGENT-MCP-EXPANSION.1` (this leaf); frontier advances to `.2`
- Predecessor: decision [`0004`](0004-agent-introspection-mcp-lane.md)
  (`agent-introspection-mcp-lane`)
- North star: `project_anvil_north_star` (auto-memory)
- Doctrine: `feedback_rules_first_generation` (no generate-then-filter)
- Reuse: `src/bin/tool_matrix.rs` (recorded `coverage_gaps`),
  `src/umbrella/mod.rs` (`ArtifactLane`), `src/mcp/mod.rs`
  (`McpServer::handle`), `src/downstream/` (controlled tools)
