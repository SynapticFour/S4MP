/// Strategy controlling plan generation behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlanningStrategy {
    /// Minimize number of steps.
    MinimalSteps,
    /// Minimize risk even if more steps are required.
    MinimalRisk,
    /// Maximize test coverage additions.
    TestFirst,
}
