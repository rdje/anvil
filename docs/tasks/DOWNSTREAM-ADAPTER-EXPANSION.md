# DOWNSTREAM-ADAPTER-EXPANSION: a generic adapter interface + more tool columns

## Metadata

- Tree ID: `DOWNSTREAM-ADAPTER-EXPANSION`
- Status: `active`
- Roadmap lane: `Usability / breadth — more downstream tool reach (north star, idea 3)`
- Created: `2026-06-17`
- Last updated: `2026-06-18` (`.2b.1` done — sv2v downstream adapter + MCP selectability/discoverability; frontier `.2b.2` [tool_matrix column + real-tool gate + docs])
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
  Status: `active`
  Goal: `The registry refactor + the catalog query: introduce the Adapter trait + the closed adapters() registry; re-express Verilator/Yosys/Icarus as the first three registered adapters with byte-identical id/argv/warning-detection; route the orchestrators through the registry; add the SCHEMA-DERIVED adapter-catalog query/resource (decision 0017 discoverability). Split (refining decision 0020's .2a) into the downstream core, the catalog query, and the orchestrator routing so each sub-slice is provably byte-identical.`
  Children: `DOWNSTREAM-ADAPTER-EXPANSION.2a.1`, `DOWNSTREAM-ADAPTER-EXPANSION.2a.2`, `DOWNSTREAM-ADAPTER-EXPANSION.2a.3`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2a.1`
  Status: `done`
  Goal: `The downstream library core: a pub trait Adapter { id, binary, run(&AdapterRunCx)->Vec<ToolInvocation> } + AdapterRunCx/AdapterTarget + three built-in unit-struct adapters (Verilator/Yosys/Icarus) whose run() delegates VERBATIM to the existing run_verilator/run_yosys/run_iverilog_compile (+ _design), a closed pub fn adapters() registry, and AcceptanceTool::adapter() mapping the enum into it (enum stays the canonical built-in identity — not retired). Refactor downstream::validate to dispatch via tool.adapter().run(&cx) instead of the hard-coded match. Scope: src/downstream/mod.rs only.`
  Acceptance: `validate emits byte-identical ToolInvocations (same labels/argv/order; Yosys Both still 2 rows; mem-guard checked once per selected tool); snapshots 6/6; lib proofs (registry has the 3 built-ins with expected ids/binaries; AcceptanceTool::adapter round-trips; validate-through-adapter shape); cargo check/test --lib/clippy/fmt green; default-off / DUT byte-identical. The optional extract_facts hook lands at .2c (slang); validate_tool_specs + tool_matrix routing is .2a.3.`
  Result: `Landed in src/downstream/mod.rs: pub trait Adapter: Sync { id, binary, run(&AdapterRunCx)->Result<Vec<ToolInvocation>> } + AdapterRunCx{binary,out_dir,target,yosys_mode,language} + AdapterTarget{Module,Design}(Copy) + 3 built-in unit-struct adapters delegating verbatim to run_* + static ADAPTER_REGISTRY/pub fn adapters() + AcceptanceTool::adapter(). validate refactored to tool.adapter().run(&cx) (byte-identical). The Adapter:Sync supertrait makes the static registry valid (E0515 fix). +2 lib proofs.`
  Verification: `cargo check --all-targets clean; cargo test --lib 545/0 (+2: adapter_registry_holds_the_three_builtins, acceptance_tool_maps_to_its_registered_adapter); snapshots 6/6 byte-identical; tool_matrix 75/0; anvil 12/0; clippy -D warnings clean; fmt --check clean; DUT byte-identical (umbrella byte-identical tests + snapshots). RAM-guarded heavy steps (decision 0003).`
  Commit: `DOWNSTREAM-ADAPTER-EXPANSION.2a.1`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2a.2`
  Status: `done`
  Goal: `The SCHEMA-DERIVED adapter-catalog discoverability surface (decision 0017): project the closed adapters() registry as { id, binary, present (a PATH --version probe), supports_facts } — surfaced as an MCP resource (anvil://catalog/adapters) and/or a pure query, plus the introspection/schema touch if any; book/USER_GUIDE. So an agent can discover which tools exist and which are installed over the API alone.`
  Result: `Landed the new MCP resource anvil://catalog/adapters. downstream gains a defaulted Adapter::supports_facts() (built-ins false; slang overrides at .2c) + a serializable AdapterInfo{id,binary,present,supports_facts} + pub fn adapter_catalog() projecting adapters() with a live tool_version() PATH probe for present. mcp resources_list advertises it + resources_read serves { "adapters": [...] }. No introspection SCHEMA_VERSION bump (a new resource, not a new introspection field). Book: api-resources-prompts.md static-resource table row + agent-mcp.md resource list. +1 mcp proof.`
  Verification: `cargo test --lib 546/0 (+1: adapter_catalog_resource_lists_the_registry); snapshots 6/6 byte-identical; clippy -D warnings clean; fmt --check clean; mdbook build clean; book_examples 3/3. DUT byte-identical (no generator/introspection-schema change). RAM-guarded.`
  Commit: `DOWNSTREAM-ADAPTER-EXPANSION.2a.2`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2a.3`
  Status: `done`
  Goal: `Route validate_tool_specs (version axis, single-yosys-mode, caller-supplied binary) and the tool_matrix per-unit invocation (run_module_tools/run_design_tools) through the registry, keeping the fixed ModuleReport/DesignReport columns + banked reports + --resume byte-identical. This is the bridge that makes adding a new adapter column (sv2v at .2b) a near-one-line registry add.`
  Acceptance: `(1) downstream::run_tool_spec dispatches via spec.kind.adapter().run(&AdapterRunCx{..}) instead of the hard-coded match spec.kind, byte-identical: each spec still yields exactly one ToolInvocation, the Yosys version axis still collapses Both -> WithoutAbc to a single row, and the relabel + tool_version stamp stay in validate_tool_specs. (2) tool_matrix run_module_tools/run_design_tools build one AdapterTarget (Module/Design) + per-column AdapterRunCx and dispatch each fixed column through AcceptanceTool::{Verilator,Yosys,Iverilog}.adapter().run(&cx); the verilator/yosys/iverilog_compile columns, skip flags, verilator_only no-op, --language selector, and Yosys-mode row count are byte-identical, so banked reports + --resume + snapshots are untouched. (3) the now-unused run_* primitive imports are dropped from tool_matrix.rs (clippy -D warnings clean). (4) a downstream lib proof asserts the per-kind single-row routing incl. the Yosys Both->single collapse. Gate: cargo check --all-targets; cargo test --lib (+1 proof); snapshots 6/6; tool_matrix tests; clippy -D warnings; fmt --check; default-off / DUT byte-identical.`
  Result: `Landed the registry routing for the two remaining downstream callers, both byte-identical. (a) src/downstream/mod.rs: run_tool_spec replaced its hard-coded match spec.kind with one AdapterTarget (Module/Design) + AdapterRunCx{binary=spec.binary, yosys_mode=single (Both->WithoutAbc collapse kept), language=None} dispatched via spec.kind.adapter().run(&cx).into_iter().next() (+ a generalized defensive fallback, still unreachable for the built-ins); the relabel + tool_version stamp stay in validate_tool_specs. (b) src/bin/tool_matrix.rs: run_module_tools + run_design_tools each build one AdapterTarget + a per-column run_column closure dispatching through AcceptanceTool::{Verilator,Yosys,Iverilog}.adapter().run(&cx), preserving the fixed verilator/yosys/iverilog_compile columns, skip flags, verilator_only Verilator-only no-op, the --language selector, and the Yosys-mode row count; the six now-unused run_* primitive imports were dropped (the primitives stay live behind the adapters). So adding sv2v (.2b) is now a registry entry + a column field + a routing line — no new invocation site. Default-off / DUT byte-identical (snapshots 6/6 untouched).`
  Verification: `cargo check --all-targets clean; cargo test --lib 547/0 (+1: validate_tool_specs_routes_each_kind_through_its_adapter_single_row — proves the Yosys Both->single collapse survives the registry routing); tests/snapshots.rs 6/6 byte-identical; cargo test --bin tool_matrix 75/0; anvil bin + pipeline + divergence_e2e (portable divergence path exercises the rerouted validate_tool_specs) exit 0; clippy --all-targets -D warnings clean; fmt --all --check clean; mdbook build clean; check_memory_architecture + KM gen/check green. Heavy steps RAM-guarded (decision 0003).`
  Commit: `DOWNSTREAM-ADAPTER-EXPANSION.2a.3`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2b`
  Status: `active`
  Goal: `The first new adapter, sv2v, as an accept/reject transpile column: registered descriptor + tools-selectable + queryable verdict in ValidateReport/DivergenceReport/the matrix column; friendly absent-tool no-op + an #[ignore] real-tool gate; book/USER_GUIDE/README/KM card. Pre-split (mirroring .2a) into the additive downstream+MCP surface (.2b.1) and the byte-identical-sensitive tool_matrix column + the real-tool gate + docs (.2b.2), so each sub-slice commits independently.`
  Children: `DOWNSTREAM-ADAPTER-EXPANSION.2b.1`, `DOWNSTREAM-ADAPTER-EXPANSION.2b.2`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2b.1`
  Status: `done`
  Goal: `The sv2v downstream adapter + its MCP selectability/discoverability (additive; default-off / DUT byte-identical). src/downstream/mod.rs: an Sv2v variant on AcceptanceTool (from_name("sv2v")/binary()="sv2v"/adapter()), run_sv2v + run_sv2v_design primitives (sv2v <file> module; --top=<top> + files design; transpile accept/reject, no fact hook), an Sv2vAdapter (id/binary "sv2v"; run dispatches Module/Design; supports_facts=false), a 4th ADAPTER_REGISTRY entry, and a first_tool_warning "sv2v" arm (case-insensitive warning: like iverilog). src/mcp/mod.rs: add "sv2v" to the four tools-enum schemas + the parse_validate_tools error message + the controlled-tools description, so sv2v is selectable + discoverable over the API (decision 0017). NO tool_matrix column yet (that is .2b.2).`
  Acceptance: `sv2v is selectable via the tools arg of validate/divergence/hunt (AcceptanceTool::from_name) and appears in adapters()/adapter_catalog() (the anvil://catalog/adapters resource gains a 4th entry, present=false locally since sv2v is absent — the friendly no-op). Lib proofs: registry holds the 4 builtins with expected ids/binaries; from_name("sv2v")==Some(Sv2v) + binary()=="sv2v"; AcceptanceTool::Sv2v.adapter() round-trips; sv2v warning detection; a portable validate run selecting sv2v with a missing binary fails to spawn cleanly (no panic, not ok). The existing mcp/lib tests asserting the 3-tool list are updated to 4. Gate: cargo check --all-targets; cargo test --lib (incl. mcp::tests); snapshots 6/6 byte-identical (no generator change); clippy -D warnings; fmt --check; mdbook build clean; default-off / DUT byte-identical.`
  Result: `Landed the sv2v downstream adapter + its MCP selectability/discoverability, additive / DUT byte-identical. src/downstream/mod.rs: AcceptanceTool::Sv2v (from_name("sv2v")/binary()="sv2v"/adapter()=&SV2V_ADAPTER) + run_sv2v (sv2v <file>) + run_sv2v_design (sv2v --top=<top> <files…>) transpile-accept/reject primitives (no fact hook) + an Sv2vAdapter (supports_facts=false) + a 4th ADAPTER_REGISTRY entry + a first_tool_warning "sv2v" arm (case-insensitive warning:, like iverilog). src/mcp/mod.rs: "sv2v" added to the four tools-enum schemas (validate/divergence/minimize/hunt) + the validate description + the parse_validate_tools allow-list error. So sv2v is selectable via the tools arg and appears in adapters()/adapter_catalog() (anvil://catalog/adapters now 4 entries; present=false locally since sv2v is absent — the friendly no-op). Book synced (agent-mcp.md fixed-allow-list + validate row; api-tools.md tools enum + controlled-tools allow-list; api-resources-prompts.md catalog row). +2 net-new lib proofs (adapter_catalog_projects_every_registered_adapter; mcp parse_validate_tools_accepts_sv2v_and_rejects_unknown) + extended the registry/warning/adapter-map/allow-list/catalog-resource/validate_tool_specs-per-kind proofs to 4 adapters. No tool_matrix column (.2b.2). Default-off / DUT byte-identical (snapshots 6/6, no generator change).`
  Verification: `cargo check --all-targets clean; cargo test --lib 549/0 (+2 net: adapter_catalog_projects_every_registered_adapter, parse_validate_tools_accepts_sv2v_and_rejects_unknown; existing registry/warning/routing/catalog-resource proofs extended to 4 adapters); snapshots 6/6 byte-identical (no generator change); clippy --all-targets -D warnings clean; fmt --all --check clean; mdbook build clean; check_memory_architecture + KM gen/check green. Heavy steps RAM-guarded (decision 0003).`
  Commit: `DOWNSTREAM-ADAPTER-EXPANSION.2b.1`

- ID: `DOWNSTREAM-ADAPTER-EXPANSION.2b.2`
  Status: `pending`
  Goal: `The tool_matrix sv2v acceptance column (byte-identical-sensitive) mirroring the --iverilog-compile precedent: a --sv2v opt-in flag + sv2v_bin + ModuleReport/DesignReport.sv2v: Option<ToolInvocation> (serde skip_serializing_if) routed through the registry, checkpoint fields + --resume guard, per-tool tally + an opportunistic saw_sv2v_* coverage fact (never a required gate), the friendly absent-tool no-op; an #[ignore] real-tool gate (tests/sv2v_e2e.rs, the hunt_e2e/divergence_e2e precedent) for when sv2v is installed; book (agent-mcp.md / api-resources-prompts.md / synthesizability.md downstream surface) + USER_GUIDE + README CLI surface + a KM card. Default-off ⇒ banked reports + --resume + snapshots 6/6 byte-identical.`
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
| 1 | `DOWNSTREAM-ADAPTER-EXPANSION.2b.2` | `pending` | `sv2v` `tool_matrix` column (byte-identical-sensitive, mirrors `--iverilog-compile`) + the `#[ignore]` real-tool gate + book/USER_GUIDE/README/KM. |
| 2 | `DOWNSTREAM-ADAPTER-EXPANSION.2c` | `pending` | `slang` — the richer adapter landing the optional JSON-AST `extract_facts` hook. |

Done: `.2b.1` — the `sv2v` downstream adapter (4th `AcceptanceTool` / `run_sv2v` /
`Sv2vAdapter` / registry entry) + MCP selectability (`tools` enums) +
discoverability (`adapter_catalog()` now 4 entries), additive / DUT byte-identical.

Done: `.1` (design ADR, decision `0020`); `.2a.1` (the closed `Adapter` registry
core + `validate` routed through it, byte-identical); `.2a.2` (the
`anvil://catalog/adapters` discoverability resource, decision `0017`); `.2a.3`
(`validate_tool_specs` + the `tool_matrix` per-unit columns routed through the
registry, byte-identical — `.2a` complete).

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
| `2026-06-17` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.1` | `cargo check --all-targets clean; cargo test --lib 545/0 (+2 registry proofs); snapshots 6/6 byte-identical; tool_matrix 75/0; anvil 12/0; clippy -D warnings clean; fmt --check clean; DUT byte-identical (umbrella + snapshots); RAM-guarded` | `done` |
| `2026-06-17` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.2` | `cargo test --lib 546/0 (+1 catalog proof); snapshots 6/6 byte-identical; clippy -D warnings clean; fmt --check clean; mdbook build clean; book_examples 3/3; no introspection SCHEMA_VERSION bump; DUT byte-identical; RAM-guarded` | `done` |
| `2026-06-18` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.3` | `cargo check --all-targets clean; cargo test --lib 547/0 (+1: validate_tool_specs_routes_each_kind_through_its_adapter_single_row); snapshots 6/6 byte-identical; tool_matrix 75/0; anvil+pipeline+divergence_e2e exit 0; clippy -D warnings clean; fmt --check clean; mdbook build clean; check_memory_architecture + KM gen/check green; DUT byte-identical; RAM-guarded` | `done` |
| `2026-06-18` | `DOWNSTREAM-ADAPTER-EXPANSION.2b.1` | `cargo check --all-targets clean; cargo test --lib 549/0 (+2 net: adapter_catalog_projects_every_registered_adapter, parse_validate_tools_accepts_sv2v_and_rejects_unknown; registry/warning/routing/catalog proofs extended to 4 adapters); snapshots 6/6 byte-identical (no generator change); clippy -D warnings clean; fmt --check clean; mdbook build clean; check_memory_architecture + KM gen/check green; DUT byte-identical; RAM-guarded` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DOWNSTREAM-ADAPTER-EXPANSION` | `USABILITY-LANE-OWNERSHIP.1 — register 7 owner-directed usability/capability lanes + API-first decision 0017` | Tree registered (not yet started); frontier `.1` (design ADR) pending. |
| `DOWNSTREAM-ADAPTER-EXPANSION.1` | `DOWNSTREAM-ADAPTER-EXPANSION.1 — adapter-interface ADR (decision 0020)` | Design ADR (`412e5ff`); pre-split `.2` → `.2a`/`.2b`/`.2c`; frontier advances to `.2a`. |
| `DOWNSTREAM-ADAPTER-EXPANSION.2a.1` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.1 — closed Adapter registry in src/downstream` | The registry core + `validate` routed through it, byte-identical. `.2a` split into `.2a.1`/`.2a.2`/`.2a.3`; frontier advances to `.2a.2`. |
| `DOWNSTREAM-ADAPTER-EXPANSION.2a.2` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.2 — anvil://catalog/adapters discoverability resource` | The SCHEMA-DERIVED adapter catalog over MCP (decision `0017`); `Adapter::supports_facts` + `AdapterInfo`/`adapter_catalog()`. Frontier advances to `.2a.3`. |
| `DOWNSTREAM-ADAPTER-EXPANSION.2a.3` | `DOWNSTREAM-ADAPTER-EXPANSION.2a.3 — route validate_tool_specs + tool_matrix columns through the adapter registry` | The last two downstream callers (`validate_tool_specs` via `run_tool_spec`; the `tool_matrix` `run_module_tools`/`run_design_tools` columns) routed through the registry, byte-identical; six now-unused `run_*` imports dropped from `tool_matrix.rs`. `.2a` complete; frontier advances to `.2b` (sv2v). |
| `DOWNSTREAM-ADAPTER-EXPANSION.2b.1` | `DOWNSTREAM-ADAPTER-EXPANSION.2b.1 — sv2v downstream adapter + MCP selectability/discoverability` | The first new adapter: `AcceptanceTool::Sv2v` + `run_sv2v`/`run_sv2v_design` + `Sv2vAdapter` + a 4th registry entry + a `first_tool_warning` arm; `mcp` `tools` enums + `parse_validate_tools` allow-list updated to 4; book synced. Additive / DUT byte-identical (no `tool_matrix` column — that is `.2b.2`). `.2b` split into `.2b.1`/`.2b.2`; frontier advances to `.2b.2`. |

## Changelog

- `2026-06-17`: Created task tree (registration via `USABILITY-LANE-OWNERSHIP.1`).
- `2026-06-17`: `.1` design ADR done — decision `0020` (closed compile-time `Adapter`
  registry; `sv2v` first, `slang` second; API-completeness via the existing `tools`
  arg + the new adapter-catalog projection). `.2` pre-split into `.2a`/`.2b`/`.2c`;
  frontier advanced to `.2a`.
- `2026-06-17`: `.2a` split into `.2a.1` (downstream registry core) / `.2a.2`
  (adapter-catalog discoverability) / `.2a.3` (orchestrator routing) so each
  sub-slice is provably byte-identical (refining decision `0020`'s `.2a`).
  **`.2a.1` done** — `src/downstream/mod.rs` gains `trait Adapter` + `AdapterRunCx`/
  `AdapterTarget` + 3 built-in adapters delegating verbatim to `run_*` + `static
  ADAPTER_REGISTRY`/`adapters()` + `AcceptanceTool::adapter()`; `validate` routed
  through the registry, byte-identical. Gate green (lib 545/0, snapshots 6/6,
  tool_matrix 75/0, anvil 12/0, clippy/fmt). Frontier advanced to `.2a.2`.
- `2026-06-17`: **`.2a.2` done** — the `anvil://catalog/adapters` discoverability
  resource (decision `0017`): `downstream` gains a defaulted `Adapter::supports_facts()`
  + `AdapterInfo` + `adapter_catalog()` (a SCHEMA-DERIVED projection of `adapters()`
  with a live `tool_version()` PATH probe for `present`); `mcp` advertises + serves it;
  book synced (`api-resources-prompts.md` + `agent-mcp.md`). No introspection schema
  bump. Gate green (lib 546/0, snapshots 6/6, clippy/fmt, mdbook, book_examples 3/3).
  Frontier advanced to `.2a.3`.
- `2026-06-18`: **`.2a.3` done** — routed the two remaining downstream callers through
  the closed `Adapter` registry, completing `.2a`. `downstream::run_tool_spec` (the
  `validate_tool_specs` version axis) now dispatches via `spec.kind.adapter().run(&cx)`
  with the Yosys `Both`→single collapse preserved and the relabel + `tool_version` stamp
  still in `validate_tool_specs`; `tool_matrix`'s `run_module_tools` / `run_design_tools`
  dispatch each fixed `verilator`/`yosys`/`iverilog_compile` column through
  `AcceptanceTool::*.adapter().run(&cx)` (one `AdapterTarget` + a per-column
  `run_column` closure), with the skip flags / `verilator_only` no-op / `--language`
  selector / Yosys-mode row count all preserved; the six now-unused `run_*` primitive
  imports were dropped from `tool_matrix.rs` (the primitives stay live behind the
  adapters). Byte-identical: fixed columns + banked reports + `--resume` + snapshots 6/6
  untouched. Adding `sv2v` (`.2b`) is now a registry entry + a column field + a routing
  line. Gate green (lib 547/0 +1 routing proof, snapshots 6/6, tool_matrix 75/0,
  anvil+pipeline+divergence_e2e exit 0, clippy/fmt, mdbook, check_memory_architecture +
  KM). Frontier advanced to `.2b`.
- `2026-06-18`: `.2b` pre-split into `.2b.1` (downstream sv2v adapter + MCP
  selectability/discoverability) / `.2b.2` (the byte-identical-sensitive `tool_matrix`
  column + the `#[ignore]` real-tool gate + book/USER_GUIDE/README/KM), mirroring the
  `.2a` split. **`.2b.1` done** — `src/downstream/mod.rs` gains the first new adapter,
  `sv2v`: `AcceptanceTool::Sv2v` (`from_name`/`binary` `"sv2v"`/`adapter()`) +
  `run_sv2v` (`sv2v <file>`) + `run_sv2v_design` (`sv2v --top=<top> <files…>`) transpile
  accept/reject primitives (no fact hook) + an `Sv2vAdapter` (`supports_facts=false`) +
  a 4th `ADAPTER_REGISTRY` entry + a `first_tool_warning` `"sv2v"` arm; `src/mcp/mod.rs`
  adds `"sv2v"` to the four `tools` enum schemas + the `validate` description + the
  `parse_validate_tools` allow-list error, so `sv2v` is selectable via the `tools` arg
  and discoverable in `adapter_catalog()` (`anvil://catalog/adapters` now 4 entries,
  `present=false` locally since `sv2v` is absent — the friendly no-op). Book synced
  (`agent-mcp.md` / `api-tools.md` / `api-resources-prompts.md`). Additive / DUT
  byte-identical (snapshots 6/6, no generator change). Gate green (lib 549/0 +2 net
  proofs, clippy/fmt, mdbook, KM). Frontier advanced to `.2b.2`.
