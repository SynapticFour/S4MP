use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// Imports external sources into physical snapshot artifacts.
pub trait Importer: Plugin {
    fn import(&self, ctx: &mut InvocationContext<'_>, source_uri: &str) -> Result<()>;
}
