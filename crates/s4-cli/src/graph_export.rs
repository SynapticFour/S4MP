//! Graph export formats (DOT, JSON) for visualization.

use crate::workspace::GraphProjectionPayload;
use s4_core::{Result, S4Error};
use s4_graph::{EdgeKind, NodeKind};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as _;

/// Parsed `--filter` tokens for graph export.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphExportFilter {
    /// When true, include all nodes and edges.
    pub include_all: bool,
    /// Node kinds to include explicitly (e.g. `callable`, `type`).
    pub node_kinds: HashSet<String>,
    /// Edge kinds to include explicitly (e.g. `calls`, `defines`).
    pub edge_kinds: HashSet<String>,
}

/// Supported export formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphExportFormat {
    /// Graphviz DOT digraph.
    Dot,
    /// Pretty-printed JSON subset (nodes + edges after filtering).
    Json,
}

impl GraphExportFormat {
    /// Parse a CLI `--format` value.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown format names.
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "dot" => Ok(Self::Dot),
            "json" => Ok(Self::Json),
            other => Err(S4Error::InvalidInput(format!(
                "unsupported export format '{other}' (expected 'dot' or 'json')"
            ))),
        }
    }
}

/// Parse comma-separated filter tokens (`callable,calls,type`).
#[must_use]
pub fn parse_filter(filter: &str) -> GraphExportFilter {
    let tokens: HashSet<String> = filter
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();

    if tokens.is_empty() || tokens.contains("all") {
        return GraphExportFilter {
            include_all: true,
            ..GraphExportFilter::default()
        };
    }

    let node_kinds: HashSet<String> = tokens
        .iter()
        .filter(|t| is_node_kind_token(t))
        .cloned()
        .collect();
    let edge_kinds: HashSet<String> = tokens
        .iter()
        .filter(|t| is_edge_kind_token(t))
        .cloned()
        .collect();

    GraphExportFilter {
        include_all: false,
        node_kinds,
        edge_kinds,
    }
}

/// Render a filtered graph projection in the requested format.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn render_export(
    payload: &GraphProjectionPayload,
    filter: &GraphExportFilter,
    format: GraphExportFormat,
) -> Result<String> {
    let (nodes, edges) = apply_filter(payload, filter);
    match format {
        GraphExportFormat::Dot => Ok(render_dot(payload, &nodes, &edges)),
        GraphExportFormat::Json => render_json(payload, &nodes, &edges),
    }
}

fn apply_filter<'a>(
    payload: &'a GraphProjectionPayload,
    filter: &GraphExportFilter,
) -> (BTreeSet<u64>, Vec<&'a s4_graph::Edge>) {
    if filter.include_all {
        let nodes: BTreeSet<u64> = payload.nodes.iter().map(|n| n.id.0).collect();
        let edges: Vec<_> = payload.edges.iter().collect();
        return (nodes, edges);
    }

    let mut included_edges: Vec<&s4_graph::Edge> = payload
        .edges
        .iter()
        .filter(|e| filter.edge_kinds.contains(edge_kind_token(&e.kind)))
        .collect();

    let mut included_nodes: BTreeSet<u64> = payload
        .nodes
        .iter()
        .filter(|n| filter.node_kinds.contains(node_kind_token(&n.kind)))
        .map(|n| n.id.0)
        .collect();

    for edge in &included_edges {
        included_nodes.insert(edge.from.0);
        included_nodes.insert(edge.to.0);
    }

    included_edges
        .retain(|e| included_nodes.contains(&e.from.0) && included_nodes.contains(&e.to.0));

    (included_nodes, included_edges)
}

fn render_dot(
    payload: &GraphProjectionPayload,
    nodes: &BTreeSet<u64>,
    edges: &[&s4_graph::Edge],
) -> String {
    let graph_name = sanitize_dot_id(&payload.source_alias);
    let mut out = String::new();
    let _ = writeln!(out, "digraph \"{graph_name}\" {{");
    let _ = writeln!(out, "  rankdir=LR;");
    let _ = writeln!(out, "  node [shape=box, fontsize=10];");
    let _ = writeln!(
        out,
        "  label=\"{} ({} nodes, {} edges)\";",
        escape_dot_label(&payload.source_alias),
        nodes.len(),
        edges.len()
    );

    for node in &payload.nodes {
        if !nodes.contains(&node.id.0) {
            continue;
        }
        let kind = node_kind_token(&node.kind);
        let color = node_color(kind);
        let _ = writeln!(
            out,
            "  n{} [label=\"{}: {}\", style=filled, fillcolor=\"{}\"];",
            node.id.0,
            kind,
            escape_dot_label(&node.label),
            color
        );
    }

    for edge in edges {
        let _ = writeln!(
            out,
            "  n{} -> n{} [label=\"{}\"];",
            edge.from.0,
            edge.to.0,
            edge_kind_token(&edge.kind)
        );
    }

    out.push_str("}\n");
    out
}

fn render_json(
    payload: &GraphProjectionPayload,
    nodes: &BTreeSet<u64>,
    edges: &[&s4_graph::Edge],
) -> Result<String> {
    let filtered_nodes: Vec<_> = payload
        .nodes
        .iter()
        .filter(|n| nodes.contains(&n.id.0))
        .cloned()
        .collect();
    let filtered_edges: Vec<_> = edges.iter().map(|e| (*e).clone()).collect();
    let export = serde_json::json!({
        "source_alias": payload.source_alias,
        "nodes": filtered_nodes,
        "edges": filtered_edges,
    });
    serde_json::to_string_pretty(&export)
        .map_err(|e| S4Error::Storage(format!("failed to serialize graph export JSON: {e}")))
}

fn node_kind_token(kind: &NodeKind) -> &str {
    match kind {
        NodeKind::Module => "module",
        NodeKind::Symbol => "symbol",
        NodeKind::Callable => "callable",
        NodeKind::Type => "type",
        NodeKind::Package => "package",
        NodeKind::Extension(_) => "extension",
    }
}

fn edge_kind_token(kind: &EdgeKind) -> &str {
    match kind {
        EdgeKind::Defines => "defines",
        EdgeKind::References => "references",
        EdgeKind::Calls => "calls",
        EdgeKind::Implements => "implements",
        EdgeKind::DependsOn => "depends_on",
        EdgeKind::Extension(_) => "extension",
    }
}

fn is_node_kind_token(token: &str) -> bool {
    matches!(
        token,
        "module" | "symbol" | "callable" | "type" | "package" | "extension"
    )
}

fn is_edge_kind_token(token: &str) -> bool {
    matches!(
        token,
        "defines" | "references" | "calls" | "implements" | "depends_on" | "extension"
    )
}

fn node_color(kind: &str) -> &'static str {
    match kind {
        "callable" => "#dbeafe",
        "type" => "#dcfce7",
        "module" => "#f3f4f6",
        "symbol" => "#fef9c3",
        _ => "#ffffff",
    }
}

fn sanitize_dot_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_dot_label(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_graph::{Edge, Node, NodeId};

    fn sample_payload() -> GraphProjectionPayload {
        GraphProjectionPayload {
            source_alias: "hc-rust".to_string(),
            nodes: vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Module,
                    label: "lib.rs".to_string(),
                    signature: None,
                    qualified: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "run".to_string(),
                    signature: None,
                    qualified: None,
                },
                Node {
                    id: NodeId(2),
                    kind: NodeKind::Callable,
                    label: "helper".to_string(),
                    signature: None,
                    qualified: None,
                },
            ],
            edges: vec![
                Edge {
                    from: NodeId(0),
                    to: NodeId(1),
                    kind: EdgeKind::Defines,
                },
                Edge {
                    from: NodeId(1),
                    to: NodeId(2),
                    kind: EdgeKind::Calls,
                },
            ],
        }
    }

    #[test]
    fn filter_callable_and_calls_includes_endpoints() {
        let payload = sample_payload();
        let filter = parse_filter("callable,calls");
        let (nodes, edges) = apply_filter(&payload, &filter);
        assert!(nodes.contains(&1));
        assert!(nodes.contains(&2));
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].kind, EdgeKind::Calls);
    }

    #[test]
    fn render_dot_contains_nodes_and_edges() {
        let filter = parse_filter("all");
        let dot = render_export(&sample_payload(), &filter, GraphExportFormat::Dot).expect("dot");
        assert!(dot.contains("digraph \"hc_rust\""));
        assert!(dot.contains("n1"));
        assert!(dot.contains("callable: run"));
    }
}
