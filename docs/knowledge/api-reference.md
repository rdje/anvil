---
id: api-reference
title: Where ANVIL's agent/automation API is documented — the mdBook API Reference
answers:
  - "where is ANVIL's API documented"
  - "is there an API reference for anvil"
  - "how do I call the anvil-mcp tools"
  - "what tools does the anvil MCP server expose"
  - "what are the anvil-mcp JSON-RPC methods"
  - "what JSON-RPC error codes does anvil-mcp return"
  - "what is the difference between a protocol error and an isError tool result"
  - "what anvil:// resource URIs are there"
  - "what MCP prompts does anvil expose"
  - "what is the anvil introspection schema_version"
  - "what is the anvil-mcp protocolVersion"
  - "how do I read the anvil API contract"
date: 2026-06-17
status: current
tags: [api, mcp, reference, book, introspection, json-rpc, agent, docs]
evidence: book/src/api-reference.md (Overview & Protocol — JSON-RPC envelope, transports, lifecycle methods, error model, versioning); book/src/api-tools.md (the 9 tools with input schemas + examples); book/src/api-resources-prompts.md (resources + the 5 prompts); book/src/api-introspection.md (the --introspect document envelope + the 4 analyze query schemas + the schema_version contract); book/src/agent-mcp.md (the narrative tutorial that links the reference); src/mcp/mod.rs (the source of truth: tools_list / resources_list / prompts / dispatch / error codes); docs/AGENT_INTROSPECTION_SCHEMA.md (the field-by-field wire contract); docs/decisions/0017-api-first-everything-mcp-accessible.md
reverify: 'mdbook build book   (the API Reference pages build clean; their schemas are derived verbatim from src/mcp/mod.rs tools_list / resources_list / prompts and docs/AGENT_INTROSPECTION_SCHEMA.md)'
---

# `BOOK-API-REFERENCE` — where ANVIL's API is documented

ANVIL's **agent/automation API** is documented in the mdBook in two layers
(progressive disclosure, `feedback_book_doctrine`):

- **Tutorial** — `book/src/agent-mcp.md` ("Driving anvil from an AI Agent"):
  *why* the API exists and how to drive the bug-hunting loop.
- **Formal API Reference** (child pages, the machine-precise contract):
  - `book/src/api-reference.md` — **Overview & Protocol**: the two entry points
    (`anvil --introspect` + `anvil-mcp`), the JSON-RPC 2.0 envelope, stdio/HTTP
    transports, the lifecycle methods (`initialize`/`ping`/`tools.*`/
    `resources.*`/`prompts.*`), the **error model** (protocol errors `-32700`
    parse / `-32601` method-not-found / `-32602` invalid-params **vs** tool-level
    `isError: true` results), content-addressing, and the versioning/stability
    contract (`protocolVersion 2024-11-05`, `schema_version 1.11` MINOR/MAJOR, the
    `SCHEMA-DERIVED` no-new-truth invariant).
  - `book/src/api-tools.md` — the **9 tools** (`generate`, `introspect`,
    `dump_config`, `analyze`, `coverage_gaps` [pure]; `validate`, `minimize`,
    `hunt`, `divergence` [controlled]), each with a parameter table
    (name/type/required/default/constraints), the result shape, errors, and a
    request → response example.
  - `book/src/api-resources-prompts.md` — the `anvil://…` resources
    (`catalog/knobs`, `catalog/lanes`, `audit/log`,
    `artifact/<run_id>/{sv,introspection,manifest,analysis/<query>}`) and the
    **5 workflow prompts**.
  - `book/src/api-introspection.md` — the `--introspect` document envelope, the
    four `analyze` query result schemas (`output_support` / `input_reach` /
    `flop_reset_provenance` / `module_reachability`), and the `schema_version`
    stability contract.

The reference is **accurate to the code**: the tool schemas are exactly what
`tools/list` returns (source of truth `src/mcp/mod.rs`), and the document shapes
summarise the field-by-field wire contract `docs/AGENT_INTROSPECTION_SCHEMA.md`.
The API is **default-off / DUT byte-identical** and adds no new computed truth —
it realises the API-first mandate [[api-first-everything-mcp-accessible]]
(decision `0017`).

See also [[bug-hunt-cli]] (the `hunt` loop), [[acceptance-divergence]] (the
`divergence` tool), and [[semantic-introspection-analyze-tool]] (the `analyze`
queries).
