//! # s4-llm
//!
//! LLM-agnostic reasoning contracts. Providers are plugins — never core dependencies.

#![warn(missing_docs)]

/// Context bundle types.
pub mod context;
/// Reasoning policy types.
pub mod policy;
/// Proposal and model metadata types.
pub mod proposal;
/// LLM provider trait.
pub mod provider;
/// Reasoning request types.
pub mod request;

pub use context::ContextBundle;
pub use policy::ReasonPolicy;
pub use proposal::{ModelMetadata, Proposal, ProposalKind, ProposedClaim};
pub use provider::LlmProvider;
pub use request::{ReasonIntent, ReasonRequest};
