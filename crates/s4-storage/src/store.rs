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

    /// Persist `artifact` under its content hash and record a **pointer** at `id`.
    ///
    /// Secondary indexes (USIR cache keys) must not store envelopes at a non-hash
    /// path. [`crate::FileSystemStore`] `read` follows the pointer to the
    /// content-addressed blob. Unlike [`Self::write`], `id` is an index key, not
    /// the envelope hash.
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails.
    fn write_at(&mut self, id: ArtifactId, artifact: &Artifact) -> Result<()>;
}
