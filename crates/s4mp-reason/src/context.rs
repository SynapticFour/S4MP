use s4mp_core::ArtifactId;

/// Bundle of artifact references passed to reasoners as context.
#[derive(Clone, Debug, Default)]
pub struct ContextBundle {
    pub artifact_ids: Vec<ArtifactId>,
}
