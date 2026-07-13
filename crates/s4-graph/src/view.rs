use crate::{Edge, GraphLayer, Node, NodeId};
use s4_core::Result;

/// Read-only view over a materialized graph layer.
pub trait GraphView: Send + Sync {
    /// Layer represented by this view.
    fn layer(&self) -> GraphLayer;

    /// Lookup node by ID.
    fn node(&self, id: NodeId) -> Option<&Node>;

    /// Iterate all edges.
    fn edges(&self) -> Box<dyn Iterator<Item = &Edge> + '_>;

    /// Number of nodes in the view.
    fn node_count(&self) -> usize;
}

/// Mutable graph builder (implementation-provided).
pub trait GraphBuilder: Send + Sync {
    /// Add a node to the graph under construction.
    ///
    /// # Errors
    ///
    /// Returns an error if the node cannot be added.
    fn add_node(&mut self, node: Node) -> Result<()>;

    /// Add an edge to the graph under construction.
    ///
    /// # Errors
    ///
    /// Returns an error if the edge cannot be added.
    fn add_edge(&mut self, edge: Edge) -> Result<()>;

    /// Finalize into a read-only view.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is invalid.
    fn build(self: Box<Self>) -> Result<Box<dyn GraphView>>;
}
