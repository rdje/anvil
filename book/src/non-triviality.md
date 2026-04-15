# Non-Triviality and Dependency Tracking

A generator that emits semantically valid SV is not enough. If a
synthesizer reduces the entire module to constants, the output is
useless — you have tested nothing downstream.

## The three levels of correctness

1. **Syntactic:** parses without error.
2. **Semantic:** elaborates, types check, names resolve.
3. **Functional:** outputs genuinely depend on inputs; logic survives
   constant propagation.

`anvil` targets level 3. This chapter is about how.

## Dep-set tracking

Every node in the IR caches a `DepSet` — the set of primary inputs
whose values can influence this node's value.

```
Constant.deps     = {}
PrimaryInput.deps = {self}
FlopQ.deps        = { virtual_input_for_this_flop }  // see below
Gate.deps         = union(operand.deps for operand in operands)
```

At the cone root of each module output, the generator requires:

```
output_node.deps.len() >= 1
```

(counting flop virtual inputs as contributing deps). If the cone
collapsed to a constant, `deps` is empty and the cone is regenerated.

## The flop virtual-input trick

A flop's Q output, from the perspective of the *combinational cone
feeding an output*, is a leaf — the cone terminates there. But the
flop itself is driven by some D-cone that reaches primary inputs
eventually.

If we counted flop-Q deps as empty (because the flop was just
instantiated and has no declared dependencies yet), every output
driven purely by a flop would fail the non-triviality check.

Solution: assign each flop a *virtual input ID*. The flop's Q node
has `deps = {virtual_id}`. The non-triviality check treats virtual IDs
as contributing dependencies. Later, when the flop's D-cone is
generated, we require the D-cone's deps to be non-empty too —
recursively ensuring that every flop's D ultimately reaches primary
inputs (possibly through other flops).

This ensures sequential circuits are also non-trivial: no flop exists
that is driven by constants, so the whole circuit is reactive.

## Structural anti-collapse rules

Cheap, local rules during generation prevent the obvious dead-logic
patterns:

| Pattern            | Why it collapses         | Rule                                |
|--------------------|--------------------------|-------------------------------------|
| `a ^ a`            | = 0                      | Forbid identical XOR operands       |
| `a & 0`            | = 0                      | Forbid all-zero AND mask            |
| `a | all_ones`     | = all_ones               | Forbid all-ones OR mask             |
| `a & all_ones`     | = a (trivial)            | Forbid all-ones AND mask            |
| `a | 0`            | = a (trivial)            | Forbid all-zero OR mask             |
| `mux(s, a, a)`     | = a                      | Forbid identical mux arms           |
| `a - a`            | = 0                      | Forbid identical SUB operands       |
| `a == a`           | = 1                      | Forbid identical comparison operands|
| shift by 0         | = a (trivial)            | Minimum shift amount = 1            |

These rules are checked during operand selection, before the gate is
added to the IR. The cost is tiny; the benefit is that the
most-obvious collapse patterns are structurally impossible.

## Algebraic residue

No amount of structural rules catches algebraic identities like:

- `(a + b) - b == a`
- `(a & b) | (a & ~b) == a`
- `(a << 2) | (a << 2) == a << 2`

These require symbolic reasoning over the expression tree. `anvil`
does not attempt it. A real synthesizer will fold such patterns, and
the result will be smaller than expected — but not trivially constant,
because the surrounding cone still references independent inputs.

If this becomes a problem in practice (observed by post-synthesis
analysis), the fix is to add a cheap canonicalizer that rewrites
obvious identities and then re-checks dep-sets. For v1 we accept the
residue.

## Why not use the oracle to filter?

An earlier design sketched an oracle (a Rust evaluator) that would run
random input vectors through the IR and discard modules whose outputs
never changed. This is both:

- **Unnecessary for our actual goal** (generating synthesizable RTL,
  not testing tools).
- **Expensive** (N vectors × M outputs × cone evaluation per module).

Dep-set tracking + structural rules get us to "almost never
trivially constant" for a tiny fraction of the cost. We drop the
oracle entirely. Users who want tool-testing workflows can run
downstream tools (Verilator, Yosys) against the output themselves.
