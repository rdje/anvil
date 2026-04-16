# Construction Strategies

`anvil` supports three named strategies for constructing a module's
internal logic. The strategies differ in **when** gates are created
relative to each other, and consequently in **how symmetric**
cross-output sharing is.

The strategy is selectable per run via `--construction-strategy`.
The default is **`interleaved`**. A fourth value, `graph-first`, is
retained as a silent alias for `interleaved` (see the retired-
strategy section at the end of this chapter).

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

## `sequential`

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

## `interleaved` *(default)*

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

## Comparison table

| Strategy       | Declaration-order bias | Within-module symmetry  | Status |
|----------------|------------------------|-------------------------|--------|
| `sequential`   | present (systematic)   | asymmetric (systematic) | implemented |
| `shuffled`     | removed                | asymmetric (randomized) | implemented |
| `interleaved`  | removed                | near-symmetric          | **default** |

## Interaction with existing rules

All three strategies preserve the structural rules catalog:

- **Rule 1 (Combinational no-loop):** arena-index monotonicity holds
  in all three. Gates are added in some total order; operands always
  reference earlier-added gates.
- **Rule 9 (Non-triviality):** enforced per-output via
  `build_cone_with_retry`.
- **Rule 16 (Module-wide sharing):** strongest in `interleaved`
  (near-symmetric cross-cone sharing via the global frame queue);
  partial in `shuffled` (declaration-order-independent but still
  per-seed-ordered); weakest in `sequential` (systematic
  declaration-order bias).
- **Rule 18 (No orphan gates):** all three strategies are
  demand-driven — every gate is created to fulfil a specific
  consumer demand. `build_cone` snapshots state before operand
  construction and rolls back on anti-collapse rejection;
  `process_signal_frame` (the interleaved frame machine) delivers
  an existing operand as the anti-collapse fallback rather than
  creating a new node. Zero orphans across 4 strategies × 6 seeds
  at default knobs.

## Retired: `graph-first`

An earlier fourth strategy, `graph-first`, grew a pool of top-level
units *before* any drive-roots were picked, with each unit's
operands taken from the current pool. It produced the most
symmetric cross-output sharing but was *speculative*: the pool
contained units with no guaranteed consumer, and 13–27 % of gates
per module ended up as orphans (Rule 18 violation — see slice
`b78550d`).

**Status:** retired. The CLI accepts `--construction-strategy
graph-first` as a silent alias for `interleaved` so existing
scripts / configs keep working; internally the speculative code
path is unreachable.

**Why not just fix graph-first?** A demand-driven version of
graph-first *is* `interleaved` — the only way to guarantee every
pool unit has a consumer is to drive construction from consumer
demand, which is exactly what the frame-queue machinery does.
Keeping graph-first as a separate variant would duplicate code
without adding behavioural distinction.

**If you want graph-first's symmetric-sharing property:** use
`--construction-strategy interleaved` (the default). It gives you
the same cross-cone sharing guarantee, built demand-first, with
zero orphans.

## Implementation status

- `sequential` — **implemented**. CLI:
  `--construction-strategy sequential`.
- `shuffled` — **implemented**. Builds cones in a seeded random
  permutation of declaration order. CLI:
  `--construction-strategy shuffled`.
- `interleaved` — **implemented, default**. Frame state machine:
  output cones share one global work queue; each step pops a random
  `SignalFrame` and processes it. Gates pending more operands live
  in an in-flight table; when the last operand resolves, the gate
  finalizes. CLI: `--construction-strategy interleaved`. **Scope
  note:** block internals (flop D-cones, comb-mux sub-cones) still
  build depth-first; only *output-cone* frames interleave.
- `graph-first` — **retired** (see above). Silent alias for
  `interleaved`. The `--graph-first-pool-size` knob is still
  accepted but no longer read.

Reproducibility of any previously-generated output against its
original seed + knobs is guaranteed because the effective knobs are
recorded in the manifest. Output produced under the old graph-first
strategy is no longer bit-reproducible against HEAD — the strategy
name still works but now routes to interleaved. For historical
reproducibility, pin the exact pre-`b78550d` commit.
