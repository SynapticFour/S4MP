use crate::store::{Store, StoreReader, StoreWriter};
use crate::Artifact;
use s4_core::{ArtifactId, Result, S4Error};
use std::fs;
use std::path::{Path, PathBuf};

/// File-backed artifact store: one JSON file per artifact under `<root>/<hex-id>.json`.
#[derive(Clone, Debug)]
pub struct FileSystemStore {
    root: PathBuf,
}

impl FileSystemStore {
    /// Open or create a store rooted at `root`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| {
            S4Error::Other(format!(
                "failed to create artifact store directory {}: {e}",
                root.display()
            ))
        })?;
        Ok(Self { root })
    }

    /// Default workspace store at `.s4/store` relative to `workspace_root`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn workspace(workspace_root: impl AsRef<Path>) -> Result<Self> {
        Self::new(workspace_root.as_ref().join(".s4").join("store"))
    }

    fn artifact_path(&self, id: &ArtifactId) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }
}

impl StoreReader for FileSystemStore {
    fn read(&self, id: &ArtifactId) -> Result<Option<Artifact>> {
        let path = self.artifact_path(id);
        if !path.is_file() {
            return Ok(None);
        }
        let bytes = fs::read(&path).map_err(|e| {
            S4Error::Other(format!("failed to read artifact {}: {e}", path.display()))
        })?;
        let artifact: Artifact = serde_json::from_slice(&bytes).map_err(|e| {
            S4Error::Other(format!(
                "failed to deserialize artifact {}: {e}",
                path.display()
            ))
        })?;
        Ok(Some(artifact))
    }

    fn contains(&self, id: &ArtifactId) -> bool {
        self.artifact_path(id).is_file()
    }
}

impl StoreWriter for FileSystemStore {
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId> {
        let id = artifact.id();
        let path = self.artifact_path(&id);
        if path.is_file() {
            return Ok(id);
        }
        let bytes = serde_json::to_vec_pretty(artifact).map_err(|e| {
            S4Error::Other(format!("failed to serialize artifact for store: {e}"))
        })?;
        fs::write(&path, bytes).map_err(|e| {
            S4Error::Other(format!("failed to write artifact {}: {e}", path.display()))
        })?;
        Ok(id)
    }
}

impl Store for FileSystemStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactKind;
    use s4_core::SchemaVersion;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_store() -> FileSystemStore {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("s4-store-test-{n}"));
        let _ = std::fs::remove_dir_all(&root);
        FileSystemStore::new(&root).expect("store")
    }

    #[test]
    fn write_and_read_round_trip() {
        let mut store = temp_store();
        let artifact = Artifact {
            kind: ArtifactKind::PhysicalSnapshot,
            schema_version: SchemaVersion::CURRENT,
            payload: serde_json::json!({"files": []}),
        };
        let id = store.write(&artifact).expect("write");
        assert!(store.contains(&id));
        let loaded = store.read(&id).expect("read").expect("artifact");
        assert_eq!(loaded.kind, ArtifactKind::PhysicalSnapshot);
    }
}
