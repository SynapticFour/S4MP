//! Performance benchmark placeholders for S4MP.

#[cfg(test)]
mod tests {
    use s4mp_store::{MemoryStore, StoreWriter};
    use s4mp_core::SchemaVersion;
    use s4mp_schema::{Artifact, ArtifactKind};

    #[test]
    fn store_write_smoke() {
        let mut store = MemoryStore::new();
        let artifact = Artifact {
            kind: ArtifactKind::PhysicalSnapshot,
            schema_version: SchemaVersion::CURRENT,
            payload: serde_json::json!({}),
        };
        let _ = store.write(&artifact).unwrap();
    }
}
