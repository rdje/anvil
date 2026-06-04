# Knowledge Map

> **AUTO-GENERATED — DO NOT EDIT.** Regenerate with `knowledge-map/scripts/gen_knowledge_map.sh`.
> Source of truth = YAML front-matter in: `docs/knowledge docs/decisions`. Edit the fact files, never this map.
> A fact is any `.md` whose front-matter has a non-empty `answers:` list.
> **3** facts · **12** question keys.

## Questions → fact

- "are local checkout paths allowed in the book" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "do I need a task tree before changing code" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "how should project file paths be written in live docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "repo-root-relative paths in ANVIL docs" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should ANVIL docs use absolute paths" -> [live-doc-path-portability](docs/decisions/0002-live-doc-path-portability.md) · 2026-06-04
- "should I run the full cargo test suite for Knowledge Map docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "should I run the full cargo test suite for memory architecture docs" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what RAM threshold stops a full suite" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "what is ANVIL's task-tree doctrine" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "what is the commit workflow" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04
- "when is focused workflow validation enough" -> [resource-safe-validation](docs/decisions/0003-resource-safe-validation.md) · 2026-06-04
- "when should git_message_brief.txt be cleared" -> [task-tree-and-commit-doctrine](docs/decisions/0001-task-tree-and-commit-doctrine.md) · 2026-06-04

## Facts (by id)

### live-doc-path-portability
_Live docs and book use repo-root-relative project paths_

- **answers:** should ANVIL docs use absolute paths | how should project file paths be written in live docs | repo-root-relative paths in ANVIL docs | are local checkout paths allowed in the book
- **date:** 2026-06-04 · **status:** current
- **evidence:** `docs/decisions/0002-live-doc-path-portability.md; DEVELOPMENT_NOTES.md; CHANGES.md`
- **source:** [`docs/decisions/0002-live-doc-path-portability.md`](docs/decisions/0002-live-doc-path-portability.md)

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
