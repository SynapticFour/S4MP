use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// LLM output packaged as a proposal — never ground truth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    /// Always [`ProposalLifecycle::Proposed`] for LLM-originated output.
    pub lifecycle: ProposalLifecycle,
    /// Proposal classification.
    pub kind: ProposalKind,
    /// Individual proposed claims.
    pub claims: Vec<ProposedClaim>,
    /// Rationale artifact for audit.
    pub rationale: ArtifactId,
    /// Provider metadata for reproducibility.
    pub model: Option<ModelMetadata>,
}

impl Proposal {
    /// Construct a proposal that is permanently tagged as [`ProposalLifecycle::Proposed`].
    ///
    /// Callers cannot mark LLM output as accepted through this constructor.
    #[must_use]
    pub fn proposed(
        kind: ProposalKind,
        claims: Vec<ProposedClaim>,
        rationale: ArtifactId,
        model: Option<ModelMetadata>,
    ) -> Self {
        Self {
            lifecycle: ProposalLifecycle::Proposed,
            kind,
            claims,
            rationale,
            model,
        }
    }
}

/// Lifecycle of an LLM proposal. Providers may only emit [`ProposalLifecycle::Proposed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalLifecycle {
    /// Awaiting human or policy acceptance — the only state LLM providers may emit.
    Proposed,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_forces_proposed() {
        let p = Proposal::proposed(
            ProposalKind::Explanation,
            vec![],
            ArtifactId::from_content(b"rationale"),
            None,
        );
        assert_eq!(p.lifecycle, ProposalLifecycle::Proposed);
    }
}
