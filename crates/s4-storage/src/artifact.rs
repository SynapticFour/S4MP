use s4_core::{ArtifactId, Result, S4Error, SchemaVersion};
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
    /// Canonical JSON bytes hashed to produce [`Artifact::id`].
    ///
    /// The store persists these exact bytes (compact JSON, not pretty-printed).
    ///
    /// # Errors
    ///
    /// Returns [`S4Error::Storage`] if the envelope cannot be serialized.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| S4Error::Storage(format!("failed to serialize artifact envelope: {e}")))
    }

    /// Derive the content-addressed identifier for this artifact.
    ///
    /// # Errors
    ///
    /// Returns [`S4Error::Storage`] if the envelope cannot be serialized.
    pub fn id(&self) -> Result<ArtifactId> {
        Ok(ArtifactId::from_content(&self.canonical_bytes()?))
    }

    /// Reject envelopes whose schema is not [`SchemaVersion::CURRENT`].
    ///
    /// # Errors
    ///
    /// Returns [`S4Error::SchemaVersionMismatch`] when versions differ.
    pub fn expect_current_schema(&self) -> Result<()> {
        if self.schema_version == SchemaVersion::CURRENT {
            Ok(())
        } else {
            Err(S4Error::SchemaVersionMismatch {
                expected: SchemaVersion::CURRENT.to_string(),
                actual: self.schema_version.to_string(),
            })
        }
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
    /// Correspondence map cache pointer (deterministic id, not a content hash of the payload).
    UsirCache,
    /// Plugin-defined extension kind.
    Extension(String),
}
