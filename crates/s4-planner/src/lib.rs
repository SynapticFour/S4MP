//! # s4-planner
//!
//! Refactoring planning contracts. Plans are proposals — never auto-applied by core.

#![warn(missing_docs)]

/// Refactoring plan types.
pub mod plan;
/// Planner trait.
pub mod planner;
/// Planning strategy enumeration.
pub mod strategy;

pub use plan::{PlanRisk, PlanStep, PlanStepKind, RefactorPlan};
pub use planner::Planner;
pub use strategy::PlanningStrategy;
