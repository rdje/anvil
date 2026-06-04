# 0002 - Live docs and book use repo-root-relative project paths

- Date: 2026-06-04
- Status: accepted
- Tags: docs, portability, continuity

## Context

`LIVE-DOC-PATH-HYGIENE.1` removed maintainer-local checkout paths from
ANVIL's live docs and task-tree surface. Absolute local checkout paths
make docs non-portable across machines, harnesses, and future agents.

## Decision

Use repo-root-relative paths for project files in live docs, task-tree
files, and mdBook-facing documentation. Examples: `README.md`,
`book/src/ir.md`, `src/bin/tool_matrix.rs`, `target/tmp/...`.

External validation evidence is different. Banked artifacts outside the
repository, such as `/tmp/anvil-...`, may remain absolute because they
identify evidence outside the checkout rather than project files inside
it.

## Consequences

- A cloned repo no longer carries one maintainer's local filesystem
  structure in its docs.
- Future docs should avoid `file://`, editor URIs, and private checkout
  prefixes.
- Drift searches can mechanically catch regressions.

## Links

- Task-trees: `LIVE-DOC-PATH-HYGIENE.1`,
  `MEMORY-ARCHITECTURE-DOC.2`
- Docs: `DEVELOPMENT_NOTES.md`, `CHANGES.md`
