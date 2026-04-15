//! SignalPool: the set of already-declared signals that cone recursion
//! may terminate at.

use crate::ir::{DepSet, NodeId};

#[derive(Debug, Default, Clone)]
pub struct PoolEntry {
    pub node: NodeId,
    pub width: u32,
    pub deps: DepSet,
}

#[derive(Debug, Default, Clone)]
pub struct SignalPool {
    entries: Vec<PoolEntry>,
}

impl SignalPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, node: NodeId, width: u32, deps: DepSet) {
        self.entries.push(PoolEntry { node, width, deps });
    }

    pub fn of_width(&self, w: u32) -> impl Iterator<Item = &PoolEntry> {
        self.entries.iter().filter(move |e| e.width == w)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PoolEntry> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
