use crate::{Certificate, CertificationPolicy};
use s4_core::Result;
use s4_verification::VerificationResult;

/// Issues certificates from verification results.
pub trait CertificateIssuer: Send + Sync {
    /// Issue a certificate when verification passes policy rules.
    ///
    /// # Errors
    ///
    /// Returns an error if issuance fails.
    fn issue(
        &self,
        policy: &CertificationPolicy,
        verification: &VerificationResult,
    ) -> Result<Certificate>;

    /// Revoke an existing certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if revocation fails.
    fn revoke(&self, certificate: &Certificate, reason: &str) -> Result<Certificate>;
}
