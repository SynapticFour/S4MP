use crate::Fact;
use s4_core::{ArtifactId, Result};
use s4_parser::UsirModule;

/// Materializes knowledge facts from USIR and graph inputs.
pub trait KnowledgeMaterializer: Send + Sync {
    /// Build facts from a USIR module artifact reference.
    fn from_usir(&self, module: &UsirModule) -> Result<Vec<Fact>>;

    /// Persist facts and return the projection artifact ID.
    fn materialize(&self, facts: &[Fact]) -> Result<ArtifactId>;
}
