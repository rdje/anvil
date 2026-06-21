//! Post-construction IR finalization passes.
//!
//! Rule 18 says zero orphan gates at the end of construction — every
//! gate must have at least one consumer (another gate's operand, a
//! flop field, or an output drive). Today's generator enforces this
//! by construction via `build_cone`'s snapshot/rollback and
//! `process_signal_frame`'s existing-operand fallback. That keeps
//! the IR Rule-18-clean without any post-pass.
//!
//! This module houses post-construction finalization passes:
//!
//! - `merge_equivalent_gates(&mut m)`: a bounded semantic-sharing pass
//!   for combinational nodes. Under `identity_mode = node-id` with
//!   effective factorization level `EGraph`, gates with the same
//!   endpoint-preserving proof collapse to one earlier canonical node,
//!   including an existing endpoint or constant when the proof matches.
//! - `merge_equivalent_flops(&mut m)`: a conservative stateful
//!   sharing pass that runs only once flop D-cones exist. Under
//!   `identity_mode = node-id` with effective factorization level
//!   at least `Cse`, flops with the same emitted state semantics
//!   (`width`, reset, clock domain, same canonical leaf endpoints, and a
//!   D-cone functionality proven either by the current normalized proof
//!   form or by a bounded small-support semantic check) are collapsed to
//!   one state element. The one coinductive exception is exact
//!   reset-defined self-hold (`D == own Q`): equal reset/domain/width
//!   self-hold flops may share one state element because reset
//!   establishes equality and the transition preserves it.
//! - `merge_equivalent_fsms(&mut m)`: a deterministic FSM-sharing pass
//!   that uses the same endpoint-preserving proof discipline for the
//!   selector cone, plus the reset-defined FSM transition/output tables,
//!   to collapse duplicate generated FSM blocks.
//! - `fold_proven_gates(&mut m)`: a downstream-cleanliness pass that
//!   revisits already-built gates using the final graph. It folds any
//!   gate whose current cone is provably exact and rewires muxes whose
//!   selector is now provably constant.
//! - `flatten_posthoc_associative_gates(&mut m)`: a normalisation pass
//!   that restores associative flattening after later remap passes have
//!   changed which already-built node an operand points at.
//! - `fold_mixed_associative_constants(&mut m)`: a late constant-fold
//!   cleanup that aggregates multiple constant operands inside settled
//!   associative gates after remap passes expose opportunities that did
//!   not exist at original intern time.
//! - `compact_node_ids(&mut m)`: a defensive reachability pass that
//!   walks from roots, identifies any node that became orphaned by a
//!   construction-time rewrite (e.g. the `Not(Not(x)) → x`
//!   peephole, which leaves the inner `Not` referenced only by the
//!   outer `Not` call), drops dead flops whose `Q` is never reached
//!   by the live graph, and compacts the `m.nodes` / `m.flops` arenas
//!   to only the reachable set.
//!
//! The merge is intentionally conservative, not a general
//! sequential-equivalence engine: it compares endpoint-preserving proof
//! forms over the already-normalized IR, and adds only a bounded
//! small-support semantic check on top. It does not try to prove wider
//! coinductive equalities that the current factorization ladder has not
//! already canonicalized. The compaction pass is idempotent and a no-op
//! when there are no orphans. It exists primarily to unblock rewrites
//! that would otherwise orphan intermediate gates. Without it those
//! rewrites would have to be suppressed to stay Rule-18-clean (as they
//! were before this module).
//!
//! ## Guarantees
//!
//! After `compact_node_ids(&mut m)`:
//!
//! - `m.nodes` contains only nodes reachable from some surviving
//!   output drive-root.
//! - Every `NodeId` in `m.nodes[*].operands`, `m.drives`, and the
//!   `Flop` / `FlopMux` fields points to a valid index in the new
//!   `m.nodes`.
//! - `m.flops` contains only state elements whose `Q` is observed by
//!   the retained graph, and virtual flop deps are remapped to the
//!   compacted `FlopId` space.
//! - The dedup tables (`gate_instances`, `const_instances`) are
//!   rebuilt against the new `NodeId` space. Entries whose target
//!   was unreachable are dropped; surviving entries reference the
//!   new indices.
//! - Topological order is preserved: operands of any Gate in
//!   `m.nodes[i]` live at indices `< i`. This matches the invariant
//!   exploited by `Metrics::compute` (forward-walk depth
//!   computation).
//!
//! ## Non-goals
//!
//! `merge_equivalent_gates` and `merge_equivalent_flops` are not
//! general equivalence provers, and `compact_node_ids` is not a
//! semantic merge at all. Wider semantic equivalence across arbitrary
//! gate trees, larger supports, and richer stateful motifs remains the
//! e-graph aspiration (Rule 21c). Memories are deliberately not merged
//! by the current state passes: the inferrable-memory template has no
//! reset-defined array contents, so equal write/read cones are not proof
//! that two independent memory instances store equal state.

use super::types::{
    Flop, FlopId, FlopMux, FsmEncoding, FsmId, GateOp, Module, MuxArm, Node, NodeId, PortId,
    ResetKind,
};
use crate::config::FactorizationLevel;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const BASELINE_SEMANTIC_SUPPORT_BITS: u32 = 10;
const MAX_SEMANTIC_SUPPORT_BITS: u32 = 12;
const MAX_SEMANTIC_EXACT_ENDPOINTS: usize = 3;
const MAX_MERGE_SEMANTIC_CONE_NODES: usize = 128;
const MAX_CLEANUP_SEMANTIC_CONE_NODES: usize = 64;
const MAX_MERGE_SEMANTIC_WORK_UNITS: usize =
    (1usize << BASELINE_SEMANTIC_SUPPORT_BITS) * MAX_MERGE_SEMANTIC_CONE_NODES;
const MAX_CLEANUP_SEMANTIC_WORK_UNITS: usize =
    (1usize << BASELINE_SEMANTIC_SUPPORT_BITS) * MAX_CLEANUP_SEMANTIC_CONE_NODES;

#[derive(Debug, Clone, Copy)]
struct SemanticProofLimits {
    max_support_bits: u32,
    max_cone_nodes: usize,
    max_work_units: usize,
}

const MERGE_SEMANTIC_LIMITS: SemanticProofLimits = SemanticProofLimits {
    max_support_bits: MAX_SEMANTIC_SUPPORT_BITS,
    max_cone_nodes: MAX_MERGE_SEMANTIC_CONE_NODES,
    max_work_units: MAX_MERGE_SEMANTIC_WORK_UNITS,
};

const CLEANUP_SEMANTIC_LIMITS: SemanticProofLimits = SemanticProofLimits {
    max_support_bits: MAX_SEMANTIC_SUPPORT_BITS,
    max_cone_nodes: MAX_CLEANUP_SEMANTIC_CONE_NODES,
    max_work_units: MAX_CLEANUP_SEMANTIC_WORK_UNITS,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FlopSignature {
    width: u32,
    clock_domain: u32,
    d: FlopDSignature,
    reset_val: u128,
    reset_kind: ResetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FlopDSignature {
    Cone(ConeProof),
    ResetDefinedSelfHold,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FsmSignature {
    num_states: u32,
    encoding: FsmEncoding,
    sel: ConeProof,
    sel_width: u32,
    transitions: Vec<Vec<u32>>,
    outputs: Vec<u128>,
    out_width: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConeProof {
    Structural(StructuralSigId),
    Semantic(SemanticConeProof),
}

type StructuralSigId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StructuralNodeShape {
    PrimaryInput {
        port: PortId,
        width: u32,
    },
    InstanceOutput {
        instance: crate::ir::InstanceId,
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
    MemRead {
        mem: crate::ir::MemId,
        width: u32,
    },
    FsmOut {
        fsm: crate::ir::FsmId,
        width: u32,
    },
    Gate {
        op: GateOp,
        width: u32,
        operands: Vec<StructuralSigId>,
    },
}

#[derive(Debug, Default)]
struct StructuralSignatureCtx {
    shapes: Vec<StructuralNodeShape>,
    interner: HashMap<StructuralNodeShape, StructuralSigId>,
}

impl StructuralSignatureCtx {
    fn intern(&mut self, shape: StructuralNodeShape) -> StructuralSigId {
        if let Some(&sig_id) = self.interner.get(&shape) {
            return sig_id;
        }
        let sig_id = self.shapes.len() as StructuralSigId;
        self.shapes.push(shape.clone());
        self.interner.insert(shape, sig_id);
        sig_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LeafEndpoint {
    PrimaryInput {
        port: PortId,
        width: u32,
    },
    InstanceOutput {
        instance: crate::ir::InstanceId,
        port: PortId,
        width: u32,
    },
    FlopQ {
        flop: FlopId,
        width: u32,
    },
    MemRead {
        mem: crate::ir::MemId,
        width: u32,
    },
    FsmOut {
        fsm: crate::ir::FsmId,
        width: u32,
    },
}

impl LeafEndpoint {
    fn width(self) -> u32 {
        match self {
            LeafEndpoint::PrimaryInput { width, .. }
            | LeafEndpoint::InstanceOutput { width, .. }
            | LeafEndpoint::FlopQ { width, .. }
            | LeafEndpoint::MemRead { width, .. }
            | LeafEndpoint::FsmOut { width, .. } => width,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SemanticConeProof {
    endpoints: Vec<LeafEndpoint>,
    outputs: Vec<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GateSignature {
    width: u32,
    proof: ConeProof,
}

fn structural_node_sig_id(
    m: &Module,
    node_id: NodeId,
    memo: &mut HashMap<NodeId, StructuralSigId>,
    ctx: &mut StructuralSignatureCtx,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> StructuralSigId {
    if let Some(&sig_id) = memo.get(&node_id) {
        return sig_id;
    }

    let sig_id = match &m.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => ctx.intern(StructuralNodeShape::PrimaryInput {
            port: *port,
            width: *width,
        }),
        Node::InstanceOutput {
            instance,
            port,
            width,
        } => ctx.intern(StructuralNodeShape::InstanceOutput {
            instance: *instance,
            port: *port,
            width: *width,
        }),
        Node::Constant { width, value } => ctx.intern(StructuralNodeShape::Constant {
            width: *width,
            value: *value,
        }),
        Node::FlopQ { flop, width } => ctx.intern(StructuralNodeShape::FlopQ {
            flop: canonical_flop_endpoint(*flop, quotient),
            width: *width,
        }),
        Node::MemRead { mem, width } => ctx.intern(StructuralNodeShape::MemRead {
            mem: *mem,
            width: *width,
        }),
        Node::FsmOut { fsm, width } => ctx.intern(StructuralNodeShape::FsmOut {
            fsm: *fsm,
            width: *width,
        }),
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => {
            let operand_sigs = operands
                .iter()
                .map(|&operand| structural_node_sig_id(m, operand, memo, ctx, quotient))
                .collect();
            ctx.intern(StructuralNodeShape::Gate {
                op: *op,
                width: *width,
                operands: operand_sigs,
            })
        }
    };

    memo.insert(node_id, sig_id);
    sig_id
}

fn bitmask(width: u32) -> u128 {
    if width >= 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}

/// Canonicalize a `FlopQ` leaf id under an optional bisimulation quotient.
///
/// The exact flop/gate/FSM identity passes — and the cleanup prover — call
/// every proof function with `quotient = None`, which returns the id
/// unchanged, so their proofs stay byte-identical. Only
/// [`merge_bisimilar_flops`] supplies `Some(class_rep_map)`, rewriting each
/// `FlopQ` endpoint to its current partition-class representative so two
/// flops' D-cones are compared *up to the current state correspondence*
/// (`IDENTITY-DEEPENING`, decision `0007`).
fn canonical_flop_endpoint(flop: FlopId, quotient: Option<&HashMap<FlopId, FlopId>>) -> FlopId {
    match quotient {
        Some(map) => map.get(&flop).copied().unwrap_or(flop),
        None => flop,
    }
}

fn collect_leaf_endpoints(
    m: &Module,
    node_id: NodeId,
    memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> BTreeSet<LeafEndpoint> {
    if let Some(endpoints) = memo.get(&node_id) {
        return endpoints.clone();
    }

    let endpoints = match &m.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => BTreeSet::from([LeafEndpoint::PrimaryInput {
            port: *port,
            width: *width,
        }]),
        Node::InstanceOutput {
            instance,
            port,
            width,
        } => BTreeSet::from([LeafEndpoint::InstanceOutput {
            instance: *instance,
            port: *port,
            width: *width,
        }]),
        Node::FlopQ { flop, width } => BTreeSet::from([LeafEndpoint::FlopQ {
            flop: canonical_flop_endpoint(*flop, quotient),
            width: *width,
        }]),
        Node::MemRead { mem, width } => BTreeSet::from([LeafEndpoint::MemRead {
            mem: *mem,
            width: *width,
        }]),
        Node::FsmOut { fsm, width } => BTreeSet::from([LeafEndpoint::FsmOut {
            fsm: *fsm,
            width: *width,
        }]),
        Node::Constant { .. } => BTreeSet::new(),
        Node::Gate { operands, .. } => {
            let mut out = BTreeSet::new();
            for &operand in operands {
                out.extend(collect_leaf_endpoints(m, operand, memo, quotient));
            }
            out
        }
    };

    memo.insert(node_id, endpoints.clone());
    endpoints
}

fn cone_within_node_budget(
    m: &Module,
    node_id: NodeId,
    budget: usize,
    seen: &mut HashSet<NodeId>,
) -> bool {
    if !seen.insert(node_id) {
        return true;
    }
    if seen.len() > budget {
        return false;
    }
    let Node::Gate { operands, .. } = &m.nodes[node_id as usize] else {
        return true;
    };
    for &operand in operands {
        if !cone_within_node_budget(m, operand, budget, seen) {
            return false;
        }
    }
    true
}

fn assignment_count_for_support(support_bits: u32) -> Option<usize> {
    if support_bits >= usize::BITS {
        None
    } else {
        Some(1usize << support_bits)
    }
}

fn semantic_work_within_budget(
    assignment_count: usize,
    cone_nodes: usize,
    max_work_units: usize,
) -> bool {
    assignment_count.saturating_mul(cone_nodes) <= max_work_units
}

fn semantic_proof_eligibility(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    limits: SemanticProofLimits,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> Option<(Vec<LeafEndpoint>, usize)> {
    if m.nodes[node_id as usize].width() > 128 {
        return None;
    }

    let endpoints: Vec<LeafEndpoint> = collect_leaf_endpoints(m, node_id, endpoint_memo, quotient)
        .into_iter()
        .collect();
    let support_bits: u32 = endpoints.iter().map(|endpoint| endpoint.width()).sum();
    if support_bits > limits.max_support_bits {
        return None;
    }

    let assignment_count = assignment_count_for_support(support_bits)?;

    // Semantic proofs stay intentionally bounded: large settled cones
    // with tiny endpoint support can still explode runtime if every
    // assignment walks the whole cone. The work budget preserves the
    // old 10-bit worst-case envelope while allowing slightly wider
    // support only for shallow cones.
    let mut seen = HashSet::new();
    if !cone_within_node_budget(m, node_id, limits.max_cone_nodes, &mut seen) {
        return None;
    }
    if !semantic_work_within_budget(assignment_count, seen.len(), limits.max_work_units) {
        return None;
    }

    Some((endpoints, assignment_count))
}

fn evaluate_node_under_assignment(
    m: &Module,
    node_id: NodeId,
    assignment: u128,
    endpoint_offsets: &HashMap<LeafEndpoint, u32>,
    memo: &mut HashMap<NodeId, u128>,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> u128 {
    if let Some(&value) = memo.get(&node_id) {
        return value;
    }

    let value = match &m.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => {
            let endpoint = LeafEndpoint::PrimaryInput {
                port: *port,
                width: *width,
            };
            let offset = endpoint_offsets[&endpoint];
            (assignment >> offset) & bitmask(*width)
        }
        Node::InstanceOutput {
            instance,
            port,
            width,
        } => {
            let endpoint = LeafEndpoint::InstanceOutput {
                instance: *instance,
                port: *port,
                width: *width,
            };
            let offset = endpoint_offsets[&endpoint];
            (assignment >> offset) & bitmask(*width)
        }
        Node::FlopQ { flop, width } => {
            let endpoint = LeafEndpoint::FlopQ {
                flop: canonical_flop_endpoint(*flop, quotient),
                width: *width,
            };
            let offset = endpoint_offsets[&endpoint];
            (assignment >> offset) & bitmask(*width)
        }
        Node::MemRead { mem, width } => {
            let endpoint = LeafEndpoint::MemRead {
                mem: *mem,
                width: *width,
            };
            let offset = endpoint_offsets[&endpoint];
            (assignment >> offset) & bitmask(*width)
        }
        Node::FsmOut { fsm, width } => {
            let endpoint = LeafEndpoint::FsmOut {
                fsm: *fsm,
                width: *width,
            };
            let offset = endpoint_offsets[&endpoint];
            (assignment >> offset) & bitmask(*width)
        }
        Node::Constant { width, value } => *value & bitmask(*width),
        Node::Gate {
            op,
            operands,
            width,
            ..
        } => {
            let width_mask = bitmask(*width);
            let operand_values: Vec<u128> = operands
                .iter()
                .map(|&operand| {
                    evaluate_node_under_assignment(
                        m,
                        operand,
                        assignment,
                        endpoint_offsets,
                        memo,
                        quotient,
                    )
                })
                .collect();
            match op {
                GateOp::And => operand_values
                    .iter()
                    .copied()
                    .fold(width_mask, |acc, v| acc & v),
                GateOp::Or => operand_values.iter().copied().fold(0u128, |acc, v| acc | v),
                GateOp::Xor => operand_values.iter().copied().fold(0u128, |acc, v| acc ^ v),
                GateOp::Not => (!operand_values[0]) & width_mask,
                GateOp::Add => operand_values
                    .iter()
                    .copied()
                    .fold(0u128, |acc, v| acc.wrapping_add(v) & width_mask),
                GateOp::Sub => operand_values[0].wrapping_sub(operand_values[1]) & width_mask,
                GateOp::Mul => operand_values
                    .iter()
                    .copied()
                    .fold(1u128, |acc, v| acc.wrapping_mul(v) & width_mask),
                GateOp::Eq => (operand_values[0] == operand_values[1]) as u128,
                GateOp::Neq => (operand_values[0] != operand_values[1]) as u128,
                GateOp::Lt => (operand_values[0] < operand_values[1]) as u128,
                GateOp::Gt => (operand_values[0] > operand_values[1]) as u128,
                GateOp::Le => (operand_values[0] <= operand_values[1]) as u128,
                GateOp::Ge => (operand_values[0] >= operand_values[1]) as u128,
                GateOp::Mux => {
                    if operand_values[0] == 0 {
                        operand_values[2] & width_mask
                    } else {
                        operand_values[1] & width_mask
                    }
                }
                GateOp::CaseMux => {
                    let sel = operand_values[0] as usize;
                    let data_arms = operand_values.len().saturating_sub(1);
                    if sel < data_arms {
                        operand_values[sel + 1] & width_mask
                    } else {
                        0
                    }
                }
                GateOp::CasezMux => {
                    let sel_width = m.nodes[operands[0] as usize].width();
                    let sel_mask = bitmask(sel_width);
                    let sel = operand_values[0] & sel_mask;
                    let mut matched = None;
                    for arm in operand_values[1..].chunks_exact(3) {
                        let pattern = arm[0] & sel_mask;
                        let wildcard_mask = arm[1] & sel_mask;
                        let care_mask = (!wildcard_mask) & sel_mask;
                        if (sel & care_mask) == (pattern & care_mask) {
                            matched = Some(arm[2] & width_mask);
                            break;
                        }
                    }
                    matched.unwrap_or(0)
                }
                GateOp::ForFold {
                    kind,
                    trip_count,
                    chunk_width,
                } => {
                    let mut acc = match kind {
                        crate::ir::ForFoldKind::And => bitmask(*chunk_width),
                        crate::ir::ForFoldKind::Xor
                        | crate::ir::ForFoldKind::Or
                        | crate::ir::ForFoldKind::Add => 0,
                    };
                    for idx in 0..*trip_count {
                        let shift = idx.saturating_mul(*chunk_width);
                        let chunk = if shift >= 128 {
                            0
                        } else {
                            (operand_values[0] >> shift) & bitmask(*chunk_width)
                        };
                        acc = match kind {
                            crate::ir::ForFoldKind::Xor => (acc ^ chunk) & width_mask,
                            crate::ir::ForFoldKind::Or => (acc | chunk) & width_mask,
                            crate::ir::ForFoldKind::And => (acc & chunk) & width_mask,
                            crate::ir::ForFoldKind::Add => acc.wrapping_add(chunk) & width_mask,
                        };
                    }
                    acc & width_mask
                }
                GateOp::Slice { hi, lo } => {
                    let slice_width = hi - lo + 1;
                    (operand_values[0] >> lo) & bitmask(slice_width)
                }
                GateOp::Concat => {
                    let mut out = 0u128;
                    for (&operand, operand_value) in operands.iter().zip(operand_values.iter()) {
                        let operand_width = m.nodes[operand as usize].width();
                        out = if operand_width >= 128 {
                            operand_value & bitmask(operand_width)
                        } else {
                            (out << operand_width) | (operand_value & bitmask(operand_width))
                        };
                    }
                    out & width_mask
                }
                GateOp::RedAnd => {
                    let src_width = m.nodes[operands[0] as usize].width();
                    (operand_values[0] == bitmask(src_width)) as u128
                }
                GateOp::RedOr => (operand_values[0] != 0) as u128,
                GateOp::RedXor => (operand_values[0].count_ones() & 1) as u128,
                GateOp::Shl => {
                    let amt = operand_values[1];
                    if amt >= u128::from(*width) {
                        0
                    } else {
                        operand_values[0].wrapping_shl(amt as u32) & width_mask
                    }
                }
                GateOp::Shr => {
                    let amt = operand_values[1];
                    if amt >= u128::from(*width) {
                        0
                    } else {
                        (operand_values[0] >> amt) & width_mask
                    }
                }
            }
        }
    };

    memo.insert(node_id, value);
    value
}

/// Test-only convenience wrapper: the MERGE-limit semantic proof over the
/// concrete (non-quotient) endpoint identity. Production `cone_proof`
/// inlines `semantic_cone_proof_with_limits` so it can thread the
/// bisimulation quotient; the budget regression tests use this short form.
#[cfg(test)]
fn semantic_cone_proof(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
) -> Option<SemanticConeProof> {
    semantic_cone_proof_with_limits(m, node_id, endpoint_memo, MERGE_SEMANTIC_LIMITS, None)
}

fn semantic_cone_proof_with_limits(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    limits: SemanticProofLimits,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> Option<SemanticConeProof> {
    let (endpoints, assignment_count) =
        semantic_proof_eligibility(m, node_id, endpoint_memo, limits, quotient)?;
    let mut endpoint_offsets: HashMap<LeafEndpoint, u32> = HashMap::new();
    let mut next_offset = 0u32;
    for endpoint in &endpoints {
        endpoint_offsets.insert(*endpoint, next_offset);
        next_offset += endpoint.width();
    }

    let mut outputs = Vec::with_capacity(assignment_count);
    for assignment in 0..assignment_count {
        let mut memo: HashMap<NodeId, u128> = HashMap::new();
        outputs.push(evaluate_node_under_assignment(
            m,
            node_id,
            assignment as u128,
            &endpoint_offsets,
            &mut memo,
            quotient,
        ));
    }

    let mut kept_endpoints = Vec::new();
    let mut kept_offsets = Vec::new();
    for endpoint in &endpoints {
        let offset = endpoint_offsets[endpoint];
        let endpoint_mask = bitmask(endpoint.width()) << offset;
        let independent = outputs.iter().enumerate().all(|(assignment, value)| {
            let canonical_assignment = (assignment as u128) & !endpoint_mask;
            *value == outputs[canonical_assignment as usize]
        });
        if !independent {
            kept_offsets.push(offset);
            kept_endpoints.push(*endpoint);
        }
    }

    if kept_endpoints.len() != endpoints.len() {
        let kept_support_bits: u32 = kept_endpoints.iter().map(|endpoint| endpoint.width()).sum();
        let kept_assignment_count = 1usize << kept_support_bits;
        let mut reduced_outputs = Vec::with_capacity(kept_assignment_count);
        for reduced_assignment in 0..kept_assignment_count {
            let mut reduced_offset = 0u32;
            let mut original_assignment = 0u128;
            for (endpoint, original_offset) in kept_endpoints.iter().zip(kept_offsets.iter()) {
                let value =
                    ((reduced_assignment as u128) >> reduced_offset) & bitmask(endpoint.width());
                original_assignment |= value << original_offset;
                reduced_offset += endpoint.width();
            }
            reduced_outputs.push(outputs[original_assignment as usize]);
        }
        return Some(SemanticConeProof {
            endpoints: kept_endpoints,
            outputs: reduced_outputs,
        });
    }

    Some(SemanticConeProof { endpoints, outputs })
}

fn cleanup_exact_proof_eligible(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
) -> bool {
    // This cleanup-only exact prover is intentionally stricter than the
    // bounded semantic merge passes above: it exists to scrub obvious
    // constants for downstream-tool cleanliness, not to widen the main
    // semantic-sharing contract at any runtime cost. Both the bounds-
    // based exact prover and the semantic truth-table fallback must
    // obey this gate so cleanup never walks large associative cones.
    if m.nodes[node_id as usize].width() > 8 {
        return false;
    }

    let endpoints = collect_leaf_endpoints(m, node_id, endpoint_memo, None);
    if endpoints.len() > MAX_SEMANTIC_EXACT_ENDPOINTS {
        return false;
    }

    let mut seen = HashSet::new();
    let support_bits: u32 = endpoints.iter().map(|endpoint| endpoint.width()).sum();
    let Some(assignment_count) = assignment_count_for_support(support_bits) else {
        return false;
    };
    support_bits <= CLEANUP_SEMANTIC_LIMITS.max_support_bits
        && cone_within_node_budget(
            m,
            node_id,
            CLEANUP_SEMANTIC_LIMITS.max_cone_nodes,
            &mut seen,
        )
        && semantic_work_within_budget(
            assignment_count,
            seen.len(),
            CLEANUP_SEMANTIC_LIMITS.max_work_units,
        )
}

fn cleanup_exact_value(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    bounds_exact_memo: &mut HashMap<NodeId, Option<u128>>,
    semantic_exact_memo: &mut HashMap<NodeId, Option<u128>>,
) -> Option<u128> {
    if let Some(value) = bounds_exact_memo.get(&node_id) {
        return *value;
    }
    let exact = match &m.nodes[node_id as usize] {
        Node::Gate {
            op,
            operands,
            width: 1,
            ..
        } if matches!(
            op,
            GateOp::Eq | GateOp::Neq | GateOp::Lt | GateOp::Gt | GateOp::Le | GateOp::Ge
        ) && operands.len() == 2 =>
        {
            crate::gen::cone::obvious_unsigned_compare_result(m, *op, operands[0], operands[1])
        }
        Node::Gate {
            op: GateOp::Shl | GateOp::Shr,
            ..
        } => crate::gen::cone::prove_node_exact_value_from_bounds(m, node_id),
        _ => None,
    }
    .or_else(|| {
        if !cleanup_exact_proof_eligible(m, node_id, endpoint_memo) {
            return None;
        }
        // Cleanup is intentionally cheaper than the generator's own
        // intern-time exact prover: at this late stage we only want
        // cheap downstream-cleanliness wins, not a second pass over the
        // full small-set / bounds search surface.
        semantic_exact_value(m, node_id, endpoint_memo, semantic_exact_memo)
    });
    bounds_exact_memo.insert(node_id, exact);
    exact
}

fn semantic_exact_value(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    memo: &mut HashMap<NodeId, Option<u128>>,
) -> Option<u128> {
    if let Some(value) = memo.get(&node_id) {
        return *value;
    }
    if !cleanup_exact_proof_eligible(m, node_id, endpoint_memo) {
        memo.insert(node_id, None);
        return None;
    }
    let exact =
        semantic_cone_proof_with_limits(m, node_id, endpoint_memo, CLEANUP_SEMANTIC_LIMITS, None)
            .and_then(|proof| {
                let first = *proof.outputs.first()?;
                proof
                    .outputs
                    .iter()
                    .all(|&value| value == first)
                    .then_some(first)
            });
    memo.insert(node_id, exact);
    exact
}

fn cone_proof(
    m: &Module,
    node_id: NodeId,
    structural_memo: &mut HashMap<NodeId, StructuralSigId>,
    structural_ctx: &mut StructuralSignatureCtx,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    semantic_memo: &mut HashMap<NodeId, Option<SemanticConeProof>>,
    quotient: Option<&HashMap<FlopId, FlopId>>,
) -> ConeProof {
    if let Some(proof) = semantic_memo.get(&node_id) {
        if let Some(proof) = proof {
            return ConeProof::Semantic(proof.clone());
        }
    } else {
        let proof = semantic_cone_proof_with_limits(
            m,
            node_id,
            endpoint_memo,
            MERGE_SEMANTIC_LIMITS,
            quotient,
        );
        semantic_memo.insert(node_id, proof);
        if let Some(Some(proof)) = semantic_memo.get(&node_id) {
            return ConeProof::Semantic(proof.clone());
        }
    }

    ConeProof::Structural(structural_node_sig_id(
        m,
        node_id,
        structural_memo,
        structural_ctx,
        quotient,
    ))
}

/// Merge duplicate combinational gates after every cone is known.
///
/// This is the first bounded semantic fragment of the `EGraph` intent:
/// under `identity_mode = node-id` with requested/effective `EGraph`,
/// two gates can collapse even when their literal subgraph shapes
/// differ, and a gate can fold to an existing endpoint or constant,
/// provided ANVIL can prove they implement the same function over the
/// same canonical leaf endpoints.
///
/// The proof is intentionally bounded:
///
/// - first try the same endpoint-aware proof machinery used by state;
/// - use bounded small-support semantic truth tables when available;
/// - otherwise fall back to the already-normalized structural proof.
///
/// Returns the number of gates rewired to an earlier canonical node.
pub fn merge_equivalent_gates(m: &mut Module) -> u32 {
    use crate::config::{FactorizationLevel, IdentityMode};

    if m.identity_mode != IdentityMode::NodeId
        || m.effective_factorization_level() < FactorizationLevel::EGraph
    {
        return 0;
    }

    let mut canonical_by_sig: HashMap<GateSignature, NodeId> = HashMap::new();
    let mut structural_memo: HashMap<NodeId, StructuralSigId> = HashMap::new();
    let mut structural_ctx = StructuralSignatureCtx::default();
    let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
    let mut semantic_memo: HashMap<NodeId, Option<SemanticConeProof>> = HashMap::new();
    let mut node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    let mut removed = 0u32;

    for node_id in 0..m.nodes.len() as NodeId {
        let width = m.nodes[node_id as usize].width();
        let is_gate = matches!(&m.nodes[node_id as usize], Node::Gate { .. });
        let sig = GateSignature {
            width,
            proof: cone_proof(
                m,
                node_id,
                &mut structural_memo,
                &mut structural_ctx,
                &mut endpoint_memo,
                &mut semantic_memo,
                None,
            ),
        };
        if is_gate {
            if let Some(&canonical) = canonical_by_sig.get(&sig) {
                node_remap.insert(node_id, canonical);
                removed += 1;
            } else {
                canonical_by_sig.insert(sig, node_id);
            }
        } else {
            canonical_by_sig.entry(sig).or_insert(node_id);
        }
    }

    if removed == 0 {
        return 0;
    }

    removed -= prune_duplicate_introducing_add_mul_remaps(m, &mut node_remap);
    if removed == 0 {
        return 0;
    }

    for node in &mut m.nodes {
        if let Node::Gate { operands, .. } = node {
            for operand in operands.iter_mut() {
                rewrite_node_id_if_mapped(operand, &node_remap);
            }
        }
    }
    for (_, root) in &mut m.drives {
        rewrite_node_id_if_mapped(root, &node_remap);
    }
    rewrite_instance_inputs_from_partial_map(m, &node_remap);
    for flop in &mut m.flops {
        rewrite_flop_from_partial_map(flop, &node_remap);
    }

    rebuild_instance_tables(m);
    removed
}

/// Merge duplicate flops after every D-cone is known.
///
/// This is the first conservative stateful extension of the
/// NodeId-as-identity doctrine: a flop's identity cannot be decided
/// at birth because its semantics are not known until the worklist
/// finishes building `d`. After that point, if two flops have the
/// same emitted state signature (`width`, reset, and a D-cone with the
/// same canonical leaf endpoints plus the same currently-proven
/// functionality), every consumer of the duplicate Q can safely be
/// redirected to the canonical Q. Clock domain is part of that emitted
/// state signature: two equal-looking flops in different domains are
/// distinct state elements.
///
/// The pass is gated by the effective identity mode:
///
/// - `identity_mode = relaxed` or effective level `None` => no merge.
/// - `identity_mode = node-id` and effective level `>= Cse` => the
///   endpoint-preserving state-identity pass is enabled.
///
/// The merge is intentionally conservative:
///
/// - compares D-cones by a leaf-aware proof form: bounded small-support
///   semantic signature when available, otherwise a structural
///   signature over the already-normalized IR;
/// - treats exact reset-defined self-hold (`D == own Q` with a reset)
///   as a coinductive proof class whose own-Q endpoint is intentionally
///   alpha-renamed away;
/// - treats "same functionality" as the doctrine, while only claiming
///   the proof subset the current normalization ladder can actually
///   establish today;
/// - ignores construction-only provenance (`FlopKind`, cleared
///   `FlopMux` operands) once `d` exists;
/// - preserves first occurrence as canonical.
///
/// Returns the number of removed duplicate flops.
pub fn merge_equivalent_flops(m: &mut Module) -> u32 {
    use crate::config::{FactorizationLevel, IdentityMode};

    if m.flops.len() < 2 {
        return 0;
    }
    if m.identity_mode != IdentityMode::NodeId
        || m.effective_factorization_level() < FactorizationLevel::Cse
    {
        return 0;
    }

    let mut canonical_by_sig: HashMap<FlopSignature, FlopId> = HashMap::new();
    let mut structural_memo: HashMap<NodeId, StructuralSigId> = HashMap::new();
    let mut structural_ctx = StructuralSignatureCtx::default();
    let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
    let mut semantic_memo: HashMap<NodeId, Option<SemanticConeProof>> = HashMap::new();
    let mut old_to_canonical_old: Vec<FlopId> = (0..m.flops.len() as FlopId).collect();
    let mut removed = 0u32;

    for flop in &m.flops {
        let Some(d) = flop.d else {
            return 0;
        };
        let sig = FlopSignature {
            width: flop.width,
            clock_domain: m.flop_domain(flop.id),
            d: flop_d_signature(
                m,
                flop,
                d,
                &mut structural_memo,
                &mut structural_ctx,
                &mut endpoint_memo,
                &mut semantic_memo,
            ),
            reset_val: flop.reset_val,
            reset_kind: flop.reset_kind,
        };
        if let Some(&canonical_old) = canonical_by_sig.get(&sig) {
            old_to_canonical_old[flop.id as usize] = canonical_old;
            removed += 1;
        } else {
            canonical_by_sig.insert(sig, flop.id);
        }
    }

    finalize_flop_merge(m, old_to_canonical_old, removed)
}

/// Shared tail of every flop-merge pass.
///
/// Given `old_to_canonical_old` — each old `FlopId` mapped to the
/// canonical old `FlopId` it collapses into, with every canonical (and
/// every un-merged) flop mapping to itself — and the `removed` duplicate
/// count, this renumbers the surviving flops densely, rewires every Q
/// consumer / virtual flop dep / explicit clock-domain entry to the
/// canonical state element, and rebuilds the instance dedup tables.
///
/// The tail is partition-agnostic: it does not care *why* two flops were
/// proven equal, only which one survives. Both
/// [`merge_equivalent_flops`] (exact signature partition) and
/// [`merge_bisimilar_flops`] (bisimulation partition) build an
/// `old_to_canonical_old` map and hand it here. Extracting it keeps the
/// exact pass byte-identical while letting the bisimulation pass reuse the
/// same proven rewrite (`IDENTITY-DEEPENING.2a` design).
fn finalize_flop_merge(m: &mut Module, old_to_canonical_old: Vec<FlopId>, removed: u32) -> u32 {
    if removed == 0 {
        return 0;
    }

    let mut old_to_new: Vec<FlopId> = vec![0; m.flops.len()];
    let mut next_new: FlopId = 0;
    for old in 0..m.flops.len() as FlopId {
        if old_to_canonical_old[old as usize] == old {
            old_to_new[old as usize] = next_new;
            next_new += 1;
        }
    }
    for old in 0..m.flops.len() as FlopId {
        let canonical_old = old_to_canonical_old[old as usize];
        old_to_new[old as usize] = old_to_new[canonical_old as usize];
    }

    let mut q_node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    for old in 0..m.flops.len() as FlopId {
        let canonical_old = old_to_canonical_old[old as usize];
        if canonical_old != old {
            q_node_remap.insert(m.flops[old as usize].q, m.flops[canonical_old as usize].q);
        }
    }

    let old_flops = std::mem::take(&mut m.flops);
    let mut new_flops: Vec<Flop> = Vec::with_capacity(old_flops.len() - removed as usize);
    for mut flop in old_flops {
        let old = flop.id as usize;
        if old_to_canonical_old[old] != flop.id {
            continue;
        }
        flop.id = old_to_new[old];
        rewrite_flop_from_partial_map(&mut flop, &q_node_remap);
        new_flops.push(flop);
    }
    m.flops = new_flops;

    for node in &mut m.nodes {
        match node {
            Node::PrimaryInput { .. }
            | Node::Constant { .. }
            | Node::InstanceOutput { .. }
            | Node::MemRead { .. }
            | Node::FsmOut { .. } => {}
            Node::FlopQ { flop, .. } => {
                *flop = old_to_new[*flop as usize];
            }
            Node::Gate { operands, deps, .. } => {
                for operand in operands.iter_mut() {
                    rewrite_node_id_if_mapped(operand, &q_node_remap);
                }
                deps.remap_flop_virtuals(&old_to_new);
            }
        }
    }

    for (_, root) in &mut m.drives {
        rewrite_node_id_if_mapped(root, &q_node_remap);
    }
    rewrite_instance_inputs_from_partial_map(m, &q_node_remap);
    for flop in &mut m.flops {
        rewrite_flop_from_partial_map(flop, &q_node_remap);
    }

    remap_explicit_flop_domains_after_merge(m, &old_to_canonical_old, &old_to_new);
    rebuild_instance_tables(m);
    removed
}

fn remap_explicit_flop_domains_after_merge(
    m: &mut Module,
    old_to_canonical_old: &[FlopId],
    old_to_new: &[FlopId],
) {
    if m.flop_domains.is_empty() {
        return;
    }

    let old_domains = std::mem::take(&mut m.flop_domains);
    let mut new_domains = BTreeMap::new();
    for old in 0..old_to_canonical_old.len() {
        let old_id = old as FlopId;
        if old_to_canonical_old[old] != old_id {
            continue;
        }
        if let Some(domain) = old_domains.get(&old_id).copied() {
            new_domains.insert(old_to_new[old], domain);
        }
    }
    m.flop_domains = new_domains;
}

fn flop_d_signature(
    m: &Module,
    flop: &Flop,
    d: NodeId,
    structural_memo: &mut HashMap<NodeId, StructuralSigId>,
    structural_ctx: &mut StructuralSignatureCtx,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
    semantic_memo: &mut HashMap<NodeId, Option<SemanticConeProof>>,
) -> FlopDSignature {
    if flop.reset_kind != ResetKind::None && d == flop.q {
        return FlopDSignature::ResetDefinedSelfHold;
    }

    FlopDSignature::Cone(cone_proof(
        m,
        d,
        structural_memo,
        structural_ctx,
        endpoint_memo,
        semantic_memo,
        None,
    ))
}

/// Bucket-size cap for the bisimulation partition refinement.
///
/// Only `(width, reset_kind, reset_val, clock_domain)` buckets with at most
/// this many flops are refined; larger buckets fall back to the exact
/// [`merge_equivalent_flops`] pass only. This bounds the `O(k² · iters)`
/// refinement (`iters ≤ k`) on pathological modules without ever silently
/// dropping a candidate — the over-cap bucket simply keeps the conservative
/// (exact-only) result. The cap is deliberately generous: a generated leaf
/// rarely holds this many flops sharing one exact `(width, reset, domain)`
/// shape, so it almost never triggers
/// (`docs/decisions/0007-identity-deepening-first-extension.md`).
const N_BISIM_FLOPS: usize = 64;

/// Stable, total discriminant for `ResetKind` so it can key the bisimulation
/// bucket map deterministically (`ResetKind` is `Hash`/`Eq` but not `Ord`).
fn reset_kind_discriminant(kind: ResetKind) -> u8 {
    match kind {
        ResetKind::None => 0,
        ResetKind::Sync => 1,
        ResetKind::Async => 2,
    }
}

/// Merge flops proven sequentially equivalent by bounded bisimulation.
///
/// This is the first `IDENTITY-DEEPENING` extension (decision `0007`): a
/// **default-off, opt-in** greatest-fixpoint partition refinement that lifts
/// the recorded mutually-recursive-register / non-exact-feedback no-merge
/// boundary (`reset-defined-self-hold-flop-identity`) at the flop level. It
/// strictly generalizes — and never retires — the exact self-hold and
/// same-endpoint D-cone classes already merged by
/// [`merge_equivalent_flops`].
///
/// Gating (all required):
///
/// - the opt-in `Module::bisimulation_flop_merge` knob (default `false`, so
///   emitted RTL stays byte-identical unless explicitly requested);
/// - `identity_mode = node-id` with effective `factorization_level`
///   `e-graph` (the same rung that already gates semantic gate merge);
/// - at least two flops, all with a settled D-cone.
///
/// Algorithm (Kanellakis–Smolka coarsest stable partition):
///
/// 1. **Base case.** Bucket flops by `(width, reset_kind, reset_val,
///    clock_domain)`. Different buckets are never identified — a reset-value
///    mismatch fails the `t = 0` base case and a domain mismatch is unsound.
///    **Resetless flops (`reset_kind = None`) are pinned as singletons**:
///    with no reset there is no provable equal initial state, so a
///    state correspondence has no base case and bisimulation cannot soundly
///    fire (this preserves the `reset-defined-self-hold-flop-identity`
///    boundary — the exact pass keeps resetless self-holds apart via
///    concrete `FlopQ` endpoint identity, which quotienting would erase).
/// 2. **Refinement step.** Within a refinable bucket, keep two flops in one
///    class iff their D-cones — with every `FlopQ` endpoint rewritten to its
///    *current class representative* (the quotient signature) — are proven
///    equal by the existing bounded endpoint-preserving proof over the
///    quotient endpoint set. Split classes whose members disagree; repeat
///    until no class splits.
/// 3. **Soundness (coinduction).** At the fixpoint the partition is a
///    bisimulation: reset makes every class's members equal at `t = 0`, and
///    the stable quotient transition preserves equality, so corresponding
///    `Q`s are equal for all time. Merging them is observationally sound.
///
/// Over-budget D-cones (beyond the 12-bit / 128-node / 131072-work
/// `MERGE_SEMANTIC_LIMITS`) take the structural fallback inside `cone_proof`;
/// a candidate that cannot be discharged simply stays split (never a guess).
///
/// Returns the number of removed duplicate flops; the surviving rewrite is
/// the shared [`finalize_flop_merge`].
pub fn merge_bisimilar_flops(m: &mut Module) -> u32 {
    use crate::config::{FactorizationLevel, IdentityMode};

    if !m.bisimulation_flop_merge {
        return 0;
    }
    if m.flops.len() < 2 {
        return 0;
    }
    if m.identity_mode != IdentityMode::NodeId
        || m.effective_factorization_level() < FactorizationLevel::EGraph
    {
        return 0;
    }
    // Every flop must have a settled D-cone (mirrors `merge_equivalent_flops`).
    if m.flops.iter().any(|flop| flop.d.is_none()) {
        return 0;
    }

    // The coarsest stable bisimulation partition of `m`'s flops. `None` when
    // no bucket is refinable (the exact pass already settled everything ⇒ no
    // bisimulation merge). Factored into `bisimulation_partition` so the
    // cross-module whole-module sequential-equivalence proof
    // (`IDENTITY-DEEPENING.3b`) can reuse the identical refinement on a
    // combined module without duplicating it.
    let classes = match bisimulation_partition(m) {
        Some(classes) => classes,
        None => return 0,
    };

    // Collapse each converged class onto its representative (min `FlopId`).
    let mut old_to_canonical_old: Vec<FlopId> = (0..m.flops.len() as FlopId).collect();
    let mut removed = 0u32;
    for class in &classes {
        let rep = *class.iter().min().expect("partition class is non-empty");
        for &flop in class {
            if flop != rep {
                old_to_canonical_old[flop as usize] = rep;
                removed += 1;
            }
        }
    }

    finalize_flop_merge(m, old_to_canonical_old, removed)
}

/// Compute the coarsest stable bisimulation partition of `m`'s flops by
/// greatest-fixpoint refinement (the core of [`merge_bisimilar_flops`],
/// factored out so the cross-module whole-leaf-module sequential-equivalence
/// proof can reuse the *identical* refinement on a combined module —
/// `IDENTITY-DEEPENING.3b`).
///
/// Returns each equivalence class as an ascending-`FlopId` `Vec`, the classes
/// in deterministic order. Returns `None` when no `(width, reset_kind,
/// reset_val, domain)` bucket is reset-defined with at least two flops inside
/// the `N_BISIM_FLOPS` cap — i.e. nothing the exact pass has not already
/// settled, so [`merge_bisimilar_flops`] returns `0` without touching the
/// module (preserving its byte-identical behaviour).
///
/// **Non-mutating.** Callers must ensure every flop has a settled D-cone.
/// Resetless flops are pinned to singleton classes: with no reset there is no
/// provable equal initial state, so a state correspondence has no base case.
fn bisimulation_partition(m: &Module) -> Option<Vec<Vec<FlopId>>> {
    // Base case: bucket by (width, reset_kind, reset_val, clock_domain).
    let mut buckets: BTreeMap<(u32, u8, u128, u32), Vec<FlopId>> = BTreeMap::new();
    for flop in &m.flops {
        let key = (
            flop.width,
            reset_kind_discriminant(flop.reset_kind),
            flop.reset_val,
            m.flop_domain(flop.id),
        );
        buckets.entry(key).or_default().push(flop.id);
    }

    // Initial partition. A bucket is refinable only when it is reset-defined
    // (discriminant != 0), has at least two flops, and is within the cap;
    // every other flop is pinned to its own singleton class (never merged
    // here). `classes` is built in BTreeMap key order, each class in
    // ascending `FlopId` order, so the whole pass is deterministic.
    let mut classes: Vec<Vec<FlopId>> = Vec::new();
    let mut has_refinable = false;
    for ((_, reset_disc, _, _), bucket) in &buckets {
        let reset_defined = *reset_disc != reset_kind_discriminant(ResetKind::None);
        if reset_defined && bucket.len() >= 2 && bucket.len() <= N_BISIM_FLOPS {
            has_refinable = true;
            classes.push(bucket.clone());
        } else {
            for &flop in bucket {
                classes.push(vec![flop]);
            }
        }
    }
    if !has_refinable {
        return None;
    }

    // Greatest-fixpoint refinement: split until no class splits.
    loop {
        // `rep_map` covers ALL flops: each flop -> the min `FlopId` in its
        // class (its quotient class representative).
        let mut rep_map: HashMap<FlopId, FlopId> = HashMap::new();
        for class in &classes {
            let rep = *class.iter().min().expect("partition class is non-empty");
            for &flop in class {
                rep_map.insert(flop, rep);
            }
        }

        // GOTCHA (decision 0007 / `.2a`): the structural / endpoint /
        // semantic memos are `NodeId`-keyed and assume a FIXED endpoint
        // identity. The class map changes between iterations, so a stale
        // memo would be unsound — rebuild all four fresh each iteration.
        let mut structural_memo: HashMap<NodeId, StructuralSigId> = HashMap::new();
        let mut structural_ctx = StructuralSignatureCtx::default();
        let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
        let mut semantic_memo: HashMap<NodeId, Option<SemanticConeProof>> = HashMap::new();

        let mut next_classes: Vec<Vec<FlopId>> = Vec::with_capacity(classes.len());
        let mut any_split = false;

        for class in &classes {
            if class.len() < 2 {
                next_classes.push(class.clone());
                continue;
            }

            // Quotient D-signature per member (every `FlopQ` -> class rep).
            let signed: Vec<(FlopId, ConeProof)> = class
                .iter()
                .map(|&flop_id| {
                    let d = m.flops[flop_id as usize]
                        .d
                        .expect("d present (caller checks settled D-cones)");
                    let proof = cone_proof(
                        m,
                        d,
                        &mut structural_memo,
                        &mut structural_ctx,
                        &mut endpoint_memo,
                        &mut semantic_memo,
                        Some(&rep_map),
                    );
                    (flop_id, proof)
                })
                .collect();

            // Deterministic, order-stable grouping by signature equality:
            // members keep their ascending-`FlopId` order; each joins the
            // first existing group with an equal signature, else opens a new
            // one. (No `HashMap` over signatures — that would leak iteration
            // order into the emitted RTL.)
            let mut groups: Vec<(ConeProof, Vec<FlopId>)> = Vec::new();
            for (flop_id, proof) in signed {
                if let Some(slot) = groups.iter_mut().find(|(sig, _)| *sig == proof) {
                    slot.1.push(flop_id);
                } else {
                    groups.push((proof, vec![flop_id]));
                }
            }

            if groups.len() > 1 {
                any_split = true;
            }
            for (_, members) in groups {
                next_classes.push(members);
            }
        }

        classes = next_classes;
        if !any_split {
            break;
        }
    }

    Some(classes)
}

/// Union flop-count cap for the cross-module sequential-equivalence proof.
///
/// `modules_sequentially_equivalent` materializes a combined module holding both
/// candidates' flops, so the bisimulation refinement on that union is bounded by
/// `O(k² · iters)` with `k = a.flops + b.flops`. Pairs whose combined flop count
/// exceeds this cap are skipped (the pair conservatively fails to merge, never a
/// guess). Mirrors [`N_BISIM_FLOPS`]; chosen so every combined `(width, reset,
/// domain)` bucket also stays within the per-bucket [`N_BISIM_FLOPS`] refinement
/// cap (`IDENTITY-DEEPENING.3b`, decision `0008`).
const N_BISIM_MODULE_FLOPS: usize = 64;

/// Whether `m` is inside the first-cut scope of the whole-leaf-module
/// sequential-equivalence proof (`IDENTITY-DEEPENING.3b`, decision `0008`).
///
/// Eligible: a **stateful flops-only leaf module** — it has local flops, every
/// flop has a settled D-cone and a real reset (resetless flops have no `t = 0`
/// base case, so a cross-module state correspondence is unprovable and the
/// module is conservatively skipped, carrying the `0007`/`.2b` resetless
/// boundary forward), and it has no memories, FSMs, child instances, width
/// parameter, packed-aggregate projection, or explicit multi-clock domains.
/// Each exclusion is a separately-recorded boundary / named future leaf, none
/// retired.
pub(crate) fn sequential_leaf_eligible(m: &Module) -> bool {
    m.has_local_flops()
        && !m.has_local_memories()
        && !m.has_local_fsms()
        && m.instances.is_empty()
        && m.param_env.is_none()
        && m.aggregate_layout.is_none()
        && m.clock_domains.is_empty()
        && m.flops
            .iter()
            .all(|flop| flop.reset_kind != ResetKind::None)
        && m.flops.iter().all(|flop| flop.d.is_some())
}

/// A sorted `(PortId, width)` port shape (one side of a module interface).
type PortShape = Vec<(PortId, u32)>;

/// `(sorted inputs, sorted outputs)` keyed by `(PortId, width)`. Two modules
/// with equal keys have identical public interfaces, so rewriting instances of
/// one to the other preserves every parent-side port-id binding (the same
/// interface base case `dedup_semantic_modules` enforces).
fn module_port_key(m: &Module) -> (PortShape, PortShape) {
    let mut inputs: PortShape = m.inputs.iter().map(|p| (p.id, p.width)).collect();
    let mut outputs: PortShape = m.outputs.iter().map(|p| (p.id, p.width)).collect();
    inputs.sort_unstable();
    outputs.sort_unstable();
    (inputs, outputs)
}

/// Shift every `NodeId` inside a `FlopMux` by `node_offset` (used when copying
/// module `b`'s flops into the combined proof module). The proof itself never
/// reads `FlopMux`, but keeping the combined module internally consistent avoids
/// dangling ids if any future reader does.
fn shift_flop_mux(mux: &FlopMux, node_offset: NodeId) -> FlopMux {
    match mux {
        FlopMux::None => FlopMux::None,
        FlopMux::OneHot(arms) => FlopMux::OneHot(
            arms.iter()
                .map(|arm| MuxArm {
                    data: arm.data + node_offset,
                    sel: arm.sel + node_offset,
                })
                .collect(),
        ),
        FlopMux::Encoded { sel, data } => FlopMux::Encoded {
            sel: sel + node_offset,
            data: data.iter().map(|d| d + node_offset).collect(),
        },
    }
}

/// Materialize the temporary **combined module** `a.nodes ++ b.nodes` /
/// `a.flops ++ b.flops` used by the cross-module proof.
///
/// `b`'s `NodeId`s are offset by `a.nodes.len()` and its `FlopId`s by
/// `a.flops.len()`; every operand / `FlopQ` / flop `d`/`q`/`mux` reference is
/// remapped accordingly. Crucially, `b`'s `PrimaryInput { port, width }` nodes
/// keep their `port`, so A's and B's primary inputs **unify for free** in the
/// shared `LeafEndpoint::PrimaryInput { port, width }` vocabulary — that is what
/// makes a single bisimulation class span flops from *both* modules
/// (`IDENTITY-DEEPENING.3b.1`). Only the fields the proof reads (`nodes`,
/// `flops`) are populated; everything else is `Module::default()`.
fn build_combined_module(a: &Module, b: &Module) -> Module {
    let node_offset = a.nodes.len() as NodeId;
    let flop_offset = a.flops.len() as FlopId;

    let mut nodes: Vec<Node> = a.nodes.clone();
    for node in &b.nodes {
        nodes.push(match node {
            Node::PrimaryInput { port, width } => Node::PrimaryInput {
                port: *port,
                width: *width,
            },
            Node::Constant { width, value } => Node::Constant {
                width: *width,
                value: *value,
            },
            Node::FlopQ { flop, width } => Node::FlopQ {
                flop: flop + flop_offset,
                width: *width,
            },
            Node::InstanceOutput {
                instance,
                port,
                width,
            } => Node::InstanceOutput {
                instance: *instance,
                port: *port,
                width: *width,
            },
            Node::MemRead { mem, width } => Node::MemRead {
                mem: *mem,
                width: *width,
            },
            Node::FsmOut { fsm, width } => Node::FsmOut {
                fsm: *fsm,
                width: *width,
            },
            Node::Gate {
                op,
                operands,
                width,
                deps,
            } => Node::Gate {
                op: *op,
                operands: operands.iter().map(|o| o + node_offset).collect(),
                width: *width,
                deps: deps.clone(),
            },
        });
    }

    let mut flops: Vec<Flop> = a.flops.clone();
    for flop in &b.flops {
        flops.push(Flop {
            id: flop.id + flop_offset,
            width: flop.width,
            d: flop.d.map(|d| d + node_offset),
            q: flop.q + node_offset,
            reset_val: flop.reset_val,
            reset_kind: flop.reset_kind,
            kind: flop.kind,
            mux: shift_flop_mux(&flop.mux, node_offset),
        });
    }

    Module {
        nodes,
        flops,
        ..Module::default()
    }
}

/// Prove two stateful flops-only leaf modules observationally (sequentially)
/// equivalent by a **bounded cross-module bisimulation**
/// (`IDENTITY-DEEPENING.3b`, decision `0008`).
///
/// This is the sequential generalization of the combinational
/// `dedup_semantic_modules` truth-table proof: it lifts the flop-level
/// greatest-fixpoint partition refinement ([`bisimulation_partition`]) to the
/// disjoint union of both modules' flops, then proves every output cone equal
/// under the resulting state quotient.
///
/// Steps (decision `0008`):
///
/// 1. **Interface base case.** Identical input & output ports by
///    `(PortId, width)` (so an instance rewrite preserves every parent-side
///    binding). Both modules must be [`sequential_leaf_eligible`], and the
///    combined flop count within [`N_BISIM_MODULE_FLOPS`].
/// 2. **State correspondence.** Materialize the combined module (A's and B's
///    primary inputs unified by `(PortId, width)`) and run the coarsest stable
///    bisimulation partition on its union state. Same-class flops (possibly from
///    both modules) provably hold equal values for all time (reset base case +
///    stable quotient transition — the `.2b` coinduction, now across two
///    machines).
/// 3. **Output equality.** For every output port, A's drive cone equals B's
///    drive cone under the *final* quotient (one shared structural interner makes
///    the proof ids mutually comparable; the fixed quotient keeps the
///    `NodeId`-keyed memos sound across all ports).
///
/// Returns `true` only when all three hold. Any over-budget cone, interface
/// mismatch, resetless flop, or unprovable correspondence ⇒ `false` (never a
/// guess). Pure / non-mutating.
pub(crate) fn modules_sequentially_equivalent(a: &Module, b: &Module) -> bool {
    if !sequential_leaf_eligible(a) || !sequential_leaf_eligible(b) {
        return false;
    }
    if a.flops.len() + b.flops.len() > N_BISIM_MODULE_FLOPS {
        return false;
    }
    if module_port_key(a) != module_port_key(b) {
        return false;
    }

    // 2-3. Combined module + coarsest stable bisimulation on the union state.
    let combined = build_combined_module(a, b);
    let classes = match bisimulation_partition(&combined) {
        Some(classes) => classes,
        None => return false,
    };
    let mut rep_map: HashMap<FlopId, FlopId> = HashMap::new();
    for class in &classes {
        let rep = *class.iter().min().expect("partition class is non-empty");
        for &flop in class {
            rep_map.insert(flop, rep);
        }
    }

    // 4. Per-output-port drive-cone equality under the final quotient.
    let node_offset = a.nodes.len() as NodeId;
    let a_drives: BTreeMap<PortId, NodeId> = a.drives.iter().copied().collect();
    let b_drives: BTreeMap<PortId, NodeId> = b.drives.iter().copied().collect();

    let mut structural_memo: HashMap<NodeId, StructuralSigId> = HashMap::new();
    let mut structural_ctx = StructuralSignatureCtx::default();
    let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
    let mut semantic_memo: HashMap<NodeId, Option<SemanticConeProof>> = HashMap::new();

    for port in &a.outputs {
        let (Some(&a_node), Some(&b_node)) = (a_drives.get(&port.id), b_drives.get(&port.id))
        else {
            return false;
        };
        let proof_a = cone_proof(
            &combined,
            a_node,
            &mut structural_memo,
            &mut structural_ctx,
            &mut endpoint_memo,
            &mut semantic_memo,
            Some(&rep_map),
        );
        let proof_b = cone_proof(
            &combined,
            b_node + node_offset,
            &mut structural_memo,
            &mut structural_ctx,
            &mut endpoint_memo,
            &mut semantic_memo,
            Some(&rep_map),
        );
        if proof_a != proof_b {
            return false;
        }
    }

    true
}

/// Merge duplicate generated FSM blocks after their selector cones are known.
///
/// This is a deterministic-block extension of the same state-identity
/// discipline as [`merge_equivalent_flops`]. A generated FSM has a
/// reset-defined initial state, an explicit transition table, and an
/// explicit Moore-output table. Under `identity_mode = node-id`, two
/// FSMs with the same table/encoding/output signature and the same
/// selector proof represent the same state machine, so consumers of the
/// duplicate `FsmOut` can be redirected to the canonical block.
///
/// Memories deliberately do not use this pass: their contents are not
/// reset-defined in ANVIL's current inferrable-memory template.
pub fn merge_equivalent_fsms(m: &mut Module) -> u32 {
    use crate::config::{FactorizationLevel, IdentityMode};

    if m.fsms.len() < 2 {
        return 0;
    }
    if m.identity_mode != IdentityMode::NodeId
        || m.effective_factorization_level() < FactorizationLevel::Cse
    {
        return 0;
    }

    let mut canonical_by_sig: HashMap<FsmSignature, FsmId> = HashMap::new();
    let mut structural_memo: HashMap<NodeId, StructuralSigId> = HashMap::new();
    let mut structural_ctx = StructuralSignatureCtx::default();
    let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
    let mut semantic_memo: HashMap<NodeId, Option<SemanticConeProof>> = HashMap::new();
    let mut old_to_canonical_old: Vec<FsmId> = (0..m.fsms.len() as FsmId).collect();
    let mut removed = 0u32;

    for fsm in &m.fsms {
        // CAPABILITY-BREADTH-EXPANSION.2b (decision 0024): a Mealy FSM's
        // output depends on `sel`, so its identity would also have to key on
        // the full `mealy_outputs` table. Until that keying lands, Mealy FSMs
        // are conservatively excluded from the merge (each stays its own
        // canonical block) — sound, nothing retired (the memories-stay-
        // state-by-instance precedent). Moore FSMs are unaffected.
        if fsm.is_mealy() {
            continue;
        }
        let sig = FsmSignature {
            num_states: fsm.num_states,
            encoding: fsm.encoding,
            sel: cone_proof(
                m,
                fsm.sel,
                &mut structural_memo,
                &mut structural_ctx,
                &mut endpoint_memo,
                &mut semantic_memo,
                None,
            ),
            sel_width: fsm.sel_width,
            transitions: fsm.transitions.clone(),
            outputs: fsm.outputs.clone(),
            out_width: fsm.out_width,
        };
        if let Some(&canonical_old) = canonical_by_sig.get(&sig) {
            old_to_canonical_old[fsm.id as usize] = canonical_old;
            removed += 1;
        } else {
            canonical_by_sig.insert(sig, fsm.id);
        }
    }

    if removed == 0 {
        return 0;
    }

    let mut old_to_new: Vec<FsmId> = vec![0; m.fsms.len()];
    let mut next_new: FsmId = 0;
    for old in 0..m.fsms.len() as FsmId {
        if old_to_canonical_old[old as usize] == old {
            old_to_new[old as usize] = next_new;
            next_new += 1;
        }
    }
    for old in 0..m.fsms.len() as FsmId {
        let canonical_old = old_to_canonical_old[old as usize];
        old_to_new[old as usize] = old_to_new[canonical_old as usize];
    }

    let mut first_fsm_out_by_old_fsm: Vec<Option<NodeId>> = vec![None; m.fsms.len()];
    for (node_id, node) in m.nodes.iter().enumerate() {
        if let Node::FsmOut { fsm, .. } = node {
            let slot = *fsm as usize;
            if slot < first_fsm_out_by_old_fsm.len() && first_fsm_out_by_old_fsm[slot].is_none() {
                first_fsm_out_by_old_fsm[slot] = Some(node_id as NodeId);
            }
        }
    }

    let mut fsm_out_node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    for (node_id, node) in m.nodes.iter().enumerate() {
        let Node::FsmOut { fsm, .. } = node else {
            continue;
        };
        let old = *fsm;
        let canonical_old = old_to_canonical_old[old as usize];
        let Some(canonical_node) = first_fsm_out_by_old_fsm
            .get(canonical_old as usize)
            .and_then(|node| *node)
        else {
            continue;
        };
        let node_id = node_id as NodeId;
        if canonical_node != node_id {
            fsm_out_node_remap.insert(node_id, canonical_node);
        }
    }

    let old_fsms = std::mem::take(&mut m.fsms);
    let mut new_fsms = Vec::with_capacity(old_fsms.len() - removed as usize);
    for mut fsm in old_fsms {
        let old = fsm.id as usize;
        if old_to_canonical_old[old] != fsm.id {
            continue;
        }
        fsm.id = old_to_new[old];
        rewrite_node_id_if_mapped(&mut fsm.sel, &fsm_out_node_remap);
        new_fsms.push(fsm);
    }
    m.fsms = new_fsms;

    for node in &mut m.nodes {
        match node {
            Node::FsmOut { fsm, .. } => {
                *fsm = old_to_new[*fsm as usize];
            }
            Node::Gate { operands, deps, .. } => {
                for operand in operands.iter_mut() {
                    rewrite_node_id_if_mapped(operand, &fsm_out_node_remap);
                }
                deps.remap_fsm_virtuals(&old_to_new);
            }
            Node::PrimaryInput { .. }
            | Node::Constant { .. }
            | Node::InstanceOutput { .. }
            | Node::FlopQ { .. }
            | Node::MemRead { .. } => {}
        }
    }

    for (_, root) in &mut m.drives {
        rewrite_node_id_if_mapped(root, &fsm_out_node_remap);
    }
    rewrite_instance_inputs_from_partial_map(m, &fsm_out_node_remap);
    for flop in &mut m.flops {
        rewrite_flop_from_partial_map(flop, &fsm_out_node_remap);
    }
    for mem in &mut m.memories {
        rewrite_node_id_if_mapped(&mut mem.we, &fsm_out_node_remap);
        rewrite_node_id_if_mapped(&mut mem.waddr, &fsm_out_node_remap);
        rewrite_node_id_if_mapped(&mut mem.wdata, &fsm_out_node_remap);
        rewrite_node_id_if_mapped(&mut mem.raddr, &fsm_out_node_remap);
    }

    rebuild_instance_tables(m);
    removed
}

/// Revisit built gates using the final graph and fold any gate whose
/// current value is provably exact.
///
/// Some exact proofs only become visible once later sharing/remap steps
/// settle the subgraph a gate sees. This pass intentionally runs after
/// construction for downstream-tool cleanliness: exact constants and
/// constant-selector muxes are not useful stress by themselves, and
/// leaving them behind turns clean-tool output into a warning problem.
///
/// Returns the number of gates simplified in place or rewired away.
pub fn fold_proven_gates(m: &mut Module) -> u32 {
    let mut node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    let mut endpoint_memo: HashMap<NodeId, BTreeSet<LeafEndpoint>> = HashMap::new();
    let mut bounds_exact_memo: HashMap<NodeId, Option<u128>> = HashMap::new();
    let mut semantic_exact_memo: HashMap<NodeId, Option<u128>> = HashMap::new();
    let mut simplified = 0u32;

    for node_id in 0..m.nodes.len() as NodeId {
        let snapshot = match &m.nodes[node_id as usize] {
            Node::Gate {
                op,
                operands,
                width,
                ..
            } => Some((*op, operands.clone(), *width)),
            _ => None,
        };
        let Some((op, operands, width)) = snapshot else {
            continue;
        };

        if let Node::Gate {
            operands: current_operands,
            ..
        } = &mut m.nodes[node_id as usize]
        {
            for operand in current_operands.iter_mut() {
                rewrite_node_id_if_mapped(operand, &node_remap);
            }
        }

        let exact_value = cleanup_exact_value(
            m,
            node_id,
            &mut endpoint_memo,
            &mut bounds_exact_memo,
            &mut semantic_exact_memo,
        );

        if let Some(value) = exact_value {
            let value = value & bitmask(width);
            if !matches!(
                &m.nodes[node_id as usize],
                Node::Constant {
                    width: existing_width,
                    value: existing_value
                } if *existing_width == width && *existing_value == value
            ) {
                m.nodes[node_id as usize] = Node::Constant { width, value };
                simplified += 1;
            }
            continue;
        }

        if op == GateOp::Mux && operands.len() == 3 {
            let (sel, on_true, on_false) = match &m.nodes[node_id as usize] {
                Node::Gate { operands, .. } => (operands[0], operands[1], operands[2]),
                _ => continue,
            };
            if let Some(sel_value) = cleanup_exact_value(
                m,
                sel,
                &mut endpoint_memo,
                &mut bounds_exact_memo,
                &mut semantic_exact_memo,
            ) {
                let chosen = match sel_value {
                    0 => on_false,
                    1 => on_true,
                    _ => continue,
                };
                node_remap.insert(node_id, chosen);
                simplified += 1;
            }
        }
    }

    if !node_remap.is_empty() {
        simplified -= prune_duplicate_introducing_add_mul_remaps(m, &mut node_remap);
        for node in &mut m.nodes {
            if let Node::Gate { operands, .. } = node {
                for operand in operands.iter_mut() {
                    rewrite_node_id_if_mapped(operand, &node_remap);
                }
            }
        }
        for (_, root) in &mut m.drives {
            rewrite_node_id_if_mapped(root, &node_remap);
        }
        rewrite_instance_inputs_from_partial_map(m, &node_remap);
        for flop in &mut m.flops {
            rewrite_flop_from_partial_map(flop, &node_remap);
        }
    }

    if simplified > 0 || !node_remap.is_empty() {
        rebuild_instance_tables(m);
    }
    simplified
}

/// Late mixed-constant aggregation for settled associative gates.
///
/// Intern-time constant folding already combines mixed constants when a
/// gate is first created, but later remap passes can expose new
/// opportunities (`1 + x + inner`, where `inner` later collapses to
/// `1`). This pass revisits the settled graph and aggregates the
/// constant side again without relying on reconstruction.
pub fn fold_mixed_associative_constants(m: &mut Module) -> u32 {
    if m.effective_factorization_level() < FactorizationLevel::ConstantFold {
        return 0;
    }

    let mut node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    let mut simplified = 0u32;

    for node_id in 0..m.nodes.len() as NodeId {
        let snapshot = match &m.nodes[node_id as usize] {
            Node::Gate {
                op,
                operands,
                width,
                ..
            } if matches!(
                op,
                GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
            ) =>
            {
                Some((*op, operands.clone(), *width))
            }
            _ => None,
        };
        let Some((op, operands, width)) = snapshot else {
            continue;
        };

        if let Node::Gate {
            operands: current_operands,
            ..
        } = &mut m.nodes[node_id as usize]
        {
            for operand in current_operands.iter_mut() {
                rewrite_node_id_if_mapped(operand, &node_remap);
            }
        }

        let identity = match op {
            GateOp::Add | GateOp::Xor | GateOp::Or => 0,
            GateOp::Mul => 1,
            GateOp::And => bitmask(width),
            _ => unreachable!(),
        };

        let mut const_count = 0usize;
        let mut const_acc = identity;
        let mut dynamic: Vec<NodeId> = Vec::with_capacity(operands.len());
        for operand in operands {
            let operand = node_remap.get(&operand).copied().unwrap_or(operand);
            match &m.nodes[operand as usize] {
                Node::Constant {
                    width: const_width,
                    value,
                } if *const_width == width => {
                    const_count += 1;
                    const_acc = match op {
                        GateOp::And => const_acc & *value,
                        GateOp::Or => const_acc | *value,
                        GateOp::Xor => const_acc ^ *value,
                        GateOp::Add => const_acc.wrapping_add(*value) & bitmask(width),
                        GateOp::Mul => const_acc.wrapping_mul(*value) & bitmask(width),
                        _ => unreachable!(),
                    };
                }
                _ => dynamic.push(operand),
            }
        }

        if const_count < 2 {
            continue;
        }

        let keep_aggregate = const_acc != identity || dynamic.is_empty();
        if keep_aggregate {
            let (cid, _is_new) = m.intern_constant(width, const_acc);
            dynamic.push(cid);
        }
        if m.effective_factorization_level() >= FactorizationLevel::Commutative {
            dynamic.sort_unstable();
        }

        match dynamic.len() {
            0 => unreachable!("all-constant associative gate should have kept an aggregate"),
            1 => {
                node_remap.insert(node_id, dynamic[0]);
                simplified += 1;
            }
            _ => {
                if let Node::Gate {
                    operands: current_operands,
                    ..
                } = &mut m.nodes[node_id as usize]
                {
                    if *current_operands != dynamic {
                        *current_operands = dynamic;
                        simplified += 1;
                    }
                }
            }
        }
    }

    if !node_remap.is_empty() {
        simplified -= prune_duplicate_introducing_add_mul_remaps(m, &mut node_remap);
        for node in &mut m.nodes {
            if let Node::Gate { operands, .. } = node {
                for operand in operands.iter_mut() {
                    rewrite_node_id_if_mapped(operand, &node_remap);
                }
            }
        }
        for (_, root) in &mut m.drives {
            rewrite_node_id_if_mapped(root, &node_remap);
        }
        rewrite_instance_inputs_from_partial_map(m, &node_remap);
        for flop in &mut m.flops {
            rewrite_flop_from_partial_map(flop, &node_remap);
        }
    }

    if simplified > 0 || !node_remap.is_empty() {
        rebuild_instance_tables(m);
    }
    simplified
}

/// Re-run the associative layer after post-construction remap passes.
///
/// Intern-time flattening keeps the live IR free of legal nested
/// associative shapes, but later remap passes (for example semantic
/// gate merging or constant-selector mux rewrites) can reintroduce
/// `Add(x, Add(y, z))`-style forms by changing which already-built
/// node an operand points at. This pass restores the same
/// same-op/same-width normal form in-place on the settled graph,
/// respecting the current duplicate policy for `Add` / `Mul`.
pub fn flatten_posthoc_associative_gates(m: &mut Module) -> u32 {
    if m.effective_factorization_level() < FactorizationLevel::Associative {
        return 0;
    }

    let mut node_remap: HashMap<NodeId, NodeId> = HashMap::new();
    let mut flattened = 0u32;

    for node_id in 0..m.nodes.len() as NodeId {
        let snapshot = match &m.nodes[node_id as usize] {
            Node::Gate {
                op,
                operands,
                width,
                ..
            } if matches!(
                op,
                GateOp::And | GateOp::Or | GateOp::Xor | GateOp::Add | GateOp::Mul
            ) =>
            {
                Some((*op, operands.clone(), *width))
            }
            _ => None,
        };
        let Some((op, operands, width)) = snapshot else {
            continue;
        };

        if let Node::Gate {
            operands: current_operands,
            ..
        } = &mut m.nodes[node_id as usize]
        {
            for operand in current_operands.iter_mut() {
                rewrite_node_id_if_mapped(operand, &node_remap);
            }
        }

        let mut flat = Vec::with_capacity(operands.len());
        let mut any_spliced = false;
        for operand_id in operands {
            let operand_id = node_remap.get(&operand_id).copied().unwrap_or(operand_id);
            match &m.nodes[operand_id as usize] {
                Node::Gate {
                    op: inner_op,
                    operands: inner_ops,
                    width: inner_w,
                    ..
                } if *inner_op == op && *inner_w == width => {
                    flat.extend(inner_ops.iter().copied());
                    any_spliced = true;
                }
                _ => flat.push(operand_id),
            }
        }

        let pre_normalize = flat.clone();
        match op {
            GateOp::And | GateOp::Or => {
                use std::collections::HashSet;
                let mut seen = HashSet::new();
                flat.retain(|id| seen.insert(*id));
            }
            GateOp::Xor => {
                use std::collections::{HashMap, HashSet};
                let mut counts: HashMap<NodeId, u32> = HashMap::new();
                for id in &flat {
                    *counts.entry(*id).or_insert(0) += 1;
                }
                flat.retain(|id| counts[id] % 2 == 1);
                let mut seen = HashSet::new();
                flat.retain(|id| seen.insert(*id));
            }
            GateOp::Add | GateOp::Mul => {
                if m.operand_duplication_rate < 1.0 {
                    use std::collections::HashMap;
                    let mut counts: HashMap<NodeId, u32> = HashMap::new();
                    for id in &flat {
                        *counts.entry(*id).or_insert(0) += 1;
                    }
                    if counts.values().any(|count| *count > 1) {
                        continue;
                    }
                }
            }
            _ => unreachable!(),
        }

        if !any_spliced && flat == pre_normalize {
            continue;
        }

        match flat.len() {
            0 => {
                debug_assert!(matches!(op, GateOp::Xor));
                m.nodes[node_id as usize] = Node::Constant { width, value: 0 };
                flattened += 1;
            }
            1 => {
                node_remap.insert(node_id, flat[0]);
                flattened += 1;
            }
            _ => {
                if let Node::Gate {
                    operands: current_operands,
                    ..
                } = &mut m.nodes[node_id as usize]
                {
                    *current_operands = flat;
                    flattened += 1;
                }
            }
        }
    }

    if !node_remap.is_empty() {
        for node in &mut m.nodes {
            if let Node::Gate { operands, .. } = node {
                for operand in operands.iter_mut() {
                    rewrite_node_id_if_mapped(operand, &node_remap);
                }
            }
        }
        for (_, root) in &mut m.drives {
            rewrite_node_id_if_mapped(root, &node_remap);
        }
        rewrite_instance_inputs_from_partial_map(m, &node_remap);
        for flop in &mut m.flops {
            rewrite_flop_from_partial_map(flop, &node_remap);
        }
    }

    if flattened > 0 || !node_remap.is_empty() {
        rebuild_instance_tables(m);
    }
    flattened
}

/// Compact `m.nodes` to only the nodes reachable from some root.
/// Returns the number of nodes removed (useful for the
/// `Metrics::nodes_compacted` counter).
///
/// See module docs for guarantees and non-goals.
pub fn compact_node_ids(m: &mut Module) -> u32 {
    let n = m.nodes.len();
    if n == 0 {
        return 0;
    }

    // 1. Mark reachable nodes by BFS from every surviving holder.
    //    Today that means:
    //    - every output drive-root
    //    - every instance input binding
    //
    //    Gates recurse through operands. A `FlopQ` leaf is the bridge
    //    into sequential state: once some live consumer reaches Q, the
    //    owning flop becomes live and its D / mux metadata join the walk.
    let mut reachable = vec![false; n];
    let mut reachable_flops = vec![false; m.flops.len()];
    let mut stack: Vec<NodeId> = Vec::new();
    let mark_node = |id: NodeId, reachable: &mut [bool], stack: &mut Vec<NodeId>| {
        if !reachable[id as usize] {
            reachable[id as usize] = true;
            stack.push(id);
        }
    };

    for (_, root) in &m.drives {
        mark_node(*root, &mut reachable, &mut stack);
    }
    for instance in &m.instances {
        for (_, node_id) in &instance.inputs {
            mark_node(*node_id, &mut reachable, &mut stack);
        }
    }

    while let Some(nid) = stack.pop() {
        match &m.nodes[nid as usize] {
            Node::Gate { operands, .. } => {
                // Operands are u32 — copy to avoid borrow issues.
                let ops: Vec<NodeId> = operands.clone();
                for op in ops {
                    mark_node(op, &mut reachable, &mut stack);
                }
            }
            Node::FlopQ { flop, .. } => {
                let flop_idx = *flop as usize;
                if reachable_flops[flop_idx] {
                    continue;
                }
                reachable_flops[flop_idx] = true;
                let flop = &m.flops[flop_idx];
                if let Some(d) = flop.d {
                    mark_node(d, &mut reachable, &mut stack);
                }
                match &flop.mux {
                    FlopMux::None => {}
                    FlopMux::OneHot(arms) => {
                        for arm in arms {
                            mark_node(arm.data, &mut reachable, &mut stack);
                            mark_node(arm.sel, &mut reachable, &mut stack);
                        }
                    }
                    FlopMux::Encoded { sel, data } => {
                        mark_node(*sel, &mut reachable, &mut stack);
                        for d in data {
                            mark_node(*d, &mut reachable, &mut stack);
                        }
                    }
                }
            }
            Node::MemRead { mem, .. } => {
                // Load-bearing (PHASE-6-ADVANCED-MOTIFS.2.1a): a
                // reachable MemRead keeps the memory's write/read
                // source cones alive, exactly as a reachable FlopQ
                // keeps its flop's D cone. Memories are never
                // dead-eliminated in Phase 6.2.1 (no unread memory
                // can arise), so the MemId is stable — no remap.
                let mem = &m.memories[*mem as usize];
                for src in [mem.we, mem.waddr, mem.wdata, mem.raddr] {
                    mark_node(src, &mut reachable, &mut stack);
                }
            }
            Node::FsmOut { fsm, .. } => {
                // Load-bearing (PHASE-6-ADVANCED-MOTIFS.3.2a): a
                // reachable FsmOut keeps the FSM's transition-select
                // source cone alive, exactly as a reachable MemRead
                // keeps the memory's address/data cones. FSMs are
                // never dead-eliminated in 6.3.2a (no generator yet;
                // and an FSM, like a memory, is never pruned), so the
                // FsmId is stable — no remap.
                let fsm = &m.fsms[*fsm as usize];
                mark_node(fsm.sel, &mut reachable, &mut stack);
            }
            Node::PrimaryInput { .. } | Node::Constant { .. } | Node::InstanceOutput { .. } => {}
        }
    }

    // 2. Early exit if nothing to remove.
    let removed = reachable.iter().filter(|b| !**b).count() as u32;
    if removed == 0 {
        return 0;
    }

    // 3. Build old-id → new-id mapping. Order-preserving so
    //    topological order is preserved: node at old index i with
    //    reachable[i] == true goes to the next available new
    //    index. Operands of any Gate keep `new_id < parent_new_id`
    //    because they had `old_id < parent_old_id` (IR invariant)
    //    and both survived.
    let mut old_to_new: HashMap<NodeId, NodeId> = HashMap::with_capacity(n);
    let mut next_new_id: NodeId = 0;
    for old_id in 0..n as NodeId {
        if reachable[old_id as usize] {
            old_to_new.insert(old_id, next_new_id);
            next_new_id += 1;
        }
    }
    let mut old_flop_to_new: Vec<FlopId> = (0..m.flops.len() as FlopId).collect();
    let mut next_new_flop: FlopId = 0;
    for old_flop in 0..m.flops.len() {
        if reachable_flops[old_flop] {
            old_flop_to_new[old_flop] = next_new_flop;
            next_new_flop += 1;
        }
    }

    // Helper to remap a NodeId. Panics if the id isn't in the map —
    // that would be a bookkeeping bug (caller held a reference to an
    // unreachable node, which means we failed to mark it reachable).
    let remap = |id: NodeId, map: &HashMap<NodeId, NodeId>| -> NodeId {
        *map.get(&id).unwrap_or_else(|| {
            panic!(
                "compact_node_ids: NodeId {} is referenced by a surviving holder \
                 but wasn't marked reachable — BFS or holder enumeration bug",
                id
            )
        })
    };

    // 4. Rewrite `m.nodes` in place: keep only reachable nodes, in
    //    order, remapping their inner NodeId references.
    let mut new_nodes: Vec<Node> = Vec::with_capacity(next_new_id as usize);
    for (old_id, node) in m.nodes.drain(..).enumerate() {
        if !reachable[old_id] {
            continue;
        }
        let remapped = match node {
            Node::PrimaryInput { port, width } => Node::PrimaryInput { port, width },
            Node::Constant { width, value } => Node::Constant { width, value },
            Node::FlopQ { flop, width } => Node::FlopQ {
                flop: old_flop_to_new[flop as usize],
                width,
            },
            Node::MemRead { mem, width } => Node::MemRead { mem, width },
            Node::FsmOut { fsm, width } => Node::FsmOut { fsm, width },
            Node::InstanceOutput {
                instance,
                port,
                width,
            } => Node::InstanceOutput {
                instance,
                port,
                width,
            },
            Node::Gate {
                op,
                operands,
                width,
                mut deps,
            } => {
                let new_operands: Vec<NodeId> = operands
                    .into_iter()
                    .map(|o| remap(o, &old_to_new))
                    .collect();
                deps.remap_flop_virtuals(&old_flop_to_new);
                Node::Gate {
                    op,
                    operands: new_operands,
                    width,
                    deps,
                }
            }
        };
        new_nodes.push(remapped);
    }
    m.nodes = new_nodes;

    // 5. Rewrite `m.drives`.
    for (_, root) in m.drives.iter_mut() {
        *root = remap(*root, &old_to_new);
    }
    for instance in &mut m.instances {
        for (_, node_id) in &mut instance.inputs {
            *node_id = remap(*node_id, &old_to_new);
        }
    }

    // 6. Rewrite flops, dropping any state element whose `Q` is not
    //    reachable from a surviving output cone.
    let old_flops = std::mem::take(&mut m.flops);
    let mut new_flops: Vec<Flop> = Vec::with_capacity(next_new_flop as usize);
    for (old_flop, mut flop) in old_flops.into_iter().enumerate() {
        if !reachable_flops[old_flop] {
            continue;
        }
        flop.id = old_flop_to_new[old_flop];
        rewrite_flop(&mut flop, &old_to_new, &remap);
        new_flops.push(flop);
    }
    m.flops = new_flops;
    remap_explicit_flop_domains_after_compaction(m, &reachable_flops, &old_flop_to_new);

    // 7. Rebuild dedup tables against the new NodeId space. Drop
    //    entries whose target was unreachable; remap surviving ones.
    let old_gate_instances = std::mem::take(&mut m.gate_instances);
    let mut new_gate_instances: HashMap<(GateOp, Vec<NodeId>, u32), Vec<NodeId>> =
        HashMap::with_capacity(old_gate_instances.len());
    for ((op, key_operands, width), ids) in old_gate_instances {
        // A key-operand being unreachable means this cache entry
        // points at a dead AST; skip the whole entry.
        if key_operands.iter().any(|id| !old_to_new.contains_key(id)) {
            continue;
        }
        let remapped_key_operands: Vec<NodeId> = key_operands
            .into_iter()
            .map(|id| remap(id, &old_to_new))
            .collect();
        let remapped_ids: Vec<NodeId> = ids
            .into_iter()
            .filter_map(|id| old_to_new.get(&id).copied())
            .collect();
        if remapped_ids.is_empty() {
            continue;
        }
        new_gate_instances.insert((op, remapped_key_operands, width), remapped_ids);
    }
    m.gate_instances = new_gate_instances;

    let old_const_instances = std::mem::take(&mut m.const_instances);
    let mut new_const_instances: HashMap<(u32, u128), Vec<NodeId>> =
        HashMap::with_capacity(old_const_instances.len());
    for (key, ids) in old_const_instances {
        let remapped_ids: Vec<NodeId> = ids
            .into_iter()
            .filter_map(|id| old_to_new.get(&id).copied())
            .collect();
        if remapped_ids.is_empty() {
            continue;
        }
        new_const_instances.insert(key, remapped_ids);
    }
    m.const_instances = new_const_instances;

    removed
}

fn remap_explicit_flop_domains_after_compaction(
    m: &mut Module,
    reachable_flops: &[bool],
    old_flop_to_new: &[FlopId],
) {
    if m.flop_domains.is_empty() {
        return;
    }

    let old_domains = std::mem::take(&mut m.flop_domains);
    let mut new_domains = BTreeMap::new();
    for (old, reachable) in reachable_flops.iter().copied().enumerate() {
        if !reachable {
            continue;
        }
        let old_id = old as FlopId;
        if let Some(domain) = old_domains.get(&old_id).copied() {
            new_domains.insert(old_flop_to_new[old], domain);
        }
    }
    m.flop_domains = new_domains;
}

fn rewrite_flop(
    flop: &mut Flop,
    map: &HashMap<NodeId, NodeId>,
    remap: &impl Fn(NodeId, &HashMap<NodeId, NodeId>) -> NodeId,
) {
    if let Some(d) = flop.d {
        flop.d = Some(remap(d, map));
    }
    flop.q = remap(flop.q, map);
    match &mut flop.mux {
        FlopMux::None => {}
        FlopMux::OneHot(arms) => {
            for arm in arms {
                arm.data = remap(arm.data, map);
                arm.sel = remap(arm.sel, map);
            }
        }
        FlopMux::Encoded { sel, data } => {
            *sel = remap(*sel, map);
            for d in data.iter_mut() {
                *d = remap(*d, map);
            }
        }
    }
}

fn rewrite_flop_from_partial_map(flop: &mut Flop, map: &HashMap<NodeId, NodeId>) {
    if let Some(d) = flop.d.as_mut() {
        rewrite_node_id_if_mapped(d, map);
    }
    rewrite_node_id_if_mapped(&mut flop.q, map);
    match &mut flop.mux {
        FlopMux::None => {}
        FlopMux::OneHot(arms) => {
            for arm in arms {
                rewrite_node_id_if_mapped(&mut arm.data, map);
                rewrite_node_id_if_mapped(&mut arm.sel, map);
            }
        }
        FlopMux::Encoded { sel, data } => {
            rewrite_node_id_if_mapped(sel, map);
            for d in data.iter_mut() {
                rewrite_node_id_if_mapped(d, map);
            }
        }
    }
}

fn rewrite_node_id_if_mapped(id: &mut NodeId, map: &HashMap<NodeId, NodeId>) {
    if let Some(&new_id) = map.get(id) {
        *id = new_id;
    }
}

fn rewrite_instance_inputs_from_partial_map(m: &mut Module, map: &HashMap<NodeId, NodeId>) {
    for instance in &mut m.instances {
        for (_, node_id) in &mut instance.inputs {
            rewrite_node_id_if_mapped(node_id, map);
        }
    }
}

fn prune_duplicate_introducing_add_mul_remaps(
    m: &Module,
    node_remap: &mut HashMap<NodeId, NodeId>,
) -> u32 {
    if node_remap.is_empty() || m.operand_duplication_rate >= 1.0 {
        return 0;
    }

    let mut pruned = 0u32;
    loop {
        let mut offenders = BTreeSet::new();

        for node in &m.nodes {
            let Node::Gate { op, operands, .. } = node else {
                continue;
            };
            if !matches!(op, GateOp::Add | GateOp::Mul) {
                continue;
            }

            let mut positions_by_target: HashMap<NodeId, Vec<usize>> = HashMap::new();
            for (idx, operand) in operands.iter().copied().enumerate() {
                let target = node_remap.get(&operand).copied().unwrap_or(operand);
                positions_by_target.entry(target).or_default().push(idx);
            }

            for (target, positions) in positions_by_target {
                if positions.len() < 2 {
                    continue;
                }

                let mut kept_target = false;
                for pos in positions {
                    let original = operands[pos];
                    if original == target {
                        kept_target = true;
                        continue;
                    }
                    if kept_target {
                        offenders.insert(original);
                    } else {
                        kept_target = true;
                    }
                }
            }
        }

        if offenders.is_empty() {
            break;
        }

        for offender in offenders {
            if node_remap.remove(&offender).is_some() {
                pruned += 1;
            }
        }
    }

    pruned
}

fn rebuild_instance_tables(m: &mut Module) {
    m.gate_instances.clear();
    m.const_instances.clear();

    let nodes = &m.nodes;
    let gate_instances = &mut m.gate_instances;
    let const_instances = &mut m.const_instances;

    for (id, node) in nodes.iter().enumerate() {
        let node_id = id as NodeId;
        match node {
            Node::PrimaryInput { .. }
            | Node::FlopQ { .. }
            | Node::MemRead { .. }
            | Node::FsmOut { .. }
            | Node::InstanceOutput { .. } => {}
            Node::Constant { width, value } => {
                const_instances
                    .entry((*width, *value))
                    .or_default()
                    .push(node_id);
            }
            Node::Gate {
                op,
                operands,
                width,
                ..
            } => {
                gate_instances
                    .entry((*op, operands.clone(), *width))
                    .or_default()
                    .push(node_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FactorizationLevel, IdentityMode};
    use crate::ir::validate::validate;
    use crate::ir::{
        ClockDomain, DepSet, Direction, Flop, FlopKind, FlopMux, Instance, Port, ResetKind,
    };

    /// No-op on a clean IR: all nodes reachable, nothing compacted.
    /// Built at `FactorizationLevel::Cse` so fold rules don't
    /// collapse the constant operand out of the Add gate.
    #[test]
    fn compact_is_noop_when_all_reachable() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::Cse,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let (c7, _) = m.intern_constant(8, 7);
        let (add, _) = m.intern_gate(GateOp::Add, vec![x, c7], 8, DepSet::from_port(0));
        m.drives.push((1, add));

        let before = m.nodes.len();
        let removed = compact_node_ids(&mut m);
        assert_eq!(removed, 0, "clean IR should not compact anything");
        assert_eq!(m.nodes.len(), before);
    }

    /// Injecting an orphan gate (no drive, no consumer) and
    /// compacting should remove it and leave the reachable set
    /// intact.
    #[test]
    fn compact_removes_injected_orphan() {
        // Build at FactorizationLevel::None so intern creates real
        // Add nodes without dedup collapsing them.
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::None,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let (c7, _) = m.intern_constant(8, 7);
        // Reachable gate: driven to output.
        let (live_add, _) = m.intern_gate(GateOp::Add, vec![x, c7], 8, DepSet::from_port(0));
        m.drives.push((1, live_add));

        // Orphan gate: built, never referenced.
        let (c3, _) = m.intern_constant(8, 3);
        let (_orphan, _) = m.intern_gate(GateOp::Sub, vec![x, c3], 8, DepSet::from_port(0));

        let orphan_count_before = count_orphan_gates(&m);
        assert!(orphan_count_before > 0, "test should inject an orphan");

        let n_before = m.nodes.len();
        let removed = compact_node_ids(&mut m);
        assert!(
            removed >= 1,
            "expected at least the Sub orphan to be removed"
        );
        assert!(m.nodes.len() < n_before);

        // Drive root still valid and pointing at a live Add.
        let (_, drive_root) = m.drives[0];
        match &m.nodes[drive_root as usize] {
            Node::Gate {
                op: GateOp::Add, ..
            } => {}
            other => panic!("drive root should still be the Add gate, got {other:?}"),
        }

        // Post-compaction: zero orphan gates.
        assert_eq!(count_orphan_gates(&m), 0);
    }

    /// Compaction preserves topological order: every gate's
    /// operands have smaller NodeId than itself.
    #[test]
    fn compact_preserves_topological_order() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::None,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 });
        let x: NodeId = 0;
        let (c1, _) = m.intern_constant(8, 1);
        let (c2, _) = m.intern_constant(8, 2);
        // Live chain: x + 1, then that + 2.
        let (a1, _) = m.intern_gate(GateOp::Add, vec![x, c1], 8, DepSet::from_port(0));
        let (a2, _) = m.intern_gate(GateOp::Add, vec![a1, c2], 8, DepSet::from_port(0));
        m.drives.push((1, a2));
        // Orphan between them.
        let (c99, _) = m.intern_constant(8, 99);
        let (_orphan, _) = m.intern_gate(GateOp::Sub, vec![a1, c99], 8, DepSet::from_port(0));

        compact_node_ids(&mut m);

        for (idx, node) in m.nodes.iter().enumerate() {
            if let Node::Gate { operands, .. } = node {
                for &op in operands {
                    assert!(
                        (op as usize) < idx,
                        "topological order broken: gate at {idx} references operand {op}"
                    );
                }
            }
        }
    }

    #[test]
    fn compact_drops_flops_whose_q_is_never_observed() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::None,
            ..Module::default()
        };
        m.outputs.push(Port {
            id: 0,
            name: "y".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 });
        m.nodes.push(Node::FlopQ { flop: 1, width: 1 });
        let zero = push_constant(&mut m, 1, 0);
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(zero),
            q: 0,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 1,
            d: Some(zero),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((0, 0));

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "only the dead flop Q node should be removed");
        assert_eq!(m.flops.len(), 1, "unused state element should be pruned");
        assert_eq!(m.flops[0].id, 0);
        assert_eq!(m.flops[0].q, 0);
        assert!(matches!(&m.nodes[0], Node::FlopQ { flop: 0, width: 1 }));
        validate(&m).expect("compacting away dead flops must preserve IR validity");
    }

    #[test]
    fn compact_remaps_explicit_flop_domains() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: crate::config::FactorizationLevel::None,
            ..Module::default()
        };
        m.outputs.push(Port {
            id: 0,
            name: "y".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 });
        m.nodes.push(Node::FlopQ { flop: 1, width: 1 });
        let zero = push_constant(&mut m, 1, 0);
        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(zero),
            q: 0,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 1,
            d: Some(zero),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::None,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        attach_two_clock_domains(&mut m);
        m.flop_domains.insert(0, 0);
        m.flop_domains.insert(1, 1);
        m.drives.push((0, 1));

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "only the dead flop Q node should be removed");
        assert_eq!(m.flops.len(), 1);
        assert_eq!(m.flops[0].id, 0);
        assert_eq!(m.flops[0].q, 0);
        assert_eq!(
            m.flop_domain(0),
            1,
            "the surviving old flop-1 domain must follow the new FlopId"
        );
        validate(&m).expect("compacting explicit-domain flops must preserve IR validity");
    }

    #[test]
    fn merge_equivalent_flops_rewrites_consumers_and_deps() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, 0, 0);
        m.instances.push(Instance {
            id: 0,
            name: "u_0".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(0, 2)],
            param_bindings: Vec::new(),
        });

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);
        assert_eq!(m.flops_merged, 0, "pass returns count; caller records it");
        assert_eq!(
            m.instances[0].inputs,
            vec![(0, 1)],
            "instance inputs must be remapped away from stale duplicate Q nodes"
        );

        let Node::Gate { operands, deps, .. } = &m.nodes[3] else {
            panic!("drive root should still be the add gate");
        };
        assert_eq!(operands, &vec![1, 1]);
        assert_eq!(deps.len(), 1, "virtual flop deps should coalesce");
        assert!(deps.contains_flop_virtual(0));

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "duplicate FlopQ should become unreachable");
        validate(&m).expect("merged module should still validate");
        assert_eq!(m.flops[0].id, 0);
        assert_eq!(m.flops[0].q, 1);
        match &m.nodes[1] {
            Node::FlopQ { flop, width } => {
                assert_eq!((*flop, *width), (0, 8));
            }
            other => panic!("expected surviving canonical FlopQ, got {other:?}"),
        }
    }

    #[test]
    fn compact_remaps_instance_input_bindings() {
        let mut m = Module {
            name: "parent".into(),
            ..Module::default()
        };
        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 });
        m.nodes.push(Node::PrimaryInput { port: 1, width: 1 });
        m.instances.push(Instance {
            id: 0,
            name: "u_0".into(),
            module: "child".into(),
            role: crate::ir::InstanceRole::PlannedChild,
            inputs: vec![(0, 1)],
            param_bindings: Vec::new(),
        });

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "dead primary input should be removed");
        assert_eq!(m.nodes.len(), 1, "only the bound input should remain");
        assert_eq!(m.instances[0].inputs, vec![(0, 0)]);
    }

    #[test]
    fn merge_equivalent_flops_respects_relaxed_identity() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::Relaxed, FactorizationLevel::EGraph, 0, 0);
        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.flops.len(), 2);
        let Node::Gate { operands, deps, .. } = &m.nodes[3] else {
            panic!("fixture root should be an add gate");
        };
        assert_eq!(operands, &vec![1, 2]);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn merge_equivalent_flops_keeps_distinct_reset_signatures() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, 0, 1);
        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_equivalent_flops_keeps_distinct_clock_domains() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, 0, 0);
        attach_two_clock_domains(&mut m);
        m.flop_domains.insert(0, 0);
        m.flop_domains.insert(1, 1);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0, "cross-domain state must not merge");
        assert_eq!(m.flops.len(), 2);
        assert_eq!(m.flop_domain(0), 0);
        assert_eq!(m.flop_domain(1), 1);
        validate(&m).expect("cross-domain no-merge fixture should remain valid");
    }

    #[test]
    fn merge_equivalent_flops_merges_same_explicit_clock_domain() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, 0, 0);
        attach_two_clock_domains(&mut m);
        m.flop_domains.insert(0, 1);
        m.flop_domains.insert(1, 1);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);
        assert_eq!(
            m.flop_domain(0),
            1,
            "explicit surviving domain must be remapped to the dense FlopId"
        );

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "duplicate Q should become unreachable");
        assert_eq!(m.flop_domain(0), 1);
        validate(&m).expect("same-domain merged fixture should validate after compaction");
    }

    #[test]
    fn merge_equivalent_flops_keeps_self_feedback_cones_distinct_when_q_endpoints_differ() {
        let mut m = self_feedback_flop_fixture();

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0, "different q endpoints must stay distinct");
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_equivalent_flops_merges_reset_defined_self_hold_flops() {
        let mut m = self_hold_flop_fixture(8, 8, 0, 0, ResetKind::Async);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);
        assert_eq!(
            m.drives,
            vec![(0, 0), (1, 0)],
            "duplicate self-hold Q consumers should use the canonical Q"
        );

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "duplicate self-hold Q should compact");
        validate(&m).expect("merged self-hold flops should still validate");
    }

    #[test]
    fn merge_equivalent_flops_keeps_resetless_self_hold_flops_distinct() {
        let mut m = self_hold_flop_fixture(8, 8, 0, 0, ResetKind::None);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(
            removed, 0,
            "without reset there is no reset-defined equality point"
        );
        assert_eq!(m.flops.len(), 2);
    }

    // ----- IDENTITY-DEEPENING.2b: bounded bisimulation flop merge -----

    #[test]
    fn merge_bisimilar_flops_merges_mutual_swap_registers() {
        // The mutual swap (D_f = Q_g, D_g = Q_f, equal reset) is exactly
        // the recorded mutually-recursive-register no-merge boundary that
        // the exact pass provably cannot prove: each flop's D-cone keys a
        // *different* concrete FlopQ endpoint.
        let mut exact = mutual_swap_flop_fixture(ResetKind::Async, 0);
        assert_eq!(
            merge_equivalent_flops(&mut exact),
            0,
            "exact pass cannot prove mutually-recursive registers equivalent"
        );
        assert_eq!(exact.flops.len(), 2);

        // Bisimulation (knob on) identifies them: reset gives the t=0 base
        // case, and the quotient transition (FlopQ -> class rep) is stable.
        let mut m = mutual_swap_flop_fixture(ResetKind::Async, 0);
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            1,
            "bisimulation should merge the mutually-recursive register pair"
        );
        assert_eq!(m.flops.len(), 1);

        compact_node_ids(&mut m);
        validate(&m).expect("merged mutual-swap registers should validate");

        // Downstream-bank reproduction hook (IDENTITY-DEEPENING.2b): the
        // mutual swap of two equal-reset registers collapses to one
        // self-holding register, which is downstream-clean across Verilator,
        // both Yosys modes, and Icarus. Default no-op; re-bank with
        //   ANVIL_DUMP_BISIM_SV=1 cargo test --lib \
        //     merge_bisimilar_flops_merges_mutual_swap_registers
        // then lint /tmp/anvil-bisim-merged.sv with the three tools.
        if std::env::var("ANVIL_DUMP_BISIM_SV").is_ok() {
            std::fs::write("/tmp/anvil-bisim-merged.sv", crate::emit::to_sv(&m)).unwrap();
        }
    }

    #[test]
    fn merge_bisimilar_flops_is_default_off() {
        let mut m = mutual_swap_flop_fixture(ResetKind::Async, 0);
        m.bisimulation_flop_merge = false; // the default
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            0,
            "default-off knob must not merge (byte-identical contract)"
        );
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_bisimilar_flops_keeps_resetless_mutual_swap_distinct() {
        // No reset => no provable equal initial state => the bisimulation
        // base case fails, so the mutual swap must NOT merge. This
        // preserves the reset-defined-self-hold-flop-identity boundary.
        let mut m = mutual_swap_flop_fixture(ResetKind::None, 0);
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            0,
            "resetless mutually-recursive state has no base case"
        );
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_bisimilar_flops_respects_relaxed_identity() {
        let mut m = mutual_swap_flop_fixture(ResetKind::Async, 0);
        m.identity_mode = IdentityMode::Relaxed;
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            0,
            "relaxed identity is the real off-switch"
        );
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_bisimilar_flops_requires_egraph_level() {
        let mut m = mutual_swap_flop_fixture(ResetKind::Async, 0);
        m.factorization_level = FactorizationLevel::Cse;
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            0,
            "bisimulation requires effective e-graph (parity with semantic gate merge)"
        );
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_bisimilar_flops_keeps_non_bisimilar_flops_distinct() {
        // f: D = Q_g ; g: D = a (primary input). Same width/reset/domain,
        // but their transitions differ under every state correspondence, so
        // refinement must split them and merge nothing.
        let mut m = non_bisimilar_flop_fixture();
        assert_eq!(
            merge_bisimilar_flops(&mut m),
            0,
            "flops with genuinely different transitions must not merge"
        );
        assert_eq!(m.flops.len(), 2);
    }

    /// A two-deep delay-line stateful leaf module (`out` = `in` delayed two
    /// cycles). When `double_not` is set, the first flop's D-cone is the
    /// semantically-equal `~~in` instead of a bare `in`, so the module is
    /// *structurally distinct* (an extra two `Not` gates) while staying
    /// *sequentially equivalent* — exactly the cross-module merge target.
    fn delay2_leaf(name: &str, double_not: bool, reset_kind: ResetKind) -> Module {
        let mut m = Module {
            name: name.into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        // Shared interface: clk(0), rst_n(1), in(2) -> out(3).
        m.inputs.push(Port {
            id: 0,
            name: "clk".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "rst_n".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 2,
            name: "in".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 3,
            name: "out".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.clock = Some(0);
        m.reset = Some(1);

        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 0 = in
        let d0: NodeId = if double_not {
            m.nodes.push(Node::Gate {
                op: GateOp::Not,
                operands: vec![0],
                width: 1,
                deps: DepSet::from_port(2),
            }); // 1 = ~in
            m.nodes.push(Node::Gate {
                op: GateOp::Not,
                operands: vec![1],
                width: 1,
                deps: DepSet::from_port(2),
            }); // 2 = ~~in
            2
        } else {
            0
        };
        let q0_node = m.nodes.len() as NodeId;
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 }); // Q_0
        let q1_node = m.nodes.len() as NodeId;
        m.nodes.push(Node::FlopQ { flop: 1, width: 1 }); // Q_1

        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(d0), // stage 0 captures `in` (possibly via ~~in)
            q: q0_node,
            reset_val: 0,
            reset_kind,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 1,
            d: Some(q0_node), // stage 1 captures stage 0's Q
            q: q1_node,
            reset_val: 0,
            reset_kind,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.drives.push((3, q1_node)); // out = Q_1 (two-cycle delay)

        rebuild_instance_tables(&mut m);
        m
    }

    /// A one-deep delay module sharing the delay-line interface and flop shape
    /// but observing stage 0 (`out` = `in` delayed *one* cycle): genuinely
    /// NOT sequentially equivalent to [`delay2_leaf`].
    fn delay1_leaf(name: &str) -> Module {
        let mut m = delay2_leaf(name, false, ResetKind::Async);
        // Rewire the output drive from Q_1 (node 2) to Q_0 (node 1).
        let q0_node = 1 as NodeId;
        m.drives.clear();
        m.drives.push((3, q0_node));
        rebuild_instance_tables(&mut m);
        m
    }

    #[test]
    fn modules_sequentially_equivalent_merges_structurally_distinct_delay_lines() {
        // Two two-cycle delay lines, one built with a redundant `~~in` D-cone:
        // structurally distinct (so `dedup_modules` keeps them apart) but
        // sequentially equivalent up to the identity state correspondence the
        // cross-module bisimulation discovers.
        let a = delay2_leaf("delay_a", false, ResetKind::Async);
        let b = delay2_leaf("delay_b", true, ResetKind::Async);
        assert!(
            modules_sequentially_equivalent(&a, &b),
            "two-cycle delay lines must be proven sequentially equivalent"
        );
        // Symmetric.
        assert!(modules_sequentially_equivalent(&b, &a));
    }

    #[test]
    fn modules_sequentially_equivalent_rejects_non_equivalent_delays() {
        let a = delay2_leaf("delay2", false, ResetKind::Async);
        let c = delay1_leaf("delay1");
        assert!(
            !modules_sequentially_equivalent(&a, &c),
            "one-cycle and two-cycle delays differ for some input sequence"
        );
    }

    #[test]
    fn modules_sequentially_equivalent_rejects_resetless_modules() {
        // No reset => no t=0 base case => the cross-module correspondence is
        // unprovable. Carries the resetless boundary forward.
        let a = delay2_leaf("delay_a", false, ResetKind::None);
        let b = delay2_leaf("delay_b", true, ResetKind::None);
        assert!(
            !modules_sequentially_equivalent(&a, &b),
            "resetless stateful modules have no bisimulation base case"
        );
    }

    #[test]
    fn modules_sequentially_equivalent_rejects_interface_mismatch() {
        let a = delay2_leaf("delay_a", false, ResetKind::Async);
        let mut b = delay2_leaf("delay_b", true, ResetKind::Async);
        // Widen the output port: same behaviour class, different interface, so
        // an instance rewrite would break parent-side bindings => reject.
        b.outputs[0].width = 2;
        assert!(
            !modules_sequentially_equivalent(&a, &b),
            "mismatched output interface must not merge"
        );
    }

    #[test]
    fn merge_equivalent_flops_keeps_self_hold_reset_mismatches_distinct() {
        let mut m = self_hold_flop_fixture(8, 8, 0, 1, ResetKind::Async);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.flops.len(), 2);
    }

    #[test]
    fn merge_equivalent_flops_keeps_self_hold_domain_mismatches_distinct() {
        let mut m = self_hold_flop_fixture(8, 8, 0, 0, ResetKind::Async);
        attach_two_clock_domains(&mut m);
        m.flop_domains.insert(0, 0);
        m.flop_domains.insert(1, 1);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.flops.len(), 2);
        validate(&m).expect("cross-domain self-hold fixture should remain valid");
    }

    #[test]
    fn merge_equivalent_flops_keeps_self_hold_width_mismatches_distinct() {
        let mut m = self_hold_flop_fixture(8, 4, 0, 0, ResetKind::Async);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.flops.len(), 2);
        validate(&m).expect("width-mismatch self-hold fixture should remain valid");
    }

    #[test]
    fn merge_equivalent_flops_merges_same_endpoint_duplicate_d_cones() {
        let mut m = non_self_duplicate_d_fixture();

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);

        let compacted = compact_node_ids(&mut m);
        assert_eq!(
            compacted, 2,
            "duplicate D-cone and Q should become unreachable"
        );
        validate(&m).expect("merged duplicate D-cones should still validate");
    }

    #[test]
    fn merge_equivalent_flops_merges_small_semantic_equivalents_with_same_endpoints() {
        let mut m = semantic_equivalent_d_fixture();

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);

        let compacted = compact_node_ids(&mut m);
        assert_eq!(
            compacted, 3,
            "duplicate semantic cone subtree and Q should become unreachable"
        );
        validate(&m).expect("merged semantic-equivalent D-cones should still validate");
    }

    #[test]
    fn merge_equivalent_gates_merges_small_semantic_equivalents() {
        let mut m = semantic_equivalent_gate_fixture();

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(removed, 1);

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "duplicate semantic gate should compact");
        validate(&m).expect("merged semantic-equivalent gates should still validate");
    }

    #[test]
    fn merge_equivalent_gates_folds_small_semantic_gate_to_existing_endpoint() {
        let mut m = semantic_endpoint_fold_gate_fixture();

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(
            m.drives[0].1, 0,
            "gate proven equal to the canonical endpoint should be rewired to that endpoint"
        );

        let compacted = compact_node_ids(&mut m);
        assert_eq!(
            compacted, 4,
            "folded endpoint-equivalent cone should become unreachable"
        );
        validate(&m).expect("endpoint-folded semantic gate should still validate");
    }

    #[test]
    fn merge_equivalent_gates_folds_small_semantic_gate_to_existing_constant() {
        let mut m = semantic_constant_fold_gate_fixture();

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(
            m.drives[0].1, 1,
            "gate proven equal to an existing constant should be rewired to that constant"
        );

        let compacted = compact_node_ids(&mut m);
        assert_eq!(
            compacted, 3,
            "constant-equivalent helper cone should become unreachable"
        );
        validate(&m).expect("constant-folded semantic gate should still validate");
    }

    #[test]
    fn merge_equivalent_gates_keeps_same_shape_different_endpoints_distinct() {
        let mut m = same_shape_different_endpoints_gate_fixture();

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(
            removed, 3,
            "tautology sub-cones may merge and each root may fold to its own endpoint"
        );
        assert_eq!(
            m.drives,
            vec![(4, 0), (5, 2)],
            "same-shape cones over different endpoints must not collapse to the same canonical node"
        );

        validate(&m).expect("same-shape different-endpoint gates should still validate");
    }

    #[test]
    fn merge_equivalent_gates_respects_requested_level() {
        let mut m = semantic_equivalent_gate_fixture();
        m.factorization_level = FactorizationLevel::Peephole;

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(removed, 0);
    }

    #[test]
    fn merge_equivalent_gates_respects_relaxed_identity() {
        use crate::config::IdentityMode;

        let mut m = semantic_equivalent_gate_fixture();
        m.identity_mode = IdentityMode::Relaxed;

        let removed = merge_equivalent_gates(&mut m);
        assert_eq!(removed, 0);
    }

    #[test]
    fn merge_equivalent_fsms_merges_duplicate_blocks_and_remaps_consumers() {
        let mut m = duplicate_fsm_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, false);

        let removed = merge_equivalent_fsms(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.fsms.len(), 1);

        let Node::Gate { operands, deps, .. } = &m.nodes[3] else {
            panic!("drive root should still be the fsm-output combiner");
        };
        assert_eq!(
            operands,
            &vec![1, 1],
            "duplicate FsmOut consumers must be rewired to the canonical output"
        );
        assert_eq!(deps.len(), 1, "virtual FSM deps should coalesce");
        assert!(deps.contains_fsm_virtual(0));

        let compacted = compact_node_ids(&mut m);
        assert_eq!(compacted, 1, "duplicate FsmOut should become unreachable");
        validate(&m).expect("merged duplicate FSMs should still validate");
        assert_eq!(m.fsms[0].id, 0);
        match &m.nodes[1] {
            Node::FsmOut { fsm, width } => {
                assert_eq!((*fsm, *width), (0, 8));
            }
            other => panic!("expected surviving canonical FsmOut, got {other:?}"),
        }
    }

    #[test]
    fn merge_equivalent_fsms_respects_relaxed_identity() {
        let mut m = duplicate_fsm_fixture(IdentityMode::Relaxed, FactorizationLevel::EGraph, false);

        let removed = merge_equivalent_fsms(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.fsms.len(), 2);
    }

    #[test]
    fn merge_equivalent_fsms_respects_factorization_off() {
        let mut m = duplicate_fsm_fixture(IdentityMode::NodeId, FactorizationLevel::None, false);

        let removed = merge_equivalent_fsms(&mut m);
        assert_eq!(removed, 0);
        assert_eq!(m.fsms.len(), 2);
    }

    #[test]
    fn merge_equivalent_fsms_keeps_distinct_selector_proofs() {
        let mut m = duplicate_fsm_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, true);

        let removed = merge_equivalent_fsms(&mut m);
        assert_eq!(removed, 0, "different selector endpoints are distinct FSMs");
        assert_eq!(m.fsms.len(), 2);
    }

    #[test]
    fn semantic_merge_proof_skips_large_low_support_cones() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        let x = push_primary(&mut m, 0, 1);
        let y = push_primary(&mut m, 1, 1);
        let mut root = x;
        for _ in 0..(MAX_MERGE_SEMANTIC_CONE_NODES + 8) {
            root = push_raw_gate(&mut m, GateOp::Xor, vec![root, y], 1);
        }

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            semantic_cone_proof(&m, root, &mut endpoint_memo).is_none(),
            "semantic merge proof must skip large cones even when endpoint support stays tiny"
        );
    }

    #[test]
    fn semantic_merge_proof_accepts_tiny_twelve_bit_cones() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        let operands: Vec<NodeId> = (0..MAX_SEMANTIC_SUPPORT_BITS)
            .map(|port| push_primary(&mut m, port, 1))
            .collect();
        let root = push_raw_gate(&mut m, GateOp::Concat, operands, MAX_SEMANTIC_SUPPORT_BITS);

        let mut endpoint_memo = std::collections::HashMap::new();
        let proof =
            semantic_cone_proof(&m, root, &mut endpoint_memo).expect("tiny 12-bit proof fits");
        assert_eq!(proof.endpoints.len(), MAX_SEMANTIC_SUPPORT_BITS as usize);
        assert_eq!(proof.outputs.len(), 1usize << MAX_SEMANTIC_SUPPORT_BITS);
    }

    #[test]
    fn semantic_merge_proof_skips_twelve_bit_cones_over_work_budget() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        let inputs: Vec<NodeId> = (0..MAX_SEMANTIC_SUPPORT_BITS)
            .map(|port| push_primary(&mut m, port, 1))
            .collect();
        let mut root = inputs[0];
        for &input in &inputs[1..] {
            root = push_raw_gate(&mut m, GateOp::Xor, vec![root, input], 1);
        }
        let max_twelve_bit_nodes =
            MAX_MERGE_SEMANTIC_WORK_UNITS / (1usize << MAX_SEMANTIC_SUPPORT_BITS);
        while m.nodes.len() <= max_twelve_bit_nodes {
            root = push_raw_gate(&mut m, GateOp::Xor, vec![root, inputs[0]], 1);
        }

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            semantic_cone_proof(&m, root, &mut endpoint_memo).is_none(),
            "12-bit proofs must stay within the old 10-bit work envelope"
        );
    }

    fn push_primary(m: &mut Module, port: PortId, width: u32) -> NodeId {
        if !m.inputs.iter().any(|existing| existing.id == port) {
            m.inputs.push(Port {
                id: port,
                name: format!("i_{port}"),
                width,
                dir: Direction::In,
            });
        }
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput { port, width });
        node_id
    }

    fn push_constant(m: &mut Module, width: u32, value: u128) -> NodeId {
        let (node_id, _) = m.intern_constant(width, value);
        node_id
    }

    fn deps_of(m: &Module, id: NodeId) -> DepSet {
        match &m.nodes[id as usize] {
            Node::PrimaryInput { port, .. } => DepSet::from_port(*port),
            Node::Constant { .. } => DepSet::new(),
            Node::InstanceOutput { instance, port, .. } => {
                DepSet::from_instance_output_virtual(*instance, *port)
            }
            Node::FlopQ { flop, .. } => DepSet::from_flop_virtual(*flop),
            Node::MemRead { mem, .. } => DepSet::from_mem_virtual(*mem),
            Node::FsmOut { fsm, .. } => DepSet::from_fsm_virtual(*fsm),
            Node::Gate { deps, .. } => deps.clone(),
        }
    }

    fn push_gate(m: &mut Module, op: GateOp, operands: Vec<NodeId>, width: u32) -> NodeId {
        let dep_sets: Vec<DepSet> = operands
            .iter()
            .map(|operand| deps_of(m, *operand))
            .collect();
        let dep_refs: Vec<&DepSet> = dep_sets.iter().collect();
        let deps = DepSet::union(&dep_refs);
        let (node_id, _) = m.intern_gate(op, operands, width, deps);
        node_id
    }

    fn push_raw_gate(m: &mut Module, op: GateOp, operands: Vec<NodeId>, width: u32) -> NodeId {
        let dep_sets: Vec<DepSet> = operands
            .iter()
            .map(|operand| deps_of(m, *operand))
            .collect();
        let dep_refs: Vec<&DepSet> = dep_sets.iter().collect();
        let deps = DepSet::union(&dep_refs);
        let node_id = m.nodes.len() as NodeId;
        m.nodes.push(Node::Gate {
            op,
            operands,
            width,
            deps,
        });
        node_id
    }

    #[test]
    fn flatten_posthoc_associative_gates_flattens_legal_nested_add() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::Associative,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 1);
        let b = push_primary(&mut m, 1, 1);
        let c = push_primary(&mut m, 2, 1);
        let inner = push_raw_gate(&mut m, GateOp::Add, vec![b, c], 1);
        let outer = push_raw_gate(&mut m, GateOp::Add, vec![a, inner], 1);

        let flattened = flatten_posthoc_associative_gates(&mut m);
        assert_eq!(flattened, 1);
        assert!(
            matches!(
                &m.nodes[outer as usize],
                Node::Gate {
                    op: GateOp::Add,
                    operands,
                    width: 1,
                    ..
                } if operands == &vec![a, b, c]
            ),
            "post-remap associative pass should splice the inner Add operands into the outer Add"
        );
    }

    #[test]
    fn flatten_posthoc_associative_gates_keeps_duplicate_bearing_add_nested() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::Associative,
            operand_duplication_rate: 0.0,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 1);
        let b = push_primary(&mut m, 1, 1);
        let inner = push_raw_gate(&mut m, GateOp::Add, vec![a, b], 1);
        let outer = push_raw_gate(&mut m, GateOp::Add, vec![a, inner], 1);

        let flattened = flatten_posthoc_associative_gates(&mut m);
        assert_eq!(flattened, 0);
        assert!(
            matches!(
                &m.nodes[outer as usize],
                Node::Gate {
                    op: GateOp::Add,
                    operands,
                    width: 1,
                    ..
                } if operands == &vec![a, inner]
            ),
            "duplicate-bearing Add nesting must remain intact under the strict duplicate policy"
        );
    }

    #[test]
    fn flatten_posthoc_associative_gates_dedups_idempotent_duplicates() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::Associative,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 1);
        let b = push_primary(&mut m, 1, 1);
        let and = push_raw_gate(&mut m, GateOp::And, vec![a, b, a], 1);

        let flattened = flatten_posthoc_associative_gates(&mut m);
        assert_eq!(flattened, 1);
        assert!(
            matches!(
                &m.nodes[and as usize],
                Node::Gate {
                    op: GateOp::And,
                    operands,
                    width: 1,
                    ..
                } if operands == &vec![a, b]
            ),
            "post-remap associative pass should still dedup idempotent duplicates even without a same-op child splice"
        );
    }

    #[test]
    fn fold_mixed_associative_constants_cancels_duplicate_ones_in_width1_add() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::ConstantFold,
            ..Module::default()
        };
        let x = push_primary(&mut m, 0, 1);
        let y = push_primary(&mut m, 1, 1);
        let one = push_constant(&mut m, 1, 1);
        let add = push_raw_gate(&mut m, GateOp::Add, vec![one, x, one, y], 1);

        let simplified = fold_mixed_associative_constants(&mut m);
        assert_eq!(simplified, 1);
        assert!(
            matches!(
                &m.nodes[add as usize],
                Node::Gate {
                    op: GateOp::Add,
                    operands,
                    width: 1,
                    ..
                } if operands == &vec![x, y]
            ),
            "1-bit Add with two literal ones should cancel them modulo 2"
        );
    }

    #[test]
    fn fold_proven_gates_revisits_constant_selector_mux_chains() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let bit = push_primary(&mut m, 0, 1);
        let zero1 = push_constant(&mut m, 1, 0);
        let zero5 = push_constant(&mut m, 5, 0);
        let hi = push_constant(&mut m, 5, 0x12);
        let c0d = push_constant(&mut m, 5, 0x0d);
        let c1a = push_constant(&mut m, 5, 0x1a);
        let c5 = push_constant(&mut m, 5, 0x05);
        let c8 = push_constant(&mut m, 5, 0x08);
        let c10 = push_constant(&mut m, 5, 0x0a);
        let c31 = push_constant(&mut m, 5, 0x1f);
        let concat = push_gate(&mut m, GateOp::Concat, vec![bit, bit, bit, bit, bit], 5);
        let dead_sel = push_gate(&mut m, GateOp::Mux, vec![zero1, hi, zero5], 5);
        let masked = push_gate(&mut m, GateOp::And, vec![c0d, concat, dead_sel, c1a], 5);
        let not_bit = push_gate(&mut m, GateOp::Sub, vec![zero1, bit], 1);
        let rhs_const = push_gate(&mut m, GateOp::And, vec![c31, c8, c5], 5);
        let rhs = push_gate(&mut m, GateOp::Mux, vec![not_bit, c10, rhs_const], 5);
        let cmp = push_gate(&mut m, GateOp::Le, vec![masked, rhs], 1);
        m.outputs.push(Port {
            id: 1,
            name: "o_0".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.drives.push((1, cmp));

        assert!(fold_proven_gates(&mut m) > 0);
        assert!(matches!(
            &m.nodes[dead_sel as usize],
            Node::Constant { width: 5, value: 0 }
        ));
        assert!(matches!(
            &m.nodes[masked as usize],
            Node::Constant { width: 5, value: 0 }
        ));
        assert!(matches!(
            &m.nodes[rhs_const as usize],
            Node::Constant { width: 5, value: 0 }
        ));
        assert!(matches!(
            &m.nodes[cmp as usize],
            Node::Constant { width: 1, value: 1 }
        ));
    }

    #[test]
    fn fold_proven_gates_revisits_overshift_compare_chain() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let wide = push_primary(&mut m, 0, 9);
        let slice = push_gate(&mut m, GateOp::Slice { hi: 3, lo: 0 }, vec![wide], 4);
        let variable = push_primary(&mut m, 1, 8);
        let sel = push_primary(&mut m, 2, 1);
        let c7b = push_constant(&mut m, 8, 0x7b);
        let cde = push_constant(&mut m, 8, 0xde);
        let c80 = push_constant(&mut m, 8, 0x80);
        let shift_amt = push_gate(&mut m, GateOp::Or, vec![variable, c7b, cde, c80], 8);
        let shifted = push_gate(&mut m, GateOp::Shr, vec![slice, shift_amt], 4);
        let maybe_slice = push_gate(&mut m, GateOp::Mux, vec![sel, slice, shifted], 4);
        let masked = push_gate(&mut m, GateOp::And, vec![shifted, maybe_slice], 4);
        let cmp = push_gate(&mut m, GateOp::Lt, vec![slice, masked], 1);
        m.outputs.push(Port {
            id: 3,
            name: "o_0".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.drives.push((3, cmp));

        assert!(fold_proven_gates(&mut m) > 0);
        assert!(matches!(
            &m.nodes[shift_amt as usize],
            Node::Constant {
                width: 8,
                value: 0xff
            }
        ));
        assert!(matches!(
            &m.nodes[cmp as usize],
            Node::Constant { width: 1, value: 0 }
        ));
    }

    #[test]
    fn fold_proven_gates_revisits_dynamic_overshift_through_wide_slice() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let wide = push_primary(&mut m, 0, 9);
        let slice = push_gate(&mut m, GateOp::Slice { hi: 7, lo: 0 }, vec![wide], 8);
        let c26 = push_constant(&mut m, 8, 0x26);
        let ceb = push_constant(&mut m, 8, 0xeb);
        let or = push_gate(&mut m, GateOp::Or, vec![slice, c26, slice, ceb], 8);
        let one = push_constant(&mut m, 1, 1);
        let shl = push_gate(&mut m, GateOp::Shl, vec![or, one], 8);
        let five = push_constant(&mut m, 8, 5);
        let rhs = push_gate(&mut m, GateOp::Sub, vec![shl, five], 8);
        let shr = push_gate(&mut m, GateOp::Shr, vec![shl, rhs], 8);
        let sink = push_gate(&mut m, GateOp::Add, vec![shr, five], 8);
        m.outputs.push(Port {
            id: 1,
            name: "o_0".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.drives.push((1, sink));

        assert!(fold_proven_gates(&mut m) > 0);
        assert!(matches!(
            &m.nodes[shr as usize],
            Node::Constant { width: 8, value: 0 }
        ));
        assert!(matches!(
            &m.nodes[sink as usize],
            Node::Constant { width: 8, value: 5 }
        ));
    }

    #[test]
    fn cleanup_exact_proof_skips_four_endpoint_cones() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 1);
        let b = push_primary(&mut m, 1, 1);
        let c = push_primary(&mut m, 2, 1);
        let d = push_primary(&mut m, 3, 1);
        let concat = push_gate(&mut m, GateOp::Concat, vec![a, b, c, d], 4);

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            !cleanup_exact_proof_eligible(&m, concat, &mut endpoint_memo),
            "post-construction exact cleanup must skip cones with more than three canonical leaf endpoints"
        );
    }

    #[test]
    fn cleanup_exact_proof_accepts_tiny_twelve_bit_three_endpoint_cones() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 4);
        let b = push_primary(&mut m, 1, 4);
        let c = push_primary(&mut m, 2, 4);
        let xor = push_raw_gate(&mut m, GateOp::Xor, vec![a, b, c], 4);
        let root = push_raw_gate(&mut m, GateOp::RedOr, vec![xor], 1);

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            cleanup_exact_proof_eligible(&m, root, &mut endpoint_memo),
            "tiny cleanup proofs may use up to twelve support bits across three endpoints"
        );
    }

    #[test]
    fn cleanup_exact_proof_skips_large_low_support_cones() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let x = push_primary(&mut m, 0, 1);
        let y = push_primary(&mut m, 1, 1);
        let mut root = x;
        for _ in 0..(MAX_CLEANUP_SEMANTIC_CONE_NODES + 8) {
            root = push_raw_gate(&mut m, GateOp::Xor, vec![root, y], 1);
        }

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            !cleanup_exact_proof_eligible(&m, root, &mut endpoint_memo),
            "post-construction exact cleanup must skip large cones even when the endpoint support stays tiny"
        );
    }

    #[test]
    fn cleanup_exact_proof_skips_twelve_bit_cones_over_work_budget() {
        let mut m = Module {
            max_ast_instances: u32::MAX,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 4);
        let b = push_primary(&mut m, 1, 4);
        let c = push_primary(&mut m, 2, 4);
        let mut root = push_raw_gate(&mut m, GateOp::Xor, vec![a, b, c], 4);
        let max_twelve_bit_nodes =
            MAX_CLEANUP_SEMANTIC_WORK_UNITS / (1usize << MAX_SEMANTIC_SUPPORT_BITS);
        while m.nodes.len() <= max_twelve_bit_nodes {
            root = push_raw_gate(&mut m, GateOp::Xor, vec![root, a], 4);
        }
        root = push_raw_gate(&mut m, GateOp::RedOr, vec![root], 1);

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            !cleanup_exact_proof_eligible(&m, root, &mut endpoint_memo),
            "cleanup proofs keep the tighter old 10-bit work envelope"
        );
    }

    #[test]
    fn fold_proven_gates_revisits_large_endpoint_unsigned_compare() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let lhs = push_primary(&mut m, 0, 2);
        let a = push_primary(&mut m, 1, 1);
        let b = push_primary(&mut m, 2, 1);
        let c = push_primary(&mut m, 3, 1);
        let d = push_primary(&mut m, 4, 1);
        let zero1 = push_constant(&mut m, 1, 0);
        let zero2 = push_constant(&mut m, 2, 0);
        let dead_sel = push_gate(&mut m, GateOp::And, vec![zero1, a, b, c, d], 1);
        let true_arm = push_gate(&mut m, GateOp::Concat, vec![a, b], 2);
        let rhs = push_gate(&mut m, GateOp::Mux, vec![dead_sel, true_arm, zero2], 2);
        let ge = push_gate(&mut m, GateOp::Ge, vec![lhs, rhs], 1);

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            !cleanup_exact_proof_eligible(&m, ge, &mut endpoint_memo),
            "the general cleanup exact prover should skip this large-endpoint compare"
        );

        assert!(fold_proven_gates(&mut m) > 0);
        assert!(matches!(
            &m.nodes[ge as usize],
            Node::Constant { width: 1, value: 1 }
        ));
    }

    #[test]
    fn fold_proven_gates_revisits_large_endpoint_overshift_shift() {
        let mut m = Module {
            max_ast_instances: 1,
            factorization_level: FactorizationLevel::None,
            ..Module::default()
        };
        let a = push_primary(&mut m, 0, 1);
        let b = push_primary(&mut m, 1, 1);
        let c = push_primary(&mut m, 2, 1);
        let d = push_primary(&mut m, 3, 1);
        let eight = push_constant(&mut m, 4, 8);
        let one = push_constant(&mut m, 2, 1);
        let rhs = push_gate(&mut m, GateOp::Concat, vec![a, b, c, d], 4);
        let rhs_masked = push_gate(&mut m, GateOp::Or, vec![rhs, eight], 4);
        let shr = push_gate(&mut m, GateOp::Shr, vec![one, rhs_masked], 2);

        let mut endpoint_memo = std::collections::HashMap::new();
        assert!(
            !cleanup_exact_proof_eligible(&m, shr, &mut endpoint_memo),
            "the general cleanup exact prover should skip this large-endpoint shift"
        );

        assert!(fold_proven_gates(&mut m) > 0);
        assert!(matches!(
            &m.nodes[shr as usize],
            Node::Constant { width: 2, value: 0 }
        ));
    }

    fn count_orphan_gates(m: &Module) -> usize {
        let n = m.nodes.len();
        let mut used = vec![false; n];
        for node in &m.nodes {
            if let Node::Gate { operands, .. } = node {
                for &op in operands {
                    used[op as usize] = true;
                }
            }
        }
        for flop in &m.flops {
            if let Some(d) = flop.d {
                used[d as usize] = true;
            }
            used[flop.q as usize] = true;
            match &flop.mux {
                FlopMux::None => {}
                FlopMux::OneHot(arms) => {
                    for arm in arms {
                        used[arm.data as usize] = true;
                        used[arm.sel as usize] = true;
                    }
                }
                FlopMux::Encoded { sel, data } => {
                    used[*sel as usize] = true;
                    for d in data {
                        used[*d as usize] = true;
                    }
                }
            }
        }
        for (_, root) in &m.drives {
            used[*root as usize] = true;
        }
        m.nodes
            .iter()
            .enumerate()
            .filter(|(i, n)| matches!(n, Node::Gate { .. }) && !used[*i])
            .count()
    }

    fn attach_two_clock_domains(m: &mut Module) {
        let next_port_id = m
            .inputs
            .iter()
            .chain(m.outputs.iter())
            .map(|p| p.id)
            .max()
            .map(|id| id + 1)
            .unwrap_or(0);
        let clk_a = next_port_id;
        let rst_n_a = next_port_id + 1;
        let clk_b = next_port_id + 2;
        let rst_n_b = next_port_id + 3;
        for (id, name) in [
            (clk_a, "clk"),
            (rst_n_a, "rst_n"),
            (clk_b, "clk_b"),
            (rst_n_b, "rst_n_b"),
        ] {
            m.inputs.push(Port {
                id,
                name: name.to_string(),
                width: 1,
                dir: Direction::In,
            });
        }
        m.clock = Some(clk_a);
        m.reset = Some(rst_n_a);
        m.clock_domains.push(ClockDomain {
            clk: clk_a,
            rst_n: rst_n_a,
            name: "a".to_string(),
        });
        m.clock_domains.push(ClockDomain {
            clk: clk_b,
            rst_n: rst_n_b,
            name: "b".to_string(),
        });
    }

    fn exact_signature_flop_fixture(
        identity_mode: IdentityMode,
        factorization_level: FactorizationLevel,
        reset0: u128,
        reset1: u128,
    ) -> Module {
        let mut m = Module {
            name: "f".into(),
            identity_mode,
            factorization_level,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 1
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 2
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![1, 2],
            width: 8,
            deps: DepSet::union(&[&DepSet::from_flop_virtual(0), &DepSet::from_flop_virtual(1)]),
        }); // 3
        m.drives.push((1, 3));

        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(0),
            q: 1,
            reset_val: reset0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 8,
            d: Some(0),
            q: 2,
            reset_val: reset1,
            reset_kind: ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::OneHot(vec![]),
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn self_feedback_flop_fixture() -> Module {
        let mut m = Module {
            name: "self_feedback".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::Cse,
            ..Module::default()
        };
        m.outputs.push(Port {
            id: 0,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });

        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 0
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 1
        m.nodes.push(Node::Constant { width: 8, value: 1 }); // 2
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![0, 2],
            width: 8,
            deps: DepSet::from_flop_virtual(0),
        }); // 3
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![1, 2],
            width: 8,
            deps: DepSet::from_flop_virtual(1),
        }); // 4
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![0, 1],
            width: 8,
            deps: DepSet::union(&[&DepSet::from_flop_virtual(0), &DepSet::from_flop_virtual(1)]),
        }); // 5
        m.drives.push((0, 5));

        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(3),
            q: 0,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 8,
            d: Some(4),
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::OneHot(vec![]),
        });

        rebuild_instance_tables(&mut m);
        m
    }

    /// Two equal-reset flops whose D-cones are each other's `Q`
    /// (`D_f = Q_g`, `D_g = Q_f`). Both `Q`s are observed by an output.
    /// This is the mutually-recursive-register class the exact pass cannot
    /// prove but bounded bisimulation can. Built with the bisimulation knob
    /// enabled under node-id / e-graph; tests toggle the gating fields.
    fn mutual_swap_flop_fixture(reset_kind: ResetKind, reset_val: u128) -> Module {
        let mut m = Module {
            name: "mutual_swap".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            bisimulation_flop_merge: true,
            ..Module::default()
        };
        m.outputs.push(Port {
            id: 0,
            name: "y0".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y1".into(),
            width: 8,
            dir: Direction::Out,
        });
        // Real clock/reset ports so the merged self-hold register emits a
        // proper `always_ff` block (downstream-clean smoke evidence).
        m.inputs.push(Port {
            id: 2,
            name: "clk".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 3,
            name: "rst_n".into(),
            width: 1,
            dir: Direction::In,
        });
        m.clock = Some(2);
        m.reset = Some(3);

        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 0 = Q_f
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 1 = Q_g
        m.drives.push((0, 0)); // y0 observes Q_f
        m.drives.push((1, 1)); // y1 observes Q_g

        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(1), // D_f = Q_g
            q: 0,
            reset_val,
            reset_kind,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 8,
            d: Some(0), // D_g = Q_f
            q: 1,
            reset_val,
            reset_kind,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    /// Two equal-reset flops with genuinely different transitions:
    /// `D_f = Q_g` but `D_g = a` (a primary input). No state correspondence
    /// makes them bisimilar, so refinement must split them.
    fn non_bisimilar_flop_fixture() -> Module {
        let mut m = Module {
            name: "non_bisim".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            bisimulation_flop_merge: true,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y0".into(),
            width: 8,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 2,
            name: "y1".into(),
            width: 8,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0 = a
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 1 = Q_f
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 2 = Q_g
        m.drives.push((1, 1)); // y0 observes Q_f
        m.drives.push((2, 2)); // y1 observes Q_g

        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(2), // D_f = Q_g
            q: 1,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 8,
            d: Some(0), // D_g = a (primary input)
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn self_hold_flop_fixture(
        width0: u32,
        width1: u32,
        reset0: u128,
        reset1: u128,
        reset_kind: ResetKind,
    ) -> Module {
        let mut m = Module {
            name: "self_hold".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::Cse,
            ..Module::default()
        };
        m.outputs.push(Port {
            id: 0,
            name: "y0".into(),
            width: width0,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y1".into(),
            width: width1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::FlopQ {
            flop: 0,
            width: width0,
        }); // 0
        m.nodes.push(Node::FlopQ {
            flop: 1,
            width: width1,
        }); // 1
        m.drives.push((0, 0));
        m.drives.push((1, 1));

        m.flops.push(Flop {
            id: 0,
            width: width0,
            d: Some(0),
            q: 0,
            reset_val: reset0,
            reset_kind,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: width1,
            d: Some(1),
            q: 1,
            reset_val: reset1,
            reset_kind,
            kind: FlopKind::QFeedback,
            mux: FlopMux::None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn non_self_duplicate_d_fixture() -> Module {
        let mut m = Module {
            name: "non_self_duplicate_d".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::Cse,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 8,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 8 }); // 0
        m.nodes.push(Node::Constant { width: 8, value: 1 }); // 1
        m.nodes.push(Node::FlopQ { flop: 0, width: 8 }); // 2
        m.nodes.push(Node::FlopQ { flop: 1, width: 8 }); // 3
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![0, 1],
            width: 8,
            deps: DepSet::from_port(0),
        }); // 4
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![0, 1],
            width: 8,
            deps: DepSet::from_port(0),
        }); // 5
        m.nodes.push(Node::Gate {
            op: GateOp::Add,
            operands: vec![2, 3],
            width: 8,
            deps: DepSet::union(&[&DepSet::from_flop_virtual(0), &DepSet::from_flop_virtual(1)]),
        }); // 6
        m.drives.push((1, 6));

        m.flops.push(Flop {
            id: 0,
            width: 8,
            d: Some(4),
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 8,
            d: Some(5),
            q: 3,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn duplicate_fsm_fixture(
        identity_mode: IdentityMode,
        factorization_level: FactorizationLevel,
        distinct_selector: bool,
    ) -> Module {
        let mut m = Module {
            name: "duplicate_fsm".into(),
            identity_mode,
            factorization_level,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "sel0".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 8,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 sel0
        let sel1 = if distinct_selector {
            m.inputs.push(Port {
                id: 2,
                name: "sel1".into(),
                width: 1,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: 2, width: 1 }); // 1 sel1
            1
        } else {
            0
        };
        let fsm0_out = m.nodes.len() as NodeId;
        m.nodes.push(Node::FsmOut { fsm: 0, width: 8 });
        let fsm1_out = m.nodes.len() as NodeId;
        m.nodes.push(Node::FsmOut { fsm: 1, width: 8 });
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![fsm0_out, fsm1_out],
            width: 8,
            deps: DepSet::union(&[&DepSet::from_fsm_virtual(0), &DepSet::from_fsm_virtual(1)]),
        });
        m.drives.push((1, m.nodes.len() as NodeId - 1));

        let transitions = vec![vec![0, 1], vec![1, 0]];
        let outputs = vec![3, 7];
        m.fsms.push(crate::ir::Fsm {
            id: 0,
            num_states: 2,
            encoding: crate::ir::FsmEncoding::Binary,
            sel: 0,
            sel_width: 1,
            transitions: transitions.clone(),
            outputs: outputs.clone(),
            out_width: 8,
            mealy_outputs: None,
        });
        m.fsms.push(crate::ir::Fsm {
            id: 1,
            num_states: 2,
            encoding: crate::ir::FsmEncoding::Binary,
            sel: sel1,
            sel_width: 1,
            transitions,
            outputs,
            out_width: 8,
            mealy_outputs: None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn semantic_equivalent_d_fixture() -> Module {
        let mut m = Module {
            name: "semantic_equivalent_d".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::Cse,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 2,
            name: "y".into(),
            width: 1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 1 }); // 1 b
        m.nodes.push(Node::FlopQ { flop: 0, width: 1 }); // 2
        m.nodes.push(Node::FlopQ { flop: 1, width: 1 }); // 3
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![1],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 4 !b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 1],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 5 a&b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 4],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 6 a&!b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![5, 6],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 7 (a&b)|(a&!b)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 4],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 8 b|!b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 8],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 9 a&(b|!b)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![2, 3],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_flop_virtual(0), &DepSet::from_flop_virtual(1)]),
        }); // 10
        m.drives.push((2, 10));

        m.flops.push(Flop {
            id: 0,
            width: 1,
            d: Some(7),
            q: 2,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });
        m.flops.push(Flop {
            id: 1,
            width: 1,
            d: Some(9),
            q: 3,
            reset_val: 0,
            reset_kind: ResetKind::Async,
            kind: FlopKind::ZeroDefault,
            mux: FlopMux::None,
        });

        rebuild_instance_tables(&mut m);
        m
    }

    fn semantic_equivalent_gate_fixture() -> Module {
        let mut m = Module {
            name: "semantic_equivalent_gate".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 2,
            name: "y0".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 3,
            name: "y1".into(),
            width: 1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 1 }); // 1 b
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![1],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 2 !b
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![0],
            width: 1,
            deps: DepSet::from_port(0),
        }); // 3 !a
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 2],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 4 a&!b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![3, 1],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 5 !a&b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![4, 5],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 6 (a&!b)|(!a&b)
        m.nodes.push(Node::Gate {
            op: GateOp::Xor,
            operands: vec![0, 1],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 7 a^b
        m.drives.push((2, 6));
        m.drives.push((3, 7));

        rebuild_instance_tables(&mut m);
        m
    }

    fn semantic_endpoint_fold_gate_fixture() -> Module {
        let mut m = Module {
            name: "semantic_endpoint_fold_gate".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "a".into(),
            width: 1,
            dir: Direction::In,
        });
        m.inputs.push(Port {
            id: 1,
            name: "b".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 2,
            name: "y".into(),
            width: 1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 a
        m.nodes.push(Node::PrimaryInput { port: 1, width: 1 }); // 1 b
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![1],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 2 !b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 3 b|!b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 3],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 4 a&(b|!b)
        m.drives.push((2, 4));

        rebuild_instance_tables(&mut m);
        m
    }

    fn semantic_constant_fold_gate_fixture() -> Module {
        let mut m = Module {
            name: "semantic_constant_fold_gate".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        m.inputs.push(Port {
            id: 0,
            name: "b".into(),
            width: 1,
            dir: Direction::In,
        });
        m.outputs.push(Port {
            id: 1,
            name: "y".into(),
            width: 1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::PrimaryInput { port: 0, width: 1 }); // 0 b
        let one = push_constant(&mut m, 1, 1); // 1
        assert_eq!(one, 1);
        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![0],
            width: 1,
            deps: DepSet::from_port(0),
        }); // 2 !b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![0, 2],
            width: 1,
            deps: DepSet::from_port(0),
        }); // 3 b|!b
        m.drives.push((1, 3));

        rebuild_instance_tables(&mut m);
        m
    }

    fn same_shape_different_endpoints_gate_fixture() -> Module {
        let mut m = Module {
            name: "same_shape_different_endpoints_gate".into(),
            identity_mode: IdentityMode::NodeId,
            factorization_level: FactorizationLevel::EGraph,
            ..Module::default()
        };
        for (id, name) in [(0, "a"), (1, "b"), (2, "c"), (3, "d")] {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: 1,
                dir: Direction::In,
            });
            m.nodes.push(Node::PrimaryInput { port: id, width: 1 });
        }
        m.outputs.push(Port {
            id: 4,
            name: "y0".into(),
            width: 1,
            dir: Direction::Out,
        });
        m.outputs.push(Port {
            id: 5,
            name: "y1".into(),
            width: 1,
            dir: Direction::Out,
        });

        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![1],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 4 !b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 4],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 5 b|!b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 5],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 6 a&(b|!b)

        m.nodes.push(Node::Gate {
            op: GateOp::Not,
            operands: vec![3],
            width: 1,
            deps: DepSet::from_port(3),
        }); // 7 !d
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![3, 7],
            width: 1,
            deps: DepSet::from_port(3),
        }); // 8 d|!d
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![2, 8],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(2), &DepSet::from_port(3)]),
        }); // 9 c&(d|!d)

        m.drives.push((4, 6));
        m.drives.push((5, 9));

        rebuild_instance_tables(&mut m);
        m
    }

    // ---- PHASE-6-ADVANCED-MOTIFS.2.1a: inferrable-memory IR core ----

    /// Minimal valid memory leaf: clk/rst_n control ports, we/waddr/
    /// wdata/raddr data inputs, one `Memory`, and a `MemRead` driving
    /// the `rdata` output. `wdata` is driven through a gate cone so a
    /// reachability regression would visibly strip it.
    fn memory_leaf(aw: u32, dw: u32, kind: crate::ir::MemKind) -> Module {
        use crate::ir::{Direction, MemKind, Memory, Port};
        let mut m = Module {
            name: "mem_leaf".into(),
            clock: Some(0),
            reset: Some(1),
            ..Module::default()
        };
        let mk_in = |m: &mut Module, id: PortId, name: &str, w: u32| {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: w,
                dir: Direction::In,
            });
        };
        mk_in(&mut m, 0, "clk", 1);
        mk_in(&mut m, 1, "rst_n", 1);
        mk_in(&mut m, 2, "we", 1);
        mk_in(&mut m, 3, "waddr", aw);
        mk_in(&mut m, 4, "wdata_a", dw);
        mk_in(&mut m, 5, "wdata_b", dw);
        mk_in(&mut m, 6, "raddr", aw);
        m.outputs.push(Port {
            id: 7,
            name: "rdata".into(),
            width: dw,
            dir: Direction::Out,
        });
        // nodes
        let we = m.nodes.len() as NodeId; // 0
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 });
        let waddr = m.nodes.len() as NodeId; // 1
        m.nodes.push(Node::PrimaryInput { port: 3, width: aw });
        let wa = m.nodes.len() as NodeId; // 2
        m.nodes.push(Node::PrimaryInput { port: 4, width: dw });
        let wb = m.nodes.len() as NodeId; // 3
        m.nodes.push(Node::PrimaryInput { port: 5, width: dw });
        let raddr = m.nodes.len() as NodeId; // 4
        m.nodes.push(Node::PrimaryInput { port: 6, width: aw });
        // wdata = wa ^ wb (a real cone feeding the memory write)
        let wdata = push_gate(&mut m, GateOp::Xor, vec![wa, wb], dw);
        let waddr_for = if matches!(kind, MemKind::SinglePort) {
            // SinglePort shares one address: raddr == waddr.
            waddr
        } else {
            raddr
        };
        let mem_id = m.memories.len() as crate::ir::MemId;
        m.memories.push(Memory {
            id: mem_id,
            addr_width: aw,
            data_width: dw,
            kind,
            we,
            waddr,
            wdata,
            raddr: waddr_for,
        });
        let rd = m.nodes.len() as NodeId;
        m.nodes.push(Node::MemRead {
            mem: mem_id,
            width: dw,
        });
        m.drives.push((7, rd));
        m
    }

    #[test]
    fn memory_leaf_roundtrips_validate_and_emit() {
        let m = memory_leaf(4, 8, crate::ir::MemKind::SimpleDualPort);
        validate(&m).expect("memory leaf must validate");
        let sv = crate::emit::to_sv(&m);
        assert!(
            sv.contains("logic [7:0] mem_0 [0:15];"),
            "must declare the inferrable array:\n{sv}"
        );
        assert!(
            sv.contains("logic [7:0] memrd_0;"),
            "must declare the registered read signal:\n{sv}"
        );
        assert!(
            sv.contains("always_ff @(posedge clk) begin"),
            "memory must use a reset-less synchronous block:\n{sv}"
        );
        assert!(
            sv.contains("mem_0[") && sv.contains("] <= ") && sv.contains("memrd_0 <= mem_0["),
            "must emit the synchronous write + registered read:\n{sv}"
        );
        // clk is exposed even with no flops (memory is sequential state).
        assert!(sv.contains("input  logic  clk") || sv.contains("input  logic clk"));
    }

    #[test]
    fn memread_keeps_memory_source_cones_through_compaction() {
        let mut m = memory_leaf(4, 8, crate::ir::MemKind::SimpleDualPort);
        // Add a genuinely dead gate that nothing references.
        let we = 0 as NodeId;
        let _dead = push_raw_gate(&mut m, GateOp::Not, vec![we], 1);
        let before = m.nodes.len();
        let removed = compact_node_ids(&mut m);
        assert!(removed >= 1, "the dead gate must be compacted away");
        assert!(m.nodes.len() < before);
        // The memory's source cones (incl. the wdata XOR) survived, so
        // the module still validates and re-emits the write/read.
        validate(&m).expect("memory module must still validate after compaction");
        let sv = crate::emit::to_sv(&m);
        assert!(
            sv.contains("memrd_0 <= mem_0[") && sv.contains("] <= "),
            "memory write/read cones must survive dead-elimination:\n{sv}"
        );
        // The Xor feeding wdata must still be present.
        assert!(
            m.nodes.iter().any(|n| matches!(
                n,
                Node::Gate {
                    op: GateOp::Xor,
                    ..
                }
            )),
            "the wdata cone (Xor) must not be dead-stripped"
        );
    }

    #[test]
    fn memread_is_structurally_distinct_and_not_cse_merged() {
        use crate::metrics::canonical_module_signature;
        // (a) A MemRead-driven module has a different canonical
        // signature than a structurally-identical PrimaryInput-driven
        // one — MemRead carries its own structural identity, so it can
        // never be CSE-merged with a non-MemRead node.
        let mem_mod = memory_leaf(4, 8, crate::ir::MemKind::SimpleDualPort);
        let mut plain = mem_mod.clone();
        plain.memories.clear();
        // redirect rdata to a plain input instead of the MemRead node
        let pin = plain.nodes.len() as NodeId;
        plain.nodes.push(Node::PrimaryInput { port: 4, width: 8 });
        plain.drives.clear();
        plain.drives.push((7, pin));
        assert_ne!(
            canonical_module_signature(&mem_mod),
            canonical_module_signature(&plain),
            "a MemRead node must be structurally distinct from a PrimaryInput"
        );
        // (b) Two distinct memories' reads are distinct leaves: they
        // are never merged, and both survive compaction.
        let mut two = memory_leaf(4, 8, crate::ir::MemKind::SimpleDualPort);
        let m1 = two.memories.len() as crate::ir::MemId;
        // a second independent memory reusing the same source nodes
        let src = &two.memories[0];
        let (we, wa, wd, ra) = (src.we, src.waddr, src.wdata, src.raddr);
        two.memories.push(crate::ir::Memory {
            id: m1,
            addr_width: 4,
            data_width: 8,
            kind: crate::ir::MemKind::SimpleDualPort,
            we,
            waddr: wa,
            wdata: wd,
            raddr: ra,
        });
        let rd1 = two.nodes.len() as NodeId;
        two.nodes.push(Node::MemRead { mem: m1, width: 8 });
        two.outputs.push(crate::ir::Port {
            id: 8,
            name: "rdata1".into(),
            width: 8,
            dir: crate::ir::Direction::Out,
        });
        two.drives.push((8, rd1));
        validate(&two).expect("two-memory module validates");
        compact_node_ids(&mut two);
        let mem_reads = two
            .nodes
            .iter()
            .filter(|n| matches!(n, Node::MemRead { .. }))
            .count();
        assert_eq!(
            mem_reads, 2,
            "two distinct memories' reads must never be CSE-merged"
        );
    }

    #[test]
    fn memory_state_identity_stays_instance_local_under_full_factorization() {
        let mut m = memory_leaf(4, 8, crate::ir::MemKind::SimpleDualPort);
        m.identity_mode = IdentityMode::NodeId;
        m.factorization_level = FactorizationLevel::EGraph;

        let m1 = m.memories.len() as crate::ir::MemId;
        let src = &m.memories[0];
        let (we, waddr, wdata, raddr) = (src.we, src.waddr, src.wdata, src.raddr);
        m.memories.push(crate::ir::Memory {
            id: m1,
            addr_width: src.addr_width,
            data_width: src.data_width,
            kind: src.kind,
            we,
            waddr,
            wdata,
            raddr,
        });
        let rd1 = m.nodes.len() as NodeId;
        m.nodes.push(Node::MemRead { mem: m1, width: 8 });
        m.outputs.push(crate::ir::Port {
            id: 8,
            name: "rdata1".into(),
            width: 8,
            dir: crate::ir::Direction::Out,
        });
        m.drives.push((8, rd1));
        validate(&m).expect("two-memory module validates before identity passes");

        assert_eq!(
            merge_equivalent_flops(&mut m),
            0,
            "memory state must not be routed through the flop merge proof"
        );
        assert_eq!(
            merge_equivalent_fsms(&mut m),
            0,
            "memory state must not be routed through the reset-defined FSM merge proof"
        );
        compact_node_ids(&mut m);
        validate(&m).expect("two-memory module validates after identity passes");

        assert_eq!(
            m.memories.len(),
            2,
            "independent memory blocks remain state-by-instance even with equal source cones"
        );
        let mem_read_ids: Vec<_> = m
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::MemRead { mem, .. } => Some(*mem),
                _ => None,
            })
            .collect();
        assert_eq!(
            mem_read_ids,
            vec![0, 1],
            "both independent memory read leaves remain addressable"
        );
    }

    // ---- PHASE-6-ADVANCED-MOTIFS.3.2a: generated-encoding FSM IR core ----

    /// Minimal valid FSM leaf: clk/rst_n control ports, two `sel`
    /// inputs XOR'd into the 1-bit transition selector (a real cone
    /// so a reachability regression visibly strips it), one `Fsm`,
    /// and a `FsmOut` driving the `q` output.
    fn fsm_leaf(num_states: u32, encoding: crate::ir::FsmEncoding) -> Module {
        use crate::ir::{Direction, Fsm, Port};
        let mut m = Module {
            name: "fsm_leaf".into(),
            clock: Some(0),
            reset: Some(1),
            ..Module::default()
        };
        let mk_in = |m: &mut Module, id: PortId, name: &str, w: u32| {
            m.inputs.push(Port {
                id,
                name: name.into(),
                width: w,
                dir: Direction::In,
            });
        };
        mk_in(&mut m, 0, "clk", 1);
        mk_in(&mut m, 1, "rst_n", 1);
        mk_in(&mut m, 2, "sel_a", 1);
        mk_in(&mut m, 3, "sel_b", 1);
        let out_width = 8u32;
        m.outputs.push(Port {
            id: 4,
            name: "q".into(),
            width: out_width,
            dir: Direction::Out,
        });
        let sa = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput { port: 2, width: 1 });
        let sb = m.nodes.len() as NodeId;
        m.nodes.push(Node::PrimaryInput { port: 3, width: 1 });
        // sel = sel_a ^ sel_b — a real generated cone feeding the FSM.
        let sel = push_gate(&mut m, GateOp::Xor, vec![sa, sb], 1);
        // Ring transitions: sel==1 advances, sel==0 holds.
        let transitions: Vec<Vec<u32>> = (0..num_states)
            .map(|s| vec![s, (s + 1) % num_states])
            .collect();
        let outputs: Vec<u128> = (0..num_states).map(|s| (s as u128) * 3 + 1).collect();
        let fsm_id = m.fsms.len() as crate::ir::FsmId;
        m.fsms.push(Fsm {
            id: fsm_id,
            num_states,
            encoding,
            sel,
            sel_width: 1,
            transitions,
            outputs,
            out_width,
            mealy_outputs: None,
        });
        let fo = m.nodes.len() as NodeId;
        m.nodes.push(Node::FsmOut {
            fsm: fsm_id,
            width: out_width,
        });
        m.drives.push((4, fo));
        m
    }

    #[test]
    fn fsm_leaf_roundtrips_validate_and_emit() {
        let m = fsm_leaf(4, crate::ir::FsmEncoding::Binary);
        validate(&m).expect("fsm leaf must validate");
        let sv = crate::emit::to_sv(&m);
        // Binary, 4 states → 2-bit state register.
        assert!(
            sv.contains("logic [1:0] fsm_state_0;"),
            "must declare the encoded-state register:\n{sv}"
        );
        assert!(
            sv.contains("logic [7:0] fsm_0;"),
            "must declare the registered Moore output:\n{sv}"
        );
        assert!(
            sv.contains("localparam logic [1:0] FSM0_S0 = 2'h0;"),
            "must emit encoding-derived state constants:\n{sv}"
        );
        assert!(
            sv.contains("always_ff @(posedge clk or negedge rst_n) begin"),
            "state register must use the async-low-reset block:\n{sv}"
        );
        assert!(
            sv.contains("if (!rst_n) fsm_state_0 <= FSM0_S0;"),
            "state must reset to state 0:\n{sv}"
        );
        assert!(
            sv.contains("case (fsm_state_0)") && sv.contains("fsm_next_0 ="),
            "must emit the next-state decode case:\n{sv}"
        );
        assert!(
            sv.contains("fsm_0 = 8'h"),
            "must emit the Moore output decode:\n{sv}"
        );
        // clk/rst_n exposed (an FSM is sequential state).
        assert!(sv.contains("clk") && sv.contains("rst_n"));
    }

    #[test]
    fn fsmout_keeps_sel_cone_through_compaction() {
        let mut m = fsm_leaf(4, crate::ir::FsmEncoding::OneHot);
        // A genuinely dead gate nothing references.
        let sa = 0 as NodeId;
        let _dead = push_raw_gate(&mut m, GateOp::Not, vec![sa], 1);
        let before = m.nodes.len();
        let removed = compact_node_ids(&mut m);
        assert!(removed >= 1, "the dead gate must be compacted away");
        assert!(m.nodes.len() < before);
        validate(&m).expect("fsm module must still validate after compaction");
        let sv = crate::emit::to_sv(&m);
        assert!(
            sv.contains("case (fsm_state_0)") && sv.contains("fsm_next_0 ="),
            "FSM next-state cone must survive dead-elimination:\n{sv}"
        );
        // The Xor feeding `sel` must still be present.
        assert!(
            m.nodes.iter().any(|n| matches!(
                n,
                Node::Gate {
                    op: GateOp::Xor,
                    ..
                }
            )),
            "the sel cone (Xor) must not be dead-stripped"
        );
    }

    #[test]
    fn fsmout_is_structurally_distinct_and_not_cse_merged() {
        use crate::metrics::canonical_module_signature;
        // (a) An FsmOut-driven module differs structurally from a
        // PrimaryInput-driven twin — FsmOut carries its own identity.
        let fsm_mod = fsm_leaf(4, crate::ir::FsmEncoding::Binary);
        let mut plain = fsm_mod.clone();
        plain.fsms.clear();
        let pin = plain.nodes.len() as NodeId;
        plain.nodes.push(Node::PrimaryInput { port: 2, width: 8 });
        plain.drives.clear();
        plain.drives.push((4, pin));
        assert_ne!(
            canonical_module_signature(&fsm_mod),
            canonical_module_signature(&plain),
            "an FsmOut node must be structurally distinct from a PrimaryInput"
        );
        // (b) Two distinct FSMs' outputs are distinct leaves: never
        // merged, both survive compaction.
        let mut two = fsm_leaf(4, crate::ir::FsmEncoding::Gray);
        let f1 = two.fsms.len() as crate::ir::FsmId;
        let src = &two.fsms[0];
        let (sel, sw, ns) = (src.sel, src.sel_width, src.num_states);
        let transitions = src.transitions.clone();
        let outputs = src.outputs.clone();
        two.fsms.push(crate::ir::Fsm {
            id: f1,
            num_states: ns,
            encoding: crate::ir::FsmEncoding::Gray,
            sel,
            sel_width: sw,
            transitions,
            outputs,
            out_width: 8,
            mealy_outputs: None,
        });
        let fo1 = two.nodes.len() as NodeId;
        two.nodes.push(Node::FsmOut { fsm: f1, width: 8 });
        two.outputs.push(crate::ir::Port {
            id: 5,
            name: "q1".into(),
            width: 8,
            dir: crate::ir::Direction::Out,
        });
        two.drives.push((5, fo1));
        validate(&two).expect("two-fsm module validates");
        compact_node_ids(&mut two);
        let fsm_outs = two
            .nodes
            .iter()
            .filter(|n| matches!(n, Node::FsmOut { .. }))
            .count();
        assert_eq!(
            fsm_outs, 2,
            "two distinct FSMs' outputs must never be CSE-merged"
        );
    }
}
