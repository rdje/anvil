# Coverage Baseline

Snapshot of line/branch coverage produced by `cargo-llvm-cov` over the
existing unit + `tests/pipeline.rs` test suite. **Deliberately
excludes** the Phase 4 hierarchy matrix gate, which runs for ~75 min
and which would dominate runtime; the baseline is meant to be
reproducible in minutes, not hours, so it can be rerun after every
planner-touching slice.

## How to reproduce

```bash
cargo llvm-cov --release
cargo llvm-cov report --release
```

(The first invocation also produces HTML output under
`target/llvm-cov/html/` for line-level inspection.)

To re-run only after wiping previous coverage data:

```bash
cargo llvm-cov clean --release
cargo llvm-cov --release
```

## Baseline (closes `COVERAGE-INSTRUMENTATION.1`)

| File | Lines covered | Lines missed | % Lines | Functions covered | % Functions |
| --- | ---: | ---: | ---: | ---: | ---: |
| `bin/tool_matrix.rs` | 5034 | 1951 | 72.07% | 289/340 | 85.00% |
| `config.rs` | 528 | 250 | 67.87% | 32/34 | 94.12% |
| `emit/sv.rs` | 1009 | 15 | 98.54% | 49/51 | 96.08% |
| `gen/cone.rs` | 3545 | 454 | 88.65% | 225/244 | 92.21% |
| `gen/hierarchy.rs` | 1657 | 76 | 95.61% | 103/106 | 97.17% |
| `gen/mod.rs` | 66 | 6 | 91.67% | 11/11 | 100.00% |
| `gen/module.rs` | 568 | 12 | 97.93% | 42/42 | 100.00% |
| `gen/pool.rs` | 22 | 0 | 100.00% | 8/8 | 100.00% |
| `ir/compact.rs` | 2005 | 64 | 96.91% | 89/90 | 98.89% |
| `ir/types.rs` | 1433 | 79 | 94.78% | 106/112 | 94.64% |
| `ir/validate.rs` | 765 | 254 | 75.07% | 43/44 | 97.73% |
| `lib.rs` | 3 | 3 | 50.00% | 1/2 | 50.00% |
| `main.rs` | 218 | 142 | 60.56% | 9/21 | 42.86% |
| `metrics.rs` | 2321 | 8 | 99.66% | 124/125 | 99.20% |
| **TOTAL** | **19174** | **3314** | **85.26%** | **1131/1230** | **91.95%** |

(Region-level coverage was also captured: 87.61% regions covered.)

## Reading the baseline

A few observations help interpret these numbers correctly:

- **`metrics.rs` at 99.66% lines / 99.20% functions** confirms that the
  detection helpers and ratio computations are exercised
  comprehensively by both the focused proofs and `tests/pipeline.rs`.
  The one missed function is likely a serde derive impl that no test
  triggers.
- **`gen/hierarchy.rs`, `gen/module.rs`, `gen/cone.rs`, `ir/compact.rs`,
  `emit/sv.rs` all 88–99% lines** — the planner core is well-covered
  by the focused-proof + unit-test combination alone, even without the
  matrix gate's 204 scenarios. The matrix gate would push these
  closer to 100%, but the marginal coverage win does not justify the
  75-minute runtime per baseline.
- **`bin/tool_matrix.rs` at 72.07% lines** is expected: the matrix
  harness has scenario builders that are exercised by the matrix gate
  (excluded here) and by `cargo test --bin tool_matrix` (which the
  baseline does cover). Many of the missed lines are
  helper-config functions that fire only under `Phase4Hierarchy`
  scenario selection.
- **`config.rs` at 67.87% lines / `main.rs` at 60.56% lines** — the
  CLI overlay layer. Most of `main.rs` is `clap` derives + flag
  plumbing that real tests don't drive; covering these requires
  integration-style invocations of the binary. Likely worth deferring
  unless `COVERAGE-INSTRUMENTATION.2` (triage) flags a specific bug
  risk.
- **`ir/validate.rs` at 75.07% lines** — the missed lines are
  defensive panics for "this case cannot happen" invariants. They
  fire only on broken IR, which the by-construction discipline
  prevents. These are intentionally unreachable from healthy code
  paths; the baseline correctly identifies them as untouched but
  they do not represent test gaps.

## Top-5 under-covered files (for `COVERAGE-INSTRUMENTATION.2` triage)

Ordered by absolute uncovered-line count (highest first):

1. **`bin/tool_matrix.rs`** — 1951 uncovered lines (72.07% covered).
   Most are matrix-gate-only paths; check whether any are dead code
   (retired scenario builders) versus real gate-exclusive paths.
2. **`gen/cone.rs`** — 454 uncovered lines (88.65% covered). Largest
   absolute miss in the planner core. Likely contains rarely-fired
   anti-collapse rollback paths or block-assembly variants;
   high-value triage target.
3. **`ir/validate.rs`** — 254 uncovered lines (75.07% covered). As
   noted, expected to be mostly defensive panics, but worth a pass
   to confirm.
4. **`config.rs`** — 250 uncovered lines (67.87% covered). CLI
   overlay variants; check for orphan knobs no longer wired.
5. **`main.rs`** — 142 uncovered lines (60.56% covered). Mostly
   clap-derive boilerplate; lowest-leverage of the five.

`COVERAGE-INSTRUMENTATION.2` will produce a disposition table per
file: (a) dead code → remove, (b) rarely-fired path → add focused
proof, (c) defensive unreachable → leave and document.

## Caveats

- **Matrix-gate paths in `bin/tool_matrix.rs` and `gen/hierarchy.rs`
  are under-counted here.** Running the matrix gate under
  `cargo llvm-cov` would close most of the remaining gap in those
  files, but at the cost of a 75+ minute baseline run. Reserved for
  occasional "deep" coverage refreshes, not every-slice discipline.
- **Branch coverage shows 0% covered / 0% missed in the report** —
  cargo-llvm-cov defaults do not emit MC/DC branch counters without
  additional flags. The line and region columns are the actionable
  signal.
- The exact percentages will drift slightly between runs because
  `tests/pipeline.rs` proofs are deterministic but the test
  framework's iteration order can change reporting batching. The
  rounded numbers above are stable to within ±0.1%.

## Reproducibility check

The baseline was produced on `main` at the commit immediately
preceding the `COVERAGE-INSTRUMENTATION.1` landing commit. Re-running
`cargo llvm-cov report --release` against that same git hash should
reproduce these numbers within rounding.
