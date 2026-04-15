# Roadmap

`anvil` grows in phases. Each phase delivers a working generator with a
larger expressive subset. No phase should land without end-to-end tests
and at least one `.sv` artifact run through Yosys or Verilator as a
synthesizability smoke check.

## Phase 0 — Scaffolding (done)

- Cargo project, module skeleton, CLI entry point.
- Design docs (`book/`) capturing the core algorithm.
- `Module`, `Port`, `Net`, `Node`, `Gate`, `Flop` IR types defined.
- CLI accepts `--seed`, `--count`, `--out`, `--config`, `--dump-config`.

**Exit criteria (met locally):** `cargo build`, `cargo test`,
`cargo clippy -D warnings`, `cargo fmt --check` all clean. Reproducibility
test passes byte-identical output for the same seed.

## Phase 1 — Combinational leaf modules (in progress)

The MVP. One module, no hierarchy, no flops, no sharing.

- Random N inputs, M outputs, random widths per port.
- Per-output fanin cone recursion with depth budget.
- Gate set: bitwise (`and`, `or`, `xor`, `not`), arithmetic
  (`+`, `-`, `==`, `<`), `mux`, `slice`, `concat`, constants.
- Width propagates top-down; dependency set propagates bottom-up.
- Non-triviality: every output's dep-set ≥ 1 input, enforced at cone root.
- Tree-shaped cones only (each internal signal has one consumer).
- SV emitter producing a single `module` with `assign` statements.

**Exit criteria:** 1000 modules generated from random seeds, all parse and
elaborate in Verilator without error, all Yosys-synthesize to non-empty
netlists.

## Phase 2 — Sequential logic

- Flop node type with clock, sync/async reset, reset value.
- One clock, one reset per module.
- Flops introduce cone boundaries: Q is a leaf for the current cone,
  D opens a new cone (worklist-driven).
- Flop-Q reuse across outputs enables shared state.
- SV emitter grows `always_ff` blocks.

**Exit criteria:** generated modules include flops, pass same
synthesizability checks, and cycle-accurate behavior matches between
Verilator and Icarus on 100+ random input traces per module.

## Phase 3 — Signal sharing (DAG cones)

- Signal pool of already-created internal wires.
- Probability knob for "reuse existing signal" vs "recurse to create new."
- Dep-set propagation correctly handles shared fanout.
- Fanout stress: a single wire can drive many consumers.

**Exit criteria:** generator produces cones with controlled sharing
factor; synthesis still succeeds; no multi-driver violations.

## Phase 4 — Structured combinational ops

- `case`/`casez` expressions.
- Priority encoders, one-hot decoders.
- Reduction operators (`&`, `|`, `^` unary).
- Shift by variable amount.
- `for`-loop unrolled logic (statically bounded).

**Exit criteria:** motif library covers common synthesizable idioms.

## Phase 5 — Hierarchy

- Module instantiation: at any cone node, optionally emit a sub-module
  call instead of a gate.
- Two sourcing modes:
  - **Library**: pre-generate a pool, pick from it.
  - **On-demand**: generate a fresh sub-module with required port widths.
- Arbitrary hierarchy depth, bounded by knob.
- Name uniqueness across the module set.

**Exit criteria:** multi-file output directory with correct top module
declared; Verilator elaboration of hierarchy succeeds.

## Phase 6 — Parameterization

- Generated modules take `parameter` declarations for widths.
- Instantiation picks parameter values from allowed ranges.
- Parameter-dependent widths propagate correctly through cone generation.

## Phase 7 — Advanced motifs

- Memories (single-port, dual-port, with inferrable patterns only).
- FSMs with explicitly generated state encodings.
- Multi-clock (with CDC-safe handshakes) — optional, expensive.

## Non-goals

- Testbenches, assertions, coverage — `anvil` generates DUT code only.
- Non-synthesizable constructs (`initial`, delays, system tasks beyond
  `$display` in debug comments).
- Language coverage beyond the synthesizable SV subset.
- Oracle / reference simulator — `anvil` is a generator, not a tool tester.
  Downstream users are free to run Verilator/Yosys against the output.
