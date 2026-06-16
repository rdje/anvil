# User Guide

## Installation

```bash
git clone <repo> anvil
cd anvil
cargo build --release
```

The binary lands at `target/release/anvil`.

When running from the source tree, Cargo's default run target is
`anvil`, so `cargo run -- ...` invokes the generator. The repo-owned
matrix harness is selected explicitly with
`cargo run --bin tool_matrix -- ...`.

## Basic usage

Generate a single module to stdout:

```bash
anvil --seed 42
```

Generate one real depth-1 hierarchical design to stdout:

```bash
anvil --seed 42 --hierarchy-depth 1 --num-leaf-modules 3
```

Generate one depth-1 hierarchical design that reuses a 2-definition
leaf library across 5 child instances:

```bash
anvil --seed 42 --hierarchy-depth 1 --num-leaf-modules 2 --num-child-instances 5
```

Generate one bounded recursive hierarchy tree whose realized depth is
picked inside `[2:3]` and whose non-leaf modules each instantiate
between 2 and 4 children:

```bash
anvil --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 3 --min-child-instances-per-module 2 --max-child-instances-per-module 4
```

Generate one bounded recursive hierarchy tree with a per-depth
branching profile layered on top of the fallback range:

```bash
anvil --seed 42 --min-hierarchy-depth 2 --max-hierarchy-depth 2 --min-child-instances-per-module 1 --max-child-instances-per-module 3 --child-instances-per-depth 0=4:4 --child-instances-per-depth 1=2:2
```

Generate 100 modules into a directory:

```bash
anvil --seed 42 --count 100 --out ./generated
```

Generate a source-level frontend/elaboration accept artifact with its
expected-facts manifest:

```bash
anvil --artifact frontend --seed 0 --lane-n-params 4 --lane-n-children 2 --out ./frontend-artifact
```

That writes `./frontend-artifact/acc_0.sv` and
`./frontend-artifact/acc_0.json`. The `.sv` keeps package constants,
top parameters, chained localparams, instance parameter bindings, and
the generate predicate symbolic; the `.json` records the resolved facts
that the parity gates compare against downstream tool output. Omitting
`--out` prints the `.sv` to stdout and the manifest to stderr so stdout
pipelines stay clean.

Generate an oracle-backed const-expression micro-design artifact:

```bash
anvil --artifact microdesign --seed 7 --lane-n-params 5 --out ./microdesign-artifact
```

Each module lands in its own `.sv` file named by seed and index, e.g.
`generated/mod_42_0007.sv`. A `manifest.json` in the output directory
records the seed, knobs, and per-module summary (port counts, widths,
node count, flop count).

When hierarchy is enabled (`--hierarchy-depth 1`), each generated
design still writes one `.sv` file per module, but `manifest.json`
switches to a `designs` array. Each design entry records the `top`
module name plus the module/file list for that design. The current
wrapper slice also separates leaf-library size
(`--num-leaf-modules`) from instantiated child count
(`--num-child-instances`): `0` preserves the legacy exact-once wrapper
behavior, smaller values under-instantiate the library, and larger
values reuse child definitions.

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
deterministic FSM block merge count (`fsms_merged`),
design-level canonical and bounded-semantic module signatures
(`canonical_module_signatures`, `semantic_module_signatures`),
structural and semantic module duplicate-pair counts
(`num_structurally_duplicate_module_pairs`,
`num_semantically_duplicate_module_pairs`),
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

# Hierarchy mode also embeds per-design composition metrics.
anvil --seed 41 --count 1 --out ./generated-hier --hierarchy-depth 1 --num-leaf-modules 3 --num-child-instances 5
# → ./generated-hier/manifest.json has both per-module metrics and
#   per-design hierarchy metrics.
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
- Raising `case_mux_prob` should raise `num_case_mux_blocks` monotonically.
- Raising `casez_mux_prob` should raise `num_casez_mux_blocks` monotonically.
- Raising `for_fold_prob` should raise `num_for_fold_blocks` monotonically.
- Raising `comb_mux_encoding_prob` should shift the `num_comb_muxes_encoded / (num_comb_muxes_one_hot + num_comb_muxes_encoded)` ratio toward the knob value.
- `nested_associative_operand_count` measures how many same-op nested
  operand slots are still flattenable under the current duplicate
  policy. At the default strict `operand_duplication_rate`, it should
  be 0 once the live Associative layer has done its work.
- Raising `flop_prob` should raise `num_flops` / `num_nodes`.
- `identity_mode=relaxed` → gate count and AST multiplicity jump because
  the NodeId-identity ladder is disabled entirely.
- Under `identity_mode=node-id`, the live bounded `e-graph` fragment can
  collapse small-support combinational cones too, including gates proven
  equal to an existing endpoint or constant after helper endpoints cancel
  out. The current truth-table proof can enumerate up to 12 endpoint-
  support bits, but only while the cone also fits the node/work budget;
  larger candidates fall back to structural identity.
  `semantic_gates_merged` tells you how much post-construction semantic
  gate sharing it found.
- Under `identity_mode=node-id`, equivalent state cones can collapse too;
  `flops_merged` tells you how much sequential sharing the post-drain
  pass found.
- With `bisimulation_flop_merge=true` in a JSON config or library
  `Config` (opt-in, `default=false`), ANVIL additionally merges flops
  proven sequentially equivalent *up to a state correspondence* — for
  example two mutually-recursive registers (`D_f=Q_g`, `D_g=Q_f`, equal
  reset) — via a bounded greatest-fixpoint bisimulation, a class the
  exact flop merge provably cannot prove. Active only under
  `identity_mode=node-id` with effective `factorization_level=e-graph`;
  resetless flops are excluded (no reset base case); over-budget cones
  fall back to structural identity. `bisimulation_flops_merged` records
  the extra sharing. There is no CLI flag (mirrors
  `hierarchy_module_dedup`); set it through a `Config` value or config
  file.
- Under `identity_mode=node-id`, duplicate deterministic FSM blocks can
  collapse when their selector proof, encoding, transition table, and
  Moore-output table match; `fsms_merged` records that sharing.
- With `hierarchy_semantic_module_dedup=true` in a JSON config or
  library `Config`, ANVIL can also collapse non-top pure-combinational
  modules whose bounded whole-module truth tables match. Supported
  classes are instance-free modules and pure-combinational wrappers
  with at most 8 child instances whose children are also inside the
  proof boundary. The pass is active only under `identity_mode=node-id`
  with effective `factorization_level=e-graph`; `identity_mode=relaxed`
  keeps it off. The proof is deliberately bounded: matching `(PortId,
  width)` interfaces, <= 12 emitted input-support bits, <= 128
  reachable output cone nodes, no flops, memories, FSMs, parameters, or
  aggregates, and <= 128-bit outputs. Leaf and wrapper proofs stay
  separate, and ancestor/descendant wrapper groups are skipped.
  `num_semantically_duplicate_module_pairs` tells you how many
  proof-equal module pairs remain.
- With `hierarchy_sequential_module_dedup=true` in a JSON config or
  library `Config` (opt-in, `default=false`, no CLI flag — like the two
  dedup knobs above), ANVIL extends that module merge to **stateful
  flops-only leaf modules**: it proves two of them observationally
  (sequentially) equivalent by a bounded *cross-module* bisimulation
  (the flop-level greatest-fixpoint refinement lifted across both
  modules' state, primary inputs unified by `(PortId, width)`) plus
  output-cone equality under the resulting quotient — the sequential
  generalization of the combinational module dedup above. Active only
  under `identity_mode=node-id` with effective
  `factorization_level=e-graph`. First cut excludes modules with
  memories, FSMs, child instances, parameters, aggregates, multiple
  clock domains, or any resetless flop; over-budget candidates fall back
  to no-merge. `num_sequentially_duplicate_module_pairs` (and the
  per-module `sequential_module_proof_signatures`) tell you how many
  sequentially-equivalent stateful-leaf pairs remain.
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
| `--case-mux-prob`      | 0.05     | Per-emission probability of a combinational case block; dynamic selectors emit `always_comb case`, constant selectors lower to `assign` |
| `--casez-mux-prob`     | 0.05     | Per-emission probability of a combinational casez block; dynamic selectors emit `always_comb casez`, constant selectors lower to `assign` |
| `--for-fold-prob`      | 0.05     | Per-emission probability of a bounded combinational fold; dynamic sources emit `always_comb for`, constant sources lower to `assign` |
| `--hierarchy-depth`    | 0        | Legacy exact hierarchy depth. `1` = depth-1 wrapper slice |
| `--num-leaf-modules`   | 0        | Number of leaf modules pre-generated for the legacy exact depth-1 wrapper slice |
| `--num-child-instances`| 0        | Child-instance count for the legacy exact depth-1 wrapper slice. `0` preserves exact-once instantiation of every generated leaf |
| `--min-hierarchy-depth` | 0       | Minimum depth for bounded recursive hierarchy mode |
| `--max-hierarchy-depth` | 0       | Maximum depth for bounded recursive hierarchy mode |
| `--min-child-instances-per-module` | 0 | Minimum child-instance count for each non-leaf module in bounded recursive hierarchy mode |
| `--max-child-instances-per-module` | 0 | Maximum child-instance count for each non-leaf module in bounded recursive hierarchy mode |
| `--child-instances-per-depth DEPTH=MIN:MAX` | none | Optional per-parent-depth child-instance override layered on top of the bounded recursive fallback range |
| `--share-prob`          | 0.3      | Per-operand probability of reusing an existing wire (DAG-cone fraction)|
| `--terminal-reuse-prob` | 0.3      | Forced-leaf probability of reusing an exact-width pool signal |
| `--constant-prob`       | 0.1      | Forced-leaf probability of emitting a constant instead of a width-adapter fallback |
| `--gate-bitwise-weight` | 3        | Relative weight for bitwise gate selection      |
| `--gate-arith-weight`   | 2        | Relative weight for arithmetic ops              |
| `--gate-struct-weight`  | 1        | Relative weight for structured ops (mux, selectable `Slice` / `Concat`, etc.)  |
| `--gate-compare-weight` | 1        | Relative weight for comparison ops at 1-bit targets |
| `--gate-reduce-weight`  | 1        | Relative weight for reduction ops at 1-bit targets |
| `--identity-mode`       | node-id  | Coarse NodeId semantics: `node-id` selects the full-factorization doctrine (`NodeId` = expression identity), `relaxed` intentionally disables it |
| `--factorization-level` | e-graph  | Current-build enforcement/proof ladder inside `node-id`: none → cse → operand-unique → commutative → associative → constant-fold → peephole → e-graph |
| `--full-factorization`  | off      | Convenience alias for `--identity-mode node-id --factorization-level e-graph` |
| `--no-full-factorization` | off    | Convenience alias for `--identity-mode relaxed --factorization-level none` |
| `--sv-version`          | 2012     | Target IEEE 1800 standard (`2012` / `2017` / `2023`). Default `2012` is the honest floor — the current default emitted subset is 1800-2012-valid, so the default (and, with every up-opt knob off, all three targets) reproduce current output byte-for-byte. A **down-gating guarantee**: the emitter never emits a construct newer than the target. Surfaced in `--dump-config` / `--introspect` (schema `1.10`). The first **up-opt** now ships — see `soft_union_slice_prob` (a config-file knob). |

The `--sv-version 2023` target unlocks the first version-distinctive
**up-opt**, a config-file knob (no CLI flag, like `aggregate_prob`):

- `soft_union_slice_prob` (default `0.0`) — per *proper low-bits* slice
  (`a[hi:0]` over a wider source), the probability the emitter renders it
  through an internal IEEE 1800-2023 `union soft` overlay
  (`u.w = src; gate = u.n`) instead of a plain bit-select. It fires only
  when **both** `soft_union_slice_prob > 0.0` **and** `--sv-version 2023`;
  below 2023 a marked slice **down-gates** to the plain `a[hi:0]`.
  Behaviour-preserving (packed-union members are LSB-aligned) and
  genuinely 2023 (heterogeneous-width packed-union members are legal only
  as `union soft`, §7.3.1). `default = 0.0` is byte-identical. Verilator
  accepts it under `--language 1800-2023`; Yosys/Icarus reject the syntax
  and are a recorded no-op. Set it in a `--config` JSON, e.g.
  `{ "seed": 1, "soft_union_slice_prob": 1.0, "sv_version": "2023", … }`.

The richer-structured **emission** surfaces have their own config-file
knobs (no CLI flag, like `soft_union_slice_prob`):

- `function_emit_prob` (default `0.0`) — the probability, per
  *qualifying* combinational `Gate`, that anvil re-renders it as a
  `function automatic` of its direct operands instead of an inline
  `assign` (decision `0012`, the **first** richer-structured emission
  surface). It is an **emit-time projection** of an already-valid cone —
  the call evaluates to exactly the inline expression, so it is
  behaviour-preserving and adds no new IR truth (the
  `soft_union_slice_prob` / `aggregate_prob` precedent). Selection is
  rules-first at construction time (no generate-then-filter). The first
  cut wraps a single gate over its direct operands; structured selectors
  (`case` / `casez` / `for`-fold) and `Slice` are excluded and still
  emit inline (a full-width `Slice` parameter would trip `-Wall
  UNUSEDSIGNAL`; nothing is retired). Combinational only — a flop `Q` is
  a leaf parameter, never recursed through. `default = 0.0` is
  byte-identical; the emitted-function count is surfaced as
  `num_emitted_combinational_functions` in `--introspect` (schema
  `1.8`). Set it in a `--config` JSON, e.g.
  `{ "seed": 42, "function_emit_prob": 1.0, "flop_prob": 0.0, … }`. The
  surface is proven downstream-clean by `tool_matrix --function-emit-gate`
  (see the matrix section below). Full walk-through:
  `book/src/structured-emission.md`.
- `generate_loop_emit_prob` (default `0.0`) — the probability, per
  *qualifying* `{N{x}}` replication, that anvil re-renders it as a
  single-level `generate for` loop instead of an inline `assign`
  (decision `0013`, the **second** richer-structured emission surface).
  It is an **emit-time projection** of an already-valid replication — the
  unrolled loop is exactly `{N{x}}`, so it is behaviour-preserving and
  adds no new IR truth (the `function_emit_prob` precedent). A replication
  qualifies when it is a `Concat` of `N ≥ 2` operands that are all the
  **same** signal **and** the replicated lane is **1 bit** wide (⇒ result
  width `N`, so `<wire>[gi] = x` is bit-faithful); a **wider lane** would
  need a part-select body and stays inline (a recorded follow-up; nothing
  retired). Mutually exclusive with `function_emit_prob` on a gate; the
  loop increment is the maximally-portable `gi = gi + 1`. Combinational
  only. `default = 0.0` is byte-identical; the emitted-loop count is
  surfaced as `num_emitted_generate_loops` in `--introspect` (schema
  `1.9`). Set it in a `--config` JSON, e.g.
  `{ "seed": 12, "generate_loop_emit_prob": 1.0, "flop_prob": 0.0, … }`.
  The surface is proven downstream-clean by
  `tool_matrix --generate-loop-gate` (see the matrix section below). Full
  walk-through: `book/src/structured-emission.md`.

- `task_emit_prob` (default `0.0`) — the probability, per *qualifying*
  combinational gate (the **same candidate set as `function_emit_prob`**),
  that anvil re-renders it as a combinational `task automatic` called from
  `always_comb` instead of an inline `assign` (decision `0014`, the
  **third** richer-structured emission surface). It is the
  `function_emit_prob` single-gate projection expressed as a *procedural*
  `task` (an `output` arg written from `always_comb` into a `<wire>__tv`
  var, with a passthrough `assign` driving the gate's net) rather than a
  value-returning `function` — an **emit-time projection** of an
  already-valid gate, so it is behaviour-preserving and adds no new IR
  truth. Structured selectors and `Slice` are excluded (the
  `function_emit_prob` reasons, nothing retired); the four emit-projections
  are mutually exclusive on a gate (the task pass runs last). Combinational
  only. `default = 0.0` is byte-identical; the emitted-task count is
  surfaced as `num_emitted_combinational_tasks` in `--introspect` (schema
  `1.10`). Set it in a `--config` JSON, e.g.
  `{ "seed": 1, "task_emit_prob": 1.0, "flop_prob": 0.0, … }`. The surface
  is proven downstream-clean by `tool_matrix --task-emit-gate` (see the
  matrix section below). Full walk-through:
  `book/src/structured-emission.md`.

The primary data-input draw happens before finalisation. Any data input
or high input bits that survive only as dead surface area are trimmed
before emission, so the emitted module interface matches the live logic
rather than the generator's provisional first draft.

Under `identity_mode=node-id` with effective factorization level
`>= cse`, finalisation also performs a conservative sequential-sharing
pass: if two flops end up with the same emitted state semantics over the
same canonical leaf endpoints, their Qs are unified before reachability
compaction. In the current generated flow this pass runs before opt-in
multi-clock promotion; synchronizer flops added by that promotion are
not re-merged by the pass. The proof key includes the flop clock-domain
tag, so cross-domain state is not unified, and broader coinductive state
equivalence is not claimed beyond exact reset-defined self-hold
(`D == own Q` on both flops with the same width/reset/domain). At
effective level `e-graph`, finalisation also runs a
bounded semantic combinational-sharing pass that can merge
different-shape small-support cones proven equivalent over the same leaf
variables. If the proof shows a helper variable does not affect the
function, that helper endpoint is minimized out; for example,
`a & (b | ~b)` rewires to `a`, while the same shape rooted on `c`
rewires to `c`. The proof is bounded, not a solver: it enumerates at
most 12 endpoint-support bits and skips candidates whose
`assignment_count * cone_node_count` exceeds the current work budget.

Multi-clock CDC is config/library-only. Set `multi_clock_prob > 0.0`
to promote eligible modules to K=2, and set
`cdc_synchronizer_stages = N` to choose the number of destination-domain
flops in the generated 1-bit synchronizer chain. The default `N = 2`
is the original 2-flop synchronizer; `N >= 3` exercises the N-flop
primitive. Inspect `num_cdc_synchronizer_chains` and
`max_cdc_synchronizer_stages` in module metrics; the legacy
`num_cdc_2_flop_synchronizers` counts exact 2-stage chains.

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

Hierarchy mode (`--hierarchy-depth 1`) keeps the same file layout but
changes the manifest shape:

```text
generated-hier/
├── manifest.json            # seed, knobs, per-design metadata
├── mod_42_0000.sv           # leaf module
├── mod_42_0001.sv           # leaf module
├── mod_42_0002.sv           # top wrapper
└── ...
```

In that mode, `manifest.json` contains `designs: [...]`, and each
design entry records:
- `index`
- `top`
- `hierarchy`
- `metrics`
- `modules: [{ file, name, metrics }, ...]`

The per-design `metrics` block is the intended trust surface for the
current hierarchy slice. It lets you judge wrapper quality without
opening the emitted `.sv`, including:

- library size vs instantiated child count
- unique-instantiated-module count and unused-library count
- reuse ratio / library-coverage ratio
- top interface shape (`top_inputs`, `top_data_inputs`,
  `top_clock_inputs`, `top_reset_inputs`, `top_outputs`)
- direct-pass-through vs parent-composed top outputs
  (`top_direct_instance_output_drives`,
  `top_parent_composed_outputs`)
- whether top outputs actually depend on child outputs
  (`top_outputs_reaching_instance_outputs`,
  `top_outputs_without_instance_outputs`,
  `top_instance_output_dependency_fraction`)
- average / maximum child-output support per top output
  (`avg_instance_output_support_per_top_output`,
  `max_instance_output_support_per_top_output`)
- parent-cone helper-instance support for parent outputs
  (`top_outputs_reaching_parent_cone_instances`,
  `hierarchy_outputs_reaching_parent_cone_instances`,
  `top_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`,
  `top_parent_cone_instance_output_fraction`,
  `hierarchy_parent_cone_instance_output_fraction`,
  `top_parent_cone_instance_flop_output_fraction`,
  `hierarchy_parent_cone_instance_flop_output_fraction`)
- parent-cone helper-instance counts and budget realization
  (`top_parent_cone_instances`, `hierarchy_parent_cone_instances`,
  `max_parent_cone_instances_per_internal_module`)
- child-input provenance
  (`child_input_bindings_from_parent_ports`,
  `child_input_bindings_from_instance_outputs`,
  `child_input_bindings_from_mixed_support`,
  `child_input_bindings_from_constants`,
  `child_input_bindings_from_parent_composed_logic`,
  `child_input_bindings_from_parent_flops`,
  `child_input_bindings_from_registered_instance_outputs`,
  `child_input_bindings_from_registered_parent_composed_logic`,
  `child_input_bindings_from_registered_mixed_support`,
  `child_input_bindings_from_registered_multistage_parent_composed_logic`,
  `child_input_bindings_from_registered_multistage_mixed_support`,
  `child_input_bindings_from_registered_multistage_instance_outputs`,
  `child_input_bindings_from_registered_multistage_parent_cone_instances`,
  `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
  `child_input_bindings_from_registered_parent_cone_instances`,
  `top_child_input_bindings_from_registered_parent_cone_instances`,
  `child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
  `child_input_bindings_from_parent_cone_instance_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_mixed_support`,
  `child_input_bindings_from_parent_cone_instances_through_parent_flops`,
  `child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `top_child_input_bindings_from_parent_cone_instance_flop_mixed_support`,
  `child_input_bindings_from_parent_cone_instances`)
- hierarchy- and top-level sibling-routing fractions
  (`instance_output_child_input_binding_fraction`,
  `top_instance_output_child_input_binding_fraction`)
- hierarchy- and top-level parent-composed child-input fractions
  (`parent_composed_child_input_binding_fraction`,
  `top_parent_composed_child_input_binding_fraction`)
- hierarchy- and top-level parent-cone helper child-input fractions
  (`parent_cone_instance_child_input_binding_fraction`,
  `top_parent_cone_instance_child_input_binding_fraction`)
- hierarchy- and top-level parent-cone helper mixed-support child-input
  fractions
  (`parent_cone_instance_mixed_support_child_input_binding_fraction`,
  `top_parent_cone_instance_mixed_support_child_input_binding_fraction`)
- hierarchy- and top-level parent-composed helper-through-state
  child-input fractions
  (`parent_cone_instance_flop_child_input_binding_fraction`,
  `top_parent_cone_instance_flop_child_input_binding_fraction`)
- hierarchy- and top-level parent-composed helper-through-state
  mixed-support child-input fractions
  (`parent_cone_instance_flop_mixed_support_child_input_binding_fraction`,
  `top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction`)
- hierarchy- and top-level parent-flop child-input fractions
  (`parent_flop_child_input_binding_fraction`,
  `top_parent_flop_child_input_binding_fraction`)
- hierarchy- and top-level registered sibling-route fractions
  (`registered_instance_output_child_input_binding_fraction`,
  `top_registered_instance_output_child_input_binding_fraction`)
- hierarchy- and top-level multi-stage registered sibling-route
  fractions
  (`registered_multistage_instance_output_child_input_binding_fraction`,
  `top_registered_multistage_instance_output_child_input_binding_fraction`)
- hierarchy- and top-level multi-stage registered mixed-support route
  fractions
  (`registered_multistage_mixed_support_child_input_binding_fraction`,
  `top_registered_multistage_mixed_support_child_input_binding_fraction`)
- hierarchy- and top-level multi-stage registered helper-sourced route
  fractions
  (`registered_multistage_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`)
- hierarchy- and top-level multi-stage registered parent-composed
  helper-sourced route fractions
  (`registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`)
- hierarchy- and top-level registered parent-composed route fractions
  (`registered_parent_composed_child_input_binding_fraction`,
  `top_registered_parent_composed_child_input_binding_fraction`)
- hierarchy- and top-level registered helper-sourced route fractions
  (`registered_parent_cone_instance_child_input_binding_fraction`,
  `top_registered_parent_cone_instance_child_input_binding_fraction`)
- hierarchy- and top-level registered helper mixed-support route
  fractions
  (`registered_parent_cone_instance_mixed_support_child_input_binding_fraction`,
  `top_registered_parent_cone_instance_mixed_support_child_input_binding_fraction`)
- hierarchy- and top-level parent-output helper mixed-support counts
  and fractions
  (`hierarchy_outputs_reaching_parent_cone_instance_mixed_support`,
  `top_outputs_reaching_parent_cone_instance_mixed_support`,
  `hierarchy_parent_cone_instance_mixed_support_output_fraction`,
  `top_parent_cone_instance_mixed_support_output_fraction`)
- local parent-state counts
  (`hierarchy_parent_local_flops`,
  `internal_module_occurrences_with_local_flops`,
  `top_local_flops`)
- control fanout to child instances
- weighted child interface / node / flop load
- per-definition instantiation histogram

The current Phase 4 slice now has two planning lanes:
- **legacy exact wrapper lane:** `hierarchy_depth = 1`,
  `num_leaf_modules`, `num_child_instances`
- **bounded recursive lane:** `min_hierarchy_depth..=max_hierarchy_depth`
  plus `min_child_instances_per_module..=max_child_instances_per_module`
  with optional repeated `child_instances_per_depth` overrides keyed by
  parent depth (`0` = top, `1` = its direct children, ...)
- both hierarchy lanes now also expose
  `hierarchy_child_source_mode = library | on-demand`
- `library` keeps the reusable child-definition pool live
- the current `on-demand` slice synthesizes children against
  parent-planned exact data-interface profiles, one profiled child
  definition per planned instance slot
- `hierarchy_sibling_route_prob` controls whether later child data
  inputs may bind from earlier sibling instance outputs; when
  `hierarchy_parent_cone_instance_prob` also fires, this direct
  unregistered route can allocate a helper child and bind from its
  output. The route stays combinational
- `hierarchy_registered_sibling_route_prob` controls whether later
  child data inputs bind through local parent flops; default `0.0`
  keeps this registered child-to-child axis opt-in. The first route
  uses an earlier sibling output as the D source; later routes may also
  use earlier parent-local Qs as D sources, creating multi-stage
  registered child-to-child chains without parent-composed logic. When
  `hierarchy_parent_cone_instance_prob`
  also fires, the direct registered route can use a helper instance
  output as the parent-flop D source
- `hierarchy_registered_child_input_cone_prob` controls whether later
  child data inputs bind through parent-local combinational logic over
  sibling-output-derived sources and then one local parent flop;
  when `hierarchy_parent_cone_instance_prob` also fires, the registered
  D cone can include a parent-cone helper output; default `0.0` keeps
  this registered parent-composed route opt-in
- `hierarchy_child_input_cone_prob` controls whether child data inputs
  bind through parent-local combinational cones over already-available
  parent sources: parent data inputs, earlier sibling instance outputs,
  and earlier parent-side route gates. When
  `hierarchy_parent_cone_instance_prob` and `hierarchy_parent_flop_prob`
  both fire, the helper output can first be registered into
  parent-local state and then consumed by the parent-composed
  child-input logic. When the helper-backed unregistered cone would
  otherwise lack parent-port support, the generator can add a
  parent-port companion so the same child-input binding proves helper
  and parent-port mixed support, including when the helper support is
  consumed through a parent-local helper Q
- `hierarchy_parent_cone_instance_prob` controls whether
  parent-composed child-input cones, direct sibling routes, direct
  registered sibling routes, registered child-input D cones, or
  parent-output cones may instantiate helper children as internal
  parent-cone sources. Helper outputs may feed child inputs directly,
  feed registered D cones, feed parent outputs, or feed
  parent-composed child-input logic through parent-local helper Qs;
  helper-backed parent-composed child-input bindings can also mix in
  parent-port support without becoming registered routes, including
  when the helper support came through a parent-local helper Q;
  default `0.0` keeps this
  helper-instantiation axis opt-in
- `max_parent_cone_instances_per_module` controls how many helper
  children one hierarchy parent may instantiate; default `1` preserves
  the first helper slice, `0` disables helper allocation even when the
  probability fires, and raised budgets now apply directly to
  parent-output-only helper composition too
- `hierarchy_parent_flop_prob` controls whether parent-side hierarchy
  cones may emit local parent flops; default `0.0` keeps the hierarchy
  parent layer combinational unless this state axis is explicitly
  enabled. When helper placement is active, parent-output helper
  sources may route through those parent-local flops before reaching a
  parent output, and parent-composed child-input helper sources may
  route through those parent-local flops before reaching a later child
  input
- the legacy exact wrapper knobs and bounded recursive range knobs are
  intentionally mutually exclusive planning lanes
- pure comb-only modules do **not** expose `clk` / `rst_n`
- sequential leaves do expose `clk` / `rst_n`
- hierarchy parents keep `clk` / `rst_n` visible iff they carry local
  state or sequential descendants through instantiated children
- top outputs can now be real parent-side cones over child instance
  outputs, combinational by default and optionally stateful when
  `hierarchy_parent_flop_prob` is enabled
- unused child outputs are emitted as explicit unconnected ports
  (`.port()`) rather than fake pass-through wires
- in bounded recursive mode, ANVIL now keeps every leaf depth inside
  the requested `[min:max]` interval and can mix shallow and deep
  branches inside one tree when the interval is open and the structure
  allows it
- `leaf_module_occurrences_by_depth` is now the direct trust metric for
  that mixed-depth shape
- non-leaf modules still pick one child count inside the requested
  child-instance interval, with per-parent-depth overrides taking
  priority where specified
- local parent flops in the composed parent layer are live when
  `hierarchy_parent_flop_prob` is non-zero

## Resource-safe runs on RAM-limited hosts

Large `cargo` builds/tests and `tool_matrix` sweeps can spike memory. On
a RAM-limited host, wrap any heavy job in the repo's watchdog so it is
aborted *before* the danger zone instead of risking an out-of-memory
reboot:

```bash
# Abort the wrapped command if used RAM reaches the threshold (default 88%).
scripts/ram_guard.sh -- cargo test
scripts/ram_guard.sh --threshold 85 -- cargo run --bin tool_matrix -- --out ./tool-matrix
```

The guard exits `99` if it had to stop the job, otherwise it propagates
the wrapped command's own status. Complementary tactics: cap build
parallelism (`cargo test -j 2 -- --test-threads=2`), prefer focused test
targets over the whole suite, and chunk big sweeps with a smaller
`--modules-per-scenario` plus `--resume` (the matrix writes per-item
checkpoints). See `docs/decisions/0003-resource-safe-validation.md`.

### `anvil`'s own internal memory governor (opt-in)

`scripts/ram_guard.sh` guards *external* jobs from the outside. For
`anvil`'s **own** bulk-generation runs there is an opt-in governor that
guards the process from the inside, so a very large `--count` or a
single pathological `(seed, knobs)` cannot drive the host toward the
reboot zone faster than an external poll can react. Two ceilings, both
**off by default** (sentinel `0`) — when off, output is byte-identical:

```bash
# Stop the run once this process's resident set reaches 8 GiB.
anvil --seed 42 --count 100000 --out ./gen --max-rss-mb 8192

# Stop once host used RAM reaches 90% (macOS memory_pressure /
# Linux /proc/meminfo, same reads as ram_guard.sh).
anvil --seed 42 --count 100000 --out ./gen --ram-abort-pct 90
```

The governor is sampled **between** generated modules/designs in the
`--out` loop, never mid-construction: it declines to start more work
rather than mutilating a half-built module (which would emit invalid
RTL). On a trip it stops cleanly with exit code **99** (matching
`ram_guard.sh`) and a stderr message naming the abort reason, the
seed, and the full effective knobs, so the aborted run is reproducible.
Output written before the abort is left in place (the trailing
`manifest.json` array may be unterminated — expected for an aborted
run). This complements the always-on per-module construction budget
`max_nodes_per_module` (config-only; bounds one module's node arena):
the node budget caps a *single* module's size, the governor caps the
*process*.

## Tool matrix sweeps

For a broader repo-owned downstream-validation sweep, use the dedicated
matrix harness:

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
- runs Verilator and Yosys as validation tools on every generated
  artifact, with an opt-in Icarus compile/elaboration column;
- writes `./tool-matrix/tool_matrix_report.json` with per-artifact tool
  results, aggregated metrics, and coverage facts; and
- exits non-zero if either validation tool fails on any generated
  artifact.

Useful options:

- `--list-scenarios` to print the built-in matrix without running it.
- `--modules-per-scenario N` to trade runtime for more coverage.
- `--phase1-gate` to auto-enable coverage-gap failure and raise the
  run to at least 1000 generated modules total.
- `--phase2-share-gate` to run the repo-owned representative
  `share_prob` sweep (`0.0`, `0.3`, `0.9`) and fail when the sharing
  gate's coverage or normalized share summary is incomplete.
- `--phase3-structured-gate` to run the repo-owned structured-surface
  closure matrix and fail unless the report proves the landed Phase 3
  surfaces (`case`, `casez`, `for`-fold, priority encoder, mux
  encodings, selectable `Slice` / `Concat`, variable shift).
- `--phase4-hierarchy-gate` to run the repo-owned hierarchy matrix and
  fail unless the report proves multifile hierarchy designs, correct
  top-module tool invocation, real child instances, real
  instance-output nodes, representative wrapper profiles
  (`num_leaf_modules ∈ {2, 4}`, exact / reuse / under-instantiation),
  representative recursive profiles (depth `2`, child-instance ranges
  `[2:3]` and `[1:3]`), the per-depth override profile
  `0=4:4,1=2:2`, explicit child-sourcing modes
  `library` and `on-demand`, real sibling-routed and registered
  sibling-routed child-input bindings, multi-stage registered sibling
  routing at the top parent and below it without helpers, real
  registered parent-composed child-input bindings, helper-sourced
  child-input bindings, helper-sourced parent outputs, registered
  parent-composed helper-sourced child-input D cones, direct sibling
  helper routes, direct registered sibling helper routes, budgeted
  helper allocation, stateful helper child-input mixed-support routing,
  and real parent-side
  composition above instance outputs.
- `--signoff-knob-sweep-gate` to run the repo-owned focused
  richer-knob-sweep matrix (`SIGNOFF-AUTOMATION-EXPANSION.2b`) and fail
  unless the report proves the four previously-unswept generator knobs
  fire **by construction**: `operand_duplication_rate` (an `Add`/`Mul`
  gate with a duplicated operand), `mux_arm_duplication_rate` (a
  degenerate `(sel)?(x):(x)` mux), `aggregate_array_prob` (a
  packed-**array** aggregate module), and the memory×fsm interplay (a
  memory module and an FSM module in one design). One focused scenario
  per knob across all three construction strategies; all four must stay
  downstream-clean (Verilator + both Yosys modes). Closes ROADMAP
  steering gap 3's hidden-bias hole for these knobs.
- `--sv-version-gate` to run the repo-owned per-version acceptance
  matrix (`SV-VERSION-TARGETING.2b.2b`) and fail unless every targeted
  IEEE 1800 standard's corpus is **accepted in the matching tool
  standard mode**. It sweeps the three targets (2012 / 2017 / 2023) over
  a focused corpus — a combinational e-graph leaf, a sequential motif
  leaf, and a recursive depth-2 hierarchy design per version — and runs
  Verilator in the matching `--language 1800-20xx` standard mode (via
  the `.2b.2a` selector) plus Yosys `-sv`. The report requires
  `saw_sv_version_2012_targeted_acceptance`,
  `saw_sv_version_2017_targeted_acceptance`,
  `saw_sv_version_2023_targeted_acceptance`, and the umbrella
  `saw_sv_version_targeted_acceptance`. Those nine common-floor scenarios
  emit byte-identical SV across the three targets, so their value is the
  per-version downstream acceptance axis, not output divergence. The gate
  also runs a tenth **up-opt scenario** (`SV-VERSION-TARGETING.3b.2b`): a
  slice-heavy 2023-targeted leaf with `soft_union_slice_prob = 1.0` that
  genuinely emits the IEEE 1800-2023 `union soft` overlay and is accepted
  by Verilator `--language 1800-2023`. Yosys/Icarus reject the `union
  soft` syntax, so that scenario runs **Verilator-only** (Yosys/Icarus
  recorded no-op) and the report requires the dedicated
  `saw_sv_version_2023_soft_union_upopt` fact. Other gates own
  construction-strategy / motif breadth, so this gate uses the
  `Interleaved` strategy only. Banked clean at
  `/tmp/anvil-sv-version-gate-upopt-r1` (10 scenarios / 20 units /
  `coverage_gaps = []` / Verilator 20/0 / Yosys 18/0 both modes).
- `--function-emit-gate` to run the repo-owned combinational `function
  automatic` emit gate (`STRUCTURED-EMISSION-EXPANSION.2b.2b`) and fail
  unless the report proves the first richer-structured emission surface
  (decision `0012`) fires **by construction** and is downstream-accepted.
  It forces `function_emit_prob = 1.0` over a comb-only single-module DUT
  across all three construction strategies, so every qualifying
  combinational gate is projected to a behaviour-preserving `function
  automatic` over its direct operands, and requires the
  `saw_combinational_function_emit` fact (a genuinely-emitted function,
  detected from the emitted SV text, accepted by Verilator **and** Yosys).
  Unlike the `union soft` up-opt, a synthesizable function is accepted by
  every tool, so the gate runs the full Verilator + both Yosys modes (+
  Icarus when `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-function-emit-gate-r1` (3 scenarios / 12 modules / 608
  emitted functions / `coverage_gaps = []` / Verilator 12/0 / Yosys 12/0
  both modes / Icarus compile 12/0).
- `--generate-loop-gate` to run the repo-owned `generate for` loop emit
  gate (`STRUCTURED-EMISSION-EXPANSION.4b.2b`) and fail unless the report
  proves the second richer-structured emission surface (decision `0013`)
  fires **by construction** and is downstream-accepted. It forces
  `generate_loop_emit_prob = 1.0` over a comb-only single-module DUT across
  all three construction strategies, so every qualifying `{N{x}}`
  1-bit-lane replication (the common one-hot `{W{sel}}` mux-mask idiom) is
  projected to a behaviour-preserving single-level `generate for` loop, and
  requires the `saw_generate_loop_emit` fact (a genuinely-emitted loop,
  detected from the emitted SV text, accepted by Verilator **and** Yosys).
  Like a function (and unlike the `union soft` up-opt), a `generate for` is
  universally synthesizable, so the gate runs the full Verilator + both
  Yosys modes (+ Icarus when `--iverilog-compile` is set) plan. Banked clean
  at `/tmp/anvil-generate-loop-gate-r1` (3 scenarios / 12 modules / 8
  emitting a loop / `coverage_gaps = []` / Verilator 12/0 / Yosys 12/0 both
  modes / Icarus compile 12/0).
- `--task-emit-gate` to run the repo-owned combinational `task automatic`
  emit gate (`STRUCTURED-EMISSION-EXPANSION.6b.2b`) and fail unless the report
  proves the third richer-structured emission surface (decision `0014`) fires
  **by construction** and is downstream-accepted. It forces
  `task_emit_prob = 1.0` over a comb-only single-module DUT across all three
  construction strategies, so every qualifying combinational gate is projected
  to a behaviour-preserving `task automatic` over its direct operands (called
  from `always_comb` into a `<wire>__tv` output var), and requires the
  `saw_combinational_task_emit` fact (a genuinely-emitted task, detected from
  the emitted SV text, accepted by Verilator **and** Yosys). Like a function
  (and unlike the `union soft` up-opt), a combinational `task` is universally
  synthesizable, so the gate runs the full Verilator + both Yosys modes (+
  Icarus when `--iverilog-compile` is set) plan. Banked clean at
  `/tmp/anvil-task-emit-gate-r1` (3 scenarios / 12 modules / 12 emitting a
  task / `coverage_gaps = []` / Verilator 12/0 / Yosys 12/0 both modes /
  Icarus compile 12/0).
- `--yosys-mode <without-abc|with-abc|both>` to choose the current
  stable `synth -noabc` path, the explicit ABC-enabled
  `abc -fast` path, or both as separate sub-runs per generated file.
- `--iverilog-compile` to add an Icarus Verilog compile/elaboration
  column. The harness shells `iverilog -g2012` for each emitted module
  or design and records the result under `iverilog_compile` in the
  report. This is warning-clean acceptance evidence only; it does not
  run a testbench or compare traces.
- `--fail-on-coverage-gap` to fail when the matrix misses one of the
  intended axes or motif/knob decision sites.
- `--skip-verilator` / `--skip-yosys` when you want to isolate one
  validation tool.
- `--diff-sim` to add an opt-in **cross-simulator semantic-agreement**
  column (`DIFFERENTIAL-SIMULATION`,
  `docs/tasks/DIFFERENTIAL-SIMULATION.md`). The matrix's existing
  Verilator+Yosys columns prove every generated artifact is *accepted*
  (parses + synthesises); this column proves it is *semantically
  equivalent* across two independent simulators (iverilog 13.0 +
  verilator 5.046). A per-axis subset selector picks the first
  scenario per major axis (combinational / sequential-flop /
  hierarchy / memory / fsm), capped at K=5, deterministic; the
  chosen names land in the report under `diff_sim_subset` and are
  persisted to `<out>/.diff-sim-subset` for `--resume`. The harness
  shells `iverilog -g2012 + vvp` and `verilator --binary`, normalizes
  the fixed-width-hex traces, byte-compares, and records each
  module's outcome under `diff_sim`
  (`ran`/`success`/`n_samples`/`skip_reason`/`mismatch_excerpt` —
  the excerpt is a retained counterexample per the Phase-7
  doctrine, never a silent pass). The
  `saw_design_with_cross_simulator_agreement` coverage fact fires
  when at least one DUT in the subset achieves byte-equal post-reset
  traces. The column is a **friendly no-op when either simulator is
  absent** (`tools_present()` probe — the matrix still exits clean;
  use `--fail-on-coverage-gap` together with `--diff-sim` if you
  want the fact gated). It runs only AFTER Verilator and Yosys are
  both clean on the module — no point asking simulators to agree on
  output a parse/synth tool already rejected. Trade-off rationale:
  the diff-sim column is opt-in (not a gate-elevation flag) because
  the full matrix is computationally infeasible to gate
  mandatorily; the per-axis subset gives signoff-quality coverage
  without 2h+ CI runtime.

Current focused smoke status after `SIGNOFF-SURFACE-EXPANSION.3`: the
built-in matrix is clean across Verilator, both repo-owned Yosys modes,
and the opt-in Icarus compile column:
`Verilator 17/0`, `Yosys without-abc 17/0`, `Yosys with-abc 17/0`,
`Icarus compile 17/0`. `tool_matrix` treats warnings as failures, so a
green run means "no errors, no warnings", not merely zero non-zero
exits. Focused evidence:
`/tmp/anvil-signoff-surface-iverilog-r1/tool_matrix_report.json`.

The completed current-code
`--phase1-gate --yosys-mode both`
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

The completed current-code Phase 3 structured-surface report at
`/tmp/anvil-tool-matrix-phase3-structured-r4/tool_matrix_report.json`
records:

- `21` scenarios
- `10` modules per scenario
- `210` total modules
- `coverage_gaps = []`
- `Verilator pass/fail = 210/0`
- `Yosys without-abc pass/fail = 210/0`
- `Yosys with-abc pass/fail = 210/0`

The latest full downstream-clean Phase 4 hierarchy report at
`/tmp/anvil-tool-matrix-phase4-hierarchy-r87/tool_matrix_report.json`
records:

- `210` scenarios
- `4` designs per scenario
- `840` total designs
- `artifact_kind = "design"`
- `coverage_gaps = []`
- `Verilator pass/fail = 840/0`
- `Yosys without-abc pass/fail = 840/0`
- `Yosys with-abc pass/fail = 840/0`
- `saw_recursive_multiple_parent_cone_instances_per_parent = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs = true`
- `saw_recursive_multiple_parent_cone_instances_per_parent_through_flops = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`
- `saw_recursive_hierarchy_direct_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_direct_registered_sibling_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_multistage_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_composed_parent_cone_instance_routing = true`
- `saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_multistage_routing = true`
- `saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_multistage_sibling_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_routing = true`
- `saw_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_recursive_hierarchy_parent_cone_instance_flop_mixed_support_outputs = true`
- `saw_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`
- `saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
- `saw_hierarchy_registered_sibling_mixed_support_routing = true`
- `saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`
- `saw_hierarchy_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_3_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_3_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_3_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_3_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_3_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_4_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_4_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_4_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_4_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_4_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_5_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_5_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_5_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_5_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_5_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_6_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_6_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_6_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_6_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_6_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_7_parent_local_flops = true`
- `saw_recursive_hierarchy_depth_7_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_depth_7_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_7_stateful_parent_port_composed_outputs = true`
- `saw_recursive_hierarchy_depth_7_stateful_parent_composed_mixed_support_child_inputs = true`
- `saw_recursive_hierarchy_three_stage_registered_parent_composed_chain = true`
- `saw_recursive_parent_cone_helper_budget_5 = true`
- `saw_recursive_hierarchy_canonical_module_signature_diversity = true`
- `saw_design_with_structurally_duplicate_modules = true`
- `saw_recursive_hierarchy_module_dedup_active = true`

That report is the latest fully banked repo-owned Phase 4
artifact, not only the older wrapper baseline. It covers the broadened
`--num-child-instances` planner, bounded recursive depth `2`,
child-instance profiles `2`, `4`, `2:3`, and `1:3`, the mixed
recursive depth-range profile `2:3`, the per-depth override profile
`0=4:4,1=2:2`, the explicit hierarchy child-sourcing modes `library`
and `on-demand`, exact profiled child-interface synthesis in the
on-demand lane, real mixed shallow/deep leaf realization, real
parent-side composition above instance outputs, real sibling-routed
hierarchy child inputs, parent-composed child-input bindings, explicit
parent-local flop state, registered sibling-routed child-input
bindings, direct registered sibling mixed-support child-input bindings,
recursive non-top direct registered sibling mixed-support child-input
bindings, registered parent-composed child-input bindings, registered
mixed-support child-input bindings, recursive non-top registered
mixed-support child-input bindings, multi-stage registered
parent-composed child-input bindings, recursive non-top multi-stage
registered parent-composed child-input bindings without helper
instances, multi-stage registered sibling-routed child-input bindings,
recursive non-top multi-stage registered sibling-routed child-input
bindings without helper instances,
recursive non-top multi-stage registered mixed-support child-input
bindings without helper instances,
mixed parent-port / child-output
parent outputs, parent-cone helper-instance child-input bindings,
parent-output helper-instance composition, recursive non-top
parent-output helper-instance composition, recursive non-top
parent-output helper mixed-support composition, budgeted helper allocation,
recursive non-top parent-output multi-helper budget evidence,
recursive non-top child-input multi-helper budget evidence,
recursive non-top stateful multi-helper budget evidence,
stateful parent-output helper routing through parent-local flops,
recursive non-top stateful parent-output helper routing through
parent-local flops,
stateful parent-composed helper child-input routing through
parent-local flops,
stateful parent-composed helper child-input mixed-support routing
through parent-local flops,
registered parent-composed helper-sourced child-input D cones,
recursive non-top registered parent-composed helper-sourced child-input
D cones with mixed parent-port support,
multi-stage direct registered sibling helper routing, multi-stage
registered parent-composed helper routing, recursive non-top
multi-stage direct registered sibling helper routing, recursive non-top
multi-stage registered parent-composed helper routing, and
generator-global module-name allocation. It also now banks the direct
sibling helper route, direct registered sibling helper route, direct
registered sibling mixed-support route, recursive non-top direct
registered sibling mixed-support route, and the
recursive exact-depth-2 axes that prove a non-top parent can route
direct sibling child inputs from helper instance outputs and can route
direct registered sibling child-input D paths from helper instance
outputs, can route registered parent-composed child-input D paths from
helper instance outputs, can route parent-composed child inputs from
helper instance outputs through parent-local flops, can mix that
stateful unregistered helper child-input support with parent data-port
support, can source parent
outputs from helper instance outputs below the top parent, can mix those
non-top parent-output helper sources with parent data-port support, can route
parent-output helper sources through parent-local flops below the top
parent, and can chain direct
registered sibling helper routes through helper-sourced parent-local Qs
below the top parent, and can chain registered parent-composed helper
routes through helper-sourced parent-local Qs below the top parent, and
can chain registered parent-composed child-input routes through earlier
parent-local Qs below the top parent without helper instances, and
can chain direct registered sibling-routed child-input routes through
earlier parent-local Qs below the top parent without helper instances, and
can combine mixed parent-port / child-output registered D support with
earlier parent-local Q reuse below the top parent without helper
instances, and
can spend a recursive non-top child-input multi-helper budget, and can
spend a recursive non-top stateful multi-helper parent-output budget
through helper-sourced parent-local Qs. It also carries the accumulated
mixed-support facts for stateful helper-backed parent outputs,
unregistered helper child-input routing, stateful helper-through-flop
child-input routing, direct registered sibling mixed-support routing,
and recursive non-top direct registered sibling mixed-support routing, recursive non-top unregistered parent-composed mixed-support child-input routing without helper instances, recursive non-top parent outputs that mix parent data ports with child outputs without helper instances or parent-local state, and recursive non-top parent outputs that mix parent data ports, child outputs, and parent-local Qs without helper instances. The earlier
coverage-only proofs at
`/tmp/anvil-tool-matrix-phase4-recursive-direct-helper-r32/tool_matrix_report.json`
and
`/tmp/anvil-tool-matrix-phase4-recursive-helper-state-r31/tool_matrix_report.json`
are superseded by the full downstream-clean `r87` bank.

The older `r21` full bank remains useful historical evidence for the
pre-parent-output-helper surface. The clean pre-fix `r22` run is kept as
root-cause evidence only: a stale total-design budget produced
`42` scenarios at `3` designs/scenario (`126` total). The live Phase 4
gate now enforces a direct `4` designs/scenario floor. The focused clean
proofs at `/tmp/anvil-hier-reuse-smoke-r1`,
`/tmp/anvil-hier-under-smoke-r2`,
`/tmp/anvil-hier-parent-compose-smoke-r1/manifest.json`,
`/tmp/anvil-hier-range-smoke-r1/manifest.json`,
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, and
`/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json` still remain
useful targeted evidence. `/tmp/anvil-hier-child-input-cone-smoke-r1/manifest.json`
is the focused proof for parent-composed child-input bindings
(`child_input_bindings_from_parent_composed_logic = 13`,
`parent_composed_child_input_binding_fraction = 0.9285714285714286`).
`/tmp/anvil-hier-parent-state-smoke-r1/manifest.json` is the focused
proof for local parent state
(`hierarchy_parent_local_flops = 8`, `top_local_flops = 8`,
`top_clock_inputs = 1`, `top_reset_inputs = 1`, and
`child_input_bindings_from_parent_flops = 1`).
`/tmp/anvil-hier-registered-sibling-smoke-r1/manifest.json` is the
focused proof for registered sibling-routed child-input bindings
(`child_input_bindings_from_registered_instance_outputs = 4`,
`registered_instance_output_child_input_binding_fraction = 0.8`,
`hierarchy_parent_local_flops = 3`, `top_clock_inputs = 1`, and
`top_reset_inputs = 1`).
`/tmp/anvil-hier-registered-child-input-cone-smoke-r2/manifest.json`
is the focused proof for registered parent-composed child-input
bindings
(`child_input_bindings_from_registered_parent_composed_logic = 3`,
`registered_parent_composed_child_input_binding_fraction = 0.75`,
`hierarchy_parent_local_flops = 3`, `top_clock_inputs = 1`, and
`top_reset_inputs = 1`).
`/tmp/anvil-hier-registered-mixed-child-input-smoke-r1/manifest.json`
is the focused proof for registered mixed-support child-input bindings
(`child_input_bindings_from_registered_mixed_support = 3`,
`registered_mixed_support_child_input_binding_fraction = 0.75`).
`cargo test recursive_hierarchy_registered_mixed_support_routes_below_top`
is the focused proof for recursive non-top registered mixed-support
child-input bindings without helper instances
(`child_input_bindings_from_registered_mixed_support >
top_child_input_bindings_from_registered_mixed_support` and
`child_input_bindings_from_registered_parent_cone_instances = 0`).
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_without_helpers_below_top`
is the focused proof for recursive non-top multi-stage registered
parent-composed child-input bindings without helper instances
(`child_input_bindings_from_registered_multistage_parent_composed_logic >
top_child_input_bindings_from_registered_multistage_parent_composed_logic`
and all registered helper-chain counters remain zero).
`/tmp/anvil-hier-registered-multistage-child-input-smoke-r1/manifest.json`
is the focused proof for multi-stage registered parent-composed
child-input bindings
(`child_input_bindings_from_registered_multistage_parent_composed_logic = 2`,
`registered_multistage_parent_composed_child_input_binding_fraction = 0.5`).
`/tmp/anvil-hier-parent-output-mix-smoke-r1/manifest.json` is the
focused proof for mixed parent-port / child-output parent outputs
(`top_parent_port_composed_outputs = 8`,
`hierarchy_parent_port_composed_outputs = 8`).
`/tmp/anvil-parent-cone-instance-smoke-r1/manifest.json` is the focused
proof for parent-cone helper-instance routing
(`top_parent_cone_instances = 1`,
`child_input_bindings_from_parent_cone_instances = 4`).
`cargo test hierarchy_parent_outputs_can_depend_on_helper_instance_outputs`
is the focused proof for parent-output helper-instance composition
(`top_outputs_reaching_parent_cone_instances > 0`,
`hierarchy_outputs_reaching_parent_cone_instances > 0`,
`top_parent_cone_instance_output_fraction > 0.0`).
`cargo test recursive_hierarchy_parent_outputs_can_depend_on_helper_instances_below_top`
is the focused proof for the same parent-output helper route below the
top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_parent_output_cone_instance` scenario.
`cargo test recursive_hierarchy_parent_outputs_mix_helper_instances_with_parent_ports_below_top`
is the focused proof for recursive non-top parent-output helper cones
that also mix parent data-port support
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instance_mixed_support >
top_outputs_reaching_parent_cone_instance_mixed_support`,
`child_input_bindings_from_parent_cone_instances = 0`,
`child_input_bindings_from_registered_parent_cone_instances = 0`, and
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops = 0`).
This focused proof is banked in the full downstream-clean `r49` Phase 4
matrix and carried forward by the `r50` Phase 4 matrix through
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_outputs = true`.
`cargo test recursive_hierarchy_parent_outputs_can_spend_helper_budget_below_top`
is the focused proof for recursive non-top parent-output helper budget
spending (`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_outputs_reaching_parent_cone_instances >
top_outputs_reaching_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`).
This focused proof is banked by the full downstream-clean `r45` Phase 4
matrix through `saw_recursive_multiple_parent_cone_instances_per_parent`.
`cargo test recursive_hierarchy_parent_cone_helper_budget_allows_multiple_helpers_below_top`
is the focused proof for recursive non-top child-input helper budget
spending (`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`top_parent_cone_instances = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`child_input_bindings_from_parent_composed_logic >
top_child_input_bindings_from_parent_composed_logic`,
`child_input_bindings_from_parent_cone_instances >
top_child_input_bindings_from_parent_cone_instances`,
`child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`,
and `child_input_bindings_from_registered_parent_cone_instances = 0`).
This focused proof is banked by the full downstream-clean `r45` Phase 4
matrix through
`saw_recursive_multiple_parent_cone_instances_per_parent_child_inputs`.
`cargo test metrics::tests::design_metrics_capture_multiple_parent_cone_instance_budget`
now also proves unregistered parent-composed helper child-input mixed
support in the budgeted helper case
(`child_input_bindings_from_parent_cone_instance_mixed_support > 0`,
`top_child_input_bindings_from_parent_cone_instance_mixed_support > 0`,
`parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`,
`child_input_bindings_from_registered_parent_cone_instances = 0`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`).
The coverage-only Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-parent-helper-child-input-mixed-check/tool_matrix_report.json`
first records this as
`saw_hierarchy_parent_cone_instance_mixed_support_routing = true` and
`saw_recursive_hierarchy_parent_cone_instance_mixed_support_routing = true`
with `coverage_gaps = []`; it skipped Verilator/Yosys and therefore
is superseded by the full downstream-clean `r87` bank for downstream-clean evidence.
`cargo test metrics::tests::design_metrics_capture_parent_composed_parent_cone_instance_flop_routes`
now also proves stateful parent-composed helper child-input mixed
support in the unregistered helper-through-flop route
(`child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0`,
`top_child_input_bindings_from_parent_cone_instance_flop_mixed_support > 0`,
`parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_flop_mixed_support_child_input_binding_fraction > 0.0`,
and registered helper route counters remain zero). The coverage-only
Phase 4 matrix probe at
`/tmp/anvil-tool-matrix-phase4-stateful-helper-child-input-mixed-check/tool_matrix_report.json`
records this as
`saw_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
and
`saw_recursive_hierarchy_parent_composed_parent_cone_instance_flop_mixed_support_routing = true`
with `coverage_gaps = []`; it skipped Verilator/Yosys and therefore
is superseded by the full downstream-clean `r50` bank for downstream-clean evidence.
`cargo test recursive_hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops_below_top`
is the focused proof for stateful parent-output helper routing below
the top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops >
top_outputs_reaching_parent_cone_instances_through_parent_flops`,
`child_input_bindings_from_parent_cone_instances = 0`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_parent_output_cone_instance_state` scenario.
`cargo test recursive_hierarchy_parent_outputs_can_spend_stateful_helper_budget_below_top`
is the focused proof for recursive non-top stateful parent-output
helper budget spending (`realized_min_leaf_depth =
realized_max_leaf_depth = 2`,
`max_parent_cone_instances_per_internal_module = 3`,
`top_parent_cone_instances = 3`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops >
top_outputs_reaching_parent_cone_instances_through_parent_flops`,
`child_input_bindings_from_parent_cone_instances = 0`,
`child_input_bindings_from_parent_cone_instances_through_parent_flops = 0`,
and `child_input_bindings_from_registered_parent_cone_instances = 0`).
This focused proof is banked by the full downstream-clean `r45` Phase 4
matrix through
`saw_recursive_multiple_parent_cone_instances_per_parent_through_flops`.
`cargo test hierarchy_parent_cone_helper_budget_allows_multiple_helpers`
is the focused proof for budgeted helper allocation through child-input
routing (`top_parent_cone_instances = 3`,
`max_parent_cone_instances_per_internal_module = 3`).
`cargo test hierarchy_parent_outputs_can_spend_helper_budget` is the
focused proof for budgeted parent-output-only helper composition
(`top_parent_cone_instances = 3`,
`max_parent_cone_instances_per_internal_module = 3`,
`child_input_bindings_from_parent_cone_instances = 0`, and parent
outputs reaching helper outputs).
`cargo test hierarchy_registered_child_input_cones_can_use_helper_instances`
is the focused proof for registered helper-sourced child-input D cones
(`child_input_bindings_from_registered_parent_cone_instances > 0`,
`registered_parent_cone_instance_child_input_binding_fraction > 0.0`).
`cargo test hierarchy_sibling_routes_can_use_helper_instances` is the
focused proof for direct sibling helper routing
(`child_input_bindings_from_registered_instance_outputs = 0`,
`child_input_bindings_from_registered_parent_cone_instances = 0`,
`child_input_bindings_from_parent_cone_instances > 0`,
`parent_cone_instance_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_child_input_binding_fraction > 0.0`, and
`num_instances > planned_child_instances`).
This focused proof is also banked in the full downstream-clean `r30`
Phase 4 matrix.
`cargo test recursive_hierarchy_sibling_routes_can_use_helper_instances_below_top`
is the focused proof for the same direct sibling helper route below the
top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`child_input_bindings_from_instance_outputs >
top_child_input_bindings_from_instance_outputs`,
`child_input_bindings_from_parent_cone_instances >
top_child_input_bindings_from_parent_cone_instances`, and both registered
helper counters stay zero).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_direct_sibling_parent_cone_instance` scenario.
`cargo test recursive_hierarchy_registered_sibling_routes_can_use_helper_instances_below_top`
is the focused proof for the same direct registered sibling helper route
below the top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_instance_outputs >
top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`, and
`child_input_bindings_from_registered_parent_composed_logic = 0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_direct_registered_sibling_parent_cone_instance_state`
scenario.
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_helper_instances_below_top`
is the focused proof for multi-stage direct registered sibling helper
routing below the top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_multistage_instance_outputs >
top_child_input_bindings_from_registered_multistage_instance_outputs`,
`child_input_bindings_from_registered_multistage_parent_cone_instances >
top_child_input_bindings_from_registered_multistage_parent_cone_instances`,
`child_input_bindings_from_registered_parent_composed_logic = 0`, and
`child_input_bindings_from_registered_multistage_parent_composed_logic = 0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_registered_sibling_parent_cone_instance_multistage_state`
scenario.
`cargo test recursive_hierarchy_registered_sibling_routes_can_chain_without_helpers_below_top`
is the focused proof for multi-stage direct registered sibling routing
below the top parent without helper instances or parent-composed logic
in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_instance_outputs >
top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_multistage_instance_outputs >
top_child_input_bindings_from_registered_multistage_instance_outputs`,
`registered_multistage_instance_output_child_input_binding_fraction > 0.0`,
and registered parent-composed plus registered helper-chain counters
stay zero). This focused proof is banked in the full downstream-clean
`r46` Phase 4 matrix through
`saw_recursive_hierarchy_registered_multistage_sibling_routing = true`.
`cargo test recursive_hierarchy_registered_multistage_mixed_support_routes_below_top`
is the focused proof for multi-stage registered mixed-support routing
below the top parent without helper instances
(`child_input_bindings_from_registered_multistage_mixed_support >
top_child_input_bindings_from_registered_multistage_mixed_support`,
`registered_multistage_mixed_support_child_input_binding_fraction > 0.0`,
and registered helper-chain counters remain zero). This focused proof is
banked in the full downstream-clean `r47` Phase 4 matrix through
`saw_recursive_hierarchy_registered_multistage_mixed_support_routing = true`.
`cargo test recursive_hierarchy_registered_parent_composed_routes_can_chain_helper_instances_below_top`
is the focused proof for multi-stage registered parent-composed helper
routing below the top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_multistage_parent_composed_logic >
top_child_input_bindings_from_registered_multistage_parent_composed_logic`,
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances >
top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`,
and `child_input_bindings_from_registered_multistage_parent_cone_instances = 0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_registered_parent_cone_instance_multistage_state`
scenario.
`cargo test recursive_hierarchy_registered_child_input_cones_can_use_helper_instances_below_top`
is the focused proof for registered parent-composed helper D-cone
routing below the top parent in an exact-depth-2 recursive hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`,
`registered_parent_composed_child_input_binding_fraction > 0.0`, and
`registered_parent_cone_instance_child_input_binding_fraction > 0.0`).
This focused proof is banked in the full downstream-clean `r45` Phase 4
matrix through the dedicated
`phase4_recur_d2_registered_parent_cone_instance_state` scenario.
`cargo test recursive_hierarchy_registered_helper_routes_mix_parent_ports_below_top`
is the focused proof for recursive non-top registered parent-composed
helper D-cone routing that also mixes parent data-port support in the
same helper-sourced D cone
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`,
`child_input_bindings_from_registered_parent_composed_logic >
top_child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_parent_cone_instances >
top_child_input_bindings_from_registered_parent_cone_instances`,
`child_input_bindings_from_registered_parent_cone_instance_mixed_support >
top_child_input_bindings_from_registered_parent_cone_instance_mixed_support`,
and
`registered_parent_cone_instance_mixed_support_child_input_binding_fraction > 0.0`).
This focused proof is banked in the full downstream-clean `r48` Phase 4
matrix through
`saw_recursive_hierarchy_registered_parent_cone_instance_mixed_support_routing = true`.
`cargo test hierarchy_registered_sibling_routes_can_use_helper_instances`
is the focused proof for direct registered sibling helper routing
(`child_input_bindings_from_registered_parent_composed_logic = 0`,
`child_input_bindings_from_registered_parent_cone_instances > 0`,
`registered_parent_cone_instance_child_input_binding_fraction > 0.0`,
and `num_instances > planned_child_instances`).
This focused proof is also banked in the full downstream-clean `r30`
Phase 4 matrix.
`cargo test hierarchy_registered_sibling_routes_can_mix_parent_port_support`
is the focused proof for direct registered sibling mixed-support routing
(`child_input_bindings_from_registered_instance_outputs > 0`,
`child_input_bindings_from_registered_sibling_mixed_support > 0`,
`registered_sibling_mixed_support_child_input_binding_fraction > 0.0`,
`child_input_bindings_from_registered_parent_composed_logic = 0`, and
`child_input_bindings_from_registered_mixed_support = 0`).
This focused proof is banked in the full downstream-clean `r51` Phase 4
matrix and is carried forward by the `r87` bank through
`saw_hierarchy_registered_sibling_mixed_support_routing = true`.
`cargo test recursive_hierarchy_registered_sibling_routes_can_mix_parent_port_support_below_top`
is the focused proof for direct registered sibling mixed-support routing
below the top parent in exact-depth-2 recursive hierarchy
(`child_input_bindings_from_registered_instance_outputs > top_child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_sibling_mixed_support > top_child_input_bindings_from_registered_sibling_mixed_support`,
`hierarchy_parent_cone_instances = 0`,
`child_input_bindings_from_registered_parent_composed_logic = 0`, and
`child_input_bindings_from_registered_mixed_support = 0`). This focused
proof is banked in the full downstream-clean `r52` Phase 4 matrix through
`saw_recursive_hierarchy_registered_sibling_mixed_support_routing = true`.
`cargo test recursive_hierarchy_parent_composed_routes_mix_parent_ports_below_top_without_helpers`
is the focused proof for recursive non-top unregistered parent-composed
mixed-support child-input routing without helper instances
(`child_input_bindings_from_parent_composed_logic > top_child_input_bindings_from_parent_composed_logic`,
`child_input_bindings_from_mixed_support > top_child_input_bindings_from_mixed_support`,
`hierarchy_parent_cone_instances = 0`,
`child_input_bindings_from_registered_instance_outputs = 0`, and
`child_input_bindings_from_registered_parent_composed_logic = 0`). This
focused proof is banked in the full downstream-clean `r53` Phase 4 matrix
through `saw_recursive_hierarchy_mixed_support_child_inputs = true`.
`cargo test hierarchy_registered_sibling_routes_can_chain_through_parent_flops`
is the focused proof for multi-stage direct registered sibling routing
without parent-composed logic
(`child_input_bindings_from_registered_instance_outputs > 0`,
`child_input_bindings_from_registered_multistage_instance_outputs > 0`,
`top_child_input_bindings_from_registered_multistage_instance_outputs > 0`,
`child_input_bindings_from_registered_parent_composed_logic = 0`, and
`registered_multistage_instance_output_child_input_binding_fraction > 0.0`).
This focused proof is banked in the full downstream-clean `r30`
Phase 4 matrix through the dedicated
`phase4_hier2_inst4_registered_sibling_multistage_state` scenario.
`cargo test hierarchy_registered_sibling_routes_can_chain_helper_instances_through_parent_flops`
is the focused proof for multi-stage direct registered sibling helper
routing without parent-composed logic
(`child_input_bindings_from_registered_multistage_parent_cone_instances > 0`,
`top_child_input_bindings_from_registered_multistage_parent_cone_instances > 0`,
`registered_multistage_parent_cone_instance_child_input_binding_fraction > 0.0`,
`child_input_bindings_from_registered_parent_composed_logic = 0`, and
`child_input_bindings_from_registered_multistage_parent_composed_logic = 0`).
This focused proof is banked in the full downstream-clean `r30`
Phase 4 matrix through the dedicated
`phase4_hier2_inst4_registered_sibling_parent_cone_instance_multistage_state`
scenario.
`cargo test hierarchy_registered_parent_composed_routes_can_chain_helper_instances_through_parent_flops`
is the focused proof for multi-stage registered parent-composed helper
routing
(`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`,
`top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances > 0`,
`registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction > 0.0`,
`child_input_bindings_from_registered_parent_composed_logic > 0`, and
`child_input_bindings_from_registered_multistage_parent_cone_instances = 0`).
This focused proof is banked in the full downstream-clean `r30`
Phase 4 matrix through the dedicated
`phase4_hier2_inst4_registered_parent_cone_instance_multistage_state`
scenario.
`cargo test hierarchy_parent_composed_helper_routes_can_use_parent_flops`
is the focused proof for stateful parent-composed helper child-input
routing
(`child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`,
`top_child_input_bindings_from_parent_cone_instances_through_parent_flops > 0`,
`parent_cone_instance_flop_child_input_binding_fraction > 0.0`,
`top_parent_cone_instance_flop_child_input_binding_fraction > 0.0`, and
`child_input_bindings_from_registered_parent_cone_instances = 0`).
This focused proof is banked in the full downstream-clean `r30`
Phase 4 matrix through the dedicated
`phase4_hier2_inst4_parent_cone_instance_state` scenario.
`cargo test recursive_hierarchy_parent_composed_helper_routes_can_use_parent_flops_below_top`
is the focused proof for the same stateful parent-composed helper
child-input route below the top parent in an exact-depth-2 recursive
hierarchy
(`realized_min_leaf_depth = realized_max_leaf_depth = 2`,
`hierarchy_parent_cone_instances > top_parent_cone_instances`,
`hierarchy_parent_local_flops > top_local_flops`, and
`child_input_bindings_from_parent_cone_instances_through_parent_flops >
top_child_input_bindings_from_parent_cone_instances_through_parent_flops`).
This focused proof is banked in the full downstream-clean `r35` Phase 4
matrix through the dedicated
`phase4_recur_d2_parent_cone_instance_state` scenario.
`cargo test hierarchy_parent_outputs_can_route_helper_instances_through_parent_flops`
is the focused proof for parent-output helper routing through
parent-local state
(`top_outputs_reaching_parent_cone_instances_through_parent_flops > 0`,
`hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops > 0`,
`top_parent_cone_instance_flop_output_fraction > 0.0`,
`hierarchy_parent_cone_instance_flop_output_fraction > 0.0`, and
`child_input_bindings_from_parent_cone_instances = 0`).
This focused proof is banked in the full downstream-clean `r30`
Phase 4 matrix through the dedicated
`phase4_hier2_inst4_parent_output_cone_instance_state` scenario.
The aborted `r8` rerun is now only
historical runtime evidence: it showed that the Phase 4 gate should use
a hierarchy-focused sequential leaf profile instead of reusing the
fattest Phase 1 motif-heavy sequential stress shape.

Current HEAD also has a focused clean recursive-hierarchy proof at
`/tmp/anvil-hier-range-smoke-r1/manifest.json`, with:

- `realized_min_leaf_depth = 2`
- `realized_max_leaf_depth = 2`
- `instance_slots_by_parent_depth = {0: 2, 1: 5}`
- `min_child_instances_per_internal_module = 2`
- `max_child_instances_per_internal_module = 3`
- `hierarchy_parent_composed_outputs = 22`

That artifact is clean in Verilator plus both repo-owned Yosys modes
and remains a useful targeted numerical trust surface for the bounded
recursive lane even after the full Phase 4 matrix closure.

Current HEAD also has a focused clean mixed-depth recursive proof at
`/tmp/anvil-hier-mixed-depth-smoke-r1/manifest.json`, with:

- `realized_min_leaf_depth = 2`
- `realized_max_leaf_depth = 3`
- `leaf_module_occurrences_by_depth = {"2": 2, "3": 4}`
- `avg_child_instances_by_parent_depth = {"0": 2.0, "1": 2.0, "2": 2.0}`
- `hierarchy_parent_composed_outputs = 40`
- `top_parent_composed_outputs = 14`

That artifact is also clean in Verilator plus both repo-owned Yosys
modes and is the current trust surface for mixed shallow/deep recursive
shape without `.sv` inspection. The refreshed repo-owned Phase 4 gate
at `r30` now includes this axis too, so the focused smoke is no longer
standing alone as evidence.

Current HEAD also has a focused clean per-depth branching proof at
`/tmp/anvil-hier-depth-profile-smoke-r1/manifest.json`, with:

- `realized_min_leaf_depth = 2`
- `realized_max_leaf_depth = 2`
- `avg_child_instances_by_parent_depth = {"0": 4.0, "1": 2.0}`
- `min_child_instances_by_parent_depth = {"0": 4, "1": 2}`
- `max_child_instances_by_parent_depth = {"0": 4, "1": 2}`
- `hierarchy_parent_composed_outputs = 36`
- `top_parent_composed_outputs = 18`

That artifact is also clean in Verilator plus both repo-owned Yosys
modes and is the current trust surface for depth-specific hierarchy
branching without `.sv` inspection.

Current HEAD also has a focused clean profiled on-demand proof at
`/tmp/anvil-hier-profiled-ondemand-smoke-r1/manifest.json`, with:

- `num_profiled_instance_slots = 3`
- `profiled_instance_fraction = 1.0`
- `profiled_instantiated_module_fraction = 1.0`
- `dep_bearing_child_input_binding_fraction = 1.0`

That artifact is also clean in Verilator plus both repo-owned Yosys
modes and is the current trust surface for exact profiled `on-demand`
child-interface synthesis without `.sv` inspection.

Current HEAD also has a focused clean sibling-routing proof at
`/tmp/anvil-hier-sibling-routing-smoke-r1/manifest.json`, with:

- `child_input_bindings_from_instance_outputs = 6`
- `top_child_input_bindings_from_instance_outputs = 6`
- `instance_output_child_input_binding_fraction = 0.75`
- `top_instance_output_child_input_binding_fraction = 0.75`

That artifact is also clean in Verilator plus both repo-owned Yosys
modes and is the current trust surface for combinational sibling-routed
hierarchy child-input binding without `.sv` inspection.

`tool_matrix` now writes per-module or per-design checkpoint sidecars
and supports `--resume`, so interrupted output trees can be continued in
place.

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

**Verilator elaboration (leaf module):**
```bash
verilator --lint-only generated/mod_42_0000.sv
```

**Yosys synthesis (leaf module):**
```bash
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; stat"
```

**Icarus compile/elaboration (leaf module):**
```bash
iverilog -g2012 -o generated/mod_42_0000.vvp generated/mod_42_0000.sv
```

**Hierarchy elaboration / synthesis (directory output):**
```bash
verilator --lint-only generated-hier/*.sv
yosys -p "read_verilog -sv generated-hier/*.sv; synth -top <top-module> -noabc; stat; check"
iverilog -g2012 -s <top-module> -o generated-hier/design.vvp generated-hier/*.sv
```

To probe the ABC-enabled path explicitly:

```bash
yosys -p "read_verilog -sv generated/mod_42_0000.sv; synth -noabc; abc -fast; opt -fast; stat; check"
```

All enabled smoke tools should succeed on every generated file. If one
fails, that's a bug in `anvil` — file an issue with the seed and knobs
from `manifest.json`.

**Frontend expected-facts parity gates (optional, require external tools):**

```bash
cargo test --test frontend_parity -- --ignored parity_against_real_yosys_hierarchy_write_json --nocapture
cargo test --test frontend_parity -- --ignored parity_against_real_verilator_json_frontend_ast --nocapture
```

The Yosys gate checks the categories Yosys exposes after
`hierarchy -top; write_json`:
Seed/Top/TopParams/Instances/GenerateBranches. Yosys folds
top-localparams and package constants away. The Verilator gate uses
`--json-only` when the local Verilator build supports it and checks the
full frontend manifest scope:
Seed/Top/PackageConstants/TopParams/TopLocalparams/Instances/
GenerateBranches. In the current local evidence, it is clean across the
5 reproducibility seeds and writes artifacts under
`target/tmp/frontend-parity-signoff-verilator-json`.

## Agent introspection and the MCP server

ANVIL ships an opt-in, read-mostly surface so an AI agent can drive the
bug-hunting loop (generate → validate → minimize → file a reproducer). It is
**default-off**: it never changes the plain build or the `--artifact dut`
byte-identical contract. Full chapter:
`book/src/agent-mcp.md`. Wire contract:
`docs/AGENT_INTROSPECTION_SCHEMA.md`. Architecture: decision `0004`.

### `--introspect` (one-shot CLI)

Add `--introspect` to a single-artifact run (no `--out`, `--count 1`) to print
a versioned JSON introspection document instead of SystemVerilog:

```bash
# Construction-truth for one (seed, knobs), as JSON.
anvil --seed 42 --introspect
```

The document is a thin, versioned envelope (`schema_version`, `anvil_version`,
`lane`, a `request` echo with a content-addressed `run_id`, an `artifact`
descriptor, the `introspection` payload, and `warnings`). Every payload section
is the exact serde projection of an existing struct — `config` ← `Config`,
`module_metrics` ← `Metrics`, `design_metrics` ← `DesignMetrics` — so it adds
**zero** new computed truth. `run_id` is a hash of
`(schema_version, anvil_version, lane, seed, knobs)`, so the document is
reproducible. The `.sv` is referenced as a resource pointer, not inlined.

Omit `--introspect` and you get SystemVerilog exactly as before. The flag
rejects `--out` / `--count > 1` so the streamed directory-output path is never
touched.

### `anvil-mcp` (Model Context Protocol server)

`anvil-mcp` is a separate binary speaking newline-delimited JSON-RPC 2.0 over
stdio — the transport Claude Code and Cursor use:

```bash
cargo build --bin anvil-mcp

# Register it with an agent, e.g. Claude Code:
claude mcp add anvil -- /path/to/anvil/target/debug/anvil-mcp
```

It exposes three MCP primitives:

- **Tools** — the pure `generate`, `introspect`, `analyze`, `dump_config`, and
  `coverage_gaps`, plus the controlled `validate` and `minimize`. `generate` /
  `introspect` take an optional `lane` (`dut` — the default — `microdesign`, or
  `frontend`), so the agent can drive all three artifact families; the non-DUT
  lanes take scoped knobs (`n_params`, `n_children`) and carry an expected-facts
  manifest. `analyze` answers a derived-**relation** query over the DUT
  `(seed, config)` IR by pure graph traversal — `query = output_support`
  (default) returns each target's transitive combinational fan-in **support
  cone** (the inputs, flop Qs, and child-instance outputs an output depends on,
  plus cone size/depth), with `target` an output port name or `"flop:<id>"`
  (omit for all outputs); `query = input_reach` returns the **dual fan-out** —
  which outputs and flop `D`-cones each source reaches — with `target` a source
  (an input port name, `"flop:<id>"` for a flop Q, or `"<instance>.<port>"`; omit
  for all sources); `query = flop_reset_provenance` returns **per-flop reset/data
  provenance** (reset kind/value, zero-vs-hold default, mux kind/arms, has_d),
  with `target` a `"flop:<id>"` (omit for all flops); `query =
  module_reachability` returns **which modules in a design are reachable** from
  the top via the instance graph (per-module reachable/depth/instantiates/
  instance_count), with `target` a **module name** (omit for all modules). Unknown
  query/target → `-32602`. Relations, not behaviour (no shadow simulator). `coverage_gaps` projects the already-computed `coverage_gaps` out
  of a recorded `tool_matrix_report.json` (inline `report` or `report_path`) so
  the agent can target *unexercised* surfaces — read-only, no recompute, no
  spawn. The controlled tools run only ANVIL's vetted downstream invocations
  (`verilator` / `yosys` / `iverilog`, a fixed allow-list), in a sandboxed temp
  dir, RAM-guarded, with no arbitrary shell and an audit log of every call.
  `minimize` shrinks the input `(seed, knobs)` (seed held fixed); it never
  mutates or repairs RTL.
- **Resources** — `anvil://catalog/knobs`, `anvil://catalog/lanes`,
  `anvil://audit/log`, and per-artifact
  `anvil://artifact/<run_id>/{sv,introspection,manifest}` (the `manifest` is
  present only for the non-DUT lanes), plus
  `anvil://artifact/<run_id>/analysis/<query>` once `analyze` has run.
- **Prompts** — five packaged workflows: `find_downstream_bug`,
  `close_coverage_gap`, `minimize_reproducer`, `triage_tool_failures`,
  `explain_artifact`. Each renders an ordered chain over the tools above. Fetch
  one with `prompts/get` (with sample arguments) and the agent executes it.

You can smoke-test it by hand by piping JSON-RPC lines in over stdio:

```bash
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
 '{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}' \
 | ./target/debug/anvil-mcp
```

#### HTTP transport (opt-in)

Besides stdio (the default), `anvil-mcp` can serve the same protocol over a
minimal hand-rolled HTTP/1.1 POST transport with `--http <addr>` — one JSON-RPC
request per `POST`, driving the exact same dispatcher (no extra dependencies).
`<addr>` is a bare port (binds loopback, `127.0.0.1:<port>` — the safe default)
or a full `IP:PORT`:

```bash
./target/debug/anvil-mcp --http 8765          # binds 127.0.0.1:8765 (loopback)
curl -s -X POST http://127.0.0.1:8765/ -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

The HTTP transport **binds loopback by default** because the controlled
`validate` / `minimize` tools run real downstream tools; binding a non-loopback
address (e.g. `0.0.0.0:8765`) exposes them to the network and prints a warning.
The per-call guardrails are identical on both transports.

The agent is an experiment driver and explainer, never a signoff oracle: a
downstream tool rejecting valid-by-construction RTL is the bug signal; ANVIL's
manifests, metrics, and the real tools' results stay the source of truth.

## Use as a library

```rust
use anvil::{Config, Generator};

let cfg = Config::default().with_seed(42);
let mut gen = Generator::new(cfg);
let module = gen.generate_module();
println!("{}", anvil::emit::to_sv(&module));
```

See `examples/` for more patterns.
