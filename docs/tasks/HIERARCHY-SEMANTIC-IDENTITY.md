# HIERARCHY-SEMANTIC-IDENTITY: Semantic Module Identity

## Metadata

- Tree ID: `HIERARCHY-SEMANTIC-IDENTITY`
- Status: `active`
- Roadmap lane: `NodeId as identity / hierarchical module identity`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next safe hierarchy/module identity expansions beyond
canonical structural module signatures, starting with bounded,
interface-preserving semantic equivalence where ANVIL can prove the
whole module behavior.

## Non-Goals

- No arbitrary whole-design equivalence engine.
- No merge of modules with different public interfaces.
- No merge through sequential, memory, FSM, or instance state until
  their proof boundaries are explicit.
- No change to the existing structural `hierarchy_module_dedup`
  behavior unless the new semantic layer is explicitly gated.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- Structural module dedup remains covered and unchanged unless a new
  semantic gate is explicitly enabled.
- A bounded semantic module identity class lands with merge/no-merge
  tests, or each candidate is deferred with concrete proof-boundary
  evidence.
- User-facing hierarchy/factorization docs explain the new module
  identity behavior and examples.
- Each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `HIERARCHY-SEMANTIC-IDENTITY`
  Status: `active`
  Goal: `Broaden hierarchy/module identity beyond structural signatures.`
  Children: `HIERARCHY-SEMANTIC-IDENTITY.1`, `HIERARCHY-SEMANTIC-IDENTITY.2`, `HIERARCHY-SEMANTIC-IDENTITY.3`

- ID: `HIERARCHY-SEMANTIC-IDENTITY.1`
  Status: `done`
  Goal: `Implement bounded semantic dedup for pure combinational leaf modules with identical public interfaces.`
  Acceptance: `Semantically equal but structurally different pure combinational leaf modules can merge under an explicit proof boundary; sequential/stateful/module-instance cases remain no-merge; docs explain the boundary.`
  Verification: `cargo test -q semantic_dedup; cargo test -q semantic_module_dedup_flag_collapses_bounded_pure_comb_modules; cargo test -q module_dedup_pass_collapses_structurally_duplicate_modules; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check`
  Commit: `447da5b HIERARCHY-SEMANTIC-IDENTITY.1 - add semantic module dedup`

- ID: `HIERARCHY-SEMANTIC-IDENTITY.2`
  Status: `done`
  Goal: `Evaluate extension beyond pure combinational leaves.`
  Acceptance: `The next safe module-equivalence class is implemented or deferred with proof-boundary evidence.`
  Verification: `cargo test -q semantic_dedup; cargo test -q semantic_module_dedup_flag_collapses_bounded_pure_comb_modules; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check`
  Commit: `pending this commit`

- ID: `HIERARCHY-SEMANTIC-IDENTITY.3`
  Status: `pending`
  Goal: `Close the hierarchy semantic identity frontier.`
  Acceptance: `The tree records all landed semantic module identity behavior and explicit blockers for unsupported module classes.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `HIERARCHY-SEMANTIC-IDENTITY.3` | `pending` | Close the hierarchy semantic identity frontier by recording landed classes and explicit blockers for unsupported classes. |

## Decisions

- `2026-06-05`: Start with pure combinational leaves only. Stateful,
  hierarchical, and memory/FSM modules stay outside semantic module
  dedup until their own proof inputs exist.
- `2026-06-05`: Landed `hierarchy_semantic_module_dedup` as a
  separate default-off semantic pass. It is active only under
  node-id/e-graph and only for non-top pure-combinational,
  instance-free, state-free, concrete modules with matching `(PortId,
  width)` public interfaces and bounded truth-table proof size.
- `2026-06-05`: Extended the semantic proof to bounded
  pure-combinational hierarchy wrappers. A wrapper is admitted only
  when it has at most 8 child instances, every child is also inside the
  proof boundary, every instance binding is concrete, and the merge
  group has no ancestor/descendant relation. Leaf and wrapper proof
  classes stay separate.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `HIERARCHY-SEMANTIC-IDENTITY.1` | `semantic module dedup unit/integration tests; snapshots; cargo check/clippy/fmt; mdBook build/test/book examples; memory/knowledge-map checks; git diff --check` | `passed` |
| `2026-06-05` | `HIERARCHY-SEMANTIC-IDENTITY.2` | `semantic module dedup unit/integration tests; snapshots; cargo check/clippy/fmt; mdBook build/test/book examples; memory/knowledge-map checks; git diff --check` | `passed` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `HIERARCHY-SEMANTIC-IDENTITY.1` | `447da5b HIERARCHY-SEMANTIC-IDENTITY.1 - add semantic module dedup` | `Bounded pure-combinational semantic module dedup plus docs/metrics.` |
| `HIERARCHY-SEMANTIC-IDENTITY.2` | `pending this commit` | `Bounded pure-combinational wrapper semantic module dedup plus cycle guard.` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `HIERARCHY-SEMANTIC-IDENTITY.1`.
- `2026-06-05`: Completed `HIERARCHY-SEMANTIC-IDENTITY.1` by adding
  a default-off bounded semantic module dedup pass for pure
  combinational leaves, design metrics for semantic module signatures,
  and user-facing docs/examples. Frontier advanced to `.2`.
- `2026-06-05`: Completed `HIERARCHY-SEMANTIC-IDENTITY.2` by extending
  the semantic proof to bounded pure-combinational wrappers whose child
  semantics can be recursively proven. Leaf/wrapper proof classes stay
  separate, and ancestor/descendant groups are skipped to avoid
  hierarchy cycles. Frontier advanced to `.3`.
