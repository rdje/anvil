# Hierarchy: Modules of Modules

The cone-recursion algorithm produces one **leaf module**: ports plus
internal logic, no sub-instances. That is the first level of
abstraction.

The second level instantiates leaf (or sub-hierarchy) modules inside
larger modules. The algorithm is structurally identical — it is *the
same recursion* — with one extra choice at each cone node:
"instantiate a sub-module and use one of its output ports."

This chapter describes hierarchy in the **current circuit-IR framing**.
The broadened roadmap now also includes future source-level
frontend/elaboration artifact families, which will likely need a
parameter / package / type aware source-level IR in addition to this
gate-and-instance view.

## Generation order

```
generate_design(rng, knobs):
    library = []
    for _ in 0..knobs.num_leaf_modules:
        library.push(generate_leaf_module(rng, knobs))

    top = generate_hierarchical_module(rng, knobs, library, depth=0)
    return Design { top, all_modules: library + top + ... }
```

Two sub-module sourcing strategies:

**Library mode.** Pre-generate a pool of leaf modules. When the
hierarchical generator needs a sub-module with specific port widths,
search the pool. If a match exists, instantiate it. If not, either
fall back to gate-level logic or generate one on demand.

**On-demand mode.** Whenever the hierarchical generator decides to
emit an instance, generate a fresh sub-module sized to the parent's
needs. The pool is appended-to, not searched first.

A mix of both, controlled by a knob (`--library-prob`), produces
realistic patterns: some heavily-instantiated modules (memories,
adders) and many one-off custom blocks.

## The extended choice set

In the leaf generator, `pick_node_kind` returns one of:

```
{ Terminal, Gate(g), Flop }
```

In the hierarchical generator, it also returns:

```
{ Terminal, Gate(g), Flop, Instance(m, output_port_idx) }
```

When `Instance(m, k)` is picked:

1. Allocate a sub-module instance of module `m`.
2. The chosen output port (index `k`, with matching width) becomes
   the result of this cone node.
3. The sub-module's input ports become *new sub-cones to drive*,
   added to the worklist.
4. The instance's other output ports become available in the signal
   pool (they can be picked as terminals later).

The worklist mechanism that already handles flop D-cones extends
naturally: instance input cones go on the same worklist.

## Hierarchy depth

`--hierarchy-depth N` bounds how many levels of sub-modules can nest.
Depth 0 = leaf modules only (Phase 1 behavior). Depth 1 = top module
instantiates leaves but leaves themselves contain no instances.
Depth 2 = leaves can also instantiate sub-leaves. And so on.

In practice, depth 2–3 is usually enough to produce interesting
hierarchical patterns. Deeper than that and elaboration time grows
fast.

## Naming uniqueness

Module names: `mod_<seed>_<index>`. Instance names: `u_<idx>`. Port
and wire naming follows the leaf convention. The emitter ensures
uniqueness within each scope; cross-module name collisions are
impossible because each module has its own namespace.

When emitting multiple files (one per module), the emitter writes a
manifest so downstream tools know which file declares which module
and which is the top.

## Width matching for instances

When the parent cone requires width W and considers instantiating
sub-module `m`, it filters `m`'s output ports to those with width W.
If none match, it cannot use `m` here — fall back to a gate or
generate on-demand. This filtering happens upfront when picking
candidates; we never instantiate and then discover the widths don't
fit.

## Why this generalizes cleanly

The choice "what drives this signal?" doesn't care whether the answer
is a gate, a flop, a primary input, or a sub-module output port. They
are all just nodes that produce a value of some width. The generation
algorithm is one recursion; the IR has one node type per choice; the
emitter has one printer per node type. Adding hierarchy is *not* a new
algorithm — it is a new node kind in the same algorithm.

This is the payoff of the circuit-graph framing. A grammar-based
design would need separate productions and separate annotation flow
for instantiation; the IR view absorbs it as a node variant.
