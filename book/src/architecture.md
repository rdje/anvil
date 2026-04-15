# Architecture of the Rust Implementation

## Crate layout

```
src/
├── main.rs          # CLI entry point (clap-derived)
├── lib.rs           # public API, re-exports
├── config.rs        # Config struct + serde + CLI overlay
├── ir/
│   ├── mod.rs       # re-exports
│   ├── types.rs     # Module, Port, Node, Gate, Flop, DepSet
│   └── validate.rs  # invariant checker (safety net; never rejects in prod)
├── gen/
│   ├── mod.rs       # Generator struct, public entry points
│   ├── cone.rs      # fanin cone recursion
│   ├── module.rs    # leaf-module generator
│   ├── hierarchy.rs # hierarchical-module generator (Phase 5+)
│   └── pool.rs      # SignalPool
└── emit/
    ├── mod.rs       # re-exports
    └── sv.rs        # IR -> SystemVerilog text
```

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
pub enum Node { PrimaryInput{...}, Constant{...}, FlopQ{...}, Gate{...} }
pub enum GateOp { And, Or, Xor, Not, Add, Sub, ..., Mux, Slice{...}, ... }
pub struct Flop { ... }
pub struct DepSet(BitSet);

// gen/mod.rs
pub struct Generator { rng: ChaCha8Rng, cfg: Config }
impl Generator {
    pub fn new(cfg: Config) -> Self;
    pub fn generate_module(&mut self) -> Module;
    pub fn generate_design(&mut self) -> Design;   // Phase 5+
}

// emit/sv.rs
pub fn to_sv(m: &Module) -> String;
pub fn to_sv_design(d: &Design) -> Vec<(String, String)>; // (filename, content)
```

## Testing strategy

Three layers:

**Unit tests** in each module (`#[cfg(test)] mod tests { ... }`). Test
IR constructors enforce invariants. Test gate-width rules. Test
dep-set propagation. Test emitter on hand-built IRs.

**Integration tests** in `tests/`. Generate N modules across a seed
range. Assert all pass IR validation. Assert output is non-empty and
contains expected keywords (`module`, `endmodule`, `assign` or
`always_ff`). Assert reproducibility: same seed = byte-identical
output.

**External smoke tests** (gated by an env var, not required for
`cargo test`):

- `ANVIL_SMOKE_VERILATOR=1 cargo test --test smoke_verilator` — runs
  `verilator --lint-only` on each generated `.sv`.
- `ANVIL_SMOKE_YOSYS=1 cargo test --test smoke_yosys` — runs Yosys
  synthesis.

CI enables the external smoke tests. Developers without those tools
installed can still run `cargo test` and get meaningful coverage.

## Error handling

`anvil` should not fail silently or on valid configurations. The
error taxonomy:

- `ConfigError` — invalid knobs (e.g., `min_width > max_width`). Caught
  at `Config::validate()` before any generation begins.
- `GeneratorBug` — internal invariant violation. `panic!` with a
  message including the seed and the state that violated the invariant.
  These are bugs to fix, not user errors.
- `IoError` — failed to write output file. Surfaced to the user.

The generator never produces invalid IR. If it does, that's a
`GeneratorBug`, not a recoverable error.

## CLI

```
anvil [OPTIONS]

Options:
  --seed <SEED>              RNG seed [default: 0]
  --count <N>                Number of modules [default: 1]
  --out <DIR>                Output directory (default: stdout)
  --config <FILE>            Load knobs from JSON
  --dump-config              Print effective knobs as JSON and exit
  --min-inputs <N>           [default: 2]
  --max-inputs <N>           [default: 8]
  ...
  -h, --help                 Print help
  -V, --version              Print version
```

Piping stdout is valid for count=1 (no directory required). For
count>1, `--out` is required so that per-module files and the
manifest have somewhere to go.
