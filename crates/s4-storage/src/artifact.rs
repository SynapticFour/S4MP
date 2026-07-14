use s4_core::{ArtifactId, SchemaVersion};
use serde::{Deserialize, Serialize};

/// Typed artifact envelope stored in the CAS.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    /// Logical kind of the artifact payload.
    pub kind: ArtifactKind,
    /// Schema version of the payload.
    pub schema_version: SchemaVersion,
    /// Opaque, schema-versioned payload.
    pub payload: serde_json::Value,
}

impl Artifact {
    /// Derive the content-addressed identifier for this artifact.
    #[must_use]
    pub fn id(&self) -> ArtifactId {
        ArtifactId::from_content(&serde_json::to_vec(self).unwrap_or_default())
    }
}

/// Known artifact kinds. Extensions use namespaced string variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// Physical repository snapshot.
    PhysicalSnapshot,
    /// Syntax tree for a compilation unit.
    SyntaxTree,
    /// Universal Semantic IR module.
    UsirModule,
    /// Materialized graph projection.
    GraphProjection,
    /// Analysis finding or metric bundle.
    AnalysisResult,
    /// LLM-generated proposal (always proposed lifecycle).
    ReasonProposal,
    /// Verification certificate.
    Certificate,
    /// Cross-graph node correspondence map (Java↔Rust porting, etc.).
    CorrespondenceMap,
    /// Plugin-defined extension kind.
    Extension(String),
}
