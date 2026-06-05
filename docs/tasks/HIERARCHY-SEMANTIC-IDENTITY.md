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
  Status: `pending`
  Goal: `Implement bounded semantic dedup for pure combinational leaf modules with identical public interfaces.`
  Acceptance: `Semantically equal but structurally different pure combinational leaf modules can merge under an explicit proof boundary; sequential/stateful/module-instance cases remain no-merge; docs explain the boundary.`
  Verification: `pending`
  Commit: `pending`

- ID: `HIERARCHY-SEMANTIC-IDENTITY.2`
  Status: `pending`
  Goal: `Evaluate extension beyond pure combinational leaves.`
  Acceptance: `The next safe module-equivalence class is implemented or deferred with proof-boundary evidence.`
  Verification: `pending`
  Commit: `pending`

- ID: `HIERARCHY-SEMANTIC-IDENTITY.3`
  Status: `pending`
  Goal: `Close the hierarchy semantic identity frontier.`
  Acceptance: `The tree records all landed semantic module identity behavior and explicit blockers for unsupported module classes.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `HIERARCHY-SEMANTIC-IDENTITY.1` | `pending` | Pure combinational leaf modules are the smallest whole-module class with a bounded truth-table proof. |

## Decisions

- `2026-06-05`: Start with pure combinational leaves only. Stateful,
  hierarchical, and memory/FSM modules stay outside semantic module
  dedup until their own proof inputs exist.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `HIERARCHY-SEMANTIC-IDENTITY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `HIERARCHY-SEMANTIC-IDENTITY.1` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `HIERARCHY-SEMANTIC-IDENTITY.1`.
