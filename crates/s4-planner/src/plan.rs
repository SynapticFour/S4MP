use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// Ordered refactoring plan (proposal artifact).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RefactorPlan {
    /// Plan title.
    pub title: String,
    /// Ordered steps.
    pub steps: Vec<PlanStep>,
    /// Overall risk assessment.
    pub risk: PlanRisk,
    /// Source findings artifact.
    pub source_findings: Option<ArtifactId>,
}

/// Single step in a refactoring plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step description.
    pub description: String,
    /// Step kind.
    pub kind: PlanStepKind,
    /// Target location or symbol reference.
    pub target: String,
}

/// Refactoring step classification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepKind {
    /// Extract into new module or function.
    Extract,
    /// Move symbol or file.
    Move,
    /// Rename symbol.
    Rename,
    /// Inline symbol.
    Inline,
    /// Add or update test coverage.
    AddTest,
    /// Manual review required.
    ManualReview,
}

/// Plan risk level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanRisk {
    /// Low risk — isolated change.
    Low,
    /// Medium risk — cross-module impact.
    Medium,
    /// High risk — architectural change.
    High,
}
