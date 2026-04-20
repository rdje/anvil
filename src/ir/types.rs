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

const FLOP_VIRTUAL_TAG: u32 = 0x8000_0000;

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
    /// Identity mode — the coarse answer to "what does a `NodeId`
    /// mean?" See `Config::identity_mode`. `NodeId` keeps the
    /// factorization ladder live; `Relaxed` disables it and forces
    /// fresh node allocation for every AST.
    pub identity_mode: crate::config::IdentityMode,
    /// Requested factorization level — which layers of the dedup
    /// chain are active when `identity_mode == NodeId`. See
    /// `Config::factorization_level` and the
    /// `FactorizationLevel` enum (`src/config.rs`) for the
    /// ladder: `none → cse → operand-unique → commutative →
    /// associative → constant-fold → peephole → e-graph`.
    /// Default `EGraph` (theoretical ceiling); `effective()`
    /// clamps down to the highest currently-implemented layer
    /// (today `Peephole`). When `identity_mode == Relaxed`, the
    /// effective level is forced to `None`.
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
    /// post-drain exact-signature state-sharing pass. Once D-cones
    /// exist, flops with identical emitted state semantics
    /// (`width`, reset, `d`) collapse to one state element when
    /// the effective factorization level is at least `Cse`.
    /// Surfaced via `Metrics::flops_merged`.
    pub flops_merged: u32,

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
}

/// Identifier for each probability-roll knob. One variant per
/// `gen_bool(cfg.<prob>)` site across the generator. Keep
/// `Copy + Hash + Eq` cheap — the enum is keyed into a
/// `HashMap` on every roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnobId {
    /// `Config::flop_prob` — per-depth chance that a leaf is
    /// a flop block (`build_flop_leaf`).
    FlopProb,
    /// `Config::comb_mux_prob` — per-depth chance of a comb-mux
    /// block (`build_comb_mux`).
    CombMuxProb,
    /// `Config::priority_encoder_prob` — per-depth chance of a
    /// priority-encoder block.
    PriorityEncoderProb,
    /// `Config::coefficient_prob` — chance that an Add/Sub/Mul
    /// becomes a linear-combination motif.
    CoefficientProb,
    /// `Config::const_shift_amount_prob` — chance that a Shl/Shr
    /// takes a constant shift amount.
    ConstShiftAmountProb,
    /// `Config::const_comparand_prob` — chance that a comparison's
    /// RHS is a constant.
    ConstComparandProb,
    /// `Config::constant_prob` — chance that a forced leaf without a
    /// matching-width signal becomes a fresh constant instead of a
    /// width-adapter from an existing dep-bearing source.
    ConstantProb,
    /// `Config::terminal_reuse_prob` — chance that a forced leaf with
    /// a matching-width signal reuses that signal instead of emitting
    /// a fresh constant.
    TerminalReuseProb,
    /// `Config::comb_mux_encoding_prob` — encoded vs one-hot
    /// comb-mux shape.
    CombMuxEncodingProb,
    /// `Config::flop_mux_encoding_prob` — encoded vs one-hot
    /// flop-D-mux shape.
    FlopMuxEncodingProb,
    /// `Config::share_prob` — chance of a DAG-sharing fork at
    /// operand slots.
    ShareProb,
    /// `Config::flop_qfeedback_prob` — ZeroDefault vs QFeedback
    /// flop kind.
    FlopQFeedbackProb,
}

impl KnobId {
    /// Stable lowercase string key for serialisation into
    /// `Metrics`. Matches the `Config` field name, minus the
    /// `_prob` suffix where present.
    pub fn name(&self) -> &'static str {
        match self {
            KnobId::FlopProb => "flop_prob",
            KnobId::CombMuxProb => "comb_mux_prob",
            KnobId::PriorityEncoderProb => "priority_encoder_prob",
            KnobId::CoefficientProb => "coefficient_prob",
            KnobId::ConstShiftAmountProb => "const_shift_amount_prob",
            KnobId::ConstComparandProb => "const_comparand_prob",
            KnobId::ConstantProb => "constant_prob",
            KnobId::TerminalReuseProb => "terminal_reuse_prob",
            KnobId::CombMuxEncodingProb => "comb_mux_encoding_prob",
            KnobId::FlopMuxEncodingProb => "flop_mux_encoding_prob",
            KnobId::ShareProb => "share_prob",
            KnobId::FlopQFeedbackProb => "flop_qfeedback_prob",
        }
    }
}

/// Live per-knob roll counters. `record(knob, fired)` is called
/// at every probability-roll site; the empirical ratio
/// `fires[knob] / attempts[knob]` should converge to the knob
/// value as the module grows.
#[derive(Debug, Clone, Default)]
pub struct KnobRollCounters {
    pub attempts: HashMap<KnobId, u64>,
    pub fires: HashMap<KnobId, u64>,
}

impl KnobRollCounters {
    /// Record one probability roll outcome.
    ///
    /// Called by the `roll_knob(g, m, knob, prob)` helper in
    /// `src/gen/cone.rs` after every `gen_bool(cfg.<prob>)` site.
    /// Increments `attempts[knob]`, and also `fires[knob]` when
    /// the roll returned true. The empirical ratio
    /// `fires[knob] / attempts[knob]` should converge to the
    /// configured probability across a large seed sweep; the
    /// `book/src/knobs.md` "Per-knob roll-rate validation"
    /// subsection documents how to use this to verify knob
    /// effectiveness.
    pub fn record(&mut self, knob: KnobId, fired: bool) {
        *self.attempts.entry(knob).or_insert(0) += 1;
        if fired {
            *self.fires.entry(knob).or_insert(0) += 1;
        }
    }
}

impl Module {
    /// Effective factorization level after applying the coarse
    /// identity mode.
    pub fn effective_factorization_level(&self) -> crate::config::FactorizationLevel {
        match self.identity_mode {
            crate::config::IdentityMode::Relaxed => crate::config::FactorizationLevel::None,
            crate::config::IdentityMode::NodeId => self.factorization_level.effective(),
        }
    }

    /// Intern a gate expression and return its `NodeId`.
    ///
    /// This is the single choke-point through which every gate
    /// enters `m.nodes` — the whole factorization ladder (Rule
    /// 21c in `book/src/structural-rules.md`) runs here. The
    /// caller passes an operator, an operand list, the output
    /// width, and a `DepSet`; `intern_gate` returns
    /// `(NodeId, is_new)`. `is_new` is `false` when the returned
    /// id points at a pre-existing node (CSE hit, fold / peephole
    /// / flatten short-circuit, or AST-cap reuse) and `true` when
    /// a brand-new `Node::Gate` was appended. Callers use
    /// `is_new` to gate follow-up work such as `pool.add`, so a
    /// short-circuited return never double-counts a reused node.
    ///
    /// # Pipeline (in execution order)
    ///
    /// Each layer is gated on
    /// `self.effective_factorization_level()` (see
    /// [`crate::config::FactorizationLevel`]):
    ///
    /// 1. **Associative flattening** (`>= Associative`) —
    ///    [`Module::flatten_associative`] splices any same-op
    ///    same-width inner gate operand into the outer operand
    ///    list, then applies per-op semantic normalisation
    ///    (`And`/`Or` dedup, `Xor` pair-cancel, `Add`/`Mul`
    ///    skip-on-duplicates). On short-circuit (collapse to 0 or
    ///    1 operand) the helper returns `Some((id, is_new))` and
    ///    `intern_gate` returns immediately.
    /// 2. **Commutative normalisation** (`>= Commutative`) —
    ///    operands of `And`/`Or`/`Xor`/`Add`/`Mul` are sorted by
    ///    `NodeId` so `a+b` and `b+a` share identity (Rule 21b).
    /// 3. **Constant folding** (`>= ConstantFold`) —
    ///    [`Module::fold_constants`] drops identity operands
    ///    (`x + 0`, `x * 1`, `x & all_ones`, …), substitutes
    ///    absorbing constants (`x & 0`, `x | all_ones`, `x * 0`)
    ///    when no Gate operand would be orphaned, and
    ///    short-circuits the 2-arity `Sub`/`Shl`/`Shr` rhs-zero
    ///    case.
    /// 4. **Peephole rewrites** (`>= Peephole`) —
    ///    [`Module::apply_peephole`] applies local identities:
    ///    `Not(Not(x)) → x`, `Not(cmp) → inverted cmp`, all-
    ///    constant evaluation for comparisons / `Not` / `Slice` /
    ///    reductions, full-width `Slice` identity, single-operand
    ///    `Concat`.
    /// 5. **Level-None bypass** (`== None`) — every call creates
    ///    a fresh `NodeId`, no dedup. Used for stress-testing
    ///    downstream CSE in consumer tools.
    /// 6. **AST-cap + CSE dedup** (`>= Cse`) — with the final
    ///    operand list, look up `(op, operands, width)` in
    ///    `self.gate_instances`. If the cap
    ///    (`max_ast_instances`) has been hit, return the most
    ///    recent existing instance (`is_new = false`). Otherwise
    ///    append a new `Node::Gate` and register it.
    ///
    /// # Orphan safety
    ///
    /// Layers 1, 3, 4 may leave inner gates unreferenced when
    /// they short-circuit. Rule 18 (zero orphan gates) is
    /// restored by the post-construction
    /// [`crate::ir::compact::compact_node_ids`] pass at module
    /// finalisation — see `src/gen/module.rs`.
    ///
    /// # Determinism
    ///
    /// The pipeline is deterministic given the same seed:
    /// `intern_gate` is a pure function of its arguments plus
    /// `(m.nodes, m.gate_instances, m.const_instances,
    /// m.identity_mode, m.factorization_level,
    /// m.operand_duplication_rate,
    /// m.max_ast_instances)`. No RNG is consulted here.
    pub fn intern_gate(
        &mut self,
        op: GateOp,
        mut operands: Vec<NodeId>,
        width: u32,
        deps: DepSet,
    ) -> (NodeId, bool) {
        use crate::config::FactorizationLevel;

        // Associative flattening (layer Associative and above):
        // splice any same-op same-width inner gate operands into
        // this operand list, then apply the semantic normalisation
        // for the op class (dedup for `And`/`Or`, pair-cancel for
        // `Xor`, conservative for `Add`/`Mul`). See
        // `book/src/structural-rules.md` Rule 21c. Runs BEFORE
        // commutative sort so the flattened-and-normalised list is
        // the one that gets sorted.
        if self.effective_factorization_level() >= FactorizationLevel::Associative {
            if let Some((flat_id, is_new)) = self.flatten_associative(op, &mut operands, width) {
                return (flat_id, is_new);
            }
        }

        // Commutative normalization (layer Commutative and above):
        // sort operands for commutative ops so `a + b` and `b + a`
        // share identity. Disabled at lower factorization levels.
        if self.effective_factorization_level() >= FactorizationLevel::Commutative
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
        if self.effective_factorization_level() >= FactorizationLevel::ConstantFold {
            if let Some((folded_id, is_new)) = self.fold_constants(op, &mut operands, width) {
                return (folded_id, is_new);
            }
        }

        // Peephole rewrites (layer Peephole and above): apply local
        // rewrite rules that collapse specific shapes at intern time.
        // `Not(Not(x)) → x`, `Eq/Neq(const, const)` evaluated,
        // full-width `Slice → src`, single-operand `Concat → that
        // operand`. See `book/src/structural-rules.md` Rule 21c.
        if self.effective_factorization_level() >= FactorizationLevel::Peephole {
            if let Some((rewritten_id, is_new)) = self.apply_peephole(op, &operands, width) {
                return (rewritten_id, is_new);
            }
        }

        // Level = None bypasses dedup entirely: every call creates
        // a fresh NodeId. Useful for stress-testing downstream CSE.
        if self.effective_factorization_level() == FactorizationLevel::None {
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

    /// Intern a constant of the given `width` and `value` and
    /// return `(NodeId, is_new)`.
    ///
    /// Constants are keyed by `(width, value)` in
    /// `self.const_instances`. The same AST-instance cap as
    /// `intern_gate` applies: the `N+1`-th call with the same
    /// key returns the most recently created instance instead of
    /// appending a new `Node::Constant`. `max_ast_instances = 1`
    /// (the default) enforces strict constant uniqueness — every
    /// `(width, value)` pair is materialised exactly once per
    /// module.
    ///
    /// No factorization layers apply to constants directly — the
    /// only branching is the `FactorizationLevel::None` bypass
    /// which creates a fresh `NodeId` every call. This is
    /// intentional: constants have no operands, so the only
    /// "identity" question is `(width, value)`, which CSE
    /// already handles.
    ///
    /// Callers (most helpers in `src/gen/cone.rs`) wrap this in
    /// `make_constant` which also registers the returned id in
    /// the signal pool.
    pub fn intern_constant(&mut self, width: u32, value: u128) -> (NodeId, bool) {
        use crate::config::FactorizationLevel;
        if self.effective_factorization_level() == FactorizationLevel::None {
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

    /// Constant-folding layer of the factorization ladder (Layer 5,
    /// `FactorizationLevel::ConstantFold`). Applies algebraic
    /// identities in place on `operands`, possibly short-circuiting
    /// the enclosing `intern_gate` call.
    ///
    /// Called by [`Module::intern_gate`] after associative
    /// flattening and commutative sort, before peephole rewrites
    /// and the dedup-table lookup.
    ///
    /// # Returns
    ///
    /// - `Some((id, is_new))` — the whole gate folded to a single
    ///   `NodeId`: either a surviving operand (identity drops
    ///   emptied the list down to one), an identity constant
    ///   (identity drops emptied the list completely), or an
    ///   absorbing constant (zero for `Mul`/`And`, all-ones for
    ///   `Or`). `is_new` follows the convention of
    ///   [`Module::intern_constant`].
    /// - `None` — no fold applied, **or** one or more operands
    ///   were dropped but ≥ 2 remain. The caller proceeds with
    ///   `intern_gate`'s normal dedup path on the (possibly
    ///   shrunken) list.
    ///
    /// # Rules implemented
    ///
    /// ## Associative ops (`And`/`Or`/`Xor`/`Add`/`Mul`)
    ///
    /// | Op              | All-const evaluation                                   | Identity drop        | Absorbing                                 |
    /// |-----------------|--------------------------------------------------------|----------------------|-------------------------------------------|
    /// | `And`           | bitwise AND over values                                | drop `all_ones`      | `0`                                       |
    /// | `Or`            | bitwise OR over values                                 | drop `0`             | `all_ones`                                |
    /// | `Xor`           | bitwise XOR over values                                | drop `0`             | —                                         |
    /// | `Add`           | sum over values, mod 2^width                           | drop `0`             | —                                         |
    /// | `Mul`           | product over values, mod 2^width                       | drop `1`             | `0`                                       |
    ///
    /// All-const evaluation fires when every operand is a
    /// `Node::Constant` of `width`; it supersedes the absorbing
    /// and identity-drop paths for that case. Mixed operand lists
    /// (e.g. one constant + one `Node::PrimaryInput`) still reach
    /// the absorbing / identity-drop paths.
    ///
    /// ## Non-commutative 2-arity ops
    ///
    /// | Op    | All-const evaluation                              | Rhs-zero identity |
    /// |-------|---------------------------------------------------|-------------------|
    /// | `Sub` | `(lhs - rhs) mod 2^width`                         | `a - 0 → a`       |
    /// | `Shl` | `(lhs << rhs) mod 2^width` (over-shift → 0)       | `a << 0 → a`      |
    /// | `Shr` | `lhs >> rhs` (over-shift → 0)                     | `a >> 0 → a`      |
    ///
    /// # Orphan safety
    ///
    /// Absorbing (`x & 0 → 0`, etc.) can orphan the non-constant
    /// operand sub-tree, but module finalisation now runs
    /// [`crate::ir::compact::compact_node_ids`], so those dead
    /// gates are removed before emission. That makes mixed dynamic
    /// absorbing safe again at intern time.
    ///
    /// # Non-commutative ops
    ///
    /// `Sub`/`Shl`/`Shr` are strictly 2-arity and position-
    /// sensitive. Only the rhs-zero case folds (`a - 0 = a`,
    /// `a << 0 = a`, `a >> 0 = a`). The lhs-zero cases (`0 - a`,
    /// `0 << a`, `0 >> a`) are NOT identities and are not
    /// folded.
    ///
    /// # Out of scope
    ///
    /// Comparison ops, reductions, `Not`, `Slice`, `Concat`, and
    /// `Mux` are not handled here — they belong to
    /// [`Module::apply_peephole`] (Layer 6).
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
            // All-constant evaluation: `Sub(c1, c2)`, `Shl(c1, c2)`,
            // `Shr(c1, c2)`. For Shl/Shr the rhs (shift amount)
            // constant may be narrower than `width` — we still
            // accept it as long as its own width matches its own
            // Node::Constant entry. Sub requires both to be of
            // `width`.
            let lhs_const = const_of(operands[0], &self.nodes);
            let rhs_const_any_width: Option<u128> = match &self.nodes[operands[1] as usize] {
                Node::Constant { value, .. } => Some(*value),
                _ => None,
            };
            if let (Some(lhs), Some(rhs)) = (lhs_const, rhs_const_any_width) {
                let result = match op {
                    GateOp::Sub => lhs.wrapping_sub(rhs) & all_ones,
                    GateOp::Shl => {
                        if rhs >= u128::from(width) {
                            0
                        } else {
                            lhs.wrapping_shl(rhs as u32) & all_ones
                        }
                    }
                    GateOp::Shr => {
                        if rhs >= u128::from(width) {
                            0
                        } else {
                            (lhs >> rhs) & all_ones
                        }
                    }
                    _ => unreachable!(),
                };
                self.fold_identities_applied += 1;
                let (cid, is_new) = self.intern_constant(width, result);
                crate::trace_verbose!(
                    node = cid,
                    ?op,
                    width,
                    value = result,
                    "✂️ fold_constants all-const Sub/Shl/Shr"
                );
                return Some((cid, is_new));
            }

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

        // 0. All-constant evaluation: if every operand is a same-
        //    width constant, evaluate the expression directly and
        //    return the result constant. Subsumes the absorbing
        //    path and identity-drop path for the all-const subcase
        //    (e.g. `Add(3, 5)` → 8, `Mul(4, 0)` → 0). Orphan-safe
        //    because every operand is a Constant, which doesn't
        //    count as a Gate orphan.
        let all_const_values: Option<Vec<u128>> = operands
            .iter()
            .map(|id| const_of(*id, &self.nodes))
            .collect();
        if let Some(values) = all_const_values {
            let result = match op {
                GateOp::And => values.iter().copied().fold(all_ones, |acc, v| acc & v),
                GateOp::Or => values.iter().copied().fold(0u128, |acc, v| acc | v),
                GateOp::Xor => values.iter().copied().fold(0u128, |acc, v| acc ^ v),
                GateOp::Add => values
                    .iter()
                    .copied()
                    .fold(0u128, |acc, v| acc.wrapping_add(v) & all_ones),
                GateOp::Mul => values
                    .iter()
                    .copied()
                    .fold(1u128, |acc, v| acc.wrapping_mul(v) & all_ones),
                _ => unreachable!(),
            };
            self.fold_identities_applied += 1;
            let (cid, is_new) = self.intern_constant(width, result);
            crate::trace_verbose!(
                node = cid,
                ?op,
                width,
                value = result,
                "✂️ fold_constants all-const associative evaluation"
            );
            return Some((cid, is_new));
        }

        // 1. Absorbing elements: `Mul`/`And` zero, `Or` all-ones.
        // Dead gate operands are cleaned up later by compaction.
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

        // 2. Identity elements: drop in place.
        let identity: u128 = match op {
            GateOp::Add | GateOp::Xor | GateOp::Or => 0,
            GateOp::Mul => 1,
            GateOp::And => all_ones,
            _ => return None,
        };
        operands.retain(|id| {
            !matches!(
                &self.nodes[*id as usize],
                Node::Constant { width: w, value } if *w == width && *value == identity
            )
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

    /// Associative flattening layer of the factorization ladder
    /// (Layer 4, `FactorizationLevel::Associative`). Splices any
    /// same-op same-width inner gate operands into this operand
    /// list, then applies per-op semantic normalisation.
    ///
    /// Called by [`Module::intern_gate`] BEFORE commutative sort,
    /// so the flattened-and-normalised operand list is what the
    /// sort sees.
    ///
    /// # Returns
    ///
    /// - `Some((id, is_new))` — the whole gate collapses to a
    ///   single `NodeId`: a surviving operand (post-normalisation
    ///   the list has exactly one element) or an identity
    ///   constant (only reachable for `Xor`-all-cancel → 0). The
    ///   enclosing `intern_gate` returns this directly without
    ///   ever materialising the outer gate.
    /// - `None` — no flattening applied **or** `operands` was
    ///   rewritten in place (≥ 2 operands remain post-
    ///   normalisation). The caller proceeds with the new list.
    ///
    /// ## Per-op semantic normalisation after flattening
    ///
    /// - **`And` / `Or`** — idempotent, so duplicate operands
    ///   produced by splicing are simply deduplicated
    ///   (`a & a = a`, `a | a = a`). The resulting list has
    ///   only distinct operands, preserving Rule 8 extended.
    /// - **`Xor`** — self-inverse, so duplicates pair-cancel
    ///   (`a ^ a = 0`). The semantics are to count occurrences,
    ///   drop even-count operands entirely, and keep exactly
    ///   one copy of each odd-count operand. If every operand
    ///   cancels, the result is the zero constant.
    /// - **`Add` / `Mul`** — duplicates have semantic weight
    ///   (`x + x = 2x`, `x * x = x²`), so at the default strict
    ///   `operand_duplication_rate < 1.0` we **do not flatten**
    ///   when flattening would produce duplicates. This
    ///   preserves both the construction-time uniqueness rule
    ///   and correctness: we never silently change `x + (x + y)`
    ///   into `x + y`. At `operand_duplication_rate == 1.0` the
    ///   user has opted into duplicate operands; we flatten
    ///   unconditionally.
    ///
    /// Orphan-safety: flattening can leave the inner gate
    /// unreferenced (its only holder was this caller's operand
    /// slot, now spliced out). That's cleaned up at module
    /// finalisation by `compact_node_ids` — see
    /// `src/ir/compact.rs`.
    fn flatten_associative(
        &mut self,
        op: GateOp,
        operands: &mut Vec<NodeId>,
        width: u32,
    ) -> Option<(NodeId, bool)> {
        use std::collections::{HashMap, HashSet};

        if !matches!(
            op,
            GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
        ) {
            return None;
        }

        // 1. Splice same-op same-width inner gates.
        let mut flat: Vec<NodeId> = Vec::with_capacity(operands.len());
        let mut any_spliced = false;
        for &id in operands.iter() {
            if let Node::Gate {
                op: inner_op,
                operands: inner,
                width: inner_w,
                ..
            } = &self.nodes[id as usize]
            {
                if *inner_op == op && *inner_w == width {
                    flat.extend(inner.iter().copied());
                    any_spliced = true;
                    continue;
                }
            }
            flat.push(id);
        }

        if !any_spliced {
            return None;
        }

        // 2. Per-op semantic normalisation.
        match op {
            GateOp::And | GateOp::Or => {
                let mut seen = HashSet::new();
                flat.retain(|id| seen.insert(*id));
            }
            GateOp::Xor => {
                // Pair-cancel: count occurrences, keep odd-count
                // operands with multiplicity 1.
                let mut counts: HashMap<NodeId, u32> = HashMap::new();
                for id in &flat {
                    *counts.entry(*id).or_insert(0) += 1;
                }
                flat.retain(|id| counts[id] % 2 == 1);
                let mut seen = HashSet::new();
                flat.retain(|id| seen.insert(*id));
            }
            GateOp::Add | GateOp::Mul => {
                // At strict operand_duplication_rate, duplicates
                // are forbidden AND semantically meaningful. Skip
                // the flatten when the flat list would have
                // duplicates — preserve the pre-existing nested
                // shape rather than silently changing semantics.
                if self.operand_duplication_rate < 1.0 {
                    let mut counts: HashMap<NodeId, u32> = HashMap::new();
                    for id in &flat {
                        *counts.entry(*id).or_insert(0) += 1;
                    }
                    if counts.values().any(|v| *v > 1) {
                        return None;
                    }
                }
                // Otherwise keep `flat` as-is (rate == 1.0, or no
                // duplicates arose).
            }
            _ => unreachable!(),
        }

        self.flatten_associative_applied += 1;
        crate::trace_verbose!(
            ?op,
            width,
            n_flat = flat.len(),
            "🪗 flatten_associative spliced"
        );

        // 3. Short-circuit when the normalised list is short.
        let all_ones: u128 = if width >= 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };
        match flat.len() {
            0 => {
                // Only reachable for Xor (all cancel) → 0.
                debug_assert!(matches!(op, GateOp::Xor));
                let (cid, is_new) = self.intern_constant(width, 0);
                Some((cid, is_new))
            }
            1 => {
                // Single surviving operand — return directly.
                Some((flat[0], false))
            }
            _ => {
                // And/Or: full `all_ones` absorbing case doesn't
                // apply here (no constants injected by flatten);
                // that's ConstantFold's job if the outer op had a
                // constant. Xor with all-different-odd-count
                // operands is fine. Add/Mul preserved non-dup.
                let _ = all_ones; // silence unused in some paths.
                *operands = flat;
                None
            }
        }
    }

    /// Peephole-rewrite layer of the factorization ladder
    /// (Layer 6, `FactorizationLevel::Peephole`). Applies local
    /// rewrite rules keyed on `op` and operand shapes.
    ///
    /// Called by [`Module::intern_gate`] AFTER constant folding
    /// and BEFORE the dedup-table lookup. Every rule either
    /// short-circuits to an existing node (no new gate
    /// materialised) or recursively calls `self.intern_gate` to
    /// produce a rewritten gate; orphaned intermediate gates are
    /// cleaned up by the post-construction compaction pass
    /// [`crate::ir::compact::compact_node_ids`], so Rule 18 holds
    /// at module finalisation.
    ///
    /// # Returns
    ///
    /// - `Some((id, is_new))` — a rule matched and the outer
    ///   gate is short-circuited to `id`. `is_new` reflects
    ///   whether a brand-new node was created by this call
    ///   (relevant for absorbing / constant-evaluation rules
    ///   that mint a new `Node::Constant`).
    /// - `None` — no rule matched; `intern_gate` proceeds to
    ///   the dedup-table lookup unchanged.
    ///
    /// # Rules
    ///
    /// Rules are grouped by the outer operator:
    ///
    /// ### `Not(operand)` (unary, 1 operand)
    ///
    /// 1. **Constant evaluation**: `Not(c)` → `~c & mask(width)`.
    /// 2. **Involutive collapse**: `Not(Not(x)) → x`. The inner
    ///    `Not` may become orphaned.
    /// 3. **Comparison inversion** (cross-gate peephole):
    ///    `Not(Eq(a, b)) → Neq(a, b)` and symmetric flips for
    ///    `Neq`/`Lt`/`Gt`/`Le`/`Ge`. The inverted comparison is
    ///    interned through the full pipeline, so it picks up
    ///    CSE / constant folding. The inner comparison gate may
    ///    become orphaned.
    ///
    /// ### Comparison ops: `Eq`/`Neq`/`Lt`/`Gt`/`Le`/`Ge`
    ///
    /// 4. **Both-operands-constant evaluation**: if both operands
    ///    are same-width constants, evaluate the comparison and
    ///    return a 1-bit constant. The IR contract guarantees
    ///    matching operand widths; mismatched widths defensively
    ///    skip the fold.
    /// 5. **Unsigned boundary tautologies**: for unsigned same-width
    ///    comparisons against min/max constants, fold the obvious
    ///    truths and falsehoods:
    ///    - `x < 0 → 0`, `x >= 0 → 1`
    ///    - `x <= all_ones → 1`, `x > all_ones → 0`
    ///    - `0 > x → 0`, `0 <= x → 1`
    ///    - `all_ones < x → 0`, `all_ones >= x → 1`
    ///
    /// ### `Mux(sel, a, b)` (3 operands)
    ///
    /// 6. **Constant-selector collapse**: `Mux(0, a, b) → b`,
    ///    `Mux(1, a, b) → a`.
    ///
    /// ### `Slice { hi, lo }(operand)` (1 operand)
    ///
    /// 7. **Full-width slice identity**: `Slice(hi, 0)(src)` with
    ///    `hi + 1 == src_width` returns `src`.
    /// 8. **Constant-operand evaluation**: `Slice(hi, lo)(c)` →
    ///    `(c >> lo) & mask(hi - lo + 1)`.
    ///
    /// ### `Concat(operands)` (1 or more operands)
    ///
    /// 9. **Single-operand identity**: `Concat([x])` → `x` when
    ///    `x.width == width`.
    /// 10. **All-constant bit assembly** (`operands.len() >= 2`):
    ///     every operand is a constant → pack MSB-first into one
    ///     constant (matches the SV emit convention in
    ///     `src/emit/sv.rs` where `{a, b, c}` places `a` in the
    ///     high bits). Widths must sum to the gate width; mismatch
    ///     defensively skips the fold.
    ///
    /// ### Reductions: `RedAnd`/`RedOr`/`RedXor` (1 operand)
    ///
    /// 11. **Constant-operand evaluation**:
    ///    - `RedAnd(c)` → `(c == all_ones(src_width)) as 1-bit`
    ///    - `RedOr(c)` → `(c != 0) as 1-bit`
    ///    - `RedXor(c)` → `popcount(c) & 1` as 1-bit
    ///
    /// # Design principle
    ///
    /// Each rule is an unambiguous local identity with no
    /// width-reinterpretation or type punning. Broader cross-
    /// gate rewrites like `(a + b) - b → a` or
    /// `(a & b) | (a & !b) → a` require symbolic reasoning over
    /// the expression tree (the e-graph problem) and are not
    /// implemented here — they belong to the aspirational
    /// `EGraph` layer at the top of `FactorizationLevel`.
    ///
    /// # Counter
    ///
    /// Every fire increments `self.peephole_rewrites_applied`,
    /// which `Metrics::peephole_rewrites_applied` exposes. Use
    /// this to verify knob-sweep behaviour or detect
    /// regressions.
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
            GateOp::Not if operands.len() == 1 => {
                // Constant-operand evaluation: `Not(c)` → `~c & mask(width)`.
                // Handled first because the operand is a Constant
                // rather than a Gate.
                if let Some((src_w, src_val)) = const_of(operands[0], &self.nodes) {
                    debug_assert_eq!(src_w, width);
                    let mask: u128 = if width >= 128 {
                        u128::MAX
                    } else {
                        (1u128 << width) - 1
                    };
                    let result = !src_val & mask;
                    self.peephole_rewrites_applied += 1;
                    let (cid, is_new) = self.intern_constant(width, result);
                    crate::trace_verbose!(
                        node = cid,
                        width,
                        value = result,
                        "✂️ peephole Not of constant"
                    );
                    return Some((cid, is_new));
                }

                // Inspect the single operand for collapsible shapes.
                // Clone fields we need — holding a borrow across the
                // recursive `intern_gate` call would alias self.
                let inner = match &self.nodes[operands[0] as usize] {
                    Node::Gate {
                        op: inner_op,
                        operands: inner_ops,
                        width: inner_w,
                        deps: inner_deps,
                    } => Some((*inner_op, inner_ops.clone(), *inner_w, inner_deps.clone())),
                    _ => None,
                };
                let (inner_op, inner_ops, inner_w, inner_deps) = inner?;

                // Involutive: Not(Not(x)) → x. Inner Not may be
                // orphaned; compact_node_ids cleans it up post-
                // construction.
                if inner_op == GateOp::Not && inner_w == width && inner_ops.len() == 1 {
                    let x = inner_ops[0];
                    self.peephole_rewrites_applied += 1;
                    crate::trace_verbose!(node = x, width, "✂️ peephole Not(Not(x)) → x");
                    return Some((x, false));
                }

                // Comparison inversion (cross-gate peephole):
                // `Not(cmp(a, b)) → inverted_cmp(a, b)`. Width
                // invariant: Not preserves width, comparisons are
                // always 1-bit, so both outer Not width and inner
                // comparison width must be 1 for this to apply.
                // The inner comparison gate is left unreferenced
                // (only the outer Not, now collapsed, held it);
                // compact_node_ids cleans it up.
                let inverted = match inner_op {
                    GateOp::Eq => Some(GateOp::Neq),
                    GateOp::Neq => Some(GateOp::Eq),
                    GateOp::Lt => Some(GateOp::Ge),
                    GateOp::Gt => Some(GateOp::Le),
                    GateOp::Le => Some(GateOp::Gt),
                    GateOp::Ge => Some(GateOp::Lt),
                    _ => None,
                };
                if let Some(flipped) = inverted {
                    if inner_w == 1 && width == 1 && inner_ops.len() == 2 {
                        self.peephole_rewrites_applied += 1;
                        crate::trace_verbose!(
                            ?inner_op,
                            ?flipped,
                            "✂️ peephole Not(cmp) → inverted cmp"
                        );
                        // Intern the inverted comparison through the
                        // normal pipeline so it participates in CSE
                        // / constant folding if the operands are
                        // constants. The result takes the outer
                        // Not's return slot directly.
                        return Some(self.intern_gate(flipped, inner_ops, 1, inner_deps));
                    }
                }

                None
            }
            GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
                if operands.len() == 2 =>
            {
                let operand_const = |id: NodeId| const_of(id, &self.nodes);
                let boundary_fold = match (operand_const(operands[0]), operand_const(operands[1])) {
                    (Some((lhs_w, lhs_v)), None) => {
                        let lhs_max = if lhs_w >= 128 {
                            u128::MAX
                        } else {
                            (1u128 << lhs_w) - 1
                        };
                        match op {
                            GateOp::Gt if lhs_v == 0 => Some(0),
                            GateOp::Le if lhs_v == 0 => Some(1),
                            GateOp::Lt if lhs_v == lhs_max => Some(0),
                            GateOp::Ge if lhs_v == lhs_max => Some(1),
                            _ => None,
                        }
                    }
                    (None, Some((rhs_w, rhs_v))) => {
                        let rhs_max = if rhs_w >= 128 {
                            u128::MAX
                        } else {
                            (1u128 << rhs_w) - 1
                        };
                        match op {
                            GateOp::Lt if rhs_v == 0 => Some(0),
                            GateOp::Ge if rhs_v == 0 => Some(1),
                            GateOp::Le if rhs_v == rhs_max => Some(1),
                            GateOp::Gt if rhs_v == rhs_max => Some(0),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(result) = boundary_fold {
                    self.peephole_rewrites_applied += 1;
                    let (cid, is_new) = self.intern_constant(width, result);
                    crate::trace_verbose!(
                        node = cid,
                        ?op,
                        width,
                        value = result,
                        "✂️ peephole unsigned comparison boundary"
                    );
                    return Some((cid, is_new));
                }

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
            GateOp::Mux if operands.len() == 3 => {
                let (sel_w, sel_v) = const_of(operands[0], &self.nodes)?;
                if sel_w != 1 {
                    return None;
                }
                let chosen = match sel_v {
                    0 => operands[2],
                    1 => operands[1],
                    _ => return None,
                };
                self.peephole_rewrites_applied += 1;
                crate::trace_verbose!(
                    node = chosen,
                    width,
                    sel = sel_v,
                    "✂️ peephole constant-selector Mux"
                );
                Some((chosen, false))
            }
            GateOp::Slice { hi, lo } if operands.len() == 1 => {
                // Full-width slice starting at 0 is the identity.
                let src_w = self.nodes[operands[0] as usize].width();
                if lo == 0 && hi + 1 == src_w && hi - lo + 1 == width {
                    let x = operands[0];
                    self.peephole_rewrites_applied += 1;
                    crate::trace_verbose!(node = x, width, "✂️ peephole full-width Slice → src");
                    return Some((x, false));
                }
                // All-constant: evaluate the slice at intern time.
                if let Some((_src_w, src_val)) = const_of(operands[0], &self.nodes) {
                    let slice_width = hi - lo + 1;
                    debug_assert_eq!(slice_width, width);
                    let mask: u128 = if slice_width >= 128 {
                        u128::MAX
                    } else {
                        (1u128 << slice_width) - 1
                    };
                    let result = (src_val >> lo) & mask;
                    self.peephole_rewrites_applied += 1;
                    let (cid, is_new) = self.intern_constant(width, result);
                    crate::trace_verbose!(
                        node = cid,
                        width,
                        value = result,
                        "✂️ peephole Slice of constant"
                    );
                    return Some((cid, is_new));
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
            GateOp::Concat if operands.len() >= 2 => {
                // All-constant bit assembly: every operand is a
                // constant → pack their bits MSB-first (matching
                // the SV emit convention in `src/emit/sv.rs`:
                // `{c1, c2, c3}` has `c1` in the high bits).
                //
                // For each operand in order (MSB first), shift the
                // accumulator left by that operand's width and OR
                // in the operand's value masked to its own width.
                let mut pieces: Vec<(u32, u128)> = Vec::with_capacity(operands.len());
                for &id in operands.iter() {
                    match &self.nodes[id as usize] {
                        Node::Constant { width: w, value } => pieces.push((*w, *value)),
                        _ => return None,
                    }
                }
                // Width sanity: sum of operand widths must equal
                // the gate width. If it doesn't, this is an
                // upstream bug — bail defensively rather than
                // emit a wrong-width constant.
                let total: u32 = pieces.iter().map(|(w, _)| *w).sum();
                if total != width {
                    return None;
                }
                let mut result: u128 = 0;
                for (w, v) in pieces {
                    let mask: u128 = if w >= 128 {
                        u128::MAX
                    } else {
                        (1u128 << w) - 1
                    };
                    result = if w >= 128 {
                        v & mask
                    } else {
                        (result << w) | (v & mask)
                    };
                }
                // Final mask to gate width (paranoia; should be a
                // no-op if the per-piece masks held).
                let gate_mask: u128 = if width >= 128 {
                    u128::MAX
                } else {
                    (1u128 << width) - 1
                };
                result &= gate_mask;
                self.peephole_rewrites_applied += 1;
                let (cid, is_new) = self.intern_constant(width, result);
                crate::trace_verbose!(
                    node = cid,
                    width,
                    value = result,
                    "✂️ peephole Concat of constants"
                );
                Some((cid, is_new))
            }
            // Reductions: all-const operand evaluates to a 1-bit
            // constant. Reduction output width is always 1.
            GateOp::RedAnd | GateOp::RedOr | GateOp::RedXor if operands.len() == 1 => {
                let (src_w, src_val) = const_of(operands[0], &self.nodes)?;
                let src_all_ones: u128 = if src_w >= 128 {
                    u128::MAX
                } else {
                    (1u128 << src_w) - 1
                };
                let result: u128 = match op {
                    GateOp::RedAnd => (src_val == src_all_ones) as u128,
                    GateOp::RedOr => (src_val != 0) as u128,
                    GateOp::RedXor => (src_val.count_ones() & 1) as u128,
                    _ => unreachable!(),
                };
                self.peephole_rewrites_applied += 1;
                let (cid, is_new) = self.intern_constant(width, result);
                crate::trace_verbose!(
                    node = cid,
                    ?op,
                    width,
                    value = result,
                    "✂️ peephole reduction of constant"
                );
                Some((cid, is_new))
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
        s.insert(FLOP_VIRTUAL_TAG | flop);
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

    /// Rewrite virtual flop ids after a flop merge / renumbering
    /// pass. Primary-input deps are left untouched; virtual flop
    /// deps are remapped through the provided old-id → new-id
    /// table and deduplicated naturally by the set.
    pub(crate) fn remap_flop_virtuals(&mut self, old_to_new: &[FlopId]) {
        let mut next = BTreeSet::new();
        for id in self.set.iter().copied() {
            if (id & FLOP_VIRTUAL_TAG) != 0 {
                let old = (id & !FLOP_VIRTUAL_TAG) as usize;
                let new = old_to_new
                    .get(old)
                    .copied()
                    .unwrap_or(id & !FLOP_VIRTUAL_TAG);
                next.insert(FLOP_VIRTUAL_TAG | new);
            } else {
                next.insert(id);
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
            FactorizationLevel::Peephole
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
    /// at the default factorization request (`e-graph`, which
    /// currently clamps to `Peephole`).
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

    /// Absorbing now fires even when the non-constant operand is a
    /// Gate, because final compaction removes the dead gate later.
    #[test]
    fn fold_or_all_ones_absorbs_with_gate_operand() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let (ones, _) = m.intern_constant(8, 0xFF);
        let (not_x, _) = m.intern_gate(GateOp::Not, vec![x], 8, DepSet::from_port(0));
        let before = m.nodes.len();
        let (id, is_new) = m.intern_gate(GateOp::Or, vec![not_x, ones], 8, DepSet::from_port(0));
        assert_eq!(id, ones);
        assert!(
            !is_new,
            "absorbing should reuse the interned all-ones constant"
        );
        assert_eq!(m.nodes.len(), before, "outer Or should not materialize");
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

    /// Unsigned comparisons against 0 / all-ones fold at the obvious
    /// boundaries without needing full range analysis.
    #[test]
    fn peephole_unsigned_boundary_comparisons_fold() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let x: NodeId = 0;
        let (zero, _) = m.intern_constant(4, 0);
        let (max, _) = m.intern_constant(4, 0xF);
        let cases = [
            (GateOp::Lt, vec![x, zero], 0),
            (GateOp::Ge, vec![x, zero], 1),
            (GateOp::Le, vec![x, max], 1),
            (GateOp::Gt, vec![x, max], 0),
            (GateOp::Gt, vec![zero, x], 0),
            (GateOp::Le, vec![zero, x], 1),
            (GateOp::Lt, vec![max, x], 0),
            (GateOp::Ge, vec![max, x], 1),
        ];

        for (op, operands, expected) in cases {
            let (id, _) = m.intern_gate(op, operands, 1, DepSet::from_port(0));
            match m.nodes[id as usize] {
                Node::Constant { width: 1, value } => assert_eq!(value, expected),
                ref other => panic!("expected 1-bit const {expected}, got {other:?}"),
            }
        }

        assert_eq!(m.peephole_rewrites_applied, 8);
    }

    /// `Mux(0, a, b) → b` and `Mux(1, a, b) → a`.
    #[test]
    fn peephole_mux_const_selector_collapses() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 4 });
        let a: NodeId = 0;
        let b: NodeId = 1;
        let (sel0, _) = m.intern_constant(1, 0);
        let (sel1, _) = m.intern_constant(1, 1);
        let deps = DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]);
        let before = m.nodes.len();

        let (pick_b, is_new_b) = m.intern_gate(GateOp::Mux, vec![sel0, a, b], 4, deps.clone());
        assert_eq!(pick_b, b);
        assert!(!is_new_b);

        let (pick_a, is_new_a) = m.intern_gate(GateOp::Mux, vec![sel1, a, b], 4, deps);
        assert_eq!(pick_a, a);
        assert!(!is_new_a);

        assert_eq!(
            m.nodes.len(),
            before,
            "constant-selector muxes should not materialize"
        );
        assert_eq!(m.peephole_rewrites_applied, 2);
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
        let (result, is_new) = m.intern_gate(GateOp::Concat, vec![x], 4, DepSet::from_port(0));
        assert_eq!(result, x, "Concat([x]) → x");
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before);
        assert_eq!(m.peephole_rewrites_applied, 1);
    }

    /// `Not(Not(x)) → x` at intern time. The inner `Not` is
    /// materialised; the outer `Not` short-circuits to `x` and
    /// leaves the inner as an orphan (compact_node_ids cleans it
    /// up at module finalisation, not tested here).
    #[test]
    fn peephole_double_not_collapses_with_inner_orphaned() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        let x: NodeId = 0;
        let (inner, _) = m.intern_gate(GateOp::Not, vec![x], 1, DepSet::from_port(0));
        assert_ne!(inner, x, "inner Not is a real gate");
        let before = m.nodes.len();
        let (outer, is_new) = m.intern_gate(GateOp::Not, vec![inner], 1, DepSet::from_port(0));
        assert_eq!(outer, x, "Not(Not(x)) must return x");
        assert!(!is_new);
        assert_eq!(m.nodes.len(), before, "no new gate created by outer Not");
        assert_eq!(m.peephole_rewrites_applied, 1);
        // The inner Not is now orphaned — it exists at m.nodes[inner]
        // but is referenced by no holder. `compact_node_ids` cleans
        // it up at module finalisation.
    }

    /// `Not(Eq(a, b))` rewrites to `Neq(a, b)` at intern time.
    /// The inner Eq becomes orphaned (only the outer Not, now
    /// collapsed, held it); compact_node_ids handles the orphan.
    #[test]
    fn peephole_not_eq_becomes_neq() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;
        let (inner, _) = m.intern_gate(GateOp::Eq, vec![a, b], 1, DepSet::from_port(0));
        let (outer, _) = m.intern_gate(GateOp::Not, vec![inner], 1, DepSet::from_port(0));
        match &m.nodes[outer as usize] {
            Node::Gate {
                op: GateOp::Neq,
                operands,
                width: 1,
                ..
            } => {
                let mut ids = operands.clone();
                ids.sort_unstable();
                assert_eq!(ids, vec![a, b], "Neq must carry Eq's operands");
            }
            other => panic!("expected Neq gate, got {other:?}"),
        }
        assert_eq!(m.peephole_rewrites_applied, 1);
    }

    /// `Not(Neq)` / `Not(Lt)` / `Not(Gt)` / `Not(Le)` / `Not(Ge)`
    /// all invert to their complementary comparison.
    #[test]
    fn peephole_not_comparison_inversions() {
        let cases = [
            (GateOp::Neq, GateOp::Eq),
            (GateOp::Lt, GateOp::Ge),
            (GateOp::Gt, GateOp::Le),
            (GateOp::Le, GateOp::Gt),
            (GateOp::Ge, GateOp::Lt),
        ];
        for (inner_op, expected_outer_op) in cases {
            let mut m = Module {
                max_ast_instances: 1,
                factorization_level: crate::config::FactorizationLevel::default(),
                ..Module::default()
            };
            m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
            m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
            let a: NodeId = 0;
            let b: NodeId = 1;
            let (inner, _) = m.intern_gate(inner_op, vec![a, b], 1, DepSet::from_port(0));
            let (outer, _) = m.intern_gate(GateOp::Not, vec![inner], 1, DepSet::from_port(0));
            match &m.nodes[outer as usize] {
                Node::Gate { op, .. } if *op == expected_outer_op => {}
                other => panic!(
                    "Not({inner_op:?}) should rewrite to {expected_outer_op:?}, got {other:?}"
                ),
            }
        }
    }

    /// `Not(const)` folds to `~const & mask(width)` at intern time.
    #[test]
    fn peephole_not_of_constant_folds() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        // 8-bit constant 0x5A → Not → 0xA5.
        let (c, _) = m.intern_constant(8, 0x5A);
        let (result, _) = m.intern_gate(GateOp::Not, vec![c], 8, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant { width: 8, value } => assert_eq!(*value, 0xA5),
            other => panic!("expected 8-bit const 0xA5, got {other:?}"),
        }
        assert_eq!(m.peephole_rewrites_applied, 1);
    }

    /// `Not(Eq(c1, c2))` with both operands constants: the inner Eq
    /// folds to a 1-bit constant at intern time (via const comparison
    /// peephole), and the outer Not on that constant then folds to
    /// the complementary 1-bit const (via Not-of-constant peephole
    /// landed in this slice). End-to-end: `Not(Eq(5, 7)) → 1'b1`.
    #[test]
    fn peephole_not_eq_of_constants_folds_end_to_end() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c5, _) = m.intern_constant(8, 5);
        let (c7, _) = m.intern_constant(8, 7);
        let (eq, _) = m.intern_gate(GateOp::Eq, vec![c5, c7], 1, DepSet::new());
        // Eq(5, 7) folds to 1-bit const 0.
        match &m.nodes[eq as usize] {
            Node::Constant { width: 1, value: 0 } => {}
            other => panic!("Eq(5,7) should fold to 1'b0, got {other:?}"),
        }
        // Not on that folded constant now also folds.
        let (not_eq, _) = m.intern_gate(GateOp::Not, vec![eq], 1, DepSet::new());
        match &m.nodes[not_eq as usize] {
            Node::Constant { width: 1, value: 1 } => {}
            other => panic!("Not(Eq(5,7)) should fold to 1'b1, got {other:?}"),
        }
    }

    /// `Slice(hi, lo)(const)` folds to the sliced constant.
    #[test]
    fn peephole_slice_of_constant_folds() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        // 8-bit constant 0xAB = 0b10101011. Slice(5, 2) → 0b1010 = 10.
        let (c, _) = m.intern_constant(8, 0xAB);
        let (result, _) = m.intern_gate(GateOp::Slice { hi: 5, lo: 2 }, vec![c], 4, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant { width: 4, value } => assert_eq!(*value, 0b1010),
            other => panic!("expected 4-bit const 10, got {other:?}"),
        }
    }

    /// Reductions on constants fold to the appropriate 1-bit
    /// result.
    #[test]
    fn peephole_reductions_of_constants_fold() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c_all_ones, _) = m.intern_constant(4, 0b1111);
        let (c_mixed, _) = m.intern_constant(4, 0b1010);
        let (c_zero, _) = m.intern_constant(4, 0);

        // RedAnd(all-ones) = 1, RedAnd(mixed) = 0.
        let (ra1, _) = m.intern_gate(GateOp::RedAnd, vec![c_all_ones], 1, DepSet::new());
        assert!(matches!(
            m.nodes[ra1 as usize],
            Node::Constant { value: 1, .. }
        ));
        let (ra2, _) = m.intern_gate(GateOp::RedAnd, vec![c_mixed], 1, DepSet::new());
        assert!(matches!(
            m.nodes[ra2 as usize],
            Node::Constant { value: 0, .. }
        ));

        // RedOr(zero) = 0, RedOr(mixed) = 1.
        let (ro1, _) = m.intern_gate(GateOp::RedOr, vec![c_zero], 1, DepSet::new());
        assert!(matches!(
            m.nodes[ro1 as usize],
            Node::Constant { value: 0, .. }
        ));
        let (ro2, _) = m.intern_gate(GateOp::RedOr, vec![c_mixed], 1, DepSet::new());
        assert!(matches!(
            m.nodes[ro2 as usize],
            Node::Constant { value: 1, .. }
        ));

        // RedXor(0b1010) = 0 (two 1-bits), RedXor(0b1110) = 1 (three
        // 1-bits). Use a fresh const to get the odd-bit-count case.
        let (c_odd, _) = m.intern_constant(4, 0b1110);
        let (rx_even, _) = m.intern_gate(GateOp::RedXor, vec![c_mixed], 1, DepSet::new());
        assert!(matches!(
            m.nodes[rx_even as usize],
            Node::Constant { value: 0, .. }
        ));
        let (rx_odd, _) = m.intern_gate(GateOp::RedXor, vec![c_odd], 1, DepSet::new());
        assert!(matches!(
            m.nodes[rx_odd as usize],
            Node::Constant { value: 1, .. }
        ));
    }

    // --- All-const arithmetic / structural evaluation ------------

    /// `Add(3, 5) → 8` at intern time.
    #[test]
    fn fold_all_const_add_evaluates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c3, _) = m.intern_constant(8, 3);
        let (c5, _) = m.intern_constant(8, 5);
        let (result, _) = m.intern_gate(GateOp::Add, vec![c3, c5], 8, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant { width: 8, value: 8 } => {}
            other => panic!("expected 8-bit const 8, got {other:?}"),
        }
    }

    /// `Mul(4, 5, 6)` evaluates in a variadic Mul to `120`.
    /// Mod-2^width semantics: 8-bit `Mul(100, 3) → 300 & 0xFF = 44`.
    #[test]
    fn fold_all_const_mul_wraps_modulo_width() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c100, _) = m.intern_constant(8, 100);
        let (c3, _) = m.intern_constant(8, 3);
        let (result, _) = m.intern_gate(GateOp::Mul, vec![c100, c3], 8, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant {
                width: 8,
                value: 44,
            } => {} // 300 & 0xFF = 44
            other => panic!("expected 8-bit const 44, got {other:?}"),
        }
    }

    /// `Xor(0b1010, 0b0110)` evaluates to `0b1100`.
    #[test]
    fn fold_all_const_xor_evaluates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (ca, _) = m.intern_constant(4, 0b1010);
        let (cb, _) = m.intern_constant(4, 0b0110);
        let (result, _) = m.intern_gate(GateOp::Xor, vec![ca, cb], 4, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant {
                width: 4,
                value: 0b1100,
            } => {}
            other => panic!("expected 4-bit const 0b1100, got {other:?}"),
        }
    }

    /// `Sub(10, 3) → 7`; `Sub(3, 10)` under 8-bit wraps to `249`.
    #[test]
    fn fold_all_const_sub_evaluates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (c10, _) = m.intern_constant(8, 10);
        let (c3, _) = m.intern_constant(8, 3);
        let (pos, _) = m.intern_gate(GateOp::Sub, vec![c10, c3], 8, DepSet::new());
        match &m.nodes[pos as usize] {
            Node::Constant { width: 8, value: 7 } => {}
            other => panic!("expected Sub(10, 3) = 7, got {other:?}"),
        }
        let (neg, _) = m.intern_gate(GateOp::Sub, vec![c3, c10], 8, DepSet::new());
        match &m.nodes[neg as usize] {
            Node::Constant {
                width: 8,
                value: 249,
            } => {} // 3-10 mod 256 = 249
            other => panic!("expected Sub(3, 10) mod 256 = 249, got {other:?}"),
        }
    }

    /// `Shl(0b0011, 2) → 0b1100`; shift amount ≥ width → 0.
    #[test]
    fn fold_all_const_shl_evaluates_and_clamps() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        // 4-bit value 0b0011, 3-bit shift-amount 2.
        let (val, _) = m.intern_constant(4, 0b0011);
        let (amt, _) = m.intern_constant(3, 2);
        let (shifted, _) = m.intern_gate(GateOp::Shl, vec![val, amt], 4, DepSet::new());
        match &m.nodes[shifted as usize] {
            Node::Constant {
                width: 4,
                value: 0b1100,
            } => {}
            other => panic!("expected Shl(3, 2) = 0b1100, got {other:?}"),
        }

        // Over-shift → 0.
        let (amt_big, _) = m.intern_constant(3, 5);
        let (zero, _) = m.intern_gate(GateOp::Shl, vec![val, amt_big], 4, DepSet::new());
        match &m.nodes[zero as usize] {
            Node::Constant { width: 4, value: 0 } => {}
            other => panic!("expected Shl(_, 5) over 4-bit = 0, got {other:?}"),
        }
    }

    /// `Shr(0b1100, 2) → 0b0011`.
    #[test]
    fn fold_all_const_shr_evaluates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (val, _) = m.intern_constant(4, 0b1100);
        let (amt, _) = m.intern_constant(3, 2);
        let (shifted, _) = m.intern_gate(GateOp::Shr, vec![val, amt], 4, DepSet::new());
        match &m.nodes[shifted as usize] {
            Node::Constant {
                width: 4,
                value: 0b0011,
            } => {}
            other => panic!("expected Shr(0b1100, 2) = 0b0011, got {other:?}"),
        }
    }

    /// `Concat(all-const)` assembles MSB-first: `{4'hA, 4'h5}` → 8'hA5.
    #[test]
    fn peephole_concat_of_constants_assembles_msb_first() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (hi, _) = m.intern_constant(4, 0xA);
        let (lo, _) = m.intern_constant(4, 0x5);
        let (result, _) = m.intern_gate(GateOp::Concat, vec![hi, lo], 8, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant {
                width: 8,
                value: 0xA5,
            } => {}
            other => panic!("expected 8-bit const 0xA5, got {other:?}"),
        }
    }

    /// `Concat([3'b101, 2'b01, 1'b1])` → 6-bit 0b101011 = 43.
    #[test]
    fn peephole_concat_of_constants_variadic() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        let (a, _) = m.intern_constant(3, 0b101);
        let (b, _) = m.intern_constant(2, 0b01);
        let (c, _) = m.intern_constant(1, 0b1);
        let (result, _) = m.intern_gate(GateOp::Concat, vec![a, b, c], 6, DepSet::new());
        match &m.nodes[result as usize] {
            Node::Constant {
                width: 6,
                value: 0b101011,
            } => {}
            other => panic!("expected 6-bit const 0b101011 = 43, got {other:?}"),
        }
    }

    // --- Associative flattening tests -----------------------------

    /// `Add(a, Add(b, c))` flattens to `Add(a, b, c)` at intern
    /// time. The outer call sees three operands; inner Add is
    /// orphaned (cleaned up by compact_node_ids, not tested here).
    #[test]
    fn flatten_associative_splices_same_op() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        for port in 0..3 {
            m.nodes.push(Node::PrimaryInput { port, width: 8 });
        }
        let a: NodeId = 0;
        let b: NodeId = 1;
        let c: NodeId = 2;
        let (inner, _) = m.intern_gate(GateOp::Add, vec![b, c], 8, DepSet::from_port(1));
        let (outer, _) = m.intern_gate(GateOp::Add, vec![a, inner], 8, DepSet::from_port(0));
        // outer should be a new gate with 3 operands (a, b, c) — not
        // 2 (a, inner).
        match &m.nodes[outer as usize] {
            Node::Gate {
                op: GateOp::Add,
                operands,
                ..
            } => {
                assert_eq!(operands.len(), 3, "outer Add must have flat operands");
                // Commutative sort already applied — check the set.
                let mut ids = operands.clone();
                ids.sort_unstable();
                assert_eq!(ids, vec![a, b, c]);
            }
            other => panic!("expected Add gate, got {other:?}"),
        }
        assert_eq!(m.flatten_associative_applied, 1);
    }

    /// `And(a, And(a, b))` flattens + dedups to `And(a, b)`.
    #[test]
    fn flatten_associative_and_dedups() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;
        let (inner, _) = m.intern_gate(GateOp::And, vec![a, b], 8, DepSet::from_port(0));
        let (outer, _) = m.intern_gate(GateOp::And, vec![a, inner], 8, DepSet::from_port(0));
        match &m.nodes[outer as usize] {
            Node::Gate {
                op: GateOp::And,
                operands,
                ..
            } => {
                let mut ids = operands.clone();
                ids.sort_unstable();
                assert_eq!(ids, vec![a, b], "duplicate a should dedup in And");
            }
            other => panic!("expected And gate, got {other:?}"),
        }
    }

    /// `Xor(a, Xor(a, b))` flattens + pair-cancels to `b`.
    #[test]
    fn flatten_associative_xor_pair_cancels() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;
        let (inner, _) = m.intern_gate(GateOp::Xor, vec![a, b], 8, DepSet::from_port(0));
        let (outer, _) = m.intern_gate(GateOp::Xor, vec![a, inner], 8, DepSet::from_port(0));
        // Post-flatten and cancel: [a, a, b] → [b]. Short-circuits
        // to b directly (no new Xor gate).
        assert_eq!(outer, b, "Xor(a, Xor(a, b)) must short-circuit to b");
    }

    /// `Xor(a, Xor(a, b), b)` cancels all → 0 constant.
    #[test]
    fn flatten_associative_xor_all_cancel_to_zero() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let a: NodeId = 0;
        let b: NodeId = 1;
        let (inner, _) = m.intern_gate(GateOp::Xor, vec![a, b], 8, DepSet::from_port(0));
        let (outer, _) = m.intern_gate(GateOp::Xor, vec![a, inner, b], 8, DepSet::from_port(0));
        match &m.nodes[outer as usize] {
            Node::Constant { width: 8, value: 0 } => {}
            other => panic!("expected 8-bit zero const, got {other:?}"),
        }
    }

    /// `Add(x, Add(x, y))` at strict operand_duplication_rate must
    /// NOT flatten — flattening would produce `Add(x, x, y)` which
    /// semantically differs from `Add(x, y)` (2x+y vs x+y). The
    /// helper returns None and the outer Add is interned with
    /// its original operands.
    #[test]
    fn flatten_associative_add_skips_on_duplicates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::default(),
            operand_duplication_rate: 0.0,
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 8 });
        let x: NodeId = 0;
        let y: NodeId = 1;
        let (inner, _) = m.intern_gate(GateOp::Add, vec![x, y], 8, DepSet::from_port(0));
        let (outer, _) = m.intern_gate(GateOp::Add, vec![x, inner], 8, DepSet::from_port(0));
        // The outer Add must have operands [x, inner] — not
        // flattened — preserving the 2x+y semantics.
        match &m.nodes[outer as usize] {
            Node::Gate {
                op: GateOp::Add,
                operands,
                ..
            } => {
                assert_eq!(
                    operands.len(),
                    2,
                    "Add should not flatten when duplicates would result"
                );
                assert!(operands.contains(&x));
                assert!(operands.contains(&inner));
            }
            other => panic!("expected Add gate, got {other:?}"),
        }
        assert_eq!(m.flatten_associative_applied, 0);
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
        let (concat, is_new) = m.intern_gate(GateOp::Concat, vec![x], 4, DepSet::from_port(0));
        assert_ne!(concat, x, "at level=ConstantFold peephole must not fire");
        assert!(is_new);
        assert_eq!(m.nodes.len(), before + 1);
        assert_eq!(m.peephole_rewrites_applied, 0);
    }
}
