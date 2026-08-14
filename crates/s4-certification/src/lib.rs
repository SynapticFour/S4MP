//! # s4-certification
//!
//! Certification and compliance contracts with immutable audit trails.
//!
//! The live Phase 5 path is [`evaluate::evaluate_policy`] / [`evaluate::certificate_from_evaluation`]
//! over [`s4_verification::VerificationRun`]. [`CertificateIssuer`] remains a contract for a
//! future issuer plugin and is not used by `s4 certify`.

#![warn(missing_docs)]

/// Certificate types.
pub mod certificate;
/// Policy evaluation over verification runs (Phase 5).
pub mod evaluate;
/// Certificate issuer trait.
pub mod issuer;
/// Certification policy types.
pub mod policy;

pub use certificate::{Certificate, CertificateId, CertificateStatus};
pub use evaluate::{
    certificate_from_evaluation, default_port_policy, evaluate_policy, PolicyEvaluation,
};
pub use issuer::CertificateIssuer;
pub use policy::{CertificationPolicy, PolicyRule};
