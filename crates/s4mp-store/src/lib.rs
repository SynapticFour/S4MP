//! Content-addressed artifact store (CAS) for immutable S4MP artifacts.

pub mod memory;

pub use memory::MemoryStore;

use s4mp_core::{ArtifactId, Result, S4mpError};
use s4mp_schema::Artifact;

/// Store trait — all artifact I/O crosses this boundary.
pub trait Store: StoreReader + StoreWriter {}

pub trait StoreReader {
    fn read(&self, id: &ArtifactId) -> Result<Option<Artifact>>;
    fn contains(&self, id: &ArtifactId) -> bool;
}

pub trait StoreWriter {
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId>;
}

/// Resolve an artifact or return a store error.
pub fn get(store: &impl StoreReader, id: &ArtifactId) -> Result<Artifact> {
    store
        .read(id)?
        .ok_or_else(|| S4mpError::Store(format!("artifact not found: {id}")))
}
