//! # s4-project
//!
//! Project workspace contracts: configuration, lockfiles, and snapshot references.

#![warn(missing_docs)]

/// Project configuration types.
pub mod config;
/// Plugin lockfile types.
pub mod lockfile;
/// Snapshot reference types.
pub mod snapshot;
/// Workspace trait.
pub mod workspace;

pub use config::ProjectConfig;
pub use lockfile::Lockfile;
pub use snapshot::SnapshotRef;
pub use workspace::Workspace;
