//! # s4-knowledge
//!
//! Software knowledge graph contracts: facts, provenance, and ontology.

#![warn(missing_docs)]

/// Fact types and lifecycle.
pub mod fact;
/// Knowledge materializer trait.
pub mod materializer;
/// Ontology extension registry.
pub mod ontology;
/// Provenance metadata types.
pub mod provenance;

pub use fact::{Confidence, Fact, FactKind, FactLifecycle, FactPayload};
pub use materializer::KnowledgeMaterializer;
pub use ontology::{ExtensionKindId, Ontology};
pub use provenance::{Provenance, SourceType};
