---
id: downstream-adapter-interface
title: ANVIL's downstream reach becomes pluggable through a closed, compile-time Adapter registry over the one run_tool runner + the one tool_verdict classifier — new tools (sv2v first, slang second) plug in as opt-in acceptance columns selectable and queryable over the same API, with the allow-list/sandbox/RAM-guard discipline preserved
answers:
  - "how does ANVIL add a new downstream tool"
  - "can I plug a new SystemVerilog tool into ANVIL as an acceptance column"
  - "what is the downstream adapter interface"
  - "how do I add slang or sv2v or surelog to ANVIL"
  - "is there an adapter trait or registry for downstream tools"
  - "how does ANVIL generalize run_verilator run_yosys run_iverilog"
  - "which new downstream tool does ANVIL integrate first"
  - "are new downstream adapters selectable and queryable over MCP"
  - "does adding a tool break the allow-list or sandbox discipline"
  - "how does ANVIL discover which downstream tools are installed"
  - "can an agent supply an arbitrary downstream tool binary"
  - "what is the adapter catalog query"
date: 2026-06-17
status: accepted
tags: [usability, downstream, adapter, registry, mcp, api, acceptance, sv2v, slang, surelog, north-star, breadth, architecture, agent]
evidence: docs/tasks/DOWNSTREAM-ADAPTER-EXPANSION.md (the owning tree; this is its .1 design leaf, pre-splitting .2); docs/decisions/0017-api-first-everything-mcp-accessible.md (the API-completeness gate this lane must satisfy — selectable/invocable/queryable/documented); docs/decisions/0004-agent-introspection-mcp-lane.md (the controlled-tool fixed-allow-list + sandbox + RAM-guard + audit discipline + the no-shadow-simulator ceiling); docs/decisions/0019-acceptance-divergence-hunting.md (the multi-tool divergence detector that every new column multiplies for free; the .2f library-only caller-supplied-binary trust-boundary finding this reaffirms); docs/decisions/0011-semantic-introspection-derived-query-surface.md (the SCHEMA-DERIVED projection discipline the adapter catalog/verdicts obey); src/downstream/mod.rs (the EXISTING surface this generalizes — AcceptanceTool{Verilator,Yosys,Iverilog}+from_name+binary, run_verilator/run_yosys/run_iverilog_compile (+ _design variants), run_tool the one sandboxed runner, first_tool_warning the per-tool warning predicate, tool_verdict the one accept/warn/reject classifier, ToolInvocation{tool,argv,success,exit_code,stdout_log,stderr_log,error,version}, ToolSpec{kind,binary,label} the caller-supplied-binary version axis, ValidateOptions{tools,yosys_mode,mem_limits,sandbox_root,keep_sandbox}, validate/validate_tool_specs, prepare_dut_sandbox/DutSandbox); src/bin/tool_matrix.rs (the per-tool ModuleReport/DesignReport column model + the saw_* coverage facts + tools_present() friendly-no-op precedent); src/mcp/mod.rs (the validate/divergence/hunt controlled-tool dispatch + tools arg via AcceptanceTool::from_name + the artifact cache + anvil://audit/log); src/divergence/mod.rs (divergence::run — every added column becomes a new comparable verdict for free); tests/sv_version_downstream.rs + tests/hunt_e2e.rs + tests/divergence_e2e.rs (the #[ignore] tool-gated real-tool gate precedent for absent tools)
---

# 0020 - DOWNSTREAM-ADAPTER-EXPANSION: a closed, compile-time Adapter registry generalizes the fixed Verilator/Yosys/Icarus surface, keeping one runner + one classifier + the allow-list intact

- Date: 2026-06-17
- Status: accepted — **design only; no code** (this is the `DOWNSTREAM-ADAPTER-EXPANSION.1`
  ADR leaf). It pins the adapter trait/registry shape, the allow-list/sandbox
  extension discipline, the first new adapter to land (and the second), the
  MCP/config selection + query surface (decision `0017`), and the byte-identical
  guarantees; it **pre-splits `.2`** into the refactor + per-adapter impl leaves.
  No `src/` change; default `anvil` build / `--artifact dut` byte-identical.
- Tree: `DOWNSTREAM-ADAPTER-EXPANSION.1` (design/decision leaf).
- Activated by: autonomous PNT selection (`2026-06-17`) — the owner-directed
  usability lane 3 (north-star idea 3), registered by `USABILITY-LANE-OWNERSHIP.1`
  and named in decision [`0019`](0019-acceptance-divergence-hunting.md)'s links as
  *"more tools to diverge across"*.
- Binds: decision [`0017`](0017-api-first-everything-mcp-accessible.md) (the
  API-completeness gate — every adapter selectable + queryable over MCP, the CLI a
  shim) and decision [`0004`](0004-agent-introspection-mcp-lane.md) (the
  controlled-tool fixed-allow-list + sandbox + RAM-guard + audit discipline + the
  no-shadow-simulator ceiling).

## Context

ANVIL's reason to exist is to **surface downstream-tool bugs** with RTL that is
legal by construction (`project_anvil_north_star`). Each *additional independent
front-end* (parser / elaborator / linter / transpiler / synthesizer) is a new
chance to trip a real bug — and, fed into the acceptance-divergence detector
(decision `0019`), a new pair to disagree. Today the downstream surface is a
**fixed allow-list of exactly three tools**:

- `src/downstream/mod.rs` hard-codes `AcceptanceTool { Verilator, Yosys, Iverilog }`
  (`from_name` / `binary`) and one bespoke `run_*` function per tool
  (`run_verilator` / `run_yosys` / `run_iverilog_compile`, each with a `_design`
  sibling). Adding a fourth tool today means: a new enum variant, a new `run_*`
  pair, a new `first_tool_warning` arm, a new `match` arm in `validate`, in
  `validate_tool_specs`/`run_tool_spec`, and in the `tool_matrix` column code —
  the behaviour is **scattered across call sites**, so each new tool is a sprawling
  cross-cutting edit instead of one self-contained unit.

What is *already* shared and must **not** be re-implemented per tool:

- `run_tool(tool_name, binary, argv, out_dir, stem) -> ToolInvocation` is the **one**
  sandboxed runner (spawn, capture stdout/stderr to logs, fold `first_tool_warning`
  into `success = false`). Every tool already goes through it.
- `tool_verdict(&ToolInvocation) -> ToolVerdict {Accept,Warn,Reject}` is the **one**
  accept/warn/reject classifier (decision `0019`, extracted from `hunt`), reused by
  `hunt::run` and `divergence::run`.
- `prepare_dut_sandbox` / `DutSandbox` is the **one** generate → mkdir → write-`<top>.sv`
  lifecycle; `MemGuard`/`MemLimits` is the **one** decline-to-spawn guard; the MCP
  layer is the **one** controlled-tool dispatch + artifact cache + `anvil://audit/log`.

So the only things that genuinely differ per tool are: (1) the **argv** for a
module vs a design, (2) the **warning-detection** rule, and (3) optionally a
**richer fact extraction** (the Verilator JSON-AST frontend-parity gate
`tests/frontend_parity.rs` is the precedent: a tool that emits structured AST/elab
facts beyond accept/warn/reject). Everything else is shared. The lane is therefore
a **refactor-to-a-registry + thin per-tool descriptors**, not a new engine.

**Live-toolchain probe (`2026-06-17`, this session).** `verilator` 5.046, `yosys`
0.64, `iverilog` 13.0 are on `PATH`; **`slang`, `sv2v`, `surelog`, `svlint`,
`verible`, `moore` are all absent.** This resolves the tree's open question "which
adapter first depends on local availability": no candidate is installed now, so the
first adapters land **structurally + as a friendly absent-tool no-op** (the
`tools_present()` precedent) gated by an `#[ignore]` tool-gated real-tool test (the
`tests/sv_version_downstream.rs` / `tests/hunt_e2e.rs` / `tests/divergence_e2e.rs`
precedent), upgraded to a banked real-tool proof when the binary is installed.

## Decision

**Downstream reach becomes pluggable through a *closed, compile-time* `Adapter`
registry layered over the one `run_tool` runner and the one `tool_verdict`
classifier.** "Pluggable" means *a new tool is a small, self-contained, vetted Rust
descriptor added to the registry* — **not** a runtime/loadable plugin and **not** an
agent-supplied command. The fixed-allow-list guarantee (decision `0004`) is
preserved: the set of runnable tools is still fixed at compile time and reviewed;
the registry only makes adding one a single cohesive edit. Default-off / DUT
byte-identical: new adapters are opt-in columns; the three built-ins keep their
exact labels/argv so banked `tool_matrix` reports + `--resume` checkpoints stay
byte-for-byte unchanged.

### 1. The `Adapter` trait + the closed registry

An adapter is a vetted descriptor (a `&'static dyn Adapter`, or an equivalent
data-driven `AdapterSpec` — refined at `.2a`) that owns exactly the per-tool
variation:

```text
trait Adapter {
    fn id(&self) -> &'static str;            // stable selector token + report label root
    fn binary(&self) -> &'static str;        // FIXED, vetted binary name (never agent-supplied — 0004)
    fn module_argv(&self, sv: &Path, top: &str) -> Vec<String>;
    fn design_argv(&self, sv_paths: &[PathBuf], top: &str) -> Vec<String>;
    fn warning(&self, stdout: &str, stderr: &str) -> Option<String>;   // generalizes first_tool_warning
    fn extract_facts(&self, inv: &ToolInvocation, sb: &DutSandbox) -> Option<AdapterFacts> { None }  // optional richer hook
}

fn adapters() -> &'static [&'static dyn Adapter];   // the CLOSED registry — compile-time, reviewed
```

The registry is the **single home** of the tool list. Every adapter runs through
the **unchanged** `run_tool` (so the sandbox, log capture, and `version: None`
default-path wire shape are identical) and is classified by the **unchanged**
`tool_verdict`. There is **no second runner and no second classifier**
(`feedback_full_factorization`). The trait generalizes only argv + warning +
optional facts.

`AcceptanceTool { Verilator, Yosys, Iverilog }` is **not retired**
(`feedback_never_retire_strategies`): the three built-ins become the first three
registered adapters whose `id`/argv/warning reproduce today's behaviour exactly
(`AcceptanceTool::from_name`/`binary` either delegate to the registry or remain the
canonical built-in identity, decided at `.2a`). Their existing
`ToolInvocation.tool` labels (`"verilator"`, the `yosys-<mode>` rows, the
`iverilog-compile` row) are a **hard byte-identical constraint** — banked reports
and `--resume` checkpoints key off them.

### 2. The verdict is unchanged → every detector gains the new tool for free

A new adapter's accept/warn/reject is derived by the **same** `tool_verdict` from
its `ToolInvocation` (exit code + its `warning` predicate). Therefore — with **zero**
change to their code — the moment a tool is a registered, selected adapter:

- `validate` / the MCP `validate` tool report its verdict in `ValidateReport.tools`;
- `divergence::run` (decision `0019`) compares it against the others, so a new tool
  that disagrees becomes an `accept_reject` / `accept_warn` / `warn_reject`
  divergence finding automatically — **the adapter expansion multiplies the
  bug-surface across all three detector surfaces (hunt / matrix / MCP) without
  touching any of them**;
- `tool_matrix` records it as a new column.

This compounding is the lane's whole point and is why it "feeds
`BUG-HUNT-ORCHESTRATION` + `ACCEPTANCE-DIVERGENCE-HUNTING` with more columns".

### 3. The first adapters to land (resolving the open question)

Against the live toolchain (all candidates absent), the order is chosen by
*minimal-surface-first* (the `0019` "reject/warning first, fold the richer axis in
later" cadence) and *parser-distinctness*:

1. **`sv2v` first (`.2b`)** — the **minimal adapter shape**: a pure accept/reject
   **transpile** column (`sv2v <top>.sv` → a clean exit transpiles, a non-zero exit
   / a warning is a finding). It exercises the whole trait end-to-end with **no fact
   hook**, keeping the first new-adapter leaf small. It is a genuinely independent
   SystemVerilog front-end (Haskell), so it is parser-distinct from
   Verilator/Yosys/Icarus, and `sv2v → Verilog-2005 → {Yosys, Icarus}` is a classic
   real flow that also stresses the existing columns. `brew install sv2v` is the
   intended local install for the real-tool gate.
2. **`slang` second (`.2c`)** — the **richer adapter**: a strict, fast, independent
   elaborator that also emits a JSON AST (`slang --ast-json`). It lands the
   **optional `extract_facts` hook** (the Verilator JSON-AST frontend-parity
   precedent), proving the trait's richer path: accept/reject **plus** SCHEMA-DERIVED
   structural/elaboration facts (top, ports, instances) — never behaviour.

Both are currently absent, so `.2b`/`.2c` ship as a structural column + a friendly
absent-tool no-op + an `#[ignore]` real-tool gate, satisfying the tree's acceptance
criterion ("…or its divergences retained as reproducers … absent tools are a
friendly no-op, not a hard failure"). `surelog`/UHDM and a generic
commercial-wrapper adapter are future `.2d+` leaves, each its own pick — nothing
retired.

### 4. The MCP / config selection + query surface (decision `0017`)

The API-completeness gate is satisfied **without a new tool** — adapters ride the
existing surfaces:

- **Selectable / steerable.** The `validate` / `divergence` / `hunt` controlled
  tools already take a `tools: [...]` arg parsed by `AcceptanceTool::from_name`
  (unknown ⇒ a clean `-32602`, never a spawn). That parse generalizes to the
  registry's `id`s, so `tools: ["verilator","sv2v"]` selects the new adapter through
  the **same allow-listed path**. `ValidateOptions.tools` carries the registry ids;
  `Config`/serde and the `tool_matrix` / `anvil` CLI flags are **shims** over the
  same registry (the CLI is never a superset — decision `0017`).
- **Queryable.** (a) Each adapter's per-artifact `ToolInvocation` + verdict already
  lands in `ValidateReport.tools` / `DivergenceReport.verdicts` / the matrix column,
  queryable over MCP **for free**. (b) The **registry itself becomes discoverable**:
  an **adapter catalog** is exposed as a pure, SCHEMA-DERIVED projection — `{ id,
  binary, present (a `tools_present()`-style PATH probe), supports_facts }` per
  adapter — served as an `anvil://…` resource / `dump_config`-style listing (the
  exact surface picked at `.2a`), so an agent can discover *which tools exist and
  which are installed* over the API alone (the decision-`0017` "drive it with only
  the MCP API" test). (c) An adapter's `extract_facts` output is a SCHEMA-DERIVED
  projection of that tool's own report (e.g. slang's AST), surfaced in the report —
  **never** an ANVIL behavioural oracle (the `0004` ceiling holds: an adapter reports
  *its tool's* acceptance/lint/transpile/AST facts, not intended function).
- **Documented.** `book/src/agent-mcp.md` + the API-reference pages + USER_GUIDE +
  README at each impl leaf.

### 5. The allow-list / sandbox extension discipline (decision `0004`)

A new adapter is admitted **only** under the existing guarantees:

- **Closed registry, fixed binary.** Adapters are compile-time entries with a
  **fixed binary name**; there is no `run_command` tool and no agent-supplied path.
  This is the same boundary `0019.2f` drew for the version axis: pairing an
  allow-listed *kind* with a caller-supplied *binary* (`ToolSpec`) is a strictly
  larger trust surface and stays **library-only**, not exposed over MCP/CLI — adapters
  are emphatically **not** a back door to it.
- **One runner, sandboxed.** Every adapter spawns only via `run_tool` (fixed binary,
  no shell) inside the caller-fixed per-run `prepare_dut_sandbox` temp dir, under
  `MemGuard`/`MemLimits` decline-to-spawn, with `anvil://audit/log` records on the
  controlled path.
- **Friendly absent-tool no-op.** A missing binary is the `tools_present()`
  precedent — recorded as not-run with a clear reason, **never** a hard failure and
  **never** a required coverage gate (the `--diff-sim` / `saw_acceptance_divergence`
  precedent: clean RTL accepted by every present tool is the steady state).
- **Default-off / byte-identical.** New columns are opt-in; the default
  `validate` / `tool_matrix` / DUT paths and the three built-in labels/argv are
  unchanged; no generator change ⇒ DUT byte-identical.

## Pre-split of `DOWNSTREAM-ADAPTER-EXPANSION.2` (implementation)

Ordered sub-leaves (refinable at pick time; each default-off / DUT byte-identical,
each carrying the decision-`0017` API-completeness gate):

- `.2a` — **the registry refactor + the catalog query.** Introduce the `Adapter`
  trait / `AdapterSpec` + the closed `adapters()` registry; re-express
  Verilator/Yosys/Icarus as the first three registered adapters with
  **byte-identical** `id`/argv/warning-detection; route `validate` /
  `validate_tool_specs` / the `tool_matrix` columns / `AcceptanceTool::from_name`
  through the registry; add the SCHEMA-DERIVED **adapter-catalog** query/resource
  (decision `0017` discoverability). Pure refactor + the catalog; snapshots 6/6;
  banked `tool_matrix` reports + `--resume` checkpoints unchanged.
- `.2b` — **the first new adapter, `sv2v`,** as an accept/reject transpile column:
  registered descriptor + `tools`-selectable + queryable verdict in
  `ValidateReport`/`DivergenceReport`/the matrix column; friendly absent-tool no-op
  + an `#[ignore]` real-tool gate; `book`/USER_GUIDE/README/KM card.
- `.2c` — **the second new adapter, `slang`,** with the optional `extract_facts`
  hook (JSON-AST), proving the trait's richer SCHEMA-DERIVED path; absent-tool no-op
  + `#[ignore]` gate; docs.
- `.2d+` (future) — `surelog`/UHDM, a generic commercial-wrapper adapter; each its
  own leaf, each its own pick. Nothing retired.

## Rejected alternatives

- **An arbitrary caller-supplied command / binary as an adapter (a `run_command`
  tool).** Rejected — it destroys the `0004` fixed-allow-list and the `0019.2f`
  trust boundary (it would let a caller point ANVIL at any executable). A *closed,
  compile-time* registry is "pluggable" in the way that matters — a new tool is a
  small vetted Rust unit — while the runnable set stays fixed and reviewed.
- **A dynamic / loadable plugin system (dylibs, scripts).** Rejected — runtime-loaded
  adapters defeat the allow-list *and* reproducibility/auditability. The registry is
  compile-time and closed.
- **Keeping the `AcceptanceTool` enum + just adding a variant per tool.** Rejected
  for scaling: the per-tool behaviour stays scattered across `validate` /
  `validate_tool_specs` / `first_tool_warning` / the matrix columns, and the enum
  has no room for the optional fact hook. But `AcceptanceTool` is **not retired** —
  the built-ins become the first registry entries and the enum stays the canonical
  built-in identity.
- **A second runner or a second warning/verdict classifier per tool.** Rejected
  (`feedback_full_factorization`): every adapter reuses the one `run_tool` + the one
  `first_tool_warning`-shaped predicate + the one `tool_verdict`. The trait carries
  only the *argv + warning rule + optional facts*.
- **A behavioural oracle (e.g. treat sv2v's transpiled Verilog as golden
  semantics).** Forbidden (decision `0004`, ROADMAP gap 4). Adapters report
  *acceptance / lint / transpile / AST* facts, not behaviour. Cross-*simulator*
  trace agreement is the orthogonal `--diff-sim` axis's job, not an adapter's.
- **slang first.** Rejected for *first*: it needs the richer JSON-AST `extract_facts`
  hook and is harder to install. `sv2v` is the minimal-surface first cut (`.2b`),
  `slang` the richer follow-up (`.2c`). Neither retired.
- **A CLI-only `--extra-tool` flag (no API).** Rejected — directly violates decision
  `0017` (`feedback_api_for_agents_not_humans`). Adapters are selectable + queryable
  over MCP; the CLI is a shim.

## Consequences

- The implementation (`.2`) lands an `Adapter` trait + a closed `adapters()`
  registry (the three built-ins re-expressed byte-identically) + an adapter-catalog
  query + the first two new adapters (`sv2v`, then `slang` with the fact hook) —
  composition over the proven `run_tool`/`tool_verdict`/sandbox surfaces, default-off
  / DUT byte-identical.
- Every detector deepens for free: each added adapter is a new comparable verdict in
  `divergence::run`, a new column in `tool_matrix`, and a new selectable tool in
  `hunt` / `validate` — no change to those engines (the decision-`0019` compounding).
- The fixed-allow-list + sandbox + RAM-guard + audit discipline (decision `0004`) and
  the caller-supplied-binary library-only boundary (decision `0019.2f`) are restated
  and preserved; adapters are not a trust back door.
- The decision-`0017` API-completeness gate is met without a new MCP tool: adapters
  are selectable through the existing `tools` arg and queryable through the existing
  reports + the new adapter-catalog projection.
- The structure-first / no-shadow-simulator ceiling is reaffirmed: an adapter's
  richer facts are a projection of *its own tool's* output, never ANVIL-computed
  behaviour.

## Links

- Owning tree: `DOWNSTREAM-ADAPTER-EXPANSION` (this is its `.1` design leaf;
  pre-splits `.2a`/`.2b`/`.2c`/`.2d+`).
- Parent decisions: `0017` (API-completeness gate), `0004` (MCP lane +
  fixed-allow-list + sandbox + RAM-guard + audit + no-shadow-simulator), `0011`
  (SCHEMA-DERIVED projection discipline), `0019` (the divergence detector each new
  column multiplies; its `.2f` caller-supplied-binary library-only boundary).
- Generalized surface: `src/downstream/mod.rs` (`AcceptanceTool` + `run_verilator` /
  `run_yosys` / `run_iverilog_compile` (+ `_design`), `run_tool`, `first_tool_warning`,
  `tool_verdict`, `ToolInvocation`, `ToolSpec`, `ValidateOptions`/`validate`,
  `prepare_dut_sandbox`/`DutSandbox`), `src/bin/tool_matrix.rs` (the per-tool column
  model + `tools_present()` + `saw_*` facts), `src/mcp/mod.rs` (the controlled-tool
  dispatch + `tools` arg + cache + audit), `src/divergence/mod.rs` (`divergence::run`).
- Real-tool gate precedent (absent tools): `tests/sv_version_downstream.rs`,
  `tests/hunt_e2e.rs`, `tests/divergence_e2e.rs` (`#[ignore]`, tool-gated); the
  Verilator JSON-AST fact-hook precedent: `tests/frontend_parity.rs`.
- Book: `book/src/agent-mcp.md` + the API-reference pages + USER_GUIDE + README —
  updated at the impl leaves.
- Memory: `project_anvil_north_star` (surface downstream-tool bugs — more front-ends
  multiply the loop), `feedback_api_for_agents_not_humans` (design the API for
  agents), `feedback_full_factorization` (one runner, one classifier),
  `feedback_never_retire_strategies` (nothing retired), `feedback_rules_first_generation`
  (no generator change; adapters are downstream-only).
- Synergistic lanes: `ACCEPTANCE-DIVERGENCE-HUNTING` (every column a new pair to
  diverge), `BUG-HUNT-ORCHESTRATION` (the engine the columns feed),
  `CI-PACKAGING-DISTRIBUTION` (a CI wrapper that fuzzes a maintainer's tool — itself
  a candidate adapter), `KNOB-ERGONOMICS-AND-PRESETS` (the knob profiles a
  multi-adapter sweep uses).
