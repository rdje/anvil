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

    // Construction-time hash-consing tables (Rule 21).
    // Key → Vec<NodeId> where the vector length is capped at
    // `max_ast_instances`. Callers go through `intern_gate` /
    // `intern_constant` rather than pushing into `nodes` directly.
    gate_instances:  HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>,
    const_instances: HashMap<(u32, u128),              Vec<NodeId>>,

    // Per-module knob mirrors (populated from Config at module
    // creation, immutable thereafter).
    pub max_ast_instances:        u32,  // Rule 21
    pub mux_arm_duplication_rate: f64,  // Rule 22
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

## Node construction: `intern_gate` / `intern_constant`

All `Node::Gate` and `Node::Constant` creation in the generator
goes through two methods on `Module`:

```rust
impl Module {
    pub fn intern_gate(
        &mut self,
        op: GateOp,
        operands: Vec<NodeId>,
        width: u32,
        deps: DepSet,
    ) -> (NodeId, /* is_new: */ bool);

    pub fn intern_constant(
        &mut self,
        width: u32,
        value: u128,
    ) -> (NodeId, /* is_new: */ bool);
}
```

Semantics (Rule 21):

- The pair `(op, operands, width)` is the **gate AST key**.
  The pair `(width, value)` is the **constant AST key**.
- Each key maintains a `Vec<NodeId>` of instances already
  created. On call:
  - If `vec.len() < max_ast_instances`: create a new
    `Node::Gate` / `Node::Constant`, append its `NodeId` to the
    vector, return `(node_id, true)`.
  - Otherwise: return `(*vec.last(), false)` — route the caller
    to the most-recently-created existing instance.
- Default `max_ast_instances = 1` → strict CSE (one AST = one
  node). Higher values permit bounded duplication;
  `u32::MAX` disables dedup.
- Callers that also maintain a `SignalPool` must call
  `pool.add(node_id, width, deps)` only when `is_new` is true.

Why the dedup tables live on `Module`: they are the authoritative
record of what ASTs exist. A bare `Vec<Node>` can have two
structurally-identical entries; with the tables, CSE is
structurally enforced and observable (via metrics —
`max_gate_ast_multiplicity`, `max_constant_ast_multiplicity`).

**Snapshot contract:** `build_cone_with_retry` rewinds `nodes`,
`flops`, pool, and worklist on empty-dep-root retries. It **must**
also restore `gate_instances` and `const_instances` — otherwise
stale keys point at now-truncated `NodeId`s, and a subsequent
intern call returns a different node than the key promises. This
is a load-bearing invariant.

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

Name generation is deterministic and **typed per gate kind** (Rule 12):

- `clk`, `rst_n` — clock and async-reset input ports (emitted only
  when `!m.flops.is_empty()`).
- `i_0`, `i_1`, … — primary data inputs.
- `o_0`, `o_1`, … — primary outputs.
- `<gate_kind>_<N>` — internal gate wire. `<gate_kind>` is the
  lowercase `GateOp` name (`and`, `or`, `xor`, `not`, `add`,
  `sub`, `mul`, `eq`, `neq`, `lt`, `gt`, `le`, `ge`, `mux`,
  `slice`, `concat`, `red_and`, `red_or`, `red_xor`, `shl`,
  `shr`), and `<N>` counts per kind from 0 within the module.
- `flop_<id>` — flop register, indexed by `FlopId`.

Each kind carries its own counter — `and_0`, `mux_0`, `slice_0`
coexist without collision. Per-kind counts are assigned by a
single walk of `m.nodes` at emission time (`build_names` in
`src/emit/sv.rs`); the walk's output is a pure function of
declaration order, which is a pure function of `(seed, knobs)`.

Deterministic names make diffs between seeds readable. The
`<gate_kind>_<N>` naming also makes the gate mix visible at a
glance — a module header full of `and_0…and_12; mux_0…mux_3;
flop_0…flop_9` tells you the shape of the logic without further
inspection.

## Future extensions (durable roadmap, not yet implemented)

The IR is deliberately minimal today — typed scalar vectors plus
flops, drawn from the synthesizable SV subset. Two major axes
of extension are on the roadmap and will require structural IR
changes when they land. Both are supported goals; order is not
fixed.

### Parameters and generics (Phase 5)

Goal: emit modules that take `parameter` declarations and are
reused at multiple widths / depths. Requires:

- A **width-expression language** in the IR. Today widths are
  `u32`; parameterised widths are arithmetic expressions over
  parameter symbols (`W`, `W+1`, `$clog2(N)`, `2*W-1`, …).
  Candidate: small expression-tree enum
  `WidthExpr::{Literal(u32), Param(ParamId), Add, Sub, Mul, Div,
  Clog2, Max, Min}`.
- A **parameter environment** per module describing the legal
  ranges for each parameter, plus a substitution pass at
  instantiation time that resolves `WidthExpr` → `u32`.
- **Emitter changes**: render parameter declarations in the
  module header and substitute symbolic widths in `logic [W-1:0]`
  where appropriate.
- **Hard prerequisite: Phase 4 (hierarchy)**. Parameters only
  matter at instantiation — without hierarchy, every module is
  stand-alone and there is nothing to parameterise over.

Value: stresses the parameter-resolution and elaboration code
paths in every downstream tool. Generator output becomes
reusable across widths.

Cost: significant. The width-expression language propagates
through every width check, every gate constructor, every
validator rule, and the emitter.

### Synthesizable aggregates

Aggregates split into four sub-questions with very different
economics. Recorded here so the decision trail survives future
sessions.

- **Packed `struct` / `union` / `array`** — syntactically
  distinct, semantically equivalent to a flat bit vector
  (synthesis treats them as concatenation with named
  field-access sugar). Adding them is **mostly an emitter-layer
  change**: declare a struct type, render field access as
  `.field` instead of `[hi:lo]`, keep the IR flat. No width-
  expression machinery required; no Phase-4 dependency. The
  primary value is **parser / elaboration coverage** in
  downstream tools, not new synthesis behavior. Cheap, standalone,
  can land early (roughly Phase-3-adjacent).
- **Unpacked arrays** — `reg [W-1:0] mem [0:D-1]` — is the
  **memory-inference pattern** and is already on the roadmap as
  Phase 6's memory motif. Stresses memory-inference heuristics
  in synthesizers (SRAM vs flops, single-port vs dual-port).
  This is the "real" array work.
- **Unpacked `struct` / `union` for datapath** — mostly
  non-synthesizable (tool-dependent). Not pursued.
- **Enums** — synthesizable but thin; they are typed constant
  sets with no distinct stress value beyond what constants
  already provide. Deprioritised.

### Blocks as first-class IR nodes

Not a data-type question but an organisational one: today every
"block" (comb mux, priority encoder, flop-with-mux) lowers to
`Node::Gate` / `Flop` at construction time, flattening into one
namespace. When blocks become first-class, the IR gains either a
`Node::Block { kind, instance_id, ports, internal_ids }` variant
or a scoped sub-module representation. Two emission modes must
both be supported (see session feedback memory):

- **Hierarchical** — block is its own `Module`, parent emits
  `block_kind u_<tag> (...)` with a port map.
- **Flatten with instance-tagged mangling** — internals inlined
  into the parent, names of the form
  `<block-type>_<block-ID>_<signalname>` to preserve identity
  and avoid collisions. Unprefixed flatten is forbidden.

The block-type abbreviation must not collide with any gate-kind
name (trivially true for multi-word names like `priority_encoder`,
`comb_mux`; a lint check goes into the block-kind registry when
the feature lands).

These three extensions are orthogonal. Parameters require
Phase 4 (hierarchy); packed aggregates and the block-emission
modes can land independently. Order is a scheduling decision,
not a technical one.
