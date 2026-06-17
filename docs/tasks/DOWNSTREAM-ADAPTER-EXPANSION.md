# DOWNSTREAM-ADAPTER-EXPANSION: a generic adapter interface + more tool columns

## Metadata

- Tree ID: `DOWNSTREAM-ADAPTER-EXPANSION`
- Status: `active`
- Roadmap lane: `Usability / breadth — more downstream tool reach (north star, idea 3)`
- Created: `2026-06-17`
- Last updated: `2026-06-17` (`.1` design ADR done — decision `0020`; `.2` pre-split)
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
  Children: `DOWNSTREAM-ADAPTER-EXPANSION.1`, `DOWNSTREAM-ADAPTER-EXPANSION.2`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.1`
  Status: `done`
  Goal: `Design/decision leaf (ADR, no code): pin the adapter trait/registry shape (how run_verilator/run_yosys/run_iverilog generalize to a pluggable Adapter with a uniform verdict + optional parity/AST hook), the allow-list/sandbox extension discipline, the first new adapter to land (slang or sv2v — whichever is locally installable + most parser-distinct), and the MCP/config selection + query surface (decision 0017 API-completeness). Record as the next decision record + pre-split .2 (impl).`
  Acceptance: `A decision record + a tree/DEVELOPMENT_NOTES entry pinning the adapter interface, the first new tool, and the MCP selection/query surface; docs-only; INDEX + this tree + docs/TASK_TREE.md updated.`
  Result: `Decision 0020 written — a CLOSED, compile-time Adapter registry over the ONE run_tool runner + the ONE tool_verdict classifier (no second runner/classifier); the trait carries only argv + warning-detection + an optional SCHEMA-DERIVED extract_facts hook. Built-ins re-expressed byte-identically (AcceptanceTool not retired). First adapter = sv2v (.2b, minimal accept/reject transpile column); second = slang (.2c, the JSON-AST fact hook). Live-toolchain probe: slang/sv2v/surelog all ABSENT ⇒ first cuts ship structural + friendly absent-tool no-op + #[ignore] real-tool gate. API-completeness (0017): adapters selectable via the existing tools arg + queryable via the existing reports + a new SCHEMA-DERIVED adapter-catalog projection; CLI a shim. Allow-list/sandbox/RAM-guard/audit (0004) + the 0019.2f caller-supplied-binary library-only boundary preserved; default-off / DUT byte-identical.`
  Verification: `docs-only / DUT byte-identical (no src/). decision 0020 + INDEX row + this tree + docs/TASK_TREE.md + DEVELOPMENT_NOTES; check_memory_architecture + KM gen/check green; mdbook build clean.`
  Commit: `DOWNSTREAM-ADAPTER-EXPANSION.1`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2`
  Status: `active`
  Goal: `Implement the .1 design (decision 0020): the closed Adapter trait/registry + the adapter-catalog query + the first new adapters as columns + MCP/config selection + proofs + a real-tool gate (or a clean absent-tool no-op) + book/USER_GUIDE/README/KM. Default-off / DUT byte-identical. Pre-split per decision 0020.`
  Children: `DOWNSTREAM-ADAPTER-EXPANSION.2a`, `DOWNSTREAM-ADAPTER-EXPANSION.2b`, `DOWNSTREAM-ADAPTER-EXPANSION.2c`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2a`
  Status: `pending`
  Goal: `The registry refactor + the catalog query: introduce the Adapter trait / AdapterSpec + the closed adapters() registry; re-express Verilator/Yosys/Icarus as the first three registered adapters with byte-identical id/argv/warning-detection; route validate / validate_tool_specs / the tool_matrix columns / AcceptanceTool::from_name through the registry; add the SCHEMA-DERIVED adapter-catalog query/resource (decision 0017 discoverability).`
  Acceptance: `Pure refactor + the catalog; built-in ToolInvocation.tool labels/argv unchanged; banked tool_matrix reports + --resume checkpoints byte-identical; snapshots 6/6; default-off / DUT byte-identical; cargo test/clippy/fmt green.`
  Verification: `pending`
  Commit: `pending`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2b`
  Status: `pending`
  Goal: `The first new adapter, sv2v, as an accept/reject transpile column: registered descriptor + tools-selectable + queryable verdict in ValidateReport/DivergenceReport/the matrix column; friendly absent-tool no-op + an #[ignore] real-tool gate; book/USER_GUIDE/README/KM card.`
  Acceptance: `pending (refine at pick)`
  Verification: `pending`
  Commit: `pending`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2c`
  Status: `pending`
  Goal: `The second new adapter, slang, with the optional extract_facts hook (JSON-AST), proving the trait's richer SCHEMA-DERIVED path; absent-tool no-op + #[ignore] gate; docs. (surelog/UHDM + a generic commercial-wrapper adapter are future .2d+ leaves.)`
  Acceptance: `pending (refine at pick)`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DOWNSTREAM-ADAPTER-EXPANSION.2a` | `pending` | The registry refactor + the adapter-catalog query land first (decision `0020`): the built-ins re-expressed byte-identically through the closed `Adapter` registry, so every later adapter (`.2b` sv2v, `.2c` slang) is one self-contained descriptor. Pure refactor / DUT byte-identical. |
| 2 | `DOWNSTREAM-ADAPTER-EXPANSION.2b` | `pending` | `sv2v` — the minimal accept/reject transpile column proving the trait end-to-end (absent locally ⇒ no-op + `#[ignore]` gate). |
| 3 | `DOWNSTREAM-ADAPTER-EXPANSION.2c` | `pending` | `slang` — the richer adapter landing the optional JSON-AST `extract_facts` hook. |

## Decisions

- `2026-06-17`: Registered as an owner-directed usability/breadth lane (idea 3).
  Binds decision [`0017`](../decisions/0017-api-first-everything-mcp-accessible.md).
  Generalizes the existing fixed Verilator/Yosys/Icarus surface into a pluggable,
  API-selectable adapter registry; design-first ADR before code.
- `2026-06-17` (`.1`): decision
  [`0020`](../decisions/0020-downstream-adapter-interface.md) accepted. The adapter
  surface is a **closed, compile-time `Adapter` registry** over the one `run_tool`
  runner + the one `tool_verdict` classifier — *not* a runtime plugin and *not* an
  agent-supplied command, so the decision-`0004` fixed-allow-list holds. The trait
  carries only argv (module/design) + the warning predicate + an optional
  SCHEMA-DERIVED `extract_facts` hook. Built-ins re-expressed byte-identically
  (`AcceptanceTool` not retired; the `"verilator"`/`yosys-<mode>`/`iverilog-compile`
  labels are a hard byte-identical constraint for banked reports + `--resume`).
  Because the verdict is unchanged, every added column becomes a new comparable
  verdict in `divergence::run` + a new selectable tool in `hunt`/`validate` for
  free. **First adapter = `sv2v`** (`.2b`, minimal accept/reject transpile column);
  **second = `slang`** (`.2c`, the JSON-AST fact hook). API-completeness (`0017`):
  adapters selectable via the existing `tools` arg + queryable via the existing
  reports + a new SCHEMA-DERIVED **adapter-catalog** projection; CLI a shim.
  `.2` pre-split into `.2a` (registry refactor + catalog) / `.2b` (sv2v) / `.2c`
  (slang); `.2d+` (surelog/UHDM, commercial-wrapper) future.

## Open Questions

- ~~Adapter scope per tool: pure accept/reject vs. richer parity/AST extraction.~~
  **Resolved (`.1`, decision `0020`):** the trait carries an **optional**
  `extract_facts` hook, so both shapes are first-class — `sv2v` lands the pure
  accept/reject column (`.2b`), `slang` lands the richer JSON-AST hook (`.2c`).
- ~~Which adapter first depends on local availability.~~ **Resolved (`.1`):**
  live-toolchain probe found `slang`/`sv2v`/`surelog` all **absent** (only
  verilator/yosys/iverilog present). `sv2v` is chosen first on minimal-surface +
  parser-distinctness grounds; absent tools land structural + friendly no-op +
  `#[ignore]` real-tool gate (the `sv_version_downstream` / `hunt_e2e` /
  `divergence_e2e` precedent), upgraded to a banked proof once installed.

## Blockers

- None. (Feeds `BUG-HUNT-ORCHESTRATION` + `ACCEPTANCE-DIVERGENCE-HUNTING` with
  more columns; not blocked by them.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `DOWNSTREAM-ADAPTER-EXPANSION` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-17` | `DOWNSTREAM-ADAPTER-EXPANSION.1` | `docs-only / DUT byte-identical (no src/); decision 0020 + INDEX + tree + TASK_TREE.md + DEVELOPMENT_NOTES; live-toolchain probe (slang/sv2v/surelog absent); check_memory_architecture + KM gen/check green; mdbook build clean` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DOWNSTREAM-ADAPTER-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `DOWNSTREAM-ADAPTER-EXPANSION.1` | `DOWNSTREAM-ADAPTER-EXPANSION.1 — adapter-interface ADR (decision 0020)` | Design ADR; pre-split `.2` → `.2a`/`.2b`/`.2c`; frontier advances to `.2a`. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-17`: `.1` design ADR done — decision `0020` (closed compile-time `Adapter`
  registry; `sv2v` first, `slang` second; API-completeness via the existing `tools`
  arg + the new adapter-catalog projection). `.2` pre-split into `.2a`/`.2b`/`.2c`;
  frontier advanced to `.2a`.
