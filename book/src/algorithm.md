# The Fanin Cone Algorithm

The heart of `anvil`. Written as pseudocode; the Rust implementation in
`src/gen/cone.rs` is a direct transcription.

> **Strategy note:** the pseudocode below describes the
> `sequential` construction strategy — cones built one output at a
> time in declaration order. It is the simplest recursion shape and
> the right starting point for understanding the generator. The
> default strategy is `interleaved` (a global frame queue driving
> all output cones in lockstep); `shuffled` is the same per-output
> recursion in random order. The retired `graph-first` is a silent
> alias for `interleaved`. See
> [Construction Strategies](construction-strategies.md) for the
> full comparison and retirement rationale.

## Module-level generation

```
generate_leaf_module(rng, knobs, index):
    n_in  = rand(knobs.min_inputs,  knobs.max_inputs)
    n_out = rand(knobs.min_outputs, knobs.max_outputs)

    # Reserve port ids 0 and 1 for clk and rst_n. They are shared by
    # every flop in the module. They are NOT added to the signal pool,
    # so cones cannot terminate at them.
    module = Module::new(name = f"mod_{seed}_{index:04}")
    module.inputs += [Port(id=0, name="clk",   width=1, In)]
    module.inputs += [Port(id=1, name="rst_n", width=1, In)]
    module.clock = Some(0)
    module.reset = Some(1)

    # Primary data inputs: port ids 2..2+n_in
    data_inputs = [ fresh_input(port_id = 2 + i, rand_width())
                    for i in 0..n_in ]
    module.inputs += data_inputs

    # Primary outputs: port ids start after all inputs
    outputs = [ fresh_output(port_id = 2 + n_in + i, rand_width())
                for i in 0..n_out ]
    module.outputs = outputs

    pool = SignalPool::from(data_inputs)       # seed with data inputs
    worklist = FlopWorklist::new()

    # Build an output cone per primary output. `exclude = None` because
    # there is no flop Q to isolate at this level.
    for out in outputs:
        cone_root = build_cone_with_retry(
            width = out.width, exclude = None)
        module.drives += (out.id, cone_root)

    # Drain the flop worklist to quiescence. Each drain call may itself
    # enqueue more flops.
    drain_flop_worklist(pool, worklist)

    return module
```

## Cone recursion

```
build_cone(width, depth, exclude):
    leaf_prob  = depth / max_depth
    force_leaf = depth >= max_depth OR rand() < leaf_prob

    if force_leaf:
        return pick_terminal(width, exclude)

    # Flop branch: allowed up to max_flops_per_module.
    if m.flops.len() < max_flops_per_module
       AND rand() < flop_prob:
        return build_flop_leaf(width)           # enqueues this flop

    # Comb-mux block branch: M-to-1 combinational mux, OneHot or Encoded.
    # No Q-feedback (no state). See structural-rules.md Rule 15.
    if rand() < comb_mux_prob:
        return build_comb_mux(width, depth, exclude)

    # Priority-encoder block branch: chained ternary over N 1-bit
    # request bits. Skipped if no N ∈ [min_mux_arms, max_mux_arms]
    # satisfies ceil_log2(N) == width. See Rule 17.
    if rand() < priority_encoder_prob:
        result = try_build_priority_encoder(width, depth, exclude)
        if result is Some(node): return node

    # Gate branch.
    op = pick_gate(width)

    # Motif dispatch: coefficient / const-shift / const-comparand.
    # Each is a specialised compound form that replaces the generic
    # recursion for its op family. See structural-rules.md Rules
    # 19 (coefficient), 20 (dep-bearing source required in the
    # position variants).
    if op in {Add, Sub, Mul} AND rand() < coefficient_prob:
        return build_linear_combination(op, width, depth, exclude)
    if op in {Shl, Shr} AND rand() < const_shift_amount_prob:
        return build_shift_const_amount(op, width, depth, exclude)
    if is_comparison(op) AND rand() < const_comparand_prob:
        return build_comparison_const_comparand(op, depth, exclude)

    # Snapshot construction state before operand construction (Rule 18
    # enforcement). If the composed gate fails anti-collapse, we roll
    # back — operand sub-trees built for the rejected gate vanish from
    # the IR so they don't orphan.
    snap = Snapshot::of(m.nodes, m.flops, pool, worklist,
                       m.gate_instances, m.const_instances)

    operand_widths = input_widths_for(op, width)
    operands = []
    for w in operand_widths:
        # DAG-sharing fork: with probability share_prob, terminate
        # this operand at an existing pool entry instead of recursing.
        if rand() < share_prob:
            shared = try_share(pool, w, exclude)
            if shared is Some(node):
                operands.push(node)
                continue
        operands.push(build_cone(w, depth + 1, exclude))

    # Structural anti-collapse (Rule 8 extended): reject operand
    # multisets that degenerate algebraically. The check depends on
    # factorization_level — see below. On rejection, restore snapshot
    # and fall back to pick_terminal.
    if violates_anti_collapse(op, operands):
        snap.restore()
        return pick_terminal(width, exclude)

    # `intern_gate` enforces Rule 21 (CSE) + Rule 21b (commutative
    # normalization for And/Or/Xor/Add/Mul) + AST-instance cap. The
    # same (op, sorted_operands, width) returns the same NodeId.
    (node, is_new) = m.intern_gate(op, operands, width, deps)
    if is_new:
        pool.add(node, width, deps)             # new gate is shareable
    return node
```

### Retry loop

`build_cone_with_retry` wraps `build_cone` with a bounded retry
(currently 4) that rejects trivial (empty dep-set) cone roots. On
rejection, the IR mutation is rolled back via `Vec::truncate` on
`m.nodes` and `m.flops`, and the pool, worklist, `gate_instances`,
and `const_instances` tables are restored from a clone.

After the retry budget is exhausted, the last attempt is accepted
(the validator will then reject the whole module if it is truly
trivial — but this has never been observed in practice with current
defaults).

Note the full snapshot: the dedup tables (`gate_instances`,
`const_instances`) must be restored alongside `m.nodes` or stale
entries would point at truncated NodeIds. A later `intern_gate` call
would then return a node of a different kind than its key promised.
This is a load-bearing invariant — see `DEVELOPMENT_NOTES.md`
"Construction-time CSE" for the failure mode.

## Flop worklist drain

Each flop on the worklist gets:

- A random M drawn from `{0, 2, 3, ..., max_mux_arms}` (M = 1 excluded).
- A random `FlopKind` (`ZeroDefault` | `QFeedback`).
- A random `FlopMux` style (`OneHot` | `Encoded`) via
  `flop_mux_encoding_prob`.

```
drain_flop_worklist():
    while (flop_id = worklist.pop()) is Some:
        width    = m.flops[flop_id].width
        kind     = m.flops[flop_id].kind
        q_node   = m.flops[flop_id].q
        # Rule 2 (Q-feedback freedom): the flop's own Q may appear
        # freely as a leaf in any of its data / select / direct-D
        # sub-cones. No Q-exclusion — pass exclude = None.
        exclude  = None
        M        = pick_mux_arm_count()

        if M == 0:
            d = build_cone_with_retry(width, exclude)
            m.flops[flop_id].d   = Some(d)
            m.flops[flop_id].mux = FlopMux::None
            continue

        if rand() < flop_mux_encoding_prob:
            (d, mux) = drain_flop_encoded(width, kind, q_node, M, exclude)
        else:
            (d, mux) = drain_flop_one_hot(width, kind, q_node, M, exclude)
        m.flops[flop_id].d   = Some(d)
        m.flops[flop_id].mux = mux
```

See `book/src/sequential.md` for the one-hot and encoded assembly
shapes (OR-of-masks vs chained ternary over `Eq(sel, k)`).

## Terminal selection

```
pick_terminal(width, exclude):
    # 1. Prefer matching-width pool entries with non-empty deps.
    with_deps = pool.of_width(width)
                    .filter(!excluded and deps non-empty)
    if with_deps:
        return random_pick(with_deps)

    # 2. Fall back to any matching-width entry (may be a constant).
    any_match = pool.of_width(width).filter(!excluded)
    if any_match:
        return random_pick(any_match)

    # 3. No matching width. Lazy width-adapter from the widest
    #    dep-bearing pool entry.
    src = pool.iter().filter(!excluded and deps non-empty)
                     .max_by(width)
    if src is Some:
        return make_width_adapter(src, width)   # Slice or Concat(+Slice)

    # 4. Last resort: emit a constant. Non-triviality retry will
    #    likely reject this cone and regenerate.
    return emit_constant(width)
```

## Width rules per gate

| Gate              | Output width W | Input widths                                |
|-------------------|----------------|---------------------------------------------|
| `and/or/xor/+/*`  | W              | [W, W, ...] (N ≥ 2; associative — Rule 14)  |
| `-`               | W              | [W, W] (strictly 2-arity; not associative)  |
| `not`             | W              | [W]                                         |
| `==/!=/</>/</>`   | W = 1          | [K, K] for chosen K                         |
| `mux`             | W              | [1, W, W]                                   |
| `slice[hi:lo]`    | W = hi-lo+1    | [K] for K > hi                              |
| `concat`          | W = sum(Wᵢ)    | [W₁, W₂, …]                                 |
| unary reduction   | W = 1          | [K] for chosen K                            |
| `<</>>`           | W              | [W, any]                                    |

Comparisons and reductions are the only ops where the parent width
does not directly determine input widths; for those, the generator
picks an internal operand width K freely. Shifts accept any-width
shift amounts.

The associative operators (`and/or/xor/+/*`) pick an arity N randomly
from `[cfg.min_gate_arity, cfg.max_gate_arity]` each time they are
chosen. See `book/src/structural-rules.md` Rule 14 for the full
operator-vs-block framing.

These rules are enforced both at construction (by
`input_widths_for`) and at validation (by
`ir::validate::check_gate_shape`). A mismatch between the two is a
generator bug caught at validation time.

## Dependency propagation

Every node carries a `DepSet` (ordered set of primary-input `PortId`s,
plus virtual ids for flop Qs):

- `Constant.deps     = {}`
- `PrimaryInput.deps = {self.port}`
- `FlopQ.deps        = {virtual_id(flop)}`
- `Gate.deps         = union(operand.deps)`

The cone root of each output must satisfy `deps.len() >= 1`. Flop-Q
virtual ids count toward non-triviality because a flop is itself fed
by a cone that eventually reaches primary inputs (recursively enforced
when the flop's D-cone is built).

## Structural anti-collapse rules

Cheap to enforce during generation; catch the obvious constant-folding
cases. The exact rule set depends on `factorization_level`:

At the default level (`e-graph`, currently equivalent to
`commutative`):

- **Idempotent / self-inverse N-arity ops** (`And`, `Or`, `Xor`):
  any duplicate `NodeId` in the operand list is forbidden
  (`x & x = x`, `x | x = x`, `x ^ x = 0` at any arity). Operand-
  multiset distinctness, not just pairwise.
- **`Sub` (2-arity):** `x - x = 0` forbidden.
- **`Eq` / `Neq`:** `x == x = 1`, `x != x = 0` forbidden.
- **`Mux`:** `mux(s, a, a) = a` forbidden (gated on
  `mux_arm_duplication_rate`).
- **`Add` / `Mul`:** operand duplicates *allowed by default but
  gated* on `operand_duplication_rate`. At rate 0.0 (default), no
  duplicates. At rate 1.0, `x + x = 2x` and `x * x = x²` pass
  through. Duplicates are algebraically meaningful here, so the
  user opts in.

Lowering `factorization_level` relaxes these rules. At level
`cse`, only the 2-operand algebraic-degeneracy cases (`Sub` /
`Eq` / `Neq`) fire — the rest are permitted (and picked up by
syntactic CSE at the AST-key level). At level `none` no
anti-collapse rules fire and the dedup path is bypassed entirely.

On rejection, `build_cone` restores its pre-operand-construction
snapshot and falls back to `pick_terminal`. This prevents the
rejected sub-trees from becoming orphans (Rule 18) — every gate in
the final IR has a consumer by construction.

These rules do not catch algebraic identities deeper in the tree
(`(a + b) - b`, etc.). Those survive and show up in the output. The
factorization ladder now implements `associative`,
`constant-fold`, and `peephole`; only the `e-graph` level remains
aspirational for deeper cross-gate algebraic equivalence.

After cone construction, module finalisation does one more
alignment pass before emission: it drops construction-only
`Flop.mux` operand references, compacts unreachable nodes, then
shrinks/prunes primary inputs so the emitted port surface matches
the live logic instead of a provisional wider first draft.
