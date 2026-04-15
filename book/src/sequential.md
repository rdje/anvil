# Sequential Logic: Flops and Cone Boundaries

A flop changes the structure of the generation algorithm in one
specific way: it terminates the current combinational cone and opens
a new one.

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

Phase 2 uses exactly one clock and one reset, declared as ports of
the module:

```systemverilog
module mod_42_0007 (
    input  logic        clk,
    input  logic        rst_n,
    input  logic [7:0]  i_0,
    ...
    output logic [7:0]  o_0,
    ...
);
```

Every flop uses `clk`. Every flop with a reset uses `rst_n`. Reset
kind (sync vs async, present vs absent) is chosen per flop.

Multi-clock and CDC are explicitly out of scope until Phase 7 (and
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
distribution: 50% zero, 25% all-ones, 25% other random value.

This is a knob if it ever needs tuning, but the default is sensible.
