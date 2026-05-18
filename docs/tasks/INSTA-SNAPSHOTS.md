# INSTA-SNAPSHOTS: Enforce byte-identical reproducibility with `insta` snapshot tests

## Metadata

- Tree ID: `INSTA-SNAPSHOTS`
- Status: `active`
- Roadmap lane: Quality — reproducibility regressions
- Created: `2026-05-14`
- Last updated: `2026-05-18` (`.3` landed — snapshot-acceptance protocol in COMMIT.md + book; tree CLOSED)
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
  Status: `done`
  Goal: `Add insta snapshots covering ANVIL's reachable generation modes; enforce as pre-commit check.`
  Children: `INSTA-SNAPSHOTS.1` (done), `INSTA-SNAPSHOTS.2` (done), `INSTA-SNAPSHOTS.3` (done)

- ID: `INSTA-SNAPSHOTS.1`
  Status: `done`
  Goal: `Wire insta into Cargo.toml dev-dependencies (explicit pin), create tests/snapshots.rs with one canonical leaf snapshot, and one bounded recursive snapshot. Commit lands the baseline; no drift detection yet.`
  Acceptance: `cargo test --test snapshots passes on main; cargo insta test reports two clean snapshots; no other test regresses.`
  Verification: `Cargo.toml [dev-dependencies] insta "1" → "=1.47.2" (explicit pin to the version in the local registry cache — the snapshot tooling itself must be deterministic; offline-safe; Cargo.lock UNCHANGED since "1" already resolved to 1.47.2). New tests/snapshots.rs with two fully-deterministic fixed-(seed,Config) snapshots via insta::assert_snapshot!: snapshot_canonical_leaf (seed 1, minimal combinational leaf) and snapshot_bounded_recursive_library (seed 11, exact min==max hierarchy depth 2 + exact min==max 2 child instances, library mode — a proven-shape config from tests/pipeline.rs::generates_valid_recursive_hierarchy_designs_with_bounded_shape). Each emit() asserts cfg.validate() + validate_design() before snapshotting. Baselines generated via INSTA_UPDATE=always then RE-RUN without update → both pass (byte-identical/stable: tests/snapshots/snapshots__canonical_leaf.snap 639 B + snapshots__bounded_recursive_library.snap ~227 KB committed). cargo-insta not installed → INSTA_UPDATE env used (no subcommand needed; .3 wires the pre-commit checklist + acceptance protocol). cargo fmt --all --check / clippy --all-targets -- -D warnings clean; full cargo test green incl. the new tests/snapshots.rs binary, no other test regressed (COMMIT.md gate). Open Question resolved: snapshots live under tests/snapshots/ (insta default, per-test .snap files) driven by one tests/snapshots.rs. No book/ change (.3 documents the protocol).`
  Commit: `Quality: INSTA-SNAPSHOTS.1 insta dev-dep pin + tests/snapshots.rs baseline (leaf + bounded recursive)`

- ID: `INSTA-SNAPSHOTS.2`
  Status: `done`
  Goal: `Expand snapshots to cover library/on-demand child sourcing, helper-instance routes, registered/parent-composed routes, and at least one design that exercises canonical_module_signatures (so dedup follow-up work in HIERARCHY-AWARE-IDENTITY can detect snapshot drift caused by dedup).`
  Acceptance: `Snapshots cover ≥5 distinct (seed, config) shapes spanning the listed axes; cargo insta test green.`
  Verification: `tests/snapshots.rs expanded from 2 → 6 fully-deterministic snapshot shapes (≥5, spanning all listed axes): (1) canonical_leaf [.1], (2) bounded_recursive_library [.1 — library child sourcing], (3) bounded_recursive_ondemand [HierarchyChildSourceMode::OnDemand — the other child-sourcing axis], (4) sibling_route [hierarchy_sibling_route_prob=1.0 — helper-instance/sibling route], (5) parent_composed_route [hierarchy_child_input_cone_prob=1.0 + hierarchy_parent_cone_instance_prob=1.0 — parent-composed via parent-cone helper instance], (6) dedup_canonical_signatures [the dedup proof base + hierarchy_module_dedup=true — exercises canonical_module_signatures + the post-finalisation instance-rewrite, so HIERARCHY-AWARE-IDENTITY dedup drift breaks this snapshot]. Every config is fully deterministic (fixed seed, exact min==max bounds where applicable, fixed ConstructionStrategy::Sequential for the route/dedup shapes); each proven from the corresponding tests/pipeline.rs config. emit() asserts cfg.validate() + validate_design() before snapshotting. Baselines generated via INSTA_UPDATE=always then RE-RUN without update → all 6 pass (byte-stable); 4 new tests/snapshots/snapshots__*.snap committed. cargo fmt --all --check / clippy --all-targets -- -D warnings clean; full cargo test green incl. the snapshots binary, no other test regressed (COMMIT.md gate). No book/ change (.3 documents the protocol).`
  Commit: `Quality: INSTA-SNAPSHOTS.2 expand snapshots to 6 shapes (on-demand / sibling / parent-composed / dedup)`

- ID: `INSTA-SNAPSHOTS.3`
  Status: `done`
  Goal: `Add cargo insta test to COMMIT.md's pre-commit checklist and document the snapshot-acceptance protocol (changing a snapshot is a deliberate act) in book/src/knobs.md or a new chapter.`
  Acceptance: `COMMIT.md updated; book/src/* describes the protocol; mdbook build clean.`
  Verification: `COMMIT.md non-negotiable pre-commit checklist item 1: the cargo test line annotated (it already runs tests/snapshots.rs, the insta guard; ≡ cargo insta test when cargo-insta installed) + a new "Snapshot-acceptance protocol (INSTA-SNAPSHOTS)" sub-bullet making the discipline non-negotiable: a failing snapshot is the byte-identical contract catching a real generated-SV change — NEVER silently regenerated; either an unintended drift (fix the cause, do not touch the .snap) or an intended change accepted as a deliberate, separately-reviewed act (cargo insta accept/review, or INSTA_UPDATE=accept after hand-reviewing the diff) committed in the same slice as, and explained by, the generator change; an unexplained .snap change in git status is a workflow violation. book/src/knobs.md "## Reproducibility" gains a friendly, not-scary "### Snapshot guard-rails" subsection (progressive disclosure per book-doctrine): tests/snapshots.rs pins 6 canonical (seed,config) shapes via insta under tests/snapshots/, runs in cargo test, is a contributor-only safety net (never affects the user's tool), and changing a snapshot is a deliberate reviewed act. No runnable ```bash fence added (cargo insta accept shown as inline code) ⇒ the BOOK-EXAMPLES-RUNNABLE harness is unaffected, no skip-sentinel bookkeeping. Docs/workflow only — no src/tests change (cargo test unchanged-green vs base c1acfb5); mdbook build book clean; mdbook test book exit 0; cargo fmt --all --check clean. Closes the INSTA-SNAPSHOTS tree.`
  Commit: `Docs: INSTA-SNAPSHOTS.3 snapshot-acceptance protocol (COMMIT.md checklist + book)`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | — | `CLOSED` | Tree complete. `.1` (insta `=1.47.2` pin + leaf/recursive baseline) → `.2` (6 byte-stable shapes spanning every reachable generation axis incl. dedup-canonical-signatures) → `.3` (`COMMIT.md` non-negotiable snapshot-acceptance protocol + `book/src/knobs.md` "Snapshot guard-rails") all `done`. The byte-identical-reproducibility contract is now directly enforced and the acceptance discipline is codified; no remaining work. |

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
| `2026-05-18` | `INSTA-SNAPSHOTS.2` | `tests/snapshots.rs` 2 → 6 deterministic shapes: + `bounded_recursive_ondemand` (OnDemand child sourcing), `sibling_route` (helper-instance/sibling route), `parent_composed_route` (parent-cone-instance + child-input-cone), `dedup_canonical_signatures` (dedup base + `hierarchy_module_dedup=true` — exercises `canonical_module_signatures` + instance-rewrite). Fixed seeds / `min==max` bounds / `Sequential` strategy ⇒ byte-stable; generated via `INSTA_UPDATE=always` then **re-run → all 6 pass**. `cargo fmt --all --check` / `clippy --all-targets -- -D warnings` clean; full `cargo test` green, no regression. | Done. ≥5-shapes acceptance met; frontier → `.3`. |
| `2026-05-18` | `INSTA-SNAPSHOTS.3` | `COMMIT.md` checklist item 1 annotated + a non-negotiable "Snapshot-acceptance protocol" sub-bullet (failing snapshot ⇒ never silently regenerated; deliberate reviewed `cargo insta accept`/`INSTA_UPDATE=accept` in the same slice as the explaining generator change; unexplained `.snap` change = workflow violation). `book/src/knobs.md` "## Reproducibility" gains a friendly "### Snapshot guard-rails" subsection (no runnable ```bash fence ⇒ BOOK-EXAMPLES-RUNNABLE harness unaffected). Docs/workflow only, no `src/`/`tests/` (cargo test unchanged-green vs base `c1acfb5`); `mdbook build book` clean; `mdbook test book` exit 0; `cargo fmt --all --check` clean. | Done. Closes the INSTA-SNAPSHOTS tree. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INSTA-SNAPSHOTS.1` | `Quality: INSTA-SNAPSHOTS.1 insta dev-dep pin + tests/snapshots.rs baseline (leaf + bounded recursive)` | `insta = "=1.47.2"`; 2 deterministic snapshots; stable on re-run; full suite green. |
| `INSTA-SNAPSHOTS.2` | `Quality: INSTA-SNAPSHOTS.2 expand snapshots to 6 shapes (on-demand / sibling / parent-composed / dedup)` | +4 deterministic shapes spanning all listed axes; 6/6 byte-stable; full suite green. |
| `INSTA-SNAPSHOTS.3` | `Docs: INSTA-SNAPSHOTS.3 snapshot-acceptance protocol (COMMIT.md checklist + book)` | COMMIT.md non-negotiable protocol + book "Snapshot guard-rails"; docs/workflow only; mdbook clean. Closes the tree. |

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
- `2026-05-18`: **`.2` landed** (continuous-PNT while Phase 6
  `.2.4`/`.3.4b` gate-blocked). `tests/snapshots.rs` 2 → 6
  fully-deterministic shapes covering every listed axis: library
  *and* on-demand child sourcing, helper-instance/sibling route,
  parent-composed (parent-cone-instance) route, and a
  `hierarchy_module_dedup`-on design exercising
  `canonical_module_signatures` (so `HIERARCHY-AWARE-IDENTITY` dedup
  drift breaks a snapshot). Each config proven from the
  corresponding `tests/pipeline.rs` shape; fixed seed / `min==max`
  bounds / `Sequential` strategy ⇒ byte-stable (generated via
  `INSTA_UPDATE=always`, re-verified on plain re-run, all 6 pass).
  Full `cargo test` green, no regression. Frontier → `.3`
  (`COMMIT.md` checklist + book acceptance-protocol — closes the
  tree).
- `2026-05-18`: **`.3` landed → INSTA-SNAPSHOTS tree CLOSED**
  (continuous-PNT while Phase 6 `.2.4`/`.3.4b` gate-blocked).
  `COMMIT.md`'s non-negotiable pre-commit checklist now annotates
  the `cargo test` line (it already runs the `insta` guard) and
  carries a **Snapshot-acceptance protocol**: a failing snapshot is
  never silently regenerated — it is either an unintended drift
  (fix the cause) or a deliberate, separately-reviewed acceptance
  (`cargo insta accept`/`review`, or `INSTA_UPDATE=accept` after
  hand-review) committed in the same slice as, and explained by,
  the generator change; an unexplained `.snap` change is a workflow
  violation. `book/src/knobs.md` "## Reproducibility" gains a
  friendly, not-scary "### Snapshot guard-rails" subsection
  (progressive disclosure; no runnable ```bash fence ⇒ the
  `BOOK-EXAMPLES-RUNNABLE` harness is unaffected). Docs/workflow
  only; `mdbook build`/`mdbook test` clean; no `src/`/`tests/`
  change. The byte-identical contract is now both *enforced* (by
  `cargo test`) and its acceptance discipline *codified*. Tree
  complete.
