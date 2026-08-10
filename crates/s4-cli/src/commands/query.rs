//! Knowledge-graph query (Phase 3 filter subset).

use crate::workspace::{load_graph_from_store, Workspace};
use s4_core::Result;
use s4_graph::{FilterQuery, GraphQuery};
use s4_metrics::{BasicGraphMetrics, MetricCollector, MetricKind, MetricValue};

/// Run a filter query against a built source graph.
pub fn run(source: &str, expr: &str, with_metrics: bool) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.find_source(source)?;
    let manifest = ws.load_graph_manifest(source)?;
    let store = ws.store()?;
    let graph = load_graph_from_store(&store, &manifest.graph_artifact_id)?;

    let query = FilterQuery;
    let result = query.execute(&graph, expr)?;
    println!(
        "query '{expr}' on '{source}': {} node(s)",
        result.nodes.len()
    );
    for node in &result.nodes {
        match &node.signature {
            Some(sig) => println!("  [{:?}] {} — {sig}", node.kind, node.label),
            None => println!("  [{:?}] {}", node.kind, node.label),
        }
    }

    if with_metrics {
        let mut metrics = BasicGraphMetrics::new();
        metrics.collect_from_view(&graph)?;
        println!("metrics:");
        for metric in metrics.snapshot()? {
            let name = match &metric.kind {
                MetricKind::Extension(n) => n.clone(),
                MetricKind::Coupling => "avg_calls_per_callable".into(),
                other => format!("{other:?}"),
            };
            match metric.value {
                MetricValue::Integer(v) => println!("  {name}: {v}"),
                MetricValue::Float(v) => println!("  {name}: {v:.3}"),
                MetricValue::Bool(v) => println!("  {name}: {v}"),
            }
        }
    }
    Ok(())
}
