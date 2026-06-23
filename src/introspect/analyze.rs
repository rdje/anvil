//! Derived-relation analysis surface (`SEMANTIC-INTROSPECTION-EXPANSION.2b.1`).
//!
//! The first *behavioral-adjacent* introspection query: the transitive
//! **combinational** fan-in **support cone** of a target. It answers
//! *"what does this output structurally depend on?"* — a relation derived by
//! pure post-hoc traversal of the already-emitted IR, never a behavioral
//! oracle and never a shadow simulator (the permanent ceiling fixed by
//! decision `0004` and restated in decision `0011`).
//!
//! # Invariant SCHEMA-DERIVED (inherited from `0004`/`0011`)
//!
//! This module computes **zero new generator truth**. A [`SupportCone`] is a
//! pure function of the `Module`/`Design` the generator already produced —
//! the same graph `metrics::compute` already walks. There is **no IR field
//! and no generator change**: the analysis is the `coverage_gaps`
//! project-don't-recompute precedent applied to dependency relations.
//!
//! # What the cone is (and where it stops)
//!
//! Starting from the node that drives a target, the analysis walks the fan-in
//! and classifies every reachable leaf:
//!
//! * [`Node::Gate`] — an internal node; its operands are recursed into. Counts
//!   toward `cone_nodes` and combinational `cone_depth`.
//! * [`Node::PrimaryInput`] — a leaf; the input **port name** is recorded in
//!   `support_inputs`.
//! * [`Node::FlopQ`] — a **register-boundary** leaf: the flop id is recorded in
//!   `support_flops` and the walk *stops* (a clock edge breaks the
//!   combinational path). The combinational cone feeding that flop's `D` is a
//!   separate, addressable target (`"flop:<id>"`).
//! * [`Node::InstanceOutput`] — a leaf: the cone **stops at the instance
//!   boundary** (decision `0011` Q3). The child output is recorded in
//!   `support_instance_outputs` as `"<instance>.<port>"`; recursing through the
//!   child is a future query kind.
//! * [`Node::Constant`] — a leaf that is *not* a support source (a constant
//!   depends on nothing); it still counts toward `cone_nodes`.
//! * [`Node::MemRead`] / [`Node::FsmOut`] — **opaque registered leaves**
//!   (default-off `memory_prob`/`fsm_prob`, so absent from the default DUT).
//!   Like `FlopQ` they break the combinational path, but the `.2a` cone shape
//!   has no memory/FSM support list, so they **terminate** the cone (counted in
//!   `cone_nodes`, recorded in no list). The memory side of that boundary is now
//!   surfaced by a *separate* query, [`QUERY_MEMORY_PROVENANCE`]
//!   (`SEMANTIC-INTROSPECTION-EXPANSION.7`): it does not recurse *through* a
//!   `MemRead` (the stored contents stay a register boundary) but reports the
//!   support cones of the memory's *input* ports. The FSM side of that boundary
//!   is likewise surfaced by [`QUERY_FSM_PROVENANCE`]
//!   (`SEMANTIC-INTROSPECTION-EXPANSION.8`): it does not recurse *through* an
//!   `FsmOut` (the registered state stays a register boundary) but reports the
//!   support cone of the FSM's one generated input, its transition-select cone
//!   `sel` (the transition/output table *values* stay out of scope — that is
//!   state-machine behaviour, not a relation).
//!
//! # Targets and addressing (decision `0011` Q1)
//!
//! * `target = None` ⇒ one cone per **output port**, in declaration order.
//! * `target = Some("<output port name>")` ⇒ that output's cone.
//! * `target = Some("flop:<id>")` ⇒ the combinational cone feeding that flop's
//!   `D` input.
//! * An unresolvable target ⇒ **no cone** (an empty `results` vec). The MCP
//!   `analyze` tool (`.2b.2`) maps that to JSON-RPC `-32602`; a *resolvable*
//!   target always yields exactly one cone, even when its support sets are
//!   empty (e.g. an undriven output or a `d = None` flop), so empty-`results`
//!   means "unknown target", never "known but empty".
//!
//! # Determinism
//!
//! Every support list is collected in a `BTreeSet` and emitted as a sorted
//! `Vec`, and outputs are visited in declaration order, so a
//! [`DerivedAnalysis`] is a byte-stable function of its inputs — the same
//! determinism contract the rest of the introspection surface holds.

use crate::ir::{
    Design, Flop, FlopKind, FlopMux, Fsm, FsmEncoding, GateOp, InstanceId, MemKind, Memory, Module,
    Node, NodeId, PortId, ResetKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

/// The query-kind string for the first derived query: the transitive
/// combinational fan-in support cone of a target.
pub const QUERY_OUTPUT_SUPPORT: &str = "output_support";

/// The query-kind string for the second derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.3`): the transitive combinational
/// fan-**out** reach of a source — the exact dual of [`QUERY_OUTPUT_SUPPORT`].
/// Served by [`module_input_reach`] / [`design_input_reach`] and dispatched by
/// the MCP `analyze` tool (`.3b.2`); listed in [`supported_query_kinds`].
pub const QUERY_INPUT_REACH: &str = "input_reach";

/// The query-kind string for the third derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.4`): per-flop **reset/data provenance** —
/// is each flop reset-defined vs data-driven, and how is its next state built?
/// A direct projection of [`Module::flops`](Module) (no graph walk). Served by
/// [`module_flop_provenance`] / [`design_flop_provenance`], dispatched by the MCP
/// `analyze` tool (`.4b.2`); listed in [`supported_query_kinds`].
pub const QUERY_FLOP_RESET_PROVENANCE: &str = "flop_reset_provenance";

/// The query-kind string for the fourth derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.5`): per-module **reachability** from the
/// design top — which modules in a [`Design`] are reachable from `design.top`
/// over the `Module.instances[].module` instance-graph edges, each module's
/// minimum depth from the top, the distinct child module names it directly
/// instantiates, and its direct instance count. A pure projection of
/// [`Design::modules`](Design) + the instance edges (no gate-graph walk — the
/// only query whose home is the whole design rather than one module's node
/// graph). Served by [`design_module_reachability`] / [`module_module_reachability`],
/// dispatched by the MCP `analyze` tool (`.5b.2`); listed in
/// [`supported_query_kinds`].
pub const QUERY_MODULE_REACHABILITY: &str = "module_reachability";

/// The query-kind string for the fifth derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.6`): the per-module **register-to-register
/// dependency graph** — for each flop, which flops' `Q` feed its `D` cone (its
/// direct register **predecessors**), which flops' `D` cones its own `Q` feeds (its
/// direct register **successors**), and whether it feeds itself (a self-feedback
/// register — a counter/accumulator). The register-level analog of
/// [`QUERY_MODULE_REACHABILITY`] (a graph over a node class), but reusing the
/// existing gate-graph support/reach machinery rather than the module table: a
/// direct register-graph edge `A → B` (`B ∈ depends_on_flops(A)`) means `B`'s `Q`
/// feeds `A`'s `D` through pure combinational logic (one register-stage hop). The
/// first query beyond decision `0011`'s four named kinds (the lane's "open-ended
/// breadth" clause), under the same `0004`/`0011` SCHEMA-DERIVED ceiling. Served by
/// [`module_flop_dependencies`] / [`design_flop_dependencies`], dispatched by the
/// MCP `analyze` tool (`.6b.2`); listed in [`supported_query_kinds`].
pub const QUERY_FLOP_DEPENDENCIES: &str = "flop_dependencies";

/// The query-kind string for the sixth derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.7`): per-inferrable-memory **port
/// provenance** — for each [`Memory`] block, its structural shape (address/data
/// width, `kind`, single-port flag) plus the [`SupportCone`] of each of its four
/// driving ports: the read address (`raddr`), write address (`waddr`), write data
/// (`wdata`), and write enable (`we`). It is the query that **opens the documented
/// opaque-`MemRead`-leaf boundary**: the five prior queries terminate the
/// combinational cone at a [`Node::MemRead`] (recorded in no support list), while
/// `memory_provenance` reports what drives the memory's *input* ports — without
/// recursing *through* the memory's stored contents (still a register boundary,
/// like a flop `Q`). Each port cone is built by the **same** [`build_cone`]
/// machinery `output_support` uses (one walker — full-factorization). The second
/// query beyond decision `0011`'s four named kinds (the lane's "open-ended breadth"
/// clause), under the same `0004`/`0011` SCHEMA-DERIVED ceiling. Served by
/// [`module_memory_provenance`] / [`design_memory_provenance`], dispatched by the
/// MCP `analyze` tool (`.7b.2`); listed in [`supported_query_kinds`].
pub const QUERY_MEMORY_PROVENANCE: &str = "memory_provenance";

/// The query-kind string for the seventh derived query: per-generated-encoding-FSM
/// **provenance**. For each [`Fsm`] block it reports the FSM's structural shape
/// (`num_states`, `encoding`, `state_width`, `sel_width`, `out_width`, `is_mealy`)
/// plus the support cone of its one generated input port — the transition-select
/// cone `sel`. It is the **direct sibling of [`QUERY_MEMORY_PROVENANCE`]**: the
/// query that **opens the documented opaque-`FsmOut`-leaf boundary**, exactly as
/// `memory_provenance` opened the `MemRead` boundary. The six prior queries
/// terminate the combinational cone at a [`Node::FsmOut`] (recorded in no support
/// list), while `fsm_provenance` reports what drives the FSM's `sel` input — without
/// recursing *through* the FSM's registered state (still a register boundary, like a
/// flop `Q`) and without surfacing the transition/output table *values* (that is the
/// construction-time-resolved state-machine behaviour, the `0004`/`0011`
/// no-behavioural-oracle non-goal). The `sel` cone is built by the **same**
/// [`build_cone`] machinery `output_support` uses (one walker — full-factorization).
/// The third query beyond decision `0011`'s four named kinds (the lane's "open-ended
/// breadth" clause), under the same `0004`/`0011` SCHEMA-DERIVED ceiling. Served by
/// [`module_fsm_provenance`] / [`design_fsm_provenance`], dispatched by the MCP
/// `analyze` tool (`.8b.2`); listed in [`supported_query_kinds`].
pub const QUERY_FSM_PROVENANCE: &str = "fsm_provenance";

/// The query-kind string for the eighth derived query
/// (`SEMANTIC-INTROSPECTION-EXPANSION.9`): per-node **immediate (1-hop) driver
/// adjacency**. For each [`Node`] it reports the node's kind, bit width, gate `op`
/// (for a [`Node::Gate`]), and the list of its direct operand drivers (each a
/// [`NodeRef`] — operand id + kind + a resolved handle), in **operand order**. It is
/// the **atomic node-level primitive complementing the transitive
/// [`QUERY_OUTPUT_SUPPORT`] cone**: where a support cone collapses the whole fan-in to
/// its boundary leaves (primary inputs / flop `Q`s / instance outputs) and records
/// neither the interior [`Node::Gate`]s it passed through nor their ops,
/// `node_drivers` exposes the node-level fan-in graph **one hop at a time** and
/// surfaces each node's [`GateOp`] — genuinely new information no prior query carries.
/// An agent can re-issue it for each operand that is itself a `Gate`, walking the DAG
/// hop by hop and reconstructing any cone itself. The fourth query beyond decision
/// `0011`'s four named kinds (the lane's "open-ended breadth" clause), under the same
/// `0004`/`0011` SCHEMA-DERIVED ceiling. Served by [`module_node_drivers`] /
/// [`design_node_drivers`], dispatched by the MCP `analyze` tool (`.9b.2`); listed in
/// [`supported_query_kinds`]. Its dual `node_readers` (immediate fan-out) is a natural
/// future sibling.
pub const QUERY_NODE_DRIVERS: &str = "node_drivers";

/// Every derived-query kind the MCP `analyze` tool answers today. The tool
/// rejects any `query` not in this set with `-32602`. A kind appears here
/// **only once its `run_analyze` dispatch is wired**, so the registry and the
/// dispatch never disagree. All four named kinds from decision `0011` are now
/// delivered; further kinds slot in the same way without changing the document
/// shape.
pub fn supported_query_kinds() -> &'static [&'static str] {
    &[
        QUERY_OUTPUT_SUPPORT,
        QUERY_INPUT_REACH,
        QUERY_FLOP_RESET_PROVENANCE,
        QUERY_MODULE_REACHABILITY,
        QUERY_FLOP_DEPENDENCIES,
        QUERY_MEMORY_PROVENANCE,
        QUERY_FSM_PROVENANCE,
    ]
}

/// The result of one derived-relation query over an artifact: a list of
/// per-target [`SupportCone`]s. A pure post-hoc projection of the emitted IR
/// (invariant SCHEMA-DERIVED) — no new computed truth.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedAnalysis {
    /// The query kind (e.g. [`QUERY_OUTPUT_SUPPORT`] or [`QUERY_INPUT_REACH`]).
    pub query: String,
    /// The [`QUERY_OUTPUT_SUPPORT`] payload: one [`SupportCone`] per resolved
    /// target. Empty iff an explicit `target` did not resolve (the MCP layer
    /// maps that to `-32602`), or iff this is an `input_reach` analysis.
    pub results: Vec<SupportCone>,
    /// The [`QUERY_INPUT_REACH`] payload: one [`ReachResult`] per resolved
    /// source. A **second parallel vec** rather than a tagged enum so the
    /// `output_support` document stays byte-identical — `skip_serializing_if`
    /// omits the key entirely on a support analysis (where it is always empty),
    /// so only an `input_reach` document carries it. Each query populates
    /// exactly one of `results` / `reach_results` / `flop_provenance`; the
    /// `query` field is the discriminator.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reach_results: Vec<ReachResult>,
    /// The [`QUERY_FLOP_RESET_PROVENANCE`] payload: one [`FlopProvenance`] per
    /// flop. A **third** parallel vec, same rationale as `reach_results`:
    /// `skip_serializing_if` keeps the `output_support` / `input_reach`
    /// documents byte-identical (the key is omitted unless this is a
    /// `flop_reset_provenance` analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flop_provenance: Vec<FlopProvenance>,
    /// The [`QUERY_MODULE_REACHABILITY`] payload: one [`ModuleReachability`] per
    /// module in the design. A **fourth** parallel vec, same rationale as
    /// `reach_results` / `flop_provenance`: `skip_serializing_if` keeps the
    /// `output_support` / `input_reach` / `flop_reset_provenance` documents
    /// byte-identical (the key is omitted unless this is a `module_reachability`
    /// analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_reachability: Vec<ModuleReachability>,
    /// The [`QUERY_FLOP_DEPENDENCIES`] payload: one [`FlopDependencies`] per flop.
    /// A **fifth** parallel vec, same rationale as `reach_results` /
    /// `flop_provenance` / `module_reachability`: `skip_serializing_if` keeps the
    /// four prior query documents byte-identical (the key is omitted unless this is
    /// a `flop_dependencies` analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flop_dependencies: Vec<FlopDependencies>,
    /// The [`QUERY_MEMORY_PROVENANCE`] payload: one [`MemoryProvenance`] per
    /// inferrable memory block. A **sixth** parallel vec, same rationale as the
    /// prior four: `skip_serializing_if` keeps the five prior query documents
    /// byte-identical (the key is omitted unless this is a `memory_provenance`
    /// analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_provenance: Vec<MemoryProvenance>,
    /// The [`QUERY_FSM_PROVENANCE`] payload: one [`FsmProvenance`] per
    /// generated-encoding FSM block. A **seventh** parallel vec, same rationale as
    /// the prior five: `skip_serializing_if` keeps the six prior query documents
    /// byte-identical (the key is omitted unless this is a `fsm_provenance`
    /// analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fsm_provenance: Vec<FsmProvenance>,
    /// The [`QUERY_NODE_DRIVERS`] payload: one [`NodeDrivers`] per IR node. An
    /// **eighth** parallel vec, same rationale as the prior six: `skip_serializing_if`
    /// keeps the seven prior query documents byte-identical (the key is omitted unless
    /// this is a `node_drivers` analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_drivers: Vec<NodeDrivers>,
}

/// The transitive **combinational** fan-in support of one target (an output
/// port, or a flop `D` addressed as `"flop:<id>"`). See the module docs for
/// the exact leaf classification and stopping rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportCone {
    /// The resolved target this cone is for: an output **port name**, or
    /// `"flop:<id>"` for a flop `D` cone.
    pub target: String,
    /// Primary-input **port names** the target combinationally depends on
    /// (sorted, deduplicated).
    pub support_inputs: Vec<String>,
    /// Flop ids whose `Q` the target combinationally depends on (sorted). The
    /// cone stops at each — the cone feeding the flop is `target = "flop:<id>"`.
    pub support_flops: Vec<u32>,
    /// Child-instance outputs the target depends on, as `"<instance>.<port>"`
    /// (sorted). The cone stops at the instance boundary.
    pub support_instance_outputs: Vec<String>,
    /// Number of distinct IR nodes in the transitive fan-in (root + internal
    /// gates + reached leaves).
    pub cone_nodes: usize,
    /// Maximum number of [`Node::Gate`] nodes on any path from the target's
    /// driver to a leaf — the combinational depth. `0` when the driver is
    /// itself a leaf or the target is undriven.
    pub cone_depth: usize,
}

/// The transitive combinational fan-**out** of one source — the dual of a
/// [`SupportCone`]. It answers *"what does this source structurally reach?"*:
/// the outputs whose support cone contains the source, and the flops whose `D`
/// cone contains it. Computed by **inverting** the support relation
/// ([`module_input_reach`]), so by construction a source `X` reaches a target
/// `T` iff `T`'s [`SupportCone`] lists `X` — the two queries cannot drift.
///
/// The `target` field is the reach **source** this result is about (an input
/// port name, a flop `Q` addressed `"flop:<id>"`, or a child-instance output
/// `"<instance>.<port>"`). It is named `target` for document-shape uniformity
/// with [`SupportCone`] — every derived-query result shares one "the entity
/// this result is about" key; the `query` kind sets its direction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachResult {
    /// The reach **source** this result is about: an input port name, a flop
    /// `Q` (`"flop:<id>"`), or a child-instance output (`"<instance>.<port>"`).
    pub target: String,
    /// Output **port names** the source combinationally reaches (sorted,
    /// deduplicated) — i.e. the outputs whose support cone contains the source.
    pub reaches_outputs: Vec<String>,
    /// Flop ids whose `D` cone the source combinationally reaches (sorted) —
    /// i.e. the flops whose `"flop:<id>"` D-cone support contains the source.
    pub reaches_flops: Vec<u32>,
    /// Total fan-out target count: `reaches_outputs.len() + reaches_flops.len()`.
    pub fanout_targets: usize,
}

/// The reset/data **provenance** of one flop — a direct projection of the
/// `Flop` the generator already built ([`module_flop_provenance`]). It answers
/// *is this register reset-defined or data-driven, and how is its next state
/// constructed?* The enum-valued fields are mapped to stable strings so the
/// wire shape survives an internal enum gaining variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlopProvenance {
    /// The flop id (addressed `"flop:<id>"`).
    pub flop: u32,
    /// Register width in bits.
    pub width: u32,
    /// Whether the flop has a reset (`reset_kind != none`).
    pub has_reset: bool,
    /// `"none"` | `"sync"` | `"async"` (from `Flop::reset_kind`).
    pub reset_kind: String,
    /// The reset value as a **decimal string** (from `Flop::reset_val`, a
    /// `u128`). A string, not a number, so 128-bit values round-trip exactly
    /// across any JSON consumer. Only meaningful when `has_reset`.
    pub reset_value: String,
    /// What `D` becomes when no mux select is asserted: `"zero"`
    /// (`FlopKind::ZeroDefault`) | `"hold"` (`FlopKind::QFeedback`).
    pub default_behavior: String,
    /// The D-mux structure: `"none"` (direct cone) | `"one_hot"` | `"encoded"`
    /// (from `Flop::mux`).
    pub mux_kind: String,
    /// Number of mux arms: `0` for `none`, the arm count for `one_hot`, the
    /// data-slot count for `encoded`.
    pub mux_arms: usize,
    /// Whether the flop has a `D` cone (`Flop::d.is_some()`); a dead/undriven
    /// flop has none.
    pub has_d: bool,
}

/// Where one module sits in a design's instance graph — the
/// [`QUERY_MODULE_REACHABILITY`] payload. A pure projection of
/// [`Design::modules`](Design) + the `Module.instances[].module` edges: whether
/// the module is reachable from `design.top`, its minimum instance-graph distance
/// from the top, the distinct child module names it directly instantiates (its
/// local out-edges), and its total direct instance count.
///
/// Both [`InstanceRole`](crate::ir::InstanceRole) kinds (`PlannedChild` and
/// `ParentCone` helper instances) are genuine instance edges, so both are
/// traversed for reachability and both contribute to `instantiates` /
/// `instance_count`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleReachability {
    /// The module name (the entity this entry is about).
    pub module: String,
    /// Whether the module is reachable from `design.top` via the instance graph.
    pub reachable: bool,
    /// The minimum instance-graph distance from `design.top` (`0` for the top
    /// itself). Present iff `reachable`; omitted for an unreachable module, for
    /// which a distance is undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    /// The distinct child **module names** this module directly instantiates
    /// (sorted, deduplicated) — its local out-edges in the instance graph.
    /// Present for every module, reachable or not (a local structural fact).
    pub instantiates: Vec<String>,
    /// The total number of direct child instances (`Module::instances` length);
    /// `>= instantiates.len()` when a child module is instantiated more than once.
    pub instance_count: usize,
}

/// One flop's place in a module's **register-to-register dependency graph** — the
/// [`QUERY_FLOP_DEPENDENCIES`] payload. A pure projection of the existing
/// support/reach machinery: the cone feeding this flop's `D` (its `support_flops`
/// are the predecessors) and the transpose (the flops this flop's `Q` reaches are
/// the successors). Every edge is a **direct** register-graph edge — one
/// register-stage hop through pure combinational logic — because the underlying
/// support cone is transitive combinational and stops at every register boundary.
///
/// Honest framing: each edge is individually derivable from `output_support` /
/// `input_reach` on a `"flop:<id>"` target, but no single one of those returns the
/// whole register graph; per the agent-API audience rule (decision `0011` /
/// `feedback_api_for_agents_not_humans`) this is the complete register-graph **view**
/// in one query — a relation over the emitted IR, never behaviour.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlopDependencies {
    /// The flop id (addressed `"flop:<id>"`, the entity this entry is about).
    pub flop: u32,
    /// Direct register **predecessors**: flop ids whose `Q` feeds this flop's `D`
    /// cone, i.e. this flop's D-cone `support_flops` (sorted, deduplicated).
    pub depends_on_flops: Vec<u32>,
    /// Direct register **successors**: flop ids whose `D` cone this flop's `Q`
    /// feeds, i.e. the transpose of `depends_on_flops` across the module (sorted,
    /// deduplicated).
    pub driven_flops: Vec<u32>,
    /// Whether this flop feeds **itself** (`flop ∈ depends_on_flops`): its `Q`
    /// reaches its own `D` through pure combinational logic — the structural marker
    /// of a self-feedback register (a counter / accumulator / running-state flop).
    pub self_dependent: bool,
}

/// One inferrable memory block's **port provenance** — the
/// [`QUERY_MEMORY_PROVENANCE`] payload. A pure projection of the [`Memory`] the
/// generator already built: its structural shape plus the [`SupportCone`] of each
/// of its four driving ports. It answers *what drives this memory's read address /
/// write address / write data / write enable?* — opening the boundary the opaque
/// [`Node::MemRead`] leaf hides from the other queries' cones, without recursing
/// *through* the memory's stored contents (a register boundary).
///
/// Each `*_support` field is built by the **same** [`build_cone`] machinery
/// `output_support` uses, so a memory port cone classifies its leaves exactly like
/// an output cone (primary inputs / flop `Q`s / child-instance outputs; opaque
/// `MemRead`/`FsmOut` terminate it) and each port's `target` is `"mem:<id>.<port>"`.
/// For a `SinglePort` memory the read and write addresses are the *same* node, so the
/// two address cones carry **identical support** (only their `target` labels —
/// `.raddr` vs `.waddr` — differ); `single_port` flags this.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryProvenance {
    /// The memory id (addressed `"mem:<id>"`, the entity this entry is about).
    pub mem: u32,
    /// Address width in bits (`Memory::addr_width`).
    pub addr_width: u32,
    /// Data (word) width in bits (`Memory::data_width`).
    pub data_width: u32,
    /// `"single_port"` (one shared synchronous read/write address) |
    /// `"simple_dual_port"` (one write port + one independent read port), from
    /// [`MemKind`]. Mapped to a stable string so the wire shape survives the enum
    /// gaining variants.
    pub kind: String,
    /// Whether the read and write addresses are the same cone
    /// (`MemKind::SinglePort`) ⇒ `read_addr_support == write_addr_support`.
    pub single_port: bool,
    /// Support cone feeding the **read address** (`Memory::raddr`); cone `target`
    /// `"mem:<id>.raddr"`.
    pub read_addr_support: SupportCone,
    /// Support cone feeding the **write address** (`Memory::waddr`); cone `target`
    /// `"mem:<id>.waddr"`.
    pub write_addr_support: SupportCone,
    /// Support cone feeding the **write data** (`Memory::wdata`); cone `target`
    /// `"mem:<id>.wdata"`.
    pub write_data_support: SupportCone,
    /// Support cone feeding the **write enable** (`Memory::we`); cone `target`
    /// `"mem:<id>.we"`.
    pub write_enable_support: SupportCone,
}

/// [`QUERY_FSM_PROVENANCE`] payload. A pure projection of the [`Fsm`] the generator
/// already built: its structural shape plus the [`SupportCone`] of its one generated
/// input port, the transition-select cone `sel`. It answers *what drives this FSM's
/// state transitions (and, for a Mealy FSM, its output decode)?* — opening the
/// boundary the opaque [`Node::FsmOut`] leaf hides from the other queries' cones,
/// without recursing *through* the FSM's registered state (a register boundary, like
/// a flop `Q`) and without surfacing the transition/output table *values* (the
/// construction-time-resolved state-machine behaviour — a deliberate boundary, the
/// [`MemoryProvenance`] precedent of not recursing through stored contents).
///
/// An FSM has exactly **one** generated input cone (`sel`) — unlike a memory's four
/// ports — because its transition/output tables are construction-time *constants*
/// (`Vec<u32>` / `Vec<u128>`), not [`NodeId`] cones. The `sel_support` field is built
/// by the **same** [`build_cone`] machinery `output_support` uses, so it classifies
/// its leaves exactly like an output cone (primary inputs / flop `Q`s / child-instance
/// outputs; opaque `MemRead`/`FsmOut` terminate it) and its `target` is
/// `"fsm:<id>.sel"`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsmProvenance {
    /// The FSM id (addressed `"fsm:<id>"`, the entity this entry is about).
    pub fsm: u32,
    /// Number of states (`>= 1`; reset state is index 0), from [`Fsm::num_states`].
    pub num_states: u32,
    /// `"binary"` | `"one_hot"` | `"gray"`, from [`FsmEncoding`]. Mapped to a stable
    /// string so the wire shape survives the enum gaining variants.
    pub encoding: String,
    /// Width of the encoded `state_q` register ([`FsmEncoding::state_width`] for this
    /// encoding + state count).
    pub state_width: u32,
    /// Width of the transition-select cone `sel` (its `1 << sel_width` values index
    /// the transition/output tables), from [`Fsm::sel_width`].
    pub sel_width: u32,
    /// Width of the registered output (`Node::FsmOut.width` == [`Fsm::out_width`]).
    pub out_width: u32,
    /// Whether the output is **Mealy** (a combinational decode over `(state_q, sel)`)
    /// vs **Moore** (state only), from [`Fsm::is_mealy`]. The FSM analog of
    /// [`MemoryProvenance::single_port`] — a one-bit structural output-shape flag.
    pub is_mealy: bool,
    /// Support cone feeding the transition-select input `sel` (`Fsm::sel`); cone
    /// `target` `"fsm:<id>.sel"`. The one generated input cone of an FSM.
    pub sel_support: SupportCone,
}

/// One IR node's **immediate (1-hop) driver adjacency** — the [`QUERY_NODE_DRIVERS`]
/// payload. A pure projection of one entry of [`Module::nodes`](Module): the node's
/// kind, bit width, gate `op` (for a [`Node::Gate`]), and the list of its direct
/// operand [`NodeRef`]s. Unlike a [`SupportCone`] (which collapses the transitive
/// fan-in to its boundary leaves and never names an interior gate or its op), this is
/// the **one-hop** relation for **every** node — the atomic primitive the cone queries
/// are built from.
///
/// `drivers` are kept in **operand order** (not sorted, unlike the cone support lists),
/// because operand order is semantically meaningful (`a - b` ≠ `b - a`; a `Mux`'s
/// `[sel, a, b]`); it is still deterministic, because the underlying operand `Vec` is.
/// A leaf node (every variant except `Gate`) has an empty `drivers` and no `op`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDrivers {
    /// The node id (its index in [`Module::nodes`](Module); addressed `"node:<id>"`,
    /// the entity this entry is about).
    pub node: u32,
    /// The node's kind: `"primary_input"` | `"constant"` | `"flop_q"` | `"mem_read"` |
    /// `"fsm_out"` | `"instance_output"` | `"gate"` (the seven [`Node`] variants,
    /// mapped to stable strings so the wire shape survives the enum gaining variants).
    pub kind: String,
    /// For a [`Node::Gate`], its [`GateOp`] as a stable string (e.g. `"and"`, `"mux"`,
    /// `"slice"`); omitted (`None`) for every leaf node. The base op name only — the
    /// parameterized variants' params (`Slice { hi, lo }`,
    /// `ForFold { kind, trip_count, chunk_width }`) are a documented future extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// The node's bit width ([`Node::width`]).
    pub width: u32,
    /// The node's **immediate** operand drivers, in operand order: one [`NodeRef`] per
    /// operand of a [`Node::Gate`]; empty for a leaf node.
    pub drivers: Vec<NodeRef>,
}

/// A reference to one operand node within a [`NodeDrivers`] entry — the operand's id,
/// kind, and a resolved handle. A `NodeRef` carries enough to both re-address the
/// operand (`node` / `"node:<id>"`) and recognise it (`name`) without a second query.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRef {
    /// The operand node id (addressed `"node:<id>"`).
    pub node: u32,
    /// The operand node kind (same vocabulary as [`NodeDrivers::kind`]).
    pub kind: String,
    /// A resolved, usable handle for the operand: a primary-input **port name**, a flop
    /// `Q` `"flop:<id>"`, a memory read `"mem:<id>"`, an FSM output `"fsm:<id>"`, a
    /// child-instance output `"<instance>.<port>"`, or `"node:<id>"` for an interior
    /// gate / constant (which has no external handle). Always a usable address string —
    /// the lane's resolve-handles convention ([`SupportCone`] never records a raw id).
    pub name: String,
}

/// Compute the output-support analysis for a single [`Module`].
///
/// `target = None` ⇒ a cone per output port. Instance-output leaves are named
/// `"<instance>.port<id>"` here because a bare module carries no child
/// definitions to resolve the child port name; use [`design_support_cones`]
/// for fully-resolved `"<instance>.<port-name>"` leaves. (Leaf DUT modules
/// have no instances, so this fallback is rarely exercised in practice.)
pub fn module_support_cones(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    support_cones_with(m, target, &fmt)
}

/// Compute the output-support analysis for the **top** module of a
/// [`Design`], resolving each child-instance-output leaf to its
/// `"<instance>.<child-output-port-name>"` form via the design's module table.
/// Returns an empty analysis when the named top module is absent.
pub fn design_support_cones(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_OUTPUT_SUPPORT.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    support_cones_with(top, target, &fmt)
}

/// Compute the `input_reach` analysis for a single [`Module`]: the transitive
/// combinational fan-**out** of each source (the dual of
/// [`module_support_cones`]).
///
/// `target = None` ⇒ a [`ReachResult`] per source (every input port, then every
/// flop `Q`, then every child-instance output present in the IR). An explicit,
/// resolvable `target` (an input port name, `"flop:<id>"`, or
/// `"<instance>.<port>"`) yields exactly one result — even when it reaches
/// nothing; an unresolvable target yields none (→ `-32602` at the MCP layer).
/// Instance-output leaves are named `"<instance>.port<id>"` here; use
/// [`design_input_reach`] for fully-resolved child port names.
pub fn module_input_reach(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    input_reach_with(m, target, &fmt)
}

/// Compute the `input_reach` analysis for the **top** module of a [`Design`],
/// resolving each child-instance-output source/leaf to its
/// `"<instance>.<child-output-port-name>"` form. Returns an empty analysis when
/// the named top module is absent.
pub fn design_input_reach(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_INPUT_REACH.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    input_reach_with(top, target, &fmt)
}

/// Compute the `flop_reset_provenance` analysis for a single [`Module`]: the
/// per-flop reset/data provenance (a direct projection of [`Module::flops`](Module)).
///
/// `target = None` ⇒ a [`FlopProvenance`] per flop, in ascending id order.
/// `target = Some("flop:<id>")` ⇒ that one flop; any other string (or an
/// out-of-range id) ⇒ no result (→ `-32602` at the MCP layer). A module with no
/// flops + `target = None` ⇒ an empty `flop_provenance` (the honest "no flops"
/// answer, not an error).
pub fn module_flop_provenance(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    flop_provenance_with(m, target)
}

/// Compute the `flop_reset_provenance` analysis for the **top** module of a
/// [`Design`]. Returns an empty analysis when the named top module is absent.
/// (Per-child-module flop provenance is a future extension; like the other
/// queries this operates on the top module.)
pub fn design_flop_provenance(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_FLOP_RESET_PROVENANCE.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    flop_provenance_with(top, target)
}

/// Compute the `module_reachability` analysis for a [`Design`]: which modules are
/// reachable from `design.top` via the instance graph, with each module's minimum
/// depth from the top, the distinct child module names it instantiates, and its
/// direct instance count.
///
/// `target = None` ⇒ one [`ModuleReachability`] per module in `design.modules`, in
/// ascending module-name order (deterministic, independent of the modules-vec /
/// instance order). `target = Some("<module name>")` ⇒ that one module's entry (it
/// must be a module in `design.modules`); an unknown name ⇒ no result (→ `-32602`
/// at the MCP layer). The top module is `reachable` at `depth = Some(0)`.
///
/// Defensive edge case: if `design.top` is not a module in `design.modules` (a
/// malformed design — a real ANVIL design always names a present top), the BFS
/// finds nothing and every module is reported `reachable: false`. This is the
/// honest, complete enumeration; unlike the other `design_*` builders (which
/// analyze the top module and early-return empty when it is absent),
/// `module_reachability` is a whole-module-table query, so it still answers for
/// every module that is present.
pub fn design_module_reachability(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    // Index the module table by name and BFS the instance graph from the top,
    // recording each reachable module's minimum depth. Both `InstanceRole` kinds
    // are real edges. A child name with no matching module is a recorded out-edge
    // that cannot be traversed (defensive — never panics on a malformed design).
    let by_name: HashMap<&str, &Module> = design
        .modules
        .iter()
        .map(|m| (m.name.as_str(), m))
        .collect();
    let mut depth: HashMap<&str, usize> = HashMap::new();
    if by_name.contains_key(design.top.as_str()) {
        let mut queue: VecDeque<&str> = VecDeque::new();
        depth.insert(design.top.as_str(), 0);
        queue.push_back(design.top.as_str());
        while let Some(name) = queue.pop_front() {
            let Some(&d) = depth.get(name) else { continue };
            let Some(m) = by_name.get(name).copied() else {
                continue;
            };
            // Distinct child names, visited in sorted order (min-depth BFS is
            // order-independent; sorting removes any doubt about determinism).
            let children: BTreeSet<&str> = m.instances.iter().map(|i| i.module.as_str()).collect();
            for child in children {
                if by_name.contains_key(child) && !depth.contains_key(child) {
                    depth.insert(child, d + 1);
                    queue.push_back(child);
                }
            }
        }
    }
    // Emit one entry per module, sorted by name (the determinism contract).
    let mut modules: Vec<&Module> = design.modules.iter().collect();
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    let mut module_reachability = Vec::new();
    for m in modules {
        if let Some(t) = target {
            if m.name != t {
                continue;
            }
        }
        module_reachability.push(reachability_of(m, depth.get(m.name.as_str()).copied()));
    }
    DerivedAnalysis {
        query: QUERY_MODULE_REACHABILITY.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability,
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Compute the `module_reachability` analysis for a single [`Module`] — the
/// **degenerate one-node case**. A bare module carries no child definitions to
/// traverse (the "no child defs" boundary the module variant of every query
/// hits — cf. [`format_instance_leaf_module`]), so the instance graph is a single
/// node rooted at the module itself: one [`ModuleReachability`] for `m`
/// (`reachable = true`, `depth = Some(0)`, its own distinct instantiated child
/// names + instance count). Full module-graph reachability needs a [`Design`];
/// use [`design_module_reachability`].
///
/// `target = None` or `target = Some(&m.name)` ⇒ that one entry; any other target
/// ⇒ no result (→ `-32602` at the MCP layer).
pub fn module_module_reachability(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let include = match target {
        None => true,
        Some(t) => t == m.name,
    };
    let module_reachability = if include {
        vec![reachability_of(m, Some(0))]
    } else {
        Vec::new()
    };
    DerivedAnalysis {
        query: QUERY_MODULE_REACHABILITY.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability,
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Compute the `flop_dependencies` analysis for a single [`Module`]: the
/// register-to-register dependency graph (per flop, its direct register
/// predecessors / successors / self-feedback flag).
///
/// `target = None` ⇒ a [`FlopDependencies`] per flop, in ascending id order.
/// `target = Some("flop:<id>")` ⇒ that one flop's entry (even if it has no
/// register predecessor/successor — an empty-edges entry, not an error); any other
/// string (or an out-of-range id) ⇒ no result (→ `-32602` at the MCP layer). A
/// module with no flops + `target = None` ⇒ an empty `flop_dependencies` (the
/// honest "no flops" answer). The whole register graph is always built first (a
/// flop's successors require every flop's cone), then the requested entries are
/// emitted — the same "compute-all-then-filter" shape [`input_reach_with`] holds.
pub fn module_flop_dependencies(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    flop_dependencies_with(m, target, &fmt)
}

/// Compute the `flop_dependencies` analysis for the **top** module of a
/// [`Design`]. Returns an empty analysis when the named top module is absent.
/// (Per-child-module flop dependencies are a future extension; like
/// `flop_reset_provenance` this operates on the top module.)
pub fn design_flop_dependencies(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_FLOP_DEPENDENCIES.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    flop_dependencies_with(top, target, &fmt)
}

/// Shared driver for [`module_flop_dependencies`] / [`design_flop_dependencies`]:
/// build the whole register graph, then emit the requested [`FlopDependencies`].
///
/// Step 1 reuses the **same** cone machinery `output_support` uses: each flop's
/// `D`-cone `support_flops` are its direct register **predecessors**. Step 2 takes
/// the transpose of that relation (`B ∈ depends_on(A)` ⇔ `A ∈ driven(B)`) to get
/// successors — exactly the inversion [`input_reach_with`] performs, restricted to
/// flop sources/targets — so the two directions cannot drift and there is no second
/// walker / re-derived boundary rule. `self_dependent` is `flop ∈ depends_on_flops`
/// (a register whose `Q` feeds its own `D` combinationally). Cost is
/// `O(flops × cone)`, bounded by module size (a read-only analysis).
fn flop_dependencies_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    let mut flops: Vec<&Flop> = m.flops.iter().collect();
    flops.sort_by_key(|f| f.id); // deterministic, independent of vec order

    // 1. Predecessors: each flop's D-cone support_flops. 2. Successors: the
    //    transpose. Sorted sets ⇒ deterministic bytes.
    let mut predecessors: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let mut successors: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for f in &flops {
        let cone = build_cone(m, format!("flop:{}", f.id), f.d, fmt);
        let preds: BTreeSet<u32> = cone.support_flops.iter().copied().collect();
        for &p in &preds {
            successors.entry(p).or_default().insert(f.id);
        }
        predecessors.insert(f.id, preds);
    }

    let make = |f: &Flop| -> FlopDependencies {
        let depends_on_flops: Vec<u32> = predecessors
            .get(&f.id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        let driven_flops: Vec<u32> = successors
            .get(&f.id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        let self_dependent = depends_on_flops.contains(&f.id);
        FlopDependencies {
            flop: f.id,
            depends_on_flops,
            driven_flops,
            self_dependent,
        }
    };

    let mut flop_dependencies = Vec::new();
    match target {
        None => flop_dependencies.extend(flops.iter().map(|f| make(f))),
        Some(t) => {
            // Only `"flop:<id>"` is a valid target; anything else (or an
            // out-of-range id) ⇒ no result ⇒ `-32602` at the MCP layer.
            if let Some(id) = t.strip_prefix("flop:").and_then(|r| r.parse::<u32>().ok()) {
                if let Some(f) = flops.iter().find(|f| f.id == id) {
                    flop_dependencies.push(make(f));
                }
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_FLOP_DEPENDENCIES.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies,
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Compute the `memory_provenance` analysis for a single [`Module`]: the per-memory
/// port provenance (its structural shape + the support cone of each of its four
/// driving ports).
///
/// `target = None` ⇒ a [`MemoryProvenance`] per memory, in ascending id order.
/// `target = Some("mem:<id>")` ⇒ that one memory's entry; any other string (or an
/// out-of-range id) ⇒ no result (→ `-32602` at the MCP layer). A module with no
/// memories + `target = None` ⇒ an empty `memory_provenance` (the honest "no
/// memories" answer; `memory_prob` is default-off, so the default DUT hits this).
/// Instance-output support in a port cone is named `"<instance>.port<id>"` here;
/// use [`design_memory_provenance`] for fully-resolved child port names.
pub fn module_memory_provenance(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    memory_provenance_with(m, target, &fmt)
}

/// Compute the `memory_provenance` analysis for the **top** module of a [`Design`],
/// resolving each instance-output leaf in a port cone to its
/// `"<instance>.<child-output-port-name>"` form. Returns an empty analysis when the
/// named top module is absent. (Per-child-module memory provenance is a future
/// extension; like `flop_reset_provenance` this operates on the top module.)
pub fn design_memory_provenance(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_MEMORY_PROVENANCE.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    memory_provenance_with(top, target, &fmt)
}

/// Shared driver for [`module_memory_provenance`] / [`design_memory_provenance`]:
/// project `m.memories` (ascending id) into [`MemoryProvenance`]s, building each
/// port's [`SupportCone`] with the **same** [`build_cone`] machinery
/// `output_support` uses (one walker — full-factorization). The
/// `we`/`waddr`/`wdata`/`raddr` cones are plain [`NodeId`]s (never `None`), so each
/// port cone is rooted at `Some(node)`. A `SinglePort` memory's `raddr` *is* its
/// `waddr` node, so its read/write address cones are identical by construction
/// (flagged by `single_port`).
fn memory_provenance_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    let mut mems: Vec<&Memory> = m.memories.iter().collect();
    mems.sort_by_key(|mem| mem.id); // deterministic, independent of vec order

    let make = |mem: &Memory| -> MemoryProvenance {
        let port_cone = |label: &str, node: NodeId| {
            build_cone(m, format!("mem:{}.{}", mem.id, label), Some(node), fmt)
        };
        let kind = match mem.kind {
            MemKind::SinglePort => "single_port",
            MemKind::SimpleDualPort => "simple_dual_port",
        };
        MemoryProvenance {
            mem: mem.id,
            addr_width: mem.addr_width,
            data_width: mem.data_width,
            kind: kind.to_string(),
            single_port: matches!(mem.kind, MemKind::SinglePort),
            read_addr_support: port_cone("raddr", mem.raddr),
            write_addr_support: port_cone("waddr", mem.waddr),
            write_data_support: port_cone("wdata", mem.wdata),
            write_enable_support: port_cone("we", mem.we),
        }
    };

    let mut memory_provenance = Vec::new();
    match target {
        None => memory_provenance.extend(mems.iter().map(|mem| make(mem))),
        Some(t) => {
            // Only the `"mem:<id>"` form is a valid target here; anything else
            // (or an out-of-range id) ⇒ no result ⇒ `-32602` at the MCP layer.
            if let Some(id) = t.strip_prefix("mem:").and_then(|r| r.parse::<u32>().ok()) {
                if let Some(mem) = mems.iter().find(|mem| mem.id == id) {
                    memory_provenance.push(make(mem));
                }
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_MEMORY_PROVENANCE.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance,
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Compute the `fsm_provenance` analysis for a single [`Module`]: the per-FSM
/// provenance (its structural shape + the support cone of its one generated input
/// port, the transition-select cone `sel`).
///
/// `target = None` ⇒ a [`FsmProvenance`] per FSM, in ascending id order.
/// `target = Some("fsm:<id>")` ⇒ that one FSM's entry; any other string (or an
/// out-of-range id) ⇒ no result (→ `-32602` at the MCP layer). A module with no
/// FSMs + `target = None` ⇒ an empty `fsm_provenance` (the honest "no FSMs" answer;
/// `fsm_prob` is default-off, so the default DUT hits this). Instance-output support
/// in the `sel` cone is named `"<instance>.port<id>"` here; use
/// [`design_fsm_provenance`] for fully-resolved child port names.
pub fn module_fsm_provenance(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    fsm_provenance_with(m, target, &fmt)
}

/// Compute the `fsm_provenance` analysis for the **top** module of a [`Design`],
/// resolving each instance-output leaf in the `sel` cone to its
/// `"<instance>.<child-output-port-name>"` form. Returns an empty analysis when the
/// named top module is absent. (Per-child-module FSM provenance is a future
/// extension; like `flop_reset_provenance` / `memory_provenance` this operates on the
/// top module.)
pub fn design_fsm_provenance(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_FSM_PROVENANCE.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    fsm_provenance_with(top, target, &fmt)
}

/// Shared driver for [`module_fsm_provenance`] / [`design_fsm_provenance`]: project
/// `m.fsms` (ascending id) into [`FsmProvenance`]s, building each FSM's one `sel`
/// support cone with the **same** [`build_cone`] machinery `output_support` uses (one
/// walker — full-factorization). `Fsm::sel` is a plain [`NodeId`] (never `None`), so
/// the cone is rooted at `Some(node)`. The structural fields are a direct projection
/// of the [`Fsm`] (`FsmEncoding` → a stable string + [`FsmEncoding::state_width`]);
/// the transition/output table *values* are deliberately not surfaced (state-machine
/// behaviour, not a relation).
fn fsm_provenance_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    let mut fsms: Vec<&Fsm> = m.fsms.iter().collect();
    fsms.sort_by_key(|fsm| fsm.id); // deterministic, independent of vec order

    let make = |fsm: &Fsm| -> FsmProvenance {
        let encoding = match fsm.encoding {
            FsmEncoding::Binary => "binary",
            FsmEncoding::OneHot => "one_hot",
            FsmEncoding::Gray => "gray",
        };
        FsmProvenance {
            fsm: fsm.id,
            num_states: fsm.num_states,
            encoding: encoding.to_string(),
            state_width: fsm.encoding.state_width(fsm.num_states),
            sel_width: fsm.sel_width,
            out_width: fsm.out_width,
            is_mealy: fsm.is_mealy(),
            sel_support: build_cone(m, format!("fsm:{}.sel", fsm.id), Some(fsm.sel), fmt),
        }
    };

    let mut fsm_provenance = Vec::new();
    match target {
        None => fsm_provenance.extend(fsms.iter().map(|fsm| make(fsm))),
        Some(t) => {
            // Only the `"fsm:<id>"` form is a valid target here; anything else
            // (or an out-of-range id) ⇒ no result ⇒ `-32602` at the MCP layer.
            if let Some(id) = t.strip_prefix("fsm:").and_then(|r| r.parse::<u32>().ok()) {
                if let Some(fsm) = fsms.iter().find(|fsm| fsm.id == id) {
                    fsm_provenance.push(make(fsm));
                }
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_FSM_PROVENANCE.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance,
        node_drivers: Vec::new(),
    }
}

/// Compute the `node_drivers` analysis for a single [`Module`]: per-node immediate
/// (1-hop) driver adjacency (each node's kind, width, gate `op`, and its direct
/// operand [`NodeRef`]s in operand order).
///
/// `target = None` ⇒ one [`NodeDrivers`] per node in [`Module::nodes`](Module), in
/// ascending node-id (= index) order — the whole node-level fan-in adjacency.
/// `target = Some("node:<id>")` ⇒ that one node's entry (a **leaf** node yields a
/// known-but-empty `drivers`, not an error); any other string, or an out-of-range id
/// (`id >= nodes.len()`), ⇒ no result (→ `-32602` at the MCP layer). Instance-output
/// operand handles are named `"<instance>.port<id>"` here; use [`design_node_drivers`]
/// for fully-resolved child port names.
pub fn module_node_drivers(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_module(m, inst, port);
    node_drivers_with(m, target, &fmt)
}

/// Compute the `node_drivers` analysis for the **top** module of a [`Design`],
/// resolving each instance-output operand handle to its
/// `"<instance>.<child-output-port-name>"` form. Returns an empty analysis when the
/// named top module is absent. (Per-child-module node drivers are a future extension;
/// like the other gate-graph queries this operates on the top module.)
pub fn design_node_drivers(design: &Design, target: Option<&str>) -> DerivedAnalysis {
    let Some(top) = design.modules.iter().find(|m| m.name == design.top) else {
        return DerivedAnalysis {
            query: QUERY_NODE_DRIVERS.to_string(),
            results: Vec::new(),
            reach_results: Vec::new(),
            flop_provenance: Vec::new(),
            module_reachability: Vec::new(),
            flop_dependencies: Vec::new(),
            memory_provenance: Vec::new(),
            fsm_provenance: Vec::new(),
            node_drivers: Vec::new(),
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    node_drivers_with(top, target, &fmt)
}

/// Shared driver for [`module_node_drivers`] / [`design_node_drivers`]: a single
/// one-hop pass over `m.nodes` — no transitive walk / no DFS / no memoization (the
/// purest projection alongside `flop_reset_provenance`; the `visit` walker is for the
/// transitive cone queries). Each node maps to a [`NodeDrivers`] whose `drivers` are
/// its direct operands (a `Gate`'s `operands`, in operand order; empty for a leaf)
/// resolved through [`node_ref_of`]. Pure: reads `m.nodes` only, no IR/generator change.
fn node_drivers_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    let make = |id: usize, node: &Node| -> NodeDrivers {
        let (op, drivers) = match node {
            Node::Gate { op, operands, .. } => (
                Some(gate_op_str(op).to_string()),
                operands.iter().map(|&o| node_ref_of(m, o, fmt)).collect(),
            ),
            _ => (None, Vec::new()),
        };
        NodeDrivers {
            node: id as u32,
            kind: node_kind_str(node).to_string(),
            op,
            width: node.width(),
            drivers,
        }
    };

    let mut node_drivers = Vec::new();
    match target {
        None => {
            for (id, node) in m.nodes.iter().enumerate() {
                node_drivers.push(make(id, node));
            }
        }
        Some(t) => {
            // Only the `"node:<id>"` form is a valid target; anything else (or an
            // out-of-range id) ⇒ no result ⇒ `-32602` at the MCP layer.
            if let Some(id) = t
                .strip_prefix("node:")
                .and_then(|r| r.parse::<usize>().ok())
            {
                if let Some(node) = m.nodes.get(id) {
                    node_drivers.push(make(id, node));
                }
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_NODE_DRIVERS.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers,
    }
}

/// The stable string kind of a [`Node`] (mirrors the seven variants).
fn node_kind_str(node: &Node) -> &'static str {
    match node {
        Node::PrimaryInput { .. } => "primary_input",
        Node::Constant { .. } => "constant",
        Node::FlopQ { .. } => "flop_q",
        Node::MemRead { .. } => "mem_read",
        Node::FsmOut { .. } => "fsm_out",
        Node::InstanceOutput { .. } => "instance_output",
        Node::Gate { .. } => "gate",
    }
}

/// The stable string name of a [`GateOp`] (base op name only; the parameterized
/// variants' params are a documented future extension).
fn gate_op_str(op: &GateOp) -> &'static str {
    match op {
        GateOp::And => "and",
        GateOp::Or => "or",
        GateOp::Xor => "xor",
        GateOp::Not => "not",
        GateOp::Add => "add",
        GateOp::Sub => "sub",
        GateOp::Mul => "mul",
        GateOp::Eq => "eq",
        GateOp::Neq => "neq",
        GateOp::Lt => "lt",
        GateOp::Gt => "gt",
        GateOp::Le => "le",
        GateOp::Ge => "ge",
        GateOp::Mux => "mux",
        GateOp::CaseMux => "case_mux",
        GateOp::CasezMux => "casez_mux",
        GateOp::ForFold { .. } => "for_fold",
        GateOp::Slice { .. } => "slice",
        GateOp::Concat => "concat",
        GateOp::RedAnd => "red_and",
        GateOp::RedOr => "red_or",
        GateOp::RedXor => "red_xor",
        GateOp::Shl => "shl",
        GateOp::Shr => "shr",
    }
}

/// Build a [`NodeRef`] for operand node `id`: its kind plus a resolved handle. A
/// defensive out-of-bounds id (a well-formed IR never has one, but the read-mostly
/// surface must not panic) resolves to a `"node:<id>"` ref of kind `"node"`.
fn node_ref_of(m: &Module, id: NodeId, fmt: &dyn Fn(InstanceId, PortId) -> String) -> NodeRef {
    let (kind, name) = match m.nodes.get(id as usize) {
        Some(node @ Node::PrimaryInput { port, .. }) => {
            (node_kind_str(node), input_port_name(m, *port))
        }
        Some(node @ Node::FlopQ { flop, .. }) => (node_kind_str(node), format!("flop:{flop}")),
        Some(node @ Node::MemRead { mem, .. }) => (node_kind_str(node), format!("mem:{mem}")),
        Some(node @ Node::FsmOut { fsm, .. }) => (node_kind_str(node), format!("fsm:{fsm}")),
        Some(node @ Node::InstanceOutput { instance, port, .. }) => {
            (node_kind_str(node), fmt(*instance, *port))
        }
        // A constant / interior gate has no external handle ⇒ address it by node id.
        Some(node @ (Node::Constant { .. } | Node::Gate { .. })) => {
            (node_kind_str(node), format!("node:{id}"))
        }
        None => ("node", format!("node:{id}")),
    };
    NodeRef {
        node: id,
        kind: kind.to_string(),
        name,
    }
}

/// Build one [`ModuleReachability`] for `m`. `depth` is `Some(d)` (the BFS
/// distance from the top) when the module is reachable, `None` when it is not;
/// `reachable` is `depth.is_some()`.
fn reachability_of(m: &Module, depth: Option<usize>) -> ModuleReachability {
    ModuleReachability {
        module: m.name.clone(),
        reachable: depth.is_some(),
        depth,
        instantiates: distinct_instantiated(m),
        instance_count: m.instances.len(),
    }
}

/// The distinct child module names a module directly instantiates (sorted,
/// deduplicated) — its local out-edges in the instance graph. Includes both
/// `PlannedChild` and `ParentCone` helper instances (both are real edges).
fn distinct_instantiated(m: &Module) -> Vec<String> {
    m.instances
        .iter()
        .map(|i| i.module.clone())
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}

/// Shared driver for [`module_flop_provenance`] / [`design_flop_provenance`]:
/// project `m.flops` (ascending id) into [`FlopProvenance`]s, honouring the
/// requested `target`.
fn flop_provenance_with(m: &Module, target: Option<&str>) -> DerivedAnalysis {
    let mut flops: Vec<&Flop> = m.flops.iter().collect();
    flops.sort_by_key(|f| f.id); // deterministic, independent of vec order

    let mut flop_provenance = Vec::new();
    match target {
        None => flop_provenance.extend(flops.iter().map(|f| flop_provenance_of(f))),
        Some(t) => {
            // Only the `"flop:<id>"` form is a valid target here; anything else
            // (or an out-of-range id) ⇒ no result ⇒ `-32602` at the MCP layer.
            if let Some(id) = t.strip_prefix("flop:").and_then(|r| r.parse::<u32>().ok()) {
                if let Some(f) = flops.iter().find(|f| f.id == id) {
                    flop_provenance.push(flop_provenance_of(f));
                }
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_FLOP_RESET_PROVENANCE.to_string(),
        results: Vec::new(),
        reach_results: Vec::new(),
        flop_provenance,
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Project one [`Flop`] into its [`FlopProvenance`] (enums → stable strings).
fn flop_provenance_of(f: &Flop) -> FlopProvenance {
    let reset_kind = match f.reset_kind {
        ResetKind::None => "none",
        ResetKind::Sync => "sync",
        ResetKind::Async => "async",
    };
    let default_behavior = match f.kind {
        FlopKind::ZeroDefault => "zero",
        FlopKind::QFeedback => "hold",
    };
    let (mux_kind, mux_arms) = match &f.mux {
        FlopMux::None => ("none", 0),
        FlopMux::OneHot(arms) => ("one_hot", arms.len()),
        FlopMux::Encoded { data, .. } => ("encoded", data.len()),
    };
    FlopProvenance {
        flop: f.id,
        width: f.width,
        has_reset: f.reset_kind != ResetKind::None,
        reset_kind: reset_kind.to_string(),
        reset_value: f.reset_val.to_string(),
        default_behavior: default_behavior.to_string(),
        mux_kind: mux_kind.to_string(),
        mux_arms,
        has_d: f.d.is_some(),
    }
}

/// Shared driver: resolve the requested target(s) within `m` and build a cone
/// for each, formatting instance-output leaves through `fmt`.
fn support_cones_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    let mut results = Vec::new();
    match target {
        None => {
            // One cone per output port, in declaration order (deterministic).
            for p in &m.outputs {
                let root = driver_of_port(m, p.id);
                results.push(build_cone(m, p.name.clone(), root, fmt));
            }
        }
        Some(t) => {
            // An explicit, resolvable target yields exactly one cone; an
            // unresolvable target yields none (→ `-32602` at the MCP layer).
            if let Some((canonical, root)) = resolve_target(m, t) {
                results.push(build_cone(m, canonical, root, fmt));
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_OUTPUT_SUPPORT.to_string(),
        results,
        reach_results: Vec::new(),
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// The fan-out reach accumulated for one source: the outputs and flop `D`-cones
/// it lands in. Sorted on emission (`BTreeSet`) ⇒ deterministic bytes.
#[derive(Default)]
struct ReachAccum {
    outputs: BTreeSet<String>,
    flops: BTreeSet<u32>,
}

/// Shared driver for [`module_input_reach`] / [`design_input_reach`]: invert
/// the support relation and emit a [`ReachResult`] per requested source.
///
/// The reach of a source is **defined** as the transpose of the support cones:
/// a source `X` reaches target `T` iff `T`'s [`SupportCone`] lists `X`. So we
/// build every target's cone with the **same** machinery `output_support` uses
/// (no second walker, no re-derived boundary rules) and bucket each target
/// under the sources it depends on. Cost is `O(targets × cone)`, bounded by
/// module size — acceptable for a read-only analysis (a shared reverse-index is
/// a noted future optimization, not first-cut).
fn input_reach_with(
    m: &Module,
    target: Option<&str>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> DerivedAnalysis {
    // 1. Invert: each output / flop-D cone's support feeds the reach of its
    //    sources. Keyed by source string; values accumulate in sorted sets.
    let mut reach: HashMap<String, ReachAccum> = HashMap::new();
    for p in &m.outputs {
        let cone = build_cone(m, p.name.clone(), driver_of_port(m, p.id), fmt);
        for src in cone_support_keys(&cone) {
            reach.entry(src).or_default().outputs.insert(p.name.clone());
        }
    }
    for f in &m.flops {
        let cone = build_cone(m, format!("flop:{}", f.id), f.d, fmt);
        for src in cone_support_keys(&cone) {
            reach.entry(src).or_default().flops.insert(f.id);
        }
    }

    // 2. The canonical, deterministic source universe (so an explicit target is
    //    resolvable iff it is a real source, and `None` is complete).
    let universe = source_universe(m, fmt);

    // 3. Emit. `None` ⇒ one result per source (incl. sources that reach
    //    nothing). An explicit, resolvable source ⇒ exactly one result; an
    //    unresolvable one ⇒ none (→ `-32602` at the MCP layer), exactly mirroring
    //    the `output_support` "unknown target vs known-but-empty" contract.
    let mut reach_results = Vec::new();
    match target {
        None => {
            for src in &universe {
                reach_results.push(make_reach_result(src, reach.get(src)));
            }
        }
        Some(t) => {
            if universe.iter().any(|s| s == t) {
                reach_results.push(make_reach_result(t, reach.get(t)));
            }
        }
    }
    DerivedAnalysis {
        query: QUERY_INPUT_REACH.to_string(),
        results: Vec::new(),
        reach_results,
        flop_provenance: Vec::new(),
        module_reachability: Vec::new(),
        flop_dependencies: Vec::new(),
        memory_provenance: Vec::new(),
        fsm_provenance: Vec::new(),
        node_drivers: Vec::new(),
    }
}

/// Every reach **source** named in a cone's support: input port names, flop
/// `Q`s as `"flop:<id>"`, and child-instance outputs verbatim. (A flop in a
/// cone's `support_flops` is the flop's `Q` — so as a reach source it is keyed
/// `"flop:<id>"`, the same boundary the `output_support` D-cone target uses,
/// with the direction set by the query kind.)
fn cone_support_keys(cone: &SupportCone) -> Vec<String> {
    let mut keys = Vec::with_capacity(
        cone.support_inputs.len() + cone.support_flops.len() + cone.support_instance_outputs.len(),
    );
    keys.extend(cone.support_inputs.iter().cloned());
    keys.extend(cone.support_flops.iter().map(|f| format!("flop:{f}")));
    keys.extend(cone.support_instance_outputs.iter().cloned());
    keys
}

/// The canonical, deterministic source universe of a module: every declared
/// input port (declaration order), then every flop `Q` as `"flop:<id>"`
/// (ascending id), then every child-instance output present in the IR (sorted
/// resolved name). Declared control ports (`clk`/`rst_n`) appear too and simply
/// show empty combinational reach — the honest dual of `output_support`'s
/// "one cone per declared output, even undriven".
fn source_universe(m: &Module, fmt: &dyn Fn(InstanceId, PortId) -> String) -> Vec<String> {
    let mut universe: Vec<String> = m.inputs.iter().map(|p| p.name.clone()).collect();
    let mut flop_ids: Vec<u32> = m.flops.iter().map(|f| f.id).collect();
    flop_ids.sort_unstable();
    universe.extend(flop_ids.into_iter().map(|id| format!("flop:{id}")));
    let mut insts: BTreeSet<String> = BTreeSet::new();
    for node in &m.nodes {
        if let Node::InstanceOutput { instance, port, .. } = node {
            insts.insert(fmt(*instance, *port));
        }
    }
    universe.extend(insts);
    universe
}

/// Build one [`ReachResult`] for `src` from its accumulated reach (or empty).
fn make_reach_result(src: &str, accum: Option<&ReachAccum>) -> ReachResult {
    let (reaches_outputs, reaches_flops): (Vec<String>, Vec<u32>) = match accum {
        Some(a) => (
            a.outputs.iter().cloned().collect(),
            a.flops.iter().copied().collect(),
        ),
        None => (Vec::new(), Vec::new()),
    };
    let fanout_targets = reaches_outputs.len() + reaches_flops.len();
    ReachResult {
        target: src.to_string(),
        reaches_outputs,
        reaches_flops,
        fanout_targets,
    }
}

/// The node driving output port `port`, if the module drives it.
fn driver_of_port(m: &Module, port: PortId) -> Option<NodeId> {
    m.drives
        .iter()
        .find(|(pid, _)| *pid == port)
        .map(|(_, n)| *n)
}

/// Resolve a target string to `(canonical target, root node)`:
/// * `"flop:<id>"` ⇒ that flop's `d` (which may be `None`), if the id exists.
/// * an output **port name** ⇒ its driving node (which may be absent).
///
/// Returns `None` only when the target is genuinely unknown (an unrecognised
/// name, or a `"flop:<id>"` whose id has no flop), so the caller can tell
/// "unknown target" from "known target, empty cone".
fn resolve_target(m: &Module, target: &str) -> Option<(String, Option<NodeId>)> {
    if let Some(rest) = target.strip_prefix("flop:") {
        let id: u32 = rest.parse().ok()?;
        let flop = m.flops.iter().find(|f| f.id == id)?;
        return Some((format!("flop:{id}"), flop.d));
    }
    let p = m.outputs.iter().find(|p| p.name == target)?;
    Some((p.name.clone(), driver_of_port(m, p.id)))
}

/// Build one [`SupportCone`] for `target`, rooted at `root` (the driving node,
/// or `None` for an undriven target ⇒ an empty cone).
fn build_cone(
    m: &Module,
    target: String,
    root: Option<NodeId>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> SupportCone {
    let mut inputs: BTreeSet<String> = BTreeSet::new();
    let mut flops: BTreeSet<u32> = BTreeSet::new();
    let mut insts: BTreeSet<String> = BTreeSet::new();
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    let mut depth_memo: HashMap<NodeId, usize> = HashMap::new();
    let cone_depth = match root {
        Some(r) => visit(
            m,
            r,
            &mut inputs,
            &mut flops,
            &mut insts,
            &mut visited,
            &mut depth_memo,
            fmt,
        ),
        None => 0,
    };
    SupportCone {
        target,
        support_inputs: inputs.into_iter().collect(),
        support_flops: flops.into_iter().collect(),
        support_instance_outputs: insts.into_iter().collect(),
        cone_nodes: visited.len(),
        cone_depth,
    }
}

/// Memoized post-order fan-in DFS. Returns the combinational gate-depth of
/// node `n`; collects support leaves and the visited-node set as a side
/// effect. Memoization (`depth_memo`) makes a shared DAG node cost O(1) on
/// revisit, and leaf inserts are idempotent (`BTreeSet`), so the result is
/// independent of traversal order.
#[allow(clippy::too_many_arguments)]
fn visit(
    m: &Module,
    n: NodeId,
    inputs: &mut BTreeSet<String>,
    flops: &mut BTreeSet<u32>,
    insts: &mut BTreeSet<String>,
    visited: &mut BTreeSet<NodeId>,
    depth_memo: &mut HashMap<NodeId, usize>,
    fmt: &dyn Fn(InstanceId, PortId) -> String,
) -> usize {
    if let Some(&d) = depth_memo.get(&n) {
        return d;
    }
    visited.insert(n);
    // Defensive: a well-formed IR never references a missing node, but the
    // read-mostly introspection surface must not panic on a malformed one.
    let depth = match m.nodes.get(n as usize) {
        None => 0,
        Some(Node::PrimaryInput { port, .. }) => {
            inputs.insert(input_port_name(m, *port));
            0
        }
        // A constant is a leaf but depends on nothing — not a support source.
        Some(Node::Constant { .. }) => 0,
        // Register boundary: record the flop, stop (clock edge breaks the path).
        Some(Node::FlopQ { flop, .. }) => {
            flops.insert(*flop);
            0
        }
        // Opaque registered leaves (default-off): terminate the cone. A future
        // query kind surfaces memory/FSM provenance; see the module docs.
        Some(Node::MemRead { .. }) | Some(Node::FsmOut { .. }) => 0,
        // Instance boundary: record the child output, do not recurse.
        Some(Node::InstanceOutput { instance, port, .. }) => {
            insts.insert(fmt(*instance, *port));
            0
        }
        Some(Node::Gate { operands, .. }) => {
            let operands = operands.clone();
            let mut max_child = 0;
            for op in operands {
                let d = visit(m, op, inputs, flops, insts, visited, depth_memo, fmt);
                max_child = max_child.max(d);
            }
            1 + max_child
        }
    };
    depth_memo.insert(n, depth);
    depth
}

/// The declared name of input `port`, or a `port#<id>` fallback if absent.
fn input_port_name(m: &Module, port: PortId) -> String {
    m.inputs
        .iter()
        .find(|p| p.id == port)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("port#{port}"))
}

/// The declared name of instance `inst`, or an `inst<id>` fallback.
fn instance_name(m: &Module, inst: InstanceId) -> String {
    m.instances
        .iter()
        .find(|i| i.id == inst)
        .map(|i| i.name.clone())
        .unwrap_or_else(|| format!("inst{inst}"))
}

/// Module-only instance-output leaf: `"<instance>.port<id>"` (no child def to
/// resolve the port name against).
fn format_instance_leaf_module(m: &Module, inst: InstanceId, port: PortId) -> String {
    format!("{}.port{}", instance_name(m, inst), port)
}

/// Design instance-output leaf: `"<instance>.<child-output-port-name>"`,
/// resolved via the design's module table; falls back to `"<instance>.port<id>"`
/// when the child def or port is not found.
fn format_instance_leaf_design(
    design: &Design,
    parent: &Module,
    inst: InstanceId,
    port: PortId,
) -> String {
    let name = instance_name(parent, inst);
    if let Some(i) = parent.instances.iter().find(|i| i.id == inst) {
        if let Some(child) = design.modules.iter().find(|c| c.name == i.module) {
            if let Some(p) = child.outputs.iter().find(|p| p.id == port) {
                return format!("{name}.{}", p.name);
            }
        }
    }
    format!("{name}.port{port}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::types::{
        Direction, Flop, FlopKind, FlopMux, Instance, InstanceRole, MuxArm, ResetKind,
    };
    use crate::ir::{Design, Module, Node, Port};

    fn port(id: u32, name: &str, width: u32, dir: Direction) -> Port {
        Port {
            id,
            name: name.into(),
            width,
            dir,
        }
    }

    /// `y = (a & b) | c`. Exact combinational support + counts + depth.
    #[test]
    fn combinational_support_cone_is_exact() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.inputs.push(port(1, "b", 8, Direction::In));
        m.inputs.push(port(2, "c", 8, Direction::In));
        m.outputs.push(port(3, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 }); // 1 = b
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 2 = c
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::And,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3 = a & b
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Or,
            operands: vec![3, 2],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 4 = (a&b) | c
        m.drives.push((3, 4)); // output port 3 (y) <- node 4

        let analysis = module_support_cones(&m, Some("y"));
        assert_eq!(analysis.query, QUERY_OUTPUT_SUPPORT);
        assert_eq!(analysis.results.len(), 1);
        let cone = &analysis.results[0];
        assert_eq!(cone.target, "y");
        assert_eq!(cone.support_inputs, vec!["a", "b", "c"]); // sorted
        assert!(cone.support_flops.is_empty());
        assert!(cone.support_instance_outputs.is_empty());
        assert_eq!(cone.cone_nodes, 5); // a,b,c,and,or
        assert_eq!(cone.cone_depth, 2); // or -> and -> input
    }

    /// A `FlopQ` is a register-boundary support leaf: it is recorded in
    /// `support_flops` and the walk does NOT cross it, so an input that only
    /// feeds the flop's D side is absent from the output's cone — but is
    /// present in the `"flop:<id>"` cone.
    #[test]
    fn flop_q_is_a_boundary_leaf_not_recursed_through() {
        let mut m = Module {
            name: "seq".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "a", 8, Direction::In)); // feeds output directly
        m.inputs.push(port(3, "b", 8, Direction::In)); // feeds the flop D only
        m.outputs.push(port(4, "y", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 3, width: 8 }); // 1 = b (D side)
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 2 = Q of flop 0
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 2], // a ^ Q
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(1), // D = b
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((4, 3)); // y <- a ^ Q

        // Output cone: stops at the flop; `b` (D-only) is absent.
        let out = module_support_cones(&m, Some("y"));
        let cone = &out.results[0];
        assert_eq!(cone.support_inputs, vec!["a"]);
        assert_eq!(cone.support_flops, vec![0]);
        assert_eq!(cone.cone_nodes, 3); // a, Q, xor
        assert_eq!(cone.cone_depth, 1);

        // Flop D cone: the combinational cone feeding the flop's D = just `b`.
        let dcone = module_support_cones(&m, Some("flop:0"));
        assert_eq!(dcone.results.len(), 1);
        let d = &dcone.results[0];
        assert_eq!(d.target, "flop:0");
        assert_eq!(d.support_inputs, vec!["b"]);
        assert!(d.support_flops.is_empty());
        assert_eq!(d.cone_nodes, 1);
        assert_eq!(d.cone_depth, 0); // D is directly a primary input
    }

    /// A `Constant` operand is counted in `cone_nodes` but is not a support
    /// source.
    #[test]
    fn constant_is_counted_but_not_a_support_source() {
        let mut m = Module {
            name: "k".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::Constant {
            width: 8,
            value: 0xFF,
        }); // 1 = const
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::And,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 2 = a & 0xFF
        m.drives.push((1, 2));

        let cone = &module_support_cones(&m, Some("y")).results[0];
        assert_eq!(cone.support_inputs, vec!["a"]);
        assert_eq!(cone.cone_nodes, 3); // a, const, and
        assert_eq!(cone.cone_depth, 1);
    }

    /// `MemRead`/`FsmOut` are opaque registered leaves: they terminate the
    /// cone (counted, recorded in no support list — the documented boundary).
    #[test]
    fn opaque_mem_read_terminates_the_cone() {
        let mut m = Module {
            name: "mem".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::MemRead { mem: 0, width: 8 }); // 1 = opaque mem read
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 2 = a ^ memread
        m.drives.push((1, 2));

        let cone = &module_support_cones(&m, Some("y")).results[0];
        assert_eq!(cone.support_inputs, vec!["a"]);
        assert!(cone.support_flops.is_empty());
        assert!(cone.support_instance_outputs.is_empty());
        assert_eq!(cone.cone_nodes, 3); // a, memread, xor (memread counted)
    }

    /// `target = None` ⇒ one cone per output, in declaration order.
    #[test]
    fn absent_target_yields_one_cone_per_output() {
        let mut m = Module {
            name: "two".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.inputs.push(port(1, "b", 8, Direction::In));
        m.outputs.push(port(2, "y0", 8, Direction::Out));
        m.outputs.push(port(3, "y1", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 }); // 1 = b
        m.drives.push((2, 0)); // y0 <- a
        m.drives.push((3, 1)); // y1 <- b

        let analysis = module_support_cones(&m, None);
        assert_eq!(analysis.results.len(), 2);
        assert_eq!(analysis.results[0].target, "y0");
        assert_eq!(analysis.results[0].support_inputs, vec!["a"]);
        assert_eq!(analysis.results[1].target, "y1");
        assert_eq!(analysis.results[1].support_inputs, vec!["b"]);
    }

    /// An unknown target (or an out-of-range `flop:<id>`) resolves to no cone;
    /// a *known* target with an empty cone still yields one cone.
    #[test]
    fn unknown_target_yields_no_cone() {
        let mut m = Module {
            name: "u".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));

        assert!(module_support_cones(&m, Some("nope")).results.is_empty());
        assert!(module_support_cones(&m, Some("flop:0")).results.is_empty());
        // Known output, but undriven ⇒ one (empty) cone, not zero.
        let mut undriven = m.clone();
        undriven.drives.clear();
        let r = module_support_cones(&undriven, Some("y"));
        assert_eq!(r.results.len(), 1);
        assert!(r.results[0].support_inputs.is_empty());
        assert_eq!(r.results[0].cone_nodes, 0);
        assert_eq!(r.results[0].cone_depth, 0);
    }

    /// A design's child-instance output is a named support leaf: the cone
    /// stops at the instance boundary and resolves `"<instance>.<port-name>"`.
    #[test]
    fn design_resolves_child_instance_output_port_name() {
        // Child: out port "o" driven by input "a".
        let mut child = Module {
            name: "child".into(),
            ..Module::default()
        };
        child.inputs.push(port(0, "a", 8, Direction::In));
        child.outputs.push(port(1, "o", 8, Direction::Out));
        child.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        child.drives.push((1, 0));

        // Top: instantiates child as "u0"; top output "y" <- u0.o ^ top input p.
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(port(0, "p", 8, Direction::In));
        top.outputs.push(port(1, "y", 8, Direction::Out));
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "child".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)], // child port 0 <- top node 0 (set below)
            param_bindings: vec![],
        });
        top.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = p
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1, // child's output port id ("o")
            width: 8,
        }); // 1 = u0.o
        top.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 2 = p ^ u0.o
        top.drives.push((1, 2));

        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };
        let cone = &design_support_cones(&design, Some("y")).results[0];
        assert_eq!(cone.support_inputs, vec!["p"]);
        assert_eq!(cone.support_instance_outputs, vec!["u0.o"]); // resolved name
        assert!(cone.support_flops.is_empty());
        assert_eq!(cone.cone_nodes, 3); // p, u0.o, xor
        assert_eq!(cone.cone_depth, 1);
    }

    /// The analysis is byte-stable: identical inputs ⇒ identical JSON, and the
    /// support vectors are sorted.
    #[test]
    fn analysis_is_deterministic_and_sorted() {
        let mut m = Module {
            name: "det".into(),
            ..Module::default()
        };
        // Inputs deliberately declared out of alphabetical order.
        m.inputs.push(port(0, "zebra", 8, Direction::In));
        m.inputs.push(port(1, "alpha", 8, Direction::In));
        m.inputs.push(port(2, "mike", 8, Direction::In));
        m.outputs.push(port(3, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 });
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 1, 2],
            width: 8,
            deps: crate::ir::DepSet::new(),
        });
        m.drives.push((3, 3));

        let a = module_support_cones(&m, None);
        let b = module_support_cones(&m, None);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert_eq!(a.results[0].support_inputs, vec!["alpha", "mike", "zebra"]);
    }

    /// Shared DAG nodes are counted once (memoization), and a re-converging
    /// fan-in does not double-count or change depth.
    #[test]
    fn shared_fanin_is_counted_once() {
        let mut m = Module {
            name: "share".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Not,
            operands: vec![0],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 1 = ~a
            // y = (~a) ^ (~a): both operands share node 1.
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![1, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 2
        m.drives.push((1, 2));

        let cone = &module_support_cones(&m, Some("y")).results[0];
        assert_eq!(cone.support_inputs, vec!["a"]);
        assert_eq!(cone.cone_nodes, 3); // a, ~a, xor — node 1 counted once
        assert_eq!(cone.cone_depth, 2); // xor -> not -> input
    }

    /// `y = (a & b) | c`. `input_reach` is the exact transpose of
    /// `output_support`: each input reaches `y`, and `X ∈ support(Y) ⇔ Y ∈
    /// reach(X)`.
    #[test]
    fn input_reach_is_the_transpose_of_output_support() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.inputs.push(port(1, "b", 8, Direction::In));
        m.inputs.push(port(2, "c", 8, Direction::In));
        m.outputs.push(port(3, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 }); // 1 = b
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 2 = c
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::And,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3 = a & b
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Or,
            operands: vec![3, 2],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 4 = (a&b) | c
        m.drives.push((3, 4));

        // Each input reaches exactly `y`, no flop.
        for name in ["a", "b", "c"] {
            let r = module_input_reach(&m, Some(name));
            assert_eq!(r.query, QUERY_INPUT_REACH);
            assert!(r.results.is_empty()); // input_reach populates reach_results, not results
            assert_eq!(r.reach_results.len(), 1);
            let rr = &r.reach_results[0];
            assert_eq!(rr.target, name);
            assert_eq!(rr.reaches_outputs, vec!["y"]);
            assert!(rr.reaches_flops.is_empty());
            assert_eq!(rr.fanout_targets, 1);
        }

        // Transpose property over the support cone of `y`.
        let support = module_support_cones(&m, Some("y")).results[0]
            .support_inputs
            .clone();
        for name in ["a", "b", "c"] {
            let reaches_y = module_input_reach(&m, Some(name)).reach_results[0]
                .reaches_outputs
                .contains(&"y".to_string());
            assert_eq!(support.contains(&name.to_string()), reaches_y);
        }
    }

    /// A flop `Q` as a reach source fans out to the outputs it drives; an input
    /// that only feeds a flop `D` reaches that flop (not the output past it).
    /// The exact dual of `flop_q_is_a_boundary_leaf_not_recursed_through`.
    #[test]
    fn flop_q_source_and_flop_d_side_source_are_duals() {
        let mut m = Module {
            name: "seq".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "a", 8, Direction::In)); // feeds output directly
        m.inputs.push(port(3, "b", 8, Direction::In)); // feeds the flop D only
        m.outputs.push(port(4, "y", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 3, width: 8 }); // 1 = b (D side)
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 2 = Q of flop 0
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 2], // a ^ Q
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(1), // D = b
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((4, 3)); // y <- a ^ Q

        // `a` reaches `y` only.
        let a = &module_input_reach(&m, Some("a")).reach_results[0];
        assert_eq!(a.reaches_outputs, vec!["y"]);
        assert!(a.reaches_flops.is_empty());
        assert_eq!(a.fanout_targets, 1);

        // `flop:0` is the Q as a source: it reaches `y`, no flop D-cone.
        let q = &module_input_reach(&m, Some("flop:0")).reach_results[0];
        assert_eq!(q.target, "flop:0");
        assert_eq!(q.reaches_outputs, vec!["y"]);
        assert!(q.reaches_flops.is_empty());
        assert_eq!(q.fanout_targets, 1);

        // `b` feeds the flop D only: it reaches flop 0, not `y` (the cone stops
        // at the register boundary).
        let b = &module_input_reach(&m, Some("b")).reach_results[0];
        assert!(b.reaches_outputs.is_empty());
        assert_eq!(b.reaches_flops, vec![0]);
        assert_eq!(b.fanout_targets, 1);

        // `target = None` ⇒ one result per source, in canonical order, and the
        // control ports (clk/rst_n) show empty combinational reach.
        let all = module_input_reach(&m, None);
        let targets: Vec<&str> = all
            .reach_results
            .iter()
            .map(|r| r.target.as_str())
            .collect();
        assert_eq!(targets, vec!["clk", "rst_n", "a", "b", "flop:0"]);
        let clk = &all.reach_results[0];
        assert_eq!(clk.target, "clk");
        assert!(clk.reaches_outputs.is_empty() && clk.reaches_flops.is_empty());
        assert_eq!(clk.fanout_targets, 0);
    }

    /// An unknown reach source (or an out-of-range `flop:<id>`) yields no
    /// result (→ `-32602` at the MCP layer); a *known* source that reaches
    /// nothing still yields exactly one (empty) result.
    #[test]
    fn unknown_reach_source_yields_no_result() {
        let mut m = Module {
            name: "u".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));

        assert!(module_input_reach(&m, Some("nope"))
            .reach_results
            .is_empty());
        assert!(module_input_reach(&m, Some("flop:0"))
            .reach_results
            .is_empty()); // no flop 0
        let a = module_input_reach(&m, Some("a"));
        assert_eq!(a.reach_results.len(), 1);
        assert_eq!(a.reach_results[0].reaches_outputs, vec!["y"]);
    }

    /// In a design, a child-instance output is a first-class reach **source**:
    /// the dual of `design_resolves_child_instance_output_port_name`.
    #[test]
    fn design_instance_output_is_a_reach_source() {
        let mut child = Module {
            name: "child".into(),
            ..Module::default()
        };
        child.inputs.push(port(0, "a", 8, Direction::In));
        child.outputs.push(port(1, "o", 8, Direction::Out));
        child.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        child.drives.push((1, 0));

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(port(0, "p", 8, Direction::In));
        top.outputs.push(port(1, "y", 8, Direction::Out));
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "child".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![(0, 0)],
            param_bindings: vec![],
        });
        top.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = p
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1,
            width: 8,
        }); // 1 = u0.o
        top.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 2 = p ^ u0.o
        top.drives.push((1, 2));

        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };

        let io = design_input_reach(&design, Some("u0.o"));
        assert_eq!(io.reach_results.len(), 1);
        assert_eq!(io.reach_results[0].target, "u0.o");
        assert_eq!(io.reach_results[0].reaches_outputs, vec!["y"]);
        assert_eq!(
            design_input_reach(&design, Some("p")).reach_results[0].reaches_outputs,
            vec!["y"]
        );
        // `None` enumerates the input then the instance output.
        let all = design_input_reach(&design, None);
        let targets: Vec<&str> = all
            .reach_results
            .iter()
            .map(|r| r.target.as_str())
            .collect();
        assert_eq!(targets, vec!["p", "u0.o"]);
    }

    /// `input_reach` is byte-stable and its reach vectors are sorted.
    #[test]
    fn input_reach_is_deterministic_and_sorted() {
        let mut m = Module {
            name: "det".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        // Outputs deliberately declared out of alphabetical order; `a` drives both.
        m.outputs.push(port(1, "zebra", 8, Direction::Out));
        m.outputs.push(port(2, "alpha", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.drives.push((1, 0)); // zebra <- a
        m.drives.push((2, 0)); // alpha <- a

        let rr = &module_input_reach(&m, Some("a")).reach_results[0];
        assert_eq!(rr.reaches_outputs, vec!["alpha", "zebra"]); // sorted
        assert_eq!(rr.fanout_targets, 2);

        let a = module_input_reach(&m, None);
        let b = module_input_reach(&m, None);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }

    /// The byte-identical guarantee: an `output_support` analysis serializes
    /// **without** a `reach_results` key (`skip_serializing_if`), so existing
    /// `output_support` documents are unchanged; an `input_reach` analysis
    /// carries `reach_results` and an empty `results: []`.
    #[test]
    fn output_support_document_omits_reach_results_key() {
        let mut m = Module {
            name: "k".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));

        let support = serde_json::to_value(module_support_cones(&m, None)).unwrap();
        assert!(support.as_object().unwrap().get("reach_results").is_none());

        let reach = serde_json::to_value(module_input_reach(&m, None)).unwrap();
        assert!(reach.as_object().unwrap().contains_key("reach_results"));
        assert_eq!(reach["results"].as_array().unwrap().len(), 0);
    }

    /// `flop_reset_provenance` projects each flop's fields exactly, maps every
    /// enum to its stable string, and emits flops in ascending id order.
    #[test]
    fn flop_provenance_projects_each_flop_field() {
        let mut m = Module {
            name: "fp".into(),
            ..Module::default()
        };
        // flop 1 pushed before flop 0 to prove ascending-id ordering.
        m.flops.push(Flop {
            id: 1,
            width: 4,
            d: None, // ⇒ has_d false
            q: 0,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(0),
            q: 1,
            reset_val: 5,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::OneHot(vec![MuxArm { data: 0, sel: 1 }, MuxArm { data: 0, sel: 1 }]),
        });

        let a = module_flop_provenance(&m, None);
        assert_eq!(a.query, QUERY_FLOP_RESET_PROVENANCE);
        assert!(a.results.is_empty() && a.reach_results.is_empty());
        assert_eq!(a.flop_provenance.len(), 2);

        let f0 = &a.flop_provenance[0]; // ascending id ⇒ flop 0 first
        assert_eq!(f0.flop, 0);
        assert_eq!(f0.width, 8);
        assert!(f0.has_reset);
        assert_eq!(f0.reset_kind, "async");
        assert_eq!(f0.reset_value, "5");
        assert_eq!(f0.default_behavior, "zero");
        assert_eq!(f0.mux_kind, "one_hot");
        assert_eq!(f0.mux_arms, 2);
        assert!(f0.has_d);

        let f1 = &a.flop_provenance[1];
        assert_eq!(f1.flop, 1);
        assert!(!f1.has_reset);
        assert_eq!(f1.reset_kind, "none");
        assert_eq!(f1.default_behavior, "hold"); // QFeedback
        assert_eq!(f1.mux_kind, "none");
        assert_eq!(f1.mux_arms, 0);
        assert!(!f1.has_d);
    }

    /// `target = "flop:<id>"` addresses one flop; an unknown target (bad id or
    /// non-flop string) yields no result (→ `-32602` at the MCP layer). Also
    /// covers the `sync` reset kind and the `encoded` mux arm count.
    #[test]
    fn flop_provenance_target_and_unknown_target() {
        let mut m = Module {
            name: "fp2".into(),
            ..Module::default()
        };
        m.flops.push(Flop {
            id: 3,
            width: 1,
            d: Some(0),
            q: 1,
            reset_val: 1,
            reset_kind: ResetKind::Sync,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::Encoded {
                sel: 0,
                data: vec![0, 0, 0],
            },
        });

        let one = module_flop_provenance(&m, Some("flop:3"));
        assert_eq!(one.flop_provenance.len(), 1);
        let f = &one.flop_provenance[0];
        assert_eq!(f.flop, 3);
        assert_eq!(f.reset_kind, "sync");
        assert_eq!(f.reset_value, "1");
        assert_eq!(f.mux_kind, "encoded");
        assert_eq!(f.mux_arms, 3); // data.len()

        assert!(module_flop_provenance(&m, Some("flop:9"))
            .flop_provenance
            .is_empty());
        assert!(module_flop_provenance(&m, Some("nope"))
            .flop_provenance
            .is_empty());
    }

    /// A flopless module + `target = None` yields an empty (not errored)
    /// provenance — the honest "no flops" answer.
    #[test]
    fn flopless_module_yields_empty_provenance() {
        let m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        let a = module_flop_provenance(&m, None);
        assert_eq!(a.query, QUERY_FLOP_RESET_PROVENANCE);
        assert!(a.flop_provenance.is_empty());
    }

    /// A `flop_reset_provenance` analysis serializes `flop_provenance` and omits
    /// `reach_results` (`skip_serializing_if`), keeping the other queries'
    /// documents byte-identical.
    #[test]
    fn flop_provenance_document_omits_the_other_query_vecs() {
        let mut m = Module {
            name: "fp3".into(),
            ..Module::default()
        };
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: None,
            q: 0,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        let v = serde_json::to_value(module_flop_provenance(&m, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("flop_provenance"));
        assert!(obj.get("reach_results").is_none()); // skip_serializing_if
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty
    }

    /// The design variant projects the **top** module's flops.
    #[test]
    fn design_flop_provenance_projects_the_top_module() {
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.flops.push(Flop {
            id: 0,
            width: 2,
            d: Some(0),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });
        let child = Module {
            name: "child".into(),
            ..Module::default()
        };
        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };
        let a = design_flop_provenance(&design, None);
        assert_eq!(a.flop_provenance.len(), 1);
        assert_eq!(a.flop_provenance[0].flop, 0);
        assert_eq!(a.flop_provenance[0].default_behavior, "hold");
        assert_eq!(a.flop_provenance[0].reset_kind, "async");
    }

    // --- module_reachability (`SEMANTIC-INTROSPECTION-EXPANSION.5b.1`) ---

    /// A `PlannedChild` instance named `name` of child module `module`.
    fn inst(id: u32, name: &str, module: &str) -> Instance {
        Instance {
            id,
            name: name.into(),
            module: module.into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![],
            param_bindings: vec![],
        }
    }

    /// A bare module with the given name and instance list.
    fn mod_with(name: &str, instances: Vec<Instance>) -> Module {
        Module {
            name: name.into(),
            instances,
            ..Module::default()
        }
    }

    /// BFS reachability over a multi-level design: min depth, sorted output,
    /// distinct/sorted `instantiates`, multi-instance `instance_count`, and an
    /// unreachable (orphan) module with no depth.
    #[test]
    fn design_module_reachability_bfs_depth_and_edges() {
        // top -> a (x2), b ; a -> c ; orphan unreachable.
        let top = mod_with(
            "top",
            vec![
                inst(0, "u_a0", "a"),
                inst(1, "u_a1", "a"), // a instantiated twice
                inst(2, "u_b", "b"),
            ],
        );
        let a = mod_with("a", vec![inst(0, "u_c", "c")]);
        let b = mod_with("b", vec![]);
        let c = mod_with("c", vec![]);
        let orphan = mod_with("orphan", vec![]);
        // modules pushed deliberately out of alphabetical order.
        let design = Design {
            top: "top".into(),
            modules: vec![top, c, orphan, b, a],
        };

        let an = design_module_reachability(&design, None);
        assert_eq!(an.query, QUERY_MODULE_REACHABILITY);
        // Only the module_reachability vec is populated.
        assert!(
            an.results.is_empty() && an.reach_results.is_empty() && an.flop_provenance.is_empty()
        );
        // One entry per module, sorted by name.
        let names: Vec<&str> = an
            .module_reachability
            .iter()
            .map(|r| r.module.as_str())
            .collect();
        assert_eq!(names, vec!["a", "b", "c", "orphan", "top"]);

        let by = |n: &str| {
            an.module_reachability
                .iter()
                .find(|r| r.module == n)
                .unwrap()
        };
        // top: reachable, depth 0, instantiates {a,b} (sorted/deduped), 3 instances.
        let t = by("top");
        assert!(t.reachable);
        assert_eq!(t.depth, Some(0));
        assert_eq!(t.instantiates, vec!["a", "b"]);
        assert_eq!(t.instance_count, 3); // a, a, b — count > distinct
                                         // a: depth 1, instantiates {c}.
        let ra = by("a");
        assert!(ra.reachable);
        assert_eq!(ra.depth, Some(1));
        assert_eq!(ra.instantiates, vec!["c"]);
        assert_eq!(ra.instance_count, 1);
        // b: depth 1, no children.
        let rb = by("b");
        assert_eq!(rb.depth, Some(1));
        assert!(rb.instantiates.is_empty());
        assert_eq!(rb.instance_count, 0);
        // c: depth 2 (top -> a -> c).
        let rc = by("c");
        assert!(rc.reachable);
        assert_eq!(rc.depth, Some(2));
        // orphan: unreachable, no depth.
        let ro = by("orphan");
        assert!(!ro.reachable);
        assert_eq!(ro.depth, None);
        assert!(ro.instantiates.is_empty());
    }

    /// An explicit module-name target yields that one entry; an unknown name
    /// yields none (→ `-32602` at the MCP layer).
    #[test]
    fn design_module_reachability_target_and_unknown() {
        let top = mod_with("top", vec![inst(0, "u_a", "a")]);
        let a = mod_with("a", vec![]);
        let design = Design {
            top: "top".into(),
            modules: vec![top, a],
        };

        let one = design_module_reachability(&design, Some("a"));
        assert_eq!(one.module_reachability.len(), 1);
        assert_eq!(one.module_reachability[0].module, "a");
        assert_eq!(one.module_reachability[0].depth, Some(1));

        assert!(design_module_reachability(&design, Some("nope"))
            .module_reachability
            .is_empty());
    }

    /// The module variant is the degenerate one-node case: one entry for the
    /// module itself (reachable, depth 0, its own instantiated children).
    #[test]
    fn module_module_reachability_is_a_degenerate_one_node() {
        let m = mod_with(
            "solo",
            vec![
                inst(0, "u_x0", "x"),
                inst(1, "u_x1", "x"), // x twice
                inst(2, "u_y", "y"),
            ],
        );
        let an = module_module_reachability(&m, None);
        assert_eq!(an.query, QUERY_MODULE_REACHABILITY);
        assert_eq!(an.module_reachability.len(), 1);
        let e = &an.module_reachability[0];
        assert_eq!(e.module, "solo");
        assert!(e.reachable);
        assert_eq!(e.depth, Some(0));
        assert_eq!(e.instantiates, vec!["x", "y"]); // sorted, deduped
        assert_eq!(e.instance_count, 3);

        // target = the module itself ⇒ the entry; any other ⇒ none.
        assert_eq!(
            module_module_reachability(&m, Some("solo"))
                .module_reachability
                .len(),
            1
        );
        assert!(module_module_reachability(&m, Some("x"))
            .module_reachability
            .is_empty());
    }

    /// A `module_reachability` document serializes `module_reachability` (and the
    /// always-present empty `results`), omits the other two query vecs, and omits
    /// `depth` only on an unreachable module.
    #[test]
    fn module_reachability_document_omits_the_other_query_vecs() {
        let design = Design {
            top: "top".into(),
            modules: vec![mod_with("top", vec![]), mod_with("orphan", vec![])],
        };
        let v = serde_json::to_value(design_module_reachability(&design, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("module_reachability"));
        assert!(obj.get("reach_results").is_none()); // skip_serializing_if
        assert!(obj.get("flop_provenance").is_none()); // skip_serializing_if
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty

        let entries = v["module_reachability"].as_array().unwrap();
        let top_e = entries.iter().find(|e| e["module"] == "top").unwrap();
        assert_eq!(top_e["reachable"], true);
        assert_eq!(top_e["depth"], 0); // reachable ⇒ depth present
        let orphan_e = entries.iter().find(|e| e["module"] == "orphan").unwrap();
        assert_eq!(orphan_e["reachable"], false);
        assert!(orphan_e.as_object().unwrap().get("depth").is_none()); // omitted
    }

    /// `module_reachability` is byte-stable, the output is sorted by module name,
    /// and `instantiates` is sorted within each entry.
    #[test]
    fn module_reachability_is_deterministic_and_sorted() {
        // Children + modules deliberately declared out of alphabetical order.
        let top = mod_with(
            "top",
            vec![
                inst(0, "u_z", "zebra"),
                inst(1, "u_a", "alpha"),
                inst(2, "u_m", "mike"),
            ],
        );
        let design = Design {
            top: "top".into(),
            modules: vec![
                mod_with("mike", vec![]),
                top,
                mod_with("zebra", vec![]),
                mod_with("alpha", vec![]),
            ],
        };
        let a = design_module_reachability(&design, None);
        let b = design_module_reachability(&design, None);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        let names: Vec<&str> = a
            .module_reachability
            .iter()
            .map(|r| r.module.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "mike", "top", "zebra"]);
        let top_e = a
            .module_reachability
            .iter()
            .find(|r| r.module == "top")
            .unwrap();
        assert_eq!(top_e.instantiates, vec!["alpha", "mike", "zebra"]);
    }

    /// Defensive: a malformed design whose `top` is absent from the module table
    /// still enumerates every present module (all `reachable: false`), with
    /// `instantiates` as the honest local out-edge fact.
    #[test]
    fn design_module_reachability_absent_top_reports_all_unreachable() {
        let design = Design {
            top: "ghost".into(),
            modules: vec![
                mod_with("a", vec![inst(0, "u_b", "b")]),
                mod_with("b", vec![]),
            ],
        };
        let an = design_module_reachability(&design, None);
        assert_eq!(an.module_reachability.len(), 2); // still enumerates present modules
        for e in &an.module_reachability {
            assert!(!e.reachable);
            assert_eq!(e.depth, None);
        }
        let a = an
            .module_reachability
            .iter()
            .find(|r| r.module == "a")
            .unwrap();
        assert_eq!(a.instantiates, vec!["b"]);
    }

    // --- flop_dependencies (`SEMANTIC-INTROSPECTION-EXPANSION.6b.1`) ---

    /// A module with a 2-stage register pipeline (`b → [f0] → [f1] → y`) plus a
    /// self-feedback counter (`[f2] = f2 ^ 1`). Exercises predecessors,
    /// successors (the transpose), and `self_dependent` in one shape.
    fn seq_pipeline_counter() -> Module {
        let mut m = Module {
            name: "seq".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "b", 8, Direction::In));
        m.outputs.push(port(3, "y", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 0 = b
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 1 = Q0
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 2 = Q1
        m.nodes.push(Node::FlopQ { flop: 2, width: 8 }); // 3 = Q2
        m.nodes.push(Node::Constant { width: 8, value: 1 }); // 4 = 1
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![3, 4], // Q2 ^ 1 — Q2 feeds its own D
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 5
        let mk = |id, d, q| Flop {
            id,
            width: 8,
            d: Some(d),
            q,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        };
        m.flops.push(mk(0, 0, 1)); // D = b
        m.flops.push(mk(1, 1, 2)); // D = Q0
        m.flops.push(mk(2, 5, 3)); // D = Q2 ^ 1 (self-feedback)
        m.drives.push((3, 2)); // y <- Q1
        m
    }

    /// Exact predecessors / successors / `self_dependent`, ascending-id order.
    #[test]
    fn flop_dependencies_pipeline_and_self_feedback() {
        let m = seq_pipeline_counter();
        let a = module_flop_dependencies(&m, None);
        assert_eq!(a.query, QUERY_FLOP_DEPENDENCIES);
        assert_eq!(a.flop_dependencies.len(), 3);
        let ids: Vec<u32> = a.flop_dependencies.iter().map(|e| e.flop).collect();
        assert_eq!(ids, vec![0, 1, 2]); // ascending id

        let f0 = &a.flop_dependencies[0];
        assert!(f0.depends_on_flops.is_empty()); // D = b (a primary input)
        assert_eq!(f0.driven_flops, vec![1]); // Q0 feeds flop 1's D
        assert!(!f0.self_dependent);

        let f1 = &a.flop_dependencies[1];
        assert_eq!(f1.depends_on_flops, vec![0]); // D = Q0
        assert!(f1.driven_flops.is_empty());
        assert!(!f1.self_dependent);

        let f2 = &a.flop_dependencies[2];
        assert_eq!(f2.depends_on_flops, vec![2]); // D = Q2 ^ 1
        assert_eq!(f2.driven_flops, vec![2]);
        assert!(f2.self_dependent); // a self-feedback register
    }

    /// `B ∈ depends_on(A)` ⇔ `A ∈ driven(B)` — predecessors and successors are
    /// exact transposes, so the two directions cannot drift.
    #[test]
    fn flop_dependencies_predecessors_and_successors_are_transposes() {
        let m = seq_pipeline_counter();
        let a = module_flop_dependencies(&m, None);
        let by_id: std::collections::HashMap<u32, &FlopDependencies> =
            a.flop_dependencies.iter().map(|e| (e.flop, e)).collect();
        for e in &a.flop_dependencies {
            for &p in &e.depends_on_flops {
                assert!(
                    by_id[&p].driven_flops.contains(&e.flop),
                    "flop {} depends on {p} but {p} does not drive {0}",
                    e.flop
                );
            }
            for &s in &e.driven_flops {
                assert!(by_id[&s].depends_on_flops.contains(&e.flop));
            }
        }
    }

    /// `"flop:<id>"` addressing: a resolvable flop ⇒ exactly one entry (its
    /// successors still correct because the whole graph is built before
    /// filtering); a known flop with no predecessor ⇒ an empty-edges entry;
    /// unknown / out-of-range / malformed ⇒ no entry (→ `-32602`).
    #[test]
    fn flop_dependencies_target_and_unknown() {
        let m = seq_pipeline_counter();
        let one = module_flop_dependencies(&m, Some("flop:2"));
        assert_eq!(one.flop_dependencies.len(), 1);
        assert_eq!(one.flop_dependencies[0].flop, 2);
        assert!(one.flop_dependencies[0].self_dependent);
        assert_eq!(one.flop_dependencies[0].driven_flops, vec![2]);

        let f0 = module_flop_dependencies(&m, Some("flop:0"));
        assert_eq!(f0.flop_dependencies.len(), 1);
        assert!(f0.flop_dependencies[0].depends_on_flops.is_empty());

        assert!(module_flop_dependencies(&m, Some("flop:99"))
            .flop_dependencies
            .is_empty());
        assert!(module_flop_dependencies(&m, Some("nope"))
            .flop_dependencies
            .is_empty());
        assert!(module_flop_dependencies(&m, Some("flop:abc"))
            .flop_dependencies
            .is_empty());
    }

    /// A flopless module ⇒ an empty `flop_dependencies` (the honest "no flops"
    /// answer, not an error).
    #[test]
    fn flopless_module_yields_empty_flop_dependencies() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));
        assert!(module_flop_dependencies(&m, None)
            .flop_dependencies
            .is_empty());
    }

    /// A `flop_dependencies` document omits the other four query vecs
    /// (`skip_serializing_if`), and an `output_support` document omits
    /// `flop_dependencies` ⇒ the prior documents stay byte-identical.
    #[test]
    fn flop_dependencies_document_omits_the_other_query_vecs() {
        let m = seq_pipeline_counter();
        let v = serde_json::to_value(module_flop_dependencies(&m, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("flop_dependencies"));
        assert!(obj.get("reach_results").is_none());
        assert!(obj.get("flop_provenance").is_none());
        assert!(obj.get("module_reachability").is_none());
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty

        let sup = serde_json::to_value(module_support_cones(&m, None)).unwrap();
        assert!(sup.as_object().unwrap().get("flop_dependencies").is_none());
    }

    /// The design variant projects the **top** module's register graph; an
    /// absent top ⇒ an empty analysis.
    #[test]
    fn design_flop_dependencies_projects_the_top_module() {
        let top = seq_pipeline_counter();
        let child = Module {
            name: "child".into(),
            ..Module::default()
        };
        let design = Design {
            top: "seq".into(),
            modules: vec![top, child],
        };
        let a = design_flop_dependencies(&design, None);
        assert_eq!(a.flop_dependencies.len(), 3);
        assert_eq!(a.flop_dependencies[2].driven_flops, vec![2]);

        let ghost = Design {
            top: "ghost".into(),
            modules: vec![Module {
                name: "child".into(),
                ..Module::default()
            }],
        };
        assert!(design_flop_dependencies(&ghost, None)
            .flop_dependencies
            .is_empty());
    }

    // --- memory_provenance (`SEMANTIC-INTROSPECTION-EXPANSION.7b.1`) ---

    /// A `SimpleDualPort` memory whose four ports are driven by distinct sources:
    /// `raddr <- ra`, `waddr <- wa`, `wdata <- wd ^ Q0` (an input + a flop `Q`),
    /// `we <- const`. Exercises an exact per-port support cone incl. a flop-`Q`
    /// support and an empty (constant) cone, the structural fields, and the port
    /// `target` labels.
    fn dual_port_mem_module() -> Module {
        let mut m = Module {
            name: "mem".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "ra", 4, Direction::In));
        m.inputs.push(port(3, "wa", 4, Direction::In));
        m.inputs.push(port(4, "wd", 8, Direction::In));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 4 }); // 0 = ra
        m.nodes.push(Node::PrimaryInput { port: 3, width: 4 }); // 1 = wa
        m.nodes.push(Node::PrimaryInput { port: 4, width: 8 }); // 2 = wd
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 3 = Q0
        m.nodes.push(Node::Constant { width: 1, value: 1 }); // 4 = we const
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![2, 3], // wd ^ Q0
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 5 = wdata cone
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(2), // D = wd (so the flop is well-formed; not under test here)
            q: 3,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.memories.push(Memory {
            id: 0,
            addr_width: 4,
            data_width: 8,
            kind: MemKind::SimpleDualPort,
            we: 4,    // constant ⇒ empty support
            waddr: 1, // wa
            wdata: 5, // wd ^ Q0
            raddr: 0, // ra
        });
        m
    }

    /// Each port's support cone is exact (incl. a flop-`Q` support + an empty
    /// constant cone), the structural fields project `Memory`, and the cone
    /// `target`s are `"mem:<id>.<port>"`.
    #[test]
    fn memory_provenance_projects_ports_and_support() {
        let m = dual_port_mem_module();
        let a = module_memory_provenance(&m, None);
        assert_eq!(a.query, QUERY_MEMORY_PROVENANCE);
        // Only the memory_provenance vec is populated.
        assert!(a.results.is_empty() && a.reach_results.is_empty() && a.flop_provenance.is_empty());
        assert_eq!(a.memory_provenance.len(), 1);

        let mp = &a.memory_provenance[0];
        assert_eq!(mp.mem, 0);
        assert_eq!(mp.addr_width, 4);
        assert_eq!(mp.data_width, 8);
        assert_eq!(mp.kind, "simple_dual_port");
        assert!(!mp.single_port);

        // Read address: depends on `ra` only.
        assert_eq!(mp.read_addr_support.target, "mem:0.raddr");
        assert_eq!(mp.read_addr_support.support_inputs, vec!["ra"]);
        assert!(mp.read_addr_support.support_flops.is_empty());

        // Write address: depends on `wa` only.
        assert_eq!(mp.write_addr_support.target, "mem:0.waddr");
        assert_eq!(mp.write_addr_support.support_inputs, vec!["wa"]);

        // Write data: `wd ^ Q0` ⇒ input `wd` + flop 0; one gate deep.
        assert_eq!(mp.write_data_support.target, "mem:0.wdata");
        assert_eq!(mp.write_data_support.support_inputs, vec!["wd"]);
        assert_eq!(mp.write_data_support.support_flops, vec![0]);
        assert_eq!(mp.write_data_support.cone_depth, 1);

        // Write enable: a constant ⇒ no support source (counted as one cone node).
        assert_eq!(mp.write_enable_support.target, "mem:0.we");
        assert!(mp.write_enable_support.support_inputs.is_empty());
        assert!(mp.write_enable_support.support_flops.is_empty());
        assert_eq!(mp.write_enable_support.cone_nodes, 1);
    }

    /// A `SinglePort` memory shares one address node, so the read and write address
    /// cones carry identical support (only their `target` labels differ), and
    /// `single_port` is set.
    #[test]
    fn memory_provenance_single_port_shares_address() {
        let mut m = Module {
            name: "sp".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 4, Direction::In));
        m.inputs.push(port(1, "d", 8, Direction::In));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // 0 = a (shared addr)
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 }); // 1 = d
        m.nodes.push(Node::Constant { width: 1, value: 1 }); // 2 = we
        m.memories.push(Memory {
            id: 0,
            addr_width: 4,
            data_width: 8,
            kind: MemKind::SinglePort,
            we: 2,
            waddr: 0, // shared with raddr
            wdata: 1,
            raddr: 0, // SAME node as waddr
        });

        let mp = &module_memory_provenance(&m, None).memory_provenance[0];
        assert_eq!(mp.kind, "single_port");
        assert!(mp.single_port);
        // Identical support (same source node), distinct target labels.
        assert_eq!(mp.read_addr_support.support_inputs, vec!["a"]);
        assert_eq!(
            mp.read_addr_support.support_inputs,
            mp.write_addr_support.support_inputs
        );
        assert_eq!(
            mp.read_addr_support.cone_nodes,
            mp.write_addr_support.cone_nodes
        );
        assert_eq!(mp.read_addr_support.target, "mem:0.raddr");
        assert_eq!(mp.write_addr_support.target, "mem:0.waddr");
    }

    /// `target = "mem:<id>"` addresses one memory; an unknown id, a non-`mem:`
    /// string, or a malformed id yields no result (→ `-32602` at the MCP layer).
    #[test]
    fn memory_provenance_target_and_unknown() {
        let m = dual_port_mem_module();
        let one = module_memory_provenance(&m, Some("mem:0"));
        assert_eq!(one.memory_provenance.len(), 1);
        assert_eq!(one.memory_provenance[0].mem, 0);

        assert!(module_memory_provenance(&m, Some("mem:9"))
            .memory_provenance
            .is_empty());
        assert!(module_memory_provenance(&m, Some("nope"))
            .memory_provenance
            .is_empty());
        assert!(module_memory_provenance(&m, Some("mem:abc"))
            .memory_provenance
            .is_empty());
    }

    /// `target = None` ⇒ one entry per memory in ascending id order, regardless of
    /// the `m.memories` vec order.
    #[test]
    fn memory_provenance_none_yields_all_ascending() {
        let mut m = Module {
            name: "two".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 4, Direction::In));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // 0 = a
        m.nodes.push(Node::Constant { width: 8, value: 0 }); // 1 = data const
        m.nodes.push(Node::Constant { width: 1, value: 0 }); // 2 = we const
        let mk = |id| Memory {
            id,
            addr_width: 4,
            data_width: 8,
            kind: MemKind::SinglePort,
            we: 2,
            waddr: 0,
            wdata: 1,
            raddr: 0,
        };
        // Pushed out of order (1 before 0) to prove ascending-id emission.
        m.memories.push(mk(1));
        m.memories.push(mk(0));

        let a = module_memory_provenance(&m, None);
        let ids: Vec<u32> = a.memory_provenance.iter().map(|e| e.mem).collect();
        assert_eq!(ids, vec![0, 1]);
    }

    /// A memoryless module + `target = None` ⇒ an empty (not errored)
    /// `memory_provenance` — the honest "no memories" answer (the default-off
    /// `memory_prob` case).
    #[test]
    fn memoryless_module_yields_empty_memory_provenance() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));
        let a = module_memory_provenance(&m, None);
        assert_eq!(a.query, QUERY_MEMORY_PROVENANCE);
        assert!(a.memory_provenance.is_empty());
    }

    /// A `memory_provenance` document serializes `memory_provenance`, omits the
    /// other five query vecs (`skip_serializing_if`), and keeps `results` an empty
    /// array — so an `output_support` document (which omits `memory_provenance`)
    /// stays byte-identical.
    #[test]
    fn memory_provenance_document_omits_the_other_query_vecs() {
        let m = dual_port_mem_module();
        let v = serde_json::to_value(module_memory_provenance(&m, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("memory_provenance"));
        assert!(obj.get("reach_results").is_none());
        assert!(obj.get("flop_provenance").is_none());
        assert!(obj.get("module_reachability").is_none());
        assert!(obj.get("flop_dependencies").is_none());
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty

        let sup = serde_json::to_value(module_support_cones(&m, None)).unwrap();
        assert!(sup.as_object().unwrap().get("memory_provenance").is_none());
    }

    /// The design variant projects the **top** module's memories and resolves an
    /// instance-output leaf in a port cone to `"<instance>.<child-output-port>"`;
    /// an absent top ⇒ an empty analysis.
    #[test]
    fn design_memory_provenance_projects_the_top_module() {
        // Child: out port "o" driven by input "a".
        let mut child = Module {
            name: "child".into(),
            ..Module::default()
        };
        child.inputs.push(port(0, "a", 4, Direction::In));
        child.outputs.push(port(1, "o", 4, Direction::Out));
        child.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        child.drives.push((1, 0));

        // Top: a memory whose read address is the child instance's output.
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(port(0, "d", 8, Direction::In));
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "child".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![],
            param_bindings: vec![],
        });
        top.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = d
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1, // child output "o"
            width: 4,
        }); // 1 = u0.o
        top.nodes.push(Node::Constant { width: 1, value: 0 }); // 2 = we
        top.memories.push(Memory {
            id: 0,
            addr_width: 4,
            data_width: 8,
            kind: MemKind::SimpleDualPort,
            we: 2,
            waddr: 1, // u0.o
            wdata: 0, // d
            raddr: 1, // u0.o
        });

        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };
        let a = design_memory_provenance(&design, None);
        assert_eq!(a.memory_provenance.len(), 1);
        let mp = &a.memory_provenance[0];
        // The read-address cone's instance leaf resolves to the child port name.
        assert_eq!(mp.read_addr_support.support_instance_outputs, vec!["u0.o"]);
        assert_eq!(mp.write_data_support.support_inputs, vec!["d"]);

        // Absent top ⇒ empty.
        let ghost = Design {
            top: "ghost".into(),
            modules: vec![Module {
                name: "child".into(),
                ..Module::default()
            }],
        };
        assert!(design_memory_provenance(&ghost, None)
            .memory_provenance
            .is_empty());
    }

    // ---- fsm_provenance (.8b.1) ----

    /// A module with one generated-encoding FSM whose `sel` cone is `s ^ Q0`
    /// (input `s` + flop 0). `encoding`/`mealy` parameterize the shape under test;
    /// `num_states = 4`, `sel_width = 1`, `out_width = 2`.
    fn fsm_module(encoding: FsmEncoding, mealy: bool) -> Module {
        let mut m = Module {
            name: "fsm".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "s", 1, Direction::In));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 0 = s
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 }); // 1 = Q0
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![0, 1], // s ^ Q0
            width: 1,
            deps: crate::ir::DepSet::new(),
        }); // 2 = sel cone
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(0), // D = s (well-formed; not under test here)
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        let num_states = 4u32;
        let sel_width = 1u32;
        let cols = 1usize << sel_width;
        let transitions: Vec<Vec<u32>> = (0..num_states)
            .map(|s| {
                (0..cols)
                    .map(|c| ((s as usize + c) % num_states as usize) as u32)
                    .collect()
            })
            .collect();
        let outputs: Vec<u128> = (0..num_states as u128).collect();
        let mealy_outputs = mealy.then(|| {
            (0..num_states)
                .map(|_| (0..cols).map(|c| c as u128).collect())
                .collect()
        });
        m.fsms.push(Fsm {
            id: 0,
            num_states,
            encoding,
            sel: 2, // s ^ Q0
            sel_width,
            transitions,
            outputs,
            out_width: 2,
            mealy_outputs,
        });
        m
    }

    /// The `sel` support cone is exact (input + flop-`Q` support, one gate deep),
    /// the structural fields project `Fsm`, and the cone `target` is `"fsm:<id>.sel"`.
    #[test]
    fn fsm_provenance_projects_shape_and_sel_support() {
        let m = fsm_module(FsmEncoding::Binary, false);
        let a = module_fsm_provenance(&m, None);
        assert_eq!(a.query, QUERY_FSM_PROVENANCE);
        // Only the fsm_provenance vec is populated.
        assert!(a.results.is_empty() && a.reach_results.is_empty() && a.flop_provenance.is_empty());
        assert!(
            a.module_reachability.is_empty()
                && a.flop_dependencies.is_empty()
                && a.memory_provenance.is_empty()
        );
        assert_eq!(a.fsm_provenance.len(), 1);

        let fp = &a.fsm_provenance[0];
        assert_eq!(fp.fsm, 0);
        assert_eq!(fp.num_states, 4);
        assert_eq!(fp.encoding, "binary");
        assert_eq!(fp.state_width, 2); // ceil(log2 4)
        assert_eq!(fp.sel_width, 1);
        assert_eq!(fp.out_width, 2);
        assert!(!fp.is_mealy);

        // sel = s ^ Q0 ⇒ input `s` + flop 0, one gate deep.
        assert_eq!(fp.sel_support.target, "fsm:0.sel");
        assert_eq!(fp.sel_support.support_inputs, vec!["s"]);
        assert_eq!(fp.sel_support.support_flops, vec![0]);
        assert!(fp.sel_support.support_instance_outputs.is_empty());
        assert_eq!(fp.sel_support.cone_depth, 1);
    }

    /// Each `FsmEncoding` maps to its stable string + the right `state_width`
    /// (Binary/Gray = ceil(log2 n); OneHot = n).
    #[test]
    fn fsm_provenance_encodings_and_state_width() {
        for (enc, name, width) in [
            (FsmEncoding::Binary, "binary", 2u32),
            (FsmEncoding::OneHot, "one_hot", 4u32),
            (FsmEncoding::Gray, "gray", 2u32),
        ] {
            let m = fsm_module(enc, false);
            let fp = &module_fsm_provenance(&m, None).fsm_provenance[0];
            assert_eq!(fp.encoding, name);
            assert_eq!(fp.state_width, width);
        }
    }

    /// A Mealy FSM (`mealy_outputs.is_some()`) sets `is_mealy`; a Moore FSM clears it.
    #[test]
    fn fsm_provenance_mealy_flag() {
        assert!(
            module_fsm_provenance(&fsm_module(FsmEncoding::Binary, true), None).fsm_provenance[0]
                .is_mealy
        );
        assert!(
            !module_fsm_provenance(&fsm_module(FsmEncoding::Binary, false), None).fsm_provenance[0]
                .is_mealy
        );
    }

    /// `target = "fsm:<id>"` addresses one FSM; an unknown id, a non-`fsm:` string,
    /// or a malformed id yields no result (→ `-32602` at the MCP layer).
    #[test]
    fn fsm_provenance_target_and_unknown() {
        let m = fsm_module(FsmEncoding::Binary, false);
        let one = module_fsm_provenance(&m, Some("fsm:0"));
        assert_eq!(one.fsm_provenance.len(), 1);
        assert_eq!(one.fsm_provenance[0].fsm, 0);

        assert!(module_fsm_provenance(&m, Some("fsm:9"))
            .fsm_provenance
            .is_empty());
        assert!(module_fsm_provenance(&m, Some("nope"))
            .fsm_provenance
            .is_empty());
        assert!(module_fsm_provenance(&m, Some("fsm:abc"))
            .fsm_provenance
            .is_empty());
    }

    /// `target = None` ⇒ one entry per FSM in ascending id order, regardless of the
    /// `m.fsms` vec order.
    #[test]
    fn fsm_provenance_none_yields_all_ascending() {
        let mut m = fsm_module(FsmEncoding::Binary, false);
        // Add a second FSM (id 1) reusing the same sel cone, pushed so the vec is
        // already [0]; then insert id 1 at the front to prove ascending emission.
        let second = Fsm {
            id: 1,
            num_states: 2,
            encoding: FsmEncoding::OneHot,
            sel: 2,
            sel_width: 1,
            transitions: vec![vec![0, 1], vec![1, 0]],
            outputs: vec![0, 1],
            out_width: 2,
            mealy_outputs: None,
        };
        m.fsms.insert(0, second); // vec order now [1, 0]
        let a = module_fsm_provenance(&m, None);
        let ids: Vec<u32> = a.fsm_provenance.iter().map(|e| e.fsm).collect();
        assert_eq!(ids, vec![0, 1]);
    }

    /// An FSM-less module + `target = None` ⇒ an empty (not errored) `fsm_provenance`
    /// — the honest "no FSMs" answer (the default-off `fsm_prob` case).
    #[test]
    fn fsmless_module_yields_empty_fsm_provenance() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.outputs.push(port(1, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.drives.push((1, 0));
        let a = module_fsm_provenance(&m, None);
        assert_eq!(a.query, QUERY_FSM_PROVENANCE);
        assert!(a.fsm_provenance.is_empty());
    }

    /// A `fsm_provenance` document serializes `fsm_provenance`, omits the other six
    /// query vecs (`skip_serializing_if`), and keeps `results` an empty array — so an
    /// `output_support` document (which omits `fsm_provenance`) stays byte-identical.
    #[test]
    fn fsm_provenance_document_omits_the_other_query_vecs() {
        let m = fsm_module(FsmEncoding::Binary, false);
        let v = serde_json::to_value(module_fsm_provenance(&m, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("fsm_provenance"));
        assert!(obj.get("reach_results").is_none());
        assert!(obj.get("flop_provenance").is_none());
        assert!(obj.get("module_reachability").is_none());
        assert!(obj.get("flop_dependencies").is_none());
        assert!(obj.get("memory_provenance").is_none());
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty

        let sup = serde_json::to_value(module_support_cones(&m, None)).unwrap();
        assert!(sup.as_object().unwrap().get("fsm_provenance").is_none());
    }

    /// The design variant projects the **top** module's FSMs and resolves an
    /// instance-output leaf in the `sel` cone to `"<instance>.<child-output-port>"`;
    /// an absent top ⇒ an empty analysis.
    #[test]
    fn design_fsm_provenance_projects_the_top_module() {
        // Child: out port "o" driven by input "a".
        let mut child = Module {
            name: "child".into(),
            ..Module::default()
        };
        child.inputs.push(port(0, "a", 1, Direction::In));
        child.outputs.push(port(1, "o", 1, Direction::Out));
        child.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        child.drives.push((1, 0));

        // Top: an FSM whose `sel` is the child instance's output.
        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(port(0, "clk", 1, Direction::In));
        top.inputs.push(port(1, "rst_n", 1, Direction::In));
        top.clock = Some(0);
        top.reset = Some(1);
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "child".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![],
            param_bindings: vec![],
        });
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1, // child output "o"
            width: 1,
        }); // 0 = u0.o
        top.fsms.push(Fsm {
            id: 0,
            num_states: 2,
            encoding: FsmEncoding::Binary,
            sel: 0, // u0.o
            sel_width: 1,
            transitions: vec![vec![0, 1], vec![1, 0]],
            outputs: vec![0, 1],
            out_width: 1,
            mealy_outputs: None,
        });

        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };
        let a = design_fsm_provenance(&design, None);
        assert_eq!(a.fsm_provenance.len(), 1);
        let fp = &a.fsm_provenance[0];
        assert_eq!(fp.sel_support.target, "fsm:0.sel");
        assert_eq!(fp.sel_support.support_instance_outputs, vec!["u0.o"]);
        assert!(fp.sel_support.support_inputs.is_empty());

        // Absent top ⇒ empty.
        let ghost = Design {
            top: "ghost".into(),
            modules: vec![Module {
                name: "child".into(),
                ..Module::default()
            }],
        };
        assert!(design_fsm_provenance(&ghost, None)
            .fsm_provenance
            .is_empty());
    }

    // ---- node_drivers (`SEMANTIC-INTROSPECTION-EXPANSION.9`) -------------------

    /// `y = (a & b) | c`: a Gate's `drivers` are its operands **in operand order**,
    /// each classified + resolved (an interior gate ⇒ `"node:<id>"`, an input ⇒ its
    /// port name); a leaf node has empty `drivers` and no `op`; `None` ⇒ one entry per
    /// node, ascending id; deterministic.
    #[test]
    fn node_drivers_gate_operands_in_order_with_kinds_and_names() {
        let mut m = Module {
            name: "comb".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 8, Direction::In));
        m.inputs.push(port(1, "b", 8, Direction::In));
        m.inputs.push(port(2, "c", 8, Direction::In));
        m.outputs.push(port(3, "y", 8, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 }); // 1 = b
        m.nodes.push(Node::PrimaryInput { port: 2, width: 8 }); // 2 = c
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::And,
            operands: vec![0, 1],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3 = a & b
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Or,
            operands: vec![3, 2],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 4 = (a&b) | c
        m.drives.push((3, 4));

        let a = module_node_drivers(&m, None);
        assert_eq!(a.query, QUERY_NODE_DRIVERS);
        assert_eq!(a.node_drivers.len(), 5); // one per node
        for (i, nd) in a.node_drivers.iter().enumerate() {
            assert_eq!(nd.node, i as u32); // ascending id == index
        }

        // node 4 = Or((a&b), c): op "or", operands in order [3 (gate), 2 (input c)].
        let or = &a.node_drivers[4];
        assert_eq!(or.kind, "gate");
        assert_eq!(or.op.as_deref(), Some("or"));
        assert_eq!(or.width, 8);
        let or_refs: Vec<(u32, &str, &str)> = or
            .drivers
            .iter()
            .map(|r| (r.node, r.kind.as_str(), r.name.as_str()))
            .collect();
        assert_eq!(
            or_refs,
            vec![(3, "gate", "node:3"), (2, "primary_input", "c")]
        );

        // node 3 = And(a, b): op "and", operands [0=a, 1=b] with resolved input names.
        let and = &a.node_drivers[3];
        assert_eq!(and.op.as_deref(), Some("and"));
        assert_eq!(
            and.drivers
                .iter()
                .map(|r| r.name.clone())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );

        // node 0 = primary input a: a leaf ⇒ empty drivers, no op.
        let leaf = &a.node_drivers[0];
        assert_eq!(leaf.kind, "primary_input");
        assert!(leaf.op.is_none());
        assert!(leaf.drivers.is_empty());

        // Deterministic.
        assert_eq!(a, module_node_drivers(&m, None));
    }

    /// Operand order is preserved for a non-commutative op (a `Mux`'s `[sel, a, b]`),
    /// the `GateOp` maps to a stable string, and a flop-`Q` operand resolves to
    /// `"flop:<id>"`.
    #[test]
    fn node_drivers_preserves_operand_order_and_resolves_flop_q() {
        let mut m = Module {
            name: "seq".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "clk", 1, Direction::In));
        m.inputs.push(port(1, "rst_n", 1, Direction::In));
        m.inputs.push(port(2, "sel", 1, Direction::In));
        m.inputs.push(port(3, "a", 8, Direction::In));
        m.outputs.push(port(4, "y", 8, Direction::Out));
        m.clock = Some(0);
        m.reset = Some(1);
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 0 = sel
        m.nodes.push(Node::PrimaryInput { port: 3, width: 8 }); // 1 = a
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 2 = Q
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Mux,
            operands: vec![0, 1, 2], // sel ? a : Q
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3
        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(3),
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((4, 3));

        let one = module_node_drivers(&m, Some("node:3"));
        assert_eq!(one.node_drivers.len(), 1);
        let mux = &one.node_drivers[0];
        assert_eq!(mux.node, 3);
        assert_eq!(mux.op.as_deref(), Some("mux"));
        let refs: Vec<(u32, &str, &str)> = mux
            .drivers
            .iter()
            .map(|r| (r.node, r.kind.as_str(), r.name.as_str()))
            .collect();
        assert_eq!(
            refs,
            vec![
                (0, "primary_input", "sel"),
                (1, "primary_input", "a"),
                (2, "flop_q", "flop:0"),
            ]
        );
    }

    /// Operand handles resolve for the remaining leaf classes: a constant ⇒
    /// `"node:<id>"` (no external handle), a memory read ⇒ `"mem:<id>"`, an FSM output
    /// ⇒ `"fsm:<id>"`; each is itself a leaf entry (empty drivers, no op).
    #[test]
    fn node_drivers_resolves_constant_mem_and_fsm_operand_handles() {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.outputs.push(port(0, "y", 8, Direction::Out));
        m.nodes.push(Node::Constant { width: 8, value: 5 }); // 0
        m.nodes.push(Node::MemRead { mem: 2, width: 8 }); // 1
        m.nodes.push(Node::FsmOut { fsm: 4, width: 8 }); // 2
        m.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Add,
            operands: vec![0, 1, 2],
            width: 8,
            deps: crate::ir::DepSet::new(),
        }); // 3
        m.drives.push((0, 3));

        let nd = module_node_drivers(&m, None).node_drivers;
        assert_eq!(nd[0].kind, "constant");
        assert!(nd[0].op.is_none());
        assert!(nd[0].drivers.is_empty());
        assert_eq!(nd[1].kind, "mem_read");
        assert_eq!(nd[2].kind, "fsm_out");

        let add = &nd[3];
        assert_eq!(add.op.as_deref(), Some("add"));
        let refs: Vec<(&str, &str)> = add
            .drivers
            .iter()
            .map(|r| (r.kind.as_str(), r.name.as_str()))
            .collect();
        assert_eq!(
            refs,
            vec![
                ("constant", "node:0"),
                ("mem_read", "mem:2"),
                ("fsm_out", "fsm:4"),
            ]
        );
    }

    /// `"node:<id>"` resolves to exactly one entry (even a leaf — known-but-empty);
    /// an out-of-range id or a malformed target ⇒ no entry (→ `-32602` at the MCP
    /// layer).
    #[test]
    fn node_drivers_target_and_unknown() {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 4, Direction::In));
        m.outputs.push(port(1, "y", 4, Direction::Out));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 }); // 0
        m.drives.push((1, 0));

        // Resolvable leaf target ⇒ one known-but-empty entry (NOT an error).
        let one = module_node_drivers(&m, Some("node:0"));
        assert_eq!(one.node_drivers.len(), 1);
        assert_eq!(one.node_drivers[0].node, 0);
        assert!(one.node_drivers[0].drivers.is_empty());

        // Out-of-range id, and malformed / wrong-vocabulary targets ⇒ none.
        for bad in ["node:99", "node:nope", "node:", "flop:0", "y", "0"] {
            assert!(
                module_node_drivers(&m, Some(bad)).node_drivers.is_empty(),
                "target {bad:?} should resolve to no entry"
            );
        }
    }

    /// The `node_drivers` document omits the other seven query vecs (and a leaf entry
    /// omits `op`); `output_support` omits `node_drivers` ⇒ prior documents
    /// byte-identical.
    #[test]
    fn node_drivers_document_omits_the_other_query_vecs() {
        let mut m = Module {
            name: "m".into(),
            ..Module::default()
        };
        m.inputs.push(port(0, "a", 4, Direction::In));
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let v = serde_json::to_value(module_node_drivers(&m, None)).unwrap();
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("node_drivers"));
        assert!(obj.get("reach_results").is_none());
        assert!(obj.get("flop_provenance").is_none());
        assert!(obj.get("module_reachability").is_none());
        assert!(obj.get("flop_dependencies").is_none());
        assert!(obj.get("memory_provenance").is_none());
        assert!(obj.get("fsm_provenance").is_none());
        assert_eq!(obj["results"].as_array().unwrap().len(), 0); // always present, empty
                                                                 // The lone leaf node entry omits the `op` key.
        assert!(v["node_drivers"][0]
            .as_object()
            .unwrap()
            .get("op")
            .is_none());

        let sup = serde_json::to_value(module_support_cones(&m, None)).unwrap();
        assert!(sup.as_object().unwrap().get("node_drivers").is_none());
    }

    /// The design variant resolves an instance-output operand handle to
    /// `"<instance>.<child-output-port>"` (the module variant falls back to
    /// `"<instance>.port<id>"`); an absent top ⇒ an empty analysis.
    #[test]
    fn design_node_drivers_resolves_instance_output_operand() {
        let mut child = Module {
            name: "child".into(),
            ..Module::default()
        };
        child.inputs.push(port(0, "a", 1, Direction::In));
        child.outputs.push(port(1, "o", 1, Direction::Out));
        child.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        child.drives.push((1, 0));

        let mut top = Module {
            name: "top".into(),
            ..Module::default()
        };
        top.inputs.push(port(0, "a", 1, Direction::In));
        top.outputs.push(port(1, "y", 1, Direction::Out));
        top.instances.push(Instance {
            id: 0,
            name: "u0".into(),
            module: "child".into(),
            role: InstanceRole::PlannedChild,
            inputs: vec![],
            param_bindings: vec![],
        });
        top.nodes.push(Node::InstanceOutput {
            instance: 0,
            port: 1, // child output "o"
            width: 1,
        }); // 0 = u0.o
        top.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 1 = a
        top.nodes.push(Node::Gate {
            op: crate::ir::GateOp::Xor,
            operands: vec![1, 0], // a ^ u0.o
            width: 1,
            deps: crate::ir::DepSet::new(),
        }); // 2
        top.drives.push((1, 2));

        // Module variant: no child def ⇒ "<instance>.port<id>" fallback.
        let m_xor = &module_node_drivers(&top, Some("node:2")).node_drivers[0];
        assert_eq!(m_xor.drivers[1].name, "u0.port1");

        let design = Design {
            top: "top".into(),
            modules: vec![top, child],
        };
        let d_xor = &design_node_drivers(&design, Some("node:2")).node_drivers[0];
        assert_eq!(d_xor.op.as_deref(), Some("xor"));
        assert_eq!(d_xor.drivers[1].kind, "instance_output");
        assert_eq!(d_xor.drivers[1].name, "u0.o"); // resolved child output port name

        // Absent top ⇒ empty.
        let ghost = Design {
            top: "ghost".into(),
            modules: vec![],
        };
        assert!(design_node_drivers(&ghost, None).node_drivers.is_empty());
    }
}
