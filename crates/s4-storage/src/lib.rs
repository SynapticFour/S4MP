//! # s4-storage
//!
//! Content-addressed storage contracts for immutable S4MP artifacts.
//!
//! All cross-boundary data exchange flows through artifact IDs. This crate
//! defines the storage traits; filesystem and remote backends are provided
//! by future implementation crates or plugins.

#![warn(missing_docs)]

/// Typed artifact envelopes and kinds.
pub mod artifact;
/// Snapshot manifest types.
pub mod manifest;
/// Storage reader/writer traits.
pub mod store;
/// JSON file-backed CAS store.
pub mod filesystem;

pub use artifact::{Artifact, ArtifactKind};
pub use filesystem::FileSystemStore;
pub use manifest::{Manifest, ManifestRef};
pub use store::{Store, StoreReader, StoreWriter};
