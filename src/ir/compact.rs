//! Post-construction IR finalization passes.
//!
//! Rule 18 says zero orphan gates at the end of construction — every
//! gate must have at least one consumer (another gate's operand, a
//! flop field, or an output drive). Today's generator enforces this
//! by construction via `build_cone`'s snapshot/rollback and
//! `process_signal_frame`'s existing-operand fallback. That keeps
//! the IR Rule-18-clean without any post-pass.
//!
//! This module houses three post-construction passes:
//!
//! - `merge_equivalent_gates(&mut m)`: a bounded semantic-sharing pass
//!   for combinational nodes. Under `identity_mode = node-id` with
//!   effective factorization level `EGraph`, gates with the same
//!   endpoint-preserving proof collapse to one node.
//! - `merge_equivalent_flops(&mut m)`: a conservative stateful
//!   sharing pass that runs only once flop D-cones exist. Under
//!   `identity_mode = node-id` with effective factorization level
//!   at least `Cse`, flops with the same emitted state semantics
//!   (`width`, reset, same canonical leaf endpoints, and a D-cone
//!   functionality proven either by the current normalized proof form
//!   or by a bounded small-support semantic check) are collapsed to one
//!   state element.
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
//! e-graph aspiration (Rule 21c).

use super::types::{Flop, FlopId, FlopMux, GateOp, Module, Node, NodeId, PortId, ResetKind};
use crate::config::FactorizationLevel;
use std::collections::{BTreeSet, HashMap};

const MAX_SEMANTIC_SUPPORT_BITS: u32 = 10;
const MAX_SEMANTIC_EXACT_ENDPOINTS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FlopSignature {
    width: u32,
    d: ConeProof,
    reset_val: u128,
    reset_kind: ResetKind,
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
    PrimaryInput { port: PortId, width: u32 },
    FlopQ { flop: FlopId, width: u32 },
}

impl LeafEndpoint {
    fn width(self) -> u32 {
        match self {
            LeafEndpoint::PrimaryInput { width, .. } | LeafEndpoint::FlopQ { width, .. } => width,
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
) -> StructuralSigId {
    if let Some(&sig_id) = memo.get(&node_id) {
        return sig_id;
    }

    let sig_id = match &m.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => ctx.intern(StructuralNodeShape::PrimaryInput {
            port: *port,
            width: *width,
        }),
        Node::Constant { width, value } => ctx.intern(StructuralNodeShape::Constant {
            width: *width,
            value: *value,
        }),
        Node::FlopQ { flop, width } => ctx.intern(StructuralNodeShape::FlopQ {
            flop: *flop,
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
                .map(|&operand| structural_node_sig_id(m, operand, memo, ctx))
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

fn collect_leaf_endpoints(
    m: &Module,
    node_id: NodeId,
    memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
) -> BTreeSet<LeafEndpoint> {
    if let Some(endpoints) = memo.get(&node_id) {
        return endpoints.clone();
    }

    let endpoints = match &m.nodes[node_id as usize] {
        Node::PrimaryInput { port, width } => BTreeSet::from([LeafEndpoint::PrimaryInput {
            port: *port,
            width: *width,
        }]),
        Node::FlopQ { flop, width } => BTreeSet::from([LeafEndpoint::FlopQ {
            flop: *flop,
            width: *width,
        }]),
        Node::Constant { .. } => BTreeSet::new(),
        Node::Gate { operands, .. } => {
            let mut out = BTreeSet::new();
            for &operand in operands {
                out.extend(collect_leaf_endpoints(m, operand, memo));
            }
            out
        }
    };

    memo.insert(node_id, endpoints.clone());
    endpoints
}

fn evaluate_node_under_assignment(
    m: &Module,
    node_id: NodeId,
    assignment: u128,
    endpoint_offsets: &HashMap<LeafEndpoint, u32>,
    memo: &mut HashMap<NodeId, u128>,
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
        Node::FlopQ { flop, width } => {
            let endpoint = LeafEndpoint::FlopQ {
                flop: *flop,
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
                    evaluate_node_under_assignment(m, operand, assignment, endpoint_offsets, memo)
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

fn semantic_cone_proof(
    m: &Module,
    node_id: NodeId,
    endpoint_memo: &mut HashMap<NodeId, BTreeSet<LeafEndpoint>>,
) -> Option<SemanticConeProof> {
    if m.nodes[node_id as usize].width() > 128 {
        return None;
    }

    let endpoints: Vec<LeafEndpoint> = collect_leaf_endpoints(m, node_id, endpoint_memo)
        .into_iter()
        .collect();
    let support_bits: u32 = endpoints.iter().map(|endpoint| endpoint.width()).sum();
    if support_bits > MAX_SEMANTIC_SUPPORT_BITS {
        return None;
    }

    let mut endpoint_offsets: HashMap<LeafEndpoint, u32> = HashMap::new();
    let mut next_offset = 0u32;
    for endpoint in &endpoints {
        endpoint_offsets.insert(*endpoint, next_offset);
        next_offset += endpoint.width();
    }

    let assignment_count = 1usize << support_bits;
    let mut outputs = Vec::with_capacity(assignment_count);
    for assignment in 0..assignment_count {
        let mut memo: HashMap<NodeId, u128> = HashMap::new();
        outputs.push(evaluate_node_under_assignment(
            m,
            node_id,
            assignment as u128,
            &endpoint_offsets,
            &mut memo,
        ));
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

    let endpoints = collect_leaf_endpoints(m, node_id, endpoint_memo);
    if endpoints.len() > MAX_SEMANTIC_EXACT_ENDPOINTS {
        return false;
    }

    let support_bits: u32 = endpoints.iter().map(|endpoint| endpoint.width()).sum();
    support_bits <= MAX_SEMANTIC_SUPPORT_BITS
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
        crate::gen::cone::prove_node_exact_value(m, node_id)
            .or_else(|| semantic_exact_value(m, node_id, endpoint_memo, semantic_exact_memo))
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
    let exact = semantic_cone_proof(m, node_id, endpoint_memo).and_then(|proof| {
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
) -> ConeProof {
    if let Some(proof) = semantic_memo.get(&node_id) {
        if let Some(proof) = proof {
            return ConeProof::Semantic(proof.clone());
        }
    } else {
        let proof = semantic_cone_proof(m, node_id, endpoint_memo);
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
    ))
}

/// Merge duplicate combinational gates after every cone is known.
///
/// This is the first bounded semantic fragment of the `EGraph` intent:
/// under `identity_mode = node-id` with requested/effective `EGraph`,
/// two gates can collapse even when their literal subgraph shapes
/// differ, provided ANVIL can prove they implement the same function
/// over the same canonical leaf endpoints.
///
/// The proof is intentionally bounded:
///
/// - first try the same endpoint-aware proof machinery used by state;
/// - use bounded small-support semantic truth tables when available;
/// - otherwise fall back to the already-normalized structural proof.
///
/// Returns the number of duplicate gates rewired to an earlier
/// canonical node.
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
        let Node::Gate { width, .. } = &m.nodes[node_id as usize] else {
            continue;
        };
        let sig = GateSignature {
            width: *width,
            proof: cone_proof(
                m,
                node_id,
                &mut structural_memo,
                &mut structural_ctx,
                &mut endpoint_memo,
                &mut semantic_memo,
            ),
        };
        if let Some(&canonical) = canonical_by_sig.get(&sig) {
            node_remap.insert(node_id, canonical);
            removed += 1;
        } else {
            canonical_by_sig.insert(sig, node_id);
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
/// redirected to the canonical Q.
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
            d: cone_proof(
                m,
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
            Node::PrimaryInput { .. } | Node::Constant { .. } => {}
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
    for flop in &mut m.flops {
        rewrite_flop_from_partial_map(flop, &q_node_remap);
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

        if !any_spliced {
            continue;
        }

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

    // 1. Mark reachable nodes by BFS from every output drive-root.
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
            Node::PrimaryInput { .. } | Node::Constant { .. } => {}
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
            Node::PrimaryInput { .. } | Node::FlopQ { .. } => {}
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
    use crate::ir::{DepSet, Direction, Flop, FlopKind, FlopMux, Port, ResetKind};

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
    fn merge_equivalent_flops_rewrites_consumers_and_deps() {
        let mut m =
            exact_signature_flop_fixture(IdentityMode::NodeId, FactorizationLevel::Cse, 0, 0);

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 1);
        assert_eq!(m.flops.len(), 1);
        assert_eq!(m.flops_merged, 0, "pass returns count; caller records it");

        let Node::Gate { operands, deps, .. } = &m.nodes[3] else {
            panic!("drive root should still be the add gate");
        };
        assert_eq!(operands, &vec![1, 1]);
        assert_eq!(deps.len(), 1, "virtual flop deps should coalesce");
        assert!(deps.contains(0x8000_0000));

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
    fn merge_equivalent_flops_keeps_self_feedback_cones_distinct_when_q_endpoints_differ() {
        let mut m = self_feedback_flop_fixture();

        let removed = merge_equivalent_flops(&mut m);
        assert_eq!(removed, 0, "different q endpoints must stay distinct");
        assert_eq!(m.flops.len(), 2);
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
        assert_eq!(
            compacted, 2,
            "duplicate semantic cone and dead subtrees should compact"
        );
        validate(&m).expect("merged semantic-equivalent gates should still validate");
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
            Node::FlopQ { flop, .. } => DepSet::from_flop_virtual(*flop),
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
            op: GateOp::And,
            operands: vec![0, 1],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 3 a&b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 2],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 4 a&!b
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![3, 4],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 5 (a&b)|(a&!b)
        m.nodes.push(Node::Gate {
            op: GateOp::Or,
            operands: vec![1, 2],
            width: 1,
            deps: DepSet::from_port(1),
        }); // 6 b|!b
        m.nodes.push(Node::Gate {
            op: GateOp::And,
            operands: vec![0, 6],
            width: 1,
            deps: DepSet::union(&[&DepSet::from_port(0), &DepSet::from_port(1)]),
        }); // 7 a&(b|!b)
        m.drives.push((2, 5));
        m.drives.push((3, 7));

        rebuild_instance_tables(&mut m);
        m
    }
}
