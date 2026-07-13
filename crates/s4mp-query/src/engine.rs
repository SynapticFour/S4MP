use crate::{Query, QueryExpr, QueryResult};
use s4mp_graph::GraphView;

/// Executes queries over graph views.
pub struct QueryEngine;

impl QueryEngine {
    pub fn execute(query: &Query, graph: &GraphView) -> QueryResult {
        match &query.expr {
            QueryExpr::All => QueryResult {
                nodes: graph.nodes.values().cloned().collect(),
            },
            QueryExpr::MatchNodes { kind: None } => QueryResult {
                nodes: graph.nodes.values().cloned().collect(),
            },
            QueryExpr::MatchNodes {
                kind: Some(expected),
            } => QueryResult {
                nodes: graph
                    .nodes
                    .values()
                    .filter(|n| &n.kind == expected)
                    .cloned()
                    .collect(),
            },
        }
    }
}
