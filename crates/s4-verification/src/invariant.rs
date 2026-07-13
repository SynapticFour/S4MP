use serde::{Deserialize, Serialize};

/// Opaque invariant identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvariantId(pub u64);

/// Declarative invariant to be checked against the knowledge graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invariant {
    /// Invariant identifier.
    pub id: InvariantId,
    /// Short name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
}

/// Collection of invariants applied together.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InvariantSet {
    /// Invariants in this set.
    pub invariants: Vec<Invariant>,
}
