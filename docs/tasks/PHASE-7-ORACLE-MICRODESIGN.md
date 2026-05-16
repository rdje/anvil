# PHASE-7-ORACLE-MICRODESIGN: Oracle-backed micro-design artifacts

## Metadata

- Tree ID: `PHASE-7-ORACLE-MICRODESIGN`
- Status: `active`
- Roadmap lane: Phase 7 — Oracle-backed micro-design artifacts
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Add a new artifact family: small, self-contained `.sv` files with a
**known expected-facts manifest** (e.g. `rtl_const_expr`-style) — param
/ localparam dependency chains, expression-derived widths and ranges,
generate conditions and loop bounds driven by expressions,
package-qualified constants, precedence-sensitive expressions — each
with a machine-checkable expected-facts contract and downstream parity
checks.

## Non-Goals

- Broad cone complexity / DUT RTL stress — that is the existing Phase
  1–4 lane; Phase 7 is the opposite (tiny, oracle-backed).
- A bundled reference simulator — facts are obviously-checkable
  elaboration facts, not full RTL semantics (project non-goal).
- The artifact-family selector that unifies lanes — that is Phase 9.

## Acceptance Criteria

- Reproducible micro-design corpus generator (seeded, byte-stable).
- Explicit expected-facts manifest per emitted file.
- Parity checks: downstream consumers either agree with the manifest or
  a counterexample is retained.

## Task Tree

- ID: `PHASE-7-ORACLE-MICRODESIGN`
  Status: `active`
  Goal: `Reproducible oracle-backed micro-design corpus with expected-facts contract and downstream parity checks.`
  Children: `PHASE-7-ORACLE-MICRODESIGN.1`, `PHASE-7-ORACLE-MICRODESIGN.2`

- ID: `PHASE-7-ORACLE-MICRODESIGN.1`
  Status: `pending`
  Goal: `Design the micro-design artifact family in DEVELOPMENT_NOTES.md / book: expected-facts schema, generation strategy (param/expr chains), reproducibility contract, parity-check harness shape, relationship to the existing DUT lane, rejected alternatives. Design-only.`
  Acceptance: `Design entry with expected-facts schema sketch and >=1 rejected alternative; mdbook clean; no code change.`
  Verification: `pending`
  Commit: `pending`

- ID: `PHASE-7-ORACLE-MICRODESIGN.2`
  Status: `pending`
  Goal: `Implement the micro-design generator + manifest + parity harness per .1, behind an explicit artifact-family selector flag, with a matrix/parity gate.`
  Acceptance: `Reproducible corpus + manifests; parity harness green or retains counterexamples; ROADMAP Phase 7 -> done.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-7-ORACLE-MICRODESIGN.1` | `pending` | New artifact family needs a designed expected-facts contract before any code; independent of Phase 4/5/6. |

## Decisions

- `2026-05-16`: Phase 7 introduces a *second* artifact lane; it must not
  overload the existing DUT generator path (the doctrinal lane
  separation is preserved here and unified later in Phase 9).

## Open Questions

- Manifest format (JSON schema vs sidecar comments). Owner: `.1` design.
- Whether the parity harness reuses `tool_matrix` or is a new harness.
  Owner: `.1` design.

## Blockers

- None for `.1`. `.2` benefits from but is not hard-blocked by Phase 5
  parameterization; `.1` will record whether `.2` should wait.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `PHASE-7-ORACLE-MICRODESIGN.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-7-ORACLE-MICRODESIGN.1` | `pending` | `pending` |

## Changelog

- `2026-05-16`: Created task tree as part of the directive to task-tree
  every remaining roadmap phase.
