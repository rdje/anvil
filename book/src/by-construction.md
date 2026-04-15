# Generation by Construction

The phrase means: the generator never produces something invalid and
then checks it — instead, every step of the generation process only
makes choices that are *guaranteed* to be valid by the rules built
into the generator itself.

Contrast two approaches:

**Generate-then-filter (naïve):** produce random SystemVerilog text
from a grammar, then try to parse/typecheck it, throw away the 99%
that's broken, keep the 1% that happens to be valid. Hopeless for RTL
because semantic constraints are tight enough that random text almost
never satisfies them.

**By construction (what `anvil` does):** at every decision point, the
generator only offers itself choices that maintain the invariants.
Concretely:

- When generating an `Add` node, the generator first decides the target
  width W, then generates both operands at width W. There is no moment
  when the widths could mismatch — the width is an *input* to the
  operand generator, not something checked afterward.
- When referencing a signal, the generator picks from the `SignalPool`
  of signals that actually exist with the required width. There is no
  moment when it could reference an undeclared name.
- When generating a `Slice`, the generator first picks a source node
  (with known width K), then picks `hi` and `lo` within `[0, K)`.
  There is no moment when the slice could be out of bounds.
- When generating a `Mux`, the generator forces the selector to be
  1-bit and both arms to match the target width.

The validity proof is **structural**. Reading the cone builder, you
can see that every code path preserves the invariants by definition.

## The validator is a safety net, not a gate

`anvil` will likely include an IR validator (`src/ir/validate.rs`).
Its purpose is not to reject generator output — it is to catch
generator bugs during development. If the validator ever fails on
real generator output in production use, that is a bug filed against
the generator, not expected behavior.

In CI, every test runs the validator after generation and fails the
build if it rejects anything. This converts invariant violations from
silent bugs into loud test failures.

## Why not just validate after generation?

You could. But then:

- You'd need a retry loop on invalid output.
- You'd have no bound on how long generation takes (some seeds might
  produce mostly-invalid output that gets filtered to nothing).
- The generator's author would be tempted to rely on the validator
  rather than maintaining invariants, leading to silent correctness
  drift.
- Complex invariants (like dep-set non-emptiness) become much more
  expensive to check post-hoc than to maintain incrementally.

By construction is simpler, faster, and more honest about what the
generator guarantees.
