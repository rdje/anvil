# Sequential Logic: Flops and Cone Boundaries

Flops are part of the same fanin-cone recursion as combinational logic
— they are not a later phase. The recursion principle handles them
naturally:

- **Q is a leaf in the *current* cone.** When the recursion picks
  "this signal is driven by a flop," the flop's Q output terminates
  the descent for this cone, exactly like a primary input does.
- **D opens a *new* cone.** The flop's D input becomes a fresh sub-cone
  rooted at D, queued on a worklist for later construction by the same
  `build_cone` function. That sub-cone may itself contain flops; their
  Ds get queued in turn.
- **The worklist drains to quiescence.** The main loop pops flops one
  at a time and recursively builds their D-cones until no flops remain
  pending.

This is why we did not split sequential into a later phase: it is the
same recursion with one extra choice in the node-kind picker.

## Synchronous-design discipline

Every module is **fully synchronous to a single clock domain**:

- One `clk` input port (1 bit, posedge).
- One `rst_n` input port (1 bit, async, active-low).
- *Every* flop in the module uses these two ports — no per-flop clock
  selection, no per-flop reset polarity choice, no mixed sync/async.
- All flops emit into a single `always_ff @(posedge clk or negedge rst_n)`
  block per module.

Multi-clock and CDC-safe handshakes are deferred to a much later phase
(Phase 6) and are optional even then.

## Why this discipline

A real synchronous digital design — the kind that ships in silicon —
has exactly this shape: one clock, one reset, all sequential elements
clocked together. Generating anything else risks producing modules
that are technically synthesizable but structurally unrealistic, and
that exercise tooling paths (CDC checks, mixed-edge timing) that are
out of scope for `anvil`'s mission.

Forcing the discipline by construction — there is no IR field for
per-flop clock or per-flop reset polarity — guarantees that no random
choice can violate it.

## Cone boundaries

Without flops, every primary output is the root of one combinational
cone that recurses until it hits primary inputs. A module is one
forest of combinational cones, one per output.

With flops, a cone can terminate at a flop's Q output. From the
perspective of the cone being built, Q is a leaf — same as a primary
input. But the flop itself has a D input, which must be driven by
*another* combinational cone, generated separately.

The result is a circuit with multiple "cone regions" stitched together
by flops. This matches the standard definition of synchronous digital
logic: combinational logic between registers.

## The worklist

```
flop_worklist = Queue::new()

# generate output cones first
for out in outputs:
    drive_cone(out, build_cone(...))

# drain flop worklist
while not flop_worklist.empty():
    flop = flop_worklist.pop()
    drive_cone(flop.d, build_cone(...))
```

Inside `build_cone`, when the recursion picks "this node is a flop":

1. Allocate a new `Flop` with the requested width.
2. Add it to the module.
3. Push it onto the worklist (its D-cone will be generated later).
4. Add its Q to the signal pool (so subsequent cones can share it).
5. Return the FlopQ node as the chosen sub-expression.

The worklist may grow during draining (a D-cone may itself contain
flops). The loop terminates because there is a maximum recursion depth
on each cone, the gate set is finite, and `flop_prob` < 1.

## Flop reuse

Once a flop exists, subsequent cones (other outputs, other flops' Ds)
may pick that flop's Q from the signal pool. This is how sequential
circuits get *shared state* — multiple downstream signals reading the
same register.

Without reuse, every output would have its own private flop chain,
producing unrealistic and bloated designs. Reuse probability is
controlled by the `--share-prob` knob.

Under `identity_mode = node-id`, there is a second sequential-sharing
path after drain: if two flops ended up with the same exact state
signature, they are merged even if they were born as distinct
registers. That is exact-signature only, not full sequential
equivalence.

## Clock and reset

Exactly one clock and one reset, declared as ports of the module
whenever the module contains at least one flop:

```systemverilog
module mod_42_0007 (
    input  logic        clk,
    input  logic        rst_n,
    input  logic [7:0]  i_0,
    ...
    output logic [7:0]  o_0,
    ...
);

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            flop_0 <= 8'h0;
            ...
        end else begin
            flop_0 <= add_3;
            ...
        end
    end
```

Every flop uses `clk` (posedge). Every flop uses `rst_n`
(async, active-low). Reset value is chosen per flop, biased toward 0.
There is no per-flop choice of clock domain or reset polarity — see
"Synchronous-design discipline" above.

(Flop names are `flop_<id>` per Rule 12; the D-driving wire is a
gate named `<kind>_<N>` — `add_3` above stands in for whatever op
the generator picked. See [Rule 12](structural-rules.md) for the
full naming contract.)

When a module happens to be generated with zero flops, the `clk` and
`rst_n` ports are omitted from the port list. This avoids spurious
"unused input" lint warnings on the combinational-only outputs.

Multi-clock and CDC are explicitly out of scope until Phase 6 (and
optional even then).

## Combinational cycles

Forbidden by construction. The cone recursion only references signals
that already exist in the pool when picked. The pool only contains
primary inputs, flop-Qs, and previously-generated internal wires from
*earlier* recursions. Newly-created gates are added to the pool only
*after* their operands are resolved. Therefore no gate can transitively
reference itself through pure combinational logic.

Sequential cycles (state machines) are *expected*: a flop's D can
reference its own Q, and that's a valid storage element. The flop
breaks the loop temporally.

## Flop motifs: M-to-1 mux on D

Every flop's D input is driven by either:

- **M = 0** — no mux; D is generated by a single recursive cone of width N
  (the simplest case, equivalent to a standard register).
- **M >= 2** — an M-to-1 mux in one of two encoding styles. M = 1 is
  excluded by design (a 1-arm mux is just a wire).

The encoding style is picked per-flop (`flop_mux_encoding_prob` knob):

- **One-hot style**: M 1-bit select bits, each a recursion point. The
  design contract is that at most one select fires at a time. Assembled
  as `OR_i({N{sel_i}} & data_i)` plus an optional Q-feedback term.
- **Encoded style**: one select bus of width `ceil(log2(M))`, a single
  recursion point. Value `k` routes `data_k` onto D. When `sel` falls
  outside `[0, M)` (possible when M is not a power of 2), the
  fall-through routes 0 (ZeroDefault) or Q (QFeedback).

Within each encoding style, `FlopKind` further chooses the
"no-valid-selection" behavior (ZeroDefault vs QFeedback). The two axes
are orthogonal:

| Style    | Kind          | No-select behavior                          |
|----------|---------------|---------------------------------------------|
| OneHot   | ZeroDefault   | D = 0 when every sel is 0                   |
| OneHot   | QFeedback     | D = Q when every sel is 0                   |
| Encoded  | ZeroDefault   | D = 0 when sel >= M; D = data_k when sel=k  |
| Encoded  | QFeedback     | D = Q when sel >= M; D = Q when sel=0; D = data_k when sel=k for k in [1, M) |

In the Encoded + QFeedback case, the slot at index 0 is *replaced* by
Q — there is no `data_0` sub-cone; the recursion builds only M-1
data sub-cones (indices 1..M).

The one-hot variants below describe the assembled gate tree; the
encoded variants use a chained ternary over `Eq(sel, k)` for each k.

### Kind 1 — `ZeroDefault`
```
D = ({N{sel_0}} & data_0) | ({N{sel_1}} & data_1) | ... | ({N{sel_{M-1}}} & data_{M-1})
```
When all M selects are 0, all AND-masked terms are 0 and D = 0. The
flop loads zero on the next clock edge.

### Kind 2 — `QFeedback`
```
none_selected = ~(sel_0 | sel_1 | ... | sel_{M-1})
D = ({N{sel_0}} & data_0) | ... | ({N{none_selected}} & Q)
```
When all M selects are 0, `none_selected` is 1 and the Q-feedback term
holds: D = Q. The flop holds its current value.

### Recursion structure

When M = 0, the flop's D is built by a single recursive cone of width
N. When M >= 2, every one of the M N-bit data entries and every one
of the M 1-bit select bits is a **recursion point**:

```
build_flop_d(width N, kind):
    M = pick_M()                              // 0 or 2..=max_mux_arms
    if M == 0:
        return build_cone_with_retry(N)    # Q can be a leaf freely
    arms = []
    for i in 0..M:
        data_i = build_cone_with_retry(N)  # Q can appear any number of times
        sel_i  = build_cone_with_retry(1)  # Q can appear any number of times
        arms.push(data_i, sel_i)
    return assemble(arms, kind, Q)
```

The assembly step builds `replicate-AND-OR` gate trees from the
recursively-generated leaves. Sub-cones may themselves spawn flops,
which are queued on the same worklist; the drainer loops to quiescence.

### One-hot is a contract, not enforced

The select bits are recursively generated and not constrained to be
mutually exclusive. The hardware *assumes* they are one-hot (this is a
design contract). When the contract is violated, multiple data paths
OR together — which is exactly what the gate structure produces. There
is no extra logic in the generator to enforce one-hot at runtime.

### Q-feedback in the D-cone is freely permitted

A flop's own Q may appear **any number of times** as a leaf inside
any of its data, select, or direct-D sub-cones. Q→D feedback through
arbitrary combinational logic is a legal synchronous pattern —
counters, toggles, accumulators, state machines all work this way.
The clock edge breaks the loop temporally: `Q[n+1]` depends on
`Q[n]` plus possibly other inputs.

This is independent of the explicit Q-feedback mux term in
`FlopKind::QFeedback`. The mux term fires when *no* select asserts
and is a structured fall-through path. The Q-in-sub-cone freedom
just makes Q a normal shareable leaf during cone construction.

Combinational self-reference within the Q→D logic is still impossible
— that is Rule 1 (Combinational no-loop) in the
[Structural Rules catalog](structural-rules.md).

See Rule 2 in the [Structural Rules catalog](structural-rules.md) for
the authoritative statement.

## Reset value selection

Reset values are chosen randomly per flop, with a bias toward zero
(zero is by far the most common reset value in real designs). The
current distribution: 50% zero, 25% all-ones, 25% other random value.

This is a knob if it ever needs tuning, but the default is sensible.
