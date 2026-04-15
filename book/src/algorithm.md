# The Fanin Cone Algorithm

The heart of `anvil`. Written as pseudocode; the Rust implementation in
`src/gen/cone.rs` is a direct transcription.

## Module-level generation

```
generate_module(rng, knobs):
    n_in  = rand(knobs.min_inputs,  knobs.max_inputs)
    n_out = rand(knobs.min_outputs, knobs.max_outputs)

    inputs  = [ fresh_input(rand_width())  for _ in 0..n_in  ]
    outputs = [ fresh_output(rand_width()) for _ in 0..n_out ]

    module = Module::new(inputs, outputs)

    flop_worklist = Queue::new()
    signal_pool   = SignalPool::from(inputs)

    for out in outputs:
        cone_root = build_cone(rng, knobs,
                               width  = out.width,
                               depth  = 0,
                               pool   = signal_pool,
                               flops  = flop_worklist,
                               module = module)
        module.drive(out, cone_root)
        require(cone_root.deps.len() >= 1)  // non-triviality

    while not flop_worklist.empty():
        flop = flop_worklist.pop()
        d_cone = build_cone(rng, knobs,
                            width  = flop.width,
                            depth  = 0,
                            pool   = signal_pool,
                            flops  = flop_worklist,
                            module = module)
        module.drive(flop.d, d_cone)

    return module
```

## Cone recursion

```
build_cone(rng, knobs, width, depth, pool, flops, module):
    if depth >= knobs.max_depth OR rand() < leaf_prob(depth, knobs):
        return pick_terminal(rng, knobs, width, pool)

    kind = pick_node_kind(rng, knobs)     // gate | flop | terminal

    match kind:
        case Terminal:
            return pick_terminal(rng, knobs, width, pool)

        case Flop:
            flop = Flop::new(width)
            module.add_flop(flop)
            flops.push(flop)              // D-cone generated later
            pool.add(flop.q)              // Q is now a shareable signal
            return flop.q

        case Gate(g):
            operand_widths = g.input_widths_for_output(width)
            operands = []
            for w in operand_widths:
                operands.push(build_cone(rng, knobs, w,
                                         depth + 1, pool, flops, module))
            node = module.add_gate(g, operands, width)
            pool.add(node)                // may be shared later
            return node
```

## Terminal selection

```
pick_terminal(rng, knobs, width, pool):
    candidates = pool.signals_of_width(width)
    if not candidates.empty() AND rand() < knobs.terminal_reuse_prob:
        return pick_one(candidates)
    if rand() < knobs.constant_prob:
        return Constant::random(width)   // with non-triviality guards
    // fall back: must pick an existing signal; fail if none.
    return pick_one_or_else_constant(candidates, width)
```

## Width rules per gate

| Gate            | Output width W | Input widths             |
|-----------------|----------------|--------------------------|
| `and/or/xor`    | W              | [W, W] (or [W, W, W, …]) |
| `not`           | W              | [W]                      |
| `+/-`           | W              | [W, W]                   |
| `==/!=/</>`     | W = 1          | [K, K] for chosen K      |
| `mux`           | W              | [1, W, W]                |
| `slice[hi:lo]`  | W = hi-lo+1    | [K] for K > hi           |
| `concat`        | W = sum(Wᵢ)    | [W₁, W₂, …]              |
| unary reduction | W = 1          | [K] for chosen K         |

Comparisons and reductions are the *only* ops where the parent width
does not directly determine input widths; for those, the generator
picks an internal operand width K freely.

## Dependency propagation

Every node carries a `deps: BitSet<PrimaryInputId>` field.

- `Constant.deps     = {}`
- `PrimaryInput.deps = {self.id}`
- `FlopQ.deps        = {flop.id_as_virtual_input}` (for the combinational
  cone view; D's deps are tracked separately)
- `Gate.deps         = union(input.deps for input in operands)`

The cone root of each output must satisfy `deps.len() >= 1` over
primary inputs (flop-Q deps count toward non-triviality because a flop
is itself fed by a cone that eventually reaches primary inputs).
Otherwise the output is trivially constant and the generator
regenerates that cone.

## Structural anti-collapse rules

Cheap to enforce during generation; catch most constant-folding cases:

- `a ^ a` — forbidden (pick different operands).
- `a & 0`, `a & all-ones`, `a | 0`, `a | all-ones` — forbidden as
  top-level constant operands; allowed if the constant is itself the
  output of a non-trivial sub-cone (rare).
- Mux with identical data arms — forbidden.
- Constant shift amounts of 0 on a single-operand shift — forbidden.

These do not catch algebraic identities deeper in the tree
(`(a + b) - b`, etc.). Those survive and show up in the output. The
philosophy is: prevent the *obvious* collapses cheaply; accept that
the remaining output may still contain some algebraic redundancy.
A real synthesizer will fold it away, which is fine — there will still
be surviving logic elsewhere in any sufficiently deep cone.
