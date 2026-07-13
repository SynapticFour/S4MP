//! # s4-verification
//!
//! Verification and invariant checking contracts.

#![warn(missing_docs)]

/// Invariant types.
pub mod invariant;
/// Verifier trait.
pub mod verifier;
/// Acceptance workflow trait.
pub mod workflow;

pub use invariant::{Invariant, InvariantId, InvariantSet};
pub use verifier::{VerificationResult, Verifier};
pub use workflow::AcceptanceWorkflow;
