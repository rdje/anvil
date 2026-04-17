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
    /// Factorization level — which layers of the dedup chain are
    /// active. See `Config::factorization_level`. Default `Full`.
    pub factorization_level: crate::config::FactorizationLevel,

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

    /// Number of times the `ConstantFold` layer fired during
    /// construction of this module. Each fire is one algebraic
    /// identity applied in `intern_gate` — operands dropped, an
    /// absorbing constant substituted, or a single surviving
    /// operand short-circuited. Exposed via
    /// `Metrics::fold_identities_applied` for empirical
    /// measurement of the `ConstantFold` factorization layer.
    pub fold_identities_applied: u64,

    /// Number of times the `Peephole` layer fired during
    /// construction of this module. Each fire is one local rewrite
    /// applied in `intern_gate` — double-negation collapse
    /// (`Not(Not(x)) → x`), fully-constant comparisons evaluated at
    /// intern time (`Eq(c1, c2) → 1-bit const`,
    /// `Neq(c1, c2) → 1-bit const`), full-width slice identity
    /// (`Slice(hi, lo) where hi-lo+1 == src_width → src`), and
    /// single-operand Concat (`Concat([x]) → x`). Surfaced via
    /// `Metrics::peephole_rewrites_applied`.
    pub peephole_rewrites_applied: u64,
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
        mut operands: Vec<NodeId>,
        width: u32,
        deps: DepSet,
    ) -> (NodeId, bool) {
        use crate::config::FactorizationLevel;

        // Commutative normalization (layer Commutative and above):
        // sort operands for commutative ops so `a + b` and `b + a`
        // share identity. Disabled at lower factorization levels.
        if self.factorization_level.effective() >= FactorizationLevel::Commutative
            && matches!(
                op,
                GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
            )
        {
            operands.sort_unstable();
        }

        // Constant folding (layer ConstantFold and above): apply
        // algebraic identities at intern time. `x + 0 → x`,
        // `x * 1 → x`, `x & 0 → 0`, `x | all_ones → all_ones`,
        // `x ^ 0 → x`, `x << 0 → x`, `x >> 0 → x`, and
        // `x - 0 → x`. See `book/src/structural-rules.md` Rule 21c.
        // The helper shrinks the operand list and may short-circuit
        // to a surviving NodeId or a constant; on `Some` we return
        // without consulting the dedup tables.
        if self.factorization_level.effective() >= FactorizationLevel::ConstantFold {
            if let Some((folded_id, is_new)) = self.fold_constants(op, &mut operands, width) {
                return (folded_id, is_new);
            }
        }

        // Peephole rewrites (layer Peephole and above): apply local
        // rewrite rules that collapse specific shapes at intern time.
        // `Not(Not(x)) → x`, `Eq/Neq(const, const)` evaluated,
        // full-width `Slice → src`, single-operand `Concat → that
        // operand`. See `book/src/structural-rules.md` Rule 21c.
        if self.factorization_level.effective() >= FactorizationLevel::Peephole {
            if let Some((rewritten_id, is_new)) =
                self.apply_peephole(op, &operands, width)
            {
                return (rewritten_id, is_new);
            }
        }

        // Level = None bypasses dedup entirely: every call creates
        // a fresh NodeId. Useful for stress-testing downstream CSE.
        if self.factorization_level.effective() == FactorizationLevel::None {
            let node_id = self.nodes.len() as NodeId;
            let n = operands.len();
            self.nodes.push(Node::Gate {
                op,
                operands,
                width,
                deps,
            });
            crate::trace_verbose!(
                node = node_id,
                ?op,
                width,
                n_operands = n,
                "🔗 intern_gate new (level=none, dedup bypassed)"
            );
            return (node_id, true);
        }

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
        use crate::config::FactorizationLevel;
        if self.factorization_level.effective() == FactorizationLevel::None {
            let node_id = self.nodes.len() as NodeId;
            self.nodes.push(Node::Constant { width, value });
            crate::trace_verbose!(
                node = node_id,
                width,
                value,
                "🔗 intern_constant new (level=none, dedup bypassed)"
            );
            return (node_id, true);
        }
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

    /// Apply constant-folding identities to `operands` in place.
    ///
    /// Returns `Some((id, is_new))` when a fold short-circuits the
    /// caller — either because an absorbing element turned the gate
    /// into a constant (e.g. `x & 0 → 0`), or because identity
    /// elements drained the list down to a single surviving operand
    /// or to nothing. Returns `None` when no fold applies or when
    /// the list was shrunk but still has ≥ 2 operands — in that case
    /// the caller proceeds with normal dedup on the shrunk list.
    ///
    /// The rules covered today:
    /// - `Add`/`Sub`/`Xor`/`Shl`/`Shr`: drop `0` operands.
    /// - `Mul`: drop `1` operands; any `0` returns zero constant.
    /// - `And`: drop `all_ones` operands; any `0` returns zero.
    /// - `Or`: drop `0` operands; any `all_ones` returns all-ones.
    ///
    /// Non-commutative ops (`Sub`, `Shl`, `Shr`) fold only the
    /// rhs-constant case to avoid changing semantics (e.g.
    /// `0 - a ≠ a`). That's implemented by position-aware checks:
    /// we only drop an index-1 `0` for 2-operand Sub/Shl/Shr.
    ///
    /// Comparison ops, reductions, `Not`, `Slice`, `Concat`, `Mux`
    /// are out of scope for this layer — they belong to `Peephole`.
    pub(crate) fn fold_constants(
        &mut self,
        op: GateOp,
        operands: &mut Vec<NodeId>,
        width: u32,
    ) -> Option<(NodeId, bool)> {
        // Only associative ops (Add/Mul/And/Or/Xor), plus 2-arity
        // Sub / Shl / Shr, participate in this layer.
        let is_associative = matches!(
            op,
            GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
        );
        let is_shift_or_sub = matches!(op, GateOp::Sub | GateOp::Shl | GateOp::Shr);
        if !is_associative && !is_shift_or_sub {
            return None;
        }

        let all_ones: u128 = if width >= 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };

        // Read a node's constant value if it is one. We only fold
        // against same-width constants — mixed-width is a bug upstream.
        let const_of = |id: NodeId, nodes: &[Node]| -> Option<u128> {
            match &nodes[id as usize] {
                Node::Constant { width: w, value } if *w == width => Some(*value),
                _ => None,
            }
        };

        if is_shift_or_sub && operands.len() == 2 {
            // `a - 0 → a`, `a << 0 → a`, `a >> 0 → a`.
            if let Some(0) = const_of(operands[1], &self.nodes) {
                let surviving = operands[0];
                self.fold_identities_applied += 1;
                crate::trace_verbose!(
                    node = surviving,
                    ?op,
                    width,
                    "✂️ fold_constants short-circuit (rhs zero)"
                );
                return Some((surviving, false));
            }
            return None;
        }

        if !is_associative {
            return None;
        }

        // Associative path: scan for absorbing, then drop identity
        // operands in place.
        let before = operands.len();

        // 1. Absorbing elements: `Mul`/`And` zero, `Or` all-ones.
        //
        // **Orphan-safety restriction:** absorbing turns the whole
        // expression into a constant, so every Gate operand loses
        // its only consumer (this call) and becomes a Rule 18
        // orphan. Without NodeId compaction (a future finalisation
        // pass), we can only safely apply absorbing when no operand
        // is a Gate — i.e. the "evaluate all-constant expression"
        // subset. Constants, primary inputs, and flop Qs don't
        // count as gate orphans, so they're safe to orphan here.
        // When any operand is a Gate, the absorbing rule is
        // suppressed and the outer gate materialises normally
        // (its presence keeps the Gate operands reachable).
        let no_gate_operand = operands.iter().all(|id| {
            !matches!(self.nodes[*id as usize], Node::Gate { .. })
        });
        if no_gate_operand {
            for &id in operands.iter() {
                if let Some(v) = const_of(id, &self.nodes) {
                    let absorbs = match op {
                        GateOp::Mul | GateOp::And => v == 0,
                        GateOp::Or => v == all_ones,
                        _ => false,
                    };
                    if absorbs {
                        let absorb_value = match op {
                            GateOp::Mul | GateOp::And => 0u128,
                            GateOp::Or => all_ones,
                            _ => unreachable!(),
                        };
                        self.fold_identities_applied += 1;
                        let (cid, is_new) = self.intern_constant(width, absorb_value);
                        crate::trace_verbose!(
                            node = cid,
                            ?op,
                            width,
                            value = absorb_value,
                            "✂️ fold_constants absorbing"
                        );
                        return Some((cid, is_new));
                    }
                }
            }
        }

        // 2. Identity elements: drop in place.
        let identity: u128 = match op {
            GateOp::Add | GateOp::Xor | GateOp::Or => 0,
            GateOp::Mul => 1,
            GateOp::And => all_ones,
            _ => return None,
        };
        operands.retain(|id| match &self.nodes[*id as usize] {
            Node::Constant { width: w, value } if *w == width && *value == identity => false,
            _ => true,
        });

        if operands.len() == before {
            // Nothing folded.
            return None;
        }

        // Every drop is one identity application.
        self.fold_identities_applied += (before - operands.len()) as u64;

        match operands.len() {
            0 => {
                // All operands were the identity — result is that identity.
                let (cid, is_new) = self.intern_constant(width, identity);
                crate::trace_verbose!(
                    node = cid,
                    ?op,
                    width,
                    value = identity,
                    "✂️ fold_constants collapsed to identity"
                );
                Some((cid, is_new))
            }
            1 => {
                // Single surviving operand — return it directly,
                // no new gate node. Its deps are already correct
                // from its own construction.
                let surviving = operands[0];
                crate::trace_verbose!(
                    node = surviving,
                    ?op,
                    width,
                    "✂️ fold_constants single survivor"
                );
                Some((surviving, false))
            }
            _ => {
                // ≥ 2 operands remain; let the caller proceed with
                // normal intern on the shrunk list.
                None
            }
        }
    }

    /// Apply local peephole rewrite rules. Returns `Some((id,
    /// is_new))` when a rewrite short-circuits the caller, `None`
    /// when no rule matches. Rules implemented today:
    ///
    /// - **Fully-constant comparisons**: `Eq`/`Neq`/`Lt`/`Gt`/
    ///   `Le`/`Ge` with both operands same-width constants are
    ///   evaluated at intern time to a 1-bit constant.
    /// - **Full-width `Slice`**: `Slice(hi, 0)` with
    ///   `hi + 1 == src_width` returns the source NodeId.
    /// - **Single-operand `Concat`**: `Concat([x])` → `x`.
    ///
    /// **Why no `Not(Not(x)) → x` here.** The outer `Not` being
    /// collapsed would orphan the inner `Not` gate, which was
    /// materialised by its own earlier `intern_gate` call and is
    /// referenced only by the outer call. Without NodeId
    /// compaction (a future finalisation pass) that orphan
    /// violates Rule 18. Comparison / Slice / Concat rules are
    /// safe because they replace the outer gate with a direct
    /// operand (still referenced by the caller) or a constant (not
    /// subject to the Rule 18 gate-orphan check).
    ///
    /// Rules are narrow by design: each one is an unambiguous
    /// local identity with no width-reinterpretation or type
    /// punning. Broader rewrites like `(a + b) - b → a` need
    /// cross-gate rewriting (symbolic reasoning over trees) and
    /// belong in a future layer, not here.
    pub(crate) fn apply_peephole(
        &mut self,
        op: GateOp,
        operands: &[NodeId],
        width: u32,
    ) -> Option<(NodeId, bool)> {
        // Read a constant's (width, value) if the node is one.
        let const_of = |id: NodeId, nodes: &[Node]| -> Option<(u32, u128)> {
            match &nodes[id as usize] {
                Node::Constant { width, value } => Some((*width, *value)),
                _ => None,
            }
        };

        match op {
            GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
                if operands.len() == 2 =>
            {
                let a = const_of(operands[0], &self.nodes)?;
                let b = const_of(operands[1], &self.nodes)?;
                if a.0 != b.0 {
                    // Mixed-width — IR invariant says this shouldn't
                    // happen, but be defensive: don't fold.
                    return None;
                }
                let result: u128 = match op {
                    GateOp::Eq => (a.1 == b.1) as u128,
                    GateOp::Neq => (a.1 != b.1) as u128,
                    GateOp::Lt => (a.1 < b.1) as u128,
                    GateOp::Gt => (a.1 > b.1) as u128,
                    GateOp::Le => (a.1 <= b.1) as u128,
                    GateOp::Ge => (a.1 >= b.1) as u128,
                    _ => unreachable!(),
                };
                self.peephole_rewrites_applied += 1;
                let (cid, is_new) = self.intern_constant(width, result);
                crate::trace_verbose!(
                    node = cid,
                    ?op,
                    width,
                    value = result,
                    "✂️ peephole comparison of constants"
                );
                Some((cid, is_new))
            }
            GateOp::Slice { hi, lo } if operands.len() == 1 => {
                // Full-width slice starting at 0 is the identity.
                let src_w = self.nodes[operands[0] as usize].width();
                if lo == 0 && hi + 1 == src_w && hi - lo + 1 == width {
                    let x = operands[0];
                    self.peephole_rewrites_applied += 1;
                    crate::trace_verbose!(
                        node = x,
                        width,
                        "✂️ peephole full-width Slice → src"
                    );
                    return Some((x, false));
                }
                None
            }
            GateOp::Concat if operands.len() == 1 => {
                // Concat of a single same-width operand is the identity.
                let src_w = self.nodes[operands[0] as usize].width();
                if src_w == width {
                    let x = operands[0];
                    self.peephole_rewrites_applied += 1;
                    crate::trace_verbose!(
                        node = x,
                        width,
                        "✂️ peephole single-operand Concat → operand"
                    );
                    return Some((x, false));
                }
                None
            }
            _ => None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Building two gates with the same operator, same operand
    /// *multiset*, same width — but in different orders — must
    /// return the same `NodeId`. This is commutative normalization
    /// per Rule 21: `a + b` and `b + a` are the same expression
    /// and therefore share identity.
    #[test]
    fn intern_gate_commutative_normalization() {
        let mut m = Module {
            max_ast_instances: 1,
            ..Module::default()
        };
        // Two primary inputs — gives us two distinct NodeIds.
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;

        for op in [
            GateOp::And,
            GateOp::Or,
            GateOp::Xor,
            GateOp::Add,
            GateOp::Mul,
        ] {
            // `op(a, b)` and `op(b, a)` must dedupe to one node.
            let before = m.nodes.len();
            let (id_ab, new_ab) = m.intern_gate(op, vec![a, b], 8, DepSet::from_port(0));
            let (id_ba, new_ba) = m.intern_gate(op, vec![b, a], 8, DepSet::from_port(0));
            assert!(new_ab, "{op:?}: first call must create a new node");
            assert!(!new_ba, "{op:?}: second call must reuse the existing node");
            assert_eq!(
                id_ab, id_ba,
                "{op:?}: commutative variants must share NodeId"
            );
            assert_eq!(m.nodes.len(), before + 1, "{op:?}: only one new node added");
        }
    }

    /// Non-commutative ops must NOT be normalized: `a - b` and
    /// `b - a` are different expressions and must have different
    /// `NodeId`s.
    #[test]
    fn intern_gate_preserves_non_commutative_order() {
        let mut m = Module {
            max_ast_instances: 1,
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;

        // Sub — a - b ≠ b - a.
        let (id_ab, _) = m.intern_gate(GateOp::Sub, vec![a, b], 8, DepSet::from_port(0));
        let (id_ba, _) = m.intern_gate(GateOp::Sub, vec![b, a], 8, DepSet::from_port(0));
        assert_ne!(id_ab, id_ba, "Sub must not be commutatively normalized");

        // Mux — positional roles (sel / data_true / data_false).
        // Different orderings are different expressions.
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // node 2 (sel-width)
        let s: NodeId = 2;
        let (id_sab, _) = m.intern_gate(GateOp::Mux, vec![s, a, b], 8, DepSet::from_port(0));
        let (id_sba, _) = m.intern_gate(GateOp::Mux, vec![s, b, a], 8, DepSet::from_port(0));
        assert_ne!(id_sab, id_sba, "Mux must not be commutatively normalized");
    }

    /// Helper: module seeded with one primary-input (node 0) and
    /// two constants (node 1 = zero, node 2 = all-ones for `width`),
    /// at full factorization (default, clamps to `ConstantFold`).
    #[cfg(test)]
    fn fold_fixture(width: u32) -> (Module, NodeId, NodeId, NodeId) {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width });
        let x: NodeId = 0;
        let (zero, _) = m.intern_constant(width, 0);
        let all_ones_val: u128 = if width >= 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };
        let (ones, _) = m.intern_constant(width, all_ones_val);
        (m, x, zero, ones)
    }

    /// `x + 0 → x`. The Add gate is never created; caller gets back
    /// the `x` NodeId directly.
    #[test]
    fn fold_add_zero_collapses_to_x() {
        let (mut m, x, zero, _) = fold_fixture(8);
        let before = m.nodes.len();
        let (id, is_new) = m.intern_gate(GateOp::Add, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(id, x, "Add([x, 0]) must return x");
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before, "no new gate node created");
        assert_eq!(m.fold_identities_applied, 1);
    }

    /// `x & all_ones → x`.
    #[test]
    fn fold_and_all_ones_collapses_to_x() {
        let (mut m, x, _, ones) = fold_fixture(8);
        let before = m.nodes.len();
        let (id, is_new) = m.intern_gate(GateOp::And, vec![x, ones], 8, DepSet::from_port(0));
        assert_eq!(id, x);
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before);
        assert_eq!(m.fold_identities_applied, 1);
    }

    /// `x * 0 → 0` (absorbing). Result is the zero constant, not a
    /// fresh Mul gate.
    #[test]
    fn fold_mul_zero_absorbs() {
        let (mut m, x, zero, _) = fold_fixture(8);
        let before = m.nodes.len();
        let (id, _is_new) = m.intern_gate(GateOp::Mul, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(
            id, zero,
            "Mul with a zero operand must return the zero constant"
        );
        assert_eq!(
            m.nodes.len(),
            before,
            "no new gate; zero constant was already interned"
        );
        assert_eq!(m.fold_identities_applied, 1);
    }

    /// `x | all_ones → all_ones` (absorbing).
    #[test]
    fn fold_or_all_ones_absorbs() {
        let (mut m, x, _, ones) = fold_fixture(8);
        let (id, _is_new) = m.intern_gate(GateOp::Or, vec![x, ones], 8, DepSet::from_port(0));
        assert_eq!(id, ones);
        assert_eq!(m.fold_identities_applied, 1);
    }

    /// `x ^ 0 → x`, `x * 1 → x`, `x - 0 → x`, `x << 0 → x`,
    /// `x >> 0 → x`. Sanity sweep.
    #[test]
    fn fold_miscellaneous_identities() {
        let (mut m, x, zero, _) = fold_fixture(8);
        let (one_const, _) = m.intern_constant(8, 1);

        let (id_xor, _) = m.intern_gate(GateOp::Xor, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(id_xor, x, "Xor identity");

        let (id_mul, _) = m.intern_gate(GateOp::Mul, vec![x, one_const], 8, DepSet::from_port(0));
        assert_eq!(id_mul, x, "Mul identity");

        let (id_sub, _) = m.intern_gate(GateOp::Sub, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(id_sub, x, "Sub rhs zero");

        // Shifts use a shift-amount-width constant. The existing
        // fold_fixture zero is 8-bit; fine for this shape since
        // fold_constants checks same-width against the gate width.
        // For Shl/Shr we create a same-width zero (matches the
        // operand-width convention used by anvil for shift amounts).
        let (id_shl, _) = m.intern_gate(GateOp::Shl, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(id_shl, x, "Shl by zero");

        let (id_shr, _) = m.intern_gate(GateOp::Shr, vec![x, zero], 8, DepSet::from_port(0));
        assert_eq!(id_shr, x, "Shr by zero");

        // Each fold above is one identity application.
        assert_eq!(m.fold_identities_applied, 5);
    }

    /// At `FactorizationLevel::Commutative` the ConstantFold layer
    /// must NOT fire. A fresh Add gate is created and
    /// `fold_identities_applied` stays zero.
    #[test]
    fn fold_disabled_below_constant_fold_level() {
        use crate::config::FactorizationLevel;
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::Commutative,
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let (zero, _) = m.intern_constant(8, 0);
        let before = m.nodes.len();
        let (id, is_new) = m.intern_gate(GateOp::Add, vec![x, zero], 8, DepSet::from_port(0));
        assert_ne!(
            id, x,
            "at level=Commutative fold must not fire; Add gate should exist"
        );
        assert!(is_new, "fresh Add gate expected");
        assert_eq!(m.nodes.len(), before + 1);
        assert_eq!(m.fold_identities_applied, 0);
    }

    /// `Eq(c1, c2)` evaluated at intern time — returns a 1-bit
    /// constant. Same for `Neq` with the opposite boolean.
    #[test]
    fn peephole_constant_comparison_evaluates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c5, _) = m.intern_constant(8, 5);
        let (c7, _) = m.intern_constant(8, 7);
        let (c5b, _) = m.intern_constant(8, 5);
        // Constants dedupe, so c5 == c5b.
        assert_eq!(c5, c5b);

        // Eq of equal constants → 1.
        let (eq_eq, _) = m.intern_gate(GateOp::Eq, vec![c5, c5b], 1, DepSet::new());
        match m.nodes[eq_eq as usize] {
            Node::Constant { width: 1, value: 1 } => {}
            ref other => panic!("expected 1-bit const 1, got {other:?}"),
        }

        // Eq of unequal constants → 0.
        let (eq_neq, _) = m.intern_gate(GateOp::Eq, vec![c5, c7], 1, DepSet::new());
        match m.nodes[eq_neq as usize] {
            Node::Constant { width: 1, value: 0 } => {}
            ref other => panic!("expected 1-bit const 0, got {other:?}"),
        }

        // Lt(5, 7) → 1; Lt(7, 5) → 0.
        let (lt_57, _) = m.intern_gate(GateOp::Lt, vec![c5, c7], 1, DepSet::new());
        match m.nodes[lt_57 as usize] {
            Node::Constant { width: 1, value: 1 } => {}
            ref other => panic!("expected Lt(5,7)==1, got {other:?}"),
        }

        // Neq(5, 5) → 0.
        let (neq_eq, _) = m.intern_gate(GateOp::Neq, vec![c5, c5], 1, DepSet::new());
        match m.nodes[neq_eq as usize] {
            Node::Constant { width: 1, value: 0 } => {}
            ref other => panic!("expected Neq(5,5)==0, got {other:?}"),
        }

        assert!(m.peephole_rewrites_applied >= 4);
    }

    /// `Slice(hi, 0)` where `hi + 1 == src_width` returns the source.
    #[test]
    fn peephole_full_width_slice_identity() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let before = m.nodes.len();
        let (sliced, is_new) = m.intern_gate(
            GateOp::Slice { hi: 7, lo: 0 },
            vec![x],
            8,
            DepSet::from_port(0),
        );
        assert_eq!(sliced, x, "full-width Slice → src");
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before);
        assert_eq!(m.peephole_rewrites_applied, 1);

        // Partial slice must NOT fold.
        let (partial, is_new_partial) = m.intern_gate(
            GateOp::Slice { hi: 3, lo: 0 },
            vec![x],
            4,
            DepSet::from_port(0),
        );
        assert_ne!(partial, x, "partial Slice is a real gate");
        assert!(is_new_partial);
    }

    /// `Concat([x])` → `x` when widths match.
    #[test]
    fn peephole_single_operand_concat_identity() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let x: NodeId = 0;
        let before = m.nodes.len();
        let (result, is_new) =
            m.intern_gate(GateOp::Concat, vec![x], 4, DepSet::from_port(0));
        assert_eq!(result, x, "Concat([x]) → x");
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before);
        assert_eq!(m.peephole_rewrites_applied, 1);
    }

    /// At `FactorizationLevel::ConstantFold` the Peephole layer must
    /// NOT fire: a single-operand Concat stays as a real gate and
    /// constant comparisons stay as real Eq/Neq gates.
    #[test]
    fn peephole_disabled_below_peephole_level() {
        use crate::config::FactorizationLevel;
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::ConstantFold,
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let x: NodeId = 0;
        let before = m.nodes.len();
        let (concat, is_new) =
            m.intern_gate(GateOp::Concat, vec![x], 4, DepSet::from_port(0));
        assert_ne!(concat, x, "at level=ConstantFold peephole must not fire");
        assert!(is_new);
        assert_eq!(m.nodes.len(), before + 1);
        assert_eq!(m.peephole_rewrites_applied, 0);
    }
}
