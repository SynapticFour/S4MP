use crate::Artifact;
use s4_core::{ArtifactId, Result};

/// Combined read/write storage capability.
pub trait Store: StoreReader + StoreWriter {}

/// Read-only access to the artifact store.
pub trait StoreReader {
    /// Read an artifact by ID. Returns `None` when not found.
    ///
    /// # Errors
    ///
    /// Returns an error if storage access fails.
    fn read(&self, id: &ArtifactId) -> Result<Option<Artifact>>;

    /// Returns true when the artifact exists.
    fn contains(&self, id: &ArtifactId) -> bool;
}

/// Write access to the artifact store.
pub trait StoreWriter {
    /// Persist an artifact and return its content address.
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails.
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId>;
}
