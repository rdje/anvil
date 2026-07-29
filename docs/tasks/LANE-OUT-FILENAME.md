# LANE-OUT-FILENAME: non-DUT `--out` filenames must come from builder truth, not SV re-parse

## Metadata

- Tree ID: `LANE-OUT-FILENAME`
- Status: `done`
- Roadmap lane: Quality / non-DUT lane CLI correctness (no phase reopened)
- Created: `2026-07-29`
- Last updated: `2026-07-29`
- Owner: repo-local workflow

## Goal

Make `anvil --artifact <microdesign|frontend> --out DIR` name its output
files after the artifact's **top** module, as `USER_GUIDE.md` documents,
by carrying the top name in `LaneArtifact` from the lane builder (the
oracle) instead of re-parsing the emitted SV text.

## Observation that opened this tree (`2026-07-29`, `BOOK-LANE-COVERAGE` grounding runs)

- **Measured:** `anvil --artifact frontend --seed 0 --lane-n-params 4
  --lane-n-children 2 --out ./fe-out` writes **`child_0.sv` +
  `child_0.json`**. `USER_GUIDE.md` documents `acc_0.sv` + `acc_0.json`.
  The microdesign lane happens to be correct (`mc_7.sv`) only because its
  SV has a single module.
- **Root cause (DIAGNOSIS):** `src/main.rs` `run_non_dut_lane` derives
  the filename stem via `parse_top_name(&artifact.sv)`, which returns the
  **first** `module` declaration in the SV text — but the frontend
  emitter (`src/frontend/mod.rs` `emit_sv`) emits the child-module stubs
  *before* the top module, so the first declaration is `child_0`. The
  artifact *content* is correct (the manifest records `"top": "acc_0"`);
  only the filename stem is wrong.
- **Design smell:** re-parsing emitted text contradicts the lane
  philosophy ("the builder is the oracle — no analysis pass, no
  re-parse", `src/frontend/mod.rs` module doc). The builder knows the top
  name; the CLI should never re-derive it from text.

## Non-Goals

- No SV or manifest byte changes in any lane (filenames only).
- No DUT-lane `--out` change (`--artifact dut` never routes through
  `run_non_dut_lane`; its per-module naming is untouched).

## Acceptance Criteria

- `LaneArtifact` carries `pub top: String`, populated by all three lanes
  from builder truth (DUT: `design.top`; microdesign: the `mc_<seed>`
  name, single-sourced with `emit_sv`/`emit_manifest`; frontend:
  `unit.top.name`).
- `run_non_dut_lane` uses `artifact.top`; `parse_top_name` is deleted.
- Frontend `--out` writes `acc_<seed>.sv` + `acc_<seed>.json`;
  microdesign still writes `mc_<seed>.sv` + `mc_<seed>.json`.
- Unit proof that each lane's `artifact.top` equals builder truth
  (in particular frontend ≠ `child_0`).
- Full code-hygiene gate green; snapshots byte-identical (no emitted-SV
  change anywhere).

## Task Tree

- ID: `LANE-OUT-FILENAME`
  Status: `done`
  Goal: `--out` filename stems from builder truth.
  Children: `LANE-OUT-FILENAME.1`

- ID: `LANE-OUT-FILENAME.1`
  Status: `done`
  Goal: the fix as one slice — `LaneArtifact.top` + three lane
        populations + `microdesign::top_name(seed)` single-sourcing +
        `run_non_dut_lane` switch + `parse_top_name` deletion + lane-top
        unit test + live docs.
  Acceptance: criteria above.
  Verification: measured REJECT→PASS on the reported defect —
        `--artifact frontend --seed 0 --lane-n-params 4
        --lane-n-children 2 --out DIR` wrote `child_0.sv`/`child_0.json`
        before, writes `acc_0.sv`/`acc_0.json` after; microdesign still
        writes `mc_7.sv`/`mc_7.json`. SV bytes unchanged
        (`diff` of post-fix `acc_0.sv` vs pre-fix `child_0.sv` → identical).
        New lib proof `umbrella::tests::lane_artifact_top_is_builder_truth`
        (all three lanes over seeds 0/7/42; asserts frontend top is
        `acc_<seed>` and not a `child_` stub, and that the manifest's own
        `top` field agrees with the filename stem). Full gate green:
        `cargo check --all-targets`, `cargo clippy --all-targets -D warnings`,
        `cargo fmt --all --check`, and `cargo test` under
        `scripts/ram_guard.sh --threshold 90` → **exit 0**
        (`tests/pipeline.rs` 125/0; `tests/snapshots.rs` **6/6
        byte-identical** — filenames-only change, so the DUT
        reproducibility contract is untouched); focused
        `cargo test --lib umbrella` 9/9 (8 pre-existing + the new proof).
  Commit: `LANE-OUT-FILENAME.1 — --out filenames from builder truth, not SV re-parse`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none) | — | `.1` `done`; the tree's goal is met. Reopen only if a future lane emits a shape whose `top` is not builder-known. |

## Decisions

- `2026-07-29`: Fix = carry the top name in `LaneArtifact` (builder
  truth). Rejected: parsing the manifest JSON for `"top"` in `main.rs`
  (still re-derivation, adds a parse of an emitted artifact); parsing the
  *last* module declaration (heuristic, breaks on future multi-module
  shapes); leaving it and re-documenting `child_0.sv` (documents a bug as
  behavior; the manifest's own `top` field would disagree with the
  filename).

## Open Questions

- None.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-29` | (tree opened) | measured `--out` filenames: frontend `child_0.sv` (wrong) vs USER_GUIDE `acc_0.sv`; microdesign `mc_7.sv` (right, single-module luck); root cause read directly in `src/main.rs` `parse_top_name` + `src/frontend/mod.rs` emit order | tree opened; fix owned by `.1` |
| `2026-07-29` | `LANE-OUT-FILENAME.1` | REJECT→PASS: frontend `--out` `child_0.sv` → `acc_0.sv` (+ `.json`), microdesign unchanged `mc_7.sv`; post-fix SV bytes `diff`-identical to pre-fix; new lib proof `lane_artifact_top_is_builder_truth` (3 lanes × seeds 0/7/42); `cargo check --all-targets` + `clippy -D warnings` + `fmt --check` clean; `cargo test` under `ram_guard --threshold 90` exit 0 (pipeline 125/0, snapshots **6/6 byte-identical**); `cargo test --lib umbrella` 9/9 | `done` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LANE-OUT-FILENAME.1` | `LANE-OUT-FILENAME.1 — --out filenames from builder truth, not SV re-parse` | filenames only; SV + manifest bytes unchanged in all three lanes |

## Changelog

- `2026-07-29`: Created from the `BOOK-LANE-COVERAGE.1` grounding runs.
