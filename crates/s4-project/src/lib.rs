//! # s4-project
//!
//! Project workspace contracts: configuration, lockfiles, and snapshot references.

#![warn(missing_docs)]

/// Project configuration types.
pub mod config;
/// Source ingestion and physical snapshotting.
pub mod ingest;
/// Plugin lockfile types.
pub mod lockfile;
/// Snapshot reference types.
pub mod snapshot;
/// Source registration types.
pub mod source;
/// Workspace trait.
pub mod workspace;

pub use config::ProjectConfig;
pub use ingest::{snapshot_physical, DefaultSourceIngestor, ResolvedSource, SourceIngestor};
pub use lockfile::Lockfile;
pub use snapshot::SnapshotRef;
pub use source::{SourceOrigin, SourceRef};
pub use workspace::Workspace;
