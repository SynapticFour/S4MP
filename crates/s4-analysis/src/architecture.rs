use crate::Finding;
use s4_core::Result;
use s4_graph::GraphView;

/// Architectural boundary (module, layer, service).
#[derive(Clone, Debug)]
pub struct Boundary {
    /// Boundary name.
    pub name: String,
    /// Member node labels.
    pub members: Vec<String>,
}

/// Detected or declared architectural pattern.
#[derive(Clone, Debug)]
pub struct Pattern {
    /// Pattern name.
    pub name: String,
    /// Pattern classification.
    pub kind: PatternKind,
}

/// Pattern classification.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternKind {
    /// Positive pattern conformance.
    Conforms,
    /// Anti-pattern violation.
    Violates,
}

/// Extracts architectural structure from a graph view.
pub trait ArchitectureAnalyzer: Send + Sync {
    /// Detect boundaries in the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails.
    fn extract_boundaries(&self, view: &dyn GraphView) -> Result<Vec<Boundary>>;

    /// Detect patterns and anti-patterns.
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails.
    fn extract_patterns(&self, view: &dyn GraphView) -> Result<Vec<Pattern>>;

    /// Emit findings for architectural violations.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    fn findings(&self, view: &dyn GraphView) -> Result<Vec<Finding>>;
}
