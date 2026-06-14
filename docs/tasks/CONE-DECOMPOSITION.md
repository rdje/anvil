# CONE-DECOMPOSITION: split the 5551-line cone.rs into cohesive submodules

## Metadata

- Tree ID: `CONE-DECOMPOSITION`
- Status: `done`
- Roadmap lane: `Code quality / maintainability — generator core readability`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
- Owner: repo-local workflow (owner request 2026-06-14: "carefully and
  meticulously" break cone.rs into interconnected parts)

## Goal

Break `src/gen/cone.rs` (5551 lines) into cohesive, well-named,
interconnected submodules under `src/gen/cone/`, so the generator core is
readable and reviewable. **Pure structural refactor: zero behaviour
change.** Generated RTL stays byte-identical (snapshots + `book_examples`
green without acceptance), all tests pass, no IR/knob/output change.

## Non-Goals

- **No behaviour change.** Not a rewrite, not a logic tweak, not a perf
  change. If a snapshot or `book_examples` byte differs, the extraction is
  wrong — fix the move, never accept the snapshot.
- **No public-API churn for callers.** `src/gen/module.rs`,
  `src/gen/hierarchy.rs`, and `src/ir/compact.rs` reference
  `crate::gen::cone::<symbol>` (e.g. `build_cone_with_retry`,
  `build_outputs_interleaved`, `build_graph_first`, `drain_flop_worklist`,
  `pick_terminal_dep_bearing`, `node_deps`, `make_width_adapter`,
  `FlopWorklist`, `obvious_unsigned_compare_result`,
  `prove_node_exact_value_from_bounds`). Every one of these must keep
  resolving at the same path — achieved by re-exporting moved symbols from
  the cone root (`pub(crate) use <submodule>::*;`).
- Not splitting the `#[cfg(test)] mod tests` in this tree (it stays in the
  root; an optional later tree may split it).

## Acceptance Criteria

- `src/gen/cone.rs` becomes a module root that declares `src/gen/cone/*`
  submodules and keeps only the strategy orchestration + frame types +
  re-exports + tests.
- Each extraction leaf is **byte-identical**: `cargo test --lib`
  (incl. the 42 cone tests + `node_budget`), `cargo test --test snapshots`
  6/6, and `cargo check --all-targets` all green; full `cargo test` at
  milestones (first extraction + closeout).
- `cargo clippy --all-targets -- -D warnings` + `cargo fmt --all --check`
  clean after every leaf.
- `CODEBASE_ANALYSIS.md` module map updated at closeout.

## Decisions

- `2026-06-14` (`.1`): **Rust mechanic — `cone.rs` root + `cone/`
  sibling dir.** Rust 2018 allows `src/gen/cone.rs` to coexist with
  `src/gen/cone/<name>.rs` submodules declared via `mod <name>;` in
  `cone.rs`. No rename of `cone.rs` is needed.
- `2026-06-14` (`.1`): **Flat visibility via root glob re-export.**
  Moved functions become `pub(crate)` (or keep `pub`/`pub(super)` where
  already wider). The root does `mod <name>; pub(crate) use <name>::*;`
  per submodule, so (a) external `crate::gen::cone::<symbol>` paths still
  resolve, and (b) each submodule's `use super::*;` sees all sibling
  items — preserving the original single-file all-see-all namespace with
  minimal per-call import churn. The existing test module already uses
  `use super::*;`, so wildcard imports are accepted by the lint config.
- `2026-06-14` (`.1`): **Extraction order — most self-contained first.**
  `snapshot` (tiny, validates the mechanic) → `semantic` (largest, pure
  `&Module` analysis) → `primitives` → `terminals` → `flops` → `motifs`.
  The strategy orchestration (`build_cone_with_retry`, `build_graph_first`,
  `grow_pool_one_unit`, `build_outputs_interleaved`, `process_signal_frame`,
  `deliver`, `build_cone`, `roll_knob`, `node_budget_reached`, the `Dest`/
  `SignalFrame`/`GateFrame` types, `FlopWorklist`) stays in the root.
- `2026-06-14` (`.1`): **Per-leaf validation = byte-identical-or-bust.**
  A pure code move that compiles + passes the 307 lib tests (incl. 42 cone
  tests) + 6 SV snapshots is behaviour-preserving. Full `cargo test` runs
  at the first extraction (to validate the mechanic end-to-end) and at
  closeout; intermediate leaves use lib+snapshots+check+clippy under
  `scripts/ram_guard.sh`.

## Submodule seam map (target layout)

| Submodule | Holds (representative) |
| --- | --- |
| `cone/snapshot.rs` | `ConstructionSnapshot`, `take_/rollback_construction_snapshot`, `prune_intern_tables_after_node_truncate` |
| `cone/semantic.rs` | value-set + unsigned-bounds + exact-value proofs: `width_mask`, `exact_bound`, `casez_pattern_matches`, `shift_interval_by_exact_addend`, `prove_node_exact_value[_from_bounds]`, `obvious_unsigned_compare_from_bounds`/`_result`, `exact_gate_value`, `collect_small_set`, `Small/TinyValueSet*`, all `*_value_set` fns, `node_support_size`, `can_*`, `small_value_set_min_at_least`, `node_unsigned_bounds` |
| `cone/primitives.rs` | IR builders: `ceil_log2`, `make_constant`, `make_eq_const`, `build_comparison_gate`, `make_mux`, `replicate_to_width`, `make_and/_mul/_sub/_nary_add/_nary_mul`, `make_none_selected`, `or_reduce_terms`, `make_width_adapter`, `emit_terminal_constant` |
| `cone/terminals.rs` | selection + gate-shape policy: `pick_terminal`, `pick_terminal_dep_bearing`, `pick_datas_with_dup_cap`, `pick_signals_with_dup_rate`, `try_share`, `node_deps`, `pick_gate`, `pick_structured_gate`, `pick_slice_gate`, `pick_concat_operand_widths`, `input_widths_for`, `violates_anti_collapse`, `has_duplicate_operand`, `pick_reset_value` |
| `cone/flops.rs` | `drain_flop_worklist[_pool_only]`, `drain_flop_one_hot/_encoded`, `assemble_flop_d_one_hot/_encoded`, `build_flop_leaf`, `pick_mux_arm_count` |
| `cone/motifs.rs` | block/motif builders: comb-mux/case/casez/for-fold (recursive + pool-only), priority encoder, linear-combination, shift, comparand, `make_case_mux`/`make_casez_mux`/`make_for_fold`, `pick_for_fold_*`, `build_casez_patterns`, `is_comparison_op`, coefficient pickers |
| `cone.rs` (root) | strategies + frames + `FlopWorklist` + `roll_knob` + `node_budget_reached` + `mod`/`pub(crate) use` re-exports + `#[cfg(test)] mod tests` |

## Task Tree

- ID: `CONE-DECOMPOSITION`
  Status: `active`
  Goal: `Split cone.rs into cohesive submodules, byte-identical.`
  Children: `.1`, `.2`, `.3`, `.4`, `.5`, `.6`, `.7`

- ID: `CONE-DECOMPOSITION.1`
  Status: `done`
  Goal: `Design the seam map, visibility/re-export strategy, extraction order, and byte-identical validation protocol.`
  Acceptance: `This tree's seam map + Decisions capture the plan; DEVELOPMENT_NOTES records the rationale. Docs-only.`
  Verification: `done — see Verification Log`
  Commit: `CONE-DECOMPOSITION.1 - decomposition design`

- ID: `CONE-DECOMPOSITION.2`
  Status: `done`
  Goal: `Extract cone/snapshot.rs (rollback machinery) — validates the cone.rs-root + cone/ submodule mechanic.`
  Acceptance: `snapshot machinery moved; cargo check/clippy/fmt clean; lib + snapshots byte-identical; FULL cargo test green (first extraction milestone).`
  Verification: `done — moved ConstructionSnapshot + take/rollback/prune to src/gen/cone/snapshot.rs; mod snapshot + pub(crate) use snapshot::* in root; lib 307/307, snapshots 6/6, clippy/fmt clean, full suite green. Gotcha: snapshot fields needed pub(crate) so the root-resident cone tests can still inspect them. See Verification Log.`
  Commit: `CONE-DECOMPOSITION.2 - extract cone/snapshot.rs`

- ID: `CONE-DECOMPOSITION.3`
  Status: `done`
  Goal: `Extract cone/semantic.rs (value-set / bounds / exact-value proofs — the largest, most self-contained chunk).`
  Acceptance: `semantic machinery moved; crate::ir::compact users still resolve via re-export; cargo check/clippy/fmt clean; lib + snapshots byte-identical.`
  Verification: `done — moved width_mask..obvious_unsigned_compare_result (~1360 lines) to src/gen/cone/semantic.rs; mod semantic + pub(crate) use semantic::* in root; one cross-module import (use super::node_deps), HashMap import migrated to the test module. lib 307/307, snapshots 6/6, clippy/fmt clean. See Verification Log.`
  Commit: `CONE-DECOMPOSITION.3 - extract cone/semantic.rs`

- ID: `CONE-DECOMPOSITION.4`
  Status: `done`
  Goal: `Extract cone/primitives.rs (IR-building gate makers + small helpers).`
  Acceptance: `primitives moved; cargo check/clippy/fmt clean; lib + snapshots byte-identical.`
  Verification: `done — moved the contiguous core gate makers (make_constant, make_eq_const, build_comparison_gate, make_mux, replicate_to_width, make_and/_mul/_sub/_nary_add/_nary_mul) to src/gen/cone/primitives.rs; imports use super::{is_comparison_op, node_deps, obvious_unsigned_compare_result}. lib 307/307, snapshots 6/6, clippy/fmt clean. (Mux-assembly helpers or_reduce_terms/make_none_selected, the width-adapter make_width_adapter, ceil_log2, and emit_terminal_constant are non-contiguous and land with their adjacent terminals/motifs blocks in .5/.7.) See Verification Log.`
  Commit: `CONE-DECOMPOSITION.4 - extract cone/primitives.rs`

- ID: `CONE-DECOMPOSITION.5`
  Status: `done`
  Goal: `Extract cone/terminals.rs (terminal/pool selection + gate-shape policy).`
  Acceptance: `terminals moved (incl. pub(super) node_deps / pick_terminal_dep_bearing / make_width_adapter re-exported); cargo check/clippy/fmt clean; lib + snapshots byte-identical.`
  Verification: `done — moved the contiguous block pick_terminal..node_deps (~537 lines, incl. emit_terminal_constant, pick_datas/signals, make_width_adapter, pick_gate/structured/slice, pick_concat_operand_widths, input_widths_for, violates_anti_collapse, has_duplicate_operand, try_share, node_deps) to src/gen/cone/terminals.rs; the 3 pub(super) externally-used fns bumped to pub(crate) for clean re-export. Imports: use super::{ceil_log2, roll_knob} + crate::config::Config + crate::gen::{pool::SignalPool, Generator} + crate::ir + rand::Rng + tracing. cone.rs root no longer used Config (its only Config-typed sigs moved here) → root import removed. lib 307/307, snapshots 6/6, clippy/fmt clean. cone.rs 4048→3511. See Verification Log.`
  Commit: `CONE-DECOMPOSITION.5 - extract cone/terminals.rs`

- ID: `CONE-DECOMPOSITION.6`
  Status: `done`
  Goal: `Extract cone/flops.rs (flop worklist drains + flop-D assemblers).`
  Acceptance: `flop machinery moved (drain_flop_worklist still re-exported); cargo check/clippy/fmt clean; lib + snapshots byte-identical.`
  Verification: `done — moved two contiguous ranges (drain_flop_worklist..assemble_flop_d_encoded incl. the inline ceil_log2/pick_mux_arm_count helpers; build_flop_leaf+pick_reset_value) to src/gen/cone/flops.rs in ONE perl delete pass (original line numbers). drain_flop_worklist stays pub + re-exported for module.rs/hierarchy.rs. drain_flop_worklist_pool_only kept in the root with the other pool-only builders. Imports: use super::{build_cone_with_retry, make_and/_constant/_eq_const/_mux, make_none_selected, or_reduce_terms, replicate_to_width, roll_knob, FlopWorklist} + crate::ir flop types. Cleanup: root dropped now-unused Flop/Node/ResetKind (Node migrated to the test module import). lib 307/307, snapshots 6/6, clippy/fmt clean. cone.rs 3511→3260. See Verification Log.`
  Commit: `CONE-DECOMPOSITION.6 - extract cone/flops.rs`

- ID: `CONE-DECOMPOSITION.7`
  Status: `done`
  Goal: `Extract cone/motifs.rs (block/motif builders) and close out: update CODEBASE_ANALYSIS module map; FULL cargo test; close tree.`
  Acceptance: `motifs moved; root now holds only strategies+frames+re-exports+tests; CODEBASE_ANALYSIS module map updated; FULL cargo test green (closeout milestone); clippy/fmt clean.`
  Verification: `done — moved 36 motif/block builders (~810 lines) to src/gen/cone/motifs.rs via 3 contiguous ranges (one perl delete pass). cone.rs root now holds only the recursion strategy (build_cone_with_retry/build_graph_first/grow_pool_one_unit/build_outputs_interleaved/process_signal_frame/deliver/build_cone/drain_flop_worklist_pool_only/roll_knob/node_budget_reached/frames/FlopWorklist) + tests. ALSO fixed a .6 defect: build_flop_leaf's doc comment had been orphaned in the root (mis-attached to build_comb_mux) — restored it onto build_flop_leaf in flops.rs. CODEBASE_ANALYSIS module map updated. lib 307/307, snapshots 6/6, clippy/fmt clean; FULL cargo test green (closeout milestone). cone.rs 3260→2446 (5551→2446 overall, 56% reduction). See Verification Log.`
  Commit: `CONE-DECOMPOSITION.7 - extract cone/motifs.rs + close`

## Current Frontier

Empty — the tree is `done`. All seven leaves (`.1` design + `.2`–`.7`
extractions) are complete. `src/gen/cone.rs` went from 5551 → 2446 lines
(56% reduction), with the recursion strategy in the root and six cohesive
submodules: `cone/semantic.rs` (~1360), `cone/motifs.rs` (~810),
`cone/terminals.rs` (~560), `cone/flops.rs` (~280), `cone/primitives.rs`
(~210), `cone/snapshot.rs` (~70). Every extraction byte-identical
(snapshots 6/6 throughout; full suite green at `.2` and the `.7` closeout).

## Open Questions

- Should the `#[cfg(test)] mod tests` (the 42 cone tests) eventually move
  alongside their subjects? Deferred — keeping all tests in the root this
  tree keeps each extraction a pure move of non-test code, easier to prove
  byte-identical. A later tree may co-locate tests. Owner: repo-local.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `CONE-DECOMPOSITION.1` | Full function inventory of `src/gen/cone.rs` (grep of all top-level `fn`/`struct`/`enum`/`impl`); external-user audit (`src/gen/module.rs`, `src/gen/hierarchy.rs`, `src/ir/compact.rs`) for the symbols that must stay path-stable. Docs-only; design recorded here + in `DEVELOPMENT_NOTES.md`. memory-architecture + knowledge-map self-checks; `git diff --check`. | passed (docs-only) |
| `2026-06-14` | `CONE-DECOMPOSITION.2` | `cargo check --all-targets` clean; `cargo test --lib` 307/307 (incl. the snapshot/rollback test + 42 cone tests); `cargo test --test snapshots` 6/6 (SV byte-identical); `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; FULL `cargo test` under `scripts/ram_guard.sh --threshold 88` (first-extraction milestone). One fix during the move: `ConstructionSnapshot` fields bumped private→`pub(crate)` so the root-resident cone tests can still inspect them after a snapshot/rollback round-trip. | passed |
| `2026-06-14` | `CONE-DECOMPOSITION.3` | Moved `width_mask`..`obvious_unsigned_compare_result` (~1360 lines) to `src/gen/cone/semantic.rs` via `sed` extract + `perl` visibility bump; `mod semantic; pub(crate) use semantic::*;`. Two fixups: `use super::node_deps;` (the one root symbol the proofs call) and the `std::collections::HashMap` import migrated from the cone root into the test module (the lib no longer uses it; the tests reach it via `use super::*`). `cargo check --all-targets` clean; `cargo test --lib` 307/307; `cargo test --test snapshots` 6/6 (SV byte-identical); `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean. (Full suite deferred to closeout `.7` per protocol.) | passed |
| `2026-06-14` | `CONE-DECOMPOSITION.4` | Moved the contiguous gate-maker block (`make_constant`..`make_nary_mul`, ~195 lines) to `src/gen/cone/primitives.rs`; `mod primitives; pub(crate) use primitives::*;`. Imports: `use super::{is_comparison_op, node_deps, obvious_unsigned_compare_result};` + `crate::ir`/`crate::gen::pool`. `cargo check --all-targets` clean; `cargo test --lib` 307/307; `cargo test --test snapshots` 6/6 (SV byte-identical); clippy/fmt clean. cone.rs 5551→4048. | passed |
| `2026-06-14` | `CONE-DECOMPOSITION.5` | Moved the contiguous `pick_terminal`..`node_deps` block (~537 lines) to `src/gen/cone/terminals.rs`; `mod terminals; pub(crate) use terminals::*;`. The 3 externally-used `pub(super)` fns (`pick_terminal_dep_bearing`, `make_width_adapter`, `node_deps`) bumped to `pub(crate)` (second perl pass). Fixups: `use super::{ceil_log2, roll_knob};` and the now-unused `use crate::config::Config;` removed from the cone root (its only Config-typed signatures moved into terminals). `cargo check --all-targets` clean; `cargo test --lib` 307/307; `cargo test --test snapshots` 6/6 (SV byte-identical); clippy/fmt clean. cone.rs 4048→3511. | passed |
| `2026-06-14` | `CONE-DECOMPOSITION.6` | Moved two contiguous flop ranges (`drain_flop_worklist`..`assemble_flop_d_encoded` incl. `ceil_log2`/`pick_mux_arm_count`; `build_flop_leaf`+`pick_reset_value`) to `src/gen/cone/flops.rs` (one `perl` delete pass over both ranges, original line numbers). Imports `use super::{build_cone_with_retry, make_and/_constant/_eq_const/_mux, make_none_selected, or_reduce_terms, replicate_to_width, roll_knob, FlopWorklist}` + `crate::ir` flop types. Root dropped now-unused `Flop`/`Node`/`ResetKind` (`Node` migrated into the test-module import, used there via `use super::*`). `cargo check --all-targets` clean; `cargo test --lib` 307/307; `cargo test --test snapshots` 6/6 (SV byte-identical); clippy/fmt clean. cone.rs 3511→3260. | passed |
| `2026-06-14` | `CONE-DECOMPOSITION.7` | Moved 36 motif/block builders (~810 lines: pool-only + recursive comb-mux/case/casez/for-fold, priority encoder, linear-combination, shift, comparand, `make_none_selected`/`or_reduce_terms`/`is_comparison_op`) to `src/gen/cone/motifs.rs` via 3 contiguous ranges (one `perl` delete pass). Imports `use super::{build_cone, ceil_log2, make_*, node_deps, pick_*, replicate_to_width, roll_knob, width_mask, FlopWorklist}` (motifs mutually recurse with the root's `build_cone`). Root dropped now-unused `ForFoldKind`. ALSO restored `build_flop_leaf`'s doc comment in `flops.rs` (a `.6` orphan that had mis-attached to `build_comb_mux`). `cargo check --all-targets` clean; `cargo test --lib` 307/307; `cargo test --test snapshots` 6/6 (SV byte-identical); clippy/fmt clean; FULL `cargo test` under `scripts/ram_guard.sh --threshold 88` (closeout milestone). CODEBASE_ANALYSIS module map updated. cone.rs 3260→2446. | passed |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CONE-DECOMPOSITION.1` | `CONE-DECOMPOSITION.1 - decomposition design` | Tree genesis + design. Hash `31571a5`. |
| `CONE-DECOMPOSITION.2` | `CONE-DECOMPOSITION.2 - extract cone/snapshot.rs` | Rollback machinery → `src/gen/cone/snapshot.rs`. Hash `362756d`. |
| `CONE-DECOMPOSITION.3` | `CONE-DECOMPOSITION.3 - extract cone/semantic.rs` | ~1360-line proof machinery → `src/gen/cone/semantic.rs`. Hash `915850f`. |
| `CONE-DECOMPOSITION.4` | `CONE-DECOMPOSITION.4 - extract cone/primitives.rs` | Core gate makers → `src/gen/cone/primitives.rs`. Hash `935aa52`. |
| `CONE-DECOMPOSITION.5` | `CONE-DECOMPOSITION.5 - extract cone/terminals.rs` | Terminal/pool selection → `src/gen/cone/terminals.rs`. Hash `7ac349a`. |
| `CONE-DECOMPOSITION.6` | `CONE-DECOMPOSITION.6 - extract cone/flops.rs` | Flop drains + D assemblers → `src/gen/cone/flops.rs`. Hash `7097cf3`. |
| `CONE-DECOMPOSITION.7` | `CONE-DECOMPOSITION.7 - extract cone/motifs.rs + close` | Motif/block builders → `src/gen/cone/motifs.rs`; tree CLOSED. Pending hash. |

## Changelog

- `2026-06-14`: Created tree; landed `.1` (decomposition design, docs-only). Frontier `.2` (extract `cone/snapshot.rs`).
- `2026-06-14`: Landed `.2` (extract `cone/snapshot.rs`, byte-identical; mechanic validated by full suite). Frontier `.3` (extract `cone/semantic.rs`).
- `2026-06-14`: Landed `.3` (extract `cone/semantic.rs`, ~1360 lines, byte-identical via lib+snapshots). Frontier `.4` (extract `cone/primitives.rs`).
- `2026-06-14`: Landed `.4` (extract `cone/primitives.rs`, core gate makers, byte-identical). Frontier `.5` (extract `cone/terminals.rs`).
- `2026-06-14`: Landed `.5` (extract `cone/terminals.rs`, ~537 lines, byte-identical). cone.rs 5551→3511. Frontier `.6` (extract `cone/flops.rs`).
- `2026-06-14`: Landed `.6` (extract `cone/flops.rs`, ~270 lines, byte-identical). cone.rs 5551→3260. Frontier `.7` (extract `cone/motifs.rs` + closeout).
- `2026-06-14`: Landed `.7` (extract `cone/motifs.rs`, ~810 lines; restored an orphaned `.6` doc; CODEBASE_ANALYSIS map updated; full suite green). cone.rs 5551→2446 (56% reduction). **Tree CLOSED** — 6 cohesive `cone/` submodules + strategy-core root.
