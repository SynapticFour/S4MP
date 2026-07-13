use crate::Fact;
use s4_core::{ArtifactId, Result};
use s4_parser::UsirModule;

/// Materializes knowledge facts from USIR and graph inputs.
pub trait KnowledgeMaterializer: Send + Sync {
    /// Build facts from a USIR module artifact reference.
    ///
    /// # Errors
    ///
    /// Returns an error if fact extraction fails.
    fn extract_from_usir(&self, module: &UsirModule) -> Result<Vec<Fact>>;

    /// Persist facts and return the projection artifact ID.
    ///
    /// # Errors
    ///
    /// Returns an error if materialization fails.
    fn materialize(&self, facts: &[Fact]) -> Result<ArtifactId>;
}
