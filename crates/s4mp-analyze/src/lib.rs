//! Analyzer framework: complexity, architecture, feature extraction.

pub mod context;
pub mod finding;
pub mod pipeline;

pub use context::AnalyzerContext;
pub use finding::Finding;
pub use pipeline::AnalysisPipeline;
