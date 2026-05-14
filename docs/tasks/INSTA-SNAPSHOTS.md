# INSTA-SNAPSHOTS: Enforce byte-identical reproducibility with `insta` snapshot tests

## Metadata

- Tree ID: `INSTA-SNAPSHOTS`
- Status: `active`
- Roadmap lane: Quality — reproducibility regressions
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Add `insta`-backed snapshot tests of generator output for a small set of
canonical `(seed, config)` tuples covering ANVIL's reachable generation
modes (leaf, depth-1 wrapper, recursive lane, library/on-demand child
sourcing, helper-instance routes, registered/parent-composed routes).
Every accidental output drift — HashMap iteration order, RNG re-seeding,
planner reorder, emit ordering changes — must break a snapshot and
require explicit `cargo insta accept`.

This is the cheapest direct enforcement of the "byte-identical forever"
reproducibility contract stated in `README.md` and `book/src/knobs.md`.

## Non-Goals

- Validate generator correctness. Snapshots only catch drift, not new
  bugs in the generated SystemVerilog.
- Snapshot every possible scenario. Use a curated representative set;
  the matrix gate handles broad coverage separately.
- Replace any existing test. Snapshots are additive guard-rails.

## Acceptance Criteria

- `insta` is wired into the dev-dependency set if not already a direct
  dependency, with explicit version pinning.
- A new test module (e.g., `tests/snapshots.rs`) emits canonical
  `(seed, config)` SV output through `insta::assert_snapshot!` for at
  least: one leaf module, one depth-1 wrapper design, one bounded
  recursive design (library mode), one bounded recursive design
  (on-demand mode), one design exercising helper-instance routes.
- All snapshot tests pass on the current `main`.
- `cargo insta test` is added to the canonical pre-commit checklist
  via `COMMIT.md`.
- The mdBook (`book/src/knobs.md` or a new short page) describes the
  snapshot contract: changing a snapshot is a deliberate act, not an
  accident; `cargo insta accept` requires a paired diff review.

## Task Tree

- ID: `INSTA-SNAPSHOTS`
  Status: `active`
  Goal: `Add insta snapshots covering ANVIL's reachable generation modes; enforce as pre-commit check.`
  Children: `INSTA-SNAPSHOTS.1`, `INSTA-SNAPSHOTS.2`, `INSTA-SNAPSHOTS.3`

- ID: `INSTA-SNAPSHOTS.1`
  Status: `pending`
  Goal: `Wire insta into Cargo.toml dev-dependencies (explicit pin), create tests/snapshots.rs with one canonical leaf snapshot, and one bounded recursive snapshot. Commit lands the baseline; no drift detection yet.`
  Acceptance: `cargo test --test snapshots passes on main; cargo insta test reports two clean snapshots; no other test regresses.`
  Verification: `pending`
  Commit: `pending`

- ID: `INSTA-SNAPSHOTS.2`
  Status: `pending`
  Goal: `Expand snapshots to cover library/on-demand child sourcing, helper-instance routes, registered/parent-composed routes, and at least one design that exercises canonical_module_signatures (so dedup follow-up work in HIERARCHY-AWARE-IDENTITY can detect snapshot drift caused by dedup).`
  Acceptance: `Snapshots cover ≥5 distinct (seed, config) shapes spanning the listed axes; cargo insta test green.`
  Verification: `pending`
  Commit: `pending`

- ID: `INSTA-SNAPSHOTS.3`
  Status: `pending`
  Goal: `Add cargo insta test to COMMIT.md's pre-commit checklist and document the snapshot-acceptance protocol (changing a snapshot is a deliberate act) in book/src/knobs.md or a new chapter.`
  Acceptance: `COMMIT.md updated; book/src/* describes the protocol; mdbook build clean.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `INSTA-SNAPSHOTS.1` | `pending` | Wire-up first; nothing else is verifiable until baseline snapshots exist. |

## Decisions

- `2026-05-14`: Adopting `insta` instead of a hand-rolled byte-equality test. `insta` is already in the dependency tree (visible in compile logs from the matrix gate runs); it has a mature acceptance/diff workflow (`cargo insta accept`, `cargo insta review`) that hand-rolled byte comparison cannot match.

## Open Questions

- Should snapshots live under `tests/snapshots/` (one file per shape) or under `tests/snapshots.rs` (one file with multiple `assert_snapshot!` calls)? `insta`'s default uses per-test files; the matrix output convention suggests directory-based. Owner: implementation choice in `INSTA-SNAPSHOTS.1`. Does not block other leaves.
- Should the snapshot suite also pin canonical `cargo run --bin tool_matrix` JSON-report fragments (e.g., a stable subset of `tool_matrix_report.json`), or strictly the generator's SV output? Owner: `INSTA-SNAPSHOTS.2`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `pending` | `INSTA-SNAPSHOTS.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INSTA-SNAPSHOTS.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative (alongside `DIFFERENTIAL-SIMULATION` and `COVERAGE-INSTRUMENTATION`).
