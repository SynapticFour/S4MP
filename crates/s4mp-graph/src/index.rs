use s4mp_model::{NodeId, NodeKind};
use std::collections::HashMap;

/// Indexes for fast graph traversal.
#[derive(Default)]
pub struct GraphIndex {
    by_kind: HashMap<NodeKind, Vec<NodeId>>,
}

impl GraphIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: NodeId, kind: NodeKind) {
        self.by_kind.entry(kind).or_default().push(id);
    }

    pub fn by_kind(&self, kind: &NodeKind) -> Option<&[NodeId]> {
        self.by_kind.get(kind).map(|v| v.as_slice())
    }
}
