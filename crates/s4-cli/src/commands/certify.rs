//! Certify a verification run under a policy (Phase 5).

use crate::workspace::Workspace;
use s4_certification::{
    certificate_from_evaluation, default_port_policy, evaluate_policy, CertificateId,
    CertificateStatus,
};
use s4_core::{ArtifactId, Result, S4Error, MATURITY};
use s4_verification::VerificationRun;
use std::path::PathBuf;

/// Evaluate policy over a saved verification run.
pub fn run(policy_name: &str, java: &str, rust: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    let run_path = ws
        .root()
        .join(".s4")
        .join("verification")
        .join(format!("{java}__{rust}.json"));
    if !run_path.is_file() {
        return Err(S4Error::InvalidInput(format!(
            "verification run not found at {} — run `s4 verify --java {java} --rust {rust}` first",
            run_path.display()
        )));
    }
    let bytes = std::fs::read(&run_path)
        .map_err(|e| S4Error::Storage(format!("read {}: {e}", run_path.display())))?;
    let verification: VerificationRun = serde_json::from_slice(&bytes)
        .map_err(|e| S4Error::Storage(format!("parse verification run: {e}")))?;

    let policy = if policy_name == "default" {
        default_port_policy()
    } else {
        return Err(S4Error::InvalidInput(format!(
            "unknown policy '{policy_name}' (only 'default' is shipped in Phase 5)"
        )));
    };

    let evaluation = evaluate_policy(&policy, &verification);
    let issued_at = s4_core::utc_rfc3339();
    let verification_artifact = ArtifactId::from_content(&bytes);
    let mut id_bytes = [0_u8; 8];
    id_bytes.copy_from_slice(&verification_artifact.as_bytes()[..8]);
    let certificate = certificate_from_evaluation(
        CertificateId(u64::from_be_bytes(id_bytes)),
        &policy,
        &evaluation,
        verification_artifact,
        issued_at,
    );

    let out_dir = ws.root().join(".s4").join("certificates");
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| S4Error::Storage(format!("failed to create {}: {e}", out_dir.display())))?;
    let out: PathBuf = out_dir.join(format!("{java}__{rust}__{policy_name}.json"));
    let payload = serde_json::json!({
        "maturity": MATURITY,
        "honesty": "Certificate covers verification-run policy only — not semantic Java↔Rust equivalence.",
        "evaluation": evaluation,
        "certificate": certificate,
        "verification_run_path": run_path.display().to_string(),
    });
    let out_bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| S4Error::Storage(format!("serialize certificate: {e}")))?;
    std::fs::write(&out, out_bytes)
        .map_err(|e| S4Error::Storage(format!("write {}: {e}", out.display())))?;

    println!(
        "certificate status: {:?} (policy {})",
        certificate.status, policy.name
    );
    println!("wrote {}", out.display());
    if certificate.status == CertificateStatus::Valid {
        Ok(())
    } else {
        Err(S4Error::CheckFailed(format!(
            "certification failed; failed rules: {:?}",
            evaluation.failed_rules
        )))
    }
}
