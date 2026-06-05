# MEMORY-IDENTITY-BOUNDARY: Keep Memory State Identity Instance-Local

## Metadata

- Tree ID: `MEMORY-IDENTITY-BOUNDARY`
- Status: `done`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Make the roadmap's memory-state identity boundary mechanically explicit:
under `identity_mode = node-id`, independent inferrable-memory blocks
remain state-by-instance unless ANVIL has a reset-defined proof that
their stored contents are identical.

## Non-Goals

- No memory-state merge pass.
- No change to emitted memory templates.
- No reset/init semantics added to memories.
- No CLI or knob change.

## Acceptance Criteria

- Source edits are owned by this leaf before they occur.
- A focused regression proves that two independent memories with the
  same source cones remain two `Memory` blocks and two `MemRead` leaves
  after the full-factorization state-pass boundary.
- The proof is tied to the existing memory doctrine: current memories
  have no reset-defined array contents, so identical address/write cones
  are not sufficient evidence for merging state.
- Live docs, mdBook, and Knowledge Map describe the boundary without
  implying that memory-state equivalence is impossible forever.

## Task Tree

- ID: `MEMORY-IDENTITY-BOUNDARY`
  Status: `done`
  Goal: `Protect the memory-state identity proof boundary.`
  Children: `MEMORY-IDENTITY-BOUNDARY.1`

- ID: `MEMORY-IDENTITY-BOUNDARY.1`
  Status: `done`
  Goal: `Add a full-factorization regression and documentation for instance-local memory identity.`
  Acceptance: `Independent memories with identical source cones remain distinct after the relevant identity/factorization state passes; docs explain the reset-defined proof boundary.`
  Verification: `cargo test -q memory_state_identity_stays_instance_local_under_full_factorization`; `cargo test -q ir::compact::tests::mem`; `cargo test -q ir::compact::tests::merge_equivalent_flops`; `cargo test -q ir::compact::tests::merge_equivalent_fsms`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; memory-architecture + Knowledge Map checks; `git diff --check`.
  Commit: `MEMORY-IDENTITY-BOUNDARY.1 - keep memories instance-local`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `MEMORY-IDENTITY-BOUNDARY.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Treat the next memory-identity slice as a boundary
  proof, not a merge implementation. The current inferrable-memory
  template has no reset-defined array contents, so two blocks with equal
  write/read cones are still independent state unless a future task adds
  stronger initialization or equivalence evidence.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `MEMORY-IDENTITY-BOUNDARY.1` | `cargo test -q memory_state_identity_stays_instance_local_under_full_factorization`; `cargo test -q ir::compact::tests::mem`; `cargo test -q ir::compact::tests::merge_equivalent_flops`; `cargo test -q ir::compact::tests::merge_equivalent_fsms`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --all --check`; `mdbook build book`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; full suite not run for this narrow regression/docs slice |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MEMORY-IDENTITY-BOUNDARY.1` | `MEMORY-IDENTITY-BOUNDARY.1 - keep memories instance-local` | `pending hash` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `MEMORY-IDENTITY-BOUNDARY.1`.
- `2026-06-05`: Implemented and verified the instance-local memory
  identity boundary regression; closed `MEMORY-IDENTITY-BOUNDARY.1`
  and the tree.
