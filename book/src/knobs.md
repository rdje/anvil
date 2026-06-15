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
| `--construction-strategy`     | interleaved | How module logic is constructed (see below).     |
| `--factorization-level`       | e-graph  | Dial along the sharing chain: none / cse / operand-unique / commutative / associative / constant-fold / peephole / e-graph. |
| `--max-ast-instances`         | 1        | How many times one AST may be named (1 = strict CSE). |
| `--mux-arm-duplication-rate`  | 0.0      | Probability N-to-1 mux arms may share the same signal. |
| `--operand-duplication-rate`  | 0.0      | Probability `Add`/`Mul` operand lists may repeat (0.0 = strict). |
| `--trace <level>`             | none     | Generation trace: none / low / medium / high / debug. |
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

### Snapshot guard-rails

You don't have to take "byte-identical, forever" on faith — the repo
proves it. `tests/snapshots.rs` pins the emitted SystemVerilog for a
handful of canonical `(seed, config)` shapes (a plain leaf, recursive
library/on-demand hierarchies, sibling- and parent-composed routes,
and a deduplicated design) using
[`insta`](https://insta.rs) snapshots stored under
`tests/snapshots/`. They run as part of `cargo test`.

This is purely a safety net for *contributors* — it never affects the
tool you run. If a code change accidentally perturbs output (a stray
`HashMap` iteration, an RNG re-seed, a planner or emit reorder), a
snapshot test fails and the change cannot land unnoticed. Changing a
snapshot is therefore a **deliberate** act, not an accident: you
review the diff and, only if the new output is intended, accept it
(with `cargo insta accept`/`cargo insta review`) in the same change
that caused it. There is nothing to do here as a *user* — generated
output for your `(seed, knobs)` is, and stays, stable.

## Knob taxonomy

Knobs fall into four categories:

### Structural knobs (shape)

Control the size and topology of generated modules.

- `min_inputs / max_inputs` — primary input port count range.
- `min_outputs / max_outputs` — primary output port count range.
- `min_width / max_width` — port and internal-wire width range.
- `max_depth` — maximum cone recursion depth.
- `max_nodes_per_module` — per-module construction-time node budget.
  Sentinel `0` = unlimited (the default; byte-identical to the historical
  unbounded behaviour). When non-zero, cone construction stops opening new
  sub-cones once the module's node count reaches the budget — it steers to
  existing terminals (rules-first; it never truncates a finished cone), so
  a pathological seed/knob mix cannot grow one module without bound. A
  *soft* ceiling: a bounded number of terminal/adapter nodes may still
  close already-open cones past the budget. Its effect shows up in the
  `num_nodes` metric.
- `hierarchy_depth`, `num_leaf_modules`, `num_child_instances` —
  legacy exact depth-1 wrapper hierarchy controls (Phase 4).
- `min_hierarchy_depth`, `max_hierarchy_depth`,
  `min_child_instances_per_module`,
  `max_child_instances_per_module`, `child_instances_per_depth` —
  bounded recursive hierarchy controls (Phase 4).
- `max_parent_cone_instances_per_module` — per-parent helper-instance
  budget for hierarchy parent-cone sources (Phase 4).
- `hierarchy_module_dedup` — opt-in post-finalisation pass that collapses
  structurally-identical `Module` definitions in a `Design` to one
  survivor and, after a real merge, prunes module definitions no longer
  reachable from the top (Phase 4, hierarchy-aware identity).
  Config/library-only; no CLI flag.

### Process-safety governor (runtime, not generation)

These knobs guard `anvil`'s **own process memory** on a RAM-limited
host; they never change emitted RTL, so they sit apart from the
structural knobs above. They complement `scripts/ram_guard.sh`
(which guards *external* heavy jobs from the outside) by guarding the
generator from the inside — catching a huge-`--count` or
single-pathological-module balloon that a coarse external poll can
outrun.

- `max_rss_mb` — abort a `--out` run once this process's resident set
  (RSS) reaches this many MiB. Sentinel `0` = off (the default).
- `ram_abort_pct` — abort once host used RAM reaches this percentage
  (`1..=100`). Sentinel `0` = off. Mirrors `scripts/ram_guard.sh`'s
  reads (macOS `memory_pressure` "free percentage"; Linux
  `/proc/meminfo` `MemAvailable`).

When armed, the governor is sampled **between** finished
modules/designs (decline-to-start-more), never mid-cone — it stops the
run cleanly with a deterministic non-zero exit (code `99`, matching the
shell guard) and a stderr message naming the seed + effective knobs,
rather than mutilating a built module (which would emit invalid RTL and
break valid-by-construction). RSS is checked before host-%, since a
single-process balloon can outrun the host signal. Each OS read is
best-effort: an unreadable sample never aborts a healthy run.

> **Measurement-doctrine note.** The doctrine above ("every knob must
> have a metric that captures its effect on *generated output*") does
> not apply here: a process-safety governor has *no* output effect to
> measure — when it does nothing, output is byte-identical, and when it
> fires, there is no output at all past the abort. Its behaviour is
> proven instead by the pure decision-logic unit tests in
> `src/mem_guard.rs` (`evaluate`) plus the clean exit-`99` abort path.
> Both default to the off sentinel, so the default path is
> byte-identical and consumes RNG identically (the guard is never even
> sampled).

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
  forced-leaf decision points when a matching-width signal exists.
  `1.0` = always reuse that signal; `0.0` = never reuse it and emit a
  fresh constant instead. This is the leaf-level sharing knob; it is
  orthogonal to `share_prob`, which only applies at non-leaf
  recursion points.

### Construction strategy

- `construction_strategy` — which strategy the generator uses to
  construct a module's internal logic. Values:
  - `sequential`: per-output cone recursion in declaration order.
  - `shuffled`: per-output cone recursion in a random permutation.
  - `interleaved` (**default**): output cones interleaved via a global
    frame queue.
  - `graph-first`: deprecated alias for `interleaved`, retained for
    backward-compatible CLI/config parsing. The original speculative
    graph-first builder is retired.

  See `book/src/construction-strategies.md` for the full four-way
  comparison and rationale.
- `graph_first_pool_size` — legacy knob from the retired speculative
  graph-first builder. Retained for backward-compatible configs, but
  ignored by the current live interleaved/default path.

### Priority-encoder block

- `priority_encoder_prob` — per-emission probability of a priority-
  encoder block at a compatible target width. Default `0.05`. Skip
  the block (and fall through to the usual gate path) when no N in
  `[min_mux_arms, max_mux_arms]` yields `ceil_log2(N) == target_width`.
- N range reuses the block-level `min_mux_arms` / `max_mux_arms`.

### Case-mux block

- `case_mux_prob` — per-emission probability of a combinational
  `always_comb case (sel)` block. Default `0.05`. The block uses one
  encoded select bus, M data arms, and an explicit default-to-zero
  assignment.
- If the select expression is already constant by emission time, the
  emitter lowers the case mux to a continuous `assign` of the selected
  arm or default zero. Dynamic selectors still emit the procedural
  `always_comb case` surface.
- M (arm count) range reuses the block-level `min_mux_arms` /
  `max_mux_arms` knobs; select width is `ceil(log2(M))`.

### Casez-mux block

- `casez_mux_prob` — per-emission probability of a combinational
  `always_comb casez (sel)` block. Default `0.05`. The block uses one
  encoded select bus, M data arms, wildcard patterns, and an explicit
  default-to-zero assignment.
- Generation keeps the wildcard patterns non-overlapping by
  construction, so the surface stays a wildcarded mux motif rather than
  becoming an accidental priority chain.
- If the select expression is already constant by emission time, the
  emitter lowers the casez mux to a continuous `assign` of the first
  matching arm or default zero. Dynamic selectors still emit the
  procedural `always_comb casez` surface.

### Bounded for-fold block

- `for_fold_prob` — per-emission probability of a combinational
  statically bounded `always_comb` `for`-fold block. Default `0.05`.
  The block takes one packed source bus, initializes an accumulator,
  and folds fixed-width chunks with a static trip count.
- The fold kind today is one of `xor`, `or`, `and`, or `add`.
- The trip-count range reuses `min_gate_arity` / `max_gate_arity`; the
  generated packed source width is `trip_count * chunk_width`.
- If the packed source is already a constant by emission time, the
  emitter lowers the fold to a continuous `assign` of the folded
  literal. Dynamic sources still emit the procedural bounded-loop
  surface.

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
  no matching-width signal exists but a dep-bearing width-adapter
  source does. Default `0.1`. When this misses, `pick_terminal`
  adapts an existing source instead of minting a constant.
- `gate_*_weight` — relative weights for gate categories when picking
  a gate at a non-leaf recursion point. Defaults bitwise `3`, arith
  `2`, struct `1`, compare `1`, reduce `1`.
- Termination: there is no explicit `leaf_prob_growth` knob —
  `build_cone` uses a linear `depth / max_depth` ramp, forcing a leaf
  at `max_depth`.

### AST uniqueness / duplication

- `identity_mode` — coarse NodeId semantics switch:
  - `node-id` (default): NodeId means expression identity, which
    implies full factorization by definition. The factorization ladder
    is the current build's enforcement/proof-depth dial inside that
    doctrine, including the bounded semantic gate merge at `e-graph`
    and the post-drain endpoint-aware flop merge.
  - `relaxed`: disable the ladder entirely; every
    `intern_gate` / `intern_constant` call allocates a fresh
    `NodeId` even if `factorization_level` requests more.
  This is orthogonal to construction strategy. See Rule 21c.

- `factorization_level` — current-build dial along the sharing chain:
  `none → cse → operand-unique → commutative → associative →
  constant-fold → peephole → e-graph`. Default `e-graph`
  (theoretical ceiling; activates every layer implemented today)
  when `identity_mode = node-id`. Each step implies all lower
  ones. Lower rungs are weaker enforcement of the same doctrine, not a
  different definition of `node-id`. Implemented layers land
  progressively without requiring a config change. Today the `e-graph`
  rung includes a bounded semantic gate proof, not an unbounded solver:
  up to 12 endpoint-support bits and only while the candidate cone fits
  its node/work budget. See Rule 21c.

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

- `operand_duplication_rate` — probability that an operator gate's
  operand list may contain the same `NodeId` twice (applies to
  `Add` and `Mul`; `And`/`Or`/`Xor` are *always* strict regardless
  because duplicates collapse algebraically). Range `[0.0, 1.0]`.
  Default `0.0` = strict operand uniqueness for `Add`/`Mul`.
  `1.0` = duplicates unrestricted — opt in to exercise
  `x + x = 2x` / `x * x = x²` shapes in downstream tools. See
  `book/src/structural-rules.md` Rule 8 + Rule 21c.

### Hierarchy knobs (Phase 4+)

- `hierarchy_depth` — legacy exact hierarchy-depth knob. Today `0`
  keeps the leaf-only lane and `1` selects the legacy exact wrapper
  lane.
- `num_leaf_modules` — size of the pre-generated child library for the
  legacy exact depth-1 wrapper lane.
- `num_child_instances` — instantiated child count for the legacy exact
  depth-1 wrapper lane. Default `0` preserves the legacy exact-once
  behavior ("instantiate every generated leaf definition once"). Values
  below `num_leaf_modules` under-instantiate the library; larger values
  reuse child definitions.
- `min_hierarchy_depth`, `max_hierarchy_depth` — bounded recursive
  hierarchy depth range. In the current slice, ANVIL keeps every leaf
  depth inside `[min:max]` and can now mix shallow and deep branches in
  one tree when the interval is open and the structure allows it.
- `min_child_instances_per_module`,
  `max_child_instances_per_module` — bounded recursive child-instance
  range for each non-leaf module.
- `child_instances_per_depth` — optional repeated override keyed by
  parent depth (`DEPTH=MIN:MAX`). This layers on top of the bounded
  recursive fallback range, so depth `0` can be forced to one
  branching profile while depth `1` uses another.
- `hierarchy_child_source_mode` — explicit child-sourcing mode for both
  hierarchy lanes. `library` keeps a reusable child-definition pool.
  The current `on-demand` slice synthesizes one profiled child
  definition per planned instance slot against a parent-planned exact
  data-interface profile. Control ports stay structural and are not
  part of that profile.
- `hierarchy_sibling_route_prob` — probability that later child data
  inputs bind from earlier sibling instance outputs instead of always
  binding from parent-boundary inputs. When
  `hierarchy_parent_cone_instance_prob` also fires, the direct
  unregistered route can allocate a helper child and bind from its
  output instead. Range `[0.0, 1.0]`. Default `0.35`. Direct sibling
  routing is combinational.
- `hierarchy_registered_sibling_route_prob` — probability that later
  child data inputs bind through a local parent flop. The default D
  source is an earlier sibling instance output. Once earlier parent
  flops exist from prior registered sibling routes, later routes can
  also use an earlier parent-local Q as the D source, creating a
  multi-stage registered sibling chain. When
  `hierarchy_parent_cone_instance_prob` also fires, the D source can be
  a helper instance output instead. Range `[0.0, 1.0]`. Default `0.0`,
  so the registered child-to-child axis is opt-in and remains distinct
  from the direct combinational sibling route.
- `hierarchy_registered_child_input_cone_prob` — probability that
  later child data inputs bind through parent-local combinational logic
  over already-available parent sources and then one local parent flop.
  Those sources can include parent data inputs, earlier sibling
  outputs, and earlier parent-side route gates. When parent data inputs
  and sibling outputs are both live, this route can mix both supports
  in the flop D cone. When earlier parent flops are live, later routes
  can also chain through those Qs before allocating the next parent
  flop. When `hierarchy_parent_cone_instance_prob` also fires, the D
  cone can include a parent-cone helper output. Range `[0.0, 1.0]`.
  Default `0.0`, so the registered parent-composed route is opt-in and
  remains distinct from direct registered sibling routing.
- `hierarchy_child_input_cone_prob` — probability that a child data
  input binds through a parent-local combinational cone instead of a
  direct parent-port or sibling-output route. The cone may use
  already-available parent sources: parent data inputs, earlier sibling
  instance outputs, and earlier parent-side route gates. When
  `hierarchy_parent_cone_instance_prob` and `hierarchy_parent_flop_prob`
  both fire, a required helper source can be registered into
  parent-local state first and then consumed by the parent-composed
  child-input logic. Range `[0.0, 1.0]`. Default `0.35`.
- `hierarchy_parent_cone_instance_prob` — probability that a
  parent-composed child-input cone, direct sibling route, direct
  registered sibling route, registered child-input D cone, or
  parent-output cone instantiates one helper child as an internal
  parent-cone source. The helper is separate
  from planned child slots, and its outputs can feed later child inputs
  or parent outputs through parent logic or one parent-local flop.
  Parent-output cones can consume helper sources directly or, when
  `hierarchy_parent_flop_prob` is enabled, through parent-local state.
  Parent-composed child-input cones can also consume a helper source
  through parent-local state while keeping the final child binding
  unregistered parent logic.
  Range `[0.0, 1.0]`. Default `0.0`, so helper instantiation is opt-in.
- `max_parent_cone_instances_per_module` — maximum number of helper
  child instances one hierarchy parent may instantiate as parent-cone
  sources. Default `1` preserves the first helper slice; `0` disables
  helper insertion even when `hierarchy_parent_cone_instance_prob`
  fires. In recursive designs this is a per-parent budget, so the
  hierarchy-wide helper count can exceed this value across multiple
  internal modules.
- `hierarchy_parent_flop_prob` — probability that parent-side hierarchy
  cones may emit local parent flops. This applies to parent output
  cones, parent-output helper routes, and parent-composed child-input
  cones. Range `[0.0, 1.0]`.
  Default `0.0`, so hierarchy remains combinational unless state is
  explicitly requested.
- `hierarchy_module_dedup` — opt-in `bool`, default `false`. When
  `true`, the generator runs the post-finalisation module-dedup pass
  (`src/ir/dedup.rs`) after `generate_design` has assembled and
  finalised every `Module`. The pass groups `Design::modules` by the
  same canonical FNV-1a structural signature recorded in
  `DesignMetrics.canonical_module_signatures`, keeps one survivor per
  group (the lexicographically-smallest module name; the top module is
  never merged away), rewrites every `Instance.module` reference in the
  surviving modules to point at the survivor, and drops the merged-away
  definitions. If at least one merge occurred, it then prunes module
  definitions that were reachable before dedup but are no longer
  reachable from the design top after the instance rewrites. A no-merge
  run preserves existing under-instantiated library definitions, and
  pre-existing unreferenced modules are not pruned merely because a
  separate merge happened. This extends the doctrine *"NodeId = identity of an
  expression"* up one level to *"ModuleId = identity of a hierarchical
  module template"*: structurally-identical `Module`s collapse the same
  way structurally-identical expressions already share a `NodeId`. It
  is purely structural — two modules that compute the same function via
  different gate sequences stay distinct. Default `false` preserves
  today's behaviour exactly; this knob never retires an existing mode.

  This knob is **config/library-only — there is no `--hierarchy-module-dedup`
  CLI flag**. Set it through a `Config` value (library use) or a config
  file, and confirm it with `anvil --dump-config`. Worked before/after,
  using the tight leaf constraints that make every library leaf
  structurally identical:

  ```rust,ignore
  use anvil::{Config, Generator, metrics};

  // Four library leaves, all collapsed to one canonical shape by the
  // 1-in / 1-out / width-1 / max_depth-1 constraints.
  let base = Config {
      seed: 42,
      hierarchy_depth: 1,
      num_leaf_modules: 4,
      num_child_instances: 4,
      min_inputs: 1, max_inputs: 1,
      min_outputs: 1, max_outputs: 1,
      min_width: 1, max_width: 1,
      max_depth: 1,
      terminal_reuse_prob: 1.0,
      constant_prob: 0.0,
      ..Config::default()
  };

  // Dedup OFF (default): the planner emits structural duplicates.
  let off = metrics::compute_design(&Generator::new(base.clone()).generate_design());
  assert!(off.num_structurally_duplicate_module_pairs > 0);

  // Dedup ON: duplicates collapse to a single survivor + top.
  let mut on_cfg = base;
  on_cfg.hierarchy_module_dedup = true;
  let on = metrics::compute_design(&Generator::new(on_cfg).generate_design());
  assert_eq!(on.num_structurally_duplicate_module_pairs, 0);
  assert!(on.num_modules < off.num_modules);          // fewer modules
  assert_eq!(on.num_distinct_module_signatures, on.num_modules); // all unique
  ```

  The emitted `Design` stays valid by construction: instance references
  are rewritten before the merged definitions are dropped, so
  `ir::validate::validate_design` still passes. Repo-owned proof:
  `module_dedup_pass_collapses_structurally_duplicate_modules` in
  `tests/pipeline.rs` plus the `phase4_hier1_module_dedup_active`
  matrix scenario (the `phase4_hier1_structurally_duplicate_modules`
  baseline stays in the bank with dedup off so the before/after
  comparison is visible directly in the gate output).
- `hierarchy_semantic_module_dedup` — opt-in `bool`, default `false`.
  This is a second pass, separate from `hierarchy_module_dedup`. It
  groups supported non-top pure-combinational modules by a bounded
  whole-module truth-table proof instead of by structural signature.
  The pass is active only when `identity_mode = node-id` and the
  effective `factorization_level` is `e-graph`; `identity_mode =
  relaxed` keeps it inert even if the knob is true. The current proof
  boundary is intentionally narrow:

  - no flops, memories, or FSMs locally or through descendants;
  - no parameterized or aggregate-projected module templates;
  - identical emitted data input and output interfaces by `(PortId,
    width)`;
  - total emitted input support <= 12 bits;
  - reachable output cone <= 128 nodes and inside the work budget; and
  - output widths <= 128 bits.

  The supported classes are instance-free pure-combinational modules
  and pure-combinational hierarchy wrappers with at most 8 child
  instances, where every child is also inside the proof boundary and
  every instance has concrete, non-parameterized bindings. Leaf modules
  and wrappers stay in separate proof classes, and a merge group is
  skipped if any member is an ancestor or descendant of another member.
  Those two rules prevent semantic dedup from flattening hierarchy or
  creating a hierarchy cycle through an instance rewrite.

  Within that boundary, semantically equal but structurally different
  modules can merge, such as a direct pass-through output and the same
  output computed through two `Not` gates, or two pure-combinational
  wrappers whose children compute the same bounded behavior. Outside
  that boundary, the pass skips the module; it does not degrade into a
  guess.

  This knob is **config/library-only — there is no
  `--hierarchy-semantic-module-dedup` CLI flag**. Set it through a
  `Config` value or a config file:

  ```rust,ignore
  use anvil::{Config, Generator, metrics};
  use anvil::config::{FactorizationLevel, IdentityMode};

  let mut cfg = Config {
      seed: 42,
      hierarchy_depth: 1,
      num_leaf_modules: 4,
      num_child_instances: 4,
      min_inputs: 1, max_inputs: 1,
      min_outputs: 1, max_outputs: 1,
      min_width: 1, max_width: 1,
      max_depth: 1,
      terminal_reuse_prob: 1.0,
      constant_prob: 0.0,
      identity_mode: IdentityMode::NodeId,
      factorization_level: FactorizationLevel::EGraph,
      ..Config::default()
  };

  let off = metrics::compute_design(&Generator::new(cfg.clone()).generate_design());
  assert!(off.num_semantically_duplicate_module_pairs > 0);

  cfg.hierarchy_semantic_module_dedup = true;
  let on = metrics::compute_design(&Generator::new(cfg).generate_design());
  assert_eq!(on.num_semantically_duplicate_module_pairs, 0);
  assert!(on.num_modules < off.num_modules);
  ```

  The related design metrics are
  `DesignMetrics.semantic_module_signatures` (one compact proof hash per
  module, `null` for unsupported modules) and
  `num_semantically_duplicate_module_pairs` (proof-equal pairs still
  present after any enabled dedup pass).
- The legacy exact wrapper knobs and the bounded recursive range knobs
  are intentionally **mutually exclusive**. They are two different
  planning lanes, not shorthand for the same behavior.
- `library_prob` — internal future probabilistic dial for a later
  mixed-sourcing planner. It is not the current user-facing control
  surface; current HEAD uses the explicit
  `hierarchy_child_source_mode` axis instead.

## Knob defaults

The canonical source of truth is `Config::default()` in
`src/config.rs`. Run `anvil --dump-config` to see the effective
knob set for your build.

```rust,ignore
Config {
    seed: 0,
    // Structure
    min_inputs: 2,  max_inputs: 8,
    min_outputs: 1, max_outputs: 4,
    min_width: 1,   max_width: 32,
    max_depth: 6,
    max_nodes_per_module: 0,   // 0 = unlimited (opt-in node budget)
    // Process-safety governor (runtime, not generation)
    max_rss_mb: 0,             // 0 = off (opt-in process-RSS abort, MiB)
    ram_abort_pct: 0,          // 0 = off (opt-in host used-RAM% abort)
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
    // Operator arity
    min_gate_arity: 2, max_gate_arity: 4,
    // Mix
    constant_prob: 0.1,
    gate_bitwise_weight: 3,
    gate_arith_weight:   2,
    gate_struct_weight:  1,
    gate_compare_weight: 1,
    gate_reduce_weight:  1,
    // Coefficient motif (linear combinations)
    coefficient_prob: 0.2,
    min_coefficient: 1, max_coefficient: 15,
    // Shift motif
    const_shift_amount_prob: 0.8,
    min_shift_amount: 0, max_shift_amount: 7,
    gate_shift_weight: 1,
    // Comparand motif
    const_comparand_prob: 0.3,
    min_comparand: 0, max_comparand: 255,
    // Blocks
    priority_encoder_prob: 0.05,
    comb_mux_prob: 0.1,
    comb_mux_encoding_prob: 0.5,
    // Construction strategy
    construction_strategy: ConstructionStrategy::Interleaved,
    identity_mode: IdentityMode::NodeId,
    graph_first_pool_size: 32,  // legacy; GraphFirst aliased to Interleaved
    // Factorization ladder (default request: e-graph, whose
    // bounded semantic fragment is now live)
    factorization_level: FactorizationLevel::EGraph,
    max_ast_instances: 1,
    mux_arm_duplication_rate: 0.0,
    operand_duplication_rate: 0.0,
    // Hierarchy (Phase 4+)
    hierarchy_depth: 0,
    num_leaf_modules: 0,
    num_child_instances: 0,
    hierarchy_child_source_mode: HierarchyChildSourceMode::Library,
    hierarchy_sibling_route_prob: 0.35,
    hierarchy_registered_sibling_route_prob: 0.0,
    hierarchy_registered_child_input_cone_prob: 0.0,
    hierarchy_child_input_cone_prob: 0.35,
    hierarchy_parent_cone_instance_prob: 0.0,
    max_parent_cone_instances_per_module: 1,
    hierarchy_parent_flop_prob: 0.0,
    hierarchy_module_dedup: false,
    hierarchy_semantic_module_dedup: false,
    multi_clock_prob: 0.0,
    cdc_synchronizer_stages: 2,
    min_hierarchy_depth: 0,
    max_hierarchy_depth: 0,
    min_child_instances_per_module: 0,
    max_child_instances_per_module: 0,
    child_instances_per_module_by_depth: {},
    library_prob: 0.5,
}
```

## CLI coverage

Every motif knob that affects the generator output has a dedicated
CLI flag, so all combinations are reachable without writing a config
file. The canonical list comes from `anvil --help`; the snapshot below
is accurate as of this commit.

### Run control
```text
--seed, --count, --out, --config, --dump-config
--trace <none|low|medium|high|debug>, --trace-file <path>
--metrics
--help, --version
```

### Structure
```text
--min-inputs, --max-inputs
--min-outputs, --max-outputs
--min-width, --max-width
--max-depth
```

### Memory governor (process-safety; default off)
```text
--max-rss-mb <MiB>        # abort an --out run above this process RSS
--ram-abort-pct <1..=100> # abort an --out run above this host used-RAM%
```

### Sequential
```text
--flop-prob, --max-flops-per-module
--min-mux-arms, --max-mux-arms
--flop-qfeedback-prob, --flop-mux-encoding-prob
```

### Sharing
```text
--share-prob
--terminal-reuse-prob
```

### Operator arity (N-ary for And/Or/Xor/Add/Mul)
```text
--min-gate-arity, --max-gate-arity
```

### Coefficient motif (linear combinations)
```text
--coefficient-prob
--min-coefficient, --max-coefficient
```

### Shift motif
```text
--const-shift-amount-prob
--min-shift-amount, --max-shift-amount
--gate-shift-weight
```

### Comparand motif
```text
--const-comparand-prob
--min-comparand, --max-comparand
```

### Gate mix and leaf termination
```text
--constant-prob
--gate-bitwise-weight
--gate-arith-weight
--gate-struct-weight
--gate-compare-weight
--gate-reduce-weight
```

### Blocks
```text
--priority-encoder-prob
--case-mux-prob, --casez-mux-prob
--for-fold-prob
--comb-mux-prob, --comb-mux-encoding-prob
```

### Construction strategy
```text
--construction-strategy <sequential|shuffled|interleaved|graph-first>
--graph-first-pool-size
```

### Identity / factorization
```text
--identity-mode <node-id|relaxed>
--factorization-level <none|cse|operand-unique|commutative|associative|constant-fold|peephole|e-graph>
--full-factorization
--no-full-factorization
--max-ast-instances
--mux-arm-duplication-rate
--operand-duplication-rate
```

### Hierarchy
```text
--hierarchy-depth
--num-leaf-modules
--num-child-instances
--hierarchy-child-source-mode <library|on-demand>
--hierarchy-sibling-route-prob
--hierarchy-registered-sibling-route-prob
--hierarchy-registered-child-input-cone-prob
--hierarchy-child-input-cone-prob
--hierarchy-parent-cone-instance-prob
--max-parent-cone-instances-per-module
--hierarchy-parent-flop-prob
--min-hierarchy-depth, --max-hierarchy-depth
--min-child-instances-per-module, --max-child-instances-per-module
--child-instances-per-depth DEPTH=MIN:MAX
```

### `tool_matrix` auxiliary binary
```text
--out
--base-seed
--modules-per-scenario
--list-scenarios
--fail-on-coverage-gap
--resume
--phase1-gate
--phase2-share-gate
--phase3-structured-gate
--phase4-hierarchy-gate
--signoff-knob-sweep-gate
--skip-verilator, --skip-yosys
--verilator-bin, --yosys-bin
--yosys-mode <without-abc|with-abc|both>
--iverilog-compile
--iverilog-bin
--help, --version
```

`tool_matrix` is not the generator itself; it is the repo-owned corpus
and downstream-tool harness. Its flags control scenario selection,
resume/checkpoint behavior, and which external tools are invoked.
`--iverilog-compile` is an optional Icarus Verilog acceptance column:
it shells `iverilog -g2012` for each emitted artifact and treats
warnings as failures. It does not run a testbench; use `--diff-sim`
for cross-simulator trace agreement.

### Not yet exposed via CLI (reachable via `--config FILE`)
- `use_async_reset` — unused (flops are always async-reset by discipline).
- Hierarchy field `library_prob` — future probabilistic mixed-sourcing dial for later Phase 4+ work.
- `max_nodes_per_module` — per-module node budget (default `0` =
  unlimited). Set it to a positive cap to bound peak per-module memory on
  huge or pathological workloads; see the structural-knobs entry above.
- `width_parameterization_prob` (Phase 5, default `0.0`) — per-module
  probability that a finalized width-homogeneous leaf is emitted with
  a width `parameter` and instantiated with per-instance `#(.W(v))`
  overrides. Default-off is byte-identical.
- `aggregate_prob` (Phase 5b, default `0.0`) — per-module probability
  that a finalized **non-instantiated** module's contiguous
  same-direction data ports are folded into one packed-`struct`
  emitter projection (`typedef struct packed` + one aggregate port +
  boundary alias wires/assigns). A purely emitter-surface regrouping:
  the flat IR body, validators, CSE and the dedup signature are
  untouched (a module and its projected twin dedup-collapse).
  Default-off is byte-identical for fixed seeds. Selects `struct packed`
  by default; see `aggregate_array_prob` for the uniform-width
  packed-array variant. Skips Phase 5 parameterized modules. See
  `book/src/ir.md` "Synthesizable aggregates".
- `aggregate_array_prob` (default `0.0`) — conditional on a module
  being aggregate-projected (the `aggregate_prob` roll fired), the
  probability that a **uniform-width** projected group is rendered as a
  packed *array* (`typedef logic [N-1:0][W-1:0] <name>;` with
  positional `<port>[i]` boundary aliases) instead of a packed
  `struct`. It only takes effect when **every** projected group is
  internally same-width; a mixed-width group falls the whole layout
  back to `struct packed`. A packed array is LRM-bit-equivalent to the
  field concatenation, so this is the same kind of faithful,
  semantically-empty regrouping as the struct projection — the flat IR
  body, validators, CSE and the dedup signature are untouched.
  `default = 0.0` keeps every output byte-identical (always
  `struct packed`). Delivered and proven downstream-clean: generated
  packed-array designs pass Verilator `--lint-only` and Yosys
  `synth -noabc; check`. See `book/src/ir.md` "Synthesizable
  aggregates".
- `memory_prob` (Phase 6, default `0.0`) — per-module probability
  that the free-standing single-module lane builds a rules-first
  **inferrable-memory** leaf instead of an ordinary leaf: a
  first-class `Memory` block (shared `clk`, `we`/`waddr`/`wdata`
  [+ independent `raddr` for `SimpleDualPort`]) whose registered
  read is an opaque `Node::MemRead` leaf — never CSE'd/factorized;
  the emitter renders the synchronous-write / registered-read
  template Yosys infers as `$mem_v2`. It is reset-less by design:
  reset-all array contents are not warning-clean in Yosys for this
  memory-inference lane, so memory state remains instance-local.
  Mutually exclusive with the
  Phase 5 width-parameterization and Phase 6 FSM lanes; default-off
  is byte-identical. **Delivered (Phase 6, 2026-05-18)**, proven
  downstream-clean on the `Phase4Hierarchy` gate. See
  `book/src/ir.md` "Unpacked arrays".
- `fsm_prob` (Phase 6, default `0.0`) — per-module probability that
  the single-module lane builds a rules-first **generated-encoding
  Moore FSM** block (encoding-derived `localparam` state constants
  + an async-reset state register + `always_comb` next-state /
  Moore-output `case`s) whose output is the opaque `Node::FsmOut`
  leaf. Encoding (binary / one-hot / gray) is rolled per module;
  mutually exclusive with the param/memory lanes; default-off is
  byte-identical. **Delivered (Phase 6, 2026-05-20)**, proven
  downstream-clean against the banked `Phase4Hierarchy` gate
  (closing artifact `/tmp/anvil-tool-matrix-phase6-fsm-p1`: 222
  scenarios / 888 designs, `coverage_gaps = []`, 888/0 Verilator +
  both Yosys, `saw_fsm_design = true`). **This closes Phase 6
  (advanced motifs).** Mealy outputs are the recorded post-closure
  extension and are not emitted today.
- `multi_clock_prob` (default `0.0`) — per-module probability that a
  finalized module is promoted to K=2 with a by-construction CDC
  synchronizer on one 1-bit flop-driven output. Config/library-only;
  default-off is byte-identical.
- `cdc_synchronizer_stages` (default `2`) — number of destination-domain
  flops in that generated CDC synchronizer chain. `2` is the original
  2-flop primitive; values `>= 3` opt into the N-flop synchronizer
  primitive. It is meaningful only when `multi_clock_prob` fires and the
  module has an eligible 1-bit flop-driven output. Config/library-only.

## Knob serialization

Knobs are a `serde`-derived struct. Dump the effective config, then
load it back (self-contained — paste and run as one block):

```bash
cargo run --release -- --dump-config > knobs.json
cargo run --release -- --config knobs.json --seed 42
```

Writing the effective knobs back out (the merge of defaults and CLI
overrides) lets users save and replay configurations:

```bash
cargo run --release -- --seed 42 --max-depth 8 --dump-config > my-knobs.json
cargo run --release -- --config my-knobs.json --seed 42  # byte-identical to previous
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
| `max_depth`                   | `max_gate_depth`, `gate_depth_histogram` — monotone in the knob (typically 10–100× because block-assembly helpers expand each recursion level into multiple gate layers). |
| `max_nodes_per_module`        | `num_nodes` — a non-zero budget caps it (soft ceiling); `0` (default) = unlimited |
| `flop_prob`                   | `num_flops` / `num_gates`                                  |
| `max_flops_per_module`        | `num_flops` saturation near the cap                        |
| `min_mux_arms` / `max_mux_arms` | one-hot `MuxArm` list lengths (via flop-shape metric)    |
| `flop_qfeedback_prob`         | `flops_qfeedback` / `flops_zero_default`                   |
| `flop_mux_encoding_prob`      | `flops_mux_encoded` / `flops_mux_one_hot`                  |
| `share_prob`                  | `num_shared_nodes`, `max_fanout`, `avg_fanout`             |
| `construction_strategy`       | all structural metrics shift — compare runs at same seed   |
| `graph_first_pool_size`       | legacy knob; no effect on the current live path            |
| `priority_encoder_prob`       | `num_priority_encoder_blocks` — live counter, monotone in the knob |
| `case_mux_prob`               | `num_case_mux_blocks` — live counter, monotone in the knob |
| `casez_mux_prob`              | `num_casez_mux_blocks` — live counter, monotone in the knob |
| `for_fold_prob`               | `num_for_fold_blocks` — live counter, monotone in the knob |
| `comb_mux_prob`               | `num_muxes_2to1`, `num_comb_muxes_one_hot` + `num_comb_muxes_encoded` (sum) |
| `comb_mux_encoding_prob`      | `num_comb_muxes_encoded / (num_comb_muxes_one_hot + num_comb_muxes_encoded)` ratio — converges to the knob over large sweeps |
| `coefficient_prob`            | `gates_by_kind["mul"]` uptick (each coefficient → `Mul`)   |
| `min_coefficient` / `max_coefficient` | `constants_by_width` distribution                  |
| `const_shift_amount_prob`     | `gates_by_kind["shl"]` / `gates_by_kind["shr"]` constants  |
| `gate_shift_weight`           | `gates_by_kind["shl"]` + `gates_by_kind["shr"]`            |
| `const_comparand_prob`        | `gates_by_kind["eq"]` with width-1 constants               |
| `min_comparand` / `max_comparand` | `constants_by_width` at the comparison operand width   |
| `min_gate_arity` / `max_gate_arity` | `max_operand_count_by_kind["add"]` / `["mul"]` / `["and"]` / `["or"]` / `["xor"]`; full histogram in `gate_operand_count_histogram` |
| `terminal_reuse_prob`         | `knob_roll_attempts["terminal_reuse_prob"]`, `knob_roll_fires["terminal_reuse_prob"]`; higher values raise exact-width leaf reuse |
| `constant_prob`               | `num_constants` / `num_gates`                              |
| `gate_*_weight`               | `gates_by_kind` bucket shares                              |
| `max_ast_instances`           | `max_gate_ast_multiplicity`, `max_constant_ast_multiplicity` |
| `mux_arm_duplication_rate`    | `num_muxes_degenerate`; matrix `saw_mux_arm_duplication` (`--signoff-knob-sweep-gate`) |
| `operand_duplication_rate`    | `num_operator_gates_with_duplicate_operands` — count of `Add`/`Mul` gates with a repeated operand slot (0 at rate 0.0, rises with the knob); matrix `saw_operand_duplication` (`--signoff-knob-sweep-gate`) |
| `identity_mode`               | `max_gate_ast_multiplicity`, `max_constant_ast_multiplicity`, `num_gates`, `semantic_gates_merged`, and `flops_merged`: `relaxed` disables the ladder entirely, so multiplicities rise, raw gate count rises, and both post-construction semantic merges drop to 0 |
| `factorization_level`         | `num_gates` (typically shrinks as the ladder rises toward `e-graph`); `nested_associative_operand_count` — residual flattening opportunity at / above `associative`, decreasing once that layer lands; `flops_merged` becomes eligible at `cse` and above; `semantic_gates_merged` becomes eligible at `e-graph` |
| `hierarchy_sibling_route_prob` | `child_input_bindings_from_instance_outputs`, `child_input_bindings_from_mixed_support`, `instance_output_child_input_binding_fraction`, `top_instance_output_child_input_binding_fraction` |
| `hierarchy_registered_sibling_route_prob` | `child_input_bindings_from_registered_instance_outputs`, `top_child_input_bindings_from_registered_instance_outputs`, `registered_instance_output_child_input_binding_fraction`, `top_registered_instance_output_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_instance_outputs`, `top_child_input_bindings_from_registered_multistage_instance_outputs`, `registered_multistage_instance_output_child_input_binding_fraction`, `top_registered_multistage_instance_output_child_input_binding_fraction`, `child_input_bindings_from_registered_parent_cone_instances`, `top_child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_cone_instances`, `registered_multistage_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_parent_flops`, `hierarchy_parent_local_flops` |
| `hierarchy_registered_child_input_cone_prob` | `child_input_bindings_from_registered_parent_composed_logic`, `top_child_input_bindings_from_registered_parent_composed_logic`, `registered_parent_composed_child_input_binding_fraction`, `top_registered_parent_composed_child_input_binding_fraction`, `child_input_bindings_from_registered_mixed_support`, `top_child_input_bindings_from_registered_mixed_support`, `registered_mixed_support_child_input_binding_fraction`, `top_registered_mixed_support_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_composed_logic`, `top_child_input_bindings_from_registered_multistage_parent_composed_logic`, `registered_multistage_parent_composed_child_input_binding_fraction`, `top_registered_multistage_parent_composed_child_input_binding_fraction`, `child_input_bindings_from_registered_parent_cone_instances`, `top_child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_parent_flops`, `hierarchy_parent_local_flops` |
| `hierarchy_child_input_cone_prob` | `child_input_bindings_from_parent_composed_logic`, `parent_composed_child_input_binding_fraction`, `top_parent_composed_child_input_binding_fraction`, `child_input_bindings_from_parent_cone_instances_through_parent_flops`, `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`, `parent_cone_instance_flop_child_input_binding_fraction`, `top_parent_cone_instance_flop_child_input_binding_fraction` |
| `hierarchy_parent_cone_instance_prob` | `top_parent_cone_instances`, `hierarchy_parent_cone_instances`, `max_parent_cone_instances_per_internal_module`, `child_input_bindings_from_parent_cone_instances`, `top_child_input_bindings_from_parent_cone_instances`, `parent_cone_instance_child_input_binding_fraction`, `top_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_parent_cone_instances_through_parent_flops`, `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`, `parent_cone_instance_flop_child_input_binding_fraction`, `top_parent_cone_instance_flop_child_input_binding_fraction`, `child_input_bindings_from_registered_parent_cone_instances`, `top_child_input_bindings_from_registered_parent_cone_instances`, `registered_parent_cone_instance_child_input_binding_fraction`, `top_registered_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_cone_instances`, `registered_multistage_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_cone_instance_child_input_binding_fraction`, `child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `top_child_input_bindings_from_registered_multistage_parent_composed_parent_cone_instances`, `registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`, `top_registered_multistage_parent_composed_parent_cone_instance_child_input_binding_fraction`, `top_outputs_reaching_parent_cone_instances`, `hierarchy_outputs_reaching_parent_cone_instances`, `top_parent_cone_instance_output_fraction`, `hierarchy_parent_cone_instance_output_fraction`, `top_outputs_reaching_parent_cone_instances_through_parent_flops`, `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`, `top_parent_cone_instance_flop_output_fraction`, `hierarchy_parent_cone_instance_flop_output_fraction` |
| `max_parent_cone_instances_per_module` | `max_parent_cone_instances_per_internal_module`, `top_parent_cone_instances`, `hierarchy_parent_cone_instances` |
| `hierarchy_parent_flop_prob` | `hierarchy_parent_local_flops`, `internal_module_occurrences_with_local_flops`, `top_local_flops`, `child_input_bindings_from_parent_flops`, `parent_flop_child_input_binding_fraction`, `top_parent_flop_child_input_binding_fraction`, `child_input_bindings_from_parent_cone_instances_through_parent_flops`, `top_child_input_bindings_from_parent_cone_instances_through_parent_flops`, `parent_cone_instance_flop_child_input_binding_fraction`, `top_parent_cone_instance_flop_child_input_binding_fraction`, `top_outputs_reaching_parent_cone_instances_through_parent_flops`, `hierarchy_outputs_reaching_parent_cone_instances_through_parent_flops`, `top_parent_cone_instance_flop_output_fraction`, `hierarchy_parent_cone_instance_flop_output_fraction` |
| `width_parameterization_prob` | `num_width_parameterized_modules`, `num_param_override_instances` (per-design metrics); matrix `saw_width_parameterized_design` |
| `aggregate_prob`              | `num_packed_aggregate_modules` (per-design metric); matrix `saw_packed_aggregate_design` |
| `aggregate_array_prob`        | `num_array_packed_aggregate_modules` (per-design metric; subset of `num_packed_aggregate_modules`); matrix `saw_array_packed_aggregate_design` (`--signoff-knob-sweep-gate`) |
| `memory_prob`                 | `num_memory_modules` (per-design metric); matrix `saw_inferrable_memory_design`; with `fsm_prob`, the combined `saw_memory_fsm_interplay_design` (`--signoff-knob-sweep-gate`) |
| `fsm_prob`                    | `num_fsm_modules` (per-design metric); matrix `saw_fsm_design`; with `memory_prob`, the combined `saw_memory_fsm_interplay_design` (`--signoff-knob-sweep-gate`) |

All knobs now have a concrete metric (or metric ratio) that
measures their effect. No *pending* entries remain. Future
additions will extend this table, not shrink its
pending-coverage.

### Per-knob roll-rate validation

For every probability-roll knob the metrics also expose
`knob_roll_attempts["<knob>_prob"]` and
`knob_roll_fires["<knob>_prob"]` — the raw attempt and fire
counts taken during construction. The empirical fire-rate
`fires / attempts` is a direct check on the knob:

- Default knobs at seed 42 produce ratios like
  `share_prob: 607/1999 ≈ 0.30` (default `0.3`),
  `coefficient_prob: 51/256 ≈ 0.20` (default `0.2`),
  `comb_mux_encoding_prob: 49/94 ≈ 0.52` (default `0.5`).
- A knob that consistently misses its configured rate
  indicates either a gating condition upstream (e.g.
  `flop_prob` rolls are gated on `flop_allowed`, so hitting
  `max_flops_per_module` cuts attempts) or a bug.
- The counters cover every instrumented `gen_bool(cfg.<prob>)` site in
  the generator — see `KnobId` in `src/ir/types.rs` for
  the full list (`flop_prob`, `comb_mux_prob`,
  `priority_encoder_prob`, `coefficient_prob`,
  `const_shift_amount_prob`, `const_comparand_prob`,
  `constant_prob`, `terminal_reuse_prob`,
  `comb_mux_encoding_prob`, `flop_mux_encoding_prob`,
  `share_prob`, `flop_qfeedback_prob`, `hierarchy_sibling_route_prob`,
  `hierarchy_registered_sibling_route_prob`,
  `hierarchy_registered_child_input_cone_prob`,
  `hierarchy_child_input_cone_prob`,
  `hierarchy_parent_cone_instance_prob`, `hierarchy_parent_flop_prob`).

This is the measurability doctrine in its most direct form:
every probability dial's effect is a simple division away.
