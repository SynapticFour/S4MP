use crate::{StoreReader, StoreWriter};
use s4mp_core::{ArtifactId, Result};
use s4mp_schema::Artifact;
use std::collections::HashMap;

/// In-memory CAS implementation for development and testing.
#[derive(Default)]
pub struct MemoryStore {
    artifacts: HashMap<ArtifactId, Artifact>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StoreReader for MemoryStore {
    fn read(&self, id: &ArtifactId) -> Result<Option<Artifact>> {
        Ok(self.artifacts.get(id).cloned())
    }

    fn contains(&self, id: &ArtifactId) -> bool {
        self.artifacts.contains_key(id)
    }
}

impl StoreWriter for MemoryStore {
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId> {
        let id = artifact.id();
        self.artifacts.insert(id, artifact.clone());
        Ok(id)
    }
}

impl crate::Store for MemoryStore {}
