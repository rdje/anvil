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
            r_0 <= 8'h0;
            ...
        end else begin
            r_0 <= w_42;
            ...
        end
    end
```

Every flop uses `clk` (posedge). Every flop uses `rst_n`
(async, active-low). Reset value is chosen per flop, biased toward 0.
There is no per-flop choice of clock domain or reset polarity — see
"Synchronous-design discipline" above.

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

## Reset value selection

Reset values are chosen randomly per flop, with a bias toward zero
(zero is by far the most common reset value in real designs). The
current distribution: 50% zero, 25% all-ones, 25% other random value.

This is a knob if it ever needs tuning, but the default is sensible.
