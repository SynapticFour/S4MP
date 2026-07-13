use crate::Metric;
use s4_core::Result;

/// Collects and aggregates metrics across pipeline stages.
pub trait MetricCollector: Send + Sync {
    /// Record a single metric.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    fn record(&mut self, metric: Metric) -> Result<()>;

    /// Snapshot all collected metrics.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot fails.
    fn snapshot(&self) -> Result<Vec<Metric>>;
}
