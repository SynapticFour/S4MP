use s4_core::EntityId;
use serde::{Deserialize, Serialize};

/// Opaque node identifier within a graph view.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Snapshot-scoped knowledge id for traces that refer to this graph node.
    ///
    /// Distinct from USIR module-local ids. Do not mix identifiers across graphs
    /// or snapshots.
    #[must_use]
    pub fn as_entity_id(self, graph: &str) -> EntityId {
        EntityId::new(graph, self.0)
    }
}

/// Graph node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    /// Node identifier.
    pub id: NodeId,
    /// Node classification.
    pub kind: NodeKind,
    /// Display label.
    pub label: String,
    /// Optional signature (from USIR); used by correspondence v2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Standard node kinds. Extensions use the `Extension` variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Source file or module.
    Module,
    /// Named symbol.
    Symbol,
    /// Callable entity (function, method).
    Callable,
    /// Type definition.
    Type,
    /// Package or crate boundary.
    Package,
    /// Plugin-defined kind.
    Extension(String),
}
