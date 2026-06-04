# Decision Records (Layer C of `MEMORY_ARCHITECTURE.md`)

Durable, cross-cutting facts and decisions that must survive across
sessions, AI models, and harnesses. Use one record per file, dated, with
`Context -> Decision -> Consequences`. Append, dedupe, and supersede; do
not silently rewrite history.

This is memory layer C. Facts that outlive one task-tree but should not
live in the bounded resume pointer (`MEMORY.md`) belong here. Work-state
memory stays in task trees under `docs/tasks/`; history of what changed
lives in git.

| # | Title | Date | Status | Tags |
| --- | --- | --- | --- | --- |
| [0001](0001-task-tree-and-commit-doctrine.md) | Task-tree ownership before work; strict commit workflow | 2026-06-04 | accepted | process, doctrine |
| [0002](0002-live-doc-path-portability.md) | Live docs and book use repo-root-relative project paths | 2026-06-04 | accepted | docs, portability |
| [0003](0003-resource-safe-validation.md) | Full-suite validation is resource-monitored and not mandatory for workflow-doc leaves | 2026-06-04 | accepted | validation, environment |

## How To Add A Record

1. Copy the shape of an existing record.
2. Use the next sequential number.
3. Add a row to this index.
4. Link the record from related task-tree files when relevant.
5. To change a fact, add a new record or mark the old one superseded; do
   not silently rewrite the old decision.
