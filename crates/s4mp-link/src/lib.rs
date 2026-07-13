//! Merges per-file USIR into unified semantic graph artifacts.

pub mod context;
pub mod pipeline;
pub mod resolver;

pub use context::LinkContext;
pub use pipeline::LinkPipeline;
pub use resolver::CrossLangResolver;
