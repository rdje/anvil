---
id: acceptance-divergence-hunting
title: ANVIL's acceptance-divergence finder is a shared, default-off detector — per-tool accept/warn/reject verdicts + a divergence classifier over the existing run_verilator/run_yosys/run_iverilog primitives — surfaced as a hunt detection axis, a tool_matrix column, and an MCP controlled tool, with tool-version-vs-version as a later increment
answers:
  - "how does ANVIL detect when one tool accepts and another rejects"
  - "does ANVIL have an acceptance divergence finder"
  - "what is acceptance divergence hunting"
  - "can ANVIL compare verilator and yosys accept reject verdicts"
  - "does ANVIL detect tool-version-vs-version divergence"
  - "what is a DivergenceReport"
  - "how does ANVIL classify a tool verdict accept warn reject"
  - "is acceptance divergence a hunt detection axis"
  - "does the divergence detector reuse run_verilator run_yosys run_iverilog"
  - "how is acceptance divergence surfaced over MCP"
  - "is there a tool_matrix divergence column"
  - "what counts as an acceptance divergence finding"
  - "how does ANVIL differ from --diff-sim for finding tool bugs"
  - "where does ANVIL retain a divergent artifact reproducer"
date: 2026-06-17
status: accepted
tags: [usability, divergence, downstream, acceptance, mcp, api, minimize, reproducer, diff-sim, north-star, orchestration, agent]
evidence: docs/tasks/ACCEPTANCE-DIVERGENCE-HUNTING.md (the owning tree; this is its .1 design leaf); docs/decisions/0017-api-first-everything-mcp-accessible.md (the API-completeness gate this lane must satisfy); docs/decisions/0018-bug-hunt-orchestration-loop.md (the hunt engine this detector plugs into — its .1 names ACCEPTANCE-DIVERGENCE-HUNTING as the natural next detector); docs/decisions/0004-agent-introspection-mcp-lane.md (the controlled-tool sandbox/allow-list/RAM-guard/audit discipline + the no-shadow-simulator ceiling); docs/decisions/0011-semantic-introspection-derived-query-surface.md (the SCHEMA-DERIVED projection discipline the verdicts/report obey); src/downstream/mod.rs (the EXISTING primitives this composes — AcceptanceTool{Verilator,Yosys,Iverilog}+from_name+binary, YosysMode{WithoutAbc,WithAbc,Both}+yosys_mode_slug, ToolInvocation{tool,argv,success,exit_code,stdout_log,stderr_log,error}, run_verilator/run_yosys/run_iverilog_compile (+ _design variants), first_tool_warning folding a warning into success=false, generate_dut_artifact(cfg)->(kind,top,sv), introspect_dut_artifact, validate(seed,cfg,&ValidateOptions)->ValidateReport{run_id,lane,kind,top,sandbox,tools,ok,declined}, MinimizeOptions/MinimizeReport, ValidateOptions{tools,yosys_mode,mem_limits,sandbox_root,keep_sandbox}); src/hunt/mod.rs (the inline accept/warn/reject classifier in hunt::run to be extracted, the HuntRequest.diff_sim precedent for an optional detection axis, HuntFailure.detection string, the reproducer-bundle emitter write_bundle/HuntBundle); src/diff_sim/mod.rs (the DIFFERENTIAL-SIMULATION.3b.1 + BUG-HUNT-ORCHESTRATION.2a extract-then-reuse precedent — run_agreement + DiffSimReport — and the complementary semantic-agreement axis); src/bin/tool_matrix.rs (ModuleReport.verilator:Option<ToolInvocation>/yosys:Vec<ToolInvocation>/iverilog_compile/diff_sim:Option<DiffSimReport> column precedent + select_diff_sim_subset/classify_diff_sim_axis representative subset selector + the saw_* CoverageSummary facts); src/mcp/mod.rs (tools_list/tools_call dispatch, run_validate/run_minimize/run_hunt controlled-tool shims, the artifact cache + anvil://audit/log, the analyze named-query registry, -32602 on bad input); src/introspect/mod.rs (content_run_id FNV-1a addressing, SCHEMA_VERSION "1.11", IntrospectionDocument)
---

# 0019 - ACCEPTANCE-DIVERGENCE-HUNTING: acceptance divergence is a first-class, default-off, SCHEMA-DERIVED detector shared by the hunt loop, the matrix, and MCP

- Date: 2026-06-17
- Status: accepted (design accepted; implementation pending under the pre-split
  `ACCEPTANCE-DIVERGENCE-HUNTING.2`)
- Tree: `ACCEPTANCE-DIVERGENCE-HUNTING.1` (design/decision leaf; no code — pins the
  divergence model, the verdict/report shape, the three surfaces, the
  reproducer-retention policy, the tool-version-vs-version axis, and the
  sandbox/reproducibility discipline; pre-splits `.2`)
- Activated by: autonomous PNT selection (`2026-06-17`) — the owner-recommended
  usability lane 2, named in decision [`0018`](0018-bug-hunt-orchestration-loop.md)
  as *"the natural next detector"* that plugs into the just-completed hunt engine.
- Binds: decision [`0017`](0017-api-first-everything-mcp-accessible.md) (every
  control MCP-settable, every action MCP-invocable, every result queryable, all
  documented) and decision [`0004`](0004-agent-introspection-mcp-lane.md) (the
  controlled-tool sandbox/allow-list/RAM-guard/audit discipline + the
  no-shadow-simulator ceiling). Complements
  [`DIFFERENTIAL-SIMULATION`](../tasks/DIFFERENTIAL-SIMULATION.md) (`--diff-sim`,
  cross-*simulator* trace agreement) with the orthogonal *acceptance* axis.

## Context

ANVIL's reason to exist is to **surface downstream-tool bugs** with RTL that is
legal by construction (`project_anvil_north_star`). Two artifacts that are both
*valid SystemVerilog* but get **different verdicts from two independent tools**
(one accepts, another rejects; or one warns where another is clean) is one of the
sharpest places a real downstream-tool bug lives — exactly because ANVIL's output
is valid by construction, *every* disagreement is a tool's fault, not the RTL's.

`--diff-sim` (`DIFFERENTIAL-SIMULATION`, extracted to `diff_sim::run_agreement` by
`BUG-HUNT-ORCHESTRATION.2a`) already proves cross-*simulator* **trace** agreement:
"do two simulators compute the same values?". This lane adds the complementary,
*earlier-in-the-pipeline* axis: cross-tool **acceptance** agreement — "do two
tools (or two versions of one tool) agree on whether the artifact is even legal?".
The two axes are orthogonal: diff-sim needs both tools to *accept* first
(`tool_matrix` runs it only after Verilator + Yosys are clean), whereas a divergence
*is* the case where they do not.

Every primitive already exists — the lane is a **composition**, not a new engine:

- `src/downstream/mod.rs` already exposes the per-tool invocation primitives
  `run_verilator` / `run_yosys` / `run_iverilog_compile` (+ `_design` variants),
  each returning a `ToolInvocation { tool, argv, success, exit_code, stdout_log,
  stderr_log, error }`, plus the `AcceptanceTool { Verilator, Yosys, Iverilog }`
  allow-list (`from_name` / `binary`) and `YosysMode { WithoutAbc, WithAbc, Both }`
  (`yosys_mode_slug`). `first_tool_warning` already recognises a warning per tool
  (Verilator `%Warning-`, Yosys `Warning:`, iverilog `warning:`) and `validate`
  folds it into `ok = false`.
- `src/hunt/mod.rs` already classifies a `ToolInvocation` into **reject** (non-zero
  exit) vs **warning** (clean exit but `!success`) **inline** in `hunt::run`
  (its `HuntFailure.detection` string) — the exact trinary an acceptance-divergence
  verdict needs.
- `src/bin/tool_matrix.rs` already records a per-unit per-tool acceptance matrix
  (`ModuleReport.verilator: Option<ToolInvocation>`, `.yosys: Vec<ToolInvocation>`,
  `.iverilog_compile`, `.diff_sim: Option<DiffSimReport>`) and selects a
  representative cross-axis subset (`select_diff_sim_subset` / `classify_diff_sim_axis`)
  capped at 5, deterministically. It records `saw_*` coverage facts.
- `src/mcp/mod.rs` already dispatches `validate` / `minimize` / `hunt` as controlled
  tools, caches each finding's content-addressed `run_id`
  (`anvil://artifact/<run_id>/{sv,introspection}`), appends to `anvil://audit/log`,
  and exposes a vetted named-query `analyze` registry.
- `src/introspect/mod.rs` already content-addresses every `(seed, knobs)` to a
  reproducible `run_id` (`content_run_id`, FNV-1a).

What is missing is the **divergence verdict + classifier + report**, and its three
surfaces (the hunt loop, the matrix column, the MCP tool). The user/agent should
not have to diff three tool logs by hand.

## Decision

**Acceptance divergence is a first-class, default-off, SCHEMA-DERIVED detector
that lives in one shared library entry and is reused by every surface.** It adds
**no** generation path, **no** behavioural oracle, and **no** second source of
truth — it *projects* the existing `ToolInvocation`s into per-tool verdicts and
*classifies* their disagreement. The default `anvil` build and `--artifact dut`
stay byte-identical; the detector is opt-in everywhere it appears.

### 1. The divergence model (the verdict + the classifier)

**A per-tool verdict is a trinary projection of one `ToolInvocation`** — the same
classification `hunt::run` already does inline:

| Verdict | Condition (SCHEMA-DERIVED from `ToolInvocation`) |
| --- | --- |
| `accept` | `success == true` (clean exit, no warning) |
| `warn` | `exit_code == Some(0)` **and** `success == false` (clean exit, but `first_tool_warning` fired — ANVIL output is warning-clean by construction, so any warning is a finding) |
| `reject` | non-zero / unknown exit (`exit_code != Some(0)`) |

To avoid a **second classifier** (full-factorization doctrine — `feedback_full_factorization`),
the inline accept/warn/reject logic in `hunt::run` is **extracted** into a shared
`downstream::tool_verdict(&ToolInvocation) -> ToolVerdict` (the extract-then-reuse
precedent set by `BUG-HUNT-ORCHESTRATION.2a` lifting `diff_sim::run_agreement`).
Both the hunt's `detection` string and the divergence verdict then derive from the
one classifier; `hunt` behaviour stays byte-identical.

**A *divergence* over a set of verdicts for one artifact** is "not all verdicts are
equal". The classifier reports the disagreement kind by the strongest axis present:

- `accept_reject` — the headline: ≥1 tool accepts and ≥1 rejects (a tool is wrong
  about legality).
- `accept_warn` — ≥1 accepts cleanly, ≥1 warns (a lint/severity divergence).
- `warn_reject` — ≥1 warns, ≥1 rejects.
- `version_mismatch` — same tool *kind*, two pinned versions, different verdict
  (the `.2e` tool-version axis; see §5).

The unit of comparison is a **labelled tool**: `verilator`, `yosys-without-abc`,
`yosys-with-abc`, `iverilog` (and, at `.2e`, `verilator@<label>` etc.). Yosys
`--yosys-mode both` therefore contributes *two* labelled verdicts, so a
without-abc/with-abc disagreement is itself a divergence — a real signal already
seen in the repo's history (ABC-flow warnings on valid designs).

### 2. The detector + the report shape (`DivergenceReport`, beside `DiffSimReport`/`HuntReport`)

The detector lands as a **library composer** in `src/divergence/mod.rs`
(symmetry with `src/diff_sim/` and `src/hunt/`, which compose `src/downstream/`
primitives), exposing one pure entry point:

```text
divergence::run(seed, cfg, &DivergenceOptions) -> Result<DivergenceReport>
```

It regenerates the DUT via the shared `downstream::generate_dut_artifact(cfg)`
(the exact artifact `validate` accepts), then runs **every enabled tool/mode to
completion** — *not* folding to one `ok` and *not* short-circuiting on the first
reject, because divergence needs every verdict — through the existing
`run_verilator` / `run_yosys` / `run_iverilog_compile` (`_design` for a design),
classifies each via `downstream::tool_verdict`, and classifies the disagreement.

`DivergenceOptions` mirrors `ValidateOptions`'s discipline exactly
(`tools: Vec<AcceptanceTool>` (≥2 to diverge), `yosys_mode: YosysMode`,
`mem_limits: MemLimits`, `sandbox_root: PathBuf` (caller-set), `keep_sandbox: bool`;
`.2e` adds the version `tool_specs`).

The report is a **SCHEMA-DERIVED** projection — every field comes from an existing
struct; no new computed truth:

```jsonc
DivergenceReport {
  "run_id": "…",            // content_run_id(seed, knobs) — reproducible
  "lane": "dut",
  "kind": "design|module",
  "top": "…",
  "sandbox": "…",
  "verdicts": [             // one per labelled tool — projection of each ToolInvocation
    { "tool": "verilator",        "verdict": "accept", "exit_code": 0,  "first_message": null },
    { "tool": "yosys-without-abc","verdict": "reject", "exit_code": 1,  "first_message": "<reject line>" }
  ],
  "diverged": true,
  "divergences": [          // one per disagreeing pair-class; empty when all agree
    { "kind": "accept_reject", "tools": ["verilator", "yosys-without-abc"] }
  ],
  "declined": null          // RAM-guard decline (MemGuard), same as ValidateReport.declined
}
```

`ToolVerdict { tool, verdict, exit_code, first_message }` is a projection of one
`ToolInvocation` (`tool`, the classifier output, `exit_code`, and the first
reject/warning line from `error`). `Divergence { kind, tools }` names the
disagreeing labels. Nothing here is a behavioural oracle: the tools' own verdicts
remain the source of truth, and ANVIL only *classifies* their disagreement
(decision `0004`, ROADMAP steering gap 4).

### 3. Three surfaces, one detector (decision `0017`)

The single `divergence::run` is reused by all three, so there is one detector and
no drift:

1. **A hunt detection axis** — `HuntRequest` gains an optional `divergence: bool`
   (the `diff_sim: bool` precedent). When set, `hunt::run` calls `divergence::run`
   on each swept artifact; a `diverged == true` result is a finding with
   `detection = "acceptance_divergence"`, `HuntFailure.divergence:
   Option<DivergenceReport>` carrying the report, and **no minimize** by default
   (the `validate` oracle proves "this single tool fails", not "these two tools
   *disagree*" — like the `cross_sim_mismatch` axis). This realises decision
   `0018`'s promise: "the hunt loop is the engine it will plug into."
2. **A `tool_matrix` column** — `ModuleReport`/`DesignReport` gain `divergence:
   Option<DivergenceReport>` (the `diff_sim` column precedent), a
   `--divergence` opt-in, a `saw_acceptance_divergence` coverage fact, and the
   representative subset reuses `classify_diff_sim_axis`.
3. **An MCP controlled tool `divergence`** — because it spawns external tools it is
   *controlled* (inherits the `validate`/`hunt` allow-list/sandbox/RAM-guard/audit),
   not a pure `analyze` query. Input schema mirrors `validate` + the sweep
   (`lane`/`seed`/`seeds`/`config`/`tools`/`yosys_mode`); it returns the
   `DivergenceReport`(s), caches each divergent `run_id` so
   `anvil://artifact/<run_id>/{sv,introspection}` resolve (via
   `introspect_dut_artifact`), and appends a top-level `divergence` audit record.
   The `anvil` CLI exposes it as a shim (a `--divergence` flag on `anvil hunt`
   and/or the `tool_matrix` column), never as a superset.

### 4. The reproducer-retention policy (reuse, no new format)

A divergence finding **retains the divergent artifact as a reproducer by reusing
the existing emitters** — no new bundle format:

- riding the hunt loop, it reuses `BUG-HUNT-ORCHESTRATION.2b.2b`'s `write_bundle`
  (`<bundle_root>/<run_id>/` with `repro.sv`/`knobs.json`/`introspection.json`/
  `tool-logs/`/`repro.sh`), with `hunt-verdict.json` carrying the
  `DivergenceReport` and `repro.sh` recording each labelled tool's `argv` so the
  *disagreement* re-runs, not just one side;
- as the `tool_matrix` column, it reuses the harness's existing per-scenario `.sv`
  retention + each tool's captured log;
- over MCP, the divergent `run_id` is served from the artifact cache (no on-disk
  bundle — the agent never supplies a path, decision `0004`).

### 5. Tool-version-vs-version is a *later* increment (`.2e`)

The first cut is **multi-tool same-version** divergence (Verilator vs Yosys vs
Yosys-abc vs iverilog) — the portable, higher-leverage axis that needs no extra
install. Version-vs-version (`verilator@5.046` vs `verilator@5.040`) needs the
caller to pin two *binaries*, which is environment-dependent, so it lands after,
mirroring how the hunt shipped reject/warning first and folded cross-sim in later.
The model: a labelled tool is `(AcceptanceTool kind, resolved binary, observed
version)`; the **kind stays allow-listed** (`AcceptanceTool::from_name` —
verilator/yosys/iverilog only), but the *binary* may be a caller-supplied path/PATH
shim for that kind. `ToolInvocation` gains an observed-`version` capture (parsed
from `--version`); ANVIL never manages tool installs — the caller supplies the
binaries and labels (resolves the tree's first open question).

### 6. The honesty boundary (steady state = all tools agree)

On valid-by-construction RTL the **steady state is that all tools agree (accept)** —
a divergence would be a genuine downstream-tool bug, the thing the lane exists to
*surface*, not a fixture. So `saw_acceptance_divergence` is an **opportunistic**
fact, **never a required coverage gate** (a gate requiring it would fail on clean
RTL, which is the normal case). The gates instead prove the **matrix/report is
produced, correctly classified, and queryable** — including via a synthetic
`ToolInvocation` set in a cargo-portable unit test (an injected accept/reject pair)
and an all-agree real-tool run that records the matrix with `diverged == false`.
This mirrors `BUG-HUNT-ORCHESTRATION.2e`'s honest e2e design exactly.

### 7. The sandbox / reproducibility discipline (decision `0004`)

Inherited wholesale by composing through the `downstream` primitives: seeded
ChaCha8 (no wall-clock / no `thread_rng`; same `(seed, knobs)` ⇒ same `run_id`);
allow-listed spawns (`AcceptanceTool::from_name`; unknown ⇒ clean error, never a
spawn); a per-run caller-set sandbox temp dir; `MemGuard`/`MemLimits` decline
surfaced as `declined`; `anvil://audit/log` records; default-off / byte-identical.

## Pre-split of `ACCEPTANCE-DIVERGENCE-HUNTING.2` (implementation)

Ordered sub-leaves (refinable at pick time; each default-off / DUT byte-identical,
each carrying the decision-`0017` API-completeness gate):

- `.2a` — **extract** the inline accept/warn/reject classifier from `hunt::run`
  into a shared `downstream::tool_verdict(&ToolInvocation) -> ToolVerdict` (pure
  refactor; hunt behaviour byte-identical; the `.2a`-of-hunt precedent). Proves
  `hunt::` unchanged + snapshots 6/6.
- `.2b` — the `src/divergence/` **library core**: `ToolVerdict` / `Divergence` /
  `DivergenceReport` / `DivergenceOptions` + `divergence::run` composing
  `generate_dut_artifact` + `run_verilator`/`run_yosys`/`run_iverilog_compile`
  (all enabled tools/modes to completion) + the shared classifier + the divergence
  classifier (multi-tool, same version). Cargo-portable proofs incl. a synthetic
  accept/reject `ToolInvocation` set ⇒ `accept_reject` divergence, and a no-tools
  run ⇒ friendly no-op. No CLI/MCP/version axis yet.
- `.2c` — fold the detector into **`hunt::run`** (`HuntRequest.divergence: bool` →
  `divergence::run` on each artifact → an `acceptance_divergence` finding +
  `HuntFailure.divergence`) **and** add the **`tool_matrix` column**
  (`--divergence`, `ModuleReport`/`DesignReport.divergence`, the
  `saw_acceptance_divergence` opportunistic fact, the `classify_diff_sim_axis`
  subset reuse). Cargo-portable proofs; snapshots 6/6.
- `.2d` — the **MCP `divergence` controlled tool** (input schema, `DivergenceReport`
  result, divergent-`run_id` cache population, audit record) + the `anvil` CLI shim;
  `book/src/agent-mcp.md` tool list/table; proofs (decision `0017` gate met).
- `.2e` — the **tool-version-vs-version** axis: `DivergenceOptions.tool_specs`
  (`(kind, binary, label)`) + a `ToolInvocation` observed-`version` capture +
  `version_mismatch` classification; portability note (caller supplies binaries).
- `.2f` — a **real-tool end-to-end gate** (`#[ignore]`, tool-gated) proving the
  matrix is produced + classified (all-agree steady state + a synthetic-injected
  divergence is classified) and queryable; `book/src/synthesizability.md` +
  `book/src/agent-mcp.md` + USER_GUIDE + README + a KM card; close `.2` and the tree.

## Rejected alternatives

- **A new bespoke per-tool warning/reject parser in the divergence detector.**
  Rejected: `first_tool_warning` + the exit code already classify; reusing the
  extracted `tool_verdict` keeps one source of truth and matches `hunt`'s detection
  byte-for-byte (`feedback_full_factorization`; no second classifier).
- **A pure `analyze` query instead of a controlled tool.** Rejected: divergence
  *runs external tools*, so it must inherit the controlled-tool
  allow-list/sandbox/RAM-guard/audit (decision `0004`). `analyze` is for pure IR
  traversals; this is an action.
- **A behavioural oracle to decide *which* tool is "right".** Forbidden (decision
  `0004`, gap 4). The detector classifies the disagreement; the tools remain the
  source of truth. ANVIL never adjudicates.
- **Making it hunt-only, or matrix-only, or MCP-only.** Rejected: the tree's open
  question ("rides the hunt loop, or an independent matrix column?") resolves to
  **both**, via one shared `divergence::run` — so there is exactly one detector and
  no drift, and decision `0017` (MCP-invocable + queryable + CLI-shim) is satisfied.
- **A new reproducer-bundle format for divergences.** Rejected: reuse
  `BUG-HUNT-ORCHESTRATION.2b.2b`'s `write_bundle` (hunt path) and the
  `tool_matrix` `.sv`+log retention (matrix path); only `repro.sh` records *each*
  labelled tool's `argv` so the disagreement reproduces. Nothing new, nothing
  retired.
- **Requiring `saw_acceptance_divergence` as a coverage gate.** Rejected as
  dishonest: on valid-by-construction RTL the steady state is all-agree, so a
  required-divergence gate would fail on clean output. The gate proves the matrix
  is produced/classified/queryable; a found divergence is the (opportunistic) win.
- **Folding warning into reject (dropping the trinary).** Rejected: `validate`'s
  binary `ok` is right for "is this a finding?", but divergence needs to distinguish
  `accept`/`warn`/`reject` so an accept-vs-warn lint divergence is visible, not
  collapsed into accept-vs-reject.
- **Version pinning by ANVIL-managed tool installs.** Rejected: ANVIL never manages
  installs; the caller supplies binaries/labels, the kind stays allow-listed.

## Consequences

- The implementation (`.2`) lands a shared `downstream::tool_verdict` +
  `src/divergence/` (`divergence::run` + `DivergenceReport`) + three thin surfaces
  (a hunt axis, a `tool_matrix` column, an MCP `divergence` tool) + the
  version-vs-version increment — all composition over proven surfaces, default-off /
  DUT byte-identical.
- The extracted `tool_verdict` classifier is a reuse win beyond this lane:
  `hunt::run`'s detection and any future consumer share one accept/warn/reject
  definition.
- ANVIL gains a second downstream-bug *detector* axis (acceptance divergence)
  alongside the cross-sim trace axis, both plugging into the one hunt engine — the
  north star deepens without a new engine.
- The introspection schema grows additively (per its MINOR policy) only if a
  `DivergenceReport` projection is also served as a resource; the MCP tool/audit
  registries absorb the new controlled tool without breaking determinism.
- The structure-first / no-shadow-simulator ceiling is restated: "everything
  queryable" here means the classified verdicts, not a judgement of correctness.

## Links

- Owning tree: `ACCEPTANCE-DIVERGENCE-HUNTING` (this is its `.1` design leaf;
  pre-splits `.2a`…`.2f`).
- Parent decisions: `0017` (API-completeness gate), `0004` (MCP lane +
  sandbox/allow-list/RAM-guard/audit + no-shadow-simulator), `0011`
  (SCHEMA-DERIVED projection discipline), `0018` (the hunt engine this detector
  plugs into; it named this lane).
- Composed surfaces: `src/downstream/mod.rs` (`run_verilator`/`run_yosys`/
  `run_iverilog_compile` + `_design`, `ToolInvocation`, `AcceptanceTool`,
  `YosysMode`, `first_tool_warning`, `generate_dut_artifact`,
  `introspect_dut_artifact`, `ValidateOptions`), `src/hunt/mod.rs` (the inline
  classifier to extract + the `diff_sim` optional-axis precedent + `write_bundle`),
  `src/diff_sim/mod.rs` (the extract-then-reuse + complementary-axis precedent),
  `src/bin/tool_matrix.rs` (the per-tool column + `classify_diff_sim_axis` subset +
  `saw_*` facts), `src/mcp/mod.rs` (controlled-tool dispatch + cache + audit),
  `src/introspect/mod.rs` (`content_run_id`).
- Book: `book/src/synthesizability.md` (the acceptance-subset discipline this
  stresses) + `book/src/agent-mcp.md` (the agent surfaces) — updated at `.2`.
- Memory: `project_anvil_north_star` (surface downstream-tool bugs),
  `feedback_api_for_agents_not_humans` (design the API for agents),
  `feedback_full_factorization` (one classifier, not two),
  `feedback_never_retire_strategies` (nothing retired).
- Synergistic lanes: `BUG-HUNT-ORCHESTRATION` (the engine — done),
  `DOWNSTREAM-ADAPTER-EXPANSION` (more tools to diverge across),
  `KNOB-ERGONOMICS-AND-PRESETS` (the knob profiles a divergence sweep uses),
  `CI-PACKAGING-DISTRIBUTION` (a CI wrapper that watches for divergences).
