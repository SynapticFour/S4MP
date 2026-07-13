//! Canonical schema types and extension registry for S4MP artifacts.

pub mod artifact;
pub mod extension;
pub mod manifest;

pub use artifact::{Artifact, ArtifactKind};
pub use extension::{ExtensionKindId, ExtensionRegistry};
pub use manifest::Manifest;
