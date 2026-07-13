use s4_core::ArtifactId;
use serde::{Deserialize, Serialize};

/// Opaque certificate identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificateId(pub u64);

/// Immutable certification record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate identifier.
    pub id: CertificateId,
    /// Policy that was evaluated.
    pub policy_name: String,
    /// Pass/fail status.
    pub status: CertificateStatus,
    /// Content-addressed certificate artifact.
    pub artifact: ArtifactId,
    /// ISO-8601 issuance timestamp.
    pub issued_at: String,
    /// Optional expiry timestamp.
    pub expires_at: Option<String>,
}

/// Certificate status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateStatus {
    /// All policy rules passed.
    Valid,
    /// One or more rules failed.
    Invalid,
    /// Certificate revoked.
    Revoked,
}
