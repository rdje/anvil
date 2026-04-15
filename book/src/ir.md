# The Circuit IR

The IR is the circuit. Everything else — generator, emitter, validator —
operates on it.

## Core types

```rust
pub struct Module {
    pub name:    String,
    pub inputs:  Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock:   Option<PortId>,
    pub reset:   Option<PortId>,
    pub nodes:   Vec<Node>,                // arena of internal signals
    pub flops:   Vec<Flop>,
    pub drives:  Vec<(PortId, NodeId)>,    // which node drives each output
}

pub struct Port {
    pub id:    PortId,
    pub name:  String,
    pub width: u32,
    pub dir:   Direction,                  // In | Out
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

pub enum ResetKind { None, Sync, Async }   // always Async today

pub enum FlopKind {
    ZeroDefault,             // D = 0   when no select fires
    QFeedback,               // D = Q   when no select fires
}

pub struct MuxArm {
    pub data: NodeId,
    pub sel:  NodeId,        // 1-bit in OneHot; not used for Encoded
}

pub enum FlopMux {
    None,                                         // M = 0 — no mux
    OneHot(Vec<MuxArm>),                          // M >= 2, one-hot select
    Encoded { sel: NodeId, data: Vec<NodeId> },   // M >= 2, encoded select
}

pub struct Flop {
    pub id:         FlopId,
    pub width:      u32,
    pub d:          Option<NodeId>,               // filled by drain
    pub q:          NodeId,                       // the FlopQ node
    pub reset_val:  u128,
    pub reset_kind: ResetKind,
    pub kind:       FlopKind,
    pub mux:        FlopMux,                      // filled by drain
}
```

## Why a flat `Vec<Node>`

`Module.nodes` is a simple `Vec<Node>` indexed by `NodeId: u32`. This
serves two purposes simultaneously:

1. **Arena semantics** — many `NodeId` references can point at the same
   node, enabling cheap DAG sharing (a wire computed once, consumed
   many times). A naïve tree `Box<Node>` representation would force
   copies for fanout.
2. **Serde-friendly** — the IR serializes to JSON one-shot without
   dealing with arena rehydration.

Indices are stable for the lifetime of a `Module` because we only ever
push, never remove. The bounded retry in `build_cone_with_retry`
rewinds by `Vec::truncate`, which is safe because no other code holds
`NodeId`s referring to the rewound region.

## Dependency sets

```rust
pub struct DepSet(BTreeSet<u32>);
```

Indexed by primary-input `PortId` (plus virtual ids for flops, tagged
with the high bit set to live in a disjoint numeric space).

Every `Node::Gate` caches its `DepSet` at construction time. The cache
avoids recomputing dep-sets during the non-triviality check and keeps
dep-set propagation cheap through arbitrarily deep gate trees.

`FlopQ` is treated as a virtual dep source — its deps are
`{virtual_id(flop)}`. For the *current* combinational cone's
non-triviality, a flop-Q reference contributes one dep; the flop's
D-cone enforces its own non-triviality separately when drained.

## Invariants

Enforced by generator constructors (invariant-preserving by
construction), and **also** checked by `ir::validate::validate` as a
development-time safety net.

In the generator:

1. `Gate.operands[i].width` matches the per-op rule (see
   `book/src/algorithm.md`).
2. `Gate.deps == union(operands[i].deps)`.
3. Every `NodeId` referenced as an operand exists in `Module.nodes`.
4. Every `PortId` in `drives` is an output port.
5. Each output port appears in `drives` exactly once.
6. Each `Flop::d` is filled before emission (worklist drained).
7. Each flop's `mux` matches its assembled D shape.

In `validate.rs`:

- All of the above.
- Per-gate **arity**: each `GateOp` variant has a fixed or
  variadic-with-min operand count.
- Per-gate **output width**: `Eq`-family gates produce 1-bit;
  `Slice` produces `hi - lo + 1`; `Concat` produces the sum of
  operand widths; others match output-width from operand widths.
- Per-gate **operand widths**: `Mux` sel is 1-bit, arms match output
  width; comparisons require equal operand widths; `Slice` source
  width must exceed `hi`; bitwise/arithmetic operands equal output
  width; etc.

Violation of any of these is a generator bug. The constructors do
not panic; the validator rejects with a rich error variant (node id,
op, operand index, expected vs got widths).

## Emitter contract

The emitter (`emit::to_sv`) is a pure function `Module -> String`. It
assumes all invariants hold. It does not validate. It does not reject
anything. If the IR is valid, the emitted SV is valid.

Name generation is deterministic:

- `clk`, `rst_n` — clock and async-reset input ports (emitted only
  when `!m.flops.is_empty()`).
- `i_0`, `i_1`, … — primary data inputs.
- `o_0`, `o_1`, … — primary outputs.
- `w_0`, `w_1`, … — internal gate wires (indexed by `NodeId`).
- `r_0`, `r_1`, … — flop registers (indexed by `FlopId`).

Deterministic names make diffs between seeds readable.
