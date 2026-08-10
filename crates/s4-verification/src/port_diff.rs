//! Port-diff verification run (Phase 5 thin slice).
//!
//! This verifies **artifact completeness / coverage thresholds**, not semantic
//! Java↔Rust equivalence. Certificates based on this run must keep that distinction.

use serde::{Deserialize, Serialize};

/// Persisted verification run over porting + requirements artifacts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationRun {
    /// Java source alias.
    pub java_source: String,
    /// Rust source alias.
    pub rust_source: String,
    /// Platform maturity label at run time.
    pub maturity: String,
    /// Ported correspondence count.
    pub ported: usize,
    /// Diverged (heuristic) count.
    pub diverged: usize,
    /// Missing-in-target count.
    pub missing: usize,
    /// Extra-in-target count.
    pub extra: usize,
    /// Java callable count.
    pub java_callables: usize,
    /// `ported / java_callables * 100`.
    pub coverage_pct: f32,
    /// Requirements present in workspace.
    pub requirements_total: usize,
    /// Requirements with at least one `ImplementedBy` trace.
    pub requirements_traced: usize,
    /// Explicit gaps for humans / policies.
    pub gaps: Vec<String>,
    /// Overall pass for the configured thresholds in this run.
    pub passed: bool,
    /// Human summary.
    pub summary: String,
}

/// Thresholds for a Phase 5 verification run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationThresholds {
    /// Minimum ported coverage percentage (0–100).
    pub min_coverage_pct: f32,
    /// When true, every requirement must have ≥1 `ImplementedBy` trace.
    pub require_all_requirements_traced: bool,
    /// When true, fail if any `MissingInTarget` rows remain.
    pub forbid_missing_in_target: bool,
}

impl Default for VerificationThresholds {
    fn default() -> Self {
        Self {
            min_coverage_pct: 0.0,
            require_all_requirements_traced: false,
            forbid_missing_in_target: false,
        }
    }
}

/// Inputs for [`build_verification_run`].
#[derive(Clone, Debug)]
pub struct VerificationInputs<'a> {
    /// Java source alias.
    pub java_source: &'a str,
    /// Rust source alias.
    pub rust_source: &'a str,
    /// Maturity label.
    pub maturity: &'a str,
    /// Ported count.
    pub ported: usize,
    /// Diverged count.
    pub diverged: usize,
    /// Missing count.
    pub missing: usize,
    /// Extra count.
    pub extra: usize,
    /// Java callable count.
    pub java_callables: usize,
    /// Coverage percent.
    pub coverage_pct: f32,
    /// Requirements total.
    pub requirements_total: usize,
    /// Requirements traced.
    pub requirements_traced: usize,
}

/// Build a [`VerificationRun`] from porting/requirements counters.
#[must_use]
pub fn build_verification_run(
    inputs: &VerificationInputs<'_>,
    thresholds: &VerificationThresholds,
) -> VerificationRun {
    let mut gaps = Vec::new();
    if inputs.coverage_pct < thresholds.min_coverage_pct {
        gaps.push(format!(
            "coverage {:.1}% below minimum {:.1}%",
            inputs.coverage_pct, thresholds.min_coverage_pct
        ));
    }
    if thresholds.forbid_missing_in_target && inputs.missing > 0 {
        gaps.push(format!(
            "{} Java node(s) missing in Rust target",
            inputs.missing
        ));
    }
    if thresholds.require_all_requirements_traced
        && inputs.requirements_total > inputs.requirements_traced
    {
        gaps.push(format!(
            "{}/{} requirements lack ImplementedBy traces",
            inputs.requirements_total - inputs.requirements_traced,
            inputs.requirements_total
        ));
    }
    let passed = gaps.is_empty();
    let summary = if passed {
        format!(
            "verification passed for {}→{} (coverage {:.1}%, maturity {})",
            inputs.java_source, inputs.rust_source, inputs.coverage_pct, inputs.maturity
        )
    } else {
        format!(
            "verification failed for {}→{}: {}",
            inputs.java_source,
            inputs.rust_source,
            gaps.join("; ")
        )
    };
    VerificationRun {
        java_source: inputs.java_source.into(),
        rust_source: inputs.rust_source.into(),
        maturity: inputs.maturity.into(),
        ported: inputs.ported,
        diverged: inputs.diverged,
        missing: inputs.missing,
        extra: inputs.extra,
        java_callables: inputs.java_callables,
        coverage_pct: inputs.coverage_pct,
        requirements_total: inputs.requirements_total,
        requirements_traced: inputs.requirements_traced,
        gaps,
        passed,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fails_on_coverage_threshold() {
        let run = build_verification_run(
            &VerificationInputs {
                java_source: "j",
                rust_source: "r",
                maturity: "heuristic-map-v2",
                ported: 0,
                diverged: 2,
                missing: 1,
                extra: 0,
                java_callables: 4,
                coverage_pct: 0.0,
                requirements_total: 0,
                requirements_traced: 0,
            },
            &VerificationThresholds {
                min_coverage_pct: 50.0,
                ..Default::default()
            },
        );
        assert!(!run.passed);
        assert!(!run.gaps.is_empty());
    }
}
