use s4_core::{ArtifactId, ProjectId};
use serde::{Deserialize, Serialize};

/// Platform domain event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Event classification.
    pub kind: EventKind,
    /// Originating project, if applicable.
    pub project_id: Option<ProjectId>,
    /// ISO-8601 timestamp.
    pub timestamp: String,
}

/// Known event kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Repository import finished.
    ImportCompleted {
        /// Resulting snapshot artifact.
        snapshot: ArtifactId,
    },
    /// Graph materialization finished.
    GraphUpdated {
        /// Graph projection artifact.
        projection: ArtifactId,
    },
    /// Analysis pipeline finished.
    AnalysisCompleted {
        /// Findings artifact.
        findings: ArtifactId,
    },
    /// Certificate issued or revoked.
    CertificateChanged {
        /// Certificate artifact.
        certificate: ArtifactId,
    },
    /// Extension event with namespaced kind.
    Extension(String),
}
