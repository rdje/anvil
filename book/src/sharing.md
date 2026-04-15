# Signal Sharing: Trees Become DAGs

In Phase 1, every cone is a tree: each internal signal has exactly one
consumer. This is the simplest correct generator but produces
unrealistic, bloated RTL — every gate's output is used once, then
forgotten.

Real designs are DAGs. A wire is computed once and read many times.
Phase 3 adds this.

## The mechanism

The `SignalPool` already exists from Phase 1 (it holds primary inputs
and flop-Qs for terminal selection). In Phase 3, every gate output
also enters the pool. When `pick_terminal` is called during cone
recursion, with probability `share_prob` it picks an existing pool
entry of the right width instead of creating fresh logic via gate
recursion.

```
pick_terminal(rng, knobs, width, pool):
    candidates = pool.signals_of_width(width)
    if not candidates.empty() and rand() < knobs.share_prob:
        return pick_one(candidates)   // SHARING
    ...                                // fall through to other terminals
```

That's the whole feature. Two lines of logic; the rest is bookkeeping.

## Dep-set propagation through sharing

When a gate is shared, its `DepSet` is shared too. The consumer's deps
become the union of *all* its operands' deps, including the shared
node's deps. Non-triviality continues to hold by the same argument as
before: the dep-set is a structural property of the IR, independent
of how many consumers a node has.

## Knob calibration

`share_prob` of 0.0 produces pure trees (Phase 1 behavior). 1.0 would
produce maximum sharing — almost no fresh logic per cone, just
combinations of pool entries. Useful range is 0.2–0.5.

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
  `==`, etc. — even if that node is shared from the pool. The check
  is on `NodeId` equality, which catches sharing-induced collapses.
- Mux arms cannot be the same node.

Without these, sharing would *increase* the rate of trivial collapse
because reusing the same wire on both sides of an XOR is now likely.

## What sharing does *not* do

It does not deduplicate equivalent sub-expressions that happen to be
generated independently. If two cones both build `(i_0 + i_1)` from
scratch, they remain two separate gates. Common-subexpression
elimination is the synthesizer's job, not the generator's.

If you want pre-deduplicated output for some reason, run a
canonicalization pass over the IR after generation. `anvil` does not
ship one; it would be a reasonable contribution.
