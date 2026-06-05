# SEQUENTIAL-COINDUCTIVE-IDENTITY: Broader Sequential Identity

## Metadata

- Tree ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY`
- Status: `active`
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
  Status: `active`
  Goal: `Broaden reset-defined sequential identity.`
  Children: `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`, `SEQUENTIAL-COINDUCTIVE-IDENTITY.2`, `SEQUENTIAL-COINDUCTIVE-IDENTITY.3`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`
  Status: `pending`
  Goal: `Inventory reset/domain proof preconditions and split the first implementable sequential merge candidate.`
  Acceptance: `The task tree records which candidates are sound now, which need new IR facts, and the next executable implementation leaf; no source behavior changes in this design leaf.`
  Verification: `pending`
  Commit: `pending`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.2`
  Status: `pending`
  Goal: `Implement the first proven broader sequential identity class.`
  Acceptance: `A reset/domain-safe merge class beyond existing exact flop/FSM signatures lands with focused no-merge and merge tests.`
  Verification: `pending`
  Commit: `pending`

- ID: `SEQUENTIAL-COINDUCTIVE-IDENTITY.3`
  Status: `pending`
  Goal: `Close the sequential identity frontier.`
  Acceptance: `The tree records all landed sequential expansions and explicit blockers for any remaining coinductive classes.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` | `pending` | Sequential equivalence is easy to make unsound; the proof envelope must be fixed before implementation. |

## Decisions

- `2026-06-05`: Treat reset kind, reset value, clock domain, and
  canonical endpoint proof as minimum proof inputs for any new
  sequential merge.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEQUENTIAL-COINDUCTIVE-IDENTITY.1` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `SEQUENTIAL-COINDUCTIVE-IDENTITY.1`.
