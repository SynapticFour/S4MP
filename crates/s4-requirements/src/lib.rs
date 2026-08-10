//! # s4-requirements
//!
//! Requirements graph and traceability contracts.

#![warn(missing_docs)]

/// Constraint types.
pub mod constraint;
/// Requirement node types.
pub mod requirement;
/// JSON-backed requirements store (Phase 4).
pub mod store;
/// Traceability graph trait.
pub mod trace;

pub use constraint::Constraint;
pub use requirement::{Requirement, RequirementId, RequirementKind};
pub use store::RequirementsDocument;
pub use trace::{TraceLink, TraceLinkKind, TraceabilityGraph};
