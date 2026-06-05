# SIGNOFF-SURFACE-EXPANSION: Broader Signoff Surfaces

## Metadata

- Tree ID: `SIGNOFF-SURFACE-EXPANSION`
- Status: `active`
- Roadmap lane: `Quality / signoff-level downstream confidence`
- Created: `2026-06-05`
- Last updated: `2026-06-05`
- Owner: repo-local workflow

## Goal

Exhaust the next practical signoff-surface expansions: richer CDC
primitive coverage, richer AST/source extractor parity, broader
simulator/tool parity, and larger but resource-aware regression sweeps.

## Non-Goals

- No LLM/VLM or SpecForge-specific capability.
- No tool gate that assumes a commercial/proprietary tool is present.
- No full-suite run without RAM monitoring and the 90% danger-zone
  stop rule.
- No user-facing claim that is not backed by a repo-owned check or a
  clearly marked optional external-tool gate.

## Acceptance Criteria

- Every source edit is owned by a leaf before it occurs.
- At least one richer signoff axis lands with tests and documentation,
  or the current environment/tooling blocker is recorded.
- Existing `tool_matrix`, diff-sim, mdBook example, and snapshot
  contracts remain aligned.
- Any new user-facing gate or CLI option is documented in `USER_GUIDE.md`
  and the mdBook with meaningful examples.
- Each completed leaf is committed through `COMMIT.md`.
- The tree closes only when all listed signoff axes are landed or
  explicitly deferred with evidence.

## Task Tree

- ID: `SIGNOFF-SURFACE-EXPANSION`
  Status: `active`
  Goal: `Broaden downstream and signoff confidence surfaces.`
  Children: `SIGNOFF-SURFACE-EXPANSION.1`, `SIGNOFF-SURFACE-EXPANSION.2`, `SIGNOFF-SURFACE-EXPANSION.3`, `SIGNOFF-SURFACE-EXPANSION.4`

- ID: `SIGNOFF-SURFACE-EXPANSION.1`
  Status: `pending`
  Goal: `Add the next CDC primitive or record the concrete proof/tooling blocker.`
  Acceptance: `A CDC primitive beyond the existing 2-flop synchronizer lands with generation, metrics, matrix coverage, and docs, or a blocker records why the next primitive is not yet safe.`
  Verification: `pending`
  Commit: `pending`

- ID: `SIGNOFF-SURFACE-EXPANSION.2`
  Status: `pending`
  Goal: `Add richer AST/source extractor parity where available.`
  Acceptance: `A slang or Verilator XML extractor path lands as an optional gate with scoped facts, or tool availability/scope blockers are recorded.`
  Verification: `pending`
  Commit: `pending`

- ID: `SIGNOFF-SURFACE-EXPANSION.3`
  Status: `pending`
  Goal: `Broaden simulator/tool parity beyond the current matrix where practical.`
  Acceptance: `A new optional parity axis or larger resource-aware sweep lands, with RAM-monitoring policy observed for any full-suite run.`
  Verification: `pending`
  Commit: `pending`

- ID: `SIGNOFF-SURFACE-EXPANSION.4`
  Status: `pending`
  Goal: `Close the signoff-surface frontier.`
  Acceptance: `The tree records landed axes, optional-gate boundaries, deferred tool blockers, and an empty frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-SURFACE-EXPANSION.1` | `pending` | CDC is already a live opt-in capability, so the next primitive is the nearest signoff-surface expansion. |

## Decisions

- `2026-06-05`: Keep all richer tool integrations optional and
  repo-portable. ANVIL-specific signoff work must not import SpecForge,
  docling, LLM, or VLM assumptions.

## Open Questions

- None for the current frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-05` | `SIGNOFF-SURFACE-EXPANSION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SIGNOFF-SURFACE-EXPANSION.1` | `pending` | `pending` |

## Changelog

- `2026-06-05`: Created task tree and opened
  `SIGNOFF-SURFACE-EXPANSION.1`.
