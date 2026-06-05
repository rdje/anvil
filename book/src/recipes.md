# Recipes

Short "I want to do X" cookbook. Each recipe states a goal, gives the
exact command, and explains which knobs matter.

## "I want a minimal smoke-test corpus"

Small, fast-to-generate modules for a CI Verilator/Yosys lint pass:

```bash
cargo run --release -- --seed 1 --count 50 --out ./smoke/ \
      --max-depth 3 --max-inputs 4 --max-outputs 2 \
      --max-width 16 --flop-prob 0.2 --share-prob 0.3
```

Knobs to tune:

- `--max-depth 3` keeps cones shallow → modules stay small.
- `--max-width 16` keeps data widths moderate → SV stays readable.
- `--flop-prob 0.2` gives a mix of combinational and sequential blocks.

## "I want an executable Verilator/Yosys axis matrix"

Use the repo-owned matrix harness instead of hand-running one seed at a
time:

<!-- book-test: skip — tool_matrix matrix run invokes Verilator/Yosys (external; not the generator surface) -->
```bash
cargo run --bin tool_matrix -- --out ./tool-matrix
```

This does more than emit files:

- sweeps a curated matrix over construction strategy, identity mode,
  factorization level, and two stress profiles;
- runs Verilator and Yosys on every generated file;
- writes per-scenario corpora plus `tool_matrix_report.json`; and
- fails the command if either downstream tool fails anywhere in the
  matrix.

Useful variants:

<!-- book-test: skip — tool_matrix matrix run invokes Verilator/Yosys (external; not the generator surface) -->
```bash
# See the built-in scenario names without generating anything.
cargo run --bin tool_matrix -- --list-scenarios

# Spend more runtime for more actual motif/knob coverage.
cargo run --bin tool_matrix -- --out ./tool-matrix --modules-per-scenario 4

# Treat missed matrix coverage as a failing result too.
cargo run --bin tool_matrix -- --out ./tool-matrix --fail-on-coverage-gap

# Run the repo-owned Phase 1 gate shape (>=1000 modules total).
cargo run --bin tool_matrix -- --out ./tool-matrix-phase1 --phase1-gate

# Compare Yosys with and without ABC on the same corpus.
cargo run --bin tool_matrix -- --out ./tool-matrix --yosys-mode both
```

## "I want a real hierarchy, not just one leaf module"

Generate a depth-1 parent with a library of three leaf module
definitions:

```bash
cargo run --release -- --seed 42 --out ./hier-smoke \
      --hierarchy-depth 1 \
      --num-leaf-modules 3
```

Hierarchy directory output writes one `.sv` file per module definition
plus a `manifest.json`:

```text
hier-smoke/
  mod_42_0000.sv   # leaf
  mod_42_0001.sv   # leaf
  mod_42_0002.sv   # leaf
  mod_42_0003.sv   # top parent
  manifest.json
```

The top is recorded in the manifest:

<!-- book-test: skip — needs jq + a prior run's manifest.json (external tool) -->
```bash
jq -r '.designs[0].top' ./hier-smoke/manifest.json
```

Run tools by reading every generated `.sv` file and selecting the
manifest top:

<!-- book-test: skip — needs jq + Verilator + Yosys (external tools) -->
```bash
top=$(jq -r '.designs[0].top' ./hier-smoke/manifest.json)
verilator --lint-only --top-module "$top" ./hier-smoke/*.sv
yosys -p "read_verilog -sv ./hier-smoke/*.sv; synth -top $top -noabc; stat; check"
```

## "I want to reuse a child definition many times"

Make the parent instantiate more child slots than there are unique leaf
definitions:

```bash
cargo run --release -- --seed 42 --out ./hier-reuse \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 5
```

This creates two leaf module definitions and five instance slots. The
same child definitions appear multiple times under different instance
names. Useful metrics in `manifest.json` include
`num_unique_instantiated_modules`, `num_multiuse_instantiated_modules`,
and `avg_instances_per_unique_instantiated_module`.

The opposite shape also matters:

```bash
cargo run --release -- --seed 42 --out ./hier-under \
      --hierarchy-depth 1 \
      --num-leaf-modules 5 \
      --num-child-instances 2
```

That creates a larger library than the parent uses. It stresses unused
module definitions and records the result through `num_unused_leaf_modules`
and `unused_library_fraction`.

## "I want recursive hierarchy"

Use the bounded recursive lane rather than `--hierarchy-depth 1`:

```bash
cargo run --release -- --seed 42 --out ./hier-recursive \
      --min-hierarchy-depth 2 \
      --max-hierarchy-depth 3 \
      --min-child-instances-per-module 2 \
      --max-child-instances-per-module 3
```

This can produce intermediate parent modules, not only leaves under one
top. The realized tree shape is numeric in the manifest:
`realized_min_leaf_depth`, `realized_max_leaf_depth`,
`leaf_module_occurrences_by_depth`, and
`avg_child_instances_by_parent_depth`.

To force a wide top and narrower lower level:

```bash
cargo run --release -- --seed 42 --out ./hier-profiled-depth \
      --min-hierarchy-depth 2 \
      --max-hierarchy-depth 2 \
      --min-child-instances-per-module 1 \
      --max-child-instances-per-module 3 \
      --child-instances-per-depth 0=4:4 \
      --child-instances-per-depth 1=2:2
```

Depth `0` is the top parent, depth `1` is its direct children, and so
on.

## "I want fresh children per instance slot"

The default hierarchy child source mode is `library`: define a reusable
pool and instantiate from it. To synthesize a fresh child definition for
each planned slot, use `on-demand`:

```bash
cargo run --release -- --seed 42 --out ./hier-on-demand \
      --hierarchy-depth 1 \
      --num-child-instances 4 \
      --hierarchy-child-source-mode on-demand
```

On-demand children are generated against parent-planned exact
data-interface profiles. Inspect `profiled_instance_fraction` and
`profiled_instantiated_module_fraction` in the design metrics to
confirm that the requested interfaces were actually realized.

## "I want child inputs to come from other children or parent logic"

Direct sibling routing:

```bash
cargo run --release -- --seed 42 --out ./hier-sibling-route \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 1.0
```

Parent-composed child-input cones:

```bash
cargo run --release -- --seed 42 --out ./hier-parent-cones \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-child-input-cone-prob 1.0
```

Parent-cone helper instances, where the parent instantiates an extra
helper child as a source for child-input cones:

```bash
cargo run --release -- --seed 42 --out ./hier-helper-cones \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 1.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

Useful metrics:

- `child_input_bindings_from_instance_outputs`
- `child_input_bindings_from_parent_composed_logic`
- `child_input_bindings_from_parent_cone_instances`
- `top_parent_cone_instances`
- `max_parent_cone_instances_per_internal_module`
- the matching `*_fraction` fields

To allow more than one helper child in a single parent:

```bash
cargo run --release -- --seed 42 --out ./hier-helper-budget3 \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 1.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --max-parent-cone-instances-per-module 3 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

Useful metrics:

- `max_parent_cone_instances_per_internal_module`
- `top_parent_cone_instances`
- `hierarchy_parent_cone_instances`

To focus helper instances on parent-output composition instead of
child-input bindings:

```bash
cargo run --release -- --seed 42 --out ./hier-helper-outputs \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

Useful metrics:

- `top_outputs_reaching_parent_cone_instances`
- `hierarchy_outputs_reaching_parent_cone_instances`
- `top_parent_cone_instance_output_fraction`
- `hierarchy_parent_cone_instance_output_fraction`

To make the parent-output helper route stateful, also enable
parent-local flops and keep the helper/child-input routes isolated:

```bash
cargo run --release -- --seed 42 --out ./hier-helper-output-state \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --hierarchy-parent-flop-prob 1.0 \
      --max-flops-per-module 64 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0 \
      --min-width 1 \
      --max-width 8 \
      --max-depth 1
```

Useful metrics:

- `top_outputs_reaching_parent_cone_instances_through_parent_flops`
- `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`
- `top_parent_cone_instance_flop_output_fraction`
- `hierarchy_parent_cone_instance_flop_output_fraction`
- `child_input_bindings_from_parent_cone_instances` should stay `0`
  when you want an output-only proof

To make parent-composed child-input helper routing stateful, keep the
registered child-input routes disabled and enable parent-local helper
state:

```bash
cargo run --release -- --seed 42 --out ./hier-helper-child-input-state \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 1.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --max-parent-cone-instances-per-module 1 \
      --hierarchy-parent-flop-prob 1.0 \
      --max-flops-per-module 64 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0 \
      --min-width 1 \
      --max-width 8 \
      --max-depth 1
```

Useful metrics:

- `child_input_bindings_from_parent_cone_instances_through_parent_flops`
- `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`
- `parent_cone_instance_flop_child_input_binding_fraction`
- `top_parent_cone_instance_flop_child_input_binding_fraction`
- `child_input_bindings_from_registered_parent_cone_instances` should
  stay `0` when you want the unregistered parent-composed route, not a
  registered child-input D-cone proof

## "I want registered hierarchy routes"

Registered sibling route:

```bash
cargo run --release -- --seed 42 --out ./hier-registered-sibling \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 1.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --max-flops-per-module 8
```

Registered parent-composed route:

```bash
cargo run --release -- --seed 42 --out ./hier-registered-parent-cone \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 1.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --max-flops-per-module 8
```

Registered parent-composed route whose D cones use helper instances:

```bash
cargo run --release -- --seed 42 --out ./hier-registered-helper-cones \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 1.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --max-parent-cone-instances-per-module 3 \
      --max-flops-per-module 8 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

Multi-stage registered sibling route:

```bash
cargo run --release -- --seed 42 --out ./hier-registered-sibling-chain \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 1.0 \
      --hierarchy-registered-child-input-cone-prob 0.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --hierarchy-parent-cone-instance-prob 0.0 \
      --hierarchy-parent-flop-prob 0.0 \
      --max-flops-per-module 8 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

Multi-stage registered parent-composed helper route:

```bash
cargo run --release -- --seed 42 --out ./hier-registered-helper-parent-chain \
      --hierarchy-depth 1 \
      --num-leaf-modules 2 \
      --num-child-instances 4 \
      --hierarchy-sibling-route-prob 0.0 \
      --hierarchy-registered-sibling-route-prob 0.0 \
      --hierarchy-registered-child-input-cone-prob 1.0 \
      --hierarchy-child-input-cone-prob 0.0 \
      --hierarchy-parent-cone-instance-prob 1.0 \
      --max-parent-cone-instances-per-module 1 \
      --hierarchy-parent-flop-prob 0.0 \
      --max-flops-per-module 8 \
      --terminal-reuse-prob 1.0 \
      --constant-prob 0.0
```

The route metrics distinguish the shapes:
`child_input_bindings_from_registered_instance_outputs`,
`child_input_bindings_from_registered_multistage_instance_outputs`,
`child_input_bindings_from_registered_parent_composed_logic`,
`child_input_bindings_from_registered_mixed_support`, and
`child_input_bindings_from_registered_multistage_parent_composed_logic`.
For the direct registered sibling chain, also inspect
`registered_multistage_instance_output_child_input_binding_fraction`.
When the helper route is active, also inspect
`child_input_bindings_from_registered_parent_cone_instances` and
`registered_parent_cone_instance_child_input_binding_fraction`. When a
helper-sourced registered sibling route chains through a later parent
flop, inspect
`child_input_bindings_from_registered_multistage_parent_cone_instances`
and
`registered_multistage_parent_cone_instance_child_input_binding_fraction`.
For a helper-sourced registered parent-composed route that chains
through earlier parent state, inspect
`child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`
and
`registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`.

## "I want fanout stress"

Internal wires driving many consumers — stresses common-subexpression
elimination, timing convergence, and fanout-aware buffering in
synthesis:

```bash
cargo run --release -- --seed 42 --max-depth 6 --min-inputs 6 --max-inputs 8 \
      --min-outputs 4 --max-outputs 6 \
      --share-prob 0.9 --flop-prob 0.0
```

- `--share-prob 0.9` makes nearly every operand pick an existing
  signal instead of recursing.
- More outputs (`--min-outputs 4`) means more output cones drawing
  from the same internal pool.
- `--flop-prob 0.0` keeps attention on the combinational DAG.

## "I want flop-heavy modules"

For testing sequential optimizations, clock-network synthesis, or
retiming:

```bash
cargo run --release -- --seed 7 --max-depth 4 --flop-prob 0.5 \
      --max-flops-per-module 64 \
      --flop-qfeedback-prob 0.7
```

- `--flop-prob 0.5` turns half of recursion points into flops.
- `--max-flops-per-module 64` raises the safety cap.
- `--flop-qfeedback-prob 0.7` biases toward holding registers (more
  realistic for real designs).

## "I want stress on the encoded mux decoder"

If your tooling has special-case code for case-statement / encoded
mux synthesis, exercise it specifically:

```bash
cargo run --release -- --seed 13 --flop-prob 1.0 --max-flops-per-module 16 \
      --min-mux-arms 3 --max-mux-arms 8 \
      --flop-mux-encoding-prob 1.0
```

`--flop-mux-encoding-prob 1.0` forces every flop that draws `M >= 2`
to use the encoded-select (chained-ternary) style. `--max-mux-arms 8`
gives enough arms that `ceil(log2(M))` select widths of 2 and 3 both
appear.

## "I want combinational muxes, not just flop D-muxes"

The same OneHot / Encoded styles used for flop D-muxes are available
as first-class combinational blocks. Crank `--comb-mux-prob` to make
them show up routinely:

```bash
cargo run --release -- --seed 5 --max-depth 5 --comb-mux-prob 0.4 \
      --comb-mux-encoding-prob 0.5 \
      --min-mux-arms 2 --max-mux-arms 4
```

`--comb-mux-prob 0.4` means ~40% of non-leaf recursion points
become M-to-1 combinational muxes. Use `--comb-mux-encoding-prob
0.0` to force OneHot only, `1.0` for Encoded only.

## "I want stress on the one-hot mux OR-tree"

The mirror of the previous recipe:

```bash
cargo run --release -- --seed 14 --flop-prob 1.0 --max-flops-per-module 16 \
      --min-mux-arms 3 --max-mux-arms 8 \
      --flop-mux-encoding-prob 0.0
```

Forces the one-hot style. Every flop's D becomes
`OR_i({W{sel_i}} & data_i)`, exercising replicate-concat, wide
bitwise AND, and reduce-OR patterns.

## "I want narrow-data stress"

Small widths exercise 1-bit and narrow-integer code paths in
synthesis tools that sometimes treat these specially:

```bash
cargo run --release -- --seed 20 --max-width 4 --min-width 1 \
      --max-depth 5 --flop-prob 0.2 --share-prob 0.4
```

## "I want wide-data stress"

Symmetrically, wide data exercises wide-adder, wide-concat, and
memory-macro inference paths:

```bash
cargo run --release -- --seed 30 --min-width 32 --max-width 128 \
      --max-depth 4 --flop-prob 0.2 --share-prob 0.4
```

Note: constants are truncated at 128 bits (see the code `make_constant`
helper). Module output widths beyond 128 are allowed but may emit
`128'h0` as constant operands where the adapter can't find matching
logic.

## "I want to reproduce a specific generated module"

Every `anvil` invocation is deterministic in `(seed, knobs)`. To
replay a specific module from a batch:

1. Look up its entry in `manifest.json`:

   ```json
   {
     "seed": 42,
     "config": { ... all effective knobs ... },
     "modules": [
       { "file": "mod_42_0007.sv", "name": "mod_42_0007",
         "inputs": 5, "outputs": 3, "nodes": 134 },
       ...
     ]
   }
   ```

2. Replay the exact same seed and config:

   <!-- book-test: skip — step 2 of the recipe; consumes extracted_knobs.json produced by step 1 above -->
   ```bash
   cargo run --release -- --seed 42 --count 100 --config extracted_knobs.json \
         --out ./replay/
   ```

The module at `./replay/mod_42_0007.sv` will be byte-identical to
the original.

## "I want to reproduce a single module in isolation"

To generate only one module (not a batch-and-index-into), you need
the individual module's seed. The CLI does not currently derive
per-module seeds — generate the batch then copy the one file, or use
`--seed N --count 1` and iterate N manually until you find one with
the shape you want.

(A future release may add `--module-index K` to jump straight to the
K-th module of a batch. For now, the byte-identical-batch guarantee
is the reproduction mechanism.)

## "I want to test my parser only, not synthesis"

```bash
cargo run --release -- --seed 1 --count 1000 --out ./parse-stress/ \
      --max-depth 8 --max-width 64
```

Large, deep, unusual-width modules. Parsing does not care about
semantic validity per se, so crank the structural diversity.

## "I want to drive a formal equivalence flow"

Generate many small modules with moderate complexity so the formal
tool has time to prove equivalence against some reference:

```bash
cargo run --release -- --seed 1 --count 200 --out ./equiv/ \
      --max-depth 4 --max-inputs 4 --max-outputs 2 \
      --max-width 16 --max-flops-per-module 8 \
      --share-prob 0.3
```

Equivalence flows usually don't scale to very deep cones; this recipe
keeps each module small enough to finish quickly.

## "I want to see fewer redundant expressions" (strict CSE)

This is the default. Every unique AST is named once, no matter
how many consumers reference it. A comparison like
`slice_17 == 2'h2` becomes a single `eq_0` — downstream muxes
that need the result just reference `eq_0` instead of creating
their own copy.

```bash
cargo run --release -- --seed 42
# --max-ast-instances 1 is the default.
```

To verify this is in effect, look at the metrics block:

```bash
cargo run --release -- --seed 42 --metrics 2>&1 >/dev/null | grep ast_multiplicity
# max_gate_ast_multiplicity: 1
# max_constant_ast_multiplicity: 1
```

`max_gate_ast_multiplicity: 1` means no AST is named more than
once.

## "I want duplicated expressions anyway" (bounded duplication)

Downstream synthesis tools sometimes have special handling for
redundant subexpressions. To exercise that path, raise the cap:

```bash
cargo run --release -- --seed 42 --max-ast-instances 5 --metrics 2>&1 >/dev/null | grep ast_multiplicity
# max_gate_ast_multiplicity: 3       ← higher than 1: CSE relaxed
# max_constant_ast_multiplicity: 4
```

Set `--max-ast-instances 4294967295` to effectively turn CSE off
entirely.

## "I want pathological mux shapes" (arm duplication)

The default forbids `(s) ? (x) : (x)` muxes (both arms on the
same signal — semantically a no-op). To emit them anyway for
stress testing:

```bash
# 50% chance of a duplicate arm being accepted.
cargo run --release -- --seed 42 --mux-arm-duplication-rate 0.5

# Full relaxation — arms may all be connected to the same data.
cargo run --release -- --seed 42 --mux-arm-duplication-rate 1.0
```

Verify with metrics:

```bash
cargo run --release -- --seed 42 --mux-arm-duplication-rate 1.0 --metrics 2>&1 >/dev/null \
      | grep -E "num_muxes_(2to1|degenerate)"
# num_muxes_2to1: 11         ← total 2-to-1 muxes
# num_muxes_degenerate: 1    ← of which one has (s)?(x):(x)
```

At the default (0.0) `num_muxes_degenerate` is always 0.

## "I want to verify a knob is doing something"

Every knob has a metric — see the
["Knob effectiveness map"](knobs.md#knob-effectiveness-map) at
the bottom of the knobs chapter. The pattern is: run at default,
run at a boundary value, grep the metric.

```bash
# Default flop count for seed 42:
cargo run --release -- --seed 42 --metrics 2>&1 >/dev/null | grep num_flops

# Crank flops up:
cargo run --release -- --seed 42 --flop-prob 0.5 --metrics 2>&1 >/dev/null | grep num_flops

# Expect num_flops to increase.
```

If a knob doesn't shift its metric, something's off — either the
knob is masked by another knob, has an unintended default, or
isn't actually wired. File an issue with the two metric dumps.

## "I want to sweep a knob and compare"

The metrics block is JSON; use `jq` or a short script:

<!-- book-test: skip — pipes to jq (external tool, not assumed in CI) -->
```bash
for fp in 0.0 0.1 0.3 0.5 0.7; do
  cargo run --release -- --seed 42 --flop-prob $fp --metrics 2>&1 >/dev/null \
        | jq -r "\"flop-prob=$fp num_flops=\\(.num_flops) num_nodes=\\(.num_nodes)\""
done
```

Sample output (seed 42, other knobs at defaults):

```text
flop-prob=0.0 num_flops=0  num_nodes=106
flop-prob=0.1 num_flops=6  num_nodes=131
flop-prob=0.3 num_flops=14 num_nodes=150
flop-prob=0.5 num_flops=22 num_nodes=191
flop-prob=0.7 num_flops=29 num_nodes=223
```

Clean monotone — the knob does what it says.

## "I want to see how the factorization dial affects output"

The `--factorization-level` knob is a single dial along the
sharing chain. Sweep it and see what each layer changes:

<!-- book-test: skip — pipes to jq (external tool, not assumed in CI) -->
```bash
for lvl in none cse operand-unique commutative e-graph; do
  gates=$(cargo run --release -- --seed 42 --factorization-level $lvl --metrics 2>&1 >/dev/null \
    | jq -r .num_gates)
  printf "%-16s gates=%s\n" "$lvl" "$gates"
done
```

Sample output at seed 42:

```text
none             gates=1961
cse              gates=1776
operand-unique   gates=2368
commutative      gates=2368
e-graph          gates=2368    ← default (theoretical ceiling)
```

Walking the differences:

- **`none` → `cse`** (1961 → 1776): syntactic CSE collapses
  duplicate AST nodes. Gates drop because repeated expressions
  now share a single `NodeId`.
- **`cse` → `operand-unique`** (1776 → 2368): Rule 8 starts
  rejecting `x + x`, `x & x`, etc. The rejection retries
  produce different random paths, so the module takes more
  gates to satisfy the output drives. The knob makes output
  **larger but cleaner**.
- **`operand-unique` → `commutative`**: at this seed, no
  observable change — the generator's operand-picking paths
  rarely produce both `a+b` and `b+a` in the same module. The
  layer still matters for correctness (tighter
  NodeId-as-identity contract).
- **`commutative` → `associative` / `constant-fold` /
  `peephole`**: increasingly aggressive canonicalisation. At
  some seeds these layers materially reduce gate count; at
  others they merely tighten the identity contract without
  changing aggregate size.
- **`peephole` → `e-graph`**: this is now a bounded semantic
  upgrade, not a pure alias. On seeds where small-support
  different-shape cones exist over the same canonical endpoints,
  `e-graph` can reduce gate count further; on other seeds it may
  still look identical.

Default is `e-graph` (the theoretical ceiling). The generator
activates every layer it knows how to implement; `effective()`
keeps the bounded live fragment on and leaves room for future
strengthening.

Use `--factorization-level none` when you explicitly want to
stress a downstream CSE pass on un-deduped input.

## "I want to trace what the generator is doing"

Turn on the trace. Levels go off → low → medium → high → debug:

```bash
# Module milestones + warnings only (quiet).
cargo run --release -- --seed 42 --trace low 2>log && head log
```

```text
INFO anvil: 🚀 anvil start seed=42 count=1
INFO generate_leaf_module{index=0 seed=42}: anvil::gen::module: 🚀 build module n_in=3 n_out=4 strategy=Interleaved
WARN generate_leaf_module{index=0 seed=42}: anvil::gen::cone: 🔁 cone root empty-dep, retrying attempt=0
INFO generate_leaf_module{index=0 seed=42}: anvil::gen::module: ✅ module finalized module=mod_42_0000 nodes=1104 flops=32 semantic_gates_merged=4 flops_merged=0 fsms_merged=0 drives=4 orphans=0 compacted=360 repaired_constant_drives=0 repaired_profiled_inputs=0 enforced_profiled_interface=0
INFO to_sv{module=mod_42_0000}: anvil::emit::sv: ✍️ emit SV gates=971 flops=32 instances=0 inputs=5 outputs=4
INFO anvil: ✅ anvil done
```

Raise to `medium` or `high` for phase-by-phase and per-decision
events. `--trace-file path.log` routes output to a file instead
of stderr. See [USER_GUIDE.md](../../USER_GUIDE.md#tracing-and-debugging)
for the level table and emoji legend.

## Request a new recipe

If your use case doesn't fit the above, the knob reference in
[Knobs](knobs.md) shows every lever. If a common scenario is missing
from this cookbook, file an issue with the command that works — it
will become a recipe.
