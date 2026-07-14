//! # s4-graph
//!
//! Universal code graph contracts: nodes, edges, layers, and queries.

#![warn(missing_docs)]

/// Graph edge types.
pub mod edge;
/// Graph layer enumeration.
pub mod layer;
/// In-memory graph builder and view.
pub mod memory;
/// Graph node types.
pub mod node;
/// Graph query traits.
pub mod query;
/// Graph view and builder traits.
pub mod view;

pub use edge::{Edge, EdgeKind};
pub use layer::GraphLayer;
pub use memory::{InMemoryGraph, InMemoryGraphView};
pub use node::{Node, NodeId, NodeKind};
pub use query::{GraphQuery, QueryResult};
pub use view::{GraphBuilder, GraphView};
