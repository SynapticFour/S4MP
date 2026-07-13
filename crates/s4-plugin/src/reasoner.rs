use crate::{InvocationContext, Plugin};
use s4_core::Result;

/// LLM-agnostic reasoning interface. Provider implementations are plugins.
pub trait Reasoner: Plugin {
    /// Produce proposal artifacts via `ctx`. Outputs are always `proposed` lifecycle.
    ///
    /// # Errors
    ///
    /// Returns an error if reasoning fails.
    fn reason(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
