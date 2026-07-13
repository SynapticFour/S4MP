use crate::NodeId;
use serde::{Deserialize, Serialize};

/// Directed edge between graph nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    /// Source node.
    pub from: NodeId,
    /// Target node.
    pub to: NodeId,
    /// Edge classification.
    pub kind: EdgeKind,
}

/// Standard edge kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Definition relationship.
    Defines,
    /// Reference relationship.
    References,
    /// Call relationship.
    Calls,
    /// Type implementation.
    Implements,
    /// Module or package dependency.
    DependsOn,
    /// Plugin-defined kind.
    Extension(String),
}
