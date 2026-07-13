//! LLM-agnostic reasoning interfaces. Provider implementations are plugins only.

pub mod context;
pub mod policy;
pub mod proposal;
pub mod request;

pub use context::ContextBundle;
pub use policy::ReasonPolicy;
pub use proposal::{Proposal, ProposedFact, ProposalKind};
pub use request::{ReasonIntent, ReasonRequest};
