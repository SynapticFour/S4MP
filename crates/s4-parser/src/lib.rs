//! # s4-parser
//!
//! Universal parsing orchestration and USIR contracts.

#![warn(missing_docs)]

/// Language identifier types.
pub mod language;
/// Parse pipeline trait.
pub mod pipeline;
/// Tree-sitter language frontends (v1).
pub mod plugins;
/// Parse unit types.
pub mod unit;
/// Universal Semantic IR (USIR) types.
pub mod usir;

pub use language::LanguageId;
pub use pipeline::{ParseContext, ParsePipeline};
pub use plugins::{extract_java_module, extract_rust_module, JavaParser, RustParser};
pub use unit::ParseUnit;
pub use usir::{
    UnresolvedCall, UsirEntity, UsirEntityKind, UsirLocalId, UsirModule, UsirRelation,
    UsirRelationKind,
};
