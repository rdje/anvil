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

`anvil` ships an IR validator (`src/ir/validate.rs`) with an inline
test suite covering a broad rejection surface (undefined drive
roots, canonical flop/`FlopQ` backreferences, per-gate arity,
operand widths, output drive count, flop D filled, output-cone
dep-set non-empty, …). Its purpose is not to reject generator
output — it is to catch generator bugs during development. If the
validator ever fails on real generator output, that is a bug filed
against the generator, not expected behavior.

In CI (`cargo test`), every integration test runs the validator
after generation and fails the build if it rejects anything. This
converts invariant violations from silent bugs into loud test
failures.

### Exemplar: Rule 18 (no orphan gates)

The cleanest illustration of the doctrine in action is Rule 18 ("no
orphan gates in the emitted module"). Two enforcement paths were
considered:

- **(α) Construction-time:** only create a gate when a specific
  consumer is already waiting for it. When a proposed gate fails
  anti-collapse, the operand sub-trees that were speculatively
  built for it are rolled back from `m.nodes` so they can't leak
  as orphans.
- **(β) Emission-time tree-shake:** let the generator produce
  orphans, compute the live set at emission time, emit only the
  live set.

β was rejected — it's a generate-then-filter step, violating the
contract. α was adopted: `build_cone` snapshots
`m.nodes` / `m.flops` / pool / worklist / `gate_instances` /
`const_instances` before operand construction and restores on
anti-collapse rejection; `process_signal_frame` (the interleaved
frame machine) uses one of the existing operand NodeIds as the
fallback instead of creating a new node. Zero orphans across 4
strategies × 6 seeds at default knobs.

See `DEVELOPMENT_NOTES.md` "Rule 18 α construction-time" for the
decision record.

### Grandfather clause: bounded retry

Exactly one construction-time retry exists in the generator:
`build_cone_with_retry` rejects empty-dep-set cone roots and
retries up to 4× before accepting the last attempt. The retry is
bounded and restores a full snapshot on each attempt — it is not
"generate-then-filter" but "generate, fail-fast, retry with fresh
randomness." Any other retry-and-filter pattern in the generator
would be a design regression.

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
