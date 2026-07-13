use crate::{InvocationContext, Plugin};
use s4_core::Result;

/// Validates invariants and issues certificates.
pub trait Verifier: Plugin {
    /// Verify inputs and emit certificate artifacts via `ctx`.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    fn verify(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
