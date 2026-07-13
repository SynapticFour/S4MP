use s4mp_core::ArtifactId;
use s4mp_model::{Fact, FactLifecycle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub kind: ProposalKind,
    pub claims: Vec<ProposedFact>,
    pub rationale_artifact: ArtifactId,
    pub model_metadata: Option<ModelMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    Explanation,
    RefactorPlan,
    RequirementMapping,
    ArchitectureAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposedFact {
    pub fact: Fact,
    pub lifecycle: FactLifecycle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub provider_id: String,
    pub model_id: String,
    pub prompt_hash: String,
    pub response_hash: String,
}
