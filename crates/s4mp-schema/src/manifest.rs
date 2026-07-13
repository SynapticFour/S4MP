use s4mp_core::{ArtifactId, SnapshotId};
use serde::{Deserialize, Serialize};

/// Immutable manifest linking artifacts that form a snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub id: SnapshotId,
    pub parent: Option<SnapshotId>,
    pub artifacts: Vec<ArtifactRef>,
    pub metadata: ManifestMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub id: ArtifactId,
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub message: Option<String>,
    pub timestamp: Option<String>,
}

impl Manifest {
    pub fn artifact_ids(&self) -> impl Iterator<Item = &ArtifactId> {
        self.artifacts.iter().map(|r| &r.id)
    }
}
