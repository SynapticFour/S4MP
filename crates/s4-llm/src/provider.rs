use crate::{Proposal, ReasonRequest};
use async_trait::async_trait;
use s4_core::Result;

/// Interchangeable LLM provider interface. Implementations live in plugins.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identifier for provenance and registry lookup.
    fn provider_id(&self) -> &str;

    /// Execute a reasoning request and return a proposal artifact.
    async fn reason(&self, request: ReasonRequest) -> Result<Proposal>;
}
