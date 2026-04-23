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
ensuring that every flop's D is a function of at least one endpoint
leaf: a primary input, another flop Q, or its own Q.

This ensures sequential circuits are structurally non-trivial: a flop
may be fed by primary inputs, by other flop Q endpoints, or by its own Q
under the Q-feedback freedom doctrine, but it is not accepted as a
purely constant cone.

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

**Identity / factorization gating:** operand-duplicate rejection for
`And` / `Or` / `Xor` and the default duplicate rejection for `Add` /
`Mul` are gated by
`factorization_level >= OperandUnique` inside `identity_mode = node-id`.
Below that rung, duplicate operands are permitted. The base local
degeneracy guards for `Sub`, `Eq`, and `Neq` still fire, and
`Mux(s, a, a)` is still governed by `mux_arm_duplication_rate`.
At `identity_mode = relaxed`, the dedup path is bypassed entirely, but
these generator-side local cleanup guards still prevent obvious emitted
degeneracies.

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
operand uniqueness (Rule 8 extended), commutative normalization
(Rule 21b), associative flattening, constant folding, a narrow set of
peephole rewrites, and a bounded e-graph-style semantic merge fragment
— together close the *within-gate* duplication surface:
same AST ⇒ same NodeId, no duplicate operands (at default
knobs), `a+b` and `b+a` share identity, algebraic identities
such as `x + 0`, `x * 1`, `x & all_ones` collapse at intern
time, and fully-constant comparisons / full-width slices /
single-operand concats get rewritten away before landing as
literal gates.

Sequential state now gets one conservative extra layer after drain:
under `identity_mode = node-id` with effective level `>= cse`,
flops with identical exact emitted-state signatures (`width`,
reset, `d`) are merged too.

NodeId compaction is a post-construction pass that walks from
roots and drops any gate that no longer has a consumer. It's
the infrastructure piece that lets rewrites such as
`Not(Not(x)) → x` and associative flattening fire without
violating Rule 18 — an inner gate may be left unreferenced by
the outer call's short-circuit or splice, and compaction cleans
it up at module finalisation. The count is surfaced via
`Metrics::nodes_compacted`.

Associative flattening (Layer 4) landed on top of compaction:
at intern time, any `And`/`Or`/`Xor`/`Add`/`Mul` operand that
is itself a same-op same-width gate is spliced into the outer
operand list, so `Add(a, Add(b, c))` becomes `Add(a, b, c)`.
Per-op semantic normalisation runs after the splice:
`And`/`Or` dedup (idempotent), `Xor` pair-cancel (self-
inverse), and `Add`/`Mul` conservatively skip the flatten when
duplicates would result (to preserve `x + x = 2x` /
`x * x = x²` under the strict `operand_duplication_rate`).
Fires are counted in `Metrics::flatten_associative_applied`,
and the complementary `nested_associative_operand_count`
metric — the post-construction count of flattening
opportunities — sits at zero at default knobs, a direct
verification that the layer is exhaustive there.

The remaining aspirational work lives in the top rung of
`FactorizationLevel`: cross-gate identities like
`(a + b) - b → a` need symbolic reasoning over the expression
tree. That's the e-graph problem proper — the asymptote we
climb toward without claiming to have reached.

**Syntactic vs semantic identity.** What's already implemented
covers the promise that **two syntactically identical
expressions share one node** — same AST key, same commutative
permutation, one NodeId. The ladder above extends that toward
**two semantically equivalent expressions share one node** — a
strictly harder problem that synthesis tools themselves solve
incompletely. We climb toward it one layer at a time rather than
trying to land the theoretical ceiling in one leap.

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

That does **not** forbid explicit expected-facts manifests for future
artifact families. A manifest is a declared contract for one family of
generated files; what is still rejected is a bundled general-purpose
RTL interpreter used as a global filter over all output.
