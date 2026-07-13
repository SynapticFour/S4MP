use crate::{InvocationContext, Plugin};
use s4_core::Result;

/// Imports external sources into physical snapshot artifacts.
pub trait Importer: Plugin {
    /// Import from `source_uri` and emit snapshot artifacts via `ctx`.
    fn import(&self, ctx: &mut InvocationContext<'_>, source_uri: &str) -> Result<()>;
}
