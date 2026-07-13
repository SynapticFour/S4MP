use serde::{Deserialize, Serialize};

/// Constraint attached to requirements or architecture.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint name.
    pub name: String,
    /// Machine- or human-readable expression.
    pub expression: String,
}
