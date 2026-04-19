//! Post-hoc metrics computed by walking an emitted `Module`.
//!
//! Metrics are structural facts about *what landed* in a module —
//! not about the generator's internal decisions. They are cheap to
//! compute (one pass over `m.nodes`, one pass over `m.flops`,
//! plus a reverse-fanout pass) and do not require any
//! instrumentation on the generator. Live counters for
//! knob-attempt signals (probability rolls fired / missed,
//! anti-collapse retries, tier picks) are a separate future
//! extension — see `ROADMAP.md`.
//!
//! The goal is observability per the user's directive: every knob
//! must be measurable from the generated output so we can tell
//! whether it is doing its job, whether it is redundant with
//! another knob, or whether a new knob is needed.

use crate::ir::{GateOp, Module, Node};
use serde::Serialize;
use std::collections::BTreeMap;

/// Structural summary of a single generated module. Serialisable as
/// JSON for inclusion in `manifest.json` or stderr dumps.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Metrics {
    /// Module identifier (e.g. `mod_42_0000`).
    pub module: String,

    // --- Size ---------------------------------------------------
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub num_nodes: usize,
    pub num_gates: usize,
    pub num_constants: usize,
    pub num_primary_inputs: usize,
    pub num_flop_q_refs: usize,
    pub num_flops: usize,

    // --- Per-gate-kind distribution -----------------------------
    /// Count of `Node::Gate` per `GateOp` kind (`"and"`, `"mux"`,
    /// etc.). Empty kinds omitted.
    pub gates_by_kind: BTreeMap<String, usize>,

    // --- Constants distribution ---------------------------------
    /// Count of `Node::Constant` by width. Reveals constant-width
    /// distribution (useful for the coefficient-width clamp).
    pub constants_by_width: BTreeMap<u32, usize>,
    /// Count of `Node::Constant` whose value is 0 vs all-ones vs
    /// other. Reveals the share of sentinel constants (zero fill,
    /// all-ones mask) vs meaningful literals.
    pub constants_zero: usize,
    pub constants_all_ones: usize,
    pub constants_other: usize,

    // --- Mux shape ----------------------------------------------
    /// Number of 2-to-1 `Mux` gates.
    pub num_muxes_2to1: usize,
    /// Number of 2-to-1 `Mux` gates whose two data arms are the
    /// same `NodeId` — the pathological `(s)?(x):(x)` form.
    /// Should be 0 at `mux_arm_duplication_rate = 0.0`.
    pub num_muxes_degenerate: usize,

    // --- Concat shape -------------------------------------------
    /// Number of `Concat` gates whose operands are all the same
    /// `NodeId` — emitted as `{N{expr}}`.
    pub num_concats_replication: usize,
    pub num_concats_heterogeneous: usize,

    // --- Sharing / fanout ---------------------------------------
    /// Number of internal nodes with fanout >= 2 (at least one
    /// other node references them). Measures sharing density
    /// after CSE.
    pub num_shared_nodes: usize,
    /// Maximum fanout observed on any single internal node.
    pub max_fanout: usize,
    /// Average fanout across all internal nodes (dep-bearing or
    /// not). `num_nodes == 0` → 0.0.
    pub avg_fanout: f64,

    // --- Flops --------------------------------------------------
    /// Per-kind flop count: how many `ZeroDefault` vs `QFeedback`.
    pub flops_zero_default: usize,
    pub flops_qfeedback: usize,
    /// Per-mux-shape flop count: `None` / `OneHot(M)` / `Encoded(M)`.
    pub flops_mux_none: usize,
    pub flops_mux_one_hot: usize,
    pub flops_mux_encoded: usize,

    // --- AST-instance saturation --------------------------------
    /// For each `(op, width)` pair, the maximum number of
    /// instances observed of any single AST of that kind. Should
    /// be `<= max_ast_instances` by construction. A value equal
    /// to the knob means the cap was hit — consumers are being
    /// routed to existing instances.
    pub max_gate_ast_multiplicity: usize,
    pub max_constant_ast_multiplicity: usize,

    // --- Operand-arity distribution -----------------------------
    /// Histogram of operator-gate arity (operand count) across all
    /// `Node::Gate`s. Keyed by operand count. Reveals the effective
    /// range of the `min_gate_arity` / `max_gate_arity` knobs.
    /// Non-operator nodes (comparisons, mux, slice, concat, reductions,
    /// shifts) with their fixed or variadic-positional arities are
    /// included too — all gate operand counts contribute.
    pub gate_operand_count_histogram: BTreeMap<usize, usize>,
    /// Maximum operand count observed on any single gate. For
    /// N-arity operators this is bounded above by `max_gate_arity`.
    pub max_gate_operand_count: usize,
    /// Per-op operand-count stats. Useful for distinguishing
    /// `Add`/`Mul` arity (bounded by `max_gate_arity`) from `Concat`
    /// arity (can be much larger, driven by mux-arm widths).
    pub max_operand_count_by_kind: BTreeMap<String, usize>,

    // --- Combinational depth ------------------------------------
    /// Combinational depth of each `Node::Gate`: longest path from
    /// the gate back to a leaf (primary input, constant, or flop Q).
    /// Computed by bottom-up walk over `m.nodes`, which is always
    /// in topological order (no forward references by construction).
    ///
    /// **Relationship to the `max_depth` knob:** the knob bounds
    /// the recursion depth of `build_cone`, not the IR gate-chain
    /// depth. Each `build_cone` recursion level can expand into
    /// many internal gate layers via block-assembly helpers
    /// (chained-ternary mux, OR-of-masked-arms mux, linear-
    /// combination adder trees). So `max_gate_depth` is typically
    /// 10–100× the knob value, but it is monotone in the knob —
    /// useful for verifying that raising `max_depth` produces
    /// deeper cones.
    pub max_gate_depth: usize,
    /// Histogram of per-gate combinational depth across all gates.
    /// Keyed by depth value.
    pub gate_depth_histogram: BTreeMap<usize, usize>,

    // --- Factorization-ladder telemetry -------------------------
    /// Count of operand slots on associative gates
    /// (`And`/`Or`/`Xor`/`Add`/`Mul`) whose operand is itself a
    /// same-op same-width gate *and is flattenable under the
    /// current duplicate policy*. `Add([a, Add(b, c), d])` counts
    /// 1 (the middle slot), flattening to `Add([a, b, c, d])`.
    /// `Add([a, Add(a, b)])` counts 0 at the default strict
    /// `operand_duplication_rate`, because flattening would
    /// introduce a duplicate `a` and the Associative layer
    /// intentionally preserves the nested shape in that case.
    ///
    /// The metric is post-hoc: it examines the finalized IR, not
    /// construction-time events. Running the generator over a
    /// seed sweep and summing this metric tells you how much
    /// flattening the current Associative layer still leaves on
    /// the table — justifying (or not) further work there.
    pub nested_associative_operand_count: usize,

    /// Number of times the `ConstantFold` factorization layer fired
    /// during construction. Each fire is one algebraic identity
    /// applied in `intern_gate` — either an operand dropped because
    /// it was an identity element (`x + 0`, `x & all_ones`, `x * 1`,
    /// …), an absorbing substitution (`x & 0 → 0`,
    /// `x | all_ones → all_ones`, `x * 0 → 0`), or a 2-arity rhs-
    /// zero short-circuit on `Sub` / `Shl` / `Shr`. Sourced from
    /// `Module::fold_identities_applied`. Zero at factorization
    /// levels below `ConstantFold`.
    pub fold_identities_applied: u64,

    /// Number of times the `Peephole` factorization layer fired
    /// during construction. Each fire is one local rewrite applied
    /// in `intern_gate` — double-negation collapse
    /// (`Not(Not(x)) → x`), fully-constant comparison evaluation
    /// (`Eq`/`Neq`/`Lt`/`Gt`/`Le`/`Ge` over two same-width
    /// constants), full-width `Slice(hi, 0)` identity, or
    /// single-operand `Concat` identity. Sourced from
    /// `Module::peephole_rewrites_applied`. Zero at factorization
    /// levels below `Peephole`.
    pub peephole_rewrites_applied: u64,

    /// Number of nodes removed by the post-construction
    /// `compact_node_ids` pass. Zero when every rewrite in
    /// `intern_gate` is orphan-safe by construction — non-zero
    /// when a rewrite like `Not(Not(x)) → x` leaves an inner
    /// gate unreferenced. Sourced from `Module::nodes_compacted`.
    pub nodes_compacted: u32,

    /// Number of times the `Associative` factorization layer
    /// fired during construction. Each fire is one `intern_gate`
    /// call on an associative op where at least one same-op
    /// same-width inner gate was spliced into the outer operand
    /// list. Zero at factorization levels below `Associative`.
    /// Sourced from `Module::flatten_associative_applied`.
    pub flatten_associative_applied: u64,

    // --- Per-knob probability-roll counters --------------------
    /// Attempt count per probability knob. Keyed by the knob's
    /// canonical string name (matches `Config` field, e.g.
    /// `"flop_prob"`). Every `gen_bool(cfg.<prob>)` site during
    /// construction routes through the `roll_knob` helper which
    /// increments attempts (and fires below on success). Empty
    /// knobs (no attempts taken during this module) are omitted
    /// from the map to keep JSON dumps compact.
    ///
    /// Read this with `knob_roll_fires` to compute the empirical
    /// fire-rate per knob — should converge to the configured
    /// probability across large seed sweeps. Divergences indicate
    /// either low sample count or a latent gate that prevents
    /// the roll from reaching its decision site (e.g.
    /// `flop_prob` rolls are gated by `flop_allowed`, so a module
    /// that hits `max_flops_per_module` early will see fewer
    /// attempts than expected).
    pub knob_roll_attempts: BTreeMap<String, u64>,
    /// Fire count per probability knob — the subset of attempts
    /// that returned `true`. See `knob_roll_attempts` for keying
    /// and interpretation.
    pub knob_roll_fires: BTreeMap<String, u64>,

    // --- Block-build counters -----------------------------------
    /// Number of priority-encoder block instances built in this
    /// module. Measures the `priority_encoder_prob` knob directly.
    pub num_priority_encoder_blocks: u32,
    /// Number of combinational one-hot-style mux blocks built.
    /// Together with `num_comb_muxes_encoded` measures the
    /// `comb_mux_encoding_prob` knob (the ratio should converge
    /// to the knob value over large seed sweeps).
    pub num_comb_muxes_one_hot: u32,
    /// Number of combinational encoded-style (chained-ternary)
    /// mux blocks built.
    pub num_comb_muxes_encoded: u32,
}

/// Compute metrics from a generated `Module`. Pure function — does
/// not modify the module.
pub fn compute(m: &Module) -> Metrics {
    let mut out = Metrics {
        module: m.name.clone(),
        num_inputs: m.inputs.len(),
        num_outputs: m.outputs.len(),
        num_nodes: m.nodes.len(),
        num_flops: m.flops.len(),
        ..Default::default()
    };

    // One pass: count nodes by kind, constants by shape, muxes by
    // shape, concats by shape.
    for node in &m.nodes {
        match node {
            Node::PrimaryInput { .. } => out.num_primary_inputs += 1,
            Node::FlopQ { .. } => out.num_flop_q_refs += 1,
            Node::Constant { width, value } => {
                out.num_constants += 1;
                *out.constants_by_width.entry(*width).or_insert(0) += 1;
                let all_ones: u128 = if *width >= 128 {
                    u128::MAX
                } else {
                    (1u128 << width) - 1
                };
                if *value == 0 {
                    out.constants_zero += 1;
                } else if *value == all_ones {
                    out.constants_all_ones += 1;
                } else {
                    out.constants_other += 1;
                }
            }
            Node::Gate { op, operands, .. } => {
                out.num_gates += 1;
                let kind = gate_kind_name(*op).to_string();
                *out.gates_by_kind.entry(kind.clone()).or_insert(0) += 1;

                // Operand-arity histogram + per-kind max.
                let arity = operands.len();
                *out.gate_operand_count_histogram.entry(arity).or_insert(0) += 1;
                if arity > out.max_gate_operand_count {
                    out.max_gate_operand_count = arity;
                }
                let entry = out.max_operand_count_by_kind.entry(kind).or_insert(0);
                if arity > *entry {
                    *entry = arity;
                }

                if matches!(op, GateOp::Mux) && operands.len() == 3 {
                    out.num_muxes_2to1 += 1;
                    if operands[1] == operands[2] {
                        out.num_muxes_degenerate += 1;
                    }
                }
                if matches!(op, GateOp::Concat) && !operands.is_empty() {
                    if operands.iter().all(|o| *o == operands[0]) {
                        out.num_concats_replication += 1;
                    } else {
                        out.num_concats_heterogeneous += 1;
                    }
                }
            }
        }
    }

    // Flops: per-kind and per-mux-shape counters.
    for f in &m.flops {
        match f.kind {
            crate::ir::FlopKind::ZeroDefault => out.flops_zero_default += 1,
            crate::ir::FlopKind::QFeedback => out.flops_qfeedback += 1,
        }
        match &f.mux {
            crate::ir::FlopMux::None => out.flops_mux_none += 1,
            crate::ir::FlopMux::OneHot(_) => out.flops_mux_one_hot += 1,
            crate::ir::FlopMux::Encoded { .. } => out.flops_mux_encoded += 1,
        }
    }

    // Combinational-depth pass. `m.nodes` is in topological order by
    // construction (Rule 1: combinational no-loop, arena-index
    // monotonicity). A single forward walk assigns each node its
    // depth as `max(operand depth) + 1`. Leaves (PrimaryInput,
    // Constant, FlopQ) are depth 0 — FlopQ acts as a leaf because
    // the clock edge breaks the Q→D loop temporally, so for
    // combinational depth reasoning the Q is a zero-depth source.
    let mut depth = vec![0usize; m.nodes.len()];
    for (idx, node) in m.nodes.iter().enumerate() {
        if let Node::Gate { operands, .. } = node {
            let max_operand = operands
                .iter()
                .map(|o| depth[*o as usize])
                .max()
                .unwrap_or(0);
            depth[idx] = max_operand + 1;
            *out.gate_depth_histogram.entry(depth[idx]).or_insert(0) += 1;
            if depth[idx] > out.max_gate_depth {
                out.max_gate_depth = depth[idx];
            }
        }
    }

    // Fanout pass: walk every Gate plus each emitted flop D-input to
    // build a use-count per NodeId. Primary inputs and constants are
    // included (they can have fanout like any other node). Output
    // drives also count as a use. Flop-mux operand metadata is
    // intentionally ignored here: after finalisation it is summary
    // shape information, not an emitted consumer.
    let mut fanout = vec![0usize; m.nodes.len()];
    for node in &m.nodes {
        if let Node::Gate { operands, .. } = node {
            for &op in operands {
                fanout[op as usize] += 1;
            }
        }
    }
    for f in &m.flops {
        if let Some(d) = f.d {
            fanout[d as usize] += 1;
        }
    }
    for (_, root) in &m.drives {
        fanout[*root as usize] += 1;
    }
    out.num_shared_nodes = fanout.iter().filter(|c| **c >= 2).count();
    out.max_fanout = fanout.iter().copied().max().unwrap_or(0);
    out.avg_fanout = if !fanout.is_empty() {
        fanout.iter().sum::<usize>() as f64 / fanout.len() as f64
    } else {
        0.0
    };

    // AST-instance saturation from the dedup tables.
    out.max_gate_ast_multiplicity = m
        .gate_instances
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);
    out.max_constant_ast_multiplicity = m
        .const_instances
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    // Block-build counters (populated live during construction).
    out.num_priority_encoder_blocks = m.priority_encoder_built;
    out.num_comb_muxes_one_hot = m.comb_mux_one_hot_built;
    out.num_comb_muxes_encoded = m.comb_mux_encoded_built;

    // ConstantFold factorization layer: counter sourced live from
    // `intern_gate`. Zero at levels below `ConstantFold`.
    out.fold_identities_applied = m.fold_identities_applied;
    out.peephole_rewrites_applied = m.peephole_rewrites_applied;
    out.nodes_compacted = m.nodes_compacted;
    out.flatten_associative_applied = m.flatten_associative_applied;

    // Per-knob attempt/fire counters. Convert enum keys to strings
    // for serialisation. Non-empty knobs only.
    for (knob, count) in &m.knob_rolls.attempts {
        out.knob_roll_attempts
            .insert(knob.name().to_string(), *count);
    }
    for (knob, count) in &m.knob_rolls.fires {
        out.knob_roll_fires.insert(knob.name().to_string(), *count);
    }

    // Associative-flattening-opportunities scan. For every
    // associative gate, count same-op same-width inner-gate slots
    // that the current duplicate policy would allow us to flatten.
    // Add/Mul are special: at strict `operand_duplication_rate`
    // the live Associative layer intentionally preserves nested
    // shapes that would become duplicate-bearing if flattened.
    for node in &m.nodes {
        if let Node::Gate {
            op,
            operands,
            width,
            ..
        } = node
        {
            if !matches!(
                op,
                GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
            ) {
                continue;
            }
            let nested_slots: Vec<_> = operands
                .iter()
                .copied()
                .filter(|operand_id| {
                    matches!(
                        &m.nodes[*operand_id as usize],
                        Node::Gate {
                            op: inner_op,
                            width: inner_w,
                            ..
                        } if inner_op == op && inner_w == width
                    )
                })
                .collect();
            if nested_slots.is_empty() {
                continue;
            }
            if matches!(op, GateOp::Add | GateOp::Mul) && m.operand_duplication_rate < 1.0 {
                use std::collections::HashSet;

                let mut flat: Vec<crate::ir::NodeId> = Vec::with_capacity(operands.len());
                for &operand_id in operands {
                    match &m.nodes[operand_id as usize] {
                        Node::Gate {
                            op: inner_op,
                            operands: inner_ops,
                            width: inner_w,
                            ..
                        } if inner_op == op && inner_w == width => {
                            flat.extend(inner_ops.iter().copied());
                        }
                        _ => flat.push(operand_id),
                    }
                }
                let mut seen = HashSet::new();
                if flat.iter().any(|id| !seen.insert(*id)) {
                    continue;
                }
            }
            out.nested_associative_operand_count += nested_slots.len();
        }
    }

    out
}

/// Canonical lowercase name per `GateOp`. Kept here (duplicated
/// from `emit::sv::gate_kind_name`) to avoid a cross-module
/// coupling — `metrics` must stay independent of `emit`.
fn gate_kind_name(op: GateOp) -> &'static str {
    use GateOp::*;
    match op {
        And => "and",
        Or => "or",
        Xor => "xor",
        Not => "not",
        Add => "add",
        Sub => "sub",
        Mul => "mul",
        Eq => "eq",
        Neq => "neq",
        Lt => "lt",
        Gt => "gt",
        Le => "le",
        Ge => "ge",
        Mux => "mux",
        Slice { .. } => "slice",
        Concat => "concat",
        RedAnd => "red_and",
        RedOr => "red_or",
        RedXor => "red_xor",
        Shl => "shl",
        Shr => "shr",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, FlopKind, FlopMux, Port};

    #[test]
    fn metrics_on_empty_module() {
        let m = Module {
            name: "empty".into(),
            ..Module::default()
        };
        let met = compute(&m);
        assert_eq!(met.num_nodes, 0);
        assert_eq!(met.num_gates, 0);
        assert_eq!(met.num_flops, 0);
    }

    #[test]
    fn metrics_count_gates_by_kind() {
        let mut m = Module {
            name: "k".into(),
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 4,
            dir: Direction::In,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 4 });
        let (g1, _) = m.intern_gate(GateOp::And, vec![0, 0], 4, DepSet::from_port(0));
        let (g2, _) = m.intern_gate(GateOp::Mux, vec![0, g1, g1], 4, DepSet::from_port(0));
        let _ = g2;
        let met = compute(&m);
        assert_eq!(met.gates_by_kind.get("and").copied(), Some(1));
        assert_eq!(met.gates_by_kind.get("mux").copied(), Some(1));
        // Mux with equal data arms is the degenerate form.
        assert_eq!(met.num_muxes_2to1, 1);
        assert_eq!(met.num_muxes_degenerate, 1);
    }

    #[test]
    fn metrics_count_flops_by_shape() {
        let mut m = Module {
            name: "f".into(),
            ..Module::default()
        };
        m.flops.push(crate::ir::Flop {
            id: 0,
            width: 4,
            d: Some(0),
            q: 0,
            reset_val: 0,
            reset_kind: crate::ir::ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(crate::ir::Flop {
            id: 1,
            width: 4,
            d: Some(0),
            q: 0,
            reset_val: 0,
            reset_kind: crate::ir::ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::OneHot(vec![]),
        });
        m.nodes.push(Node::Constant { width: 4, value: 0 });
        let met = compute(&m);
        assert_eq!(met.num_flops, 2);
        assert_eq!(met.flops_zero_default, 1);
        assert_eq!(met.flops_qfeedback, 1);
        assert_eq!(met.flops_mux_none, 1);
        assert_eq!(met.flops_mux_one_hot, 1);
    }
}
