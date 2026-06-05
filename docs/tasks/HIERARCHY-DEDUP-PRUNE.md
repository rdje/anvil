# HIERARCHY-DEDUP-PRUNE: Prune Unreachable Modules After Dedup

## Metadata

- Tree ID: `HIERARCHY-DEDUP-PRUNE`
- Status: `done`
- Roadmap lane: `NodeId as identity / hierarchical module identity`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Strengthen the opt-in hierarchy module-dedup layer by removing module
definitions that were reachable before dedup but become unreachable from
the design top after a real dedup merge, while preserving dedup-off and
pre-existing under-instantiation behavior.

## Non-Goals

- No new hierarchy identity knob.
- No cross-design module sharing.
- No functional module equivalence beyond the existing canonical
  structural signature.
- No change to dedup-off `num_child_instances < num_leaf_modules`
  under-instantiation behavior.

## Acceptance Criteria

- Source edits are owned by this leaf before they occur.
- `dedup_modules` still preserves the top and rewrites instance module
  references exactly as before.
- After at least one dedup merge, modules made unreachable by the merge
  are pruned deterministically.
- If no dedup merge occurs, existing unused-library definitions are not
  pruned solely because the dedup function was called.
- Focused unit tests prove prune-after-merge and no-merge preservation.
- Live docs, mdBook, and Knowledge Map describe the behavior change.

## Task Tree

- ID: `HIERARCHY-DEDUP-PRUNE`
  Status: `done`
  Goal: `Prune unreachable modules after opt-in hierarchy dedup merges.`
  Children: `HIERARCHY-DEDUP-PRUNE.1`

- ID: `HIERARCHY-DEDUP-PRUNE.1`
  Status: `done`
  Goal: `Implement and document post-dedup unreachable-module pruning.`
  Acceptance: `dedup_modules prunes modules that were reachable before dedup and become top-unreachable after at least one merge; preserves no-merge and pre-existing under-instantiation; focused tests and docs pass.`
  Verification: `cargo test -q ir::dedup::tests::dedup_`; `cargo test -q aggregate_projected_twin_dedup_collapses`; `cargo test -q parameter_aware_identity_collapses_templates_differing_only_in_design_width`; `cargo test -q module_dedup_pass_collapses_structurally_duplicate_modules`; `cargo test -q --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `mdbook build book`; full `cargo test` with RAM monitor peak 56.1%; `cargo fmt --all --check`; `mdbook test book`; memory-architecture + Knowledge Map checks; `git diff --check`.
  Commit: `HIERARCHY-DEDUP-PRUNE.1 - prune post-dedup dead modules`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `HIERARCHY-DEDUP-PRUNE.1` | `done` | Completed and closed the tree. |

## Decisions

- `2026-06-05`: Prune only modules made unreachable by an actual dedup
  merge. This keeps the historical under-instantiated-library surface
  unchanged when the structural dedup pass finds no duplicate module
  signatures, and it also preserves modules that were already
  intentionally unreferenced before dedup.

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `HIERARCHY-DEDUP-PRUNE.1` | `cargo test -q ir::dedup::tests::dedup_`; `cargo test -q aggregate_projected_twin_dedup_collapses`; `cargo test -q parameter_aware_identity_collapses_templates_differing_only_in_design_width`; `cargo test -q module_dedup_pass_collapses_structurally_duplicate_modules`; `cargo test -q --bin tool_matrix phase4_hierarchy_matrix_covers_wrapper_and_recursive_profiles`; `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings`; `mdbook build book`; full `cargo test` with RAM monitoring; `cargo fmt --all --check`; `mdbook test book`; `scripts/check_memory_architecture.sh`; `knowledge-map/scripts/check_knowledge_map.sh`; `git diff --check` | passed; full-suite RAM peak observed 56.1%, safely below the 90% stop threshold |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `HIERARCHY-DEDUP-PRUNE.1` | `HIERARCHY-DEDUP-PRUNE.1 - prune post-dedup dead modules` | `pending hash` |

## Changelog

- `2026-06-05`: Created task tree and opened `HIERARCHY-DEDUP-PRUNE.1`.
- `2026-06-05`: Implemented and verified post-dedup unreachable-module
  pruning; closed `HIERARCHY-DEDUP-PRUNE.1` and the tree.
