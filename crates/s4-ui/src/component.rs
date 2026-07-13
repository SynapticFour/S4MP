use serde::{Deserialize, Serialize};

/// Opaque component identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u64);

/// UI component descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Component {
    /// Component identifier.
    pub id: ComponentId,
    /// Component classification.
    pub kind: ComponentKind,
    /// JSON props payload for the renderer.
    pub props: serde_json::Value,
}

/// Standard component kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    /// Graph visualization.
    GraphCanvas,
    /// Finding list panel.
    FindingList,
    /// Requirement trace matrix.
    TraceMatrix,
    /// Certificate status badge.
    CertificateBadge,
    /// Extension component.
    Extension(String),
}
