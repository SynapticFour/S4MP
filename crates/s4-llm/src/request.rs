use crate::{ContextBundle, ReasonPolicy};
use serde::{Deserialize, Serialize};

/// Reasoning request sent to an LLM provider.
#[derive(Clone, Debug)]
pub struct ReasonRequest {
    /// Reasoning intent.
    pub intent: ReasonIntent,
    /// Context artifacts.
    pub context: ContextBundle,
    /// Policy constraints.
    pub policy: ReasonPolicy,
}

/// Supported reasoning intents.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonIntent {
    /// Explain code or architecture.
    Explain,
    /// Generate refactoring plan.
    RefactorPlan,
    /// Map requirements to implementation.
    MapRequirement,
    /// Review architecture for issues.
    ArchitectureReview,
}
