# API Reference: Resources & Prompts

Beyond [tools](api-tools.md) (actions), the `anvil-mcp` server exposes two more
MCP capability kinds: **resources** (read-only data, addressed by URI) and
**prompts** (parameterized workflow templates).

## Resources

Resources are read-only blobs addressed by an `anvil://…` URI. List them with
`resources/list` and fetch one with `resources/read`:

```json
{ "jsonrpc": "2.0", "id": 1, "method": "resources/read",
  "params": { "uri": "anvil://catalog/knobs" } }
```

The response wraps the content:

```json
{ "jsonrpc": "2.0", "id": 1, "result": {
    "contents": [ { "uri": "anvil://catalog/knobs",
                    "mimeType": "application/json", "text": "{ … }" } ] } }
```

`resources/read` requires a `uri`; a missing `uri` or an unknown/unresolvable URI
is a protocol error (`-32602`).

### Static resources (always present)

| URI | mimeType | Content |
| --- | --- | --- |
| `anvil://catalog/knobs` | `application/json` | the default `Config` — the full knob taxonomy with default values (the starting point you edit and pass back as a `config` argument) |
| `anvil://catalog/lanes` | `application/json` | the artifact lane catalog: `{ "default": "dut", "lanes": [ {name, description} … ] }` for `dut` / `microdesign` / `frontend` |
| `anvil://audit/log` | `application/json` | the append-only audit trail of every `validate` / `minimize` / `hunt` / `divergence` call (reproducible `run_id`, seed, and exact command lines) |

### Artifact resources (per cached `run_id`)

When you `generate` an artifact (or it is cached by `hunt` / `divergence` /
`analyze`), it becomes addressable by its content-addressed `run_id`. These
appear in `resources/list` once cached:

| URI | mimeType | Content | Present when |
| --- | --- | --- | --- |
| `anvil://artifact/<run_id>/sv` | `text/x-systemverilog` | the emitted SystemVerilog | always (any cached artifact) |
| `anvil://artifact/<run_id>/introspection` | `application/json` | the introspection document for the artifact | always |
| `anvil://artifact/<run_id>/manifest` | `application/json` | the lane's expected-facts manifest | **microdesign / frontend** lanes only |
| `anvil://artifact/<run_id>/analysis/<query>` | `application/json` | a derived-relation analysis (`output_support` / `input_reach` / `flop_reset_provenance` / `module_reachability`) | once that `analyze` `query` has run on the artifact |

Because artifacts are content-addressed, `resources/read` on an artifact URI
always returns the same bytes for the same `(seed, knobs)`. This is how an agent
reads a reproducer back without ever handling a filesystem path — the MCP path
never exposes one (decision `0004`).

## Prompts

Prompts package the agent loops end-to-end. Each is a parameterized template the
agent fetches with `prompts/get` and then executes by calling the [tools](api-tools.md)
in the order the prompt lays out. **A prompt adds no capability and computes no
new truth** — it is guidance that wires the existing tools into a workflow.

List them with `prompts/list`; render one with `prompts/get`:

```json
{ "jsonrpc": "2.0", "id": 1, "method": "prompts/get",
  "params": { "name": "explain_artifact", "arguments": { "seed": "42" } } }
```

```json
{ "jsonrpc": "2.0", "id": 1, "result": {
    "description": "…",
    "messages": [ { "role": "user", "content": { "type": "text", "text": "…" } } ] } }
```

**Prompt argument rules.** MCP prompt arguments are **strings** (note `"seed":
"42"` above, not `42`). A non-string argument, a missing **required** argument,
or an unknown prompt name is a protocol error (`-32602`).

### The five workflows

| Prompt | Arguments (required\*) | Tool chain |
| --- | --- | --- |
| `find_downstream_bug` | `seed?` (42), `tools?` (verilator,yosys), `yosys_mode?` (without-abc) | `generate` → `validate` → *(on failure)* `minimize` → read `anvil://audit/log` |
| `close_coverage_gap` | `target`\*, `seed?` (42) | `anvil://catalog/knobs` → `dump_config` → raise the gating knob + `introspect` to confirm the metric lit → `validate` |
| `minimize_reproducer` | `seed`\*, `tools?` (verilator,yosys), `yosys_mode?` (without-abc) | `minimize` → inspect `reductions` / surviving failures → read `anvil://audit/log` |
| `triage_tool_failures` | `seed?` (42), `tools?` (verilator,yosys), `yosys_mode?` (without-abc) | `validate` → classify the failing tool/mode from its `argv` + output → `anvil://audit/log` |
| `explain_artifact` | `seed?` (42) | `generate` → `introspect` (construction-truth) → read the `sv` resource → summarize |

(\* = required argument; the rest are optional with the default shown.)

For example, `prompts/get explain_artifact` with `seed = 42` renders a single
`user` message:

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

---

See also: [Tools](api-tools.md) for the actions the prompts orchestrate, and
[Introspection & Analysis Schemas](api-introspection.md) for the document shapes
they read.
