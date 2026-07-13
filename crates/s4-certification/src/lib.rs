//! # s4-certification
//!
//! Certification and compliance contracts with immutable audit trails.

#![warn(missing_docs)]

/// Certificate types.
pub mod certificate;
/// Certificate issuer trait.
pub mod issuer;
/// Certification policy types.
pub mod policy;

pub use certificate::{Certificate, CertificateId, CertificateStatus};
pub use issuer::CertificateIssuer;
pub use policy::{CertificationPolicy, PolicyRule};
