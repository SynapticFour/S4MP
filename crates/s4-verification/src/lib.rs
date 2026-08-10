//! # s4-verification
//!
//! Verification and invariant checking contracts.

#![warn(missing_docs)]

/// Invariant types.
pub mod invariant;
/// Port-diff verification runs (Phase 5).
pub mod port_diff;
/// Verifier trait.
pub mod verifier;
/// Acceptance workflow trait.
pub mod workflow;

pub use invariant::{Invariant, InvariantId, InvariantSet};
pub use port_diff::{
    build_verification_run, VerificationInputs, VerificationRun, VerificationThresholds,
};
pub use verifier::{VerificationResult, Verifier};
pub use workflow::AcceptanceWorkflow;
