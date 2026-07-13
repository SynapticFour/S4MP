use crate::Metric;
use s4_core::Result;
use s4_graph::GraphView;

/// Complexity measurement for a graph region.
#[derive(Clone, Debug)]
pub struct ComplexityMeasure {
    /// Emitted metrics.
    pub metrics: Vec<Metric>,
}

/// Analyzer that computes complexity metrics over a graph view.
pub trait ComplexityAnalyzer: Send + Sync {
    /// Analyze the given graph view and return complexity metrics.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    fn analyze(&self, view: &dyn GraphView) -> Result<ComplexityMeasure>;
}
