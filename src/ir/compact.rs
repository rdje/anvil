//! Post-construction IR finalization passes.
//!
//! Rule 18 says zero orphan gates at the end of construction — every
//! gate must have at least one consumer (another gate's operand, a
//! flop field, or an output drive). Today's generator enforces this
//! by construction via `build_cone`'s snapshot/rollback and
//! `process_signal_frame`'s existing-operand fallback. That keeps
//! the IR Rule-18-clean without any post-pass.
//!
//! This module houses two post-construction passes:
//!
//! - `merge_equivalent_flops(&mut m)`: a conservative stateful
//!   sharing pass that runs only once flop D-cones exist. Under
//!   `identity_mode = node-id` with effective factorization level
//!   at least `Cse`, flops with the same emitted state semantics
//!   (`width`, reset, same canonical leaf endpoints, and a D-cone
//!   functionality proven either by the current normalized proof form
//!   or by a bounded small-support semantic check) are collapsed to one
//!   state element.
//! - `compact_node_ids(&mut m)`: a defensive reachability pass that
//!   walks from roots, identifies any node that became orphaned by a
//!   construction-time rewrite (e.g. the `Not(Not(x)) → x`
//!   peephole, which leaves the inner `Not` referenced only by the
//!   outer `Not` call), and compacts the `m.nodes` arena to only the
//!   reachable set.
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
//! - `m.nodes` contains only nodes reachable from some root (drive,
//!   flop.d, flop.q, flop.mux field).
//! - Every `NodeId` in `m.nodes[*].operands`, `m.drives`, and the
//!   `Flop` / `FlopMux` fields points to a valid index in the new
//!   `m.nodes`.
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
//! `merge_equivalent_flops` is not a general sequential-equality
//! prover, and `compact_node_ids` is not a semantic merge at all.
//! Wider semantic equivalence across arbitrary gate trees and
//! stateful motifs remains the e-graph aspiration (Rule 21c).

use super::types::{Flop, FlopId, FlopMux, GateOp, Module, Node, NodeId, PortId, ResetKind};
use std::collections::{BTreeSet, HashMap};

const MAX_SEMANTIC_SUPPORT_BITS: u32 = 10;

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

    // 1. Mark reachable nodes by BFS from every root. Roots are:
    //    - `m.drives` (output drive-roots)
    //    - every `Flop` field that holds a `NodeId`
    //    The BFS body walks Gate operands recursively. Primary
    //    inputs / constants / FlopQ nodes have no operands — they're
    //    leaves and terminate the walk.
    let mut reachable = vec![false; n];
    let mut stack: Vec<NodeId> = Vec::new();

    for (_, root) in &m.drives {
        if !reachable[*root as usize] {
            reachable[*root as usize] = true;
            stack.push(*root);
        }
    }
    for flop in &m.flops {
        if let Some(d) = flop.d {
            if !reachable[d as usize] {
                reachable[d as usize] = true;
                stack.push(d);
            }
        }
        if !reachable[flop.q as usize] {
            reachable[flop.q as usize] = true;
            stack.push(flop.q);
        }
        match &flop.mux {
            FlopMux::None => {}
            FlopMux::OneHot(arms) => {
                for arm in arms {
                    for nid in [arm.data, arm.sel] {
                        if !reachable[nid as usize] {
                            reachable[nid as usize] = true;
                            stack.push(nid);
                        }
                    }
                }
            }
            FlopMux::Encoded { sel, data } => {
                if !reachable[*sel as usize] {
                    reachable[*sel as usize] = true;
                    stack.push(*sel);
                }
                for d in data {
                    if !reachable[*d as usize] {
                        reachable[*d as usize] = true;
                        stack.push(*d);
                    }
                }
            }
        }
    }

    while let Some(nid) = stack.pop() {
        if let Node::Gate { operands, .. } = &m.nodes[nid as usize] {
            // Operands are u32 — copy to avoid borrow issues.
            let ops: Vec<NodeId> = operands.clone();
            for op in ops {
                if !reachable[op as usize] {
                    reachable[op as usize] = true;
                    stack.push(op);
                }
            }
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
            Node::FlopQ { flop, width } => Node::FlopQ { flop, width },
            Node::Gate {
                op,
                operands,
                width,
                deps,
            } => {
                let new_operands: Vec<NodeId> = operands
                    .into_iter()
                    .map(|o| remap(o, &old_to_new))
                    .collect();
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

    // 6. Rewrite flops.
    for flop in m.flops.iter_mut() {
        rewrite_flop(flop, &old_to_new, &remap);
    }

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
    use crate::ir::{DepSet, Direction, FlopKind, Port, ResetKind};

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
}
