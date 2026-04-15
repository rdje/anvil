# Knobs and Reproducibility

## Reproducibility

Every `anvil` invocation is deterministic in `(seed, knobs)`. The same
seed and same knobs produce byte-identical output, on any platform,
forever. This is non-negotiable.

Mechanism:

- One `ChaCha8Rng` instance, seeded once from the user-provided seed.
- All randomness flows from this RNG. No `thread_rng`. No system time.
  No floating-point ops that vary across platforms.
- Iteration order over hash maps is avoided in any path that affects
  output (use `BTreeMap` or sorted-`Vec` where iteration matters).
- The RNG is *not* sub-seeded per module — instead it is consumed
  serially. This means generating module N requires generating modules
  0..N first if you want exact reproduction. To reproduce a single
  module standalone, the manifest records its individual seed (derived
  from the master seed by a deterministic stream-position scheme).

## Knob taxonomy

Knobs fall into three categories:

### Structural knobs (shape)

Control the size and topology of generated modules.

- `min_inputs / max_inputs` — primary input port count range.
- `min_outputs / max_outputs` — primary output port count range.
- `min_width / max_width` — port and internal-wire width range.
- `max_depth` — maximum cone recursion depth.
- `num_leaf_modules` — pool size for hierarchical mode.
- `hierarchy_depth` — max sub-module nesting.

### Probability knobs (mix)

Control which choices are favored at each decision point.

- `flop_prob` — probability that a cone node becomes a flop.
- `share_prob` — probability of reusing an existing pool signal at a
  terminal selection point.
- `library_prob` — probability of picking from the existing module
  pool vs generating a fresh sub-module on demand.
- `terminal_reuse_prob` — probability of reusing a pool signal even
  outside of explicit sharing decisions.
- `constant_prob` — probability of emitting a constant terminal.
- `gate_*_weight` — relative weights for gate categories (bitwise,
  arithmetic, structured, comparison, reduction).

### Termination knobs (recursion control)

Control how the cone recursion ends.

- `leaf_prob_growth` — how fast the per-node leaf probability rises
  with depth. Linear: `min(1.0, depth / max_depth)` is the default.
- `max_nodes_per_module` — hard cap; if exceeded, force termination.

## Knob defaults

Sensible defaults aim for "interesting but not overwhelming" output:

```rust
Config {
    min_inputs: 2, max_inputs: 8,
    min_outputs: 1, max_outputs: 4,
    min_width: 1, max_width: 32,
    max_depth: 6,
    flop_prob: 0.0,           // Phase 2 enables
    share_prob: 0.0,          // Phase 3 enables
    hierarchy_depth: 0,       // Phase 5 enables
    library_prob: 0.5,
    terminal_reuse_prob: 0.3,
    constant_prob: 0.1,
    gate_bitwise_weight: 3,
    gate_arith_weight: 2,
    gate_struct_weight: 1,
    leaf_prob_growth: Linear,
    max_nodes_per_module: 1000,
}
```

## Knob serialization

Knobs are a `serde`-derived struct. Loading from JSON:

```bash
anvil --config knobs.json --seed 42
```

Writing the effective knobs back out (the merge of defaults and CLI
overrides) lets users save and replay configurations:

```bash
anvil --seed 42 --max-depth 8 --dump-config > my-knobs.json
anvil --config my-knobs.json --seed 42  # identical to previous
```

The manifest file in the output directory records the effective knobs
used for that generation run. Reproducing any output requires only
the manifest entry, not the original CLI invocation.
