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
    ///
    /// # Errors
    ///
    /// Returns an error if storage access fails (as opposed to a missing file).
    fn contains(&self, id: &ArtifactId) -> Result<bool>;
}

/// Write access to the artifact store.
pub trait StoreWriter {
    /// Persist an artifact and return its content address.
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails.
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId>;

    /// Persist `artifact` under a caller-chosen id (secondary indexes).
    ///
    /// Unlike [`Self::write`], the id is not derived from the envelope bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails.
    fn write_at(&mut self, id: ArtifactId, artifact: &Artifact) -> Result<()>;
}
