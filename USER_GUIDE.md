# User Guide

## Installation

```bash
git clone <repo> anvil
cd anvil
cargo build --release
```

The binary lands at `target/release/anvil`.

## Basic usage

Generate a single module to stdout:

```bash
anvil --seed 42
```

Generate 100 modules into a directory:

```bash
anvil --seed 42 --count 100 --out ./generated
```

Each module lands in its own `.sv` file named by seed and index, e.g.
`generated/mod_42_0007.sv`. A `manifest.json` in the output directory
records the seed, knobs, and per-module summary (port counts, widths,
node count, flop count).

## Tracing and debugging

When diagnosing generator behavior — why a particular motif fired,
which retries happened, which pool entry was picked — enable the
built-in trace with `--trace <level>`:

| Level    | What you see                                                             |
|----------|--------------------------------------------------------------------------|
| `none`   | Silent (default). No overhead, no output. `off` accepted as alias.       |
| `low`    | Module start/done, strategy chosen, retry / fallback warnings.           |
| `medium` | Phase transitions inside each strategy, flop drain milestones.           |
| `high`   | Per-frame / per-cone events, motif dispatch, terminal-tier picks, anti-collapse rollbacks. |
| `debug`  | Strict super-set of `high`: every `pick_gate` return, every `intern_gate` / `intern_constant` create-or-reuse, with depth + width + node id. Use when you need to answer "who created this node?" |

Trace output goes to stderr (so stdout stays clean for generated SV)
or to a file with `--trace-file <path>`:

```bash
anvil --seed 42 --trace medium 2> trace.log
anvil --seed 42 --trace high --trace-file run.log
```

The trace format is deterministic: same `(seed, knobs)` produces the
same trace bytes. No timestamps, no thread IDs, no ANSI colors.
Emojis mark milestone / retry / fallback events (`🚀 start`,
`✅ done`, `🔁 retry`, `❌ exhausted`, `⚠️ fallback`, `✍️ emit`,
`🧱 block`, `🔧 operator`, `🍃 leaf`).

## Metrics

Every generated module is measurable. A post-hoc walk produces a
JSON metrics block covering size (nodes, gates, flops, constants),
per-kind gate distribution, constant width/value distribution,
mux shape (2-to-1 count, degenerate count), concat shape
(replication vs heterogeneous), sharing (num shared nodes, max
and average fanout), flop kind and mux-shape distribution,
AST-instance saturation (`max_gate_ast_multiplicity`,
`max_constant_ast_multiplicity` — relative to the
`max_ast_instances` cap), and operand-arity distribution
(`gate_operand_count_histogram`, `max_gate_operand_count`,
`max_operand_count_by_kind`), and combinational-depth distribution
(`max_gate_depth`, `gate_depth_histogram`).

```bash
# Dump metrics to stderr alongside the SV to stdout.
anvil --seed 42 --metrics 2> metrics.json

# Multi-module runs: metrics are always embedded in manifest.json.
anvil --seed 42 --count 100 --out ./generated
# → ./generated/manifest.json has metrics per module.
```

Typical use: sweep a knob over a few values, grep the metrics
block, verify the knob is producing the intended distribution
shift. Examples:

- `mux_arm_duplication_rate=0.0` → `num_muxes_degenerate` should be 0.
- `operand_duplication_rate=0.0` → gate operand lists have no internal duplicates.
- Raising `max_ast_instances` should raise `max_gate_ast_multiplicity`.
- Raising `max_gate_arity` should raise `max_operand_count_by_kind["add"]` exactly.
- Raising `max_depth` should raise `max_gate_depth` monotonically.
- Raising `flop_prob` should raise `num_flops` / `num_nodes`.
- `factorization_level=none` → gate count grows (no CSE); `=cse` and above
  shrinks it.

Live counters (probability rolls fired vs missed, anti-collapse
retries, terminal-tier picks) are not yet collected — the
`--trace high` output surfaces most of them on a per-event basis.

## Reproducibility

Every output is deterministic in `(seed, knobs)`. Running the same
command twice produces byte-identical files. To reproduce a specific
module, pass the exact seed reported in `manifest.json`.

## Knobs

Knobs control the shape and complexity of generated modules. Pass them
as CLI flags or via a JSON config file (`--config knobs.json`).

| Flag                    | Default  | Meaning                                         |
|-------------------------|----------|-------------------------------------------------|
| `--min-inputs`          | 2        | Minimum primary input count per module          |
| `--max-inputs`          | 8        | Maximum primary input count                     |
| `--min-outputs`         | 1        | Minimum primary output count                    |
| `--max-outputs`         | 4        | Maximum primary output count                    |
| `--min-width`           | 1        | Minimum port width in bits                      |
| `--max-width`           | 32       | Maximum port width                              |
| `--max-depth`           | 6        | Maximum cone recursion depth                    |
| `--flop-prob`           | 0.15     | Probability a cone node becomes a flop          |
| `--max-flops-per-module`| 32       | Hard cap on flops emitted per module            |
| `--min-mux-arms`        | 1        | Minimum M for the M-to-1 one-hot mux on flop D  |
| `--max-mux-arms`        | 4        | Maximum M for the M-to-1 one-hot mux on flop D  |
| `--flop-qfeedback-prob` | 0.5      | Probability of Q→D feedback when no select fires|
| `--flop-mux-encoding-prob` | 0.5   | Probability an encoded-select mux is used (vs one-hot)|
| `--min-gate-arity`      | 2        | Min arity N for associative operators (And/Or/Xor/Add/Mul)|
| `--max-gate-arity`      | 4        | Max arity N for associative operators                 |
| `--comb-mux-prob`       | 0.1      | Probability a non-leaf node becomes an M-to-1 comb mux|
| `--comb-mux-encoding-prob` | 0.5   | Per-mux probability of Encoded vs OneHot (comb muxes) |
| `--construction-strategy` | graph-first | Strategy: `sequential` | `shuffled` | `interleaved` | `graph-first` (default) |
| `--graph-first-pool-size` | 32       | Target top-level units for `graph-first` strategy    |
| `--coefficient-prob`    | 0.2      | Per-op probability of linear-combination compound motif (Add/Sub/Mul)|
| `--min-coefficient`     | 1        | Min coefficient (strictly positive)                   |
| `--max-coefficient`     | 15       | Max coefficient                                       |
| `--const-shift-amount-prob` | 0.8  | Per-shift probability the shift amount is a constant  |
| `--min-shift-amount`    | 0        | Min constant shift amount                             |
| `--max-shift-amount`    | 7        | Max constant shift amount (clamped to W-1)            |
| `--gate-shift-weight`   | 1        | Relative weight for the Shl/Shr bucket in pick_gate   |
| `--const-comparand-prob`| 0.3      | Per-comparison probability of a constant RHS (additive)|
| `--min-comparand`       | 0        | Min constant comparand value                          |
| `--max-comparand`       | 255      | Max constant comparand (clamped to 2^K - 1)           |
| `--priority-encoder-prob`| 0.05    | Per-emission probability of a priority-encoder block (N 1-bit reqs → log2(N)-bit index)|
| `--share-prob`          | 0.3      | Per-operand probability of reusing an existing wire (DAG-cone fraction)|
| `--hierarchy-depth`     | 0        | Max sub-module nesting (Phase 4)                |
| `--gate-bitwise-weight` | 3        | Relative weight for bitwise gate selection      |
| `--gate-arith-weight`   | 2        | Relative weight for arithmetic ops              |
| `--gate-struct-weight`  | 1        | Relative weight for structured ops (mux, etc.)  |

## Output layout

```
generated/
├── manifest.json            # seed, knobs, per-module metadata
├── mod_42_0000.sv           # generated modules
├── mod_42_0001.sv
└── ...
```

## Downstream verification

`anvil` does not ship an oracle or simulator. To sanity-check output:

**Verilator elaboration:**
```bash
verilator --lint-only generated/mod_42_0000.sv
```

**Yosys synthesis:**
```bash
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth; stat"
```

Both should succeed on every generated file. If one fails, that's a bug
in `anvil` — file an issue with the seed and knobs from `manifest.json`.

## Use as a library

```rust
use anvil::{Config, Generator};

let cfg = Config::default().with_seed(42);
let mut gen = Generator::new(cfg);
let module = gen.generate_module();
println!("{}", anvil::emit::to_sv(&module));
```

See `examples/` for more patterns.
