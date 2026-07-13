use crate::GraphView;
use s4mp_model::{Edge, Node};

/// Constructs graph views from facts and IR materializations.
#[derive(Default)]
pub struct GraphBuilder {
    view: GraphView,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) {
        self.view.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.view.edges.push(edge);
    }

    pub fn build(self) -> GraphView {
        self.view
    }
}
