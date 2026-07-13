use s4_core::ArtifactId;

/// Bundle of artifact references passed to LLM providers as context.
#[derive(Clone, Debug, Default)]
pub struct ContextBundle {
    /// Artifact IDs comprising the context window.
    pub artifacts: Vec<ArtifactId>,
}
