use crate::LanguageId;
use s4_core::ArtifactId;

/// Single parseable source unit.
#[derive(Clone, Debug)]
pub struct ParseUnit {
    /// Filesystem or logical path.
    pub path: String,
    /// Detected or declared language.
    pub language: LanguageId,
    /// Optional pre-loaded content artifact.
    pub content: Option<ArtifactId>,
}
