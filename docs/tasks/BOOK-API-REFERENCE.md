# BOOK-API-REFERENCE: a comprehensive, industry-standard API reference in the mdBook

## Metadata

- Tree ID: `BOOK-API-REFERENCE`
- Status: `done`
- Roadmap lane: `Docs / book — comprehensive API reference (owner-directed 2026-06-17)`
- Created: `2026-06-17`
- Last updated: `2026-06-17` (all leaves `.1`..`.4` delivered together as one pure-docs deliverable; root closed — tree `done`)
- Owner: repo-local workflow

> **Delivery note.** This is a **pure-mdBook docs** tree (task-tree-exempt from
> the code-ownership mandate; tracked here for continuity). It was originally
> pre-split `.1`..`.4` (overview / tools / resources-prompts / introspection),
> but the API reference is **one cohesive artifact**, so it landed as a **single
> leaf `.1`** in one commit (a leaf may deliver several files; this keeps the
> 1:1 leaf↔commit mapping the `commit-msg` hook enforces). DUT byte-identical
> (no `src/` touched).

## Goal

Document ANVIL's **agent/automation API** in the mdBook accurately, thoroughly,
and in the **most top-notch, industry-recommended way to describe APIs** (owner
directive, `2026-06-17`). The book is the user-facing surface (`feedback_book_doctrine`)
and the API is designed for agents, not humans (`feedback_api_for_agents_not_humans`),
so the reference must be machine-precise and complete: every method, every
parameter (type / required / default / constraints), every result shape, every
error, with worked request/response examples, plus a versioning/stability
contract.

The API surfaces to cover (all already shipped; this is **docs-only**):

1. **The `anvil-mcp` MCP server** — JSON-RPC 2.0 (`protocolVersion 2024-11-05`)
   over stdio (default) or HTTP (`--http <addr>`, loopback default):
   - lifecycle methods: `initialize`, `ping`, `tools/list`, `tools/call`,
     `resources/list`, `resources/read`, `prompts/list`, `prompts/get`;
   - **9 tools** (`generate`, `introspect`, `dump_config`, `validate`, `minimize`,
     `coverage_gaps`, `analyze`, `hunt`, `divergence`) — each with its input JSON
     Schema, result shape, and errors;
   - **resources** (`anvil://catalog/{knobs,lanes}`, `anvil://audit/log`,
     `anvil://artifact/<run_id>/{sv,introspection,manifest,analysis/<query>}`);
   - **5 prompts** (`find_downstream_bug`, `close_coverage_gap`,
     `minimize_reproducer`, `triage_tool_failures`, `explain_artifact`);
   - the **error model**: protocol errors (`-32700` parse, `-32601` method-not-found,
     `-32602` invalid-params) vs tool-level `isError: true` results.
2. **The `--introspect` document** — the versioned envelope (schema `1.11`),
   wire contract `docs/AGENT_INTROSPECTION_SCHEMA.md`.
3. **The `analyze` derived-relation queries** — `output_support` / `input_reach` /
   `flop_reset_provenance` / `module_reachability`, each result schema.
4. **Versioning & stability** — `protocolVersion`, `schema_version` MINOR/MAJOR
   policy, `run_id` content-addressing, the `SCHEMA-DERIVED` invariant.

## Non-Goals

- **No code change.** This is a pure-mdBook docs effort (task-tree-exempt by
  doctrine, but tracked here for continuity). It must not alter any schema,
  tool, or behaviour — it *documents* what ships.
- **No new ADR.** The governing decisions already exist: `0017` (API-first,
  everything MCP-accessible), `0004` (the MCP lane + sandbox/allow-list
  discipline), `0011` (`SCHEMA-DERIVED` projection). This tree realises their
  documentation.
- **No duplication of `docs/AGENT_INTROSPECTION_SCHEMA.md`** — the book links to
  it as the field-by-field wire contract and summarises the envelope.

## Documentation standard (the "top-notch, industry-recommended way")

For an RPC / MCP / agent API the industry standard is a **formal, complete
reference** alongside the existing narrative tutorial (`agent-mcp.md`):

- **JSON-Schema-faithful per-method documentation** derived verbatim from the
  code (`src/mcp/mod.rs` `tools_list` is the source of truth): for each tool, a
  parameter table (name / type / required / default / constraints / description),
  the result shape, the errors, and **one concrete request → response example**.
- **Protocol conventions up front**: the JSON-RPC envelope, the lifecycle
  methods, transports, and the two-layer error model (protocol vs `isError`).
- **A versioning & stability section**: how `protocolVersion` and
  `schema_version` evolve, what content-addressing guarantees, the
  `SCHEMA-DERIVED` no-new-truth invariant.
- **Cross-linked, progressively disclosed** (`feedback_book_doctrine`): the
  tutorial chapter leads; the reference pages are children for depth on demand.

## Acceptance Criteria

- New mdBook reference pages document **every** method, tool, parameter, result,
  resource, prompt, and error of the shipped API, accurate to the code, with
  worked examples + a versioning/stability section.
- `mdbook build book` clean; `cargo test --test book_examples` 3/3 (any runnable
  bash block carries the `book-test` sentinel as needed; the API reference is
  JSON/illustrative, not new runnable generator commands).
- A KM how-to card pointing at the reference.
- No code touched ⇒ DUT byte-identical; `tests/snapshots.rs` untouched.
- Committed per `COMMIT.md`.

## Task Tree

- ID: `BOOK-API-REFERENCE`
  Status: `done`
  Goal: `Document ANVIL's agent/automation API in the mdBook, industry-standard and complete, docs-only.`
  Result: `Done (2026-06-17) via the single implementation leaf .1. A formal, JSON-Schema-faithful API Reference added to the mdBook Reference part as four child pages under the agent-mcp.md tutorial (progressive disclosure). Pure-docs / DUT byte-identical.`
  Children: `BOOK-API-REFERENCE.1`

- ID: `BOOK-API-REFERENCE.1`
  Status: `done`
  Goal: `Write the complete API Reference as four mdBook child pages under the agent-mcp.md tutorial — api-reference.md (overview + JSON-RPC 2.0 conventions: envelope, the 8 lifecycle methods, stdio/HTTP transports, the error model [-32700/-32601/-32602 protocol errors vs tool-level isError], content-addressing, the protocolVersion 2024-11-05 / schema_version 1.11 MINOR-MAJOR / SCHEMA-DERIVED versioning contract), api-tools.md (the 9 tools, each with an input-schema parameter table + result shape + errors + a request→response example), api-resources-prompts.md (the anvil:// resources + the 5 prompts), api-introspection.md (the --introspect envelope + the four analyze query result schemas + the schema_version stability contract) — plus the SUMMARY.md entries, the agent-mcp.md tutorial→reference cross-link, and a KM how-to card. Schemas derived verbatim from src/mcp/mod.rs (tools_list/resources_list/prompts) + docs/AGENT_INTROSPECTION_SCHEMA.md. (Originally pre-split .1..4; delivered as one leaf since it is one cohesive pure-docs artifact — a leaf may deliver several files.)`
  Acceptance: `Every method/tool/parameter/result/resource/prompt/error documented accurately; mdbook build clean; book_examples 3/3; KM card added; pure-docs / DUT byte-identical.`
  Result: `Done. The four pages written and accurate to the code; SUMMARY child entries added; agent-mcp.md gains a tutorial→reference cross-link callout; docs/knowledge/api-reference.md KM card added (KM 49→50). Found-and-fixed two missing book-test:skip sentinels (a curl block + the illustrative anvil --introspect block) — the harness binds a sentinel to the immediately-following block only.`
  Verification: `mdbook build book clean; cargo test --test book_examples 3/3; bash knowledge-map/scripts/check_knowledge_map.sh OK (49→50 facts); bash scripts/check_memory_architecture.sh OK. No src/ touched ⇒ DUT byte-identical.`
  Commit: `this BOOK-API-REFERENCE.1 commit`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none) | `done` | **Tree closed `2026-06-17`.** `.1` delivered all four pages. Future API surfaces (new tools/lanes) extend the reference as they ship; no frontier remains here. |

## Decisions

- `2026-06-17`: Registered as an owner-directed docs lane. The documentation
  standard (above) is JSON-Schema-faithful per-method reference + protocol
  conventions + versioning, alongside the existing `agent-mcp.md` tutorial
  (progressive disclosure). Source of truth = `src/mcp/mod.rs` (`tools_list`,
  `resources_list`, `prompts`) + `docs/AGENT_INTROSPECTION_SCHEMA.md`. No new ADR
  (governed by decisions `0017` / `0004` / `0011`). Pure-docs / DUT byte-identical.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-17` | `BOOK-API-REFERENCE` | `tree registered (docs-only); no code` | `registered` |
| `2026-06-17` | `BOOK-API-REFERENCE.1` | `4 reference pages written (api-reference/api-tools/api-resources-prompts/api-introspection) + SUMMARY entries + agent-mcp cross-link + KM card; schemas derived from src/mcp/mod.rs + AGENT_INTROSPECTION_SCHEMA.md; mdbook build clean; cargo test --test book_examples 3/3 (found-and-fixed two missing book-test:skip sentinels — a curl block + the illustrative anvil --introspect block); KM gen+check OK (49→50); check_memory_architecture OK; no src/ touched ⇒ DUT byte-identical` | `done` |
| `2026-06-17` | `BOOK-API-REFERENCE` | `.1 done ⇒ ROOT closed; tree done` | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `BOOK-API-REFERENCE.1` | `BOOK-API-REFERENCE.1 — industry-standard API reference in the mdBook` | One pure-docs deliverable (4 new Reference pages: overview/protocol, tools, resources/prompts, introspection/analyze) + SUMMARY entries + agent-mcp tutorial cross-link + the api-reference KM card. Schemas faithful to `src/mcp/mod.rs` + `docs/AGENT_INTROSPECTION_SCHEMA.md`. mdbook clean; book_examples 3/3; KM 49→50; DUT byte-identical. Closes the tree. |

## Changelog

- `2026-06-17`: Tree **done** — the comprehensive API reference landed as one
  pure-docs commit. Four new mdBook Reference pages under the `agent-mcp.md`
  tutorial: `api-reference.md` (JSON-RPC protocol, transports, the
  `-32700`/`-32601`/`-32602`-vs-`isError` error model, content-addressing, the
  `protocolVersion`/`schema_version` versioning contract), `api-tools.md` (the 9
  tools with input-schema tables + result shapes + errors + examples),
  `api-resources-prompts.md` (the `anvil://…` resources + the 5 prompts), and
  `api-introspection.md` (the `--introspect` envelope + the 4 `analyze` query
  schemas + the stability contract). Tutorial cross-links the reference; KM card
  `api-reference` added (49→50). Accurate to the code; mdbook clean; book_examples
  3/3; pure-docs / DUT byte-identical.
- `2026-06-17`: Created task tree (owner directive: thoroughly document ANVIL's
  API in the mdBook, top-notch / industry-standard). Originally pre-split `.1`
  (scaffold + protocol/overview) → `.2` (tools) → `.3` (resources + prompts) →
  `.4` (introspection + analyze + closeout); **collapsed to a single leaf `.1`**
  at delivery since the reference is one cohesive pure-docs artifact (keeps the
  1:1 leaf↔commit mapping). Docs-only / DUT byte-identical.
