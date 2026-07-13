use crate::Metric;
use s4_core::Result;

/// Collects and aggregates metrics across pipeline stages.
pub trait MetricCollector: Send + Sync {
    /// Record a single metric.
    fn record(&mut self, metric: Metric) -> Result<()>;

    /// Snapshot all collected metrics.
    fn snapshot(&self) -> Result<Vec<Metric>>;
}
