use crate::RequirementId;
use s4_core::EntityId;
use s4_core::Result;
use serde::{Deserialize, Serialize};

/// Trace link between a requirement and an implementation artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceLink {
    /// Source requirement.
    pub requirement: RequirementId,
    /// Target code entity.
    pub target: EntityId,
    /// Link classification.
    pub kind: TraceLinkKind,
}

/// Traceability link kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceLinkKind {
    /// Requirement is satisfied by target.
    Satisfies,
    /// Requirement is verified by test target.
    VerifiedBy,
    /// Requirement is implemented by target.
    ImplementedBy,
}

/// Graph of requirements trace links.
pub trait TraceabilityGraph: Send + Sync {
    /// All requirements in the graph.
    fn requirements(&self) -> Box<dyn Iterator<Item = RequirementId> + '_>;

    /// Trace links originating from a requirement.
    fn traces_from(&self, requirement: RequirementId) -> Result<Vec<TraceLink>>;
}
