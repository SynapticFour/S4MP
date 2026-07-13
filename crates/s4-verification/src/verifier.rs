use crate::InvariantSet;
use s4_core::{ArtifactId, Result};
use s4_knowledge::Fact;
use serde::{Deserialize, Serialize};

/// Result of a verification run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether all invariants passed.
    pub passed: bool,
    /// Human-readable summary.
    pub summary: String,
    /// Violated fact references.
    pub violations: Vec<Fact>,
    /// Result artifact when persisted.
    pub artifact: Option<ArtifactId>,
}

/// Verifies facts and graph state against invariants.
pub trait Verifier: Send + Sync {
    /// Run verification against the given invariant set.
    ///
    /// # Errors
    ///
    /// Returns an error if verification cannot complete.
    fn verify(&self, invariants: &InvariantSet, facts: &[Fact]) -> Result<VerificationResult>;
}
