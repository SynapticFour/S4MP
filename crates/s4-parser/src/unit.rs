use crate::LanguageId;
use s4_core::ArtifactId;

/// Single parseable source unit.
#[derive(Clone, Debug)]
pub struct ParseUnit {
    /// Filesystem or logical path.
    pub path: String,
    /// Detected or declared language.
    pub language: LanguageId,
    /// Optional pre-loaded content artifact (future: blob/text artifact in CAS).
    pub content: Option<ArtifactId>,
    /// Optional inline source text. When set, takes precedence over reading `path`.
    pub source_text: Option<String>,
    /// Blake3 hex of the file contents from the physical snapshot, when known.
    /// Used to reuse persisted USIR artifacts across graph rebuilds.
    pub source_hash: Option<String>,
}
