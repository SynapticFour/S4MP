//! Core identifiers, errors, and versioning for the SynapticFour Method Platform.
//!
//! This crate has no dependencies on other S4MP crates.

pub mod error;
pub mod id;
pub mod version;

pub use error::S4mpError;
pub use error::Result;
pub use id::{ArtifactId, PluginId, SnapshotId};
pub use version::{ApiVersion, SchemaVersion};
