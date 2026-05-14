# COVERAGE-INSTRUMENTATION: Measure line/branch coverage of the planner and validators

## Metadata

- Tree ID: `COVERAGE-INSTRUMENTATION`
- Status: `active`
- Roadmap lane: Quality — test-discipline visibility
- Created: `2026-05-14`
- Last updated: `2026-05-14`
- Owner: repo-local workflow

## Goal

Add line/branch coverage measurement (via `cargo-llvm-cov` or
equivalent) so that the matrix-gate-plus-focused-proofs test suite
emits a concrete coverage report. Use the report to (a) expose dead
code in `src/gen/hierarchy.rs`, `src/gen/cone.rs`, `src/ir/compact.rs`,
and `src/ir/validate.rs`, (b) confirm that the curated matrix actually
touches every reachable planner branch, and (c) add focused tests for
any under-covered paths surfaced by the report.

This is hygiene more than quality-improvement on its own, but it
converts "the matrix is comprehensive by design intent" into "the
matrix is comprehensive by measurement."

## Non-Goals

- Achieve 100% line coverage. Some branches are defensive panics that
  are intentionally unreachable from the planner.
- Replace any existing test. Coverage is reporting infrastructure,
  not a new gate.
- Treat coverage numbers as a quality dial. The goal is to surface
  specific under-tested paths and act on them, not to chase a
  percentage.

## Acceptance Criteria

- `cargo-llvm-cov` (or equivalent) is installed via documented setup
  in `book/src/architecture.md` or `DEVELOPMENT_NOTES.md`, with a
  reproducible invocation.
- A baseline coverage report exists, committed as part of
  `DEVELOPMENT_NOTES.md` or a dedicated `docs/coverage-baseline.md`.
- At least one previously-under-covered branch (e.g., a defensive
  guard in `compact.rs` or a rarely-fired generator path) is either
  (a) confirmed dead and removed, or (b) covered by a new focused
  proof.
- The pre-commit checklist (`COMMIT.md`) gains an optional
  "run coverage before merging large planner changes" guidance
  step. Coverage is NOT a blocking gate — too slow per-commit — but
  is encouraged for planner-touching slices.

## Task Tree

- ID: `COVERAGE-INSTRUMENTATION`
  Status: `active`
  Goal: `Add line/branch coverage measurement; convert matrix-comprehensiveness from intent to measurement.`
  Children: `COVERAGE-INSTRUMENTATION.1`, `COVERAGE-INSTRUMENTATION.2`, `COVERAGE-INSTRUMENTATION.3`

- ID: `COVERAGE-INSTRUMENTATION.1`
  Status: `pending`
  Goal: `Install cargo-llvm-cov locally; produce a baseline coverage report against the current test suite (unit + tests/pipeline.rs, excluding the slow Phase 4 hierarchy gate). Commit the baseline report as docs/coverage-baseline.md.`
  Acceptance: `cargo llvm-cov --html runs to completion; docs/coverage-baseline.md exists with line-level numbers per crate file.`
  Verification: `pending`
  Commit: `pending`

- ID: `COVERAGE-INSTRUMENTATION.2`
  Status: `pending`
  Goal: `Triage the baseline: identify the top-5 under-covered source files by uncovered-line count. For each, decide: (a) is the uncovered region dead code? (b) is it a rarely-fired planner path that should have a focused proof? (c) is it a defensive panic that is intentionally unreachable? Record findings in DEVELOPMENT_NOTES.md.`
  Acceptance: `DEVELOPMENT_NOTES.md entry exists with the triage matrix and explicit disposition for each of the top-5 files.`
  Verification: `pending`
  Commit: `pending`

- ID: `COVERAGE-INSTRUMENTATION.3`
  Status: `pending`
  Goal: `Act on the triage: remove dead code where confirmed, add focused proofs where rarely-fired paths are real, leave defensive panics as-is. Update coverage baseline.`
  Acceptance: `Coverage baseline updated; new focused proofs (if any) commit-traceable to specific uncovered branches; cargo test all green.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COVERAGE-INSTRUMENTATION.1` | `pending` | Cannot triage without a baseline report; cannot act without triage. Linear dependency chain. |

## Decisions

- `2026-05-14`: Coverage is reporting infrastructure, not a CI-blocking gate. The matrix gate is the blocking gate; coverage is a periodic discipline check. Otherwise the wall-clock cost forces shortcuts that defeat the point.

## Open Questions

- Does `cargo-llvm-cov` give meaningful per-branch coverage on Rust
  match expressions and `if let` chains, or does it report only
  line-level coverage? Per-branch matters for `src/gen/cone.rs`
  fanin-cone branches. Owner: investigation in `COVERAGE-INSTRUMENTATION.1`.
- Should the baseline include the Phase 4 hierarchy gate's
  contribution? The gate exercises planner paths the unit/pipeline
  tests do not, but it takes ~75 min to run — including it makes
  the baseline a once-per-quarter artifact, not a weekly one.
  Owner: `COVERAGE-INSTRUMENTATION.1`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `pending` | `COVERAGE-INSTRUMENTATION.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-INSTRUMENTATION.1` | `pending` | `pending` |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative.
