# The Fanin Cone Algorithm

The heart of `anvil`. Written as pseudocode; the Rust implementation in
`src/gen/cone.rs` is a direct transcription.

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

    # Gate branch.
    op = pick_gate(width)
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

    # Structural anti-collapse: reject obvious identity patterns.
    if violates_anti_collapse(op, operands):
        return pick_terminal(width, exclude)

    node = Gate { op, operands, width, deps = union(operand deps) }
    m.nodes.push(node)
    pool.add(node, width, deps)                 # new gate is shareable
    return node
```

### Retry loop

`build_cone_with_retry` wraps `build_cone` with a bounded retry
(currently 4) that rejects trivial (empty dep-set) cone roots. On
rejection, the IR mutation is rolled back via `Vec::truncate` on
`m.nodes` and `m.flops`, and the pool + worklist are restored from
a clone. After the retry budget is exhausted, the last attempt is
accepted (the validator will then reject the whole module if it is
truly trivial — but this has never been observed in practice with
current defaults).

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
        exclude  = Some(q_node)       # Q-exclusion contract
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
cases:

- `a ^ a`, `a - a`, `a == a`, `a != a` — forbidden (operand `NodeId`
  equality check). Covers both pure-tree self-reference and
  sharing-induced self-reference (same pool entry picked twice).
- `mux(s, a, a)` — forbidden (identical data arms).

These do not catch algebraic identities deeper in the tree
(`(a + b) - b`, etc.). Those survive and show up in the output. The
philosophy is: prevent the *obvious* collapses cheaply; accept that
the remaining output may still contain algebraic redundancy. A real
synthesizer will fold it away; the surrounding cone retains its
non-trivial structure.
