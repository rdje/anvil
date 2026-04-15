//! Core IR types. Every constructor is responsible for preserving its
//! structural invariants (width consistency, dep-set correctness,
//! operand arity). The validator in `validate.rs` is a development-time
//! safety net, not a production gate.

use std::collections::BTreeSet;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Flop {
    pub id: FlopId,
    pub width: u32,
    pub d: Option<NodeId>,
    pub q: NodeId,
    pub reset_val: u128,
    pub reset_kind: ResetKind,
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
