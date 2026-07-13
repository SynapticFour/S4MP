use crate::{ContextBundle, ReasonPolicy};

#[derive(Clone, Debug)]
pub struct ReasonRequest {
    pub intent: ReasonIntent,
    pub context: ContextBundle,
    pub policy: ReasonPolicy,
}

#[derive(Clone, Debug)]
pub enum ReasonIntent {
    Explain,
    RefactorPlan,
    MapRequirement,
    ArchitectureReview,
}
