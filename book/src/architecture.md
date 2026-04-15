# Architecture of the Rust Implementation

## Crate layout

```
src/
├── main.rs          # CLI entry point (clap-derived); covers every
│                    # Phase 1/2 motif knob as a dedicated flag.
├── lib.rs           # public API, re-exports Config, Generator, Module.
├── config.rs        # Config struct + serde + CLI overlay + validation.
├── ir/
│   ├── mod.rs       # re-exports.
│   ├── types.rs     # Module, Port, Node, GateOp, Flop, FlopKind,
│   │                # FlopMux, MuxArm, DepSet, Design.
│   └── validate.rs  # invariant + per-gate shape checker; inline unit tests.
├── gen/
│   ├── mod.rs       # Generator struct, public entry points.
│   ├── cone.rs      # fanin-cone recursion (combinational + sequential);
│   │                # DAG-sharing fork; flop-mux assembly (one-hot,
│   │                # encoded); inline unit tests.
│   ├── module.rs    # leaf-module generator (clk/rst_n reservation,
│   │                # pool seeding, output cones, worklist drain).
│   └── pool.rs      # SignalPool (width-indexed, cloneable for rewind).
└── emit/
    ├── mod.rs       # re-exports.
    └── sv.rs        # IR -> SystemVerilog; inline unit tests.
```

Phase 4 (hierarchy) will add `src/gen/hierarchy.rs`; it does not exist
yet.

## Dependency direction

```
main  ->  lib  ->  gen  ->  ir
                    |        ^
                    v        |
                   emit -----+
```

- `ir` has zero dependencies on other modules.
- `gen` depends on `ir` (builds IR).
- `emit` depends on `ir` (reads IR).
- `gen` and `emit` do not depend on each other.
- `main` wires it all together.

This means `ir` can be tested in isolation, `emit` can be tested with
hand-constructed IRs (no need to invoke the generator), and `gen` can
be tested by inspecting the IR it produces without ever emitting SV.

## Key types at a glance

```rust
// ir/types.rs
pub struct Module { ... }
pub enum Node { PrimaryInput{..}, Constant{..}, FlopQ{..}, Gate{..} }
pub enum GateOp { And, Or, Xor, Not, Add, Sub, ..., Mux, Slice{..}, ... }
pub enum FlopKind { ZeroDefault, QFeedback }
pub enum FlopMux { None, OneHot(Vec<MuxArm>), Encoded { sel, data } }
pub struct Flop { ..., kind: FlopKind, mux: FlopMux }
pub struct DepSet(BTreeSet<u32>);

// gen/mod.rs
pub struct Generator { rng: ChaCha8Rng, cfg: Config, ... }
impl Generator {
    pub fn new(cfg: Config) -> Self;
    pub fn generate_module(&mut self) -> Module;
    pub fn generate_design(&mut self) -> Design;   // Phase 4+ stub
}

// emit/sv.rs
pub fn to_sv(m: &Module) -> String;
```

## Testing strategy

Three layers:

**Unit tests** live inline in each source module under
`#[cfg(test)] mod tests { ... }`. Current counts:

- `src/ir/validate.rs` — 8 tests (valid modules + each rejection
  class).
- `src/gen/cone.rs` — 7 tests (`ceil_log2`, `pick_mux_arm_count`,
  `make_width_adapter` edge cases, DAG-sharing sanity).
- `src/emit/sv.rs` — 6 tests (module header, clk/rst_n omission,
  `always_ff` shape, operator + constant rendering, Slice/Concat,
  Mux ternary).

**Integration tests** in `tests/pipeline.rs` — 2 tests: cross-seed
generation + IR validation across 20 seeds, and seed-reproducibility
byte-identical output check.

**Total: 23 tests, all passing.**

**External smoke tests** (not wired up yet) — will invoke Verilator
and Yosys against generated output. These are the remaining Phase 1
and Phase 2 exit gates.

## Error handling

`anvil` should not fail silently or on valid configurations. The
error taxonomy:

- `ConfigError` — invalid knobs (e.g., `min_width > max_width`,
  `min_mux_arms > max_mux_arms`, out-of-range probability). Caught
  at `Config::validate()` before any generation begins.
- `ValidateError` — IR invariant violation (per-gate arity, per-gate
  width, missing flop D, empty-dep-set output, etc.). Treated as a
  generator bug — if real generator output produces this, the
  generator is wrong.
- `IoError` — failed to write output file. Surfaced to the user.

The generator never produces invalid IR. If it does, that's a
generator bug, not a recoverable error.

## CLI

Every Phase 1/2 motif knob has a dedicated flag:

```
anvil [OPTIONS]

Options:
  --seed <SEED>                     RNG seed [default: 0]
  --count <N>                       Number of modules [default: 1]
  --out <DIR>                       Output directory (default: stdout)
  --config <FILE>                   Load knobs from JSON
  --dump-config                     Print effective knobs as JSON and exit

  --min-inputs <N>                  [default: 2]
  --max-inputs <N>                  [default: 8]
  --min-outputs <N>                 [default: 1]
  --max-outputs <N>                 [default: 4]
  --min-width <N>                   [default: 1]
  --max-width <N>                   [default: 32]
  --max-depth <N>                   [default: 6]

  --flop-prob <P>                   [default: 0.15]
  --max-flops-per-module <N>        [default: 32]
  --min-mux-arms <N>                [default: 1]   (effective floor is 2)
  --max-mux-arms <N>                [default: 4]
  --flop-qfeedback-prob <P>         [default: 0.5]
  --flop-mux-encoding-prob <P>      [default: 0.5]

  --share-prob <P>                  [default: 0.3]

  -h, --help                        Print help
  -V, --version                     Print version
```

Piping stdout is valid for `count = 1` (no directory required). For
`count > 1`, `--out` is required so that per-module files and the
manifest have somewhere to go.
