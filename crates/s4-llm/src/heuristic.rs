//! Offline heuristic LLM provider — no network, always [`ProposalLifecycle::Proposed`].

use crate::proposal::{ModelMetadata, Proposal, ProposalKind, ProposedClaim};
use crate::provider::LlmProvider;
use crate::request::{ReasonIntent, ReasonRequest};
use async_trait::async_trait;
use s4_core::{ArtifactId, Result};

/// Built-in offline reasoner used until a networked provider plugin is registered.
///
/// Emits deterministic stub claims from request intent + context size. Never marks
/// output as accepted — see [`Proposal::proposed`].
#[derive(Clone, Debug, Default)]
pub struct HeuristicLlmProvider;

impl HeuristicLlmProvider {
    /// Provider registry id.
    pub const ID: &'static str = "s4-reasoner-heuristic";

    /// Model id recorded in provenance metadata.
    pub const MODEL_ID: &'static str = "heuristic-offline-v0";

    /// Synchronous reasoning path (CLI-friendly; no Tokio runtime required).
    ///
    /// # Errors
    ///
    /// Currently infallible; returns [`Result`] for trait symmetry with networked providers.
    pub fn reason_sync(&self, request: &ReasonRequest) -> Result<Proposal> {
        let _ = request.policy.allow_network; // heuristic never calls the network
        let kind = kind_for_intent(&request.intent);
        let n = request.context.artifacts.len();
        let statement = format!(
            "heuristic {}: {} context artifact(s) — not model-backed; review before accepting",
            intent_label(&request.intent),
            n
        );
        let claims = vec![ProposedClaim {
            statement: statement.clone(),
            confidence: 0.15,
        }];
        let rationale_bytes = format!(
            "provider={}\nintent={:?}\ncontext_count={n}\n",
            Self::ID,
            request.intent
        );
        let rationale = ArtifactId::from_content(rationale_bytes.as_bytes());
        let response_body = serde_json_stub(&claims);
        let model = ModelMetadata {
            provider_id: Self::ID.to_string(),
            model_id: Self::MODEL_ID.to_string(),
            prompt_hash: hex32(rationale.as_bytes()),
            response_hash: hex32(ArtifactId::from_content(response_body.as_bytes()).as_bytes()),
        };
        Ok(Proposal::proposed(kind, claims, rationale, Some(model)))
    }
}

#[async_trait]
impl LlmProvider for HeuristicLlmProvider {
    fn provider_id(&self) -> &str {
        Self::ID
    }

    async fn reason(&self, request: ReasonRequest) -> Result<Proposal> {
        self.reason_sync(&request)
    }
}

fn kind_for_intent(intent: &ReasonIntent) -> ProposalKind {
    match intent {
        ReasonIntent::Explain => ProposalKind::Explanation,
        ReasonIntent::RefactorPlan => ProposalKind::RefactorPlan,
        ReasonIntent::MapRequirement => ProposalKind::RequirementMapping,
        ReasonIntent::ArchitectureReview => ProposalKind::ArchitectureAssessment,
    }
}

fn intent_label(intent: &ReasonIntent) -> &'static str {
    match intent {
        ReasonIntent::Explain => "explain",
        ReasonIntent::RefactorPlan => "refactor_plan",
        ReasonIntent::MapRequirement => "map_requirement",
        ReasonIntent::ArchitectureReview => "architecture_review",
    }
}

fn serde_json_stub(claims: &[ProposedClaim]) -> String {
    // Avoid adding serde_json to s4-llm; keep a stable response fingerprint string.
    claims
        .iter()
        .map(|c| format!("{}:{:.2}", c.statement, c.confidence))
        .collect::<Vec<_>>()
        .join("|")
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextBundle;
    use crate::policy::ReasonPolicy;
    use crate::proposal::ProposalLifecycle;

    #[test]
    fn sync_always_proposed() {
        let provider = HeuristicLlmProvider;
        let proposal = provider
            .reason_sync(&ReasonRequest {
                intent: ReasonIntent::Explain,
                context: ContextBundle {
                    artifacts: vec![ArtifactId::from_content(b"a")],
                },
                policy: ReasonPolicy::default(),
            })
            .unwrap();
        assert_eq!(proposal.lifecycle, ProposalLifecycle::Proposed);
        assert_eq!(proposal.kind, ProposalKind::Explanation);
        assert!(!proposal.claims.is_empty());
        assert_eq!(
            proposal.model.as_ref().map(|m| m.provider_id.as_str()),
            Some(HeuristicLlmProvider::ID)
        );
    }
}
