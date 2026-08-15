//! Markdown diff reports from cross-graph correspondence maps.

use crate::correspondence::{CorrespondenceEntry, CorrespondenceStatus};
use s4_graph::{GraphView, NodeKind};
use serde::Serialize;
use std::fmt::Write as _;

/// Categorized correspondence diff between a Java source graph and a Rust target graph.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiffSummary {
    /// Callable nodes in the Java graph.
    pub total_java_callables: usize,
    /// Type nodes in the Java graph.
    pub total_java_types: usize,
    /// Entries with [`CorrespondenceStatus::Ported`].
    pub ported_count: usize,
    /// Ported rows whose Java source node is a callable.
    pub ported_callable_count: usize,
    /// Entries with [`CorrespondenceStatus::Diverged`].
    pub diverged_count: usize,
    /// Entries with [`CorrespondenceStatus::MissingInTarget`].
    pub missing_count: usize,
    /// Entries with [`CorrespondenceStatus::ExtraInTarget`].
    pub extra_count: usize,
    /// `ported_callable_count / total_java_callables` (0.0 when denominator is zero).
    pub coverage_pct: f32,
}

/// Counts of diverged entries by confidence band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ConfidenceBands {
    /// Confidence ≥ 0.85.
    pub high: usize,
    /// Confidence in `[0.65, 0.85)`.
    pub medium: usize,
    /// Confidence &lt; 0.65.
    pub low: usize,
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
        match entry.status {
            CorrespondenceStatus::Ported => ported.push(entry.clone()),
            CorrespondenceStatus::Diverged => diverged.push(entry.clone()),
            CorrespondenceStatus::MissingInTarget => missing_in_target.push(entry.clone()),
            CorrespondenceStatus::ExtraInTarget => extra_in_target.push(entry.clone()),
            CorrespondenceStatus::Unmapped => unmapped.push(entry.clone()),
        }
    }

    sort_by_source_label(&mut ported, java_graph);
    sort_by_source_label(&mut diverged, java_graph);
    sort_by_source_label(&mut missing_in_target, java_graph);
    sort_by_source_label(&mut unmapped, java_graph);
    sort_by_target_id(&mut extra_in_target);

    let ported_count = ported.len();
    let ported_callable_count = ported
        .iter()
        .filter(|entry| is_java_callable(entry, java_graph))
        .count();
    let diverged_count = diverged.len();
    let missing_count = missing_in_target.len();
    let extra_count = extra_in_target.len();

    let coverage_pct = if total_java_callables == 0 {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            ported_callable_count as f32 / total_java_callables as f32 * 100.0
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
            ported_callable_count,
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

    out.push_str("> **Maturity:** `heuristic-map-v2`. ");
    out.push_str(
        "Name (+ optional signature) similarity maps only — not semantic equivalence, not a certificate. ",
    );
    out.push_str(
        "`s4 verify` / `s4 certify` evaluate coverage/policy thresholds only — not semantic equivalence.\n\n",
    );

    out.push_str("## How to review\n\n");
    out.push_str("Heuristic pairs are **Diverged**, never auto-Ported. Confirm a pair, then re-run `s4 diff` / `s4 verify` / `s4 certify`.\n\n");
    out.push_str("```bash\n");
    let _ = writeln!(
        out,
        "s4 map show --java {} --rust {}",
        report.java_source, report.rust_source
    );
    out.push_str("s4 map confirm --id <id>          # unique prefix of id= is enough\n");
    out.push_str("s4 map confirm --name add         # error if the name is ambiguous\n");
    out.push_str("```\n\n");
    out.push_str("Default `s4 certify` requires **at least one Ported** row.\n\n");

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
        "| Coverage (ported callables / Java callables) | {:.1}% |\n\n",
        report.summary.coverage_pct
    );

    out.push_str("## Confidence bands (diverged heuristics)\n\n");
    let bands = confidence_bands(&report.diverged);
    out.push_str("| Band | Count |\n");
    out.push_str("|------|------:|\n");
    let _ = writeln!(out, "| high (≥ 0.85) | {} |", bands.high);
    let _ = writeln!(out, "| medium (0.65–0.85) | {} |", bands.medium);
    let _ = writeln!(out, "| low (< 0.65) | {} |\n", bands.low);

    write_entry_section(
        &mut out,
        "Missing in Rust",
        &report.missing_in_target,
        EntryLineKind::Missing,
    );
    write_entry_section(
        &mut out,
        "Diverged (review these)",
        &report.diverged,
        EntryLineKind::Paired,
    );
    write_entry_section(
        &mut out,
        "Extra in Rust (no Java counterpart)",
        &report.extra_in_target,
        EntryLineKind::Extra,
    );
    write_entry_section(
        &mut out,
        "Ported (manually confirmed)",
        &report.ported,
        EntryLineKind::Paired,
    );

    if !report.unmapped.is_empty() {
        write_entry_section(
            &mut out,
            "Unmapped",
            &report.unmapped,
            EntryLineKind::Missing,
        );
    }

    out
}

#[derive(Clone, Copy)]
enum EntryLineKind {
    Paired,
    Missing,
    Extra,
}

fn write_entry_section(
    out: &mut String,
    heading: &str,
    entries: &[CorrespondenceEntry],
    kind: EntryLineKind,
) {
    let _ = writeln!(out, "## {heading}\n");
    if entries.is_empty() {
        out.push_str("_None._\n\n");
        return;
    }
    for entry in entries {
        let _ = writeln!(out, "- {}", format_entry_line(entry, kind));
    }
    out.push('\n');
}

fn format_entry_line(entry: &CorrespondenceEntry, kind: EntryLineKind) -> String {
    let name = entry_line_name(entry);
    let id = crate::correspondence::short_entry_id(&entry.id);
    let mut line = match kind {
        EntryLineKind::Paired => {
            let mut s = format!("**{name}** `id={id}`");
            if !matches!(entry.status, CorrespondenceStatus::Ported) {
                let _ = write!(s, " (confidence: {:.2})", entry.confidence);
            }
            s
        },
        EntryLineKind::Missing | EntryLineKind::Extra => format!("**{name}** `id={id}`"),
    };
    if let Some(sig) = entry.source_signature.as_deref().filter(|s| !s.is_empty()) {
        let _ = write!(line, "  \n  Java `{sig}`");
    }
    if let Some(sig) = entry.target_signature.as_deref().filter(|s| !s.is_empty()) {
        let _ = write!(line, "  \n  Rust `{sig}`");
    }
    line
}

/// Render a machine-readable JSON sidecar for Showcase / CI evidence packs.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn render_json(report: &DiffReport) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        maturity: &'static str,
        confidence_bands: ConfidenceBands,
        report: &'a DiffReport,
    }
    let envelope = Envelope {
        maturity: s4_core::MATURITY,
        confidence_bands: confidence_bands(&report.diverged),
        report,
    };
    serde_json::to_string_pretty(&envelope)
}

/// Bucket diverged entries by confidence for report summaries.
#[must_use]
pub fn confidence_bands(diverged: &[CorrespondenceEntry]) -> ConfidenceBands {
    let mut bands = ConfidenceBands {
        high: 0,
        medium: 0,
        low: 0,
    };
    for entry in diverged {
        if entry.confidence >= 0.85 {
            bands.high += 1;
        } else if entry.confidence >= 0.65 {
            bands.medium += 1;
        } else {
            bands.low += 1;
        }
    }
    bands
}

fn count_java_nodes(graph: &dyn GraphView) -> (usize, usize) {
    let mut callables = 0_usize;
    let mut types = 0_usize;
    for node in graph.nodes() {
        match node.kind {
            NodeKind::Callable => callables += 1,
            NodeKind::Type => types += 1,
            _ => {},
        }
    }
    (callables, types)
}

fn is_java_callable(entry: &CorrespondenceEntry, graph: &dyn GraphView) -> bool {
    entry
        .source_node
        .as_ref()
        .and_then(|node_ref| graph.node(node_ref.node))
        .is_some_and(|node| node.kind == NodeKind::Callable)
}

fn sort_by_source_label(entries: &mut [CorrespondenceEntry], graph: &dyn GraphView) {
    entries.sort_by_cached_key(|entry| source_label(entry, graph));
}

fn sort_by_target_id(entries: &mut [CorrespondenceEntry]) {
    entries.sort_by_key(|e| e.target_node.as_ref().map_or(0, |n| n.node.0));
}

fn source_label(entry: &CorrespondenceEntry, graph: &dyn GraphView) -> String {
    entry
        .display_name
        .clone()
        .or_else(|| {
            entry
                .source_node
                .as_ref()
                .and_then(|node_ref| graph.node(node_ref.node))
                .map(|node| node.label.clone())
        })
        .unwrap_or_else(|| resolve_display_name(entry, graph))
}

fn resolve_display_name(entry: &CorrespondenceEntry, graph: &dyn GraphView) -> String {
    if let Some(name) = entry.display_name.as_deref().filter(|s| !s.is_empty()) {
        return name.to_string();
    }
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
    entry
        .display_name
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| entry.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correspondence::{CorrespondenceMethod, NodeRef};
    use s4_graph::memory::InMemoryGraphView;
    use s4_graph::{Node, NodeId, NodeKind};

    fn sample_graph() -> InMemoryGraphView {
        InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Callable,
                    label: "alpha".to_string(),
                    signature: None,
                    qualified: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "beta".to_string(),
                    signature: None,
                    qualified: None,
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
                display_name: Some("alpha".into()),
                source_signature: None,
                target_signature: None,
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
                display_name: Some("beta".into()),
                source_signature: None,
                target_signature: None,
                stale: false,
            },
        ];

        let report = build_diff_report("java", "rust", &entries, &graph);
        assert_eq!(report.summary.total_java_callables, 2);
        assert_eq!(report.summary.ported_count, 1);
        assert_eq!(report.summary.ported_callable_count, 1);
        assert_eq!(report.summary.missing_count, 1);
        assert!((report.summary.coverage_pct - 50.0).abs() < f32::EPSILON);

        let md = render_markdown(&report);
        assert!(md.contains("# Diff: java -> rust"));
        assert!(md.contains("heuristic-map-v2"));
        assert!(md.contains("## Missing in Rust"));
        assert!(md.contains("## Diverged (review these)"));
        assert!(md.contains("s4 map confirm --id"));
        assert!(md.contains("`id=1`"));
        assert!(md.contains("`id=2`"));
        assert!(md.contains("Confidence bands"));
        assert!(md.contains("alpha"));
        assert!(!md.contains("manually confirmed") || md.contains("alpha"));
    }

    #[test]
    fn coverage_counts_only_ported_callables() {
        let graph = InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Callable,
                    label: "alpha".to_string(),
                    signature: None,
                    qualified: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Type,
                    label: "Calc".to_string(),
                    signature: None,
                    qualified: None,
                },
            ],
            vec![],
        );
        let entries = vec![
            CorrespondenceEntry {
                id: "fn".to_string(),
                source_node: Some(NodeRef {
                    graph: crate::correspondence::GraphId("java".to_string()),
                    node: NodeId(0),
                }),
                target_node: None,
                status: CorrespondenceStatus::Ported,
                confidence: 1.0,
                method: CorrespondenceMethod::Manual,
                note: None,
                display_name: Some("alpha".into()),
                source_signature: None,
                target_signature: None,
                stale: false,
            },
            CorrespondenceEntry {
                id: "ty".to_string(),
                source_node: Some(NodeRef {
                    graph: crate::correspondence::GraphId("java".to_string()),
                    node: NodeId(1),
                }),
                target_node: None,
                status: CorrespondenceStatus::Ported,
                confidence: 1.0,
                method: CorrespondenceMethod::Manual,
                note: None,
                display_name: Some("Calc".into()),
                source_signature: None,
                target_signature: None,
                stale: false,
            },
        ];
        let report = build_diff_report("java", "rust", &entries, &graph);
        assert_eq!(report.summary.ported_count, 2);
        assert_eq!(report.summary.ported_callable_count, 1);
        assert_eq!(report.summary.total_java_callables, 1);
        assert!((report.summary.coverage_pct - 100.0).abs() < f32::EPSILON);
    }
}
