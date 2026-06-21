# Driving anvil from an AI Agent (Introspection + MCP)

`anvil`'s whole reason for existing is to surface **downstream-tool bugs**
with RTL that is legal by construction. The fastest way to do that at scale is
a loop:

> generate → validate against Verilator / Yosys / iverilog → on a rejection,
> shrink `(seed, knobs)` to a minimal reproducer → file it.

This chapter is about handing that loop to an **AI agent**. `anvil` exposes a
thin, read-mostly *introspection* surface and an [MCP][mcp] server so an agent
can run the loop itself — without ever touching the deterministic generator
core.

Everything here is **opt-in and default-off**. The plain `anvil` build, and
the byte-for-byte `--artifact dut` contract, are completely unaffected. If you
never use these features, nothing changes.

> **This chapter is the tutorial.** For the precise, machine-readable contract —
> every method, parameter, result, resource, prompt, and error — see the formal
> **API Reference**: [Overview & Protocol](api-reference.md) ·
> [Tools](api-tools.md) · [Resources & Prompts](api-resources-prompts.md) ·
> [Introspection & Analysis Schemas](api-introspection.md).

[mcp]: https://modelcontextprotocol.io

## The two entry points

| Surface | What it is | Use it when |
| --- | --- | --- |
| `anvil --introspect` | A one-shot CLI flag that prints a structured JSON **introspection document** instead of SystemVerilog. | You want construction-truth for one `(seed, knobs)` from a script. |
| `anvil-mcp` | A separate binary: a small [MCP][mcp] server (JSON-RPC over **stdio** by default, or **HTTP** with `--http`) exposing tools, resources, and workflow prompts. | You want an AI agent (Claude Code, Cursor, …) to drive the loop. |

Both read the **same facts** ANVIL already records — metrics, the effective
config, coverage. Neither computes anything new. (The full field-by-field
contract is [`docs/AGENT_INTROSPECTION_SCHEMA.md`](https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md);
the architecture is decision record `0004`.)

## `anvil --introspect`

Add `--introspect` to a **single-artifact** run (no `--out`, `--count 1`) and
ANVIL prints a versioned JSON document describing what it built:

```bash
cargo run --release -- --seed 42 --introspect
```

```json
{
  "schema_version": "1.11",
  "anvil_version": "0.1.0",
  "lane": "dut",
  "request": {
    "seed": 42,
    "lane": "dut",
    "knobs": { "...": "the full effective Config" },
    "run_id": "3f1cad578805bd04"
  },
  "artifact": {
    "kind": "module",
    "top": "mod_42_0000",
    "sv": { "uri": "anvil://artifact/3f1cad578805bd04/mod_42_0000.sv", "bytes": 80383 },
    "sv_sha256": null,
    "manifest": null
  },
  "introspection": {
    "module_metrics": { "avg_fanout": "...", "gates_by_kind": { "...": "..." } }
  },
  "warnings": [
    "coverage section absent: single-artifact generate, not a tool_matrix run"
  ]
}
```

Things worth knowing:

- **`run_id` is a content address**, not a random nonce. It is a hash of
  `(schema_version, anvil_version, lane, seed, knobs)`. Same inputs ⇒ same
  `run_id` ⇒ the document is reproducible and cacheable. Re-run the command and
  you get the exact same bytes.
- **The `.sv` is a pointer, not inlined.** Bulk output is fetched deliberately
  (as a resource over MCP, or just generate it directly without `--introspect`).
- **`module_metrics`** here *is* `metrics::compute(&module)` — the same metrics
  the manifest already carries — re-projected under a stable key. A `design`
  run carries `design_metrics` and a per-child `modules` list instead.
- **`coverage` is absent** for a single artifact: a lone module can't prove a
  `saw_recursive_hierarchy_*` coverage fact. Coverage is a property of a
  `tool_matrix` sweep, and the document says so in `warnings`.

`--introspect` is additive: omit it and you get SystemVerilog exactly as
before.

## `anvil-mcp`: the MCP server

`anvil-mcp` is a separate binary that speaks newline-delimited JSON-RPC 2.0 on
stdio — the transport Claude Code and Cursor use. Build it and run it:

<!-- book-test: skip — builds/runs the separate anvil-mcp binary (a long-lived stdio server), not the generator CLI -->
```bash
cargo build --release --bin anvil-mcp
./target/release/anvil-mcp     # reads JSON-RPC from stdin, writes responses to stdout
```

You normally don't talk to it by hand — you register it with your agent. For
example, in Claude Code:

<!-- book-test: skip — external `claude` CLI with a placeholder path; agent-setup illustration -->
```bash
claude mcp add anvil -- /path/to/anvil/target/release/anvil-mcp
```

Once connected, the agent sees three kinds of capability: **tools** (actions),
**resources** (read-only data), and **prompts** (packaged workflows).

### Transports: stdio (default) and HTTP

`anvil-mcp` speaks the same JSON-RPC protocol over two transports:

- **stdio** (the default) — newline-delimited JSON-RPC on stdin/stdout, the
  transport Claude Code and Cursor register. Nothing extra to enable.
- **HTTP** (opt-in, `--http <addr>`) — one JSON-RPC request per HTTP `POST`,
  for agents or scripts that prefer a socket. It is a tiny hand-rolled HTTP/1.1
  transport (no extra dependencies) driving the *exact same* dispatcher, so
  every tool, resource, and prompt behaves identically.

`<addr>` is either a **bare port** — which binds loopback (`127.0.0.1:<port>`),
the safe default — or a full `IP:PORT`:

<!-- book-test: skip — starts the long-lived anvil-mcp HTTP server, not the generator CLI -->
```bash
cargo build --release --bin anvil-mcp
./target/release/anvil-mcp --http 8765        # binds 127.0.0.1:8765 (loopback)
```

Then POST JSON-RPC to it — each call is one request:

<!-- book-test: skip — talks to a running anvil-mcp HTTP server over the network -->
```bash
curl -s -X POST http://127.0.0.1:8765/ \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

> **Security.** The controlled `validate` / `minimize` tools run real downstream
> tools, so the HTTP transport **binds loopback by default**. Binding a
> non-loopback address (e.g. `0.0.0.0:8765`) exposes those tools to anyone who
> can reach the socket; `anvil-mcp` prints a warning when you do, and you should
> only do it on a network you trust. The per-call guardrails (fixed allow-list,
> sandboxed temp dir, RAM guard, audit log) apply on both transports.

### Tools

```
generate · introspect · analyze · dump_config · coverage_gaps · validate · minimize · hunt · divergence
```

| Tool | Pure? | What it does |
| --- | --- | --- |
| `generate` | ✅ pure | Build the `(seed, config)` artifact for a `lane` (default `dut`), cache it, return its `run_id` + resource URIs. |
| `introspect` | ✅ pure | Return the versioned introspection document (config echo + metrics) for that `lane`. |
| `analyze` | ✅ pure | Answer a derived-**relation** query over the DUT `(seed, config)` IR by pure graph traversal. `query` = `output_support` (the default): each target's transitive combinational fan-in **support cone** (*what does this output depend on?*). `query` = `input_reach`: the **dual fan-out** (*what does this source reach?*). `query` = `flop_reset_provenance`: per-flop **reset/data provenance** (*is this register reset-defined, and how is its next state built?*). `query` = `module_reachability`: which modules in a design are **reachable** from the top via the instance graph (*what's in this design's module tree, and what's dead?*). Relations, not behaviour. |
| `dump_config` | ✅ pure | Return the effective `Config` after validation. |
| `coverage_gaps` | ✅ pure | Project the already-computed `coverage_gaps` out of a recorded `tool_matrix_report.json` (inline `report` **or** `report_path`) — *what is not yet exercised* — so the agent can steer generation at the dark surfaces. Read-only: no generation, no tool spawn, no recompute. |
| `validate` | controlled | Generate into a sandboxed temp dir and run the selected vetted tools (`verilator` / `yosys` / `iverilog` / `sv2v` / `slang`); return per-tool reports + an overall verdict. |
| `minimize` | controlled | Delta-debug a failing `(seed, config)` to a smaller reproducer, using `validate` as the failure oracle. |
| `hunt` | controlled | The **turnkey loop**: fuzz a deterministic seed sweep (`seed` .. `seed + seeds`), run the vetted tools on each artifact, detect any reject/warning (and, with `diff_sim`, a cross-simulator trace mismatch; and, with `divergence`, a cross-*tool* acceptance disagreement), auto-`minimize` each failure, and return a structured `HuntReport` (per-seed `verdicts` + `failures` + `summary`). A thin shim over `validate`/`minimize` — no detector or minimizer of its own. Each failing `run_id` is cached, so `anvil://artifact/<run_id>/{sv,introspection}` resolve for the reproducer. Also available as the `anvil hunt` CLI subcommand (the same `hunt::run` over the command line, where `--out` drops an on-disk reproducer bundle per finding instead — see the User Guide). |
| `divergence` | controlled | **Acceptance-divergence detector**: generate the `(seed, config)` artifact, run the selected vetted tools on it, and classify whether they **disagree** on its legality — one accepts while another warns or rejects. On legal-by-construction RTL every such disagreement is a real downstream-tool bug. Returns a `DivergenceReport` (per-tool `accept`/`warn`/`reject` verdicts + the classified `divergences`, e.g. `accept_reject`). The complement of `diff_sim`'s cross-*simulator* **trace** axis — this is the cross-*tool* **acceptance** axis. `yosys_mode = both` contributes two labelled verdicts, so a without-abc-vs-with-abc disagreement is itself a divergence. Each divergent `run_id` is cached, so `anvil://artifact/<run_id>/{sv,introspection}` resolve. A single-`(seed, config)` shim over the same detector the `hunt` `divergence` axis uses — to sweep many seeds, call `hunt` with `divergence: true`. |

The five **pure** tools are read-only: no generation side effects, no shell, no
external tools. (`coverage_gaps` may read a report file you point it at, but it
*runs* nothing — it relays the gap list `tool_matrix` already computed, so the
two can never drift; `analyze` only traverses the IR the generator already
produced — relations, never a behavioural simulation.) The four *controlled*
tools (`validate`, `minimize`, the `hunt` loop that composes them, and the
`divergence` detector) run real downstream tools, but only through ANVIL's
existing hardened invocations:

- a **fixed allow-list** of tool names (`verilator`, `yosys`, `iverilog`,
  `sv2v`, `slang`) — an unknown name is a clean error, never a spawn. (`sv2v`
  is the first adapter added beyond the three originals: an `sv2v`
  SystemVerilog→Verilog-2005 **transpile** accept/reject column — a clean
  transpile accepts, a non-zero exit or a warning is a finding. `slang` is the
  second, and the first **fact-bearing** adapter: a strict, fast, independent
  SystemVerilog elaborator — a clean elaboration accepts, a warning/reject is a
  finding — that *additionally* projects a SCHEMA-DERIVED `--ast-json` view of
  the top's ports + child instances (the optional `extract_facts` hook;
  `supports_facts = true` in the catalog). Both are absent on
  most hosts today, so selecting them is a friendly no-op until the binary is on
  `PATH`; the adapter catalog's `present` field tells you which tools are
  installed.);
- a **sandboxed** per-run temp directory (the agent never supplies a path);
- the **RAM guard** declines to start more work under memory pressure;
- **no arbitrary shell** is ever exposed;
- every call is **audit-logged** with its reproducible `(run_id, seed)` and the
  exact command line (see the `anvil://audit/log` resource).

`minimize` searches only the **input** `(seed, knobs)` space and holds the seed
fixed — it never mutates or "repairs" emitted RTL. That would violate
valid-by-construction; ANVIL stays the source of truth.

The `divergence` detector also has a **tool-version-vs-version** axis (one
allow-listed tool *kind*, two caller-supplied binaries — e.g. `verilator-5.046`
vs `verilator-4.228` — classified `version_mismatch`). That axis is a **library
surface only** and is deliberately **not** exposed over MCP or the CLI: an
allow-listed kind paired with an *arbitrary caller-supplied binary path* is a
larger trust surface than the fixed-binary controlled tools (decision `0004`),
so exposing it would need its own trust-boundary design (an operator-configured
version-binary registry, never an agent-supplied path). The agent-facing
`divergence` tool runs the fixed allow-list, same as `validate`.

### Resources

Static catalogs, the audit log, and every artifact you've generated this
session:

```
anvil://catalog/knobs          the default Config (the knob taxonomy)
anvil://catalog/lanes          the artifact lanes (dut / microdesign / frontend)
anvil://catalog/adapters       the downstream adapter catalog (id/binary/present/supports_facts)
anvil://catalog/presets        the --profile preset registry (name/description/overrides)
anvil://catalog/knob-schema    the per-knob catalog (name/group/type/default/validation/cli_flag/config_only)
anvil://audit/log              the append-only validate/minimize/hunt/divergence audit trail
anvil://artifact/<run_id>/sv               the emitted SystemVerilog
anvil://artifact/<run_id>/introspection    the introspection document
anvil://artifact/<run_id>/manifest         the lane's expected-facts manifest (microdesign / frontend)
anvil://artifact/<run_id>/analysis/<query> a derived-relation analysis (output_support / input_reach / flop_reset_provenance / module_reachability)
```

Because artifacts are content-addressed, `generate` then `resources/read
anvil://artifact/<run_id>/sv` always returns the same bytes.

### Derived-relation queries: `analyze`

`analyze` answers *what does this output structurally depend on?* over the DUT
IR — a **relation**, derived by pure graph traversal, never a behavioural
simulation (anvil has no shadow simulator by doctrine). The first query kind,
`output_support` (the default), returns each target's transitive **combinational
fan-in support cone**:

```json
{ "name": "analyze", "arguments": { "seed": 7, "query": "output_support", "target": "o_0" } }
```

A reply (a `DerivedAnalysisDocument` — the same envelope as `introspect`, with an
`analysis` payload):

```json
{
  "schema_version": "1.11",
  "lane": "dut",
  "request": { "seed": 7, "run_id": "…" },
  "analysis": {
    "query": "output_support",
    "results": [
      {
        "target": "o_0",
        "support_inputs": ["i_1"],
        "support_flops": [],
        "support_instance_outputs": [],
        "cone_nodes": 3,
        "cone_depth": 2
      }
    ]
  }
}
```

- `target` addresses an **output port name**, or a flop's `D` input as
  `"flop:<id>"`; omit it to get a cone for **every** output.
- `support_inputs` / `support_flops` / `support_instance_outputs` are the
  combinational support **leaves** the target depends on. A flop `Q` is a
  register boundary (the cone stops there; query `"flop:<id>"` for what feeds its
  `D`); a child-instance output stops at the instance boundary.
- `cone_nodes` is the number of distinct fan-in nodes; `cone_depth` is the
  combinational gate depth.
- An unknown `query` or `target` is rejected with JSON-RPC `-32602`.

The result is cached and also served as the
`anvil://artifact/<run_id>/analysis/output_support` resource. Every field is a
pure projection of the IR the generator already built — no new computed truth
(the `SCHEMA-DERIVED` invariant).

#### `input_reach` — the dual fan-out

The second query kind, `input_reach`, is the **transpose** of `output_support`:
instead of *what does this output depend on?* it answers *what does this source
reach?* The `target` is a **source** — an input port name, a flop `Q` as
`"flop:<id>"`, or a child-instance output `"<instance>.<port>"` (omit for every
source):

```json
{ "name": "analyze", "arguments": { "seed": 7, "query": "input_reach", "target": "i_1" } }
```

```json
{
  "schema_version": "1.11",
  "lane": "dut",
  "request": { "seed": 7, "run_id": "…" },
  "analysis": {
    "query": "input_reach",
    "reach_results": [
      {
        "target": "i_1",
        "reaches_outputs": ["o_0"],
        "reaches_flops": [],
        "fanout_targets": 1
      }
    ]
  }
}
```

- The payload is a separate `reach_results` array (not `results`), so an
  `output_support` reply is byte-identical to before the `1.4 → 1.5` bump — the
  `reach_results` key is simply absent there.
- `reaches_outputs` / `reaches_flops` are the outputs and flop `D`-cones the
  source reaches; `fanout_targets` is their total. It is computed by **inverting**
  the support cones, so a source reaches a target exactly when that target's
  support cone lists it — `output_support` and `input_reach` cannot drift.
- `"flop:<id>"` as a *source* is the flop's **Q** (its fan-out); as an
  `output_support` *target* it is the flop's **D** cone. Same register boundary,
  opposite direction — the `query` kind chooses.
- Served as `anvil://artifact/<run_id>/analysis/input_reach`; unknown source →
  `-32602`.

#### `flop_reset_provenance` — per-flop reset/data provenance

The third query kind, `flop_reset_provenance`, answers *is each flop
reset-defined or data-driven, and how is its next state built?* It is a direct
projection of each `Flop` — no graph walk. The `target` is `"flop:<id>"` (omit
for every flop):

```json
{ "name": "analyze", "arguments": { "seed": 7, "query": "flop_reset_provenance" } }
```

```json
{
  "schema_version": "1.11",
  "lane": "dut",
  "request": { "seed": 7, "run_id": "…" },
  "analysis": {
    "query": "flop_reset_provenance",
    "flop_provenance": [
      {
        "flop": 0,
        "width": 8,
        "has_reset": true,
        "reset_kind": "async",
        "reset_value": "0",
        "default_behavior": "zero",
        "mux_kind": "one_hot",
        "mux_arms": 2,
        "has_d": true
      }
    ]
  }
}
```

- The payload is a third `flop_provenance` array (not `results` / `reach_results`),
  again `skip_serializing_if`, so `output_support` / `input_reach` replies stay
  byte-identical across the `1.5 → 1.6` bump.
- `reset_kind` is `none` / `sync` / `async`; `default_behavior` is `zero`
  (load 0 when no select asserted) or `hold` (keep `Q`); `mux_kind` is `none` /
  `one_hot` / `encoded` with `mux_arms` the arm count; `reset_value` is the reset
  value as a **decimal string** (exact for 128-bit values).
- Served as `anvil://artifact/<run_id>/analysis/flop_reset_provenance`; an unknown
  `"flop:<id>"` → `-32602`. A flopless module yields an empty result (no flops to
  report), not an error.

#### `module_reachability` — which modules are reachable from the top

The fourth query kind, `module_reachability`, answers *which modules in a design
are reachable from `design.top` via the instance graph, and how does each one sit
in it?* It is a pure min-depth BFS over `Design.modules` + the
`Module.instances[].module` edges — no gate-graph walk. The `target` is a **module
name** (omit for every module). It is most useful on a hierarchy design (a single
combinational/sequential DUT is one module, reported as a trivial root at depth 0).
`config` is the full effective `Config` (from `dump_config`); the hierarchy knobs
shown are what make the artifact a design:

```json
{ "name": "analyze", "arguments": { "seed": 42, "config": { "hierarchy_depth": 1, "num_leaf_modules": 2, "num_child_instances": 2 }, "query": "module_reachability" } }
```

```json
{
  "schema_version": "1.11",
  "lane": "dut",
  "request": { "seed": 42, "run_id": "…" },
  "artifact": { "kind": "design", "top": "top" },
  "analysis": {
    "query": "module_reachability",
    "module_reachability": [
      { "module": "child_a", "reachable": true, "depth": 1, "instantiates": [], "instance_count": 0 },
      { "module": "child_b", "reachable": true, "depth": 1, "instantiates": [], "instance_count": 0 },
      { "module": "top", "reachable": true, "depth": 0, "instantiates": ["child_a", "child_b"], "instance_count": 2 }
    ]
  }
}
```

(module names illustrative.)

- The payload is a fourth `module_reachability` array (not `results` /
  `reach_results` / `flop_provenance`), again `skip_serializing_if`, so the prior
  three replies stay byte-identical across the `1.6 → 1.7` bump.
- Each entry is one module: `reachable` (from the top), `depth` (the minimum
  instance-graph distance from the top — `0` for the top; present only when
  reachable), `instantiates` (the distinct child module names it directly
  instantiates, sorted) and `instance_count` (its direct-instance count, `>=
  instantiates.len()` when a child is instantiated more than once). Entries are
  sorted by module name; a dead (unreachable) module is reported `reachable: false`
  with no `depth`.
- Unlike the prior three queries, `target` is a **module name** (not a port name or
  `"flop:<id>"`). Served as
  `anvil://artifact/<run_id>/analysis/module_reachability`; an unknown module name →
  `-32602`.

### All three lanes, not just DUT

`generate` and `introspect` take an optional `lane` argument (`dut` —
the default — `microdesign`, or `frontend`), so the agent can drive all three
artifact families through the same tools. The non-DUT lanes take their own
scoped knobs (`n_params`, and `n_children` for `frontend`) instead of the DUT
`Config`, and each carries a deterministic **expected-facts manifest** — the
same one the Phase 7/8 parity gates check. That manifest is both inlined in the
introspection document (`microdesign_manifest` / `frontend_manifest`) and served
as the `anvil://artifact/<run_id>/manifest` resource:

```json
{ "name": "generate", "arguments": { "lane": "microdesign", "seed": 7, "n_params": 4 } }
```

The DUT lane has no semantic manifest (its check plan is synthesis acceptance,
not parity), so `anvil://artifact/<run_id>/manifest` is absent for `dut`.

### Prompts (workflows)

The five **prompts** package the agent loops end-to-end. Each is a
parameterized template the agent fetches with `prompts/get` and then executes
by calling the tools above in the order the prompt lays out. A prompt adds no
capability and computes no new truth — it is guidance that wires the existing
tools into a workflow.

| Prompt | Arguments | Chain |
| --- | --- | --- |
| `find_downstream_bug` | `seed?`, `tools?`, `yosys_mode?` | generate → validate → *(on failure)* minimize → read audit log |
| `close_coverage_gap` | `target` (required), `seed?` | knobs catalog → dump_config → raise the gating knob + introspect to confirm the metric lit → validate |
| `minimize_reproducer` | `seed` (required), `tools?`, `yosys_mode?` | minimize → inspect reductions / surviving failures → read audit log |
| `triage_tool_failures` | `seed?`, `tools?`, `yosys_mode?` | validate → classify the failing tool/mode from its `argv` + output → audit log |
| `explain_artifact` | `seed?` | generate → introspect (construction-truth) → read the `.sv` resource → summarize |

For example, `prompts/get explain_artifact` with `seed = 42` renders:

```text
Explain a generated artifact from construction-truth — ANVIL records
structure/provenance by construction, so read those facts instead of parsing
the SV.

Run this tool chain in order:
1. `generate` { "seed": 42 } -> `run_id`, `kind`, `top`.
2. `introspect` { "seed": 42 } -> read `artifact`, `config`, and
   `introspection.module_metrics` / `introspection.design_metrics`; these are
   ground truth.
3. `resources/read` `anvil://artifact/<run_id>/sv` -> the emitted SystemVerilog,
   if you need the source.
4. Summarize: lane, top module, width/depth/flop/motif structure, and which
   knobs shaped it. Do not claim whole-module intended behavior — ANVIL
   generates legal structure, not a spec.
```

## The bug-hunting loop, end to end

The loop, step by step:

1. **`generate`** a DUT for a seed.
2. **`validate`** it against `verilator` + `yosys`.
3. If the verdict is **`ok`**, the RTL is downstream-clean — pick another seed
   and repeat.
4. If a vetted tool **rejected** (or warned on) valid-by-construction RTL, that is
   a candidate **downstream-tool bug** (not an ANVIL bug). **`minimize`** shrinks
   the knobs to a small reproducer, holding the seed fixed.
5. Read **`anvil://audit/log`** for the exact, reproducible command lines and
   file it.

Because ANVIL's output is legal by construction, *a rejection is the signal*.
The agent is an experiment driver and explainer; it is **never** a signoff
oracle — ANVIL's manifests, metrics, and the real tools' results remain the
source of truth.

### One command: the `hunt` loop

You do not have to wire those steps together by hand. The **`hunt`** tool runs
the whole loop — fuzz a deterministic seed sweep → detect any reject/warning
(and, with `diff_sim`, a cross-simulator trace mismatch) → auto-`minimize` each
failure → emit a reproducer — and returns one structured `HuntReport`
(per-seed `verdicts` + `failures` + `summary`). It is a thin shim over
`validate`/`minimize`; it adds no detector and no minimizer of its own.

It is available **two ways, both shims over the same loop** (decision `0017`):

- the **MCP `hunt` tool** — an agent calls it with the sweep controls and reads
  each failing reproducer back as an `anvil://artifact/<run_id>/{sv,introspection}`
  resource (the cache is populated for every finding); the sweep is recorded in
  `anvil://audit/log`;
- the **`anvil hunt` CLI subcommand** — the same loop from the shell, printing the
  `HuntReport` JSON to stdout, with `--out <dir>` additionally dropping a
  self-contained reproducer bundle directory per finding.

<!-- book-test: skip — runs the real downstream tools (verilator/yosys); the hunt loop is tool-gated -->
```bash
# Fuzz seeds 1..16 against Verilator + Yosys; print a JSON HuntReport.
anvil hunt --seed 1 --seeds 16 --tools verilator,yosys

# Hunt a knob profile, dropping a reproducer bundle per finding.
anvil --seed 1 --dump-config > profile.json
anvil hunt --seeds 64 --config profile.json --tools verilator --out ./hunt-bundles
```

Each `--out` bundle is a directory `<run_id>/` carrying everything needed to
re-file the bug independently: `repro.sv` (the emitted RTL), `knobs.json` (the
effective/minimized `Config`), `introspection.json` (construction truth),
`hunt-verdict.json` (the finding), `tool-logs/`, and a one-command `repro.sh`
that regenerates the `.sv` from `(seed, knobs.json)` and re-runs the failing
tool. Because ANVIL is reproducible, `repro.sh` reproduces the artifact
byte-for-byte. (Full CLI flags: the User Guide's *`anvil hunt`* section.)

Since ANVIL's output is legal by construction, a clean sweep (`n_failures = 0`)
is the **expected** result — `hunt` is the engine that surfaces the rare
genuine downstream-tool bug, not a generator of ANVIL failures.

## What this lane deliberately does *not* do

- **No new generation path and no output repair.** The agent drives the
  existing rules-first generator; it never mutates or filters emitted RTL.
- **No second source of truth.** The introspection schema is a *projection* of
  the existing `metrics` / `config` / manifest / coverage structs. If a metric
  isn't already computed by construction, the agent can't see it here.
- **No stateful simulator session.** ANVIL is a pure `(seed, knobs) → artifact`
  function; there is no `run_until`, no `force_signal`, no waveform DB. (See
  [non-goals](non-goals.md).)
- **No arbitrary shell, and no effect on the default build.** Controlled tools
  run only the fixed allow-list, sandboxed and RAM-guarded; the default `anvil`
  build and `--artifact dut` stay byte-identical.

## Where to look next

- The wire contract, field by field: [`docs/AGENT_INTROSPECTION_SCHEMA.md`](https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md).
- The architecture and the simulator-advice transfer analysis: decision record
  `0004` (`docs/decisions/0004-agent-introspection-mcp-lane.md`).
- The knob taxonomy the agent tunes: [Knobs and Reproducibility](knobs.md).
- What ANVIL is for, in one page: [The Core Idea](core-idea.md).
