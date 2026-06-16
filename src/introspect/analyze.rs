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
//!   `cone_nodes`, recorded in no list). Surfacing memory/FSM provenance is a
//!   recorded future query kind, not a silent omission.
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
    Design, Flop, FlopKind, FlopMux, InstanceId, Module, Node, NodeId, PortId, ResetKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

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

/// Every derived-query kind the MCP `analyze` tool answers today. The tool
/// rejects any `query` not in this set with `-32602`. A kind appears here
/// **only once its `run_analyze` dispatch is wired**, so the registry and the
/// dispatch never disagree. The remaining future kind (`module_reachability`)
/// slots in the same way without changing the document shape.
pub fn supported_query_kinds() -> &'static [&'static str] {
    &[
        QUERY_OUTPUT_SUPPORT,
        QUERY_INPUT_REACH,
        QUERY_FLOP_RESET_PROVENANCE,
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
        };
    };
    flop_provenance_with(top, target)
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
}
