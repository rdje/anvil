# API Reference: Tools

The `anvil-mcp` server exposes **9 tools**, called with `tools/call`:

```json
{ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
  "params": { "name": "<tool>", "arguments": { … } } }
```

A successful call returns a JSON-RPC `result` whose `content[0].text` is the
(stringified) JSON document the tool produced, with `isError: false`; a failed
call returns `isError: true` with a message (see [the error model](api-reference.md#the-error-model)).
For readability the examples below show the **parsed** result document, not the
text-content wrapper.

Tools are either **pure** (read-only — no generation side effects, no shell, no
external tools) or **controlled** (they run real downstream tools through the
fixed allow-list, sandboxed + RAM-guarded + audit-logged).

| Tool | Class | One line |
| --- | --- | --- |
| [`generate`](#generate) | pure | build + cache the `(seed, lane, knobs)` artifact, return its `run_id` + resource URIs |
| [`introspect`](#introspect) | pure | the versioned introspection document for `(seed, lane, knobs)` |
| [`dump_config`](#dump_config) | pure | the effective `Config` after validation |
| [`analyze`](#analyze) | pure | a derived-relation query over the DUT IR graph |
| [`coverage_gaps`](#coverage_gaps) | pure | project the gap list out of a recorded `tool_matrix` report |
| [`validate`](#validate) | controlled | run the vetted tools on one artifact; per-tool reports + verdict |
| [`minimize`](#minimize) | controlled | delta-debug a failing `(seed, knobs)` to a smaller reproducer |
| [`hunt`](#hunt) | controlled | the turnkey fuzz → detect → minimize sweep |
| [`divergence`](#divergence) | controlled | classify cross-tool acceptance disagreement |

Common argument types used throughout:

- **`seed`** — `integer ≥ 0`, the deterministic RNG seed.
- **`config`** — `object`, a full effective `Config` exactly as
  [`dump_config`](#dump_config) emits it. Omit for defaults. (DUT lane only.)
- **`tools`** — `array` of `"verilator" | "yosys" | "iverilog" | "sv2v"`. A
  **fixed allow-list** — no arbitrary commands or binary paths. Default
  `["verilator", "yosys"]`. (`sv2v` is an `sv2v` SystemVerilog→Verilog-2005
  transpile accept/reject column; absent on most hosts today, so selecting it is
  a friendly no-op until `sv2v` is installed — check the
  [`anvil://catalog/adapters`](./api-resources-prompts.md) `present` field.)
- **`yosys_mode`** — `"without-abc" | "with-abc" | "both"`, default
  `"without-abc"`. `both` runs Yosys twice (two labelled invocations).

---

## Pure tools

### `generate`

Generate an artifact for `(seed, lane, knobs)`, cache it, and return its
content-addressed `run_id` and resource URIs.

**Parameters**

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `seed` | integer ≥ 0 | no | `0` | deterministic seed |
| `lane` | `"dut"` \| `"microdesign"` \| `"frontend"` | no | `"dut"` | artifact lane |
| `config` | object | no | defaults | **DUT lane only** |
| `n_params` | integer ≥ 0 | no | `5` | **microdesign / frontend**: parameter/localparam count |
| `n_children` | integer ≥ 0 | no | `2` | **frontend**: child-instance count |

**Result** — the artifact descriptor: the `run_id`, the `kind`
(`"module"` / `"design"`), the `top` name, and the `anvil://artifact/<run_id>/…`
resource URIs (`sv`, `introspection`, and `manifest` for the non-DUT lanes).

**Example**

```json
{ "name": "generate", "arguments": { "seed": 42 } }
```
```json
{ "run_id": "3f1cad578805bd04", "lane": "dut", "kind": "module",
  "top": "mod_42_0000",
  "resources": {
    "sv": "anvil://artifact/3f1cad578805bd04/sv",
    "introspection": "anvil://artifact/3f1cad578805bd04/introspection" } }
```

### `introspect`

Return the versioned **introspection document** for `(seed, lane, knobs)` — the
config echo plus metrics (DUT), or the lane-manifest resource pointer
(microdesign / frontend). Same parameters as [`generate`](#generate). Every
field is `SCHEMA-DERIVED`. Full envelope: [Introspection & Analysis
Schemas](api-introspection.md).

```json
{ "name": "introspect", "arguments": { "seed": 42 } }
```
```json
{ "schema_version": "1.11", "anvil_version": "0.1.0", "lane": "dut",
  "request": { "seed": 42, "lane": "dut", "knobs": { "…": "Config" },
               "run_id": "3f1cad578805bd04" },
  "artifact": { "kind": "module", "top": "mod_42_0000",
                "sv": { "uri": "anvil://artifact/3f1cad578805bd04/mod_42_0000.sv",
                        "bytes": 80383 } },
  "introspection": { "module_metrics": { "…": "Metrics" } },
  "warnings": [ "coverage section absent: single-artifact generate" ] }
```

### `dump_config`

Return the **effective `Config`** for `(seed, config)` after validation (defaults
filled in, overrides applied). The canonical way to obtain a `config` object to
pass to the other tools.

**Parameters:** `seed`, `config` (both optional).

```json
{ "name": "dump_config", "arguments": { "seed": 42 } }
```
```json
{ "seed": 42, "min_width": 1, "max_width": 64, "flop_prob": 0.3, "…": "…" }
```

### `analyze`

Answer a derived-**relation** query over the DUT `(seed, config)` IR by pure
graph traversal — relations, never behaviour (no shadow simulator). The result
schemas are documented on [Introspection & Analysis Schemas](api-introspection.md);
this entry covers the call surface.

**Parameters**

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `seed` | integer ≥ 0 | no | `0` | |
| `config` | object | no | defaults | DUT lane only |
| `query` | `"output_support"` \| `"input_reach"` \| `"flop_reset_provenance"` \| `"module_reachability"` | no | `"output_support"` | the relation kind |
| `target` | string | no | all | meaning depends on `query` (below) |

`target` by query: `output_support` → an output port name or `"flop:<id>"` (a
flop D-cone); `input_reach` → a source (input name, `"flop:<id>"` Q, or
`"<instance>.<port>"`); `flop_reset_provenance` → `"flop:<id>"`;
`module_reachability` → a module name. Omit `target` for *every* element.

**Errors** — an unknown `query` or `target` → protocol error `-32602`.

**Result** — a `DerivedAnalysisDocument` (the `introspect` envelope with an
`analysis` payload). Cached + also served as
`anvil://artifact/<run_id>/analysis/<query>`.

```json
{ "name": "analyze",
  "arguments": { "seed": 7, "query": "output_support", "target": "o_0" } }
```
```json
{ "schema_version": "1.11", "lane": "dut", "request": { "seed": 7, "run_id": "…" },
  "analysis": { "query": "output_support",
    "results": [ { "target": "o_0", "support_inputs": ["i_1"],
                   "support_flops": [], "support_instance_outputs": [],
                   "cone_nodes": 3, "cone_depth": 2 } ] } }
```

### `coverage_gaps`

Project the already-computed coverage-gap list out of a recorded
`tool_matrix_report.json`. **Read-only**: no generation, no tool spawn, no
recompute — the single gap computation stays in `tool_matrix`.

**Parameters** (provide exactly one):

| Name | Type | Required | Notes |
| --- | --- | --- | --- |
| `report` | object | one of | the parsed `tool_matrix_report.json`, inline (no filesystem access) |
| `report_path` | string | one of | path to a `tool_matrix_report.json`, read + parsed read-only (never executed) |

**Result** — the recorded `coverage_gaps` array, a `gap_count`, the dark `saw_*`
coverage facts (recorded booleans still `false`), and the downstream tool
pass/fail counts.

```json
{ "name": "coverage_gaps", "arguments": { "report_path": "./tool-matrix/tool_matrix_report.json" } }
```
```json
{ "gap_count": 0, "coverage_gaps": [], "dark_facts": [],
  "verilator": { "pass": 1005, "fail": 0 } }
```

---

## Controlled tools

All four run real downstream tools through the **fixed allow-list**
(`verilator` / `yosys` / `iverilog` / `sv2v`), generate into a **sandboxed** per-run temp
dir (the agent never supplies a path), let the **RAM guard** decline to start
more work under memory pressure, expose **no arbitrary shell**, and **audit-log**
every call to `anvil://audit/log`. On ANVIL's valid-by-construction RTL the
steady state is *acceptance*; a rejection is a candidate **downstream-tool bug**,
never an ANVIL bug — and these tools never mutate or repair RTL.

### `validate`

Generate the `(seed, config)` DUT artifact into a sandbox and run the selected
vetted tools on it; return structured per-tool reports + an overall verdict.

**Parameters:** `seed`, `config`, `tools` (default `["verilator","yosys"]`),
`yosys_mode` (default `"without-abc"`).

**Result** — a `ValidateReport`: `run_id`, `lane`, `kind`, `top`, `sandbox`, a
`tools` array of per-tool `ToolInvocation`s (`tool`, `argv`, `success`,
`exit_code`, `stdout_log`/`stderr_log`, `error`), the overall `ok`, and a
`declined` reason if the RAM guard stopped early. Yosys `both` yields two entries.

```json
{ "name": "validate", "arguments": { "seed": 42, "tools": ["verilator", "yosys"] } }
```
```json
{ "run_id": "3f1c…", "lane": "dut", "kind": "module", "top": "mod_42_0000",
  "sandbox": "/tmp/anvil-validate-3f1c…",
  "tools": [ { "tool": "verilator", "success": true, "exit_code": 0, "error": null },
             { "tool": "yosys-without-abc", "success": true, "exit_code": 0, "error": null } ],
  "ok": true, "declined": null }
```

### `minimize`

Delta-debug a failing `(seed, config)` to a smaller failing reproducer, using
`validate` as the failure oracle: shrink size bounds and disable optional motifs
while a downstream tool still rejects the artifact. Deterministic, **seed held
fixed** (it pins the reproducer), budget-bounded. It searches only the *input*
`(seed, knobs)` space — it never mutates emitted RTL.

**Parameters**

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `seed` | integer ≥ 0 | no | `0` | held fixed |
| `config` | object | no | defaults | the failing knob profile |
| `tools` | array | no | `["verilator","yosys"]` | the oracle tools |
| `yosys_mode` | enum | no | `"without-abc"` | |
| `max_oracle_calls` | integer ≥ 1 | no | `200` | hard ceiling on `validate` evaluations |

**Result** — a report with `reproduced_initial` (`false` ⇒ the case is
downstream-clean, nothing to shrink), `reductions` (which knobs shrank), and
`final_validation` (the surviving failing-tool reports), plus the minimized
`run_id`.

```json
{ "name": "minimize", "arguments": { "seed": 99, "config": { "…": "the failing Config" } } }
```

### `hunt`

The **turnkey loop**: fuzz a deterministic seed sweep, run the vetted tools on
each artifact, detect any reject/warning (and, with `diff_sim`, a cross-simulator
trace mismatch; and, with `divergence`, a cross-tool acceptance disagreement),
auto-`minimize` each failure, and return a structured `HuntReport`. A thin shim
over `validate`/`minimize` — no detector or minimizer of its own. Each failing
`run_id` is cached so `anvil://artifact/<run_id>/{sv,introspection}` resolve.

**Parameters**

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `seed` | integer ≥ 0 | no | config's seed | base seed; the sweep fuzzes `seed .. seed+seeds` |
| `seeds` | integer ≥ 1 | no | `16` | number of consecutive seeds |
| `config` | object | no | defaults | the knob profile every seed uses |
| `tools` | array | no | `["verilator","yosys"]` | |
| `yosys_mode` | enum | no | `"without-abc"` | |
| `minimize` | boolean | no | `true` | auto-minimize each failure |
| `max_oracle_calls` | integer ≥ 1 | no | `200` | per-failure minimize ceiling |
| `diff_sim` | boolean | no | `false` | also check cross-simulator trace agreement (no-op if a simulator is absent) |
| `divergence` | boolean | no | `false` | also classify cross-tool acceptance divergence on each finding |

**Result** — a `HuntReport`: `lane`, `base_seed`, `seeds`, a `verdicts` array
(per-seed `run_id` + verdict), a `failures` array (each with `seed`, `run_id`,
`failing_tool`, `detection`, the minimized reproducer, …), and a `summary`
(`n_clean`, `n_failures`). A clean sweep (`n_failures = 0`) is the expected
steady state.

```json
{ "name": "hunt", "arguments": { "seed": 1, "seeds": 16, "tools": ["verilator","yosys"] } }
```
```json
{ "lane": "dut", "base_seed": 1, "seeds": 16,
  "verdicts": [ { "seed": 1, "run_id": "…", "ok": true }, "…" ],
  "failures": [], "summary": { "n_clean": 16, "n_failures": 0 } }
```

### `divergence`

The **acceptance-divergence detector**: generate the `(seed, config)` DUT
artifact, run the selected vetted tools on it, and classify whether they
**disagree** on its legality — one accepts while another warns/rejects. On
legal-by-construction RTL a disagreement is a real downstream-tool bug. The
complement of `diff_sim`'s cross-*simulator* trace axis (this is the cross-*tool*
acceptance axis). Each divergent `run_id` is cached. To sweep many seeds, call
[`hunt`](#hunt) with `divergence: true`.

**Parameters:** `seed`, `config`, `tools` (default `["verilator","yosys"]`;
**≥ 2 labelled tools** must run for a divergence to be possible — `yosys_mode:
"both"` alone yields two labels), `yosys_mode` (default `"without-abc"`; `both`
contributes two labelled verdicts, so a without-abc-vs-with-abc disagreement is
itself a divergence).

> The tool-version-vs-version axis (one tool *kind*, two caller-supplied
> binaries) is a **library surface only** — deliberately **not** exposed here,
> because an allow-listed kind with an arbitrary caller-supplied binary path is a
> larger trust surface than the fixed-binary controlled tools. See
> [Acceptance divergence across tools](synthesizability.md#acceptance-divergence-across-tools).

**Result** — a `DivergenceReport`: `run_id`, `lane`, `kind`, `top`, `sandbox`, a
`verdicts` array of per-tool `accept`/`warn`/`reject` decisions, `diverged`, the
classified `divergences` (each `{ kind, tools }` with `kind` =
`accept_reject` / `accept_warn` / `warn_reject`), and `declined`.

```json
{ "name": "divergence", "arguments": { "seed": 42, "tools": ["verilator","yosys"], "yosys_mode": "both" } }
```
```json
{ "run_id": "3f1c…", "lane": "dut", "kind": "module", "top": "mod_42_0000",
  "sandbox": "/tmp/anvil-validate-3f1c…",
  "verdicts": [ { "tool": "verilator", "verdict": "accept", "exit_code": 0 },
                { "tool": "yosys-without-abc", "verdict": "accept", "exit_code": 0 },
                { "tool": "yosys-with-abc", "verdict": "accept", "exit_code": 0 } ],
  "diverged": false, "divergences": [] }
```

---

See also: [Resources & Prompts](api-resources-prompts.md) for the `anvil://…`
data each tool caches, and [Introspection & Analysis Schemas](api-introspection.md)
for the `introspect` / `analyze` document shapes.
