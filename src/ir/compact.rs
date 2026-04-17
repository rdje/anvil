//! Post-construction NodeId compaction.
//!
//! Rule 18 says zero orphan gates at the end of construction — every
//! gate must have at least one consumer (another gate's operand, a
//! flop field, or an output drive). Today's generator enforces this
//! by construction via `build_cone`'s snapshot/rollback and
//! `process_signal_frame`'s existing-operand fallback. That keeps
//! the IR Rule-18-clean without any post-pass.
//!
//! This module adds a defensive final pass that walks from roots,
//! identifies any node that became orphaned by a construction-time
//! rewrite (e.g. the `Not(Not(x)) → x` peephole, which leaves the
//! inner `Not` referenced only by the outer `Not` call), and
//! compacts the `m.nodes` arena to only the reachable set.
//!
//! The pass is idempotent and a no-op when there are no orphans.
//! It exists primarily to unblock rewrites that would otherwise
//! orphan intermediate gates. Without the pass those rewrites would
//! have to be suppressed to stay Rule-18-clean (as they were before
//! this module).
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
//! This pass does NOT merge semantically equivalent expressions —
//! that's the full factorization ladder's job (Rule 21 / 21b / 21c).
//! Compaction is strictly about removing unreachable nodes.

use super::types::{Flop, FlopMux, GateOp, Module, Node, NodeId};
use std::collections::HashMap;

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
                let new_operands: Vec<NodeId> =
                    operands.into_iter().map(|o| remap(o, &old_to_new)).collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DepSet, Direction, Port};

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
        let (add, _) =
            m.intern_gate(GateOp::Add, vec![x, c7], 8, DepSet::from_port(0));
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
        let (live_add, _) =
            m.intern_gate(GateOp::Add, vec![x, c7], 8, DepSet::from_port(0));
        m.drives.push((1, live_add));

        // Orphan gate: built, never referenced.
        let (c3, _) = m.intern_constant(8, 3);
        let (_orphan, _) =
            m.intern_gate(GateOp::Sub, vec![x, c3], 8, DepSet::from_port(0));

        let orphan_count_before = count_orphan_gates(&m);
        assert!(orphan_count_before > 0, "test should inject an orphan");

        let n_before = m.nodes.len();
        let removed = compact_node_ids(&mut m);
        assert!(removed >= 1, "expected at least the Sub orphan to be removed");
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
        let (a1, _) =
            m.intern_gate(GateOp::Add, vec![x, c1], 8, DepSet::from_port(0));
        let (a2, _) =
            m.intern_gate(GateOp::Add, vec![a1, c2], 8, DepSet::from_port(0));
        m.drives.push((1, a2));
        // Orphan between them.
        let (c99, _) = m.intern_constant(8, 99);
        let (_orphan, _) =
            m.intern_gate(GateOp::Sub, vec![a1, c99], 8, DepSet::from_port(0));

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
}
