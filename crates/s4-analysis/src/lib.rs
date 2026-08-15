//! # s4-analysis
//!
//! Architecture extraction, feature extraction, and analysis contracts.

#![warn(missing_docs)]

/// Architecture analyzer trait and types.
pub mod architecture;
/// Cross-graph correspondence types and heuristics.
pub mod correspondence;
/// Markdown diff reports from correspondence maps.
pub mod diff_report;
/// Feature extractor trait and types.
pub mod feature;
/// Finding types.
pub mod finding;
/// USIR to graph lowering.
pub mod lowering;
/// Ordered pass pipeline (ADR-013).
pub mod pass;
/// Analysis pipeline trait.
pub mod pipeline;

pub use architecture::{ArchitectureAnalyzer, Boundary, Pattern, PatternKind};
pub use correspondence::{
    entries_matching_id, entries_matching_name, entry_name_keys, load_correspondence_map,
    merge_correspondences, save_correspondence_map, short_entry_id, suggest_correspondences,
    CorrespondenceEntry, CorrespondenceMethod, CorrespondenceStatus, GraphId, NodeRef,
    SHORT_ENTRY_ID_LEN,
};
pub use diff_report::{
    build_diff_report, confidence_bands, render_json, render_markdown, ConfidenceBands, DiffReport,
    DiffSummary,
};
pub use feature::{Feature, FeatureExtractor, FeatureId};
pub use finding::{Finding, FindingId, Severity};
pub use lowering::usir_to_graph;
pub use pass::{Pass, PassContext, PassOutcome, PassPipeline, PORTING_PASS_ORDER};
pub use pipeline::AnalysisPipeline;
