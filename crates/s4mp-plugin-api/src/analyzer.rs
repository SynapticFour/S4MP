use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// Produces derived graph artifacts and findings from IR/graph inputs.
pub trait Analyzer: Plugin {
    fn analyze(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
