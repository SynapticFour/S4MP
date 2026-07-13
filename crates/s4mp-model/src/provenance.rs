use s4mp_core::{ArtifactId, SchemaVersion};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub source_type: SourceType,
    pub source_id: String,
    pub artifact_id: ArtifactId,
    pub timestamp: String,
    pub schema_version: SchemaVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Import,
    Parse,
    Link,
    Analysis,
    Human,
    Reasoner,
    Verifier,
}
