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
patterns. The current rule set, as enforced by
`violates_anti_collapse` in `src/gen/cone.rs`:

| Pattern                                | Why it collapses         | Rule                                                     |
|----------------------------------------|--------------------------|----------------------------------------------------------|
| `And/Or/Xor` with duplicate operand    | `x & x = x`, `x ^ x = 0` | Forbidden at any arity via operand-multiset distinctness |
| `Add/Mul` with duplicate operand       | `x + x = 2x`, `x * x = x²` (meaningful, but optional) | Forbidden by default (`operand_duplication_rate = 0.0`); opt in with the knob |
| `Sub` with equal operands (2-arity)    | `x - x = 0`              | Forbidden                                                |
| `Eq/Neq` with equal operands (2-arity) | `x == x = 1`, `x != x = 0` | Forbidden                                              |
| `Mux(s, a, a)`                         | = `a`                    | Forbidden by default (`mux_arm_duplication_rate = 0.0`); opt in with the knob |

Rules are checked after operand selection, *before* the gate is
committed to the IR. On rejection, `build_cone` restores its
pre-operand-construction snapshot and falls back to `pick_terminal`
— the operand sub-trees built for the rejected gate vanish from
the IR (Rule 18 α, ensuring no orphan gates).

**Factorization-level gating:** the rules above apply at
`factorization_level ≥ OperandUnique` (the default, effectively
`e-graph`). At level `cse` only the 2-operand algebraic-degeneracy
cases (`Sub` / `Eq` / `Neq`) fire. At level `none` the dedup path
is bypassed entirely and no anti-collapse checks run.

See [Structural Rules](structural-rules.md) Rule 8 for the
authoritative catalog entry.

## Algebraic residue

Local anti-collapse rules catch patterns inside one gate. They do
not catch algebraic identities spanning multiple gates:

- `(a + b) - b == a`
- `(a & b) | (a & ~b) == a`
- `(a + 1) + 1 == a + 2`  (associativity across the tree)

These require symbolic reasoning over the expression tree. A real
synthesizer will fold such patterns; the result will be smaller
than expected — but not trivially constant, because the surrounding
cone still references independent inputs.

**Factorization ladder.** `anvil` has started climbing this
ladder. The implemented layers — syntactic CSE (Rule 21),
operand uniqueness (Rule 8 extended), and commutative
normalization (Rule 21b) — together close the *within-gate*
duplication surface: same AST ⇒ same NodeId, no duplicate
operands (at default knobs), `a+b` and `b+a` share identity.

The remaining aspirational layers in `FactorizationLevel` —
`Associative`, `ConstantFold`, `Peephole`, `EGraph` — are not
yet implemented. When they land, the `factorization_level` dial
automatically activates them for users already at higher
levels (the default is `e-graph`, which `effective()` clamps
down to the highest implemented layer today).

See `book/src/structural-rules.md` Rule 21c for the dial, and
`DEVELOPMENT_NOTES.md` "Full factorization doctrine" for the
`NodeId = expression identity` framing.

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
