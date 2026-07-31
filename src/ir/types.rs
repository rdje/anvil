//! Core IR types. Every constructor is responsible for preserving its
//! structural invariants (width consistency, dep-set correctness,
//! operand count). The validator in `validate.rs` is a development-time
//! safety net, not a production gate.
//!
//! Vocabulary: "arity" is used only for operators (associative primitives
//! like `And`, `Add`). Blocks (`Mux`, `Flop`) have "ports" or "arms", not
//! arity. See `book/src/structural-rules.md` "Operators vs blocks".

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// `COVERAGE-STEERED-GENERATION.3b` (decision `0034`): the roll counters live
/// beside the **single** knob-roll primitive in [`crate::ir::knob_roll`], not
/// here, so that `KnobRollCounters::record` can be private to that module and
/// a second (unsteered) roll primitive becomes a compile error. Re-exported
/// so `Module::knob_rolls` and every `crate::ir::KnobRollCounters` path are
/// unchanged.
pub use crate::ir::knob_roll::KnobRollCounters;

pub type PortId = u32;
pub type NodeId = u32;
pub type FlopId = u32;
pub type MemId = u32;
/// Phase 6 (`PHASE-6-ADVANCED-MOTIFS.3`): identity of a generated-encoding
/// FSM block, sibling to [`MemId`]/[`FlopId`].
pub type FsmId = u32;
pub type InstanceId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub width: u32,
    pub dir: Direction,
}

/// A width expression for Phase 5 parameterization.
///
/// Deliberately minimal: the only forms the first parameterization
/// slice needs are a concrete literal width and a reference to the
/// owning module's single `ParamEnv` parameter. The richer
/// `Add`/`Mul`/`Clog2`/… algebra described in `book/src/ir.md` is the
/// recorded strict follow-on (architecture (B) in
/// `DEVELOPMENT_NOTES.md` "Phase 5 parameterization design"); this
/// `{ Lit, Param }` enum is its seed, not a different design.
///
/// Internally every emitted module body stays concrete `u32` (the
/// monomorphic design value); `WidthExpr` only annotates *which*
/// interface widths the emitter renders symbolically and feeds the
/// parameter-aware identity rule. The IR's load-bearing `width: u32`
/// fields are intentionally untouched.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidthExpr {
    /// A concrete, non-parameterized width.
    Lit(u32),
    /// The owning module's parameter (`ParamEnv::name`).
    Param,
}

/// Per-module parameter environment (Phase 5, single width parameter).
///
/// `design_value` is the concrete width the module body was actually
/// constructed at. The emitted `parameter` declaration defaults to
/// `design_value`, so a module instantiated *without* an override
/// elaborates byte-identically to the pre-parameterization concrete
/// module — parameterization is valid by construction. `[min, max]`
/// is the inclusive range an instantiation may legally override the
/// parameter to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamEnv {
    pub name: String,
    pub min: u32,
    pub max: u32,
    pub design_value: u32,
}

/// Packed-aggregate surface kind for the Phase 5b emitter projection.
///
/// `StructPacked` is the general, always-sound case for a group of
/// differing-width ports (a packed `struct` is LRM-defined to be
/// bit-equivalent to the concatenation of its members). `ArrayPacked`
/// is the uniform-width case, rendered as a packed array
/// (`typedef logic [N-1:0][W-1:0] <name>;`) that is likewise
/// bit-equivalent to the field concatenation — a faithful projection
/// owned by `AGGREGATE-ARRAY-PACKING` (opt-in `aggregate_array_prob`).
/// `UnionPacked` stays deferred: a union aliases distinct ports, so it
/// is not a faithful projection of a group of distinct data ports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateKind {
    /// `typedef struct packed { … } <name>;` — differing-width groups.
    StructPacked,
    /// `typedef logic [N-1:0][W-1:0] <name>;` — uniform-width groups,
    /// bit-equivalent to the concatenation of the N same-width fields
    /// (`AGGREGATE-ARRAY-PACKING`).
    ArrayPacked,
}

/// Phase 5b packed-aggregate emitter projection (architecture (P),
/// `DEVELOPMENT_NOTES.md` "Phase 5b packed-aggregate emitter projection
/// design"). A purely **additive emitter-surface** annotation: a
/// contiguous, same-direction group of data ports is rendered as one
/// packed-aggregate port plus boundary alias wires, leaving the flat
/// IR body byte-identical. Never hashed into
/// `canonical_module_signature` (aggregates change nothing semantic).
/// `None` (the `Default`) is byte-identical to pre-Phase-5b behaviour.
/// Set by the post-construction `crate::ir::aggregate` pass under the
/// opt-in `Config::aggregate_prob` knob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateGroup {
    /// Emitted SV type name, e.g. `m_in_t`.
    pub type_name: String,
    /// Emitted aggregate port name, e.g. `m_in`.
    pub port_name: String,
    /// Ordered `(field_name, PortId)` — field order is the emitted
    /// port order; the bit layout is internal to the projection.
    pub fields: Vec<(String, PortId)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateLayout {
    pub kind: AggregateKind,
    /// Input-side group, if the module's data inputs were projected.
    pub inputs: Option<AggregateGroup>,
    /// Output-side group, if the module's outputs were projected.
    pub outputs: Option<AggregateGroup>,
}

/// Exact data-interface shape requested for a generated module.
///
/// This profile intentionally excludes `clk` / `rst_n`: control ports
/// are structural consequences of sequential state and propagate
/// through instantiated ancestors by rule, while data ports are the
/// part of the interface that parent-side hierarchy planning may shape
/// explicitly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleInterfaceProfile {
    pub data_input_widths: Vec<u32>,
    pub output_widths: Vec<u32>,
}

/// One clock domain in a multi-clock module
/// (`MULTI-CLOCK-CDC.2`). Each domain carries its own clk port,
/// async-active-low reset port, and a human-readable name (e.g.,
/// `"default"`, `"clk_a"`, `"clk_b"`). The single-clock K=1
/// case is the implicit default — `Module.clock_domains` empty +
/// `Module.clock`/`reset` populated yields byte-identical
/// emission to pre-`.2` ANVIL (the
/// `Module::effective_clock_domains` accessor synthesises a
/// single `ClockDomain { clk, rst_n, name: "default" }` when
/// needed).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClockDomain {
    pub clk: PortId,
    pub rst_n: PortId,
    pub name: String,
}

/// A circuit module: ports, internal nodes, flops, and a drive map
/// from each output port to the node that drives it.
#[derive(Debug, Clone, Default)]
pub struct Module {
    pub name: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock: Option<PortId>,
    pub reset: Option<PortId>,
    /// `MULTI-CLOCK-CDC.2` — declared clock domains. Empty (the
    /// default) means K=1 single-clock — the existing
    /// `Module.clock`/`reset` slots are authoritative and the
    /// emit is byte-identical to pre-`.2` ANVIL. Non-empty means
    /// K≥1 multi-clock — every flop has a `flop_domains` entry
    /// indexing into this vector, and the emitter generates one
    /// `always_ff` block per `(domain, polarity)` tuple.
    /// `Module::effective_clock_domains` returns this vec when
    /// non-empty, else synthesises a single-element default from
    /// `Module.clock`/`reset` for backward-compatible callers.
    /// Per `MULTI-CLOCK-CDC.1`'s design (`DEVELOPMENT_NOTES.md`).
    pub clock_domains: Vec<ClockDomain>,
    /// `MULTI-CLOCK-CDC.2` — per-flop domain tag. Empty (the
    /// default) means every flop belongs to domain 0 — the K=1
    /// special case stays byte-identical. Populated when the
    /// generator picks a non-zero domain for any flop. Lookups
    /// go through `Module::flop_domain(id)` which defaults to 0
    /// when the flop is absent from the map.
    pub flop_domains: BTreeMap<FlopId, u32>,
    /// Parent-planned exact data-interface shape, when this module was
    /// synthesized to satisfy a demanded hierarchy boundary rather than
    /// choosing its own input/output counts and widths locally.
    pub planned_interface_profile: Option<ModuleInterfaceProfile>,
    pub nodes: Vec<Node>,
    pub flops: Vec<Flop>,
    /// Phase 6 inferrable-memory blocks (`PHASE-6-ADVANCED-MOTIFS.2`).
    /// Additive, `Default`-empty: a module with no `Memory` is
    /// byte-identical to pre-Phase-6 emission. A `Memory` is a
    /// first-class clocked block (sibling to `Flop`); its read result
    /// enters the gate graph only via the opaque `Node::MemRead`
    /// leaf, so the array never participates in CSE/factorization.
    pub memories: Vec<Memory>,
    /// Phase 6 generated-encoding FSM blocks
    /// (`PHASE-6-ADVANCED-MOTIFS.3`). Additive, `Default`-empty: a
    /// module with no `Fsm` is byte-identical to pre-`.3` emission.
    /// An `Fsm` is a first-class clocked block (sibling to `Flop` /
    /// `Memory`); its registered Moore output enters the gate graph
    /// only via the opaque `Node::FsmOut` leaf, so the state machine
    /// never participates in CSE/factorization.
    pub fsms: Vec<Fsm>,
    pub instances: Vec<Instance>,
    /// (output_port_id, driving_node_id)
    pub drives: Vec<(PortId, NodeId)>,
    /// Construction-time AST-instance table: `(op, operands, width) →
    /// Vec<NodeId>` tracks how many times this gate expression has
    /// been created. Cap is `max_ast_instances`. See `Config` knob.
    pub(crate) gate_instances: HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>>,
    /// Construction-time AST-instance table for constants.
    pub(crate) const_instances: HashMap<(u32, u128), Vec<NodeId>>,
    /// Maximum number of times a given AST (gate or constant) may be
    /// named (have its own `NodeId`). Default 1 = strict uniqueness
    /// (CSE). Larger values permit N copies of the same expression;
    /// `u32::MAX` effectively disables deduplication.
    pub max_ast_instances: u32,
    /// Rate at which N-to-1 mux arms may share the same data. See
    /// `Config::mux_arm_duplication_rate`. Default 0.0 = all arms
    /// distinct; 1.0 = no constraint.
    pub mux_arm_duplication_rate: f64,
    /// Rate at which operator-gate operand lists may contain
    /// duplicates. See `Config::operand_duplication_rate`. Default
    /// 0.0 = strict operand uniqueness for Add/Mul (And/Or/Xor are
    /// always strict); 1.0 = no constraint.
    pub operand_duplication_rate: f64,
    /// Identity mode — the coarse answer to "what does a `NodeId`
    /// mean?" See `Config::identity_mode`. `NodeId` selects the
    /// full-factorization doctrine (`NodeId` = expression identity);
    /// `Relaxed` disables that doctrine and forces fresh node
    /// allocation for every AST.
    pub identity_mode: crate::config::IdentityMode,
    /// Requested factorization level — which parts of the current
    /// implementation ladder are active when `identity_mode ==
    /// NodeId`. See
    /// `Config::factorization_level` and the
    /// `FactorizationLevel` enum (`src/config.rs`) for the
    /// ladder: `none → cse → operand-unique → commutative →
    /// associative → constant-fold → peephole → e-graph`.
    /// Default `EGraph` (theoretical ceiling); `effective()`
    /// clamps down to the highest currently-implemented layer
    /// (today the bounded `EGraph` fragment). When
    /// `identity_mode == Relaxed`, the effective level is forced
    /// to `None`.
    pub factorization_level: crate::config::FactorizationLevel,
    /// Opt-in bounded bisimulation flop merge (`IDENTITY-DEEPENING`,
    /// decision `0007`). When `true` — and only under
    /// `identity_mode = node-id` with effective `factorization_level`
    /// `e-graph` — the post-drain finalization runs
    /// `crate::ir::compact::merge_bisimilar_flops`, a greatest-fixpoint
    /// partition refinement that merges flops proven sequentially
    /// equivalent up to a state correspondence (e.g. mutually-recursive
    /// registers) beyond the exact reset-defined self-hold class.
    /// `default = false` keeps emitted RTL byte-identical; mirrors
    /// `Config::bisimulation_flop_merge`. See `Config` for the knob.
    pub bisimulation_flop_merge: bool,

    // --- Block-build live counters ------------------------------
    /// Number of priority-encoder block instances successfully
    /// built in this module (via `build_priority_encoder_*`).
    /// Exposed via `Metrics::num_priority_encoder_blocks`.
    pub priority_encoder_built: u32,
    /// Number of one-hot-style combinational mux blocks built
    /// (comb-mux assembly path only; flop-mux one-hot is tracked
    /// separately under `flops_mux_one_hot`).
    pub comb_mux_one_hot_built: u32,
    /// Number of encoded-style combinational mux blocks built
    /// (chained-ternary form).
    pub comb_mux_encoded_built: u32,
    /// Number of procedural combinational `case` mux blocks built.
    pub case_mux_built: u32,
    /// Number of procedural combinational `casez` mux blocks built.
    pub casez_mux_built: u32,
    /// Number of procedural combinational statically bounded for-fold
    /// blocks built.
    pub for_fold_built: u32,

    /// Number of times the `ConstantFold` layer fired during
    /// construction of this module. Each fire is one algebraic
    /// identity applied in `intern_gate` — operands dropped, an
    /// absorbing constant substituted, or a single surviving
    /// operand short-circuited. Exposed via
    /// `Metrics::fold_identities_applied` for empirical
    /// measurement of the `ConstantFold` factorization layer.
    pub fold_identities_applied: u64,

    /// Number of times the `Peephole` layer fired during
    /// construction of this module. Each fire is one rule hit in
    /// [`Module::apply_peephole`]:
    ///
    /// - `Not(Not(x)) → x` (involutive collapse)
    /// - `Not(Eq/Neq/Lt/Gt/Le/Ge) → inverted cmp` (cross-gate
    ///   comparison inversion)
    /// - `Not(const) → ~const & mask` (constant evaluation)
    /// - `Eq/Neq/Lt/Gt/Le/Ge(c1, c2) → 1-bit const` (constant
    ///   evaluation for comparisons)
    /// - `Slice(hi, 0)(src)` full-width identity → `src`
    /// - `Slice(hi, lo)(c)` constant evaluation
    /// - `Concat([x]) → x` single-operand identity
    /// - `Concat([c1, c2, ...]) → assembled const` (MSB-first
    ///   bit assembly when every operand is a constant)
    /// - `RedAnd/RedOr/RedXor(c) → 1-bit const` (constant
    ///   evaluation for reductions)
    ///
    /// See `book/src/structural-rules.md` Rule 21c for the full
    /// rule catalogue. Surfaced via
    /// `Metrics::peephole_rewrites_applied`.
    pub peephole_rewrites_applied: u64,

    /// Number of nodes removed by the post-construction
    /// `compact_node_ids` pass. Zero when the IR is Rule-18-clean
    /// by construction (the default — every rewrite inside
    /// `intern_gate` is currently orphan-safe). Becomes non-zero
    /// when a rewrite like `Not(Not(x)) → x` leaves the inner
    /// `Not` reachable only via a now-collapsed outer call.
    /// Surfaced via `Metrics::nodes_compacted`.
    pub nodes_compacted: u32,

    /// Number of duplicate flops merged away during the
    /// post-drain endpoint-preserving state-sharing pass. Once
    /// D-cones exist, flops with identical emitted state semantics
    /// over the same canonical leaf variables collapse to one
    /// state element when the effective factorization level is at
    /// least `Cse`. Today the proof is conservative: it follows the
    /// current normalized IR rather than a full sequential
    /// equivalence engine.
    /// Surfaced via `Metrics::flops_merged`.
    pub flops_merged: u32,

    /// Number of duplicate flops merged away by the opt-in bounded
    /// bisimulation flop-merge pass
    /// (`crate::ir::compact::merge_bisimilar_flops`,
    /// `IDENTITY-DEEPENING`). Non-zero only when
    /// `bisimulation_flop_merge` is enabled under node-id / e-graph and a
    /// greatest-fixpoint state correspondence (beyond exact reset-defined
    /// self-hold) proved two flops sequentially equivalent. Zero by
    /// default. Surfaced via `Metrics::bisimulation_flops_merged`.
    pub bisimulation_flops_merged: u32,

    /// Number of duplicate deterministic FSM blocks merged away during
    /// the post-construction endpoint-preserving state-sharing pass.
    /// FSMs reset to state 0 and have explicit transition/output
    /// tables, so under `identity_mode = node-id` two FSMs with the
    /// same selector proof and same table/encoding/output signature
    /// can safely collapse to one state block. Memories intentionally
    /// remain opaque because their stored contents are not reset-defined.
    /// Surfaced via `Metrics::fsms_merged`.
    pub fsms_merged: u32,

    /// Number of duplicate combinational gates merged away during
    /// the post-construction bounded semantic-sharing pass.
    /// Under the live `EGraph` fragment, small-support cones that
    /// are proven functionally equal over the same canonical leaf
    /// variables collapse to one gate. Surfaced via
    /// `Metrics::semantic_gates_merged`.
    pub semantic_gates_merged: u32,

    /// Number of times the `Associative` factorization layer fired
    /// during construction of this module. Each fire is one
    /// invocation of `intern_gate` on an associative op
    /// (`And`/`Or`/`Xor`/`Add`/`Mul`) whose operand list contained
    /// at least one same-op same-width inner gate, which was
    /// spliced into the outer operand list (possibly followed by
    /// semantic dedup/cancel per the op class). Surfaced via
    /// `Metrics::flatten_associative_applied`.
    pub flatten_associative_applied: u64,

    /// Per-knob attempt/fire counters for every probability roll
    /// taken during construction. Populated live by the
    /// `roll_knob` helper in `src/gen/cone.rs` at every
    /// `gen_bool(cfg.<prob>)` site. Surfaced via
    /// `Metrics::knob_roll_attempts` / `knob_roll_fires` so each
    /// probability knob's effect is empirically measurable: the
    /// empirical fire-rate `fires / attempts` should converge to
    /// the knob value across large seed sweeps.
    pub knob_rolls: KnobRollCounters,

    // --- Phase 5 parameterization (default-off, additive) -------
    /// When `Some`, this module carries a single width `parameter`
    /// (Phase 5). The module *body* is still concrete `u32` at
    /// `ParamEnv::design_value`; this only changes how the emitter
    /// renders the header and the marked interface ports, and how
    /// `canonical_module_signature` hashes those ports. `None`
    /// (the `Default`) is byte-identical to pre-Phase-5 behaviour.
    /// Set by the post-construction `crate::ir::param` pass under the
    /// opt-in `Config::width_parameterization_prob` knob.
    pub param_env: Option<ParamEnv>,
    /// Input port ids whose width the emitter renders symbolically as
    /// `[<param>-1:0]` and whose width
    /// `canonical_module_signature` hashes as the parameter's
    /// normalized symbolic form. Empty unless `param_env.is_some()`.
    pub parameterized_input_ports: Vec<PortId>,
    /// Output port ids parameterized the same way as
    /// `parameterized_input_ports`.
    pub parameterized_output_ports: Vec<PortId>,
    /// Phase 5b packed-aggregate emitter projection. `None` (the
    /// `Default`) ⇒ byte-identical to pre-Phase-5b emission. Set by
    /// the post-construction `crate::ir::aggregate` pass under the
    /// opt-in `Config::aggregate_prob` knob. Purely an emitter-surface
    /// regrouping; the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected.
    pub aggregate_layout: Option<AggregateLayout>,
    /// `SV-VERSION-TARGETING.3b.2` — the set of `Slice` gate `NodeId`s the
    /// emitter should render via an internal IEEE-1800-2023 `union soft`
    /// overlay (`u.w = src; <gate> = u.n`) instead of a plain `src[hi:0]`
    /// bit-select, **iff** the emission target also permits 2023
    /// (`SvVersion::permits(Sv2023)`); below 2023 it down-gates to the plain
    /// slice. Populated by the post-construction `crate::ir::soft_union`
    /// pass under the opt-in `Config::soft_union_slice_prob` knob. Empty
    /// (the `Default`) ⇒ byte-identical to pre-`.3b` emission. The overlay
    /// is behaviour-preserving (packed-union members are LSB-aligned, so
    /// `u.n == src[hi:0]`), so the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected — like
    /// `aggregate_layout`, this is an emitter-surface annotation only and
    /// is deliberately not hashed into identity.
    pub soft_union_slice_gates: BTreeSet<NodeId>,
    /// `STRUCTURED-EMISSION-EXPANSION.2b.1` — the set of combinational
    /// `Node::Gate` `NodeId`s the emitter should render as a
    /// behaviour-preserving combinational `function automatic` projection
    /// (a `<wire>__f` function over the gate's direct operands plus a call
    /// site) instead of an inline `assign <wire> = <op>;` (decision `0012`).
    /// Populated by the post-construction `crate::ir::function_emit` pass
    /// under the opt-in `Config::function_emit_prob` knob. Empty (the
    /// `Default`) ⇒ byte-identical emission. The function returns exactly
    /// the gate's value, so the projection is behaviour-preserving by
    /// construction; like `soft_union_slice_gates` / `aggregate_layout`
    /// this is an emitter-surface annotation only — the flat IR body,
    /// validators, CSE keys and `canonical_module_signature` are all
    /// unaffected and it is deliberately not hashed into identity.
    /// Disjoint from `soft_union_slice_gates` by construction (a
    /// `union soft` slice is never a function-emit candidate).
    pub function_emit_gates: BTreeSet<NodeId>,
    /// `STRUCTURED-EMISSION-EXPANSION.4b` — the set of replication
    /// `Node::Gate` `NodeId`s the emitter should render as a
    /// behaviour-preserving single-level `generate for` loop (`genvar
    /// <wire>__gi; generate for (...) assign <wire>[gi] = <x>;
    /// endgenerate`) instead of an inline `assign <wire> = {N{x}};`
    /// (decision `0013`). Each marked gate is a `GateOp::Concat` of the
    /// `{N{x}}` form (`>= 2` operands, all the same `NodeId`) with a 1-bit
    /// lane, so the unrolled loop is byte-equivalent to the inline
    /// replication. Populated by the post-construction
    /// `crate::ir::generate_loop` pass under the opt-in
    /// `Config::generate_loop_emit_prob` knob. Empty (the `Default`) ⇒
    /// byte-identical emission; like `soft_union_slice_gates` /
    /// `function_emit_gates` / `aggregate_layout` this is an
    /// emitter-surface annotation only — the flat IR body, validators, CSE
    /// keys and `canonical_module_signature` are all unaffected and it is
    /// deliberately not hashed into identity. Disjoint from
    /// `function_emit_gates` by construction (the generate-loop pass runs
    /// after function-emit and excludes already-marked gates).
    pub generate_loop_gates: BTreeSet<NodeId>,
    /// `STRUCTURED-EMISSION-EXPANSION.6b.1` — the set of combinational
    /// `Node::Gate` `NodeId`s the emitter should render as a
    /// behaviour-preserving combinational `task automatic` projection (a
    /// `<wire>__t` procedural task over the gate's direct operands, called
    /// from an `always_comb` into a `<wire>__tv` output var, with the gate's
    /// net driven `assign <wire> = <wire>__tv;`) instead of an inline
    /// `assign <wire> = <op>;` (decision `0014`). The decision `0012`
    /// single-gate parallel, but a procedural `task` rather than a
    /// value-returning `function`. Populated by the post-construction
    /// `crate::ir::task_emit` pass under the opt-in `Config::task_emit_prob`
    /// knob. Empty (the `Default`) ⇒ byte-identical emission. The task writes
    /// exactly the gate's value, so the projection is behaviour-preserving by
    /// construction; like `soft_union_slice_gates` / `function_emit_gates` /
    /// `generate_loop_gates` / `aggregate_layout` this is an emitter-surface
    /// annotation only — the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected and it is deliberately
    /// not hashed into identity. Disjoint from `function_emit_gates` /
    /// `generate_loop_gates` / `soft_union_slice_gates` by construction (the
    /// task pass runs after the others and excludes already-marked gates).
    pub task_emit_gates: BTreeSet<NodeId>,

    /// `STRUCTURED-EMISSION-EXPANSION.10b` — map from a **cone root**
    /// `Node::Gate` `NodeId` to the topo-ordered list of **absorbed interior
    /// gate** `NodeId`s the emitter should fold into a single behaviour-
    /// preserving multi-gate-cone `function automatic` (`<root>__cf`) instead
    /// of the inline per-gate `assign` chain (decision `0016`). The function's
    /// parameters are the cone's boundary leaves, its body is one function-local
    /// `logic` per interior gate in dependency order (constants folded inline),
    /// and it returns the root; the root's `assign` becomes a call. Each
    /// absorbed interior gate is used exactly once in the module (so suppressing
    /// its module-level `wire` declaration **and** its inline `assign` is safe),
    /// and the root has `>= 1` absorbed interior gate (so the body is genuinely
    /// multi-statement — a zero-interior cone is left to `function_emit`).
    /// Populated by the post-construction `crate::ir::cone_function_emit` pass
    /// under the opt-in `Config::cone_function_emit_prob` knob. Empty (the
    /// `Default`) ⇒ byte-identical emission. The deepening of the decision
    /// `0012` single-gate `function_emit_gates` from one gate to a whole cone;
    /// like the sibling emit-projection markers this is an emitter-surface
    /// annotation only — the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected and it is deliberately
    /// not hashed into identity. The roots and absorbed interiors are disjoint
    /// from `function_emit_gates` / `generate_loop_gates` / `task_emit_gates` /
    /// `soft_union_slice_gates` by construction (the cone pass runs last and
    /// excludes already-marked gates as both roots and interiors).
    pub cone_function_gates: BTreeMap<NodeId, Vec<NodeId>>,

    /// `STRUCTURED-EMISSION-EXPANSION.12b` — gates grouped for the multi-output
    /// combinational `task automatic` emit-projection (decision `0025`). Keyed by
    /// the group **leader** (lowest-`NodeId` member) → the **partner** members
    /// (the first cut is a pair ⇒ a single-element `[partner]`); the full group is
    /// `key` plus the partners. The emitter renders one `task automatic` per group
    /// with one `output` per member and a deduplicated `input` list over the
    /// members' shared non-constant operands, called once from `always_comb`; each
    /// member's net is driven by a passthrough `assign`. Like the sibling
    /// `*_gates` fields this is an **emitter-surface annotation only** — the flat
    /// IR body, validators, CSE keys and `canonical_module_signature` are
    /// untouched, and it is not hashed into identity. Disjoint from
    /// `function_emit_gates` / `generate_loop_gates` / `task_emit_gates` /
    /// `soft_union_slice_gates` / `cone_function_gates` by construction (the pass
    /// runs after the first four and excludes their marks; the cone pass runs after
    /// this one and excludes these members). Populated by the post-construction
    /// `crate::ir::multi_output_task_emit` pass under the opt-in
    /// `Config::multi_output_task_emit_prob` knob. Empty (the `Default`) ⇒
    /// byte-identical emission.
    pub multi_output_task_groups: BTreeMap<NodeId, Vec<NodeId>>,

    /// `STRUCTURED-EMISSION-EXPANSION.15b` — the set of `Node::Gate` `NodeId`s
    /// (each a 2:1 `GateOp::Mux`) the emitter should render as a
    /// behaviour-preserving procedural `always_comb` `if`/`else` projection (a
    /// `<wire>__cv` output var written `if (<sel>) <wire>__cv = <a>; else
    /// <wire>__cv = <b>;` in an `always_comb`, with the gate's net driven
    /// `assign <wire> = <wire>__cv;`) instead of the inline ternary
    /// `assign <wire> = (<sel>) ? (<a>) : (<b>);` (decision `0027`). The first
    /// procedural-conditional construct in the lane. The `if`/`else` writes
    /// exactly the mux's value (`sel == 1 ⇒ a`, `sel == 0 ⇒ b`), so the
    /// projection is behaviour-preserving by construction; like the sibling
    /// `*_gates` fields this is an **emitter-surface annotation only** — the
    /// flat IR body, validators, CSE keys and `canonical_module_signature` are
    /// all unaffected, and it is not hashed into identity. Populated by the
    /// post-construction `crate::ir::mux_if_emit` pass under the opt-in
    /// `Config::mux_if_emit_prob` knob; the pass runs **last** and excludes any
    /// gate already marked by a sibling projection, so this set is disjoint from
    /// all of them by construction. Empty (the `Default`) ⇒ byte-identical
    /// emission.
    pub mux_if_gates: BTreeSet<NodeId>,

    /// `STRUCTURED-EMISSION-EXPANSION.17b` — the eighth structured surface
    /// (decision `0028`): dynamic-selector `CaseMux` gates the emitter renders
    /// as a procedural `always_comb` `if`/`else if` **priority chain** instead
    /// of the parallel `case` statement (the N-way generalization of the
    /// seventh surface's 2:1 `Mux` → `if`/`else`). Like the sibling `*_gates`
    /// fields this is an **emitter-surface annotation only** — the flat IR body,
    /// validators, CSE keys and `canonical_module_signature` are all unaffected.
    /// Populated by the post-construction `crate::ir::case_mux_if_emit` pass
    /// under the opt-in `Config::case_mux_if_emit_prob` knob; the pass runs
    /// **last** (after `mux_if`). A `CaseMux` is already an
    /// `always_comb`-written `logic` var, so this surface needs no
    /// output-var/passthrough — only the block body swaps. Empty (the
    /// `Default`) ⇒ byte-identical emission.
    pub case_mux_if_gates: BTreeSet<NodeId>,

    /// `STRUCTURED-EMISSION-EXPANSION.19b` (decision `0029`) — the set of
    /// **dynamic-selector** `CasezMux` gates marked for the procedural
    /// `always_comb` `if`/`else if` **masked** priority-chain emit-projection
    /// (the ninth richer-structured surface). A marked gate renders its body as
    /// a chain of masked equality tests `(sel & care_mask) == value_masked`
    /// instead of the parallel `casez` statement — the wildcard generalization
    /// of `case_mux_if_gates`'s bare-equality chain. An emitter-surface
    /// annotation only: the flat IR body, validators, CSE keys and
    /// `canonical_module_signature` are all unaffected. Populated by the
    /// post-construction `crate::ir::casez_mux_if_emit` pass under the opt-in
    /// `Config::casez_mux_if_emit_prob` knob; the pass runs **last** (after
    /// `case_mux_if`). A `CasezMux` is already an `always_comb`-written `logic`
    /// var, so this surface needs no output-var/passthrough — only the block
    /// body swaps. Empty (the `Default`) ⇒ byte-identical emission.
    pub casez_mux_if_gates: BTreeSet<NodeId>,
}

impl Module {
    /// Whether this module emits any sequential state locally.
    pub fn has_local_flops(&self) -> bool {
        !self.flops.is_empty()
    }

    /// `MULTI-CLOCK-CDC.2` — domain index of `flop_id`. Returns
    /// `0` when the flop is absent from `flop_domains` (the
    /// backward-compatible default — every existing K=1 module
    /// has all flops in domain 0). Domain indices are bounded by
    /// `effective_clock_domains().len()` at construction time;
    /// `Module::validate` enforces that bound (`MULTI-CLOCK-CDC.2`).
    pub fn flop_domain(&self, flop_id: FlopId) -> u32 {
        *self.flop_domains.get(&flop_id).unwrap_or(&0)
    }

    /// `MULTI-CLOCK-CDC.2` — the clock domains the emitter uses.
    /// When `clock_domains` is non-empty it is authoritative
    /// (K≥1 multi-clock path). When empty (the K=1 default),
    /// synthesises a single `ClockDomain { clk, rst_n, name:
    /// "default" }` from the existing `Module.clock`/`reset`
    /// slots so callers see a uniform interface; returns an
    /// empty `Vec` only when neither `clock_domains` is set
    /// nor `Module.clock` is — i.e., a purely combinational
    /// module that should never reach this accessor anyway
    /// (gated by `has_local_flops`/`has_local_memories`/
    /// `has_local_fsms`).
    pub fn effective_clock_domains(&self) -> Vec<ClockDomain> {
        if !self.clock_domains.is_empty() {
            return self.clock_domains.clone();
        }
        match (self.clock, self.reset) {
            (Some(clk), Some(rst_n)) => vec![ClockDomain {
                clk,
                rst_n,
                name: "default".to_string(),
            }],
            _ => Vec::new(),
        }
    }

    /// Phase 6: a `Memory` is local sequential state (clocked block),
    /// so a memory-bearing module must expose `clk` even with no
    /// flops. Kept separate from [`has_local_flops`] so the flop
    /// decl / `always_ff` emission gates are unaffected.
    pub fn has_local_memories(&self) -> bool {
        !self.memories.is_empty()
    }

    /// Phase 6 (`PHASE-6-ADVANCED-MOTIFS.3`): an `Fsm` is local
    /// sequential state (its encoded-state register is a clocked,
    /// async-reset flop), so an FSM-bearing module carries sequential
    /// state and must expose `clk`/`rst_n`. Kept separate from
    /// [`has_local_flops`]/[`has_local_memories`] so their decl /
    /// `always_ff` emission gates are unaffected.
    pub fn has_local_fsms(&self) -> bool {
        !self.fsms.is_empty()
    }

    /// Whether this module carries sequential state either locally or
    /// through instantiated descendants in the provided design view.
    /// Without design context, only local flops are visible.
    pub fn carries_sequential_state_in(&self, modules: Option<&BTreeMap<&str, &Module>>) -> bool {
        let Some(modules) = modules else {
            return self.has_local_flops() || self.has_local_memories() || self.has_local_fsms();
        };
        self.carries_sequential_state_with_visited(modules, &mut BTreeSet::new())
    }

    fn carries_sequential_state_with_visited(
        &self,
        modules: &BTreeMap<&str, &Module>,
        visiting: &mut BTreeSet<String>,
    ) -> bool {
        if self.has_local_flops() || self.has_local_memories() || self.has_local_fsms() {
            return true;
        }
        if !visiting.insert(self.name.clone()) {
            return false;
        }

        let carries = self.instances.iter().any(|instance| {
            modules
                .get(instance.module.as_str())
                .is_some_and(|child| child.carries_sequential_state_with_visited(modules, visiting))
        });

        visiting.remove(&self.name);
        carries
    }

    /// Whether an input port is visible in the emitted module
    /// interface. `clk` / `rst_n` are construction-time IR ports for
    /// leaf modules and hierarchy wrappers. A module emits them iff it
    /// carries sequential state itself or through instantiated
    /// descendants. Pure combinational modules stay free of control
    /// ports even if they were tagged conservatively in the IR. Once a
    /// module carries local state or sequential descendants, those
    /// control ports stay visible all the way up the instantiated
    /// ancestor chain.
    pub fn is_emitted_input_port_in(
        &self,
        port_id: PortId,
        modules: Option<&BTreeMap<&str, &Module>>,
    ) -> bool {
        if (self.clock == Some(port_id) || self.reset == Some(port_id))
            && !self.carries_sequential_state_in(modules)
        {
            return false;
        }
        self.inputs.iter().any(|port| port.id == port_id)
    }

    pub fn is_emitted_input_port(&self, port_id: PortId) -> bool {
        self.is_emitted_input_port_in(port_id, None)
    }

    /// Iterate the input ports that appear in the emitted SystemVerilog
    /// module header.
    pub fn emitted_input_ports_in<'a>(
        &'a self,
        modules: Option<&'a BTreeMap<&'a str, &'a Module>>,
    ) -> impl Iterator<Item = &'a Port> + 'a {
        self.inputs
            .iter()
            .filter(move |port| self.is_emitted_input_port_in(port.id, modules))
    }

    pub fn emitted_input_ports(&self) -> impl Iterator<Item = &Port> {
        self.emitted_input_ports_in(None)
    }

    /// Iterate the emitted data inputs only, excluding structural
    /// control ports.
    pub fn emitted_data_input_ports_in<'a>(
        &'a self,
        modules: Option<&'a BTreeMap<&'a str, &'a Module>>,
    ) -> impl Iterator<Item = &'a Port> + 'a {
        self.emitted_input_ports_in(modules)
            .filter(move |port| self.clock != Some(port.id) && self.reset != Some(port.id))
    }

    pub fn emitted_data_input_ports(&self) -> impl Iterator<Item = &Port> {
        self.emitted_data_input_ports_in(None)
    }

    pub fn input_port(&self, port_id: PortId) -> Option<&Port> {
        self.inputs.iter().find(|port| port.id == port_id)
    }

    pub fn output_port(&self, port_id: PortId) -> Option<&Port> {
        self.outputs.iter().find(|port| port.id == port_id)
    }

    /// Effective factorization level after applying the coarse
    /// identity mode.
    pub fn effective_factorization_level(&self) -> crate::config::FactorizationLevel {
        match self.identity_mode {
            crate::config::IdentityMode::Relaxed => crate::config::FactorizationLevel::None,
            crate::config::IdentityMode::NodeId => self.factorization_level.effective(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceRole {
    PlannedChild,
    ParentCone,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub module: String,
    pub role: InstanceRole,
    /// Child input port id -> parent driving node id.
    pub inputs: Vec<(PortId, NodeId)>,
    /// Phase 5 parameter overrides for this instance:
    /// `(parameter_name, resolved_value)`. Empty for every
    /// non-parameterized instance (default-off / pre-Phase-5), in
    /// which case emission is byte-identical to before. When
    /// non-empty, the emitter renders a `#(.NAME(value), …)` override
    /// list on the instantiation. Populated by
    /// `PHASE-5-PARAMETERIZATION.2.2.3b` when the instantiated child
    /// carries a `ParamEnv`.
    pub param_bindings: Vec<(String, u32)>,
}

/// Phase 6 inferrable-memory kind (`PHASE-6-ADVANCED-MOTIFS.2`). Only
/// the two empirically Yosys-`$mem_v2`-inferred shapes from the `.1`
/// probe; both clean in Verilator + both repo Yosys modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemKind {
    /// One shared synchronous read/write address (`mem[addr]`).
    SinglePort,
    /// One write port (`waddr`/`we`/`wdata`) + one independent
    /// synchronous read port (`raddr`).
    SimpleDualPort,
}

/// A first-class inferrable-memory block (`PHASE-6-ADVANCED-MOTIFS.2`).
/// Clocked by the module's shared `clk` (single-clock discipline);
/// emitted as the `.1`-validated synchronous-write / synchronous-read
/// template Yosys infers as `$mem_v2`. The write/read address and
/// write-data inputs are real generated cones (`NodeId`s, dependency
/// tracked + validated); the registered read output is exposed as the
/// opaque `Node::MemRead` leaf, never folded into the expression graph.
#[derive(Debug, Clone)]
pub struct Memory {
    pub id: MemId,
    pub addr_width: u32,
    pub data_width: u32,
    pub kind: MemKind,
    /// Write-enable source (width 1).
    pub we: NodeId,
    /// Write address source (width `addr_width`).
    pub waddr: NodeId,
    /// Write data source (width `data_width`).
    pub wdata: NodeId,
    /// Read address source (width `addr_width`). For `SinglePort`
    /// this is the same node as `waddr` (one shared address).
    pub raddr: NodeId,
}

/// Generated state encoding for an [`Fsm`] (`PHASE-6-ADVANCED-MOTIFS.3`).
/// The choice fixes the state-register width and the `localparam`
/// state-constant bit-patterns; all three are downstream-clean in
/// Verilator + both repo Yosys modes (`.3.1` empirical probe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsmEncoding {
    /// State register `ceil(log2 num_states)` bits wide; constants
    /// `0,1,2,…`.
    Binary,
    /// State register `num_states` bits wide; constant `i` is
    /// `1 << i` (exactly one hot bit).
    OneHot,
    /// State register `ceil(log2 num_states)` bits wide; constants
    /// are the reflected binary (Gray) code (successive values differ
    /// by one bit).
    Gray,
}

impl FsmEncoding {
    /// Width of the state register for this encoding and state count.
    pub fn state_width(self, num_states: u32) -> u32 {
        match self {
            FsmEncoding::Binary | FsmEncoding::Gray => {
                let n = num_states.max(2);
                (u32::BITS - (n - 1).leading_zeros()).max(1)
            }
            FsmEncoding::OneHot => num_states.max(1),
        }
    }

    /// The encoded constant for state index `s` (`s < num_states`).
    pub fn state_const(self, s: u32) -> u128 {
        match self {
            FsmEncoding::Binary => s as u128,
            FsmEncoding::OneHot => 1u128 << s,
            // Reflected binary: gray(s) = s ^ (s >> 1).
            FsmEncoding::Gray => (s ^ (s >> 1)) as u128,
        }
    }
}

/// A first-class generated-encoding Moore FSM block
/// (`PHASE-6-ADVANCED-MOTIFS.3`). Clocked by the module's shared `clk`
/// with the module's async-low `rst_n` (single-clock discipline);
/// emitted as the `.3.1`-probed-clean template (encoding-derived
/// `localparam` state constants + a `state_q` register + an
/// `always_comb` next-state `case` selected by `sel` + an
/// `always_comb` Moore output `case`). `sel` is a real generated cone
/// (a `NodeId`, dependency-tracked + validated); the registered Moore
/// output is exposed as the opaque [`Node::FsmOut`] leaf, never folded
/// into the expression graph.
#[derive(Debug, Clone)]
pub struct Fsm {
    pub id: FsmId,
    /// Number of states (`>= 1`); reset state is index `0`.
    pub num_states: u32,
    pub encoding: FsmEncoding,
    /// Transition-select source cone (width `sel_width`).
    pub sel: NodeId,
    pub sel_width: u32,
    /// `transitions[state][sel_value]` = next-state index
    /// (`< num_states`). Shape `[num_states][1 << sel_width]`.
    pub transitions: Vec<Vec<u32>>,
    /// `outputs[state]` = the Moore output value for that state
    /// (masked to `out_width`). Length `num_states`. For a Mealy FSM
    /// (`mealy_outputs.is_some()`) this is retained but the emitter uses
    /// `mealy_outputs` instead.
    pub outputs: Vec<u128>,
    /// Width of the registered Moore output (`Node::FsmOut.width`).
    pub out_width: u32,
    /// `CAPABILITY-BREADTH-EXPANSION.2b` (decision `0024`) — optional
    /// **Mealy** output table. `None` ⇒ **Moore** (the default;
    /// `outputs[state]` is the decode). `Some` ⇒ **Mealy**:
    /// `mealy_outputs[state][sel_value]` (shape
    /// `[num_states][1 << sel_width]`, each entry masked to `out_width`)
    /// is the combinational output decode over the **current state and
    /// current input** — the registered `state_q` plus the
    /// input-dependent `sel` cone, the textbook Mealy form. The state
    /// register stays Moore-clocked; only the output decode reads `sel`.
    /// Default-off ⇒ `None` ⇒ byte-identical.
    pub mealy_outputs: Option<Vec<Vec<u128>>>,
}

impl Fsm {
    /// True iff this FSM emits a **Mealy** output — a combinational decode
    /// over `(state_q, sel)` (`mealy_outputs.is_some()`); false ⇒ Moore.
    pub fn is_mealy(&self) -> bool {
        self.mealy_outputs.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    PrimaryInput {
        port: PortId,
        width: u32,
    },
    Constant {
        width: u32,
        value: u128,
    },
    FlopQ {
        flop: FlopId,
        width: u32,
    },
    /// Registered read output of a `Memory` block
    /// (`PHASE-6-ADVANCED-MOTIFS.2`). An **opaque leaf**, exactly like
    /// `FlopQ`: identity-by-instance (the `MemId`), never merged by
    /// CSE / never an expression — the clock edge breaks the
    /// combinational path. `width == Memory.data_width`.
    MemRead {
        mem: MemId,
        width: u32,
    },
    /// Registered Moore output of an `Fsm` block
    /// (`PHASE-6-ADVANCED-MOTIFS.3`). An **opaque leaf**, exactly like
    /// `FlopQ`/`MemRead`: identity-by-instance (the `FsmId`), never
    /// merged by CSE / never an expression — the clock edge breaks
    /// the combinational path. `width == Fsm.out_width`.
    FsmOut {
        fsm: FsmId,
        width: u32,
    },
    InstanceOutput {
        instance: InstanceId,
        port: PortId,
        width: u32,
    },
    Gate {
        op: GateOp,
        operands: Vec<NodeId>,
        width: u32,
        deps: DepSet,
    },
}

impl Node {
    pub fn width(&self) -> u32 {
        match self {
            Node::PrimaryInput { width, .. }
            | Node::Constant { width, .. }
            | Node::FlopQ { width, .. }
            | Node::MemRead { width, .. }
            | Node::FsmOut { width, .. }
            | Node::InstanceOutput { width, .. }
            | Node::Gate { width, .. } => *width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForFoldKind {
    Xor,
    Or,
    And,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateOp {
    // Bitwise
    And,
    Or,
    Xor,
    Not,
    // Arithmetic
    Add,
    Sub,
    Mul,
    // Comparison (output is 1-bit)
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    // Structured
    Mux,      // [sel, a, b] with sel.width == 1
    CaseMux,  // [sel, data_0, data_1, ...], emitted as always_comb case
    CasezMux, // [sel, value_0, wild_0, data_0, ...], emitted as always_comb casez
    ForFold {
        kind: ForFoldKind,
        trip_count: u32,
        chunk_width: u32,
    }, // [src], emitted as always_comb for-loop over packed chunks
    Slice {
        hi: u32,
        lo: u32,
    },
    Concat, // variadic
    // Reductions (output is 1-bit)
    RedAnd,
    RedOr,
    RedXor,
    // Shifts
    Shl,
    Shr,
}

impl GateOp {
    /// The output is 1-bit regardless of input width.
    pub fn is_reduction_like(&self) -> bool {
        matches!(
            self,
            GateOp::Eq
                | GateOp::Neq
                | GateOp::Lt
                | GateOp::Gt
                | GateOp::Le
                | GateOp::Ge
                | GateOp::RedAnd
                | GateOp::RedOr
                | GateOp::RedXor
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetKind {
    None,
    Sync,
    Async,
}

/// The two supported flop motifs. Both have a one-hot M-to-1 mux on D.
/// They differ in what D becomes when no select bit is asserted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlopKind {
    /// When all M selects are 0, D = 0 (the flop loads zero next cycle).
    ZeroDefault,
    /// When all M selects are 0, D = Q (the flop holds its current value).
    QFeedback,
}

/// One arm of a flop's one-hot input mux: a data sub-cone + a 1-bit select.
#[derive(Debug, Clone)]
pub struct MuxArm {
    pub data: NodeId,
    pub sel: NodeId,
}

/// How a flop's D input is constructed. Populated by
/// `drain_flop_worklist` alongside `Flop.d`.
#[derive(Debug, Clone)]
pub enum FlopMux {
    /// M = 0. D is a direct recursive cone (no mux structure).
    None,
    /// M = 2..=max. One select bit per arm; D = OR of masked arms.
    OneHot(Vec<MuxArm>),
    /// M = 2..=max. One select bus of width ceil(log2(M)) indexes one
    /// of M data inputs via a chained ternary. For `FlopKind::QFeedback`
    /// the slot at index 0 is routed from Q instead of a recursive cone,
    /// so `data.len() == M - 1` in that case (indices 1..M-1); for
    /// `FlopKind::ZeroDefault`, `data.len() == M`.
    Encoded { sel: NodeId, data: Vec<NodeId> },
}

#[derive(Debug, Clone)]
pub struct Flop {
    pub id: FlopId,
    pub width: u32,
    pub d: Option<NodeId>,
    pub q: NodeId,
    pub reset_val: u128,
    pub reset_kind: ResetKind,
    pub kind: FlopKind,
    pub mux: FlopMux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DepAtom {
    Port(PortId),
    FlopVirtual(FlopId),
    /// Virtual endpoint for a `Memory` block's registered read
    /// (`Node::MemRead`), keyed by `MemId` so distinct memories'
    /// reads are distinct leaf endpoints — like `FlopVirtual`.
    MemVirtual(MemId),
    /// Virtual endpoint for an `Fsm` block's registered Moore output
    /// (`Node::FsmOut`), keyed by `FsmId` so distinct FSMs' outputs
    /// are distinct leaf endpoints — like `FlopVirtual`/`MemVirtual`.
    FsmVirtual(FsmId),
    InstanceOutputVirtual {
        instance: InstanceId,
        port: PortId,
    },
}

/// Set of leaf variables that a node depends on.
///
/// These leaves are the endpoint variables of the current module:
/// primary-input ports, local flop-Q leaves, and instantiated child
/// output leaves. Empty dep-set on an output cone indicates the cone is
/// trivially constant and must be regenerated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DepSet {
    set: BTreeSet<DepAtom>,
}

impl DepSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_port(p: PortId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(DepAtom::Port(p));
        Self { set: s }
    }

    pub fn from_flop_virtual(flop: FlopId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(DepAtom::FlopVirtual(flop));
        Self { set: s }
    }

    /// Virtual dep for a `Memory` block's registered read
    /// (`PHASE-6-ADVANCED-MOTIFS.2`); mirrors [`from_flop_virtual`].
    pub fn from_mem_virtual(mem: MemId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(DepAtom::MemVirtual(mem));
        Self { set: s }
    }

    /// Virtual dep for an `Fsm` block's registered Moore output
    /// (`PHASE-6-ADVANCED-MOTIFS.3`); mirrors [`from_mem_virtual`].
    pub fn from_fsm_virtual(fsm: FsmId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(DepAtom::FsmVirtual(fsm));
        Self { set: s }
    }

    pub fn from_instance_output_virtual(instance: InstanceId, port: PortId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(DepAtom::InstanceOutputVirtual { instance, port });
        Self { set: s }
    }

    pub fn union(sets: &[&DepSet]) -> Self {
        let mut out = BTreeSet::new();
        for s in sets {
            out.extend(s.set.iter().copied());
        }
        Self { set: out }
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn contains_port(&self, port: PortId) -> bool {
        self.set.contains(&DepAtom::Port(port))
    }

    pub fn has_ports(&self) -> bool {
        self.set.iter().any(|atom| matches!(atom, DepAtom::Port(_)))
    }

    pub fn contains_flop_virtual(&self, flop: FlopId) -> bool {
        self.set.contains(&DepAtom::FlopVirtual(flop))
    }

    pub fn has_flop_virtuals(&self) -> bool {
        self.set
            .iter()
            .any(|atom| matches!(atom, DepAtom::FlopVirtual(_)))
    }

    pub fn flop_virtuals(&self) -> impl Iterator<Item = FlopId> + '_ {
        self.set.iter().filter_map(|atom| match atom {
            DepAtom::FlopVirtual(flop) => Some(*flop),
            _ => None,
        })
    }

    pub fn contains_fsm_virtual(&self, fsm: FsmId) -> bool {
        self.set.contains(&DepAtom::FsmVirtual(fsm))
    }

    pub fn has_fsm_virtuals(&self) -> bool {
        self.set
            .iter()
            .any(|atom| matches!(atom, DepAtom::FsmVirtual(_)))
    }

    pub fn fsm_virtuals(&self) -> impl Iterator<Item = FsmId> + '_ {
        self.set.iter().filter_map(|atom| match atom {
            DepAtom::FsmVirtual(fsm) => Some(*fsm),
            _ => None,
        })
    }

    pub fn contains_instance_output_virtual(&self, instance: InstanceId, port: PortId) -> bool {
        self.set
            .contains(&DepAtom::InstanceOutputVirtual { instance, port })
    }

    pub fn has_instance_output_virtuals(&self) -> bool {
        self.set
            .iter()
            .any(|atom| matches!(atom, DepAtom::InstanceOutputVirtual { .. }))
    }

    pub fn instance_output_virtuals(&self) -> impl Iterator<Item = (InstanceId, PortId)> + '_ {
        self.set.iter().filter_map(|atom| match atom {
            DepAtom::InstanceOutputVirtual { instance, port } => Some((*instance, *port)),
            _ => None,
        })
    }

    /// Rewrite virtual flop ids after a flop merge / renumbering
    /// pass. Primary-input deps are left untouched; virtual flop
    /// deps are remapped through the provided old-id -> new-id
    /// table and deduplicated naturally by the set.
    pub(crate) fn remap_flop_virtuals(&mut self, old_to_new: &[FlopId]) {
        let mut next = BTreeSet::new();
        for atom in self.set.iter().copied() {
            match atom {
                DepAtom::FlopVirtual(old) => {
                    let new = old_to_new.get(old as usize).copied().unwrap_or(old);
                    next.insert(DepAtom::FlopVirtual(new));
                }
                _ => {
                    next.insert(atom);
                }
            }
        }
        self.set = next;
    }

    /// Rewrite virtual FSM ids after an FSM merge / renumbering pass.
    /// Mirrors [`Self::remap_flop_virtuals`] for deterministic
    /// generated FSM blocks.
    pub(crate) fn remap_fsm_virtuals(&mut self, old_to_new: &[FsmId]) {
        let mut next = BTreeSet::new();
        for atom in self.set.iter().copied() {
            match atom {
                DepAtom::FsmVirtual(old) => {
                    let new = old_to_new.get(old as usize).copied().unwrap_or(old);
                    next.insert(DepAtom::FsmVirtual(new));
                }
                _ => {
                    next.insert(atom);
                }
            }
        }
        self.set = next;
    }
}

/// A design is one or more modules with a designated top.
#[derive(Debug, Clone)]
pub struct Design {
    pub top: String,
    pub modules: Vec<Module>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(id: u32, name: &str, width: u32, dir: Direction) -> Port {
        Port {
            id,
            name: name.into(),
            width,
            dir,
        }
    }

    fn comb_child(name: &str) -> Module {
        let mut module = Module {
            name: name.into(),
            ..Module::default()
        };
        module.inputs.push(port(0, "a", 8, Direction::In));
        module.outputs.push(port(1, "o", 8, Direction::Out));
        module.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        module.drives.push((1, 0));
        module
    }

    fn seq_child(name: &str) -> Module {
        let mut module = Module {
            name: name.into(),
            ..Module::default()
        };
        module.inputs.push(port(0, "clk", 1, Direction::In));
        module.inputs.push(port(1, "rst_n", 1, Direction::In));
        module.inputs.push(port(2, "a", 8, Direction::In));
        module.outputs.push(port(3, "o", 8, Direction::Out));
        module.clock = Some(0);
        module.reset = Some(1);
        module.nodes.push(Node::PrimaryInput { port: 2, width: 8 });
        module.nodes.push(Node::FlopQ { flop: 0, width: 8 });
        module.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        module.drives.push((3, 1));
        module
    }

    /// The coarse identity mode is orthogonal to the requested
    /// factorization rung: the same `factorization_level = e-graph`
    /// request dedupes in `NodeId` mode, but is forcibly disabled in
    /// `Relaxed` mode.
    #[test]
    fn identity_mode_controls_whether_nodeid_means_expression_identity() {
        use crate::config::{FactorizationLevel, IdentityMode};

        let mut m_nodeid = Module {
            max_ast_instances: 1,
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        m_nodeid
            .nodes
            .push(Node::PrimaryInput { port: 0, width: 8 });
        m_nodeid
            .nodes
            .push(Node::PrimaryInput { port: 1, width: 8 });
        let (nodeid_first, nodeid_first_new) =
            m_nodeid.intern_gate(GateOp::Add, vec![0, 1], 8, DepSet::from_port(0));
        let (nodeid_second, nodeid_second_new) =
            m_nodeid.intern_gate(GateOp::Add, vec![0, 1], 8, DepSet::from_port(0));
        assert!(nodeid_first_new);
        assert!(!nodeid_second_new);
        assert_eq!(nodeid_first, nodeid_second);
        assert_eq!(
            m_nodeid.effective_factorization_level(),
            FactorizationLevel::EGraph
        );

        let mut m_relaxed = Module {
            max_ast_instances: 1,
            identity_mode: IdentityMode::Relaxed,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        m_relaxed
            .nodes
            .push(Node::PrimaryInput { port: 0, width: 8 });
        m_relaxed
            .nodes
            .push(Node::PrimaryInput { port: 1, width: 8 });
        let (relaxed_first, relaxed_first_new) =
            m_relaxed.intern_gate(GateOp::Add, vec![0, 1], 8, DepSet::from_port(0));
        let (relaxed_second, relaxed_second_new) =
            m_relaxed.intern_gate(GateOp::Add, vec![0, 1], 8, DepSet::from_port(0));
        assert!(relaxed_first_new);
        assert!(relaxed_second_new);
        assert_ne!(relaxed_first, relaxed_second);
        assert_eq!(
            m_relaxed.effective_factorization_level(),
            FactorizationLevel::None
        );
    }

    // --- All-const arithmetic / structural evaluation ------------

    // --- Associative flattening tests -----------------------------

    #[test]
    fn sequential_descendants_keep_control_ports_visible() {
        let child = seq_child("child");

        let mut parent = Module {
            name: "parent".into(),
            ..Module::default()
        };
        parent.inputs.push(port(0, "clk", 1, Direction::In));
        parent.inputs.push(port(1, "rst_n", 1, Direction::In));
        parent.inputs.push(port(2, "a", 8, Direction::In));
        parent.clock = Some(0);
        parent.reset = Some(1);
        parent.instances.push(Instance {
            id: 0,
            name: "u_child".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(0, 0), (1, 1), (2, 2)],
            param_bindings: Vec::new(),
        });

        let modules: BTreeMap<_, _> = [(&child), (&parent)]
            .into_iter()
            .map(|module| (module.name.as_str(), module))
            .collect();

        assert!(parent.carries_sequential_state_in(Some(&modules)));
        assert!(parent.is_emitted_input_port_in(0, Some(&modules)));
        assert!(parent.is_emitted_input_port_in(1, Some(&modules)));
    }

    #[test]
    fn comb_only_descendants_keep_control_ports_hidden() {
        let child = comb_child("child");

        let mut parent = Module {
            name: "parent".into(),
            ..Module::default()
        };
        parent.inputs.push(port(0, "clk", 1, Direction::In));
        parent.inputs.push(port(1, "rst_n", 1, Direction::In));
        parent.inputs.push(port(2, "a", 8, Direction::In));
        parent.clock = Some(0);
        parent.reset = Some(1);
        parent.instances.push(Instance {
            id: 0,
            name: "u_child".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(0, 2)],
            param_bindings: Vec::new(),
        });

        let modules: BTreeMap<_, _> = [(&child), (&parent)]
            .into_iter()
            .map(|module| (module.name.as_str(), module))
            .collect();

        assert!(!parent.carries_sequential_state_in(Some(&modules)));
        assert!(!parent.is_emitted_input_port_in(0, Some(&modules)));
        assert!(!parent.is_emitted_input_port_in(1, Some(&modules)));
        assert!(parent.is_emitted_input_port_in(2, Some(&modules)));
    }
}
