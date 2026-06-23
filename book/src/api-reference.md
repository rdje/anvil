# API Reference: Overview & Protocol

This is the **formal reference** for ANVIL's agent/automation API. The
[previous chapter](agent-mcp.md) is the narrative tutorial — *why* the API
exists and how to drive the bug-hunting loop. These reference pages are the
precise contract: every method, every parameter, every result, every error.

The API is **designed for machine consumers** (AI agents, CI scripts), so this
reference is written to be machine-precise: parameters are given as JSON Schema,
results as concrete shapes, and every tool carries a worked request → response
example. It is **accurate to the code** — the tool schemas here are the schemas
`tools/list` returns (source of truth: `src/mcp/mod.rs`).

Everything documented here is **opt-in and default-off**. The plain `anvil`
build and the byte-for-byte `--artifact dut` contract are unaffected.

## The two entry points

| Surface | Transport | Use it for |
| --- | --- | --- |
| `anvil --introspect` | one-shot CLI (stdout) | construction-truth for one `(seed, knobs)` from a script — see [Introspection & Analysis Schemas](api-introspection.md). |
| `anvil-mcp` | JSON-RPC 2.0 (stdio / HTTP) | an agent driving tools, resources, and prompts — the rest of this reference. |

Both return the **same facts** ANVIL already records (metrics, the effective
config, coverage, IR relations). Neither computes anything new — the
**`SCHEMA-DERIVED` invariant** (see [Versioning & stability](#versioning--stability)).

## The reference pages

| Page | Covers |
| --- | --- |
| Overview & Protocol (this page) | the JSON-RPC envelope, transports, lifecycle methods, the error model, content-addressing, versioning |
| [Tools](api-tools.md) | the 10 tools: `generate`, `introspect`, `dump_config`, `analyze`, `coverage`, `coverage_gaps`, `validate`, `minimize`, `hunt`, `divergence` |
| [Resources & Prompts](api-resources-prompts.md) | the `anvil://…` resource URIs and the 5 workflow prompts |
| [Introspection & Analysis Schemas](api-introspection.md) | the `--introspect` document, the `analyze` query result schemas, the wire contract |

## Transports

`anvil-mcp` speaks the **same** JSON-RPC 2.0 protocol over two transports; the
dispatcher is identical, so every method behaves the same on both.

- **stdio** (default) — newline-delimited JSON-RPC on stdin/stdout. One JSON
  object per line in, one per line out. This is the transport Claude Code and
  Cursor register.

  <!-- book-test: skip — starts the long-lived anvil-mcp stdio server, not the generator CLI -->
  ```bash
  cargo build --release --bin anvil-mcp
  ./target/release/anvil-mcp        # reads JSON-RPC from stdin, writes to stdout
  ```

- **HTTP** (opt-in, `--http <addr>`) — one JSON-RPC request per HTTP `POST` body.
  `<addr>` is a bare port (binds loopback `127.0.0.1:<port>`, the safe default)
  or a full `IP:PORT`.

  <!-- book-test: skip — starts the long-lived anvil-mcp HTTP server, not the generator CLI -->
  ```bash
  ./target/release/anvil-mcp --http 8765         # binds 127.0.0.1:8765
  ```

  <!-- book-test: skip — POSTs to a running anvil-mcp HTTP server over the network -->
  ```bash
  curl -s -X POST http://127.0.0.1:8765/ \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
  ```

> **Security.** The *controlled* tools (`validate` / `minimize` / `hunt` /
> `divergence`) run real downstream tools, so HTTP **binds loopback by default**.
> Binding a non-loopback address exposes them to anyone who can reach the socket;
> `anvil-mcp` prints a warning when you do. The per-call guardrails (fixed
> allow-list, sandboxed temp dir, RAM guard, audit log) apply on both transports.

## The JSON-RPC envelope

Every request and response is a JSON-RPC 2.0 object.

**Request:**

```json
{ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
  "params": { "name": "generate", "arguments": { "seed": 42 } } }
```

**Success response:**

```json
{ "jsonrpc": "2.0", "id": 1, "result": { "...": "method-specific" } }
```

**Protocol-error response:**

```json
{ "jsonrpc": "2.0", "id": 1, "error": { "code": -32602, "message": "…" } }
```

A request with no `id` (a *notification*) is accepted but produces no response.

## Lifecycle methods

| Method | Params | Result |
| --- | --- | --- |
| `initialize` | (client info; ignored) | `protocolVersion`, `capabilities`, `serverInfo`, `instructions` |
| `ping` | — | `{}` |
| `tools/list` | — | `{ "tools": [ { name, description, inputSchema } … ] }` |
| `tools/call` | `{ name, arguments }` | a **tool result** (see [the error model](#the-error-model)) |
| `resources/list` | — | `{ "resources": [ { uri, name, mimeType } … ] }` |
| `resources/read` | `{ uri }` | `{ "contents": [ { uri, mimeType, text } ] }` |
| `prompts/list` | — | `{ "prompts": [ { name, description, arguments } … ] }` |
| `prompts/get` | `{ name, arguments }` | `{ description, messages }` |

The `initialize` handshake reports the protocol version and the three
capabilities the server implements:

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
  "serverInfo": { "name": "anvil-mcp", "version": "0.1.0" },
  "instructions": "ANVIL agent-introspection. Pure tools: … Controlled tools: …"
}
```

## The error model

ANVIL distinguishes **two layers** of error, exactly as the MCP spec intends:

### Protocol errors (the JSON-RPC `error` field)

The request itself was malformed or unroutable. These appear in the response's
`error` field with a JSON-RPC code:

| Code | Name | When |
| --- | --- | --- |
| `-32700` | parse error | the request body was not valid JSON |
| `-32601` | method not found | an unknown `method` |
| `-32602` | invalid params | a missing/ill-typed argument; an unknown `resources/read` URI; an unknown prompt or a missing required prompt argument; an unknown `analyze` `query` or `target` |

```json
{ "jsonrpc": "2.0", "id": 7, "error": { "code": -32602,
  "message": "analyze: unknown target \"o_99\"" } }
```

### Tool-execution errors (`isError` inside a successful result)

A `tools/call` that routed correctly but whose tool *failed* returns a **normal
JSON-RPC `result`** whose content carries `isError: true`. This is the MCP
convention: the call succeeded at the protocol layer; the tool reports its own
failure in-band so the agent can read the message and react.

```json
{ "jsonrpc": "2.0", "id": 3, "result": {
    "content": [ { "type": "text", "text": "tool not on allow-list: vcs" } ],
    "isError": true } }
```

A tool-level error fires for, e.g., a tool name off the fixed allow-list, an
unknown tool, or a generation/serialization failure. A *successful* tool result
has the same shape with `isError: false`; its `text` is the (stringified) JSON
document the tool produced — parse that `text` to get the structured result.

```json
{ "jsonrpc": "2.0", "id": 2, "result": {
    "content": [ { "type": "text", "text": "{\"run_id\":\"3f1cad578805bd04\", …}" } ],
    "isError": false } }
```

## Content-addressing & determinism

Every artifact is identified by a **content-addressed `run_id`**: a hash of
`(schema_version, anvil_version, lane, seed, knobs)`. Same inputs ⇒ same `run_id`
⇒ byte-identical output, forever (seeded ChaCha8; no wall-clock, no `thread_rng`,
no hash-map iteration in output paths). So `generate` then
`resources/read anvil://artifact/<run_id>/sv` always returns the same bytes, and
a reproducer is fully described by its `(seed, knobs)`.

## Versioning & stability

Two independent version numbers govern compatibility:

- **`protocolVersion`** — the MCP wire protocol, currently **`2024-11-05`**.
  Reported by `initialize`.
- **`schema_version`** — the version of the **introspection / analysis
  documents** (`--introspect`, the `introspect`, `analyze`, and `coverage`
  tools), currently **`1.22`**. It follows a MINOR/MAJOR policy:
  - a **MINOR** bump (e.g. `1.21 → 1.22`) is **additive** — a new optional field or a
    new payload section, with prior replies left byte-identical (new sections use
    `skip_serializing_if`, so a query that doesn't use them is unchanged). `1.12`
    added the `coverage_readout` section + the standalone `coverage` tool document;
    `1.13` added `num_mealy_fsm_modules`; `1.14` added `num_emitted_multi_output_tasks`;
    `1.15`/`1.16`/`1.17` added the `mux_if` / `case_mux_if` / `casez_mux_if`
    emit-projection counts; and the five later `analyze` queries each added one
    payload section — `flop_dependencies` (`1.18`), `memory_provenance` (`1.19`),
    `fsm_provenance` (`1.20`), `node_drivers` (`1.21`), and `node_readers` (`1.22`).
    See [Introspection & Analysis Schemas](api-introspection.md) (and §7 of
    `docs/AGENT_INTROSPECTION_SCHEMA.md`) for the full changelog;
  - a **MAJOR** bump would be a breaking change to an existing field.

Underpinning both is the **`SCHEMA-DERIVED` invariant**: every value the API
returns is a *projection* of something ANVIL already computed by construction —
`Config`, `Metrics` / `DesignMetrics`, the IR graph, the recorded coverage. The
API adds **no new computed truth** and runs **no behavioural oracle / shadow
simulator** (decision `0004` / `0011`). If a fact is not already established by
construction, the API does not invent it.

## Where to go next

- [Tools](api-tools.md) — the 10 callable actions, with full input schemas.
- [Resources & Prompts](api-resources-prompts.md) — the `anvil://…` data and the
  packaged workflows.
- [Introspection & Analysis Schemas](api-introspection.md) — the document
  envelope and the `analyze` result shapes.
- The field-by-field wire contract: [`docs/AGENT_INTROSPECTION_SCHEMA.md`](https://github.com/rdje/anvil/blob/main/docs/AGENT_INTROSPECTION_SCHEMA.md).
