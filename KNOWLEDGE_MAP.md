# Knowledge Map

> **AUTO-GENERATED — DO NOT EDIT.** Regenerate with `knowledge-map/scripts/gen_knowledge_map.sh`.
> Source of truth = YAML front-matter in: `docs/knowledge docs/decisions`. Edit the fact files, never this map.
> A fact is any `.md` whose front-matter has a non-empty `answers:` list.
> **18** facts · **79** question keys.

## Questions → fact

- "always_comb process has no sensitivities" -> [iverilog-compile-matrix-axis](docs/knowledge/iverilog-compile-matrix-axis.md) · 2026-06-05
- "are local checkout paths allowed in the book" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "are memory blocks state by instance" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "are the five post-phase follow-up trees still active" -> [post-phase-followup-frontier-closed](docs/knowledge/post-phase-followup-frontier-closed.md) · 2026-06-05
- "can ANVIL check frontend manifests with Verilator JSON" -> [verilator-json-frontend-parity](docs/knowledge/verilator-json-frontend-parity.md) · 2026-06-05
- "can ANVIL fold a gate to an input under egraph" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "can ANVIL generate N-flop CDC synchronizers" -> [n-flop-cdc-synchronizer](docs/knowledge/n-flop-cdc-synchronizer.md) · 2026-06-05
- "can ANVIL merge duplicate FSM blocks" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "can ANVIL merge semantically equivalent modules" -> [bounded-semantic-module-identity](docs/knowledge/bounded-semantic-module-identity.md) · 2026-06-05
- "can an AI agent drive ANVIL to find downstream tool bugs" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "can equivalent flops merge across clock domains" -> [domain-aware-flop-identity](docs/knowledge/domain-aware-flop-identity.md) · 2026-06-05
- "can hierarchy_module_dedup merge structurally different modules" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "can same-shape cones over different inputs merge" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "can self-holding flops merge" -> [reset-defined-self-hold-flop-identity](docs/knowledge/reset-defined-self-hold-flop-identity.md) · 2026-06-05
- "can semantic gate merge target non-gate nodes" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "do I need a task tree before changing code" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "does ANVIL generate async FIFOs or pulse synchronizers" -> [n-flop-cdc-synchronizer](docs/knowledge/n-flop-cdc-synchronizer.md) · 2026-06-05
- "does ANVIL merge resetless self-hold flops" -> [reset-defined-self-hold-flop-identity](docs/knowledge/reset-defined-self-hold-flop-identity.md) · 2026-06-05
- "does ANVIL merge stateful modules by semantic equivalence" -> [bounded-semantic-module-identity](docs/knowledge/bounded-semantic-module-identity.md) · 2026-06-05
- "does Verilator expose frontend top localparams and package constants" -> [verilator-json-frontend-parity](docs/knowledge/verilator-json-frontend-parity.md) · 2026-06-05
- "does a and b or not b simplify to a" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "does flop merge key on Module::flop_domain" -> [domain-aware-flop-identity](docs/knowledge/domain-aware-flop-identity.md) · 2026-06-05
- "does full factorization include FSM state" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "does full factorization merge memories" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "does hierarchy module dedup prove semantic equivalence" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "does hierarchy module dedup remove unreachable modules" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "does hierarchy_module_dedup change under-instantiation" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "does semantic gate merge ignore endpoint identity" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "does tool_matrix support Icarus Verilog compile checks" -> [iverilog-compile-matrix-axis](docs/knowledge/iverilog-compile-matrix-axis.md) · 2026-06-05
- "how can ANVIL prove a 3-flop CDC synchronizer was generated" -> [n-flop-cdc-synchronizer](docs/knowledge/n-flop-cdc-synchronizer.md) · 2026-06-05
- "how many endpoint bits can semantic gate merge prove" -> [semantic-proof-budget](docs/knowledge/semantic-proof-budget.md) · 2026-06-05
- "how should project file paths be written in live docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "is SIGNOFF-SURFACE-EXPANSION closed" -> [post-phase-followup-frontier-closed](docs/knowledge/post-phase-followup-frontier-closed.md) · 2026-06-05
- "is the ANVIL MCP server inside the generator core" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "repo-root-relative paths in ANVIL docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should ANVIL docs use absolute paths" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should ANVIL expose an MCP server for AI agents" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "should I run the full cargo test suite for Knowledge Map docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "should I run the full cargo test suite for memory architecture docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what RAM threshold stops a full suite" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what RTL-simulator MCP advice applies to ANVIL" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "what did the reset-all memory probe show" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "what does --iverilog-compile do" -> [iverilog-compile-matrix-axis](docs/knowledge/iverilog-compile-matrix-axis.md) · 2026-06-05
- "what does cdc_synchronizer_stages do" -> [n-flop-cdc-synchronizer](docs/knowledge/n-flop-cdc-synchronizer.md) · 2026-06-05
- "what does endpoint-preserving identity mean" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "what does fsms_merged measure" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "what does hierarchy_semantic_module_dedup do" -> [bounded-semantic-module-identity](docs/knowledge/bounded-semantic-module-identity.md) · 2026-06-05
- "what does parity_against_real_verilator_json_frontend_ast verify" -> [verilator-json-frontend-parity](docs/knowledge/verilator-json-frontend-parity.md) · 2026-06-05
- "what happens after module dedup rewrites instances" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "what happens to flop_domains when flops are merged or compacted" -> [domain-aware-flop-identity](docs/knowledge/domain-aware-flop-identity.md) · 2026-06-05
- "what happens to helper endpoints that cancel out" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "what is ANVIL's agent / MCP interface architecture" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "what is ANVIL's task-tree doctrine" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "what is the ANVIL introspection API" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "what is the commit workflow" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "what is the current post-phase frontier" -> [post-phase-followup-frontier-closed](docs/knowledge/post-phase-followup-frontier-closed.md) · 2026-06-05
- "what is the egraph truth table budget" -> [semantic-proof-budget](docs/knowledge/semantic-proof-budget.md) · 2026-06-05
- "what is the module dedup proof boundary" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "what roadmap work remains after the five follow-up bullets" -> [post-phase-followup-frontier-closed](docs/knowledge/post-phase-followup-frontier-closed.md) · 2026-06-05
- "what sequential coinductive flop class does ANVIL support" -> [reset-defined-self-hold-flop-identity](docs/knowledge/reset-defined-self-hold-flop-identity.md) · 2026-06-05
- "when are unused module definitions pruned" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "when is focused workflow validation enough" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "when should git_message_brief.txt be cleared" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "which follow-up task trees were exhausted on 2026-06-05" -> [post-phase-followup-frontier-closed](docs/knowledge/post-phase-followup-frontier-closed.md) · 2026-06-05
- "which frontend facts does the Verilator JSON gate check" -> [verilator-json-frontend-parity](docs/knowledge/verilator-json-frontend-parity.md) · 2026-06-05
- "why are cross-domain duplicate flops kept distinct" -> [domain-aware-flop-identity](docs/knowledge/domain-aware-flop-identity.md) · 2026-06-05
- "why can FSMs merge but memories stay opaque" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "why can FSMs merge but memories stay separate" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "why did Icarus warn always_comb process has no sensitivities" -> [iverilog-compile-matrix-axis](docs/knowledge/iverilog-compile-matrix-axis.md) · 2026-06-05
- "why do larger semantic cones fall back to structural proof" -> [semantic-proof-budget](docs/knowledge/semantic-proof-budget.md) · 2026-06-05
- "why do semantically equal modules stay separate" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "why do static case muxes lower to assign" -> [iverilog-compile-matrix-axis](docs/knowledge/iverilog-compile-matrix-axis.md) · 2026-06-05
- "why does exact D equals own Q prove flop equality" -> [reset-defined-self-hold-flop-identity](docs/knowledge/reset-defined-self-hold-flop-identity.md) · 2026-06-05
- "why does semantic module dedup require matching port ids" -> [bounded-semantic-module-identity](docs/knowledge/bounded-semantic-module-identity.md) · 2026-06-05
- "why does the semantic proof stop at 12 bits" -> [semantic-proof-budget](docs/knowledge/semantic-proof-budget.md) · 2026-06-05
- "why doesn't ANVIL merge duplicate memories" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "why doesn't ANVIL need a stateful simulator-style session API" -> [agent-introspection-mcp-lane](docs/decisions/0004-agent-introspection-mcp-lane.md) · 2026-06-14
- "why don't identical truth-table shapes always share NodeIds" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "why not reset memories to make them mergeable" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05

## Facts (by id)

### agent-introspection-mcp-lane
_ANVIL exposes agent control + deep introspection as a default-off MCP adapter beside the generator core_

- **answers:** should ANVIL expose an MCP server for AI agents | what is ANVIL's agent / MCP interface architecture | is the ANVIL MCP server inside the generator core | why doesn't ANVIL need a stateful simulator-style session API | can an AI agent drive ANVIL to find downstream tool bugs | what RTL-simulator MCP advice applies to ANVIL | what is the ANVIL introspection API
- **date:** 2026-06-14 · **status:** current
- **evidence:** `docs/decisions/0004-agent-introspection-mcp-lane.md; docs/tasks/AGENT-INTROSPECTION-MCP.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/decisions/0004-agent-introspection-mcp-lane.md`](docs/decisions/0004-agent-introspection-mcp-lane.md)

### bounded-semantic-module-identity
_Bounded pure-combinational module semantic identity can merge_

- **answers:** can ANVIL merge semantically equivalent modules | what does hierarchy_semantic_module_dedup do | why does semantic module dedup require matching port ids | does ANVIL merge stateful modules by semantic equivalence
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/dedup.rs; src/metrics.rs; book/src/hierarchy.md; book/src/knobs.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/bounded-semantic-module-identity.md`](docs/knowledge/bounded-semantic-module-identity.md)

### combinational-semantic-endpoint-fold
_Bounded semantic gate proofs can fold to existing endpoints_

- **answers:** can ANVIL fold a gate to an input under egraph | does a and b or not b simplify to a | what happens to helper endpoints that cancel out | can semantic gate merge target non-gate nodes
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/combinational-semantic-endpoint-fold.md`](docs/knowledge/combinational-semantic-endpoint-fold.md)

### domain-aware-flop-identity
_Flop identity includes the clock/reset domain_

- **answers:** can equivalent flops merge across clock domains | does flop merge key on Module::flop_domain | why are cross-domain duplicate flops kept distinct | what happens to flop_domains when flops are merged or compacted
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; book/src/sequential.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/domain-aware-flop-identity.md`](docs/knowledge/domain-aware-flop-identity.md)

### endpoint-identity-boundary
_Semantic gate merging preserves canonical leaf endpoints_

- **answers:** can same-shape cones over different inputs merge | does semantic gate merge ignore endpoint identity | what does endpoint-preserving identity mean | why don't identical truth-table shapes always share NodeIds
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/endpoint-identity-boundary.md`](docs/knowledge/endpoint-identity-boundary.md)

### fsm-identity-merge
_Deterministic generated FSM blocks can merge under node-id identity_

- **answers:** can ANVIL merge duplicate FSM blocks | why can FSMs merge but memories stay opaque | what does fsms_merged measure | does full factorization include FSM state
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; src/gen/module.rs; src/metrics.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/fsm-identity-merge.md`](docs/knowledge/fsm-identity-merge.md)

### hierarchy-dedup-prune
_Hierarchy module dedup prunes definitions made unreachable by a merge_

- **answers:** does hierarchy module dedup remove unreachable modules | does hierarchy_module_dedup change under-instantiation | when are unused module definitions pruned | what happens after module dedup rewrites instances
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/dedup.rs; book/src/knobs.md; book/src/hierarchy.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/hierarchy-dedup-prune.md`](docs/knowledge/hierarchy-dedup-prune.md)

### hierarchy-identity-boundary
_Hierarchy module dedup is structural, not semantic_

- **answers:** does hierarchy module dedup prove semantic equivalence | can hierarchy_module_dedup merge structurally different modules | why do semantically equal modules stay separate | what is the module dedup proof boundary
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/dedup.rs; book/src/hierarchy.md; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/hierarchy-identity-boundary.md`](docs/knowledge/hierarchy-identity-boundary.md)

### iverilog-compile-matrix-axis
_tool_matrix has an optional Icarus compile axis_

- **answers:** does tool_matrix support Icarus Verilog compile checks | what does --iverilog-compile do | why do static case muxes lower to assign | why did Icarus warn always_comb process has no sensitivities | always_comb process has no sensitivities
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/bin/tool_matrix.rs; src/emit/sv.rs; book/src/synthesizability.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/iverilog-compile-matrix-axis.md`](docs/knowledge/iverilog-compile-matrix-axis.md)

### live-doc-path-portability
_Live docs and book use repo-root-relative project paths_

- **answers:** should ANVIL docs use absolute paths | how should project file paths be written in live docs | repo-root-relative paths in ANVIL docs | are local checkout paths allowed in the book
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0002-live-doc-path-portability.md; DEVELOPMENT_NOTES.md; CHANGES.md`
- **source:** [`docs/decisions/0002-live-doc-path-portability.md`](docs/decisions/0002-live-doc-path-portability.md)

### memory-identity-boundary
_Inferrable memories stay instance-local under full factorization_

- **answers:** why doesn't ANVIL merge duplicate memories | does full factorization merge memories | are memory blocks state by instance | why can FSMs merge but memories stay separate | why not reset memories to make them mergeable | what did the reset-all memory probe show
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/memory-identity-boundary.md`](docs/knowledge/memory-identity-boundary.md)

### n-flop-cdc-synchronizer
_N-flop CDC synchronizer is config-selectable_

- **answers:** can ANVIL generate N-flop CDC synchronizers | what does cdc_synchronizer_stages do | how can ANVIL prove a 3-flop CDC synchronizer was generated | does ANVIL generate async FIFOs or pulse synchronizers
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/gen/multi_clock.rs; src/config.rs; src/metrics.rs; src/bin/tool_matrix.rs; book/src/sequential.md; book/src/knobs.md`
- **source:** [`docs/knowledge/n-flop-cdc-synchronizer.md`](docs/knowledge/n-flop-cdc-synchronizer.md)

### post-phase-followup-frontier-closed
_The five 2026-06-05 post-phase follow-up trees are closed_

- **answers:** are the five post-phase follow-up trees still active | what roadmap work remains after the five follow-up bullets | is SIGNOFF-SURFACE-EXPANSION closed | what is the current post-phase frontier | which follow-up task trees were exhausted on 2026-06-05
- **date:** 2026-06-05 · **status:** current
- **evidence:** `docs/TASK_TREE.md; docs/tasks/SIGNOFF-SURFACE-EXPANSION.md; ROADMAP.md; CODEBASE_ANALYSIS.md`
- **source:** [`docs/knowledge/post-phase-followup-frontier-closed.md`](docs/knowledge/post-phase-followup-frontier-closed.md)

### reset-defined-self-hold-flop-identity
_Exact reset-defined self-hold flops can merge_

- **answers:** can self-holding flops merge | what sequential coinductive flop class does ANVIL support | does ANVIL merge resetless self-hold flops | why does exact D equals own Q prove flop equality
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; book/src/sequential.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/reset-defined-self-hold-flop-identity.md`](docs/knowledge/reset-defined-self-hold-flop-identity.md)

### resource-safe-validation
_Full-suite validation is resource-monitored and not mandatory for workflow-doc memory and retrieval leaves_

- **answers:** should I run the full cargo test suite for memory architecture docs | should I run the full cargo test suite for Knowledge Map docs | what RAM threshold stops a full suite | when is focused workflow validation enough
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0003-resource-safe-validation.md; COMMIT.md; MEMORY.md`
- **source:** [`docs/decisions/0003-resource-safe-validation.md`](docs/decisions/0003-resource-safe-validation.md)

### semantic-proof-budget
_Bounded semantic proofs use support, node, and work budgets_

- **answers:** how many endpoint bits can semantic gate merge prove | why does the semantic proof stop at 12 bits | what is the egraph truth table budget | why do larger semantic cones fall back to structural proof
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/semantic-proof-budget.md`](docs/knowledge/semantic-proof-budget.md)

### task-tree-and-commit-doctrine
_Task-tree ownership before work and strict commit workflow_

- **answers:** what is ANVIL's task-tree doctrine | do I need a task tree before changing code | what is the commit workflow | when should git_message_brief.txt be cleared
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0001-task-tree-and-commit-doctrine.md; docs/TASK_TREE.md; COMMIT.md; MEMORY_ARCHITECTURE.md`
- **source:** [`docs/decisions/0001-task-tree-and-commit-doctrine.md`](docs/decisions/0001-task-tree-and-commit-doctrine.md)

### verilator-json-frontend-parity
_Verilator JSON checks all frontend manifest categories_

- **answers:** can ANVIL check frontend manifests with Verilator JSON | what does parity_against_real_verilator_json_frontend_ast verify | does Verilator expose frontend top localparams and package constants | which frontend facts does the Verilator JSON gate check
- **date:** 2026-06-05 · **status:** current
- **evidence:** `tests/frontend_parity.rs; book/src/ir.md; USER_GUIDE.md; ROADMAP.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/verilator-json-frontend-parity.md`](docs/knowledge/verilator-json-frontend-parity.md)
