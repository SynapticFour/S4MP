use crate::view::{GraphBuilder, GraphView};
use crate::{Edge, GraphLayer, Node, NodeId};
use s4_core::{Result, S4Error};
use std::collections::HashMap;

/// In-memory graph builder backed by vectors.
#[derive(Clone, Debug, Default)]
pub struct InMemoryGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_ids: HashMap<NodeId, usize>,
}

impl InMemoryGraph {
    /// Create an empty graph builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl GraphBuilder for InMemoryGraph {
    fn add_node(&mut self, node: Node) -> Result<()> {
        if self.node_ids.contains_key(&node.id) {
            return Err(S4Error::InvalidId(format!(
                "duplicate node id: {}",
                node.id.0
            )));
        }
        let index = self.nodes.len();
        self.node_ids.insert(node.id, index);
        self.nodes.push(node);
        Ok(())
    }

    fn add_edge(&mut self, edge: Edge) -> Result<()> {
        if !self.node_ids.contains_key(&edge.from) {
            return Err(S4Error::InvalidId(format!(
                "edge source node not found: {}",
                edge.from.0
            )));
        }
        if !self.node_ids.contains_key(&edge.to) {
            return Err(S4Error::InvalidId(format!(
                "edge target node not found: {}",
                edge.to.0
            )));
        }
        self.edges.push(edge);
        Ok(())
    }

    fn build(self: Box<Self>) -> Result<Box<dyn GraphView>> {
        Ok(Box::new(InMemoryGraphView::new(self.nodes, self.edges)))
    }
}

/// Read-only in-memory [`GraphView`] over semantic-layer nodes and edges.
#[derive(Clone, Debug)]
pub struct InMemoryGraphView {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_index: HashMap<NodeId, usize>,
}

impl InMemoryGraphView {
    /// Construct a view from finalized node and edge lists.
    #[must_use]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let node_index = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id, index))
            .collect();
        Self {
            nodes,
            edges,
            node_index,
        }
    }
}

impl GraphView for InMemoryGraphView {
    fn layer(&self) -> GraphLayer {
        GraphLayer::Semantic
    }

    fn node(&self, id: NodeId) -> Option<&Node> {
        self.node_index
            .get(&id)
            .and_then(|&index| self.nodes.get(index))
    }

    fn edges(&self) -> Box<dyn Iterator<Item = &Edge> + '_> {
        Box::new(self.edges.iter())
    }

    fn nodes(&self) -> Box<dyn Iterator<Item = &Node> + '_> {
        Box::new(self.nodes.iter())
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
