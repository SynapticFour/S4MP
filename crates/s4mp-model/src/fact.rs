use crate::{Edge, Node, Provenance};
use serde::{Deserialize, Serialize};

/// A fact is a node or edge with lifecycle and confidence metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fact {
    pub lifecycle: FactLifecycle,
    pub confidence: Confidence,
    pub provenance: Provenance,
    pub payload: FactPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactPayload {
    Node(Node),
    Edge(Edge),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactLifecycle {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
}

/// Confidence score from 0.0 to 1.0. Deterministic facts use 1.0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Confidence(pub f32);

impl Confidence {
    pub const CERTAIN: Self = Self(1.0);

    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}
