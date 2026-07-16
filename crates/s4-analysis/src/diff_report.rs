//! Markdown diff reports from cross-graph correspondence maps.

use crate::correspondence::{CorrespondenceEntry, CorrespondenceStatus};
use s4_graph::{GraphView, NodeId, NodeKind};
use std::fmt::Write as _;

/// Categorized correspondence diff between a Java source graph and a Rust target graph.
#[derive(Clone, Debug, PartialEq)]
pub struct DiffReport {
    /// Java-side graph alias.
    pub java_source: String,
    /// Rust-side graph alias.
    pub rust_source: String,
    /// Manually or deterministically confirmed mappings.
    pub ported: Vec<CorrespondenceEntry>,
    /// Paired mappings requiring review.
    pub diverged: Vec<CorrespondenceEntry>,
    /// Java nodes with no Rust counterpart.
    pub missing_in_target: Vec<CorrespondenceEntry>,
    /// Rust nodes with no Java counterpart.
    pub extra_in_target: Vec<CorrespondenceEntry>,
    /// Java nodes not yet mapped or suggested.
    pub unmapped: Vec<CorrespondenceEntry>,
    /// Aggregate counts and coverage metrics.
    pub summary: DiffSummary,
}

/// High-level metrics for a [`DiffReport`].
#[derive(Clone, Debug, PartialEq)]
pub struct DiffSummary {
    /// Callable nodes in the Java graph.
    pub total_java_callables: usize,
    /// Type nodes in the Java graph.
    pub total_java_types: usize,
    /// Entries with [`CorrespondenceStatus::Ported`].
    pub ported_count: usize,
    /// Entries with [`CorrespondenceStatus::Diverged`].
    pub diverged_count: usize,
    /// Entries with [`CorrespondenceStatus::MissingInTarget`].
    pub missing_count: usize,
    /// Entries with [`CorrespondenceStatus::ExtraInTarget`].
    pub extra_count: usize,
    /// `ported_count / total_java_callables` (0.0 when denominator is zero).
    pub coverage_pct: f32,
}

/// Build a categorized diff report from correspondence entries and the Java graph.
///
/// Entries are bucketed by [`CorrespondenceEntry::status`] and sorted by source node label
/// (Java-anchored rows) or target node id (Rust-only extras).
#[must_use]
pub fn build_diff_report(
    java_source: &str,
    rust_source: &str,
    entries: &[CorrespondenceEntry],
    java_graph: &dyn GraphView,
) -> DiffReport {
    let (total_java_callables, total_java_types) = count_java_nodes(java_graph);

    let mut ported = Vec::new();
    let mut diverged = Vec::new();
    let mut missing_in_target = Vec::new();
    let mut extra_in_target = Vec::new();
    let mut unmapped = Vec::new();

    for entry in entries {
        let entry = with_display_label(entry.clone(), java_graph);
        match entry.status {
            CorrespondenceStatus::Ported => ported.push(entry),
            CorrespondenceStatus::Diverged => diverged.push(entry),
            CorrespondenceStatus::MissingInTarget => missing_in_target.push(entry),
            CorrespondenceStatus::ExtraInTarget => extra_in_target.push(entry),
            CorrespondenceStatus::Unmapped => unmapped.push(entry),
        }
    }

    sort_by_source_label(&mut ported, java_graph);
    sort_by_source_label(&mut diverged, java_graph);
    sort_by_source_label(&mut missing_in_target, java_graph);
    sort_by_source_label(&mut unmapped, java_graph);
    sort_by_target_id(&mut extra_in_target);

    let ported_count = ported.len();
    let diverged_count = diverged.len();
    let missing_count = missing_in_target.len();
    let extra_count = extra_in_target.len();

    let coverage_pct = if total_java_callables == 0 {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            ported_count as f32 / total_java_callables as f32 * 100.0
        }
    };

    DiffReport {
        java_source: java_source.to_string(),
        rust_source: rust_source.to_string(),
        ported,
        diverged,
        missing_in_target,
        extra_in_target,
        unmapped,
        summary: DiffSummary {
            total_java_callables,
            total_java_types,
            ported_count,
            diverged_count,
            missing_count,
            extra_count,
            coverage_pct,
        },
    }
}

/// Render a human-readable Markdown report.
#[must_use]
pub fn render_markdown(report: &DiffReport) -> String {
    let mut out = String::new();

    let _ = write!(
        out,
        "# Diff: {} -> {}\n\n",
        report.java_source, report.rust_source
    );

    out.push_str("## Summary\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|------:|\n");
    let _ = writeln!(
        out,
        "| Java callables | {} |",
        report.summary.total_java_callables
    );
    let _ = writeln!(out, "| Java types | {} |", report.summary.total_java_types);
    let _ = writeln!(out, "| Ported | {} |", report.summary.ported_count);
    let _ = writeln!(out, "| Diverged | {} |", report.summary.diverged_count);
    let _ = writeln!(
        out,
        "| Missing in Rust | {} |",
        report.summary.missing_count
    );
    let _ = writeln!(out, "| Extra in Rust | {} |", report.summary.extra_count);
    let _ = write!(
        out,
        "| Coverage (ported / callables) | {:.1}% |\n\n",
        report.summary.coverage_pct
    );

    out.push_str("## Fehlt im Rust-Port\n\n");
    if report.missing_in_target.is_empty() {
        out.push_str("_None._\n\n");
    } else {
        for entry in &report.missing_in_target {
            let _ = writeln!(out, "- {}", entry_line_name(entry));
        }
        out.push('\n');
    }

    out.push_str("## Vermutlich abweichend (zur Prüfung)\n\n");
    if report.diverged.is_empty() {
        out.push_str("_None._\n\n");
    } else {
        for entry in &report.diverged {
            let _ = writeln!(
                out,
                "- {} (confidence: {:.2})",
                entry_line_name(entry),
                entry.confidence
            );
        }
        out.push('\n');
    }

    out.push_str("## Zusätzlich im Rust-Port (kein Java-Pendant)\n\n");
    if report.extra_in_target.is_empty() {
        out.push_str("_None._\n\n");
    } else {
        for entry in &report.extra_in_target {
            let _ = writeln!(out, "- {}", entry_line_name(entry));
        }
        out.push('\n');
    }

    out.push_str("## Bestätigt portiert\n\n");
    if report.ported.is_empty() {
        out.push_str("_None._\n\n");
    } else {
        for entry in &report.ported {
            let _ = writeln!(out, "- {}", entry_line_name(entry));
        }
        out.push('\n');
    }

    out
}

fn count_java_nodes(graph: &dyn GraphView) -> (usize, usize) {
    let mut callables = 0_usize;
    let mut types = 0_usize;
    for index in 0..graph.node_count() as u64 {
        if let Some(node) = graph.node(NodeId(index)) {
            match node.kind {
                NodeKind::Callable => callables += 1,
                NodeKind::Type => types += 1,
                _ => {},
            }
        }
    }
    (callables, types)
}

fn sort_by_source_label(entries: &mut [CorrespondenceEntry], graph: &dyn GraphView) {
    entries.sort_by_key(|entry| source_label(entry, graph));
}

fn sort_by_target_id(entries: &mut [CorrespondenceEntry]) {
    entries.sort_by_key(|e| e.target_node.as_ref().map_or(0, |n| n.node.0));
}

fn with_display_label(
    mut entry: CorrespondenceEntry,
    graph: &dyn GraphView,
) -> CorrespondenceEntry {
    if entry.note.is_none() {
        entry.note = Some(resolve_display_name(&entry, graph));
    }
    entry
}

fn source_label(entry: &CorrespondenceEntry, graph: &dyn GraphView) -> String {
    entry
        .source_node
        .as_ref()
        .and_then(|node_ref| graph.node(node_ref.node))
        .map_or_else(
            || resolve_display_name(entry, graph),
            |node| node.label.clone(),
        )
}

fn resolve_display_name(entry: &CorrespondenceEntry, graph: &dyn GraphView) -> String {
    if let Some(source) = &entry.source_node {
        if let Some(node) = graph.node(source.node) {
            return node.label.clone();
        }
        return format!("node:{}", source.node.0);
    }
    if let Some(target) = &entry.target_node {
        return format!("node:{}", target.node.0);
    }
    entry.id.clone()
}

fn entry_line_name(entry: &CorrespondenceEntry) -> String {
    entry.note.clone().unwrap_or_else(|| entry.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correspondence::{CorrespondenceMethod, NodeRef};
    use s4_graph::memory::InMemoryGraphView;
    use s4_graph::{Node, NodeKind};

    fn sample_graph() -> InMemoryGraphView {
        InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Callable,
                    label: "alpha".to_string(),
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "beta".to_string(),
                },
            ],
            vec![],
        )
    }

    #[test]
    fn build_diff_report_computes_coverage() {
        let graph = sample_graph();
        let entries = vec![
            CorrespondenceEntry {
                id: "1".to_string(),
                source_node: Some(NodeRef {
                    graph: crate::correspondence::GraphId("java".to_string()),
                    node: NodeId(0),
                }),
                target_node: None,
                status: CorrespondenceStatus::Ported,
                confidence: 1.0,
                method: CorrespondenceMethod::Manual,
                note: None,
                stale: false,
            },
            CorrespondenceEntry {
                id: "2".to_string(),
                source_node: Some(NodeRef {
                    graph: crate::correspondence::GraphId("java".to_string()),
                    node: NodeId(1),
                }),
                target_node: None,
                status: CorrespondenceStatus::MissingInTarget,
                confidence: 0.0,
                method: CorrespondenceMethod::NameHeuristic,
                note: None,
                stale: false,
            },
        ];

        let report = build_diff_report("java", "rust", &entries, &graph);
        assert_eq!(report.summary.total_java_callables, 2);
        assert_eq!(report.summary.ported_count, 1);
        assert_eq!(report.summary.missing_count, 1);
        assert!((report.summary.coverage_pct - 50.0).abs() < f32::EPSILON);

        let md = render_markdown(&report);
        assert!(md.contains("# Diff: java -> rust"));
        assert!(md.contains("## Fehlt im Rust-Port"));
    }
}
