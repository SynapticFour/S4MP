use crate::{InvocationContext, Plugin};
use s4_core::Result;

/// Parses source files into syntax and partial USIR artifacts.
pub trait Parser: Plugin {
    /// Parse `path` and emit artifacts via `ctx`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    fn parse(&self, ctx: &mut InvocationContext<'_>, path: &str) -> Result<()>;
}
