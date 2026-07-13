//! Knowledge graph domain model: nodes, edges, facts, and provenance.

pub mod edge;
pub mod fact;
pub mod node;
pub mod provenance;

pub use edge::{Edge, EdgeKind};
pub use fact::{Confidence, Fact, FactLifecycle};
pub use node::{Node, NodeId, NodeKind};
pub use provenance::Provenance;
