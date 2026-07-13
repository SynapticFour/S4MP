use s4_core::Result;
use s4_knowledge::{Fact, FactLifecycle};

/// Workflow for accepting or rejecting proposed facts.
pub trait AcceptanceWorkflow: Send + Sync {
    /// Accept a proposed fact, transitioning lifecycle to `Accepted`.
    ///
    /// # Errors
    ///
    /// Returns an error if acceptance fails.
    fn accept(&mut self, fact: &Fact) -> Result<Fact>;

    /// Reject a proposed fact.
    ///
    /// # Errors
    ///
    /// Returns an error if rejection fails.
    fn reject(&mut self, fact: &Fact, reason: &str) -> Result<Fact>;

    /// Returns the target lifecycle for this workflow step.
    fn target_lifecycle(&self) -> FactLifecycle;
}
