use s4mp_core::{ArtifactId, SchemaVersion};
use serde::{Deserialize, Serialize};

/// Typed artifact envelope stored in the content-addressed store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub schema_version: SchemaVersion,
    pub payload: serde_json::Value,
}

/// Known artifact kinds. Extension kinds use namespaced string IDs.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    PhysicalSnapshot,
    SyntaxTree,
    UsirModule,
    GraphProjection,
    AnalysisFinding,
    ReasonProposal,
    Certificate,
    Extension(String),
}

impl Artifact {
    pub fn id(&self) -> ArtifactId {
        ArtifactId::from_content(&serde_json::to_vec(self).unwrap_or_default())
    }
}
