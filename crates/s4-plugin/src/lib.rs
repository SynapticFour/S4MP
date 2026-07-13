//! # s4-plugin
//!
//! Plugin system contracts. All volatile implementations attach here.

#![warn(missing_docs)]

/// Analyzer plugin trait.
pub mod analyzer;
/// Plugin host trait.
pub mod host;
/// Importer plugin trait.
pub mod importer;
/// Plugin manifest types.
pub mod manifest;
/// Parser plugin trait.
pub mod parser;
/// Base plugin traits and invocation context.
pub mod plugin;
/// Reasoner plugin trait.
pub mod reasoner;
/// Verifier plugin trait.
pub mod verifier;

pub use analyzer::Analyzer;
pub use host::PluginHost;
pub use importer::Importer;
pub use manifest::{CapabilitySet, PluginManifest};
pub use parser::Parser;
pub use plugin::{InvocationContext, Plugin, PluginOutput};
pub use reasoner::Reasoner;
pub use verifier::Verifier;
