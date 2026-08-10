//! Evaluate certification policies over verification runs (Phase 5).

use crate::certificate::{Certificate, CertificateId, CertificateStatus};
use crate::policy::CertificationPolicy;
use s4_core::ArtifactId;
use s4_verification::VerificationRun;
use serde::{Deserialize, Serialize};

/// Outcome of evaluating a policy against a verification run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    /// Policy name.
    pub policy: String,
    /// Resulting certificate status.
    pub status: CertificateStatus,
    /// Failed rule ids.
    pub failed_rules: Vec<String>,
    /// Notes for operators.
    pub notes: Vec<String>,
}

/// Evaluate a Phase 5 policy against a verification run.
///
/// Supported `rule_ref` values:
/// - `verification_passed` — require `run.passed`
/// - `min_coverage:<pct>` — require coverage ≥ pct
/// - `no_missing` — require `run.missing == 0`
/// - `requirements_traced` — require all requirements traced when any exist
#[must_use]
pub fn evaluate_policy(policy: &CertificationPolicy, run: &VerificationRun) -> PolicyEvaluation {
    let mut failed = Vec::new();
    let mut notes = Vec::new();
    notes.push(format!("maturity={}", run.maturity));
    notes
        .push("certification covers verification artifacts only — not semantic equivalence".into());

    for rule in &policy.rules {
        let ok = match rule.rule_ref.as_str() {
            "verification_passed" => run.passed,
            "no_missing" => run.missing == 0,
            "requirements_traced" => {
                run.requirements_total == 0 || run.requirements_traced == run.requirements_total
            },
            other if other.starts_with("min_coverage:") => other
                .strip_prefix("min_coverage:")
                .and_then(|s| s.parse::<f32>().ok())
                .is_some_and(|min| run.coverage_pct + f32::EPSILON >= min),
            other => {
                notes.push(format!("unknown rule_ref '{other}' treated as fail"));
                false
            },
        };
        if !ok {
            failed.push(rule.id.clone());
        }
    }

    let status = if failed.is_empty() {
        CertificateStatus::Valid
    } else {
        CertificateStatus::Invalid
    };

    PolicyEvaluation {
        policy: policy.name.clone(),
        status,
        failed_rules: failed,
        notes,
    }
}

/// Build a certificate record from an evaluation + verification artifact id.
#[must_use]
pub fn certificate_from_evaluation(
    id: CertificateId,
    policy: &CertificationPolicy,
    evaluation: &PolicyEvaluation,
    verification_artifact: ArtifactId,
    issued_at: impl Into<String>,
) -> Certificate {
    Certificate {
        id,
        policy_name: policy.name.clone(),
        status: evaluation.status,
        artifact: verification_artifact,
        issued_at: issued_at.into(),
        expires_at: None,
    }
}

/// Default Phase 5 policy: verification must pass (thresholds set at verify time).
#[must_use]
pub fn default_port_policy() -> CertificationPolicy {
    CertificationPolicy {
        name: "default".into(),
        version: "0.1.0".into(),
        rules: vec![crate::policy::PolicyRule {
            id: "vpass".into(),
            description: "Underlying verification run must pass".into(),
            rule_ref: "verification_passed".into(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_verification::{build_verification_run, VerificationInputs};

    #[test]
    fn default_policy_follows_run() {
        let run = build_verification_run(
            &VerificationInputs {
                java_source: "j",
                rust_source: "r",
                maturity: "heuristic-map-v2",
                ported: 1,
                diverged: 0,
                missing: 0,
                extra: 0,
                java_callables: 1,
                coverage_pct: 100.0,
                requirements_total: 0,
                requirements_traced: 0,
            },
            &s4_verification::VerificationThresholds::default(),
        );
        let eval = evaluate_policy(&default_port_policy(), &run);
        assert_eq!(eval.status, CertificateStatus::Valid);
    }
}
