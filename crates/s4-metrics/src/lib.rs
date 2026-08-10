//! # s4-metrics
//!
//! Complexity and software metrics contracts.

#![warn(missing_docs)]

/// Basic graph metrics (Phase 3).
pub mod basic;
/// Metric collector trait.
pub mod collector;
/// Complexity analyzer trait.
pub mod complexity;
/// Metric value types.
pub mod metric;

pub use basic::BasicGraphMetrics;
pub use collector::MetricCollector;
pub use complexity::{ComplexityAnalyzer, ComplexityMeasure};
pub use metric::{Metric, MetricKind, MetricValue};
