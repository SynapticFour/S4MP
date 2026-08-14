//! # s4-verification
//!
//! Verification, invariant checking, and acceptance workflow contracts.
//!
//! The live Phase 5 path used by `s4 verify` / `s4 certify` is [`port_diff::VerificationRun`]
//! plus `s4_certification::evaluate_policy`. [`Verifier`] / [`VerificationResult`] remain
//! contracts for a future invariant engine and are not wired to the CLI.

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
