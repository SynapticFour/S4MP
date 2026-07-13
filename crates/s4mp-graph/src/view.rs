use s4mp_model::{Edge, Fact, Node};
use std::collections::HashMap;

/// Immutable in-memory graph view.
#[derive(Clone, Default, Debug)]
pub struct GraphView {
    pub nodes: HashMap<s4mp_model::NodeId, Node>,
    pub edges: Vec<Edge>,
    pub facts: Vec<Fact>,
}

impl GraphView {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
