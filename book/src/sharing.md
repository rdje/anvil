# Signal Sharing: Trees Become DAGs

`anvil` supports both tree-shaped and DAG-shaped cones, with the
choice made **per recursion point**. A single module can mix both
freely: some operands recurse to create fresh logic (tree-local), some
terminate at existing signals (DAG-local). The same module can look
tree-ish in one sub-cone and DAG-ish in the next.

The `share_prob` knob controls the per-operand decision. Two extreme
settings collapse the behavior to a single mode across the whole run:

- `share_prob = 0.0` — no non-leaf sharing. The only reuse comes from
  `pick_terminal`'s leaf-level pool picks (always on). Internal gates
  have exactly one consumer each. Cones are tree-ish.
- `share_prob = 1.0` — every operand that *can* be shared is shared.
  Fresh gates are created only when no matching-width pool entry
  exists with non-empty deps. Cones are maximally-DAG-ish.

In between, each value mixes both shapes per recursion point. That is
Phase 2's guiding principle: "tree or DAG" is not a global mode choice
— it is a local decision made wherever the recursion needs an operand.

## The mechanism

The `SignalPool` holds every signal with a declared width and a known
dep-set: primary inputs, flop-Qs, and every `Gate` node as it is
created. During `build_cone`, when the recursion needs an operand at
width W, it rolls the `share_prob` coin:

```
for w in operand_widths:
    if rand() < share_prob and try_share(pool, w, exclude) is Some(node):
        operands.push(node)          # SHARING — terminate at pool entry
    else:
        operands.push(build_cone(w)) # RECURSE — create fresh logic
```

`try_share` returns a random matching-width pool entry with non-empty
deps, honoring the optional `exclude` filter used by rare sites that
need to forbid a specific `NodeId` from being picked. Flop D-cones
do **not** exclude the flop's own Q — Rule 2 (Q-feedback freedom)
allows Q as a free leaf in any sub-cone of its own D, because the
clock edge breaks the Q→D loop temporally.

This is the *non-leaf* sharing path. It complements the *leaf* sharing
that has been present since Phase 1 (when recursion hits `max_depth`
or rolls a forced-leaf coin, `pick_terminal` picks from the same
pool). Together they cover every operand position.

## Dep-set propagation through sharing

When a gate is shared, its `DepSet` is shared too. The consumer's deps
become the union of *all* its operands' deps, including the shared
node's deps. Non-triviality continues to hold by the same argument as
before: the dep-set is a structural property of the IR, independent
of how many consumers a node has.

## Knob calibration

The default is `share_prob = 0.3`: every operand has a 30% chance of
being shared. Useful range is 0.2–0.7.

Higher sharing means:

- More realistic-looking RTL (closer to hand-written).
- Tighter cones with more fanout per signal.
- Easier-to-collapse output (synthesis can find more common
  subexpressions, but post-synthesis netlists are still non-trivial).

Lower sharing means:

- Wider, deeper, more sprawling logic.
- Each output has its own private cone with little reuse.
- Closer to fuzzer-style stress patterns.

## Forbidden sharing patterns

The non-triviality structural rules still apply post-sharing, now
covering a richer set than when Phase 2 first landed — see
[Rule 8 extended](structural-rules.md) for the authoritative catalog.
Summary:

- **`And` / `Or` / `Xor` at any arity:** no `NodeId` may appear
  more than once in the operand list. Operand-multiset
  distinctness — catches N-way collisions (`x ^ x ^ x ^ x = 0`
  just as reliably as 2-arity `x ^ x`).
- **`Sub`, `Eq`, `Neq` (2-arity):** operands must differ.
- **`Add` / `Mul`:** operand duplicates forbidden by default, gated
  on `operand_duplication_rate`. Set the knob > 0.0 to allow
  `x + x = 2x` and `x * x = x²` shapes.
- **`Mux`:** `mux(s, a, a) = a` forbidden by default, gated on
  `mux_arm_duplication_rate`. Same applies to the N-to-1 extension
  (no single data signal repeats across arms under the knob).

Without these, sharing would *increase* the rate of trivial
collapse because reusing the same wire in multiple operand slots is
now likely.

On anti-collapse rejection, `build_cone` restores its pre-operand
snapshot and falls back to `pick_terminal` — the operand sub-trees
built for the rejected gate vanish from the IR (Rule 18 α), so
speculatively-shared signals don't orphan.

## Construction-time CSE (Rule 21)

Beyond the per-operand share/recurse fork, `anvil` performs
**syntactic CSE at intern time**: every `Node::Gate` and
`Node::Constant` creation goes through `Module::intern_gate` /
`intern_constant`, which dedupes by `(op, operands, width)` /
`(width, value)`. Two cones that independently build
`Add([i_0, i_1], 8)` get the *same* `NodeId` — one node, two
consumers.

The `max_ast_instances` knob caps how many distinct `NodeId`s may
represent the same AST. Default `1` = strict CSE. At
`factorization_level ≥ Commutative` (the default), operands of
`And / Or / Xor / Add / Mul` are sorted before interning, so
`a + b` and `b + a` also share identity.

This makes sharing "deeper" than the per-operand coin: even when
neither cone picked the share-path, two independently-constructed
identical expressions collapse to one node automatically. The coin
controls whether the recursion *terminates* at an existing pool
entry (early cut-off, smaller sub-cone); CSE controls whether
logically-identical sub-cones share identity (same-op same-operand
dedup, no cut-off). They compose.

See [Rule 21 (AST-instance cap)](structural-rules.md) and
[Rule 21b (commutative normalization)](structural-rules.md) for the
full factorization-ladder framing.

## Cross-output and cross-cone sharing

Every cone in a module — each primary output's cone and each flop's
D-cone — shares the same module-wide `SignalPool`. A gate built
while constructing output 0's cone is immediately eligible as a leaf
or a DAG-sharing candidate inside output 1's cone, output 2's cone,
and any flop D-cone drained later. There is no per-cone isolation.

The `sequential` construction strategy builds outputs in
declaration order, so later-declared outputs see more sharing
candidates than earlier ones. That asymmetry is a construction
artifact.

`interleaved` (the default) eliminates the asymmetry by driving all
output cones in lockstep via a single global frame queue; each
cone's leaf-level picks see the full module-wide pool. `shuffled`
randomises output build order per seed, amortising the asymmetry
across a seed sweep rather than eliminating it. The fourth
historical strategy, `graph-first`, is retired and now routes to
`interleaved` — see the
[Construction Strategies](construction-strategies.md) chapter's
"Retired" section.

See Rule 16 in the [Structural Rules catalog](structural-rules.md)
for the authoritative statement on cross-cone sharing.

## No combinational cycles possible

The pool only contains signals that *already exist* when an operand
decision is made. The gate currently being assembled is added to the
pool *after* its operands are resolved. Therefore no gate can
transitively reference itself through pure combinational logic.
This is arena-index monotonicity — [Rule 1](structural-rules.md) in
the catalog.

Flop Q→D feedback through combinational logic *is* a legal pattern
(counters, accumulators, state machines all work this way). The
clock edge breaks the loop temporally: `Q[n+1]` is computed from
`Q[n]` plus possibly other signals. See
[Rule 2 (Q-feedback freedom)](structural-rules.md) for the
authoritative statement.
