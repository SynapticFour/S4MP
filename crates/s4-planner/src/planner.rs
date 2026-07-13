use crate::{PlanningStrategy, RefactorPlan};
use s4_analysis::Finding;
use s4_core::{ArtifactId, Result};

/// Produces refactoring plans from findings and graph context.
pub trait Planner: Send + Sync {
    /// Generate a plan from analysis findings.
    ///
    /// # Errors
    ///
    /// Returns an error if planning fails.
    fn plan_from_findings(
        &self,
        findings: &[Finding],
        strategy: PlanningStrategy,
    ) -> Result<RefactorPlan>;

    /// Persist plan as a proposal artifact.
    ///
    /// # Errors
    ///
    /// Returns an error if persistence fails.
    fn emit_plan(&self, plan: &RefactorPlan) -> Result<ArtifactId>;
}
