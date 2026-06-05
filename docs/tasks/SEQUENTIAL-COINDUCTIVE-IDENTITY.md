# SEQUENTIAL-COINDUCTIVE-IDENTITY: Broader Sequential Identity

## Metadata

- Tree ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY`
- Status: `done`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next sound sequential identity expansions beyond exact
flop-D and deterministic generated-FSM sharing, without merging state
unless ANVIL can prove reset-defined behavioral equivalence.

## Non-Goals

- No unsound state merge based only on syntactic similarity.
- No merge across clock domains or reset domains unless the proof
  explicitly includes those domains.
- No retirement of `identity_mode = relaxed`.
- No memory-array merge; that belongs to
  `MEMORY-STATE-IDENTITY`.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- The tree either lands a new reset-defined sequential equivalence
  class with focused tests or records a measured proof-boundary blocker
  for each candidate class.
- Existing flop and FSM merge behavior remains covered.
- User-facing docs explain any new merge class and any retained
  no-merge boundary.
- Each completed leaf is committed through `COMMIT.md`.
- The tree closes only when all identified sequential candidates have
  landed or have explicit blocker evidence.

## Task Tree

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY`
  Status: `done`
  Goal: `Broaden reset-defined sequential identity.`
  Children: `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`, `SEQUENTIAL-COINDUCTIVE-IDENTITY.2` (`.2.1`, `.2.2`), `SEQUENTIAL-COINDUCTIVE-IDENTITY.3`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`
  Status: `done`
  Goal: `Inventory reset/domain proof preconditions and split the first implementable sequential merge candidate.`
  Acceptance: `The task tree records which candidates are sound now, which need new IR facts, and the next executable implementation leaf; no source behavior changes in this design leaf.`
  Verification: `task-tree inventory, mdBook drift correction, memory/knowledge-map checks`
  Commit: `50746ef`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2`
  Status: `done`
  Goal: `Implement the prerequisite domain proof input and the first proven broader sequential identity class.`
  Acceptance: `Domain-safe state identity lands before any broader coinductive merge; then a reset/domain-safe merge class beyond existing exact flop/FSM signatures lands with focused no-merge and merge tests.`
  Children: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`, `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2`
  Verification: `container complete; see child leaves`
  Commit: `container; no direct commit`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`
  Status: `done`
  Goal: `Add clock-domain proof input to existing flop identity.`
  Acceptance: `Flop identity signatures include Module::flop_domain; same-domain duplicate-D flops still merge; cross-domain duplicate-D/reset flops do not merge; user-facing docs state the domain boundary.`
  Verification: `cargo test -q merge_equivalent_flops; cargo test -q compact_remaps_explicit_flop_domains; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; memory/knowledge-map checks; git diff --check`
  Commit: `06b89f2`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2`
  Status: `done`
  Goal: `Implement reset-defined self-hold coinductive flop identity.`
  Acceptance: `Same-domain, same-width, same-reset flops whose D is exactly their own Q merge after reset-defined proof; reset/domain/width mismatches and non-self-update cases remain no-merge; mdBook examples explain the retained boundary.`
  Verification: `cargo test -q merge_equivalent_flops; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; memory/knowledge-map checks; git diff --check`
  Commit: `1abc41b`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.3`
  Status: `done`
  Goal: `Close the sequential identity frontier.`
  Acceptance: `The tree records all landed sequential expansions and explicit blockers for any remaining coinductive classes.`
  Verification: `task-tree closeout; ROADMAP/docs/TASK_TREE sync; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check`
  Commit: `pending this commit`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| - | - | - | No remaining sequential frontier. |

## Decisions

- `2026-06-05`: Treat reset kind, reset value, clock domain, and
  canonical endpoint proof as minimum proof inputs for any new
  sequential merge.
- `2026-06-05`: Existing duplicate-D flop sharing is sound only inside
  one clock/reset domain. The generated multi-clock promotion pass runs
  after leaf finalization today, so promotion-added synchronizer flops
  are not re-merged by the current generator flow; nevertheless the IR
  has `Module::flop_domain`, and any post-domain merge helper must key
  on it explicitly.
- `2026-06-05`: Existing generated-FSM sharing remains an exact-table
  merge only. `Fsm` has no per-FSM domain field and is emitted on the
  module's shared clock/reset path, so broader FSM coinduction or
  multi-domain FSM sharing is not implementable until the IR records
  that domain/reset fact per FSM.
- `2026-06-05`: The first broader coinductive candidate is exact
  self-hold flop identity: for same-domain, same-width, same-reset
  flops with `D = own Q`, reset makes the two Q values equal and the
  transition relation preserves equality forever. That is strictly
  narrower than arbitrary sequential equivalence.
- `2026-06-05`: Arbitrary mutually-recursive registers, equivalent
  update functions over different state names, convergence after one or
  more cycles, retimed state, and CDC state sharing are blocked by
  missing bounded transition-relation proofs and/or missing IR domain
  facts.

## Open Questions

- None for the current frontier.

## Blockers

- Broader coinductive classes beyond exact self-hold are intentionally
  blocked until ANVIL has a bounded transition-relation proof instead
  of only per-cone endpoint proofs.
- FSM coinduction beyond exact generated tables remains blocked until
  `Fsm` carries explicit per-FSM domain/reset facts and a transition
  relation proof.
- Memory state remains out of scope for this tree and is owned by
  `MEMORY-STATE-IDENTITY`.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` | `task-tree inventory; mdBook sequential/factorization/structural-rule drift correction; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; mdbook build book; git diff --check` | `passed` |
| `2026-06-05` | `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1` | `cargo test -q merge_equivalent_flops; cargo test -q compact_remaps_explicit_flop_domains; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check` | `passed` |
| `2026-06-05` | `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2` | `cargo test -q merge_equivalent_flops; cargo test -q --test snapshots; cargo check --all-targets; cargo clippy --all-targets -- -D warnings; cargo fmt --all --check; mdbook build book; mdbook test book; cargo test -q --test book_examples; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check` | `passed` |
| `2026-06-05` | `SEQUENTIAL-COINDUCTIVE-IDENTITY.3` | `task-tree closeout; ROADMAP/docs/TASK_TREE sync; scripts/check_memory_architecture.sh; knowledge-map/scripts/check_knowledge_map.sh; git diff --check` | `passed` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` | `50746ef SEQUENTIAL-COINDUCTIVE-IDENTITY.1 - inventory proof envelope` | `Inventory reset/domain proof envelope and split .2.1/.2.2 implementation leaves.` |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1` | `06b89f2 SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1 - key flops by domain` | `Domain-aware flop signature prerequisite.` |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2` | `1abc41b SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2 - merge self-hold flops` | `Exact reset-defined self-hold coinductive merge.` |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY.3` | `pending this commit` | `Close sequential frontier and record remaining blockers.` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`.
- `2026-06-05`: Completed `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`
  inventory. Split implementation into `.2.1` domain-aware flop
  signatures and `.2.2` exact self-hold coinductive identity.
- `2026-06-05`: Opened `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`
  for implementation.
- `2026-06-05`: Completed `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.1`
  by adding clock-domain identity to flop merge signatures and
  remapping explicit `flop_domains` entries during merge/compaction.
- `2026-06-05`: Opened `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2`
  for implementation.
- `2026-06-05`: Completed `SEQUENTIAL-COINDUCTIVE-IDENTITY.2.2`
  by adding the exact reset-defined self-hold merge class and retained
  no-merge regressions for resetless, reset/domain/width mismatch, and
  non-exact feedback cases.
- `2026-06-05`: Closed the tree. Landed domain-aware flop identity and
  exact reset-defined self-hold coinduction; retained broader
  transition-relation, FSM-domain, and memory-state blockers.
