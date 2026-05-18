# The Circuit IR

The IR is the circuit. Everything else — generator, emitter, validator —
operates on it.

## Core types

```rust,ignore
pub struct Design {
    pub top:     String,
    pub modules: Vec<Module>,
}

pub struct Module {
    pub name:    String,
    pub inputs:  Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock:   Option<PortId>,
    pub reset:   Option<PortId>,
    pub nodes:   Vec<Node>,                // arena of internal signals
    pub flops:   Vec<Flop>,
    pub instances: Vec<Instance>,
    pub drives:  Vec<(PortId, NodeId)>,    // which node drives each output

    // Construction-time hash-consing tables (Rule 21).
    // Key → Vec<NodeId> where the vector length is capped at
    // `max_ast_instances`. Callers go through `intern_gate` /
    // `intern_constant` rather than pushing into `nodes` directly.
    gate_instances:  HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>,
    const_instances: HashMap<(u32, u128),              Vec<NodeId>>,

    // Per-module knob mirrors (populated from Config at module
    // creation, immutable thereafter).
    pub max_ast_instances:        u32,                       // Rule 21
    pub mux_arm_duplication_rate: f64,                       // Rule 22
    pub operand_duplication_rate: f64,                       // Rule 8 extended
    pub identity_mode:            IdentityMode,              // Rule 21c coarse mode
    pub factorization_level:      FactorizationLevel,        // Rule 21c

    // --- Post-hoc telemetry (incremented live, surfaced via Metrics) --
    pub priority_encoder_built:    u32,   // block-build counters
    pub comb_mux_one_hot_built:    u32,
    pub comb_mux_encoded_built:    u32,
    pub case_mux_built:            u32,
    pub casez_mux_built:           u32,
    pub for_fold_built:            u32,
    pub fold_identities_applied:   u64,   // ConstantFold layer fires
    pub peephole_rewrites_applied: u64,   // Peephole layer fires
    pub flatten_associative_applied: u64, // Associative layer fires
    pub flops_merged:             u32,    // endpoint-aware flop merges
    pub semantic_gates_merged:    u32,    // bounded semantic gate merges
    pub nodes_compacted:           u32,   // compact_node_ids removals
    pub knob_rolls:                KnobRollCounters, // per-knob attempts/fires
}

pub struct Port {
    pub id:    PortId,
    pub name:  String,
    pub width: u32,
    pub dir:   Direction,                  // In | Out
}

pub struct Instance {
    pub id:     InstanceId,
    pub name:   String,
    pub module: String,                    // child module name
    pub inputs: Vec<(PortId, NodeId)>,     // child input port -> parent node
}

pub enum Node {
    PrimaryInput { port: PortId, width: u32 },
    Constant     { width: u32, value: u128 },
    FlopQ        { flop: FlopId, width: u32 },
    InstanceOutput { instance: InstanceId, port: PortId, width: u32 },
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
    CaseMux,                 // [sel, data_0, data_1, ...]
    CasezMux,                // [sel, value_0, wild_0, data_0, ...]
    ForFold { kind: ForFoldKind, trip_count: u32, chunk_width: u32 },
    Slice { hi: u32, lo: u32 },
    Concat,                  // variadic
    // Reductions (output is 1-bit)
    RedAnd, RedOr, RedXor,
    // Shifts
    Shl, Shr,                // [value, amount]
}

pub enum ForFoldKind { Xor, Or, And, Add }

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

The hierarchy slice is intentionally narrow, but it is no longer just
placeholder structure:

- `Design` is real, not aspirational.
- `Module.instances` and `Node::InstanceOutput` are real parent-side
  IR surfaces.
- Parent modules now build parent-side composition layers over child
  `InstanceOutput` leaves, combinational by default and optionally
  stateful when `hierarchy_parent_flop_prob` is requested.
- The generator supports both the legacy exact depth-1 wrapper lane and
  the bounded recursive hierarchy lane, with library/on-demand child
  sourcing, sibling-routed child-input bindings, parent-composed
  child-input cones, parent-cone helper instances for parent-composed
  child-input cones, direct sibling routes, direct registered
  sibling-route D sources, registered child-input D cones, and
  parent-output cones, including helper sources routed through
  parent-local state for parent outputs and unregistered
  parent-composed child-input logic,
  explicit per-parent helper budgeting, and optional local parent flops.

The remaining open hierarchy work is richer parent-local behavior
beyond the landed state/helper surfaces: helper placement beyond the
current parent-composed child-input / stateful parent-composed
child-input / direct sibling / direct registered sibling / registered
D-cone / parent-output / stateful parent-output seams, broader registered
hierarchy routing, and
hierarchy-aware identity/factorization.

## Node construction: `intern_gate` / `intern_constant`

All `Node::Gate` and `Node::Constant` creation in the generator
goes through two methods on `Module`:

```rust,ignore
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

### CSE semantics (Rule 21, bottom of the ladder)

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

### The full intern pipeline (Rule 21c)

CSE is the bottom of the factorization ladder. The coarse switch
above it is `identity_mode`: at `Relaxed`, the effective level is
forced to `None` and every call allocates a fresh node; at
`NodeId`, the full-factorization doctrine is selected and
`intern_gate` runs the current-build ladder below before the
dedup-table lookup so syntactically-different-but-semantically-
equivalent expressions land on the same CSE key. Lower rungs are
implementation/debug settings inside that doctrine, not a different
meaning of `NodeId`:

1. **Associative flattening** (`>= Associative`): splice
   same-op operands, normalise per-op semantics.
2. **Commutative sort** (`>= Commutative`): sort operands of
   `And`/`Or`/`Xor`/`Add`/`Mul` by `NodeId`.
3. **Constant folding** (`>= ConstantFold`): drop identity
   operands (`x + 0`, `x * 1`, `x & all_ones`, …), substitute
   absorbing constants, with dead gate operands later cleaned up by
   compaction.
4. **Peephole rewrites** (`>= Peephole`): local identities —
   `Not(Not(x))`, `Not(cmp) → inverted cmp`, constant-selector
   `Mux`, unsigned comparison-boundary tautologies, all-constant
   evaluation for comparisons / `Not` / `Slice` / reductions,
   full-width `Slice`, single-operand `Concat`.
5. **Level-None bypass**: at `FactorizationLevel::None`, skip
   every layer above and append a fresh node (diagnostic/stress mode
   inside the `node-id` matrix, not the doctrinal meaning of
   `NodeId` identity).
6. **AST-cap + CSE dedup**: described just above.

Post-construction sharing has two additional steps outside
`intern_gate`: at effective level `e-graph`, finalisation may merge
small-support combinational cones proven equivalent over the same
canonical leaf endpoints; after every flop's `d` exists, finalisation
may also merge flops by emitted-state meaning under
`identity_mode = node-id` with effective level `>= cse`.

Each layer except the bypass can short-circuit the call to a
pre-existing or synthesised node, in which case `is_new` is
`false` or reflects `intern_constant`'s result for synthesised
constants.

The full layer-by-layer narrative with per-rule tables and
orphan-safety reasoning lives in
[The Factorization Pipeline](factorization.md). Rule 21c in
[Structural Rules](structural-rules.md#21c--identity-mode--factorization-level-user-controllable-dial)
is the formal rule catalogue.

### Orphan safety: the compaction pass

Layers 1, 3, and 4 can leave gates unreferenced. Rule 18
(zero orphan gates) is restored at module finalisation by
`crate::ir::compact::compact_node_ids` — a BFS-based pass that
rewrites `m.nodes` (and every `NodeId` holder in `m.drives` /
`m.flops` / the dedup tables) down to the reachable set.
Without this pass, layers 1/3/4 would have to suppress any
rewrite that orphans an intermediate gate; with it, they fire
freely.

### Snapshot contract

`build_cone_with_retry` rewinds `nodes`, `flops`, pool, and
worklist on empty-dep-root retries. It **must** also restore
`gate_instances` and `const_instances` — otherwise stale keys
point at now-truncated `NodeId`s, and a subsequent intern call
returns a different node than the key promises. This is a
load-bearing invariant.

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

```rust,ignore
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
5. Every drive-root `NodeId` exists in `Module.nodes`.
6. Each output port appears in `drives` exactly once.
7. Each `Flop::d` is filled before emission (worklist drained).
8. Each `Flop::q` points at a `Node::FlopQ` whose `flop` backref and
   `width` match the owning flop.
9. Every `Node::FlopQ` is the canonical `q` node of a real flop.
10. Each flop's `mux` matches its assembled D shape and only
    references live `NodeId`s.

In `validate.rs`:

- All of the above.
- `m.flops[idx].id == idx` for every dense flop-table slot.
- Per-gate **arity**: each `GateOp` variant has a fixed or
  variadic-with-min operand count.
- Per-gate **output width**: `Eq`-family gates produce 1-bit;
  `Slice` produces `hi - lo + 1`; `Concat` produces the sum of
  operand widths; others match output-width from operand widths.
- Per-gate **operand widths**: `Mux` sel is 1-bit, arms match output
  width; comparisons require equal operand widths; `Slice` source
  width must exceed `hi`; bitwise/arithmetic operands equal output
  width; etc.
- Dense instance ids inside each module.
- Every instance input binding points at a live node.
- Every `Node::InstanceOutput` points at a real local instance.
- At the design level: unique module names, real top module, complete
  child input bindings, complete child output exposure, width matches
  across bindings/exposures, and an acyclic module graph.

Violation of any of these is a generator bug. The constructors do
not panic; the validator rejects with a rich error variant (port,
flop field, node id, op, operand index, expected vs got widths).

## Emitter contract

The emitter now has two layers:

- `emit::to_sv(&Module)` for leaf-only modules, and
- `emit::to_sv_in_design(&Module, &Design)` /
  `emit::to_sv_design(&Design)` for hierarchy-aware emission.

It assumes all invariants hold. It does not validate. It does not
reject anything. If the IR/design is valid, the emitted SV is valid.

Name generation is deterministic and **typed per gate kind** (Rule 12):

- `clk`, `rst_n` — clock and async-reset input ports (emitted only
  when the module has local flops).
- `i_0`, `i_1`, … — primary data inputs.
- `o_0`, `o_1`, … — primary outputs.
- `<gate_kind>_<N>` — internal gate wire. `<gate_kind>` is the
  lowercase `GateOp` name (`and`, `or`, `xor`, `not`, `add`,
  `sub`, `mul`, `eq`, `neq`, `lt`, `gt`, `le`, `ge`, `mux`,
  `slice`, `concat`, `red_and`, `red_or`, `red_xor`, `shl`,
  `shr`), and `<N>` counts per kind from 0 within the module.
- `flop_<id>` — flop register, indexed by `FlopId`.
- `instout_<instance>_<port>` — named wire for a child instance output
  inside a parent module.

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
- **Prerequisite now satisfied in useful form:** Phase 4 hierarchy is
  live enough that parameters have a real place to attach
  (instantiation), across both the legacy depth-1 wrapper lane and the
  bounded recursive hierarchy lane. It is still not the full parameter
  story: parameter-aware child selection and parameter-dependent parent
  generation remain future work.

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

  **Delivered (Phase 5b, 2026-05-18).** This is implemented as the
  opt-in `aggregate_prob` knob (default `0.0` → byte-identical).
  When it fires, a non-instantiated module's contiguous
  same-direction *data* ports are folded into one packed-`struct`
  port via a post-construction `Module.aggregate_layout`
  annotation that the emitter renders as a `typedef struct packed`
  + a single aggregate port + boundary alias wires/assigns — the
  flat IR body, validators, CSE and the dedup signature are all
  untouched (a module and its projected twin dedup-collapse). The
  scaffold is scoped to `struct packed`, to non-instantiated
  modules, and skips Phase 5 parameterized modules; `union`/`array`
  packing, parent-side aggregate connections and the
  param/aggregate cross-product are recorded follow-on sub-slices.
  Closed against the `Phase4Hierarchy` matrix gate
  (`phase5b_packed_aggregate` scenario, downstream-clean — Verilator
  + both Yosys). See `book/src/knobs.md` `aggregate_prob`,
  `docs/tasks/PHASE-5B-AGGREGATES.md`, and the `ROADMAP.md` Phase 5b
  exit criteria.
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
