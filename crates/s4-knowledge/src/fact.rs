use crate::Provenance;
use s4_graph::{Edge, Node};
use serde::{Deserialize, Serialize};

/// A knowledge fact with lifecycle and confidence metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fact {
    /// Fact classification.
    pub kind: FactKind,
    /// Lifecycle state.
    pub lifecycle: FactLifecycle,
    /// Confidence score.
    pub confidence: Confidence,
    /// Origin metadata.
    pub provenance: Provenance,
    /// Underlying payload.
    pub payload: FactPayload,
}

/// High-level fact classification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    /// Structural graph fact.
    Structural,
    /// Semantic inference.
    Semantic,
    /// Architectural assertion.
    Architectural,
    /// AI-generated proposal.
    Proposed,
}

/// Fact lifecycle — LLM outputs start as `Proposed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactLifecycle {
    /// Awaiting acceptance.
    Proposed,
    /// Accepted as truth.
    Accepted,
    /// Rejected.
    Rejected,
    /// Superseded by newer fact.
    Superseded,
}

/// Confidence score in the range `0.0`–`1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Confidence(pub f32);

impl Confidence {
    /// Deterministic, fully trusted fact.
    pub const CERTAIN: Self = Self(1.0);

    /// Clamp a raw value into the valid range.
    #[must_use]
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

/// Fact payload variants.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactPayload {
    /// Graph node fact.
    Node(Node),
    /// Graph edge fact.
    Edge(Edge),
}
