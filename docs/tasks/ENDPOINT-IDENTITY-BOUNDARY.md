# ENDPOINT-IDENTITY-BOUNDARY: Preserve Canonical Leaf Endpoints

## Metadata

- Tree ID: `ENDPOINT-IDENTITY-BOUNDARY`
- Status: `done`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Make the endpoint-preserving part of node-id semantic sharing explicit:
two cones with the same local truth-table shape must not merge when they
depend on different canonical leaf endpoints.

## Non-Goals

- No expansion of the semantic proof budget.
- No new equivalence engine.
- No change to emitted RTL or generator distributions.
- No CLI or knob change.

## Acceptance Criteria

- Source edits are owned by this leaf before they occur.
- A focused regression proves that semantic gate merging keeps
  same-shape cones over different primary-input endpoints distinct.
- Live docs, mdBook, and Knowledge Map describe the endpoint boundary.

## Task Tree

- ID: `ENDPOINT-IDENTITY-BOUNDARY`
  Status: `done`
  Goal: `Protect endpoint preservation in node-id semantic gate merging.`
  Children: `ENDPOINT-IDENTITY-BOUNDARY.1`

- ID: `ENDPOINT-IDENTITY-BOUNDARY.1`
  Status: `done`
  Goal: `Add a regression and docs for same-shape/different-endpoint semantic-gate no-merge.`
  Acceptance: `Two equivalent local truth-table shapes over different endpoints do not merge; docs explain why endpoint preservation is required.`
  Verification: `cargo test -q merge_equivalent_gates_keeps_same_shape_different_endpoints_distinct`; `cargo test -q ir::compact::tests::merge_equivalent_gates`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; memory-architecture + Knowledge Map checks; `git diff --check`.
  Commit: `ENDPOINT-IDENTITY-BOUNDARY.1 - preserve semantic endpoints`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `ENDPOINT-IDENTITY-BOUNDARY.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Protect endpoint preservation before expanding any
  semantic proof budget. A same-shaped local truth table over different
  primary inputs is not the same expression identity in ANVIL's
  doctrine.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `ENDPOINT-IDENTITY-BOUNDARY.1` | `cargo test -q merge_equivalent_gates_keeps_same_shape_different_endpoints_distinct`; `cargo test -q ir::compact::tests::merge_equivalent_gates`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; full suite not run for this narrow regression/docs slice |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `ENDPOINT-IDENTITY-BOUNDARY.1` | `ENDPOINT-IDENTITY-BOUNDARY.1 - preserve semantic endpoints` | `pending hash` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `ENDPOINT-IDENTITY-BOUNDARY.1`.
- `2026-06-05`: Implemented and verified the endpoint-preserving
  semantic gate-merge boundary regression; closed
  `ENDPOINT-IDENTITY-BOUNDARY.1` and the tree.
