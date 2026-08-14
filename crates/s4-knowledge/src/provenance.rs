use s4_core::{ArtifactId, SchemaVersion};
use serde::{Deserialize, Serialize};

/// Origin metadata for a knowledge fact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    /// Source category.
    pub source_type: SourceType,
    /// Source identifier (plugin, user, pipeline stage).
    pub source_id: String,
    /// Artifact that produced this fact.
    pub artifact_id: ArtifactId,
    /// RFC-3339 UTC timestamp.
    pub timestamp: String,
    /// Schema version at creation time.
    pub schema_version: SchemaVersion,
}

/// Category of fact producer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Repository import.
    Import,
    /// Language parser.
    Parse,
    /// IR linker.
    Link,
    /// Deterministic analyzer.
    Analysis,
    /// Human author.
    Human,
    /// LLM reasoner plugin.
    Reasoner,
    /// Verification engine.
    Verifier,
}
