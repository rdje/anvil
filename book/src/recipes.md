# Recipes

Short "I want to do X" cookbook. Each recipe states a goal, gives the
exact command, and explains which knobs matter.

## "I want a minimal smoke-test corpus"

Small, fast-to-generate modules for a CI Verilator/Yosys lint pass:

```bash
anvil --seed 1 --count 50 --out ./smoke/ \
      --max-depth 3 --max-inputs 4 --max-outputs 2 \
      --max-width 16 --flop-prob 0.2 --share-prob 0.3
```

Knobs to tune:

- `--max-depth 3` keeps cones shallow → modules stay small.
- `--max-width 16` keeps data widths moderate → SV stays readable.
- `--flop-prob 0.2` gives a mix of combinational and sequential blocks.

## "I want fanout stress"

Internal wires driving many consumers — stresses common-subexpression
elimination, timing convergence, and fanout-aware buffering in
synthesis:

```bash
anvil --seed 42 --max-depth 6 --min-inputs 6 --max-inputs 8 \
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
anvil --seed 7 --max-depth 4 --flop-prob 0.5 \
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
anvil --seed 13 --flop-prob 1.0 --max-flops-per-module 16 \
      --min-mux-arms 3 --max-mux-arms 8 \
      --flop-mux-encoding-prob 1.0
```

`--flop-mux-encoding-prob 1.0` forces every flop that draws `M >= 2`
to use the encoded-select (chained-ternary) style. `--max-mux-arms 8`
gives enough arms that `ceil(log2(M))` select widths of 2 and 3 both
appear.

## "I want stress on the one-hot mux OR-tree"

The mirror of the previous recipe:

```bash
anvil --seed 14 --flop-prob 1.0 --max-flops-per-module 16 \
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
anvil --seed 20 --max-width 4 --min-width 1 \
      --max-depth 5 --flop-prob 0.2 --share-prob 0.4
```

## "I want wide-data stress"

Symmetrically, wide data exercises wide-adder, wide-concat, and
memory-macro inference paths:

```bash
anvil --seed 30 --min-width 32 --max-width 128 \
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

   ```bash
   anvil --seed 42 --count 100 --config extracted_knobs.json \
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
anvil --seed 1 --count 1000 --out ./parse-stress/ \
      --max-depth 8 --max-width 64
```

Large, deep, unusual-width modules. Parsing does not care about
semantic validity per se, so crank the structural diversity.

## "I want to drive a formal equivalence flow"

Generate many small modules with moderate complexity so the formal
tool has time to prove equivalence against some reference:

```bash
anvil --seed 1 --count 200 --out ./equiv/ \
      --max-depth 4 --max-inputs 4 --max-outputs 2 \
      --max-width 16 --max-flops-per-module 8 \
      --share-prob 0.3
```

Equivalence flows usually don't scale to very deep cones; this recipe
keeps each module small enough to finish quickly.

## Request a new recipe

If your use case doesn't fit the above, the knob reference in
[Knobs](knobs.md) shows every lever. If a common scenario is missing
from this cookbook, file an issue with the command that works — it
will become a recipe.
