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
deps, honoring the Q-exclusion contract used for flop D-cones.

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

The non-triviality structural rules still apply post-sharing:

- A gate cannot have the same node as both inputs of `xor`, `sub`,
  `==`, `!=` — even if that node is shared from the pool. The check
  is on `NodeId` equality, which catches sharing-induced collapses at
  gate-assembly time.
- Mux arms cannot be the same node.

Without these, sharing would *increase* the rate of trivial collapse
because reusing the same wire on both sides of an XOR is now likely.

## What sharing does *not* do

It does not deduplicate equivalent sub-expressions that happen to be
generated independently. If two cones both build `(i_0 + i_1)` from
scratch, they remain two separate gates. Common-subexpression
elimination is the synthesizer's job, not the generator's.

## Cross-output and cross-cone sharing

Every cone in a module — each primary output's cone and each flop's
D-cone — shares the same module-wide `SignalPool`. A gate built
while constructing output 0's cone is immediately eligible as a leaf
or a DAG-sharing candidate inside output 1's cone, output 2's cone,
and any flop D-cone drained later. There is no per-cone isolation.

Outputs are built in declaration order, so later-declared outputs
see more sharing candidates than earlier ones. This is an artifact
of the implementation but matches real-design patterns (later logic
often consumes earlier-computed intermediates).

See Rule 16 in the [Structural Rules catalog](structural-rules.md)
for the authoritative statement.

## No cycles possible

The pool only contains signals that *already exist* when an operand
decision is made. The gate currently being assembled is added to the
pool *after* its operands are resolved. Therefore no gate can
transitively reference itself through sharing. Flop-Q-to-D feedback
is the only legal cycle, and it is gated by the existing Q-exclusion
contract (see `book/src/sequential.md`).
