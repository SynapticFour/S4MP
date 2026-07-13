use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// LLM output packaged as a proposal — never ground truth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    /// Proposal classification.
    pub kind: ProposalKind,
    /// Individual proposed claims.
    pub claims: Vec<ProposedClaim>,
    /// Rationale artifact for audit.
    pub rationale: ArtifactId,
    /// Provider metadata for reproducibility.
    pub model: Option<ModelMetadata>,
}

/// Proposal classification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    /// Natural language explanation.
    Explanation,
    /// Refactoring plan proposal.
    RefactorPlan,
    /// Requirement mapping proposal.
    RequirementMapping,
    /// Architecture assessment.
    ArchitectureAssessment,
}

/// Single claim within a proposal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposedClaim {
    /// Claim statement.
    pub statement: String,
    /// Confidence assigned by the model.
    pub confidence: f32,
}

/// Provider-agnostic model invocation metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Provider identifier (e.g. `"openai-compatible"`).
    pub provider_id: String,
    /// Model identifier.
    pub model_id: String,
    /// Hash of the prompt for reproducibility.
    pub prompt_hash: String,
    /// Hash of the raw response.
    pub response_hash: String,
}
