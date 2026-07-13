use crate::{InvocationContext, Plugin};
use s4_core::Result;

/// Produces derived graph artifacts and findings.
pub trait Analyzer: Plugin {
    /// Run analysis using input artifacts in `ctx`.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    fn analyze(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
