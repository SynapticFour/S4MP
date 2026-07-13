use crate::Finding;
use s4_core::{ArtifactId, Result};

/// Orchestrates multiple analyzers over graph/knowledge artifacts.
pub trait AnalysisPipeline: Send + Sync {
    /// Run all configured analyzers and return finding artifact IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if the pipeline fails.
    fn run(&self, graph_artifact: ArtifactId) -> Result<(Vec<Finding>, Vec<ArtifactId>)>;
}
