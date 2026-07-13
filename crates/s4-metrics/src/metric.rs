use s4_graph::NodeId;
use serde::{Deserialize, Serialize};

/// Recorded software metric.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    /// Metric classification.
    pub kind: MetricKind,
    /// Measured value.
    pub value: MetricValue,
    /// Related graph node, if any.
    pub node: Option<NodeId>,
}

/// Standard metric kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    /// Cyclomatic complexity.
    CyclomaticComplexity,
    /// Lines of code.
    LinesOfCode,
    /// Cognitive complexity.
    CognitiveComplexity,
    /// Coupling metric.
    Coupling,
    /// Extension metric.
    Extension(String),
}

/// Typed metric value.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValue {
    /// Integer measurement.
    Integer(i64),
    /// Floating-point measurement.
    Float(f64),
    /// Boolean flag.
    Bool(bool),
}
