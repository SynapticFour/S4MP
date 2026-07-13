use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// Parses source files into syntax and partial USIR artifacts.
pub trait Parser: Plugin {
    fn parse(&self, ctx: &mut InvocationContext<'_>, path: &str) -> Result<()>;
}
