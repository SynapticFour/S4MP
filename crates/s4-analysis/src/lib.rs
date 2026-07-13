//! # s4-analysis
//!
//! Architecture extraction, feature extraction, and analysis contracts.

#![warn(missing_docs)]

/// Architecture analyzer trait and types.
pub mod architecture;
/// Feature extractor trait and types.
pub mod feature;
/// Finding types.
pub mod finding;
/// Analysis pipeline trait.
pub mod pipeline;

pub use architecture::{ArchitectureAnalyzer, Boundary, Pattern, PatternKind};
pub use feature::{Feature, FeatureExtractor, FeatureId};
pub use finding::{Finding, FindingId, Severity};
pub use pipeline::AnalysisPipeline;
