//! Construction-snapshot rollback machinery (`CONE-DECOMPOSITION.2`).
//!
//! A `ConstructionSnapshot` records the lengths of the four growing
//! construction surfaces (module nodes, module flops, the signal pool,
//! the flop worklist) so a speculative build that turns out trivial /
//! invalid can be rolled back to exactly the pre-attempt state. Extracted
//! verbatim from `cone.rs`; behaviour is unchanged.

use super::FlopWorklist;
use crate::gen::pool::SignalPool;
use crate::ir::{Module, NodeId};

#[derive(Clone, Copy)]
pub(crate) struct ConstructionSnapshot {
    // `pub(crate)` so the cone tests (which live in the `cone` root) can
    // inspect the recorded lengths after a snapshot/rollback round-trip.
    pub(crate) nodes_len: usize,
    pub(crate) flops_len: usize,
    pub(crate) pool_len: usize,
    pub(crate) worklist_len: usize,
}

pub(crate) fn take_construction_snapshot(
    m: &Module,
    pool: &SignalPool,
    worklist: &FlopWorklist,
) -> ConstructionSnapshot {
    ConstructionSnapshot {
        nodes_len: m.nodes.len(),
        flops_len: m.flops.len(),
        pool_len: pool.len(),
        worklist_len: worklist.len(),
    }
}

pub(crate) fn rollback_construction_snapshot(
    m: &mut Module,
    pool: &mut SignalPool,
    worklist: &mut FlopWorklist,
    snapshot: ConstructionSnapshot,
) {
    m.nodes.truncate(snapshot.nodes_len);
    m.flops.truncate(snapshot.flops_len);
    pool.truncate(snapshot.pool_len);
    worklist.truncate(snapshot.worklist_len);
    prune_intern_tables_after_node_truncate(m, snapshot.nodes_len as NodeId);
}

fn prune_intern_tables_after_node_truncate(m: &mut Module, cutoff: NodeId) {
    if cutoff >= m.nodes.len() as NodeId
        && m.gate_instances
            .values()
            .all(|ids| ids.last().map(|id| *id < cutoff).unwrap_or(true))
        && m.const_instances
            .values()
            .all(|ids| ids.last().map(|id| *id < cutoff).unwrap_or(true))
    {
        return;
    }

    m.gate_instances.retain(|_, ids| {
        ids.retain(|id| *id < cutoff);
        !ids.is_empty()
    });
    m.const_instances.retain(|_, ids| {
        ids.retain(|id| *id < cutoff);
        !ids.is_empty()
    });
}
