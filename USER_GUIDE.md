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
bounded semantic gate-merge count (`semantic_gates_merged`),
endpoint-preserving flop-merge count (`flops_merged`),
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
- Raising `priority_encoder_prob` should raise `num_priority_encoder_blocks` monotonically.
- Raising `comb_mux_encoding_prob` should shift the `num_comb_muxes_encoded / (num_comb_muxes_one_hot + num_comb_muxes_encoded)` ratio toward the knob value.
- `nested_associative_operand_count` measures how many same-op nested
  operand slots are still flattenable under the current duplicate
  policy. At the default strict `operand_duplication_rate`, it should
  be 0 once the live Associative layer has done its work.
- Raising `flop_prob` should raise `num_flops` / `num_nodes`.
- `identity_mode=relaxed` → gate count and AST multiplicity jump because
  the NodeId-identity ladder is disabled entirely.
- Under `identity_mode=node-id`, the live bounded `e-graph` fragment can
  collapse small-support combinational cones too; `semantic_gates_merged`
  tells you how much post-construction semantic gate sharing it found.
- Under `identity_mode=node-id`, equivalent state cones can collapse too;
  `flops_merged` tells you how much sequential sharing the post-drain
  pass found.
- `factorization_level=none` (under `identity_mode=node-id`) → gate count
  grows; `=cse` and above shrinks it.

Live probability-roll counters are collected in
`knob_roll_attempts` / `knob_roll_fires`, so every `gen_bool`
site now has explicit attempt/fire telemetry. Anti-collapse
retries and terminal-tier picks are still primarily visible in
`--trace high`.

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
| `--construction-strategy` | interleaved | Strategy: `sequential` | `shuffled` | `interleaved` (default) | `graph-first` (deprecated alias) |
| `--graph-first-pool-size` | 32       | Legacy knob retained for backward-compatible configs; ignored by the current live path |
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
| `--terminal-reuse-prob` | 0.3      | Forced-leaf probability of reusing an exact-width pool signal |
| `--constant-prob`       | 0.1      | Forced-leaf probability of emitting a constant instead of a width-adapter fallback |
| `--gate-bitwise-weight` | 3        | Relative weight for bitwise gate selection      |
| `--gate-arith-weight`   | 2        | Relative weight for arithmetic ops              |
| `--gate-struct-weight`  | 1        | Relative weight for structured ops (mux, etc.)  |
| `--gate-compare-weight` | 1        | Relative weight for comparison ops at 1-bit targets |
| `--gate-reduce-weight`  | 1        | Relative weight for reduction ops at 1-bit targets |
| `--identity-mode`       | node-id  | Coarse NodeId semantics: `node-id` selects the full-factorization doctrine (`NodeId` = expression identity), `relaxed` intentionally disables it |
| `--factorization-level` | e-graph  | Current-build enforcement/proof ladder inside `node-id`: none → cse → operand-unique → commutative → associative → constant-fold → peephole → e-graph |
| `--full-factorization`  | off      | Convenience alias for `--identity-mode node-id --factorization-level e-graph` |
| `--no-full-factorization` | off    | Convenience alias for `--identity-mode relaxed --factorization-level none` |

The primary data-input draw happens before finalisation. Any data input
or high input bits that survive only as dead surface area are trimmed
before emission, so the emitted module interface matches the live logic
rather than the generator's provisional first draft.

Under `identity_mode=node-id` with effective factorization level
`>= cse`, finalisation also performs a conservative sequential-sharing
pass: if two flops end up with the same emitted state semantics over the
same canonical leaf endpoints, their Qs are unified before reachability
compaction. At effective level `e-graph`, finalisation also runs a
bounded semantic combinational-sharing pass that can merge
different-shape small-support cones proven equivalent over the same leaf
variables.

Interpretation note: doctrinally, `identity_mode=node-id` means
`NodeId` is the identity of an expression, which implies full
factorization by definition. `factorization_level` is the current
build's approximation/proof-depth dial inside that doctrine, plus a
useful stress/debug axis for matrix sweeps; it does not redefine what
`node-id` means. `relaxed` is the only intentional mode where equivalent
expressions may keep different `NodeId`s.

Treat the adversarial surface as orthogonal axes, not one blended
"randomness" knob: construction strategy (`sequential`, `shuffled`,
`interleaved`, `graph-first` alias), identity mode (`node-id` vs
`relaxed`), factorization level, motif/category weights, and the
probability knobs are independent controls. Efficient downstream stress
comes from exercising that matrix without hidden implementation bias.

## Output layout

```
generated/
├── manifest.json            # seed, knobs, per-module metadata
├── mod_42_0000.sv           # generated modules
├── mod_42_0001.sv
└── ...
```

## Tool matrix sweeps

For a broader repo-owned downstream sweep, use the dedicated matrix
harness:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix
```

To continue an interrupted sweep on the same output tree:

```bash
cargo run --bin tool_matrix -- --out ./tool-matrix --resume
```

What it does:

- builds a curated scenario matrix over construction strategy,
  identity mode, factorization level, and two stress profiles
  (share-heavy comb-only, motif-heavy sequential);
- generates a per-scenario corpus under `./tool-matrix/<scenario>/`;
- runs Verilator and Yosys on every generated file;
- writes `./tool-matrix/tool_matrix_report.json` with per-file tool
  results, aggregated metrics, and coverage facts; and
- exits non-zero if either downstream tool fails on any generated file.

Useful options:

- `--list-scenarios` to print the built-in matrix without running it.
- `--modules-per-scenario N` to trade runtime for more coverage.
- `--phase1-gate` to auto-enable coverage-gap failure and raise the
  run to at least 1000 generated modules total.
- `--phase2-share-gate` to run the repo-owned representative
  `share_prob` sweep (`0.0`, `0.3`, `0.9`) and fail when the sharing
  gate's coverage or normalized share summary is incomplete.
- `--yosys-mode <without-abc|with-abc|both>` to choose the current
  stable `synth -noabc` path, the explicit ABC-enabled
  `abc -fast` path, or both as separate sub-runs per generated file.
- `--fail-on-coverage-gap` to fail when the matrix misses one of the
  intended axes or motif/knob decision sites.
- `--skip-verilator` / `--skip-yosys` when you want to isolate one
  downstream consumer.

Current local smoke status after the full current-code Phase 1 closure:
the built-in matrix is 15/15 clean in Verilator and 15/15 clean in
Yosys under `--yosys-mode without-abc`. `tool_matrix` treats warnings
as failures, so a green run means "no errors, no warnings", not merely
zero non-zero exits. A small `--yosys-mode both` probe is clean in both
Yosys sub-modes too: `without-abc = 15/15 pass`, `with-abc = 15/15
pass`. The completed current-code `--phase1-gate --yosys-mode both`
report at `/tmp/anvil-tool-matrix-phase1-real-r21/tool_matrix_report.json`
records:

- `15` scenarios
- `67` modules per scenario
- `1005` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 1005/0`
- `Yosys without-abc pass/fail = 1005/0`
- `Yosys with-abc pass/fail = 1005/0`

The completed current-code Phase 2 sharing report at
`/tmp/anvil-tool-matrix-phase2-share-r1/tool_matrix_report.json`
records:

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

`tool_matrix` now writes per-module
checkpoint sidecars and supports `--resume`, so interrupted output trees
can be continued in place.

`--resume` only reuses saved tool results when the saved tool surface
matches the current run (`skip_verilator`, `skip_yosys`, and
`yosys_mode`). New same-binary checkpoints also carry a generator
checkpoint, an `sv` hash, and a runtime fingerprint, so a rerun on the
same binary can skip replaying already-proven modules while still
checking file integrity. Older trees without that metadata are upgraded
by the strict replay-and-validate path. Resume is intentionally
byte-stable: if regenerated `.sv` no longer matches the saved artifact
after a generator-semantics change, use a fresh output directory for
the new run.

## Downstream verification

`anvil` does not ship an oracle or simulator. To sanity-check output:

**Verilator elaboration:**
```bash
verilator --lint-only generated/mod_42_0000.sv
```

**Yosys synthesis:**
```bash
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; stat"
```

To probe the ABC-enabled path explicitly:

```bash
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; abc -fast; opt -fast; stat; check"
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
