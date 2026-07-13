use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// LLM-agnostic reasoning interface. Implementations live in reasoner plugins.
pub trait Reasoner: Plugin {
    fn reason(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
