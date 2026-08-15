use crate::store::{Store, StoreReader, StoreWriter};
use crate::Artifact;
use s4_core::{ArtifactId, Result, S4Error};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// File-backed artifact store: one JSON file per artifact under `<root>/<hex-id>.json`.
///
/// Identifiers are Blake3 of **compact** canonical JSON (the bytes written to disk).
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
            S4Error::Storage(format!(
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

    fn commit_bytes(&self, id: &ArtifactId, bytes: &[u8]) -> Result<()> {
        let path = self.artifact_path(id);
        if path.is_file() {
            return Ok(());
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, bytes).map_err(|e| {
            S4Error::Storage(format!("failed to write artifact {}: {e}", tmp.display()))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            S4Error::Storage(format!("failed to commit artifact {}: {e}", path.display()))
        })?;
        Ok(())
    }

    fn write_pointer(&self, index_id: &ArtifactId, content_id: &ArtifactId) -> Result<()> {
        if index_id == content_id {
            return Ok(());
        }
        let pointer = serde_json::json!({
            "s4_cas_pointer": 1,
            "content_id": content_id.to_string(),
        });
        let bytes = serde_json::to_vec(&pointer)
            .map_err(|e| S4Error::Storage(format!("failed to serialize CAS pointer: {e}")))?;
        self.commit_bytes(index_id, &bytes)
    }

    fn parse_pointer(bytes: &[u8]) -> Option<ArtifactId> {
        let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        if value.get("s4_cas_pointer")?.as_u64()? != 1 {
            return None;
        }
        value.get("content_id")?.as_str()?.parse().ok()
    }
}

impl StoreReader for FileSystemStore {
    fn read(&self, id: &ArtifactId) -> Result<Option<Artifact>> {
        let path = self.artifact_path(id);
        match fs::read(&path) {
            Ok(bytes) => {
                if let Some(content_id) = Self::parse_pointer(&bytes) {
                    return self.read(&content_id);
                }
                let artifact: Artifact = serde_json::from_slice(&bytes).map_err(|e| {
                    S4Error::Storage(format!(
                        "failed to deserialize artifact {}: {e}",
                        path.display()
                    ))
                })?;
                Ok(Some(artifact))
            },
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(S4Error::Storage(format!(
                "failed to read artifact {}: {e}",
                path.display()
            ))),
        }
    }

    fn contains(&self, id: &ArtifactId) -> Result<bool> {
        match fs::metadata(self.artifact_path(id)) {
            Ok(meta) => Ok(meta.is_file()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
            Err(e) => Err(S4Error::Storage(format!(
                "failed to stat artifact {id}: {e}"
            ))),
        }
    }
}

impl StoreWriter for FileSystemStore {
    fn write(&mut self, artifact: &Artifact) -> Result<ArtifactId> {
        let bytes = artifact.canonical_bytes()?;
        let id = ArtifactId::from_content(&bytes);
        self.commit_bytes(&id, &bytes)?;
        Ok(id)
    }

    fn write_at(&mut self, id: ArtifactId, artifact: &Artifact) -> Result<()> {
        let content_id = self.write(artifact)?;
        self.write_pointer(&id, &content_id)
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
        assert!(store.contains(&id).expect("contains"));
        let loaded = store.read(&id).expect("read").expect("artifact");
        assert_eq!(loaded.kind, ArtifactKind::PhysicalSnapshot);
        let on_disk = std::fs::read(store.artifact_path(&id)).expect("file");
        assert_eq!(on_disk, artifact.canonical_bytes().expect("bytes"));
        assert_eq!(id, artifact.id().expect("id"));
    }

    #[test]
    fn write_at_stores_pointer_not_raw_envelope() {
        let mut store = temp_store();
        let artifact = Artifact {
            kind: ArtifactKind::UsirCache,
            schema_version: SchemaVersion::CURRENT,
            payload: serde_json::json!({"usir_id": "abc"}),
        };
        let index = ArtifactId::from_content(b"index-key");
        store.write_at(index, &artifact).expect("write_at");
        let loaded = store.read(&index).expect("read").expect("artifact");
        assert_eq!(loaded.kind, ArtifactKind::UsirCache);
        let on_disk = std::fs::read(store.artifact_path(&index)).expect("file");
        assert!(
            String::from_utf8_lossy(&on_disk).contains("s4_cas_pointer"),
            "index path must be a pointer, got {}",
            String::from_utf8_lossy(&on_disk)
        );
        let content_id = artifact.id().expect("id");
        assert!(store.contains(&content_id).expect("contains content"));
    }
}
