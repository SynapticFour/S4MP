//! Stable plugin API — the only surface plugins may depend on from S4MP core.

pub mod analyzer;
pub mod importer;
pub mod linker;
pub mod manifest;
pub mod parser;
pub mod plugin;
pub mod reasoner;
pub mod verifier;

pub use analyzer::Analyzer;
pub use importer::Importer;
pub use linker::Linker;
pub use manifest::{CapabilitySet, PluginManifest};
pub use parser::Parser;
pub use plugin::{InvocationContext, Plugin, PluginOutput};
pub use reasoner::Reasoner;
pub use verifier::Verifier;
