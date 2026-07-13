//! # s4-parser
//!
//! Universal parsing orchestration and USIR contracts.

#![warn(missing_docs)]

/// Language identifier types.
pub mod language;
/// Parse pipeline trait.
pub mod pipeline;
/// Parse unit types.
pub mod unit;
/// Universal Semantic IR (USIR) types.
pub mod usir;

pub use language::LanguageId;
pub use pipeline::ParsePipeline;
pub use unit::ParseUnit;
pub use usir::{UsirEntity, UsirEntityKind, UsirModule, UsirRelation, UsirRelationKind};
