# Decision Records (Layer C of `MEMORY_ARCHITECTURE.md`)

Durable, cross-cutting facts and decisions that must survive across
sessions, AI models, and harnesses. Use one record per file, dated, with
`Context -> Decision -> Consequences`. Append, dedupe, and supersede; do
not silently rewrite history.

This is memory layer C. Facts that outlive one task-tree but should not
live in the bounded resume pointer (`MEMORY.md`) belong here. Work-state
memory stays in task trees under `docs/tasks/`; history of what changed
lives in git.

Decision records may also carry Knowledge Map front matter. A non-empty
`answers:` list makes the record discoverable in the generated
`KNOWLEDGE_MAP.md` retrieval index.

| # | Title | Date | Status | Tags |
| --- | --- | --- | --- | --- |
| [0001](0001-task-tree-and-commit-doctrine.md) | Task-tree ownership before work; strict commit workflow | 2026-06-04 | accepted | process, doctrine |
| [0002](0002-live-doc-path-portability.md) | Live docs and book use repo-root-relative project paths | 2026-06-04 | accepted | docs, portability |
| [0003](0003-resource-safe-validation.md) | Full-suite validation is resource-monitored and not mandatory for workflow-doc memory and retrieval leaves | 2026-06-04 | accepted | validation, environment, safety |
| [0004](0004-agent-introspection-mcp-lane.md) | Agent control + deep introspection exposed as a default-off MCP adapter beside the generator core | 2026-06-14 | accepted | mcp, agent, api, architecture, introspection |
| [0005](0005-agent-mcp-expansion-surface.md) | Broaden the read-mostly agent/MCP surface by projecting recorded facts, routing non-DUT lanes, and adding an optional HTTP transport | 2026-06-15 | accepted | mcp, agent, coverage, transport, architecture, introspection |
| [0006](0006-signoff-automation-first-increment.md) | The first SIGNOFF-AUTOMATION-EXPANSION increment promotes unswept generator knobs into explicit matrix axes + coverage facts | 2026-06-15 | accepted | signoff, tool-matrix, coverage, adversarial, sweep, quality |
| [0007](0007-identity-deepening-first-extension.md) | The first IDENTITY-DEEPENING extension is bounded bisimulation-based sequential flop equivalence (default-off, reusing the bounded combinational endpoint proof) | 2026-06-15 | accepted | identity, sequential, factorization, bisimulation, coinduction, flop-merge |
| [0008](0008-identity-deepening-whole-module-sequential-equivalence.md) | The second IDENTITY-DEEPENING extension is bounded whole-leaf-module sequential equivalence via cross-module bisimulation (default-off, beside dedup_semantic_modules) | 2026-06-15 | accepted | identity, sequential, factorization, bisimulation, coinduction, module-dedup, hierarchy |
| [0009](0009-sv-version-targeting.md) | ANVIL gains a --sv-version capability gate that targets a chosen IEEE 1800 standard valid-by-construction (default byte-identical) | 2026-06-15 | accepted | capability, sv-version, emission, downstream, valid-by-construction, north-star, breadth |
| [0010](0010-sv-version-first-upopt-soft-packed-union.md) | ANVIL's first version-distinctive up-opt is a heterogeneous-width packed `union soft` (IEEE 1800-2023 §7.3.1), default-off / byte-identical | 2026-06-16 | accepted | capability, sv-version, up-opt, emission, downstream, soft-union, 2023, valid-by-construction, north-star |
| [0011](0011-semantic-introspection-derived-query-surface.md) | ANVIL gains a first-class, MCP-queryable, SCHEMA-DERIVED derived-relation introspection API; the first query is the transitive support cone of an output | 2026-06-16 | accepted | introspection, mcp, semantic, derived-query, schema-derived, agent, api, north-star |
| [0012](0012-structured-emission-first-surface-combinational-function.md) | ANVIL's first richer-structured SV surface is a default-off, valid-by-construction combinational `function automatic` emit-projection of an existing cone | 2026-06-16 | accepted | capability, structured-emission, function, emission, downstream, valid-by-construction, rules-first, breadth, north-star |
| [0013](0013-structured-emission-second-surface-generate-loop.md) | ANVIL's second richer-structured SV surface is a default-off, valid-by-construction `generate for` loop emit-projection of an existing replicated construction | 2026-06-16 | accepted | capability, structured-emission, generate, genvar, emission, downstream, valid-by-construction, rules-first, breadth, north-star |
| [0014](0014-structured-emission-third-surface-combinational-task.md) | ANVIL's third richer-structured SV surface is a default-off, valid-by-construction combinational `task automatic` emit-projection of an existing combinational gate | 2026-06-16 | accepted | capability, structured-emission, task, emission, downstream, valid-by-construction, rules-first, breadth, north-star |

## How To Add A Record

1. Copy the shape of an existing record.
2. Use the next sequential number.
3. Add a row to this index.
4. Link the record from related task-tree files when relevant.
5. To change a fact, add a new record or mark the old one superseded; do
   not silently rewrite the old decision.
