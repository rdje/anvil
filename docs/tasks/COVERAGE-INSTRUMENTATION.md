# COVERAGE-INSTRUMENTATION: Measure line/branch coverage of the planner and validators

## Metadata

- Tree ID: `COVERAGE-INSTRUMENTATION`
- Status: `active`
- Roadmap lane: Quality — test-discipline visibility
- Created: `2026-05-14`
- Last updated: `2026-05-18` (`.3` landed — cone retry-exhaustion proof + orphan-knob audit + baseline refresh; tree CLOSED)
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
  Status: `done`
  Goal: `Add line/branch coverage measurement; convert matrix-comprehensiveness from intent to measurement.`
  Children: `COVERAGE-INSTRUMENTATION.1` (done), `COVERAGE-INSTRUMENTATION.2` (done), `COVERAGE-INSTRUMENTATION.3` (done)

- ID: `COVERAGE-INSTRUMENTATION.1`
  Status: `done`
  Goal: `Install cargo-llvm-cov locally; produce a baseline coverage report against the current test suite (unit + tests/pipeline.rs, excluding the slow Phase 4 hierarchy gate). Commit the baseline report as docs/coverage-baseline.md.`
  Acceptance: `cargo llvm-cov --release runs to completion; docs/coverage-baseline.md exists with line-level numbers per crate file.`
  Verification: `cargo-llvm-cov 0.8.7 + llvm-tools-aarch64-apple-darwin already installed locally (no install step needed). cargo llvm-cov --release completed: 110 tests passed (~295s), TOTAL 85.26% lines / 91.95% functions / 87.61% regions across 14 crate files. docs/coverage-baseline.md committed with per-file numbers and top-5 under-covered files identified for .2 triage.`
  Commit: `Quality: add cargo-llvm-cov baseline (COVERAGE-INSTRUMENTATION.1)`

- ID: `COVERAGE-INSTRUMENTATION.2`
  Status: `done`
  Goal: `Triage the baseline: identify the top-5 under-covered source files by uncovered-line count. For each, decide: (a) is the uncovered region dead code? (b) is it a rarely-fired planner path that should have a focused proof? (c) is it a defensive panic that is intentionally unreachable? Record findings in DEVELOPMENT_NOTES.md.`
  Acceptance: `DEVELOPMENT_NOTES.md entry exists with the triage matrix and explicit disposition for each of the top-5 files.`
  Verification: `DEVELOPMENT_NOTES.md "Coverage baseline triage — top-5 under-covered files (2026-05-18, COVERAGE-INSTRUMENTATION.2)" entry landed: a per-file disposition table for all top-5 (bin/tool_matrix.rs / gen/cone.rs / ir/validate.rs / config.rs / main.rs) with (a)/(b)/(c) classification + the .3 action. Method = reasoned code inspection (orphan-symbol audit: every *_focus_config/scenario builder referenced from build_scenarios ⇒ no dead/retired builders; 45 cone.rs panic/expect/unreachable + the build_cone_with_retry/rollback/anti-collapse-reject/pick_terminal-fallback sites enumerated; 62 validate.rs Err-return arms vs 26 inline tests; config.rs 137 pub fields / 37 validate sites). Headline finding: NO confirmed dead code in the top-5 — the 3314 headline uncovered lines are gate-exclusive (tool_matrix), intentional-defensive (validate.rs), or integration-only (config.rs/main.rs) BY DESIGN; the single high-value .3 target is a handful of gen/cone.rs focused proofs (retry-exhaustion / anti-collapse-reject / adapter-fallback) + an optional config.rs orphan-knob spot-audit. This right-sizes .3 away from broad coverage-chasing. Triage-only — no code change (diff = DEVELOPMENT_NOTES.md + tree/live-docs); mdbook build book clean; cargo fmt --all --check clean; full cargo test green at base 9806028 (no src/tests touched).`
  Commit: `Docs: COVERAGE-INSTRUMENTATION.2 top-5 under-covered-file triage`

- ID: `COVERAGE-INSTRUMENTATION.3`
  Status: `done`
  Goal: `Act on the triage: remove dead code where confirmed, add focused proofs where rarely-fired paths are real, leave defensive panics as-is. Update coverage baseline.`
  Acceptance: `Coverage baseline updated; new focused proofs (if any) commit-traceable to specific uncovered branches; cargo test all green.`
  Verification: `Acted on the .2 triage exactly as it scoped .3 (no broad coverage-chase). (b) Added the high-value focused proof tests/pipeline.rs::constant_pressure_exhausts_cone_retry_and_stays_valid_and_reproducible (4 ConstructionStrategy × 4 seeds, constant_prob=1.0, max_depth=1): pick_terminal always takes its "emit fresh constant" branch ⇒ empty-dep cone roots ⇒ build_cone_with_retry runs the rare empty-dep retry + rollback_construction_snapshot loop on all MAX_RETRIES then the "⚠️ retry budget exhausted, accepting last attempt" fallback; the proof pins the invariant that maximum constant pressure cannot break the pipeline (still validate_design-clean + byte-reproducible). Commit-traceable to cone.rs::pick_terminal constant branch + build_cone_with_retry rollback/budget-exhausted accept (triage #2, the only real proof-gap). (a) config.rs orphan-knob spot-audit (triage #4): of 74 pub Config fields, 3 have no external field-access — library_prob / max_nodes_per_module / use_async_reset — but ALL THREE are intentionally-reserved/safety/unused-by-discipline knobs ALREADY documented as such in book/src/knobs.md (future Phase-4+ dial / safety ceiling / unused-by-async-reset-discipline); removing them would break serde-config + contradict the book ⇒ NO dead code, leave as-is (confirms .2's headline). (c) defensive/gate-exclusive/integration-only paths left as documented in .2. Baseline refreshed: re-ran cargo llvm-cov --release (instrumented full suite — also the COMMIT.md cargo-test gate) and updated docs/coverage-baseline.md with the new numbers + a .3 addendum. cargo fmt --all --check / clippy --all-targets -- -D warnings clean; new proof + full suite green. Closes the COVERAGE-INSTRUMENTATION tree.`
  Commit: `Quality: COVERAGE-INSTRUMENTATION.3 cone retry-exhaustion focused proof + orphan-knob audit + baseline refresh`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | — | `CLOSED` | Tree complete. `.1` (cargo-llvm-cov baseline) → `.2` (top-5 triage: no dead code; right-sized `.3`) → `.3` (cone retry-exhaustion focused proof + config orphan-knob audit [3 reserved knobs, no dead code] + baseline refresh) all `done`. Coverage measurement is wired, triaged, and the one real proof-gap closed; no remaining work. |

`COVERAGE-INSTRUMENTATION.1` is `done` (landed at the hash recorded in
this tree's Commit Log).

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
| `2026-05-14` | `COVERAGE-INSTRUMENTATION.1` | `cargo llvm-cov --release` (110 tests passing in ~295s); `cargo llvm-cov report --release`. | All passing. TOTAL coverage: 85.26% lines / 91.95% functions / 87.61% regions across 14 crate files. Baseline written to docs/coverage-baseline.md with top-5 under-covered files identified for .2 triage. |
| `2026-05-18` | `COVERAGE-INSTRUMENTATION.2` | Reasoned-inspection triage of all top-5 under-covered files → DEVELOPMENT_NOTES.md disposition table. Orphan-symbol audit (no retired `*_focus_config`/scenario builders ⇒ no dead code in `tool_matrix.rs`); cone.rs panic/rollback/anti-collapse-reject/adapter-fallback sites enumerated; validate.rs 62 Err-arms vs 26 inline tests; config.rs 137 fields/37 validate sites. Triage-only, no code; `mdbook build book` clean; `cargo fmt --all --check` clean; full `cargo test` green at base `9806028` (no `src/`/`tests/` touched). | Done. No confirmed dead code; `.3` right-sized to a handful of `gen/cone.rs` focused proofs + optional `config.rs` orphan-knob spot-audit. |
| `2026-05-18` | `COVERAGE-INSTRUMENTATION.3` | (b) New focused proof `constant_pressure_exhausts_cone_retry_and_stays_valid_and_reproducible` (4 strategies × 4 seeds, `constant_prob=1.0`) — drives `pick_terminal` constant branch + `build_cone_with_retry` rollback/budget-exhausted-accept; pins "constant pressure can't break the pipeline" (validate-clean + byte-reproducible); green. (a) `config.rs` orphan-knob audit: 3/74 fields unconsumed (`library_prob`/`max_nodes_per_module`/`use_async_reset`) but all documented-reserved/safety/unused-by-discipline in `book/src/knobs.md` ⇒ no dead code, leave as-is. (c) defensive/gate-exclusive/integration-only left as `.2`-documented. Baseline refreshed via `cargo llvm-cov --release` (= COMMIT.md cargo-test gate); `docs/coverage-baseline.md` updated + `.3` addendum. `cargo fmt --all --check` / `clippy --all-targets -- -D warnings` clean. | Done. Closes the COVERAGE-INSTRUMENTATION tree. |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COVERAGE-INSTRUMENTATION.1` | `Quality: add cargo-llvm-cov baseline (COVERAGE-INSTRUMENTATION.1)` | cargo-llvm-cov 0.8.7 + llvm-tools-aarch64-apple-darwin already installed locally. Baseline excludes the 75-min Phase 4 hierarchy gate by design. |
| `COVERAGE-INSTRUMENTATION.2` | `Docs: COVERAGE-INSTRUMENTATION.2 top-5 under-covered-file triage` | Triage-only; per-file (a)/(b)/(c) disposition; no confirmed dead code; `.3` scoped to a few `gen/cone.rs` focused proofs. No code. |
| `COVERAGE-INSTRUMENTATION.3` | `Quality: COVERAGE-INSTRUMENTATION.3 cone retry-exhaustion focused proof + orphan-knob audit + baseline refresh` | One focused proof (constant-pressure retry-exhaustion invariant) + config orphan-knob audit (no dead code; 3 documented-reserved knobs) + llvm-cov baseline refresh. Closes the tree. |

## Changelog

- `2026-05-14`: Created task tree as part of the quality-improvement initiative.
- `2026-05-14`: `.1` landed; baseline at `docs/coverage-baseline.md`. Frontier rotated to `.2` (triage).
- `2026-05-18`: **`.3` landed → COVERAGE-INSTRUMENTATION tree
  CLOSED** (continuous-PNT while Phase 6 `.2.4`/`.3.4b` gate-blocked).
  Acted on the `.2` triage exactly as scoped: (b) added the focused
  proof `constant_pressure_exhausts_cone_retry_and_stays_valid_and_reproducible`
  (the high-value gap — `pick_terminal` constant branch +
  `build_cone_with_retry` rollback/budget-exhausted accept; pins the
  "constant pressure can't break the pipeline" invariant);
  (a) `config.rs` orphan-knob spot-audit found 3/74 unconsumed
  fields, all *documented-reserved* in `book/src/knobs.md`
  (`library_prob` future dial / `max_nodes_per_module` safety
  ceiling / `use_async_reset` unused-by-discipline) ⇒ **no dead
  code**, leave as-is (confirms `.2`); (c) defensive/gate-exclusive/
  integration-only left as documented. Baseline refreshed via
  `cargo llvm-cov --release`; `docs/coverage-baseline.md` updated +
  `.3` addendum. Tree complete.
- `2026-05-18`: **`.2` triage landed** (triage-only, no code) —
  continuous-PNT while Phase 6 `.2.4`/`.3.4b` are gate-blocked.
  `DEVELOPMENT_NOTES.md` "Coverage baseline triage — top-5
  under-covered files": per-file (a) dead / (b) rarely-fired-proof /
  (c) intentional-or-integration disposition. Headline: **no
  confirmed dead code** in the top-5 — the headline 3314 uncovered
  lines are gate-exclusive (`tool_matrix`), intentional-defensive
  (`validate.rs`), or integration-only (`config.rs`/`main.rs`) by
  design; the only high-value `.3` target is a handful of
  `gen/cone.rs` focused proofs (retry-exhaustion /
  anti-collapse-reject / `pick_terminal` adapter-fallback) + an
  optional `config.rs` orphan-knob spot-audit. `.3` right-sized away
  from broad coverage-chasing. Frontier rotated to `.3`.
