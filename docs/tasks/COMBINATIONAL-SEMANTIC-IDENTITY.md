# COMBINATIONAL-SEMANTIC-IDENTITY: Broader Combinational Semantic Identity

## Metadata

- Tree ID: `COMBINATIONAL-SEMANTIC-IDENTITY`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next sound, bounded expansions of combinational semantic
identity under `identity_mode = node-id` and `factorization_level =
e-graph`, while preserving the existing canonical-endpoint boundary.

## Non-Goals

- No cross-endpoint merging.
- No unbounded SAT/SMT engine.
- No semantic rewrite that can change emitted RTL under
  `identity_mode = relaxed`.
- No performance regression from proof-budget expansion without a
  focused guard.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- The bounded semantic layer can collapse at least one currently-open
  same-endpoint identity class that the lower ladder does not already
  catch, or the leaf records a real blocker with evidence.
- Endpoint preservation remains covered by regression tests.
- Metrics, live docs, and mdBook explain any new merge/fold behavior.
- Focused checks pass, broader gates run when the blast radius warrants
  them, and each completed leaf is committed through `COMMIT.md`.
- The tree closes only when its frontier is empty because all known
  sound bounded expansions have either landed or been explicitly
  deferred with a proof-boundary reason.

## Task Tree

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY`
  Status: `active`
  Goal: `Broaden same-endpoint combinational semantic identity.`
  Children: `COMBINATIONAL-SEMANTIC-IDENTITY.1`, `COMBINATIONAL-SEMANTIC-IDENTITY.2`, `COMBINATIONAL-SEMANTIC-IDENTITY.3`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.1`
  Status: `pending`
  Goal: `Land the first safe same-endpoint semantic fold beyond gate-to-gate merging.`
  Acceptance: `A gate whose bounded semantic proof equals an existing endpoint or constant is rewired to that existing node at the e-graph rung; endpoint-distinct no-merge tests still pass; docs describe the new fold boundary.`
  Verification: `pending`
  Commit: `pending`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.2`
  Status: `pending`
  Goal: `Audit and extend bounded proof budgets only where focused tests prove runtime stays controlled.`
  Acceptance: `Current hard proof limits are either raised with bounded tests and metrics coverage or kept with an explicit measured blocker.`
  Verification: `pending`
  Commit: `pending`

- ID: `COMBINATIONAL-SEMANTIC-IDENTITY.3`
  Status: `pending`
  Goal: `Close the combinational semantic frontier.`
  Acceptance: `The task file records all landed expansions, any deferred proof limits, validation, and an empty frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COMBINATIONAL-SEMANTIC-IDENTITY.1` | `pending` | It is the narrowest next capability: fold a proven gate to an already-existing same-endpoint value without expanding endpoint cardinality. |

## Decisions

- `2026-06-05`: Start with proven gate-to-existing-node folds before
  increasing support limits. This expands visible identity behavior
  while preserving the endpoint discipline protected by
  `ENDPOINT-IDENTITY-BOUNDARY.1`.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `COMBINATIONAL-SEMANTIC-IDENTITY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COMBINATIONAL-SEMANTIC-IDENTITY.1` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `COMBINATIONAL-SEMANTIC-IDENTITY.1`.
