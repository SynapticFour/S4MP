//! # s4-storage
//!
//! Content-addressed storage contracts for immutable S4MP artifacts.
//!
//! Primary knowledge blobs (USIR, graphs, snapshots, correspondence maps) are
//! content-addressed. Workspace pointers and human reports may live as sidecars
//! under `.s4/` (see `s4-cli`).

#![warn(missing_docs)]

/// Typed artifact envelopes and kinds.
pub mod artifact;
/// JSON file-backed CAS store.
pub mod filesystem;
/// Snapshot manifest types.
pub mod manifest;
/// Storage reader/writer traits.
pub mod store;

pub use artifact::{Artifact, ArtifactKind};
pub use filesystem::FileSystemStore;
pub use manifest::{Manifest, ManifestRef};
pub use store::{Store, StoreReader, StoreWriter};
