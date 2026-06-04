# LIVE-DOC-PATH-HYGIENE: Repo-Relative Live-Doc Paths

## Metadata

- Tree ID: `LIVE-DOC-PATH-HYGIENE`
- Status: `done`
- Roadmap lane: `Workflow / live-doc hygiene`
- Created: `2026-06-04`
- Last updated: `2026-06-04`
- Owner: repo-local workflow

## Goal

Remove local machine path drift from the live docs and book so project file
references are repo-root-relative, while preserving external validation artifact
paths where those paths identify banked evidence outside the repository.

## Non-Goals

- No source-code or generated-RTL behavior changes.
- No roadmap phase reclassification.
- No rewrite of `/tmp` evidence paths that intentionally name external banked
  validation artifacts.

## Acceptance Criteria

- Local absolute repo paths under the maintainer's checkout are rewritten to
  repo-root-relative paths in live docs, task trees, and the mdBook.
- Stale closed-tree `active` metadata found during the path audit is corrected
  so the task-tree registry and task files agree.
- Focused drift searches find no remaining maintainer-local checkout prefixes
  in the live-doc/book surface.
- `mdbook build book` passes.
- Standard `COMMIT.md` precommit checks run before commit.
- The completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `LIVE-DOC-PATH-HYGIENE`
  Status: `done`
  Goal: `Keep live-doc and mdBook path references portable across checkouts.`
  Children: `LIVE-DOC-PATH-HYGIENE.1`

- ID: `LIVE-DOC-PATH-HYGIENE.1`
  Status: `done`
  Goal: `Rewrite local absolute repo paths to repo-root-relative references and align stale task-tree status metadata discovered by the audit.`
  Acceptance: `Path drift searches are clean for local checkout paths; stale closed-tree active statuses are corrected; mdBook and COMMIT.md checks pass; docs record the outcome.`
  Verification: `rg local-checkout-path drift search clean; stale active-status search clean except template; git diff --check clean; mdbook build book clean; cargo check --all-targets clean; cargo test clean; cargo clippy --all-targets -- -D warnings clean; cargo fmt --all --check clean.`
  Commit: `Docs: LIVE-DOC-PATH-HYGIENE.1 repo-relative live-doc paths`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LIVE-DOC-PATH-HYGIENE.1` | `done` | Completed: local checkout path drift removed, stale closed-tree statuses aligned, validation clean. |

## Decisions

- `2026-06-04`: Treat repo-local checkout paths as drift to rewrite, but keep
  `/tmp` banked validation paths intact because they are external evidence
  locations, not project-file references.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-04` | `LIVE-DOC-PATH-HYGIENE.1` | `rg local-checkout-path drift search`; `rg 'Status: \`active\`' docs/tasks/*.md`; `git diff --check`; `mdbook build book`; `cargo check --all-targets`; `cargo test`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check` | Done — all checks clean; active-status search reports only `docs/tasks/TEMPLATE.md` plus this tree before closure. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LIVE-DOC-PATH-HYGIENE.1` | `Docs: LIVE-DOC-PATH-HYGIENE.1 repo-relative live-doc paths` | Hash can be backfilled in a later live-doc update per `COMMIT.md`. |

## Changelog

- `2026-06-04`: Created task tree and opened `LIVE-DOC-PATH-HYGIENE.1`.
- `2026-06-04`: Completed `LIVE-DOC-PATH-HYGIENE.1`; tree closed.
