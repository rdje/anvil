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

use crate::ir::{Design, InstanceId, Module, Node, NodeId, PortId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// The query-kind string for the first derived query: the transitive
/// combinational fan-in support cone of a target.
pub const QUERY_OUTPUT_SUPPORT: &str = "output_support";

/// Every derived-query kind this surface answers today. The MCP `analyze`
/// tool (`.2b.2`) rejects any `query` not in this set with `-32602`. New
/// kinds (`input_reach`, `flop_reset_provenance`, `module_reachability`) slot
/// in here without changing the document shape.
pub fn supported_query_kinds() -> &'static [&'static str] {
    &[QUERY_OUTPUT_SUPPORT]
}

/// The result of one derived-relation query over an artifact: a list of
/// per-target [`SupportCone`]s. A pure post-hoc projection of the emitted IR
/// (invariant SCHEMA-DERIVED) — no new computed truth.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedAnalysis {
    /// The query kind (e.g. [`QUERY_OUTPUT_SUPPORT`]).
    pub query: String,
    /// One entry per resolved target. Empty iff an explicit `target` did not
    /// resolve (the MCP layer maps that to `-32602`).
    pub results: Vec<SupportCone>,
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
        };
    };
    let fmt = |inst: InstanceId, port: PortId| format_instance_leaf_design(design, top, inst, port);
    support_cones_with(top, target, &fmt)
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
    use crate::ir::types::{Direction, Flop, FlopKind, FlopMux, Instance, InstanceRole, ResetKind};
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
}
