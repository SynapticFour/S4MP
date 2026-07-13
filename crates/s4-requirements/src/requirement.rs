use serde::{Deserialize, Serialize};

/// Opaque requirement identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RequirementId(pub u64);

/// Formal requirement node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Requirement {
    /// Requirement identifier.
    pub id: RequirementId,
    /// Requirement classification.
    pub kind: RequirementKind,
    /// Human-readable statement.
    pub statement: String,
}

/// Requirement classification.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementKind {
    /// Functional requirement.
    Functional,
    /// Non-functional requirement.
    NonFunctional,
    /// Safety or security constraint.
    Constraint,
    /// Test case derived from requirement.
    Test,
}
