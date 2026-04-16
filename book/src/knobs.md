# Knobs and Reproducibility

This chapter is the full catalog. You don't need to read it
top-to-bottom — it's organised as a reference. **Most users only
touch 2–4 knobs for their scenario.** The [Recipes](recipes.md)
chapter has ready-made combinations for common tasks; start
there if you just want a working command.

## Quick reference

The full catalog is below. This table is a scannable summary of
the knobs you're most likely to touch day-to-day:

| Knob                          | Default  | What it controls                                      |
|-------------------------------|----------|-------------------------------------------------------|
| `--seed`                      | 0        | RNG seed. Same seed + same knobs = identical output.  |
| `--count`                     | 1        | How many modules to generate in one run.              |
| `--min-inputs` / `--max-inputs` | 2 / 8  | Primary input port count range.                       |
| `--min-outputs` / `--max-outputs` | 1 / 4 | Primary output port count range.                     |
| `--min-width` / `--max-width` | 1 / 32   | Port and internal-wire width range.                   |
| `--max-depth`                 | 6        | Maximum cone recursion depth.                         |
| `--flop-prob`                 | 0.15     | Probability that a recursion point becomes a flop.    |
| `--share-prob`                | 0.3      | Probability of sharing an existing signal vs recursing. |
| `--construction-strategy`     | graph-first | How module logic is constructed (see below).      |
| `--factorization-level`       | e-graph  | Dial along the sharing chain: none / cse / operand-unique / commutative / associative / constant-fold / peephole / e-graph. |
| `--max-ast-instances`         | 1        | How many times one AST may be named (1 = strict CSE). |
| `--mux-arm-duplication-rate`  | 0.0      | Probability N-to-1 mux arms may share the same signal. |
| `--trace <level>`             | off      | Generation trace: off / low / medium / high / debug.  |
| `--metrics`                   | off      | Print per-module metrics JSON to stderr.              |

Everything else is available for fine control — see the categories
below. All knobs can also be set via a JSON file with `--config`,
and the effective merge of defaults + CLI overrides is printed by
`--dump-config`.

## Measurement doctrine

**No knob is privileged.** Every knob introduced in `anvil` is
subject to the same rule: its effect on generated output must be
empirically measurable, via the post-hoc metrics walker
(`src/metrics.rs`) and/or the live trace output (`--trace`). A
knob that exists but whose effect we cannot quantify is a knob
we cannot tell is working, redundant, or mis-specified.

Concretely, whenever a new knob lands:

1. A field in `Metrics` (or an existing metric) must capture the
   knob's intended effect.
2. The knob's section in this chapter must name the metric that
   measures it — see "Knob effectiveness map" at the bottom of
   the page.
3. A CLI spot-check (at default and at the boundary values 0.0 /
   1.0 / min / max) should show the metric shifting in the
   expected direction.

If none of the existing metrics captures the knob's effect, add
a metric. Landing a knob without its metric is a workflow
violation.

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

- `construction_strategy` — which strategy the generator uses to
  construct a module's internal logic. Values:
  - `sequential`: per-output cone recursion in declaration order.
  - `shuffled`: per-output cone recursion in a random permutation.
  - `interleaved`: output cones interleaved via a global frame queue.
  - `graph-first` (**default**): no per-output cone recursion; grow
    a gate pool of top-level units, drain flop D-cones with pool-only
    picks, then pick drive-roots from the pool.

  See `book/src/construction-strategies.md` for the full four-way
  comparison and rationale.
- `graph_first_pool_size` — target number of top-level units (gate /
  flop / comb-mux block) grown in the pool by the `graph-first`
  strategy. Default 32. Does not count internal primitive gates
  generated by comb-mux assembly or flop-mux assembly. Only consulted
  when `construction_strategy == graph-first`.

### Priority-encoder block

- `priority_encoder_prob` — per-emission probability of a priority-
  encoder block at a compatible target width. Default `0.05`. Skip
  the block (and fall through to the usual gate path) when no N in
  `[min_mux_arms, max_mux_arms]` yields `ceil_log2(N) == target_width`.
- N range reuses the block-level `min_mux_arms` / `max_mux_arms`.

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

### Coefficient motif (linear combinations)

- `coefficient_prob` — per-op probability (when `build_cone` picks
  `Add`, `Sub`, or `Mul`) of emitting the linear-combination
  compound form instead of a standard operator. Default `0.2`.
  Shapes: Add `y = Σ sᵢ·cᵢ`, Sub `y = s1·c1 − s2·c2 − … − sn·cn`,
  Mul `y = c · s1 · s2 · … · sn`. See
  `book/src/structural-rules.md` "Roles of constants in RTL".
- `min_coefficient / max_coefficient` — strictly-positive integer
  range for the drawn coefficients. Defaults `1, 15`.

### Shift-amount motif

- `const_shift_amount_prob` — per-shift probability that `Shl`/`Shr`
  emits `value << const` / `value >> const` (constant amount) instead
  of `value << signal` (barrel shifter). Default `0.8` — real designs
  overwhelmingly use constant amounts.
- `min_shift_amount / max_shift_amount` — range for the drawn
  constant shift amount, clamped to `[0, W-1]` for a W-bit value.
  Defaults `0, 7`.
- `gate_shift_weight` — relative weight for the shifts bucket (Shl,
  Shr) in `pick_gate`. Default `1`. Shifts are disabled at width 1.

### Comparand motif

- `const_comparand_prob` — per-comparison probability the RHS is a
  constant literal instead of a recursive signal cone. Additive to
  signal-vs-signal comparisons (the default remains signal-vs-signal
  when the coin doesn't fire). Default `0.3`. LHS is always a signal.
- `min_comparand / max_comparand` — range for the constant RHS,
  clamped to `[0, 2^K - 1]` for the chosen internal operand width K.
  Defaults `0, 255`.

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

### AST uniqueness / duplication

- `factorization_level` — coarse dial along the sharing chain:
  `none → cse → operand-unique → commutative → associative →
  constant-fold → peephole → e-graph`. Default `e-graph`
  (theoretical ceiling; activates every layer implemented today).
  Each step implies all lower ones. Implemented layers land
  progressively without requiring a config change. See Rule 21c.

- `max_ast_instances` — maximum number of times a given AST
  (`(op, operands, width)` for gates, `(width, value)` for
  constants) may be materialised as a named node in one module.
  Default `1` = strict uniqueness (construction-time CSE). See
  Rule 21 in `book/src/structural-rules.md`. Values:
  - `1` (default): one AST = one node. No `eq_0` / `eq_9` computing
    the same thing.
  - `K > 1`: up to K copies of the same AST before callers are
    routed to the last-created instance.
  - `u32::MAX`: effectively disables dedup.

- `mux_arm_duplication_rate` — probability that an arm of an N-to-1
  mux may be connected to a data signal already used by another
  arm of the same mux. Range `[0.0, 1.0]`. Default `0.0` = all
  arms distinct (best-effort). `1.0` = no constraint. See Rule 22
  in `book/src/structural-rules.md`.

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

## Knob effectiveness map

Per the measurement doctrine above, every knob has at least one
metric that captures its effect. The table below is the contract:
grep the metric, vary the knob across its range, confirm the
metric moves in the expected direction. If it doesn't, the knob
is either broken, masked by another knob, or redundant — all of
which are bugs worth investigating.

| Knob                          | Metric(s) that measure effectiveness                       |
|-------------------------------|------------------------------------------------------------|
| `min_inputs` / `max_inputs`   | `num_inputs`                                               |
| `min_outputs` / `max_outputs` | `num_outputs`                                              |
| `min_width` / `max_width`     | port widths (in `manifest.json`), `constants_by_width`     |
| `max_depth`                   | (pending live counter); `max_fanout` as proxy today        |
| `flop_prob`                   | `num_flops` / `num_gates`                                  |
| `max_flops_per_module`        | `num_flops` saturation near the cap                        |
| `min_mux_arms` / `max_mux_arms` | one-hot `MuxArm` list lengths (via flop-shape metric)    |
| `flop_qfeedback_prob`         | `flops_qfeedback` / `flops_zero_default`                   |
| `flop_mux_encoding_prob`      | `flops_mux_encoded` / `flops_mux_one_hot`                  |
| `share_prob`                  | `num_shared_nodes`, `max_fanout`, `avg_fanout`             |
| `construction_strategy`       | all structural metrics shift — compare runs at same seed   |
| `graph_first_pool_size`       | `num_gates` (GraphFirst only)                              |
| `priority_encoder_prob`       | per-kind `mux` chains — today indistinguishable; pending block metric |
| `comb_mux_prob`               | `num_muxes_2to1` (includes encoded chains)                 |
| `comb_mux_encoding_prob`      | (pending: flop-shape metric doesn't cover comb muxes yet)  |
| `coefficient_prob`            | `gates_by_kind["mul"]` uptick (each coefficient → `Mul`)   |
| `min_coefficient` / `max_coefficient` | `constants_by_width` distribution                  |
| `const_shift_amount_prob`     | `gates_by_kind["shl"]` / `gates_by_kind["shr"]` constants  |
| `gate_shift_weight`           | `gates_by_kind["shl"]` + `gates_by_kind["shr"]`            |
| `const_comparand_prob`        | `gates_by_kind["eq"]` with width-1 constants               |
| `min_comparand` / `max_comparand` | `constants_by_width` at the comparison operand width   |
| `min_gate_arity` / `max_gate_arity` | `max_operand_count_by_kind["add"]` / `["mul"]` / `["and"]` / `["or"]` / `["xor"]`; full histogram in `gate_operand_count_histogram` |
| `constant_prob`               | `num_constants` / `num_gates`                              |
| `gate_*_weight`               | `gates_by_kind` bucket shares                              |
| `max_ast_instances`           | `max_gate_ast_multiplicity`, `max_constant_ast_multiplicity` |
| `mux_arm_duplication_rate`    | `num_muxes_degenerate`                                     |

Entries marked *pending* are knobs whose effect is not yet
captured by a structural metric. Each is a known gap — either
the metric will be added in a future slice, or the knob will be
shown not to need a dedicated metric because its effect is
subsumed by one already in the table.
