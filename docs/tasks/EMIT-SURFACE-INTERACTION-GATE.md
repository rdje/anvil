# EMIT-SURFACE-INTERACTION-GATE: nine emit projections, never proven together

## Metadata

- Tree ID: `EMIT-SURFACE-INTERACTION-GATE`
- Status: `active`
- Roadmap lane: Quality / signoff (steering gaps 1 + 3); serves `STRUCTURED-EMISSION-EXPANSION`
- Created: `2026-07-30`
- Last updated: `2026-07-30`
- Owner: repo-local workflow (agent-picked under the owner's `2026-07-30` autonomy directive)

## Goal

Prove the nine structured-emission surfaces are downstream-clean **in combination**,
and make `--profile structured-emission-max` mean what its name says.

## Observation that opened this tree (`2026-07-30`, measured)

Two facts, both re-derivable:

**1. Every gate exercises exactly one surface.**

```
$ for f in function_emit generate_loop task_emit multi_output_task cone_function \
           mux_if case_mux_if casez_mux_if; do
    awk "/fn ${f}_focus_config/,/^}/" src/bin/tool_matrix.rs | grep -cE '_emit_prob = 1\.0'
  done
1 1 1 1 1 1 1 1
```

**2. The preset covers 4 of 9.**

```
$ awk '/structured-emission-max/,/^\s*}/' src/config.rs | grep 'emit_prob'
function_emit_prob: Some(1.0)
generate_loop_emit_prob: Some(1.0)
task_emit_prob: Some(1.0)
cone_function_emit_prob: Some(1.0)
```

Missing: `multi_output_task_emit_prob`, `mux_if_emit_prob`, `case_mux_if_emit_prob`,
`casez_mux_if_emit_prob` (and `soft_union_slice_prob`, which is version-gated and
may reasonably stay out). The preset predates surfaces 6–9 and its name is now
false.

## Why this matters more than it looks

The nine passes are **mutually exclusive on a gate**. That exclusion is the entire
soundness argument for stacking nine overlapping projections: each pass calls a
`sibling_marked`-style predicate and skips any gate another pass already claimed,
and the passes run in a fixed order (`function_emit` → `generate_loop` →
`task_emit` → `multi_output_task` → `cone_function` → `mux_if` → `case_mux_if` →
`casez_mux_if`, with `soft_union` earliest).

With exactly one probability at `1.0` and the rest at `0.0`, **every sibling set is
empty**. So the exclusion predicates are, under every gate this repo owns, dead
code that always returns `false`. The one property holding the surfaces apart has
no test.

That is also precisely the shape ANVIL exists to generate: legal, unusual,
interaction-heavy RTL (ROADMAP steering gap 1). A module carrying a `function
automatic`, a `generate for`, a multi-output `task`, a cone function, a procedural
`if/else` and two masked priority chains **at once** is far more interesting to a
downstream tool than eight modules each carrying one.

**This is not a claim that any surface is wrong.** Each is individually banked
downstream-clean, and the exclusion logic is written and unit-tested at the
function level. The gap is that no *end-to-end downstream* run has ever had two of
them live simultaneously.

## Non-Goals

- **Not** a tenth surface. `STRUCTURED-EMISSION-EXPANSION` `.20+` owns new surfaces
  (nested `generate`; `interface`/`modport` remains empirically disqualified).
- **Not** changing any surface's semantics or its default. All nine stay
  default-off ⇒ DUT byte-identical.
- **Not** retiring the eight single-surface gates. They isolate a regression to one
  surface; a combined gate cannot. Both are wanted (`feedback_never_retire_strategies`).

## Acceptance Criteria

- A repo-owned `tool_matrix` gate runs all eight `*_emit_prob` surfaces at once and
  is downstream-clean (Verilator + both Yosys modes + Icarus), `coverage_gaps = []`.
- The report proves **co-occurrence**, not just cleanliness: a single emitted module
  carries ≥ 2 distinct surfaces, keyed off the existing per-surface metrics rather
  than a new identifier token (the `case_mux_if` metric-keyed precedent).
- `--profile structured-emission-max` sets every non-version-gated surface, with a
  test pinning preset ↔ knob-list agreement so it cannot drift again.
- Evidence banked as a `docs/evidence/` digest (decision `0030`) — this tree is the
  natural second customer of that mechanism.

## Task Tree

- ID: `EMIT-SURFACE-INTERACTION-GATE`
  Status: `active`
  Children: `.1` (design), `.2` (preset + drift test), `.3` (the combined gate),
            `.4` (harden the cone-absorption consumer census — opened by `.1`)

- ID: `EMIT-SURFACE-INTERACTION-GATE.1`
  Status: `done` (`2026-07-30`)
  Goal: design ADR — decide (a) whether the combined gate is one scenario with all
        eight on or a small sweep, (b) how co-occurrence is *proven* from metrics
        without a new token, (c) whether `soft_union_slice_prob` joins (it is
        version-gated and Yosys/Icarus reject it, so probably a separate
        Verilator-only scenario), and (d) the expected interaction risks worth
        naming up front — above all `cone_function`'s interior **absorption** vs the
        other passes' per-gate marking, which is the one pair that does not merely
        skip but *suppresses another gate's module wire*.
  Acceptance: a `docs/decisions/00NN-*.md` with Context / Decision / Consequences.
  Delivered: [`docs/decisions/0032-emit-surface-interaction-gate.md`](../decisions/0032-emit-surface-interaction-gate.md).
        (a) a 3 + 1 sweep — three universal comb-only scenarios (one per construction
        strategy) with all eight surfaces at `0.25`, plus one saturation scenario at
        `1.0`; (b) a derived `distinct_emit_surfaces` count projected from the nine
        `ModuleReport.emitted_*` booleans that already exist (the decision `0028`
        metric-keyed precedent — no new token); (c) `soft_union` joins as a separate
        Verilator-only `--sv-version 2023` scenario, kept out of the universal three
        so they retain their Yosys + Icarus columns; (d) five interaction risks
        reasoned through from source, four predicted-and-measured clean, one
        (`compute_use_counts` omits `Memory`/`Fsm` consumers) split out as `.4`.
        Plus the pivotal measurement: `--profile structured-emission-max` sets four
        surfaces and emits **one**.

- ID: `EMIT-SURFACE-INTERACTION-GATE.2`
  Status: `done` (`2026-07-30`)
  Goal: make `--profile structured-emission-max` set every non-version-gated
        surface **at `0.25`** (decision `0032` (e) — all-at-`1.0` is measured to emit
        one surface), raise the three selector-shape knobs so the procedural surfaces
        have candidates, correct the preset description, and add a test asserting the
        preset covers exactly the intended knob set so a tenth surface cannot silently
        omit itself. Also correct the two user-facing statements that describe the
        preset (`USER_GUIDE.md:363`, `README.md:875`) and the Knowledge Map `reverify`
        line for `knob-presets-and-cli-flags`, which pins the old four-knobs-at-`1.0`
        expectation.
  Acceptance: preset ↔ knob-list test green; `--dump-config --profile
        structured-emission-max` shows all eight; a single module under the preset
        carries ≥ 2 distinct emitted surfaces; default path byte-identical.
  Delivered: two `pub const`s in `src/config.rs` —
        `STRUCTURED_EMISSION_MAX_PROB = 0.25` and
        `STRUCTURED_EMISSION_MAX_SELECTOR_PROB = 0.35` — with the preset re-pointed
        at them (eight `*_emit_prob` + `comb_mux_prob`/`case_mux_prob`/
        `casez_mux_prob`), a rewritten description that says *why not `1.0`*, and
        the anti-drift test
        `structured_emission_max_preset_covers_every_non_version_gated_surface`,
        which derives the required knob set from `knob_catalog()`'s
        `structured_emission` group minus the version-gated
        `soft_union_slice_prob` — so a tenth surface joins by construction or the
        test fails. Measured: the preset emits **8/8** surfaces on seeds 1–5 (was
        1/8), and a 12-module corpus is clean on all four tool columns with
        **every** module carrying all eight. Docs corrected in `README.md`,
        `USER_GUIDE.md`, `book/src/knobs.md`, the new
        `book/src/structured-emission.md` "Combining the surfaces" section (three
        runnable examples), and the `knob-presets-and-cli-flags` KM card + its
        `reverify` line.

- ID: `EMIT-SURFACE-INTERACTION-GATE.3`
  Status: `pending`
  Goal: the repo-owned combined gate + coverage facts + banked digest, per decision
        `0032` (a)/(b)/(c): `--emit-surface-interaction-gate`, the derived
        `distinct_emit_surfaces`, and the facts
        `saw_multi_surface_emit_interaction` (≥ 2),
        `saw_all_emit_surfaces_in_one_module` (≥ 8), and
        `saw_all_nine_emit_surfaces_in_one_module` (the 2023 scenario).
  Acceptance: `coverage_gaps = []`, all tool columns clean, co-occurrence facts lit,
        digest committed under `docs/evidence/`.

- ID: `EMIT-SURFACE-INTERACTION-GATE.4`
  Status: `pending`
  Goal: harden `cone_function_emit::compute_use_counts` to count `Memory`
        (`we`/`waddr`/`wdata`/`raddr`) and `Fsm` (`sel`) consumers. Today the census
        omits them, so a gate consumed once by a cone edge and once by a memory port
        would read as single-use, be absorbed, and have its module wire deleted out
        from under the memory block. Unreachable today only because
        `build_memory_leaf` / `build_fsm_block` construct gate-free modules — an
        accident of shape, not a stated invariant.
  Acceptance: the census counts both; a regression test pins that a gate feeding a
        memory port is never absorbed; `tests/snapshots.rs` untouched (a provable
        no-op on every currently-constructible module ⇒ DUT byte-identical).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `.3` | `pending` | **Current frontier.** The gate + the digest. It reuses the same `0.25` configuration `.2` just shipped, so the gate is literally a proof of the preset. |
| 2 | `.4` | `pending` | The absorption-census hardening. Byte-identical today; ordered after the gate because it is a latent-trap fix, not a blocker. |
| — | `.2` | `done` | Preset honest: 8/8 surfaces emitted (was 1/8), anti-drift test derived from the knob catalog, docs + book corrected. |
| — | `.1` | `done` | Design ADR landed as decision `0032`. |

## Decisions

- `2026-07-30`: Opened as its own tree rather than as `STRUCTURED-EMISSION-EXPANSION.20`,
  because `.20+` in that lane is reserved for *new surfaces* and this is a gate over
  existing ones. Cross-referenced from that tree instead.
- `2026-07-30`: Agent-picked under the owner's standing autonomy directive
  (`MEMORY.md` standing directives, `2026-07-30`) rather than surfaced as a question.
- `2026-07-30` (`.1`, decision `0032`): the gate is a **3 + 1 sweep** at an
  intermediate probability, not one all-at-`1.0` scenario — because all-at-`1.0` is
  measured to collapse to 3 live surfaces (the first pass in the fixed order claims
  every gate its candidate set overlaps). `0.25` is adopted for both the gate and the
  preset; it maximises the *least-represented* surface's count (max-min over seeds
  1–5).
- `2026-07-30` (`.1`, decision `0032`): the preset's meaning is pinned as maximal
  surface **diversity**, not maximal saturation of one surface — the two are opposed
  under mutual exclusion, and the current preset picks the wrong one.

## Open Questions

- ~~Does `cone_function` absorption interact safely with a `multi_output_task` member
  or a `mux_if` output var when both are live?~~ **Resolved at `.1`** (decision `0032`
  §6 + (d)): yes, on both counts — absorption requires a *global* use count of `1`, a
  `multi_output_task` member is `sibling_marked` (so never a root or interior), and a
  `mux_if`-marked `Mux` is excluded from the cone pass which runs before it. Predicted
  clean from source, then measured clean (215 multi-output tasks co-existing with 131
  cone functions and 138 `mux_if` blocks in one 24-module corpus, four tool columns
  clean).
- **New, opened by `.1`:** should `.4`'s consumer census be factored into a shared
  `ir::use_counts` helper rather than living privately in `cone_function_emit.rs`? Any
  future absorbing pass needs the identical census
  (`feedback_full_factorization`).

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-30` | (tree opened) | Measured: 8/8 focus configs set exactly one `_emit_prob = 1.0`; the `structured-emission-max` preset sets 4 of 9 | observation recorded; tree registered |
| `2026-07-30` | `.1` | Source audit of all nine `annotate_*` exclusion predicates | exclusion matrix is complete and lower-triangular — every pass excludes exactly its predecessors; no hole |
| `2026-07-30` | `.1` | `anvil --seed {1,2,3} --profile structured-emission-max --introspect` | preset sets 4 surfaces, emits **1** — 796 / 1057 / 917 combinational functions, every other surface exactly `0` |
| `2026-07-30` | `.1` | All eight `*_emit_prob = 1.0`, comb-only selector-rich shape, 12 modules | exactly **3** live surfaces on 12/12 modules (saturation collapse); clean on Verilator + both Yosys modes + Icarus |
| `2026-07-30` | `.1` | All eight at `0.25`, same shape, 24 modules | **8** distinct surfaces in 20/24 modules, 7 in 4/24; clean 24/24 on Verilator `--lint-only`, Yosys without-abc, Yosys with-abc, and `iverilog -g2012`, **zero warnings** |
| `2026-07-30` | `.1` | Same, per construction strategy (12 modules each) | per-module surface floor `sequential` 6 / `shuffled` 8 / `interleaved` 7; max `8` on all three |
| `2026-07-30` | `.1` | Probability calibration, default shape, seeds 1–5 | `0.25` maximises the min per-surface count (mean min 73.8 vs 51.4 / 70.2 / 37.0 at `0.15` / `0.35` / `0.50`) |
| `2026-07-30` | `.1` | Nine surfaces under `--sv-version 2023 --soft-union-slice-prob 1.0`, 8 modules | Verilator-clean under `--language 1800-2023`; 8/8 genuinely emit `union soft`; 6/8 also carry all eight other surfaces |
| `2026-07-30` | `.1` | Negative control: the same shape with **all** surfaces off, `-Wall` | 23/24 fail `UNUSEDSIGNAL` — the `-Wall` failures were a shape artifact, not surface stacking; the repo's bar is bare `--lint-only` (`src/downstream/mod.rs:154`) |
| `2026-07-30` | `.1` | `--memory-prob 1.0 --cone-function-emit-prob 1.0`, 40 modules | no undeclared-identifier failure — the `compute_use_counts` gap is unreachable today because `build_memory_leaf` / `build_fsm_block` construct gate-free modules; recorded as `.4` |
| `2026-07-30` | `.2` | `cargo test --lib config::` | 36/36 pass, including the new drift gate |
| `2026-07-30` | `.2` | **Negative control 1** — delete `mux_if_emit_prob` from the preset | drift gate FAILS with an actionable message naming the omitted surface; restored |
| `2026-07-30` | `.2` | **Negative control 2** — set `task_emit_prob` to `1.0` instead of the shared value | drift gate FAILS (`must use the shared intermediate probability`); restored |
| `2026-07-30` | `.2` | `anvil --seed {1..5} --profile structured-emission-max --introspect` | **8/8 surfaces live on every seed** (was 1/8 before this leaf) |
| `2026-07-30` | `.2` | 12-module preset corpus, seed 42, all four tool columns | 12/12 clean on Verilator `--lint-only`, Yosys without-abc, Yosys with-abc, `iverilog -g2012`; **zero** warnings; all 12 modules carry all 8 surfaces |
| `2026-07-30` | `.2` | The book's new sv2023 example (`--profile structured-emission-max --sv-version 2023 --soft-union-slice-prob 1.0`, 4 modules) | exit 0; 4/4 Verilator-clean under `--language 1800-2023`; 4/4 genuinely emit `union soft` |
| `2026-07-30` | `.2` | `mdbook build book` + `cargo test --test book_examples` | exit 0; 3/3 harness tests pass, covering the three new runnable examples |
| `2026-07-30` | `.2` | `cargo test` (full suite, under `scripts/ram_guard.sh --threshold 90`) | green — `tests/snapshots.rs` byte-identical (default path untouched) |
| `2026-07-30` | `.2` | `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` | clean |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `EMIT-SURFACE-INTERACTION-GATE.1 — decision 0032: gate the nine surfaces in combination` (`7664761`) | Docs-only design leaf; delivers `docs/decisions/0032-emit-surface-interaction-gate.md` and opens `.4` |
| `.2` | `EMIT-SURFACE-INTERACTION-GATE.2 — structured-emission-max emits 8 surfaces, not 1` | Preset + anti-drift test + docs/book; default path byte-identical |

## Changelog

- `2026-07-30`: Created from a source-level observation that the nine emit
  projections' mutual-exclusion invariant has no end-to-end gate, and that
  `--profile structured-emission-max` covers 4 of 9 surfaces.
- `2026-07-30` (`.1`): design ADR landed as decision `0032`. The opening observation
  strengthened from measurement — the preset does not merely cover 4 of 9, it emits
  **1 of 9** — and a fourth leaf `.4` opened for a latent `cone_function` absorption
  trap found while reasoning the interaction risks through from source.
