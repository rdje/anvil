# SEQUENTIAL-IDENTITY: Stronger Sequential Identity

## Metadata

- Tree ID: `SEQUENTIAL-IDENTITY`
- Status: `done`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Advance the roadmap's `NodeId`/full-factorization objective by extending
sound post-construction identity sharing to additional deterministic
sequential blocks, without weakening the existing by-construction RTL
contract or merging state whose semantics ANVIL cannot prove equal.

## Non-Goals

- No general sequential-equivalence engine.
- No memory-state merge in this tree's first leaf; generated memories
  are not reset-defined in the same way as FSMs.
- No retirement of `identity_mode = relaxed`; it remains the semantic
  off-switch for identity/factorization.

## Acceptance Criteria

- Source edits are owned by a leaf before they occur.
- New merges are gated by `identity_mode = node-id` and the effective
  factorization level.
- Duplicate deterministic FSM blocks with the same reset-defined state
  semantics and same selector proof collapse to one block.
- FSM virtual dependencies and `FsmOut` references are remapped safely.
- Focused unit tests prove merge, relaxed/no-factorization no-op, and
  distinct-selector no-merge behavior.
- Live docs and mdBook describe the new identity capability if
  user-facing behavior changes.

## Task Tree

- ID: `SEQUENTIAL-IDENTITY`
  Status: `done`
  Goal: `Strengthen sequential identity under the existing full-factorization doctrine.`
  Children: `SEQUENTIAL-IDENTITY.1`

- ID: `SEQUENTIAL-IDENTITY.1`
  Status: `done`
  Goal: `Merge equivalent generated FSM blocks under node-id identity.`
  Acceptance: `A post-construction pass deduplicates deterministic FSMs with equal selector proofs, encoding, transitions, outputs, widths, and state counts; dependent FsmOut nodes and FsmVirtual dep atoms are remapped; focused tests pass.`
  Verification: `cargo check --all-targets`; focused FSM/compact/metrics/tool_matrix tests; full `cargo test` with RAM monitor peak 55.9%; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; Knowledge Map + memory-architecture checks; `git diff --check`.
  Commit: `SEQUENTIAL-IDENTITY.1 — merge equivalent FSM blocks`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEQUENTIAL-IDENTITY.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Start with FSMs, not memories. FSMs reset to state 0
  and have deterministic transition/output tables, making exact
  selector-proof-based sharing sound. Memories are intentionally left
  opaque because their stored contents are not reset-defined here.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `SEQUENTIAL-IDENTITY.1` | `cargo check --all-targets`; `cargo test -q ir::compact::tests::merge_equivalent_fsms`; `cargo test -q ir::compact::tests::merge_equivalent`; `cargo test -q metrics_count_flops_by_shape`; `cargo test -q --bin tool_matrix aggregate`; full `cargo test` with RAM monitoring; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `mdbook build book`; `mdbook test book`; `knowledge-map/scripts/check_knowledge_map.sh`; `scripts/check_memory_architecture.sh`; `git diff --check` | passed; full-suite RAM peak observed 55.9%, final observed 45.2%, no 90% danger-threshold approach |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEQUENTIAL-IDENTITY.1` | `SEQUENTIAL-IDENTITY.1 — merge equivalent FSM blocks` | Pending hash; closes tree. |

## Changelog

- `2026-06-05`: Created task tree and opened `SEQUENTIAL-IDENTITY.1`.
- `2026-06-05`: Completed `SEQUENTIAL-IDENTITY.1` and closed the tree.
