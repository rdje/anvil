# MEMORY-STATE-IDENTITY: Reset-Defined Memory Identity

## Metadata

- Tree ID: `MEMORY-STATE-IDENTITY`
- Status: `active`
- Roadmap lane: `NodeId as identity / full-factorization mode`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the safe path from the current instance-local memory boundary
to any reset-defined memory-state sharing ANVIL can prove.

## Non-Goals

- No merging of current reset-less inferrable memories.
- No assumption that equal read/write cones imply equal stored array
  contents.
- No generate-then-filter memory legality repair.
- No memory merge that changes emitted RTL under
  `identity_mode = relaxed`.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- Reset-less memories remain instance-local and covered by regression
  tests.
- If reset-defined memory identity is implemented, the reset/init
  semantics are explicit in the IR/emitter and downstream-clean.
- If reset-defined memory identity cannot be implemented safely in the
  current synthesizable subset, the blocker is recorded with evidence.
- User-facing docs explain the memory-state identity boundary and any
  new reset-defined memory behavior.
- Each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `MEMORY-STATE-IDENTITY`
  Status: `active`
  Goal: `Determine and implement safe reset-defined memory-state identity.`
  Children: `MEMORY-STATE-IDENTITY.1`, `MEMORY-STATE-IDENTITY.2`, `MEMORY-STATE-IDENTITY.3`

- ID: `MEMORY-STATE-IDENTITY.1`
  Status: `pending`
  Goal: `Design the reset-defined memory proof boundary.`
  Acceptance: `The task tree and design notes record whether ANVIL can add a synthesizable reset-defined memory template suitable for sharing, plus the next executable implementation leaf or a blocker.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-STATE-IDENTITY.2`
  Status: `pending`
  Goal: `Implement reset-defined memory identity when the proof boundary is available.`
  Acceptance: `A reset-defined memory template and merge/no-merge tests land, or this leaf is split/deferred with a concrete synthesizability blocker.`
  Verification: `pending`
  Commit: `pending`

- ID: `MEMORY-STATE-IDENTITY.3`
  Status: `pending`
  Goal: `Close the memory-state identity frontier.`
  Acceptance: `The tree records landed memory identity behavior or explicit blocker evidence, and the current reset-less boundary remains documented.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `MEMORY-STATE-IDENTITY.1` | `pending` | Memory state sharing is only sound if reset/init semantics are explicit first. |

## Decisions

- `2026-06-05`: Keep the existing reset-less inferrable memory
  template instance-local until a reset-defined template exists and is
  downstream-clean.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `MEMORY-STATE-IDENTITY.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `MEMORY-STATE-IDENTITY.1` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `MEMORY-STATE-IDENTITY.1`.
