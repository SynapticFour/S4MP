//! # s4-llm
//!
//! LLM-agnostic reasoning contracts. Providers are plugins — never core dependencies.

#![warn(missing_docs)]

/// Context bundle types.
pub mod context;
/// Offline heuristic provider (Phase 6).
pub mod heuristic;
/// Reasoning policy types.
pub mod policy;
/// Proposal and model metadata types.
pub mod proposal;
/// LLM provider trait.
pub mod provider;
/// Reasoning request types.
pub mod request;

pub use context::ContextBundle;
pub use heuristic::HeuristicLlmProvider;
pub use policy::ReasonPolicy;
pub use proposal::{ModelMetadata, Proposal, ProposalKind, ProposalLifecycle, ProposedClaim};
pub use provider::LlmProvider;
pub use request::{ReasonIntent, ReasonRequest};
