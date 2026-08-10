use crate::{EdgeKind, GraphView, Node, NodeId, NodeKind};
use s4_core::{Result, S4Error};

/// Result of a graph query.
#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    /// Matching nodes.
    pub nodes: Vec<Node>,
}

/// Graph query interface (S4QL foundation).
pub trait GraphQuery: Send + Sync {
    /// Execute a query expression against a graph view.
    ///
    /// # Errors
    ///
    /// Returns an error if the query is invalid or execution fails.
    fn execute(&self, view: &dyn GraphView, expression: &str) -> Result<QueryResult>;
}

/// Built-in query expression shapes (Phase 3 filter subset).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryExpr {
    /// Match all nodes.
    All,
    /// Match nodes of a given kind (`kind:callable`).
    MatchKind(NodeKind),
    /// Substring match on label (`label~add`).
    LabelContains(String),
}

impl QueryExpr {
    /// Parse a Phase 3 filter expression.
    ///
    /// Supported:
    /// - `all`
    /// - `kind:<callable|type|module|symbol|package>`
    /// - `label~<substring>` (case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns an error for unknown shapes.
    pub fn parse(expression: &str) -> Result<Self> {
        let expr = expression.trim();
        if expr.is_empty() || expr.eq_ignore_ascii_case("all") {
            return Ok(Self::All);
        }
        if let Some(kind) = expr.strip_prefix("kind:") {
            let kind = parse_node_kind(kind)?;
            return Ok(Self::MatchKind(kind));
        }
        if let Some(sub) = expr.strip_prefix("label~") {
            return Ok(Self::LabelContains(sub.to_ascii_lowercase()));
        }
        Err(S4Error::Other(format!(
            "unsupported query '{expression}' (expected all | kind:<k> | label~<substr>)"
        )))
    }
}

fn parse_node_kind(raw: &str) -> Result<NodeKind> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "callable" => Ok(NodeKind::Callable),
        "type" => Ok(NodeKind::Type),
        "module" => Ok(NodeKind::Module),
        "symbol" => Ok(NodeKind::Symbol),
        "package" => Ok(NodeKind::Package),
        other => Err(S4Error::Other(format!("unknown node kind '{other}'"))),
    }
}

/// Deterministic in-process filter query (Phase 3).
#[derive(Clone, Debug, Default)]
pub struct FilterQuery;

impl GraphQuery for FilterQuery {
    fn execute(&self, view: &dyn GraphView, expression: &str) -> Result<QueryResult> {
        let parsed = QueryExpr::parse(expression)?;
        let mut nodes = Vec::new();
        for index in 0..view.node_count() as u64 {
            let Some(node) = view.node(NodeId(index)) else {
                continue;
            };
            let matched = match &parsed {
                QueryExpr::All => true,
                QueryExpr::MatchKind(kind) => &node.kind == kind,
                QueryExpr::LabelContains(sub) => node.label.to_ascii_lowercase().contains(sub),
            };
            if matched {
                nodes.push(node.clone());
            }
        }
        Ok(QueryResult { nodes })
    }
}

/// Structural diff between two graphs keyed by `(kind, label)`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphDiff {
    /// Present only in the left (baseline) graph.
    pub only_left: Vec<(NodeKind, String)>,
    /// Present only in the right (candidate) graph.
    pub only_right: Vec<(NodeKind, String)>,
    /// Present in both.
    pub shared: Vec<(NodeKind, String)>,
}

impl GraphDiff {
    /// Diff two graph views by node kind + label identity.
    #[must_use]
    pub fn from_views(left: &dyn GraphView, right: &dyn GraphView) -> Self {
        let left_keys = node_keys(left);
        let right_keys = node_keys(right);
        let only_left = left_keys.difference(&right_keys).cloned().collect();
        let only_right = right_keys.difference(&left_keys).cloned().collect();
        let shared = left_keys.intersection(&right_keys).cloned().collect();
        Self {
            only_left,
            only_right,
            shared,
        }
    }

    /// Count of `Calls` edges in a view.
    #[must_use]
    pub fn call_edge_count(view: &dyn GraphView) -> usize {
        view.edges().filter(|e| e.kind == EdgeKind::Calls).count()
    }
}

fn node_keys(view: &dyn GraphView) -> std::collections::BTreeSet<(NodeKind, String)> {
    let mut keys = std::collections::BTreeSet::new();
    for index in 0..view.node_count() as u64 {
        if let Some(node) = view.node(NodeId(index)) {
            keys.insert((node.kind.clone(), node.label.clone()));
        }
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::InMemoryGraphView;
    use crate::Node;

    #[test]
    fn filter_by_kind_and_label() {
        let view = InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Callable,
                    label: "add".into(),
                    signature: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Type,
                    label: "Calculator".into(),
                    signature: None,
                },
            ],
            vec![],
        );
        let q = FilterQuery;
        let callables = q.execute(&view, "kind:callable").unwrap();
        assert_eq!(callables.nodes.len(), 1);
        let labeled = q.execute(&view, "label~calc").unwrap();
        assert_eq!(labeled.nodes.len(), 1);
    }

    #[test]
    fn graph_diff_detects_added_removed() {
        let left = InMemoryGraphView::new(
            vec![Node {
                id: NodeId(0),
                kind: NodeKind::Callable,
                label: "a".into(),
                signature: None,
            }],
            vec![],
        );
        let right = InMemoryGraphView::new(
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
            vec![],
        );
        let diff = GraphDiff::from_views(&left, &right);
        assert!(diff.only_left.is_empty());
        assert_eq!(diff.only_right.len(), 1);
        assert_eq!(diff.shared.len(), 1);
    }
}
