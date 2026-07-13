use serde::{Deserialize, Serialize};

/// Certification policy composed of rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationPolicy {
    /// Policy name.
    pub name: String,
    /// Policy version string.
    pub version: String,
    /// Rules to evaluate.
    pub rules: Vec<PolicyRule>,
}

/// Single policy rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier.
    pub id: String,
    /// Rule description.
    pub description: String,
    /// Reference to invariant or verifier rule set.
    pub rule_ref: String,
}
