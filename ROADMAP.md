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

## Phase 1 — Single-module MVP (in progress)

One module, no hierarchy, no inter-module sharing. Combinational *and*
sequential logic from the start — flops are part of the same fanin-cone
recursion (Q is a leaf, D opens a new sub-cone, worklist drains).

- Random N inputs, M outputs, random widths per port.
- Per-output fanin cone recursion with depth budget.
- Gate set: bitwise (`and`, `or`, `xor`, `not`), arithmetic
  (`+`, `-`, `==`, `<`), `mux`, `slice`, `concat`, constants.
- **Sequential discipline:** single `clk` (posedge) and single `rst_n`
  (async, active-low) shared by every flop in the module. One
  `always_ff @(posedge clk or negedge rst_n)` block per module.
- Width propagates top-down; dependency set propagates bottom-up.
- Non-triviality: every output and every flop-D cone has dep-set ≥ 1,
  enforced at cone root.
- Tree-shaped cones only (each internal signal has one consumer).
- SV emitter produces `module` + `assign` + `always_ff`.

**Exit criteria:** 1000 modules generated from random seeds, all parse
and elaborate in Verilator without error, all Yosys-synthesize to
non-empty netlists, both with and without flops.

## Phase 2 — Signal sharing (DAG cones) (in progress)

- Signal pool of already-created internal wires.
- Per-operand `share_prob` decision: recurse (tree) or reuse (DAG).
  Mixing is the default — a single gate's operands can freely combine
  shared and freshly-built sub-cones.
- Dep-set propagation correctly handles shared fanout.
- Fanout stress: a single wire can drive many consumers.
- Anti-collapse rules still apply post-share (no `x ^ x` even when both
  operands come from pool reuse).

**Exit criteria:** generator produces cones with controlled sharing
factor; synthesis still succeeds; no multi-driver violations; Verilator
lint passes on a representative seed sweep with `share_prob` ∈ {0.0, 0.3, 0.9}.

## Phase 3 — Structured combinational ops

- `case`/`casez` expressions.
- Priority encoders, one-hot decoders.
- Reduction operators (`&`, `|`, `^` unary).
- Shift by variable amount.
- `for`-loop unrolled logic (statically bounded).

**Exit criteria:** motif library covers common synthesizable idioms.

## Phase 4 — Hierarchy

- Module instantiation: at any cone node, optionally emit a sub-module
  call instead of a gate.
- Two sourcing modes:
  - **Library**: pre-generate a pool, pick from it.
  - **On-demand**: generate a fresh sub-module with required port widths.
- Arbitrary hierarchy depth, bounded by knob.
- Name uniqueness across the module set.

**Exit criteria:** multi-file output directory with correct top module
declared; Verilator elaboration of hierarchy succeeds.

## Phase 5 — Parameterization

- Generated modules take `parameter` declarations for widths.
- Instantiation picks parameter values from allowed ranges.
- Parameter-dependent widths propagate correctly through cone generation.
- **Hard prerequisite:** Phase 4 (hierarchy). Parameters only matter
  at instantiation time.
- IR-level design recorded in `book/src/ir.md` "Future extensions /
  Parameters and generics".

## Phase 5b — Synthesizable aggregates (scheduled alongside Phase 5,
## order not fixed)

Three sub-paths, each with its own cost and payoff (full analysis in
`book/src/ir.md` "Future extensions / Synthesizable aggregates"):

- **Packed struct / union / array** — emitter-layer change only; IR
  stays flat. Low cost. Primary value: parser / elaboration coverage
  in downstream tools. Can land independently of Phase 4.
- **Unpacked arrays** — the memory-inference pattern. Covered by
  Phase 6 below.
- **Unpacked struct / union for datapath, enums** — deprioritised
  (unpacked datapath is mostly non-synthesizable; enums add no
  distinct stress value beyond typed constants).

## Phase 6 — Advanced motifs

- Memories (single-port, dual-port, with inferrable patterns only).
- FSMs with explicitly generated state encodings.
- Multi-clock with CDC-safe handshakes — optional, expensive. Until
  this lands, every module remains fully synchronous to a single clock.

## Non-goals

- Testbenches, assertions, coverage — `anvil` generates DUT code only.
- Non-synthesizable constructs (`initial`, delays, system tasks beyond
  `$display` in debug comments).
- Language coverage beyond the synthesizable SV subset.
- Oracle / reference simulator — `anvil` is a generator, not a tool tester.
  Downstream users are free to run Verilator/Yosys against the output.
