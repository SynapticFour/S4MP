use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// Reference to an immutable project snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotRef {
    /// Manifest or root artifact for the snapshot.
    pub manifest: ArtifactId,
    /// Optional label (branch, tag, revision).
    pub label: Option<String>,
}
