# INSTA-SNAPSHOTS: Enforce byte-identical reproducibility with `insta` snapshot tests

## Metadata

- Tree ID: `INSTA-SNAPSHOTS`
- Status: `active`
- Roadmap lane: Quality — reproducibility regressions
- Created: `2026-05-14`
- Last updated: `2026-05-18` (`.1` landed — insta pinned + baseline snapshots; frontier → `.2`)
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
  Status: `done`
  Goal: `Wire insta into Cargo.toml dev-dependencies (explicit pin), create tests/snapshots.rs with one canonical leaf snapshot, and one bounded recursive snapshot. Commit lands the baseline; no drift detection yet.`
  Acceptance: `cargo test --test snapshots passes on main; cargo insta test reports two clean snapshots; no other test regresses.`
  Verification: `Cargo.toml [dev-dependencies] insta "1" → "=1.47.2" (explicit pin to the version in the local registry cache — the snapshot tooling itself must be deterministic; offline-safe; Cargo.lock UNCHANGED since "1" already resolved to 1.47.2). New tests/snapshots.rs with two fully-deterministic fixed-(seed,Config) snapshots via insta::assert_snapshot!: snapshot_canonical_leaf (seed 1, minimal combinational leaf) and snapshot_bounded_recursive_library (seed 11, exact min==max hierarchy depth 2 + exact min==max 2 child instances, library mode — a proven-shape config from tests/pipeline.rs::generates_valid_recursive_hierarchy_designs_with_bounded_shape). Each emit() asserts cfg.validate() + validate_design() before snapshotting. Baselines generated via INSTA_UPDATE=always then RE-RUN without update → both pass (byte-identical/stable: tests/snapshots/snapshots__canonical_leaf.snap 639 B + snapshots__bounded_recursive_library.snap ~227 KB committed). cargo-insta not installed → INSTA_UPDATE env used (no subcommand needed; .3 wires the pre-commit checklist + acceptance protocol). cargo fmt --all --check / clippy --all-targets -- -D warnings clean; full cargo test green incl. the new tests/snapshots.rs binary, no other test regressed (COMMIT.md gate). Open Question resolved: snapshots live under tests/snapshots/ (insta default, per-test .snap files) driven by one tests/snapshots.rs. No book/ change (.3 documents the protocol).`
  Commit: `Quality: INSTA-SNAPSHOTS.1 insta dev-dep pin + tests/snapshots.rs baseline (leaf + bounded recursive)`

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
| 1 | `INSTA-SNAPSHOTS.2` | `pending` | `.1` **done** — `insta` pinned `=1.47.2`; `tests/snapshots.rs` baseline (canonical leaf + bounded recursive library) generated, stable on re-run, full suite green. `.2` expands to ≥5 shapes spanning library/on-demand child sourcing, helper-instance routes, registered/parent-composed routes, and a `canonical_module_signatures`-exercising design (so dedup drift is detectable). Unblocked; one cargo-test slice. |

## Decisions

- `2026-05-14`: Adopting `insta` instead of a hand-rolled byte-equality test. `insta` is already in the dependency tree (visible in compile logs from the matrix gate runs); it has a mature acceptance/diff workflow (`cargo insta accept`, `cargo insta review`) that hand-rolled byte comparison cannot match.

## Open Questions

- Should snapshots live under `tests/snapshots/` (one file per shape) or under `tests/snapshots.rs` (one file with multiple `assert_snapshot!` calls)? **Resolved by `.1`**: one `tests/snapshots.rs` driving multiple named `insta::assert_snapshot!` calls, with `insta`'s default per-test `.snap` files under `tests/snapshots/` (`snapshots__<name>.snap`). Best of both — single test file, reviewable per-shape `.snap` diffs.
- Should the snapshot suite also pin canonical `cargo run --bin tool_matrix` JSON-report fragments (e.g., a stable subset of `tool_matrix_report.json`), or strictly the generator's SV output? Owner: `INSTA-SNAPSHOTS.2`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `INSTA-SNAPSHOTS.1` | `insta` pinned `=1.47.2` (Cargo.lock unchanged); `tests/snapshots.rs` with `snapshot_canonical_leaf` + `snapshot_bounded_recursive_library` (fixed deterministic configs, validate+validate_design before snapshot). Baselines via `INSTA_UPDATE=always` then **re-run without update → both pass** (byte-stable). `cargo fmt --all --check` / `clippy --all-targets -- -D warnings` clean; full `cargo test` green incl. the new binary, no regression. | Done. Frontier → `.2`. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INSTA-SNAPSHOTS.1` | `Quality: INSTA-SNAPSHOTS.1 insta dev-dep pin + tests/snapshots.rs baseline (leaf + bounded recursive)` | `insta = "=1.47.2"`; 2 deterministic snapshots; stable on re-run; full suite green. |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative (alongside `DIFFERENTIAL-SIMULATION` and `COVERAGE-INSTRUMENTATION`).
- `2026-05-18`: **`.1` landed** (continuous-PNT while Phase 6
  `.2.4`/`.3.4b` gate-blocked). `insta` pinned `=1.47.2` (Cargo.lock
  unchanged — `"1"` already resolved there); `tests/snapshots.rs`
  with two fully-deterministic baseline snapshots (canonical leaf +
  bounded recursive library, proven-shape config); generated via
  `INSTA_UPDATE=always` and **re-verified stable on a plain re-run**
  (the byte-identical contract holds). `cargo fmt`/`clippy
  -D warnings` clean; full `cargo test` green incl. the new
  `tests/snapshots.rs` binary, no other test regressed. Both Open
  Questions touching `.1` resolved (one `snapshots.rs` driving
  per-test `.snap` files under `tests/snapshots/`). Frontier → `.2`
  (expand to ≥5 shapes).
