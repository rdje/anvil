# Knowledge Map

> **AUTO-GENERATED — DO NOT EDIT.** Regenerate with `knowledge-map/scripts/gen_knowledge_map.sh`.
> Source of truth = YAML front-matter in: `docs/knowledge docs/decisions`. Edit the fact files, never this map.
> A fact is any `.md` whose front-matter has a non-empty `answers:` list.
> **9** facts · **36** question keys.

## Questions → fact

- "are local checkout paths allowed in the book" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "are memory blocks state by instance" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "can ANVIL fold a gate to an input under egraph" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "can ANVIL merge duplicate FSM blocks" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "can hierarchy_module_dedup merge structurally different modules" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "can same-shape cones over different inputs merge" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "can semantic gate merge target non-gate nodes" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "do I need a task tree before changing code" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "does a and b or not b simplify to a" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "does full factorization include FSM state" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "does full factorization merge memories" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "does hierarchy module dedup prove semantic equivalence" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "does hierarchy module dedup remove unreachable modules" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "does hierarchy_module_dedup change under-instantiation" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "does semantic gate merge ignore endpoint identity" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "how should project file paths be written in live docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "repo-root-relative paths in ANVIL docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should ANVIL docs use absolute paths" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should I run the full cargo test suite for Knowledge Map docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "should I run the full cargo test suite for memory architecture docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what RAM threshold stops a full suite" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what does endpoint-preserving identity mean" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05
- "what does fsms_merged measure" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "what happens after module dedup rewrites instances" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "what happens to helper endpoints that cancel out" -> [combinational-semantic-endpoint-fold](docs/knowledge/combinational-semantic-endpoint-fold.md) · 2026-06-05
- "what is ANVIL's task-tree doctrine" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "what is the commit workflow" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "what is the module dedup proof boundary" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "when are unused module definitions pruned" -> [hierarchy-dedup-prune](docs/knowledge/hierarchy-dedup-prune.md) · 2026-06-05
- "when is focused workflow validation enough" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "when should git_message_brief.txt be cleared" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "why can FSMs merge but memories stay opaque" -> [fsm-identity-merge](docs/knowledge/fsm-identity-merge.md) · 2026-06-05
- "why can FSMs merge but memories stay separate" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "why do semantically equal modules stay separate" -> [hierarchy-identity-boundary](docs/knowledge/hierarchy-identity-boundary.md) · 2026-06-05
- "why doesn't ANVIL merge duplicate memories" -> [memory-identity-boundary](docs/knowledge/memory-identity-boundary.md) · 2026-06-05
- "why don't identical truth-table shapes always share NodeIds" -> [endpoint-identity-boundary](docs/knowledge/endpoint-identity-boundary.md) · 2026-06-05

## Facts (by id)

### combinational-semantic-endpoint-fold
_Bounded semantic gate proofs can fold to existing endpoints_

- **answers:** can ANVIL fold a gate to an input under egraph | does a and b or not b simplify to a | what happens to helper endpoints that cancel out | can semantic gate merge target non-gate nodes
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/combinational-semantic-endpoint-fold.md`](docs/knowledge/combinational-semantic-endpoint-fold.md)

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

### live-doc-path-portability
_Live docs and book use repo-root-relative project paths_

- **answers:** should ANVIL docs use absolute paths | how should project file paths be written in live docs | repo-root-relative paths in ANVIL docs | are local checkout paths allowed in the book
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0002-live-doc-path-portability.md; DEVELOPMENT_NOTES.md; CHANGES.md`
- **source:** [`docs/decisions/0002-live-doc-path-portability.md`](docs/decisions/0002-live-doc-path-portability.md)

### memory-identity-boundary
_Inferrable memories stay instance-local under full factorization_

- **answers:** why doesn't ANVIL merge duplicate memories | does full factorization merge memories | are memory blocks state by instance | why can FSMs merge but memories stay separate
- **date:** 2026-06-05 · **status:** current
- **evidence:** `src/ir/compact.rs; book/src/factorization.md; DEVELOPMENT_NOTES.md`
- **source:** [`docs/knowledge/memory-identity-boundary.md`](docs/knowledge/memory-identity-boundary.md)

### resource-safe-validation
_Full-suite validation is resource-monitored and not mandatory for workflow-doc memory and retrieval leaves_

- **answers:** should I run the full cargo test suite for memory architecture docs | should I run the full cargo test suite for Knowledge Map docs | what RAM threshold stops a full suite | when is focused workflow validation enough
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0003-resource-safe-validation.md; COMMIT.md; MEMORY.md`
- **source:** [`docs/decisions/0003-resource-safe-validation.md`](docs/decisions/0003-resource-safe-validation.md)

### task-tree-and-commit-doctrine
_Task-tree ownership before work and strict commit workflow_

- **answers:** what is ANVIL's task-tree doctrine | do I need a task tree before changing code | what is the commit workflow | when should git_message_brief.txt be cleared
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0001-task-tree-and-commit-doctrine.md; docs/TASK_TREE.md; COMMIT.md; MEMORY_ARCHITECTURE.md`
- **source:** [`docs/decisions/0001-task-tree-and-commit-doctrine.md`](docs/decisions/0001-task-tree-and-commit-doctrine.md)
