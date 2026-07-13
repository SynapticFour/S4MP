use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// Immutable manifest linking artifacts that form a coherent snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    /// Root artifact of this manifest.
    pub root: ArtifactId,
    /// Optional parent manifest for incremental history.
    pub parent: Option<ArtifactId>,
    /// Member artifacts and their roles.
    pub members: Vec<ManifestRef>,
}

/// Reference to an artifact within a manifest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestRef {
    /// Content-addressed artifact identifier.
    pub id: ArtifactId,
    /// Role label (e.g. `"usir"`, `"graph"`).
    pub role: String,
}
