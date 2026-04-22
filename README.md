# anvil
Single entry point for the project.

## Project objective
`anvil` is a constrained-random generator of **synthesizable
SystemVerilog RTL**. Today its implemented lane produces syntactically
valid, semantically correct, synthesizable, and structurally
non-trivial modules by building a typed circuit graph via fanin-cone
recursion and emitting SV from it.

The intended destination is stronger than "valid enough": `anvil`
should become a **signoff-level-quality random RTL generator** whose
outputs are boringly clean in mainstream downstream tools while still
being rich enough to break them. The product goal is **legal,
reproducible, adversarial RTL** that can expose real parser,
elaboration, synthesis, and lint bugs precisely because it stays inside
the accepted synthesizable envelope.

Whole-module intended functionality is not the target. By construction,
the recursive fanin-cone process mainly aims at legal structure and
tool-ingestible complexity; absent a specification, most generated
modules are expected to be functionally arbitrary or outright
gibberish, and that is acceptable.

The long-term scope is broader than one leaf-module format. The user
has now made that explicit: the current "leaf-module typed circuit
generator" is the starting point, not the end state. ANVIL is meant to
grow into the go-to tool for **multiple families of pseudo-random,
valid-by-construction, synthesizable HDL artifacts** — for example the
current DUT RTL lane, future oracle-backed micro-design corpora, and
future frontend/elaboration-oriented accept corpora with explicit
expected-facts manifests.

**Three load-bearing principles:**
1. **Recursion is the core algorithm.** The generator answers one question — *"what drives this signal?"* — and recurses. Every level of abstraction (gate, cone, module, hierarchy) is the same recursion with a richer choice set. Iteration is the exception; recursion is the default. Anything that can be expressed as a recursive descent over a typed circuit graph should be.
2. **Every emitted module is valid by construction.** No generate-then-filter. No post-hoc repair. If a generator output fails semantic validation or synthesis, that is a generator bug, not expected behavior.
3. **Every output is reproducible.** Byte-identical output for the same `(seed, knobs)` pair, across platforms, forever. Seeded ChaCha8; no `thread_rng`; no wall-clock entropy; no hash-map iteration order in output paths.

See `ROADMAP.md` for the phased scope of the current leaf RTL lane plus
the broader future artifact families.

## Fast ramp-up (recommended reading order)
1. `README.md` (this file): canonical entry point and project map.
2. `SESSION_BOOTSTRAP.md`: what a fresh session should read first to regain full context.
3. `USER_GUIDE.md`: live CLI, knobs, and downstream verification workflow.
4. `ROADMAP.md`: current priorities and phased milestones.
5. `CODEBASE_ANALYSIS.md`: live Rust-workspace analysis aligned to the roadmap and active code reality.
6. `DEVELOPMENT_NOTES.md`: engineering rationale behind design decisions.
7. `MEMORY.md`: compact, operational continuity/handoff snapshot with git hashes.
8. `CHANGES.md`: fully detailed description of completed changes.
9. `COMMIT.md`: canonical commit workflow.
10. `book/`: mdBook — a live doc of equal standing with the short-form files. Structured in five parts: *Using anvil* (Getting Started / Tutorial / Recipes), *How It Works* (Core Idea / Algorithm / IR), *Correctness Guarantees*, *Motif Catalogue*, *Reference*. The user-facing chapters lead; design chapters follow. Recovery requires reading it.

Only the documents above are status authority. The mdBook is explicitly part of this set — not reference material adjacent to it.

## Key project file paths
### Crate layout
- `Cargo.toml`
- `src/main.rs`            CLI entry point
- `src/lib.rs`              library root
- `src/config.rs`           knobs, CLI overlay, validation
- `src/ir/types.rs`         `Module`, `Node`, `GateOp`, `Flop`, `DepSet`
- `src/ir/validate.rs`      IR invariant checker (safety net)
- `src/gen/mod.rs`          `Generator` entry points
- `src/gen/cone.rs`         fanin-cone recursion
- `src/gen/module.rs`       leaf-module generator
- `src/gen/pool.rs`         `SignalPool` for terminal selection
- `src/emit/sv.rs`          IR → SystemVerilog pretty-printer

### Tests and examples
- `tests/pipeline.rs`       end-to-end: generate → validate → emit
- `examples/generate_one.rs` minimal library usage
- `src/bin/tool_matrix.rs`  curated Verilator/Yosys scenario-matrix harness

### Design docs (mdBook, live)
- `book/book.toml`
- `book/src/SUMMARY.md`
- `book/src/core-idea.md`           canonical statement of the algorithm
- `book/src/algorithm.md`           fanin-cone pseudocode and width rules
- `book/src/ir.md`                   circuit IR reference
- `book/src/by-construction.md`      generation-by-construction argument
- `book/src/synthesizability.md`     subset-enforcement discipline
- `book/src/non-triviality.md`       dep-set tracking, anti-collapse rules
- `book/src/sequential.md`           Phase 2 cone boundaries
- `book/src/sharing.md`              Phase 2 DAG sharing
- `book/src/hierarchy.md`            hierarchy and future composition layers
- `book/src/knobs.md`                knob taxonomy, reproducibility contract
- `book/src/architecture.md`         Rust module layout and testing strategy
- `book/src/non-goals.md`            explicit scope refusals
- `book/src/why-not-grammar.md`      IR vs annotated EBNF

## Build and validation commands
```bash
# Build
cargo build

# Core tests (IR validation + reproducibility)
cargo test

# Generate one module to stdout
cargo run -- --seed 42

# Generate 100 modules into a directory
cargo run -- --seed 42 --count 100 --out ./generated

# Library-usage example
cargo run --example generate_one

# Tool-clean matrix sweep
cargo run --bin tool_matrix -- --out ./tool-matrix

# Lint and formatting
cargo clippy --all-targets
cargo fmt --all

# mdBook (design docs)
mdbook build book
mdbook serve book
```

### Downstream smoke tests (optional, require external tools)
```bash
# Elaboration sanity check (requires Verilator)
verilator --lint-only generated/mod_42_0000.sv

# Synthesis sanity check (requires Yosys)
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; stat"
```

Both should succeed on every generated file. A failure is a generator bug; file with the seed and the effective knobs from `manifest.json`.

For a broader repo-owned sweep across construction strategies,
identity modes, factorization levels, and stress profiles:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix
```

That writes per-scenario generated corpora plus
`tool_matrix_report.json`, and exits non-zero if Verilator or Yosys
fails on any generated file or emits any warning. Current local smoke
status after the post-construction proof-cleanup slice: the built-in
matrix is 15/15 clean in Verilator and 15/15 clean in Yosys under the
current default `--yosys-mode without-abc`.

The harness now has an explicit Yosys mode axis too:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode without-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode with-abc
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both
```

`without-abc` remains the default because it is the current stable
baseline. `with-abc` now means the repo-owned warning-clean ABC path
(`synth -noabc; abc -fast; opt -fast; stat; check`) rather than the
raw default `synth` script, because the latter's ABC flow was tripping
non-actionable combinational-network warnings on valid generated
designs. A small repo-owned `--yosys-mode both` probe is now clean in
both sub-modes: `without-abc = 15/15 pass`, `with-abc = 15/15 pass`.
A completed current-code `--phase1-gate --yosys-mode both` report now
exists at `/tmp/anvil-tool-matrix-phase1-real-r21`. The final
`tool_matrix_report.json` records:

- `15` scenarios
- `67` modules per scenario
- `1005` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 1005/0`
- `Yosys without-abc pass/fail = 1005/0`
- `Yosys with-abc pass/fail = 1005/0`

The completed current-code Phase 2 sharing report now also exists at
`/tmp/anvil-tool-matrix-phase2-share-r1`. Its final
`tool_matrix_report.json` records:

- `18` scenarios
- `12` modules per scenario
- `216` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 216/0`
- `Yosys without-abc pass/fail = 216/0`
- `Yosys with-abc pass/fail = 216/0`
- normalized share sweep:
  - `share_prob = 0.0`: `shared_node_fraction = 0.4122`
  - `share_prob = 0.3`: `shared_node_fraction = 0.4232`
  - `share_prob = 0.9`: `shared_node_fraction = 0.4386`

`tool_matrix` writes per-module
checkpoint sidecars and supports `--resume`, so interrupted output trees
can be continued in place instead of always forking a fresh `--out`
directory.

For the repo-owned Phase 1 gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase1 --phase1-gate
```

That auto-enables coverage-gap failure and raises the per-scenario
module count high enough to generate at least 1000 modules total. To
continue an interrupted run on the same tree:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase1 --phase1-gate --resume
```

For the repo-owned Phase 2 sharing gate shape:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix-phase2-share --phase2-share-gate --yosys-mode both
```

That runs the representative `share_prob` sweep (`0.0`, `0.3`, `0.9`)
across 18 built-in sharing scenarios and records a normalized
`share_sweep` summary in the report so the knob can be proven against
the landed graph shape rather than only against generator-side rolls.

## Current CLI truth
- `anvil --seed N` generates a single module to stdout.
- `anvil --seed N --count M --out DIR` generates M modules into DIR with a `manifest.json`.
- `anvil --dump-config` prints the effective knobs as JSON.
- `anvil --identity-mode <node-id|relaxed>` is the coarse NodeId semantics switch; `node-id` selects the full-factorization doctrine (`NodeId` = expression identity), while `relaxed` is the intentional off-switch where equivalent expressions may keep different `NodeId`s.
- `anvil --factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>` is the current-build implementation/proof-depth dial inside `node-id`; lower rungs are weaker enforcement of the same doctrine, not a different meaning of `node-id`.
- `anvil --full-factorization` requests `--identity-mode node-id --factorization-level e-graph`; `anvil --no-full-factorization` requests `--identity-mode relaxed --factorization-level none`.
- `tool_matrix --yosys-mode <without-abc|with-abc|both>` controls
  whether the repo-owned Yosys harness runs the current `synth -noabc`
  path, the explicit ABC-enabled `abc -fast` path, or both.
- `tool_matrix --resume` reuses per-module checkpoints from an existing
  `--out` tree when the saved tool surface matches the current run. New
  same-binary checkpoints also carry a generator checkpoint, an `sv`
  hash, and a runtime fingerprint, so a rerun on the same binary can
  skip replaying already-proven modules while still checking file
  integrity. Older trees without that metadata fall back to the strict
  replay-and-validate path and are upgraded in place. Resume is
  intentionally byte-stable: if regenerated `.sv` no longer matches the
  saved artifact after a generator-semantics change, start from a fresh
  `--out` tree instead of forcing reuse across that boundary.
- `tool_matrix --phase2-share-gate` runs the repo-owned representative
  sharing sweep over `share_prob ∈ {0.0, 0.3, 0.9}` and fails on
  coverage gaps. Its report now includes a `share_sweep` summary with
  normalized `shared_node_fraction` because stronger sharing collapses
  total node count and therefore makes the raw shared-node count a bad
  control metric.
- Current scope: single-module combinational **and sequential**
  generation, DAG sharing default-on, bounded semantic `e-graph`
  fragment live under `--identity-mode node-id`, no hierarchy yet, and
  no artifact-family selector yet. Broader valid-by-construction
  synthesizable artifact families are roadmap work. See `ROADMAP.md`
  for phase gating.

## Maintenance rule
`README.md` is updated whenever project entry-point information changes materially (objective, ramp-up flow, key paths, or CLI surface). It does not need updates for every commit.

## License
Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

Read `SESSION_BOOTSTRAP.md` and start from there.
