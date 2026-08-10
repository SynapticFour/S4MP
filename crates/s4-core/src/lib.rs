//! # s4-core
//!
//! Foundation types for the `SynapticFour` Method Platform (S4MP).
//!
//! This crate is the innermost layer of the workspace. It defines identifiers,
//! errors, and versioning contracts used by every other crate. It must not
//! depend on any other `s4-*` crate.

#![warn(missing_docs)]

/// Error types and the platform `Result` alias.
pub mod error;
/// Content-addressed and domain identifiers.
pub mod id;
/// Programming language identifiers.
pub mod language;
/// Product maturity labels for honest CLI and report surfaces.
pub mod maturity;
/// Common re-exports for workspace crates.
pub mod prelude;
/// Schema and plugin API versioning.
pub mod version;

pub use error::{Result, S4Error};
pub use id::{ArtifactId, EntityId, PluginId, ProjectId};
pub use language::LanguageId;
pub use maturity::{MATURITY, MATURITY_NOTICE};
pub use version::{ApiVersion, SchemaVersion};
