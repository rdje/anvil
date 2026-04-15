# The Circuit IR

The IR is the circuit. Everything else — generator, emitter, future
validators — operates on it.

## Core types

```rust
pub struct Module {
    pub name:    String,
    pub inputs:  Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock:   Option<PortId>,   // Phase 2+
    pub reset:   Option<PortId>,   // Phase 2+
    pub nodes:   Arena<Node>,      // all internal signals
    pub flops:   Vec<Flop>,        // Phase 2+
    pub drives:  Vec<(PortId, NodeId)>, // which node drives each output
}

pub struct Port {
    pub id:    PortId,
    pub name:  String,
    pub width: u32,
    pub dir:   Direction, // In | Out
}

pub enum Node {
    PrimaryInput { port: PortId, width: u32 },
    Constant     { width: u32, value: u128 },
    FlopQ        { flop: FlopId, width: u32 },
    Gate {
        op:       GateOp,
        operands: Vec<NodeId>,
        width:    u32,
        deps:     DepSet,
    },
}

pub enum GateOp {
    // Bitwise
    And, Or, Xor, Not,
    // Arithmetic
    Add, Sub, Mul,
    // Comparison (output is 1-bit)
    Eq, Neq, Lt, Gt, Le, Ge,
    // Structured
    Mux,                     // [sel, a, b]
    Slice { hi: u32, lo: u32 },
    Concat,                  // variadic
    // Reductions (output is 1-bit)
    RedAnd, RedOr, RedXor,
    // Shifts
    Shl, Shr,                // [value, amount]
}

pub struct Flop {
    pub id:        FlopId,
    pub width:     u32,
    pub d:         Option<NodeId>,  // filled after D-cone generation
    pub q:         NodeId,           // the FlopQ node
    pub reset_val: u128,
    pub reset_kind: ResetKind,       // Sync | Async | None
}
```

## Why an arena

`Arena<Node>` stores nodes contiguously, hands out `NodeId` indices,
and allows cheap sharing (fanout). A naïve tree representation with
`Box<Node>` children would force copying when a wire feeds multiple
consumers; the arena lets many `NodeId` references point at the same
node.

This matters for Phase 3 (sharing) but is worth building in from Phase 1
— the extra complexity is small and retrofitting is annoying.

## Dependency sets

```rust
pub struct DepSet(BitSet);   // indexed by PrimaryInput::port
```

Every `Node::Gate` caches its `DepSet` at construction time. The cache
avoids recomputing dep-sets during the non-triviality check and during
future optimizations (e.g., detecting shared sub-cones).

`FlopQ` is treated as a virtual dep source — its deps are the union of
its D-cone's deps, but for the purpose of the *current* combinational
cone's non-triviality, a flop-Q reference contributes one dep.

## Invariants

Enforced by IR constructors (not checked after the fact):

1. `Gate.operands[i].width == expected_input_width(op, Gate.width, i)`
2. `Gate.deps == union(operands[i].deps)` — plus the flop virtual
   contribution for any `FlopQ` in the operand set.
3. Every `NodeId` referenced as an operand exists in `Module.nodes`.
4. Every `PortId` in `drives` is an output port.
5. Each output port appears in `drives` exactly once.
6. Each `Flop::d` is filled before emission (worklist drained).

Violation of any of these is a generator bug, not invalid user input.
The constructors `panic!` on violation to catch generator bugs loudly
during development.

## Emitter contract

The emitter is a pure function `Module -> String`. It assumes all
invariants hold. It does not validate. It does not reject anything.
If the IR is valid, the emitted SV is valid.

Name generation is deterministic: `i_0`, `i_1`, … for inputs;
`o_0`, `o_1`, … for outputs; `w_0`, `w_1`, … for internal wires;
`r_0`, `r_1`, … for flops. This makes diffs between seeds readable.
