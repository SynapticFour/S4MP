use crate::GraphView;

/// Named graph layers (physical, semantic, architectural, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraphLayer {
    Physical,
    Syntax,
    Semantic,
    Structural,
    Architectural,
    Feature,
    Requirements,
    Quality,
}

/// A projection of the graph restricted to a layer.
#[derive(Clone, Debug)]
pub struct GraphProjection {
    pub layer: GraphLayer,
    pub view: GraphView,
}

impl GraphProjection {
    pub fn new(layer: GraphLayer, view: GraphView) -> Self {
        Self { layer, view }
    }
}
