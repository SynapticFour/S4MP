use crate::{GraphView, Node, NodeKind};
use s4_core::Result;

/// Result of a graph query.
#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    /// Matching nodes.
    pub nodes: Vec<Node>,
}

/// Graph query interface (S4QL foundation).
pub trait GraphQuery: Send + Sync {
    /// Execute a query expression against a graph view.
    ///
    /// # Errors
    ///
    /// Returns an error if the query is invalid or execution fails.
    fn execute(&self, view: &dyn GraphView, expression: &str) -> Result<QueryResult>;
}

/// Built-in query expression shapes (parser to be added later).
#[derive(Clone, Debug)]
pub enum QueryExpr {
    /// Match all nodes.
    All,
    /// Match nodes of a given kind.
    MatchKind(NodeKind),
}
