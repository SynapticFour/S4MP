//! # s4-requirements
//!
//! Requirements graph and traceability contracts.

#![warn(missing_docs)]

/// Constraint types.
pub mod constraint;
/// Requirement node types.
pub mod requirement;
/// Traceability graph trait.
pub mod trace;

pub use constraint::Constraint;
pub use requirement::{Requirement, RequirementId, RequirementKind};
pub use trace::{TraceLink, TraceLinkKind, TraceabilityGraph};
