# Construction Strategies

`anvil` supports (or will support) four named strategies for
constructing a module's internal logic. The strategies differ in
**when** gates are created relative to each other, and consequently
in **how symmetric** cross-output sharing is.

The strategy is selectable per run via a knob (`construction_strategy`
— CLI flag planned). The current behavior is **`sequential`**. The
planned default, once implementation lands, is **`graph-first`**.

## Why this is a choice, not a detail

"How we build the circuit" is not a user-visible property of the
*output* — a generated SV module is just a DAG — but it profoundly
shapes the *distribution* of outputs. A construction strategy that
builds output 0's cone to completion before starting output 1 will
systematically produce less cross-output sharing in output 0's cone
than in output 1's cone, because output 1 sees more candidate signals
when its leaves are picked. That declaration-order bias is a
construction artifact, not a property any user would ask for.

The four strategies below are distinct engineering choices for
managing (or removing) that artifact.

## `sequential`  *(current behavior)*

**How:** Build cones one output at a time in **declaration order**
(`output 0` first, then `output 1`, …, then `output n_out-1`). Each
cone uses the full depth-first recursion with the shared
`SignalPool`. Each new gate is added to the pool immediately, so
later-declared outputs see more sharing candidates than earlier-
declared ones.

**Removes:** nothing — this is the construction artifact itself.

**Retains:** full declaration-order bias and within-module ordering
asymmetry.

**Complexity:** the simplest strategy; also the current implementation.

**When to pick:** reproducibility of `anvil` output against prior
runs that were generated before the other strategies landed; or when
you *want* a systematic bias between output port indices (e.g., to
exercise tooling that treats port indices specially).

**Name rationale:** "sequential" captures the key property — outputs
are built one-after-another in a fixed order. Not to be confused
with anti-parallelism; every strategy is sequential at the
instruction level. The distinguishing property here is the fixed
declaration-order iteration over outputs.

## `shuffled`

**How:** Build cones one output at a time, but visit the outputs in a
random permutation of the declaration order rather than `0, 1, ...,
n_out-1`. Per-output cone construction is the same depth-first
recursion as today. Record results in declaration order for emission.

**Removes:** declaration-order bias. Any given seed picks a permutation;
averaged across a corpus, every output has equal probability of being
first-built or last-built.

**Retains:** within-module ordering asymmetry. Some output is
still first-built and has fewer sharing candidates than the
last-built one. The asymmetry is randomized, not eliminated.

**Complexity:** trivial. ~5 lines of code over the current
implementation. Arena-index monotonicity (Rule 1) preserved.

**When to pick:** when you want reproducible per-output depth-first
cone shapes (e.g., for deterministic subsetting of a generated
module), and you accept that per-module sharing is asymmetric.

## `interleaved`

**How:** Maintain an explicit work queue of pending frames across all
cones. A *frame* is a pending recursion step: "build a gate of width
W for position P in cone K's operand list". At each construction
step, pop a random frame from the queue, process it — build one gate
or one terminal, possibly enqueue child frames for its operands —
and continue until all queues are empty.

**Removes:** within-module ordering asymmetry. Cones grow in lockstep.
Each cone's leaves (chosen late, when the pool is mostly-built) see
gates built by every other cone's earlier frames. Sharing is
near-symmetric per-module.

**Retains:** a per-output cone conceptual structure (each gate still
belongs to a "cone" — the one whose root will eventually reach it).

**Complexity:** moderate. Requires converting the recursive
`build_cone` into an explicit frame-pushing state machine with
placeholder operand slots and a completion mechanism. Arena-index
monotonicity (Rule 1) still holds — gates are added in some total
order and only reference earlier ones — but the order is now
interleaved rather than per-output depth-first.

**When to pick:** when you want per-output cone shapes but with
symmetric cross-output sharing.

## `graph-first`  *(default)*

**How:** Build a pool of K gates with no per-output structure. Each
new gate's operands are picked from existing pool entries
(arena-monotonic). Once the pool has grown enough, pick a drive-root
for each output by selecting a pool entry of matching width with
non-empty deps (falling back to the lazy width-adapter if none
matches).

**Removes:** all construction ordering asymmetry. There are no "cones"
during construction — the circuit is one monolithic DAG. Sharing is
truly symmetric because every output picks its drive-root from the
same completed pool, and every gate's operands came from the same
pool.

**Retains:** the cone-per-output conceptual view — the fanin-reachable
subset of the pool from output K's drive-root is the "cone of output
K". It's a retrospective view rather than a construction axis.

**Complexity:** high for the implementation, but actually *simpler*
as a mental model. The `max_depth` knob re-interprets: depth emerges
from gate-creation order and operand-selection patterns rather than
being a direct recursion-depth bound. A new knob (`target_nodes` or
`pool_growth_size`) replaces the per-cone depth budget as the primary
size control.

**When to pick:** when you want the most realistic shared-DAG output
and are willing to let cone depth emerge rather than bound it
directly.

**Why it is the default:** the user-visible output of `anvil` is a
DAG. Generating the DAG directly rather than through per-output
recursion matches the object being generated. The cone-per-output
construction idiom is a human-friendly fiction; `graph-first` drops
the fiction where it no longer helps.

## Comparison table

| Strategy       | Declaration-order bias | Within-module symmetry  | Implementation cost |
|----------------|------------------------|-------------------------|---------------------|
| `sequential`   | present (systematic)   | asymmetric (systematic) | already implemented |
| `shuffled`     | removed                | asymmetric (randomized) | trivial             |
| `interleaved`  | removed                | near-symmetric          | moderate            |
| `graph-first`  | removed                | symmetric               | high (planned default) |

## Interaction with existing rules

All three strategies preserve the structural rules catalog:

- **Rule 1 (Combinational no-loop):** arena-index monotonicity holds
  in all three. Gates are added in some total order; operands always
  reference earlier-added gates.
- **Rule 9 (Non-triviality):** enforced per-output. `shuffled` and
  `interleaved` use the existing `build_cone_with_retry` retry;
  `graph-first` retries drive-root selection if no matching-width
  dep-bearing entry exists and adapter construction also fails.
- **Rule 16 (Module-wide sharing):** strengthened by `interleaved`
  and `graph-first` (symmetric); partially-applied by `shuffled`
  (declaration-order-independent but still per-seed-ordered).

## Implementation status

- `sequential` — **implemented**. Current default. CLI:
  `--construction-strategy sequential`.
- `shuffled` — **implemented**. Builds cones in a seeded random
  permutation of declaration order. CLI:
  `--construction-strategy shuffled`.
- `interleaved` — **implemented**. Frame state machine: output cones
  share one global work queue; each step pops a random `SignalFrame`
  and processes it. Gates pending more operands live in an in-flight
  table; when the last operand resolves, the gate finalizes. CLI:
  `--construction-strategy interleaved`. **Scope note:** block
  internals (flop D-cones, comb-mux sub-cones) still build
  depth-first; only *output-cone* frames interleave. Full symmetry
  (including block internals) awaits `graph-first`.
- `graph-first` — planned. Architectural shift from per-output cones
  to a pool-first DAG. Becomes the default when it lands.

When `graph-first` lands, the `construction_strategy` knob default
will flip, and a user who wants prior behavior pins to
`--construction-strategy sequential`. Reproducibility of any
previously-generated output against its original seed + knobs is
guaranteed because the effective knobs are recorded in the manifest.

See `MEMORY.md` next-up list for the current implementation sequence.
