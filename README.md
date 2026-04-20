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
- `book/src/sharing.md`              Phase 3 DAG sharing
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
matrix is 15/15 clean in Verilator and 15/15 clean in Yosys.

## Current CLI truth
- `anvil --seed N` generates a single module to stdout.
- `anvil --seed N --count M --out DIR` generates M modules into DIR with a `manifest.json`.
- `anvil --dump-config` prints the effective knobs as JSON.
- `anvil --identity-mode <node-id|relaxed>` is the coarse NodeId semantics switch; `node-id` keeps the factorization ladder live, `relaxed` disables it.
- `anvil --full-factorization` requests `--identity-mode node-id --factorization-level e-graph`; `anvil --no-full-factorization` requests `--identity-mode relaxed --factorization-level none`.
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
