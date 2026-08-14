//! Basic graph-level metrics for Phase 3.

use crate::metric::{Metric, MetricKind, MetricValue};
use crate::MetricCollector;
use s4_core::Result;
use s4_graph::{EdgeKind, GraphView, NodeKind};

/// Collects simple counts from a [`GraphView`].
#[derive(Clone, Debug, Default)]
pub struct BasicGraphMetrics {
    metrics: Vec<Metric>,
}

impl BasicGraphMetrics {
    /// Create an empty collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute node/edge counts from `view` and record them.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    pub fn collect_from_view(&mut self, view: &dyn GraphView) -> Result<()> {
        let mut callables = 0_i64;
        let mut types = 0_i64;
        let mut modules = 0_i64;
        for node in view.nodes() {
            match node.kind {
                NodeKind::Callable => callables += 1,
                NodeKind::Type => types += 1,
                NodeKind::Module => modules += 1,
                _ => {},
            }
        }
        let calls = i64::try_from(view.edges().filter(|e| e.kind == EdgeKind::Calls).count())
            .unwrap_or(i64::MAX);

        self.record(Metric {
            kind: MetricKind::Extension("callable_count".into()),
            value: MetricValue::Integer(callables),
            node: None,
        })?;
        self.record(Metric {
            kind: MetricKind::Extension("type_count".into()),
            value: MetricValue::Integer(types),
            node: None,
        })?;
        self.record(Metric {
            kind: MetricKind::Extension("module_count".into()),
            value: MetricValue::Integer(modules),
            node: None,
        })?;
        self.record(Metric {
            kind: MetricKind::Extension("calls_edge_count".into()),
            value: MetricValue::Integer(calls),
            node: None,
        })?;
        let fanout = if callables == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                calls as f64 / callables as f64
            }
        };
        self.record(Metric {
            kind: MetricKind::Coupling,
            value: MetricValue::Float(fanout),
            node: None,
        })?;
        Ok(())
    }
}

impl MetricCollector for BasicGraphMetrics {
    fn record(&mut self, metric: Metric) -> Result<()> {
        self.metrics.push(metric);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<Metric>> {
        Ok(self.metrics.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_graph::memory::InMemoryGraphView;
    use s4_graph::{Edge, Node, NodeId};

    #[test]
    fn collects_callable_and_call_counts() {
        let view = InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Callable,
                    label: "a".into(),
                    signature: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "b".into(),
                    signature: None,
                },
            ],
            vec![Edge {
                from: NodeId(0),
                to: NodeId(1),
                kind: EdgeKind::Calls,
            }],
        );
        let mut metrics = BasicGraphMetrics::new();
        metrics.collect_from_view(&view).unwrap();
        let snap = metrics.snapshot().unwrap();
        assert!(snap.iter().any(|m| {
            matches!(
                (&m.kind, &m.value),
                (
                    MetricKind::Extension(name),
                    MetricValue::Integer(2)
                ) if name == "callable_count"
            )
        }));
    }
}
