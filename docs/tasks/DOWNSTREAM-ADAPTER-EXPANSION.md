# DOWNSTREAM-ADAPTER-EXPANSION: a generic adapter interface + more tool columns

## Metadata

- Tree ID: `DOWNSTREAM-ADAPTER-EXPANSION`
- Status: `active`
- Roadmap lane: `Usability / breadth — more downstream tool reach (north star, idea 3)`
- Created: `2026-06-17`
- Last updated: `2026-06-17`
- Owner: repo-local workflow

## Goal

Widen ANVIL's downstream reach by making the **acceptance-column axis pluggable**:
a generic downstream-tool adapter interface so new tools plug in as acceptance
columns with little new core code. Add adapters beyond today's
Verilator / Yosys / Icarus — candidates: **slang**, **sv2v**, **Surelog/UHDM**,
and a generic wrapper for commercial/other tools. Each new tool widens the
bug-surface (more parsers/elaborators to trip), and each adapter is API-selectable
with its results queryable over MCP. Builds on the hardened
`src/downstream/` allow-list + the `tool_matrix` column model.

## Non-Goals

- No vendoring/bundling of the tools — adapters shell out to external,
  allow-listed, sandboxed, RAM-guarded binaries (decision `0004`).
- No behavioural oracle; adapters report acceptance/lint/synth verdicts (and,
  where applicable, parity/AST facts), not behaviour.
- No new generator semantics; default DUT output stays byte-identical.

## Acceptance Criteria

- A generic adapter trait/registry exists and at least one new real adapter
  (e.g. `slang` or `sv2v`) is integrated as an acceptance column, warning-clean
  on the ANVIL corpus (or its divergences retained as reproducers).
- **API-completeness gate (decision `0017`):** adapters are selectable via the
  MCP/config API (not CLI-only), and each adapter's per-artifact verdict is
  queryable via the MCP/introspection API; the CLI/`tool_matrix` flags are shims
  over the same surface.
- Each adapter preserves the allow-list + sandbox + RAM-guard + `anvil://audit/log`
  discipline; absent tools are a friendly no-op (the existing `tools_present()`
  precedent), not a hard failure.
- Default-off / DUT byte-identical; documented in `book/src/agent-mcp.md` +
  README + USER_GUIDE; committed through `COMMIT.md`.

## Task Tree

- ID: `DOWNSTREAM-ADAPTER-EXPANSION`
  Status: `active`
  Goal: `A generic, API-selectable downstream-adapter interface + new tool columns (slang / sv2v / Surelog-UHDM / commercial wrappers), reusing the hardened src/downstream allow-list + the tool_matrix column model.`
  Children: `DOWNSTREAM-ADAPTER-EXPANSION.1`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.1`
  Status: `pending`
  Goal: `Design/decision leaf (ADR, no code): pin the adapter trait/registry shape (how run_verilator/run_yosys/run_iverilog generalize to a pluggable Adapter with a uniform verdict + optional parity/AST hook), the allow-list/sandbox extension discipline, the first new adapter to land (slang or sv2v — whichever is locally installable + most parser-distinct), and the MCP/config selection + query surface (decision 0017 API-completeness). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the adapter interface, the first new tool, and the MCP selection/query surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Verification: `pending`
  Commit: `pending`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2`
  Status: `pending`
  Goal: `Implement the .1 design: the Adapter trait/registry + the first new adapter as a column + MCP/config selection + proofs + a real-tool gate (or a clean absent-tool no-op) + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split when picked (one leaf per added adapter).`
  Acceptance: `pending (set at .1)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DOWNSTREAM-ADAPTER-EXPANSION.1` | `pending` | Design-first ADR pins the pluggable adapter interface + the MCP selection/query surface (decision `0017`) before any code; each later adapter lands as its own `.2+` leaf. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability/breadth lane (idea 3).
  Binds decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md).
  Generalizes the existing fixed Verilator/Yosys/Icarus surface into a pluggable,
  API-selectable adapter registry; design-first ADR before code.

## Open Questions

- Adapter scope per tool: pure accept/reject vs. richer parity/AST extraction
  (e.g. `slang` JSON AST, like the Verilator JSON-AST frontend parity gate).
  *(Resolved per-adapter; the trait must allow both. Decided at `.1`.)*
- Which adapter first depends on local availability (`slang` was absent in the
  Phase-8 environment). *(Resolved at `.1` against the live toolchain.)*

## Blockers

- None. (Feeds `BUG-HUNT-ORCHESTRATION` + `ACCEPTANCE-DIVERGENCE-HUNTING` with
  more columns; not blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `DOWNSTREAM-ADAPTER-EXPANSION` | `tree registered (docs-only); no code` | `registered` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DOWNSTREAM-ADAPTER-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
