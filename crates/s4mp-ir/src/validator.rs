use crate::IrModule;
use s4mp_core::{Result, S4mpError};

/// Validates USIR invariants before materialization into the graph.
pub struct IrValidator;

impl IrValidator {
    pub fn validate(module: &IrModule) -> Result<()> {
        if module.name.is_empty() {
            return Err(S4mpError::Other("USIR module name must not be empty".into()));
        }
        Ok(())
    }
}
