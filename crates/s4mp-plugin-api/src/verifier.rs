use crate::{InvocationContext, Plugin};
use s4mp_core::Result;

/// Validates invariants and issues certificates.
pub trait Verifier: Plugin {
    fn verify(&self, ctx: &mut InvocationContext<'_>) -> Result<()>;
}
