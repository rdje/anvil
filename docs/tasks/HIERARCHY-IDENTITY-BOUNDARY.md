# HIERARCHY-IDENTITY-BOUNDARY: Keep Module Dedup Structural

## Metadata

- Tree ID: `HIERARCHY-IDENTITY-BOUNDARY`
- Status: `done`
- Roadmap lane: `NodeId as identity / hierarchical module identity`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Make the current hierarchy-identity proof boundary mechanically
explicit: `hierarchy_module_dedup` merges only canonical structural
module signatures, not arbitrary semantic equivalence between modules.

## Non-Goals

- No semantic module-equivalence engine.
- No change to `canonical_module_signature`.
- No new hierarchy identity knob.
- No change to emitted hierarchy behavior.

## Acceptance Criteria

- Source edits are owned by this leaf before they occur.
- A focused regression proves that two semantically equivalent but
  structurally distinct module definitions remain distinct under
  `dedup_modules`.
- Live docs, mdBook, and Knowledge Map explain that deeper hierarchical
  equivalence remains future work, while the existing structural dedup
  pass is intentional and conservative.

## Task Tree

- ID: `HIERARCHY-IDENTITY-BOUNDARY`
  Status: `done`
  Goal: `Protect the structural-only hierarchy module-dedup boundary.`
  Children: `HIERARCHY-IDENTITY-BOUNDARY.1`

- ID: `HIERARCHY-IDENTITY-BOUNDARY.1`
  Status: `done`
  Goal: `Add a regression and documentation for the structural-only module-dedup boundary.`
  Acceptance: `Semantically equivalent but structurally different module definitions do not merge; docs describe the boundary.`
  Verification: `cargo test -q dedup_keeps_semantic_equivalent_structurally_distinct_modules_separate`; `cargo test -q ir::dedup::tests::dedup_`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; memory-architecture + Knowledge Map checks; `git diff --check`.
  Commit: `HIERARCHY-IDENTITY-BOUNDARY.1 - keep module dedup structural`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `HIERARCHY-IDENTITY-BOUNDARY.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Treat hierarchy identity as structural-only until a
  future task introduces a real module-level semantic-equivalence proof.
  The current signature deliberately hashes module structure, not a
  theorem about whole-module function equivalence.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `HIERARCHY-IDENTITY-BOUNDARY.1` | `cargo test -q dedup_keeps_semantic_equivalent_structurally_distinct_modules_separate`; `cargo test -q ir::dedup::tests::dedup_`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; full suite not run for this narrow regression/docs slice |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `HIERARCHY-IDENTITY-BOUNDARY.1` | `HIERARCHY-IDENTITY-BOUNDARY.1 - keep module dedup structural` | `pending hash` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `HIERARCHY-IDENTITY-BOUNDARY.1`.
- `2026-06-05`: Implemented and verified the structural-only
  hierarchy module-dedup boundary regression; closed
  `HIERARCHY-IDENTITY-BOUNDARY.1` and the tree.
