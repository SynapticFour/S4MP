use s4_graph::NodeId;

/// Opaque finding identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FindingId(pub u64);

/// Analysis finding emitted by an analyzer.
#[derive(Clone, Debug)]
pub struct Finding {
    /// Finding identifier.
    pub id: FindingId,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Related graph node, if any.
    pub related_node: Option<NodeId>,
}

/// Finding severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Informational.
    Info,
    /// Warning — potential issue.
    Warning,
    /// Error — definite issue.
    Error,
    /// Critical — blocks certification.
    Critical,
}
