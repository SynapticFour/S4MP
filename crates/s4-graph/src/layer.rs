/// Named graph layers in the S4MP multi-graph model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraphLayer {
    /// Files, blobs, VCS metadata.
    Physical,
    /// AST and tokens.
    Syntax,
    /// USIR semantic layer.
    Semantic,
    /// Packages and dependencies.
    Structural,
    /// Boundaries, patterns, layers.
    Architectural,
    /// Capabilities and entry points.
    Feature,
    /// Formal requirements.
    Requirements,
    /// Metrics, findings, certificates.
    Quality,
}
