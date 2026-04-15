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
  `0..N` first if you want exact reproduction. To reproduce a single
  module standalone, the manifest records its individual seed (derived
  from the master seed by a deterministic stream-position scheme).

## Knob taxonomy

Knobs fall into four categories:

### Structural knobs (shape)

Control the size and topology of generated modules.

- `min_inputs / max_inputs` — primary input port count range.
- `min_outputs / max_outputs` — primary output port count range.
- `min_width / max_width` — port and internal-wire width range.
- `max_depth` — maximum cone recursion depth.
- `max_nodes_per_module` — hard cap on node count; currently a safety
  ceiling, not hit in practice.
- `num_leaf_modules` — pool size for hierarchical mode (Phase 4).
- `hierarchy_depth` — max sub-module nesting (Phase 4).

### Sequential knobs (flops and mux motifs)

Control flop emission and D-input mux shape.

- `flop_prob` — per-non-leaf-node probability that a cone node becomes
  a flop. Default `0.15`.
- `max_flops_per_module` — hard cap on flops per module. Default `32`.
  Once hit, `build_cone` no longer considers the Flop branch.
- `min_mux_arms / max_mux_arms` — range for M, the number of mux arms
  on a flop's D input. Effective minimum is 2 (M=1 is excluded by
  design). Defaults `1, 4`.
- `flop_qfeedback_prob` — per-flop probability of the `QFeedback`
  kind (D = Q when no select fires) vs `ZeroDefault` (D = 0 when no
  select fires). Default `0.5`.
- `flop_mux_encoding_prob` — per-flop probability of the Encoded mux
  style (chained ternary over `Eq(sel, k)`) vs the OneHot style
  (OR-of-masked arms). Default `0.5`.
- `use_async_reset` — currently unused; flops are always async-reset
  by the single-CLK / single-RST_N discipline. Retained as a knob in
  case future work enables sync-reset as an option.

### Sharing knobs (tree vs DAG)

Control how often cone recursion terminates at an existing signal
instead of creating fresh logic.

- `share_prob` — per-operand probability of DAG-sharing (reuse an
  existing matching-width pool entry) at non-leaf decision points.
  Default `0.3`. See `sharing.md` for the tree-vs-DAG-per-recursion
  semantics.
- `terminal_reuse_prob` — probability of reusing a pool signal at
  forced-leaf decision points. Not currently consulted by
  `pick_terminal` (the always-prefer-matching-width policy there
  supersedes it); retained for future tuning.

### Construction strategy

- `construction_strategy` — which strategy `build_leaf_module` uses
  to order cone construction across outputs. Current values:
  - `sequential` (default): declaration order, one output at a time.
  - `shuffled`: random permutation of declaration order per seed.

  Planned future values: `interleaved`, `graph-first`. See
  `book/src/construction-strategies.md` for the full four-way
  comparison and rationale.

### Combinational mux block

- `comb_mux_prob` — probability that a non-leaf recursion point
  becomes an M-to-1 combinational mux block (instead of an operator
  gate). Default `0.1`. Ordering: flop takes priority if it also
  rolls; comb mux takes priority over operator.
- `comb_mux_encoding_prob` — per-mux probability of the Encoded
  style (chained ternary over `Eq(sel, k)`, `ceil(log2(M))`-bit
  select bus) vs the OneHot style (M 1-bit select signals, OR of
  masked arms). Default `0.5`.
- M (arm count) range reuses the block-level `min_mux_arms` /
  `max_mux_arms` knobs (shared with flop D-muxes).
- No Q-feedback knob for comb muxes — they have no state.

### Operator N-arity

- `min_gate_arity / max_gate_arity` — range for N, the arity of
  associative operators (`And`, `Or`, `Xor`, `Add`, `Mul`) when they
  are picked by `build_cone`. Each operator emission draws
  `N = rand(min_gate_arity..=max_gate_arity)` independently.
  Defaults `2, 4`. `Sub` is strictly 2-arity (not associative) and
  is not affected by this range. See Rule 14 in
  `book/src/structural-rules.md`.

### Motif mix and termination

- `constant_prob` — probability of emitting a constant terminal when
  no matching-width signal exists. Default `0.1`.
- `gate_*_weight` — relative weights for gate categories when picking
  a gate at a non-leaf recursion point. Defaults bitwise `3`, arith
  `2`, struct `1`, compare `1`, reduce `1`.
- Termination: there is no explicit `leaf_prob_growth` knob —
  `build_cone` uses a linear `depth / max_depth` ramp, forcing a leaf
  at `max_depth`.

### Hierarchy knobs (Phase 4+)

- `library_prob` — probability of picking from the pre-generated
  module pool vs generating a fresh sub-module on demand.

## Knob defaults

```rust
Config {
    seed: 0,
    // Structure
    min_inputs: 2,  max_inputs: 8,
    min_outputs: 1, max_outputs: 4,
    min_width: 1,   max_width: 32,
    max_depth: 6,
    max_nodes_per_module: 1000,
    // Sequential
    flop_prob: 0.15,
    max_flops_per_module: 32,
    min_mux_arms: 1, max_mux_arms: 4,
    flop_qfeedback_prob: 0.5,
    flop_mux_encoding_prob: 0.5,
    use_async_reset: true,
    // Sharing
    share_prob: 0.3,
    terminal_reuse_prob: 0.3,
    // Mix
    constant_prob: 0.1,
    gate_bitwise_weight: 3,
    gate_arith_weight:   2,
    gate_struct_weight:  1,
    gate_compare_weight: 1,
    gate_reduce_weight:  1,
    // Hierarchy (Phase 4+)
    hierarchy_depth: 0,
    num_leaf_modules: 0,
    library_prob: 0.5,
}
```

## CLI coverage

Every motif knob above that affects Phase 1/2 output has a dedicated
CLI flag, so all combinations are reachable without writing a config
file:

```
--seed, --count, --out, --config, --dump-config
--min-inputs, --max-inputs, --min-outputs, --max-outputs
--min-width, --max-width, --max-depth
--flop-prob, --max-flops-per-module
--min-mux-arms, --max-mux-arms
--flop-qfeedback-prob, --flop-mux-encoding-prob
--share-prob
```

Knobs without a CLI flag today (gate weights, `constant_prob`,
`library_prob`, hierarchy fields) are reachable via `--config FILE`.

## Knob serialization

Knobs are a `serde`-derived struct. Loading from JSON:

```bash
anvil --config knobs.json --seed 42
```

Writing the effective knobs back out (the merge of defaults and CLI
overrides) lets users save and replay configurations:

```bash
anvil --seed 42 --max-depth 8 --dump-config > my-knobs.json
anvil --config my-knobs.json --seed 42  # byte-identical to previous
```

The manifest file in the output directory records the effective knobs
used for that generation run. Reproducing any output requires only
the manifest entry, not the original CLI invocation.
