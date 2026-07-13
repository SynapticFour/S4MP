use s4_core::Result;
use s4_graph::GraphView;

/// Opaque feature identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeatureId(pub u64);

/// Product or system feature mapped to code.
#[derive(Clone, Debug)]
pub struct Feature {
    /// Feature identifier.
    pub id: FeatureId,
    /// Feature name.
    pub name: String,
    /// Entry-point node labels.
    pub entry_points: Vec<String>,
}

/// Extracts features from graph and knowledge inputs.
pub trait FeatureExtractor: Send + Sync {
    /// Extract features from a graph view.
    fn extract(&self, view: &dyn GraphView) -> Result<Vec<Feature>>;
}
