//! Core IR types. Every constructor is responsible for preserving its
//! structural invariants (width consistency, dep-set correctness,
//! operand count). The validator in `validate.rs` is a development-time
//! safety net, not a production gate.
//!
//! Vocabulary: "arity" is used only for operators (associative primitives
//! like `And`, `Add`). Blocks (`Mux`, `Flop`) have "ports" or "arms", not
//! arity. See `book/src/structural-rules.md` "Operators vs blocks".

use std::collections::{BTreeSet, HashMap};

pub type PortId = u32;
pub type NodeId = u32;
pub type FlopId = u32;

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

/// A circuit module: ports, internal nodes, flops, and a drive map
/// from each output port to the node that drives it.
#[derive(Debug, Clone, Default)]
pub struct Module {
    pub name: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub clock: Option<PortId>,
    pub reset: Option<PortId>,
    pub nodes: Vec<Node>,
    pub flops: Vec<Flop>,
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
}

impl Module {
    /// Intern a gate expression. If `(op, operands, width)` has already
    /// been created `< max_ast_instances` times, create a new
    /// `Node::Gate` and register it. Otherwise return the most recent
    /// existing instance. Returns `(NodeId, is_new)` so callers can
    /// gate their `pool.add` call on actual creation.
    pub fn intern_gate(
        &mut self,
        op: GateOp,
        operands: Vec<NodeId>,
        width: u32,
        deps: DepSet,
    ) -> (NodeId, bool) {
        let cap = self.max_ast_instances.max(1) as usize;
        let key = (op, operands.clone(), width);
        if let Some(vec) = self.gate_instances.get(&key) {
            if vec.len() >= cap {
                let existing = *vec.last().expect("cap >= 1 ensures vec is non-empty");
                let existing_width = self.nodes[existing as usize].width();
                debug_assert_eq!(
                    existing_width, width,
                    "intern_gate dedup returned node with wrong width: op={:?} key_width={} got_width={}",
                    op, width, existing_width
                );
                crate::trace_verbose!(
                    node = existing,
                    ?op,
                    width,
                    "♻️ intern_gate reuse (AST cap hit)"
                );
                return (existing, false);
            }
        }
        let node_id = self.nodes.len() as NodeId;
        self.nodes.push(Node::Gate {
            op,
            operands: operands.clone(),
            width,
            deps,
        });
        self.gate_instances.entry(key).or_default().push(node_id);
        crate::trace_verbose!(
            node = node_id,
            ?op,
            width,
            n_operands = operands.len(),
            "🔗 intern_gate new"
        );
        (node_id, true)
    }

    /// Intern a constant. Same cap semantics as `intern_gate`.
    pub fn intern_constant(&mut self, width: u32, value: u128) -> (NodeId, bool) {
        let cap = self.max_ast_instances.max(1) as usize;
        let key = (width, value);
        if let Some(vec) = self.const_instances.get(&key) {
            if vec.len() >= cap {
                let existing = *vec.last().expect("cap >= 1 ensures vec is non-empty");
                let existing_width = self.nodes[existing as usize].width();
                debug_assert_eq!(
                    existing_width, width,
                    "intern_constant dedup returned node with wrong width: key_width={} got_width={}",
                    width, existing_width
                );
                crate::trace_verbose!(
                    node = existing,
                    width,
                    value,
                    "♻️ intern_constant reuse (AST cap hit)"
                );
                return (existing, false);
            }
        }
        let node_id = self.nodes.len() as NodeId;
        self.nodes.push(Node::Constant { width, value });
        self.const_instances.entry(key).or_default().push(node_id);
        crate::trace_verbose!(node = node_id, width, value, "🔗 intern_constant new");
        (node_id, true)
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
            | Node::Gate { width, .. } => *width,
        }
    }
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
    Mux, // [sel, a, b] with sel.width == 1
    Slice { hi: u32, lo: u32 },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Set of primary-input port ids (plus virtual ids for flops) that a node
/// depends on. Empty dep-set on an output cone indicates the cone is
/// trivially constant and must be regenerated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DepSet {
    set: BTreeSet<u32>,
}

impl DepSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_port(p: PortId) -> Self {
        let mut s = BTreeSet::new();
        s.insert(p);
        Self { set: s }
    }

    pub fn from_flop_virtual(flop: FlopId) -> Self {
        // Virtual ids live in a disjoint numeric space from port ids.
        // We tag flop virtual ids with the high bit set.
        let mut s = BTreeSet::new();
        s.insert(0x8000_0000 | flop);
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

    pub fn contains(&self, id: u32) -> bool {
        self.set.contains(&id)
    }
}

/// A design is one or more modules with a designated top.
#[derive(Debug, Clone)]
pub struct Design {
    pub top: String,
    pub modules: Vec<Module>,
}
