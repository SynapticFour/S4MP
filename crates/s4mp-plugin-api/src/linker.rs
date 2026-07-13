use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// Merges per-file USIR into a unified semantic module.
pub trait Linker: Plugin {
    fn link(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
