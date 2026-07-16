use crate::workspace::{load_correspondence_entries, load_graph_from_store, Workspace};
use s4_analysis::{build_diff_report, render_markdown};
use s4_core::Result;

/// Render a Markdown diff report for a Java/Rust port pair.
pub fn run(java: &str, rust: &str, out: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.find_source(java)?;
    ws.find_source(rust)?;

    let java_manifest = ws.load_graph_manifest(java)?;
    let map_manifest = ws.load_map_manifest(java, rust)?;

    let store = ws.store()?;
    println!("Loading Java graph '{java}'...");
    let java_graph = load_graph_from_store(&store, &java_manifest.graph_artifact_id)?;

    println!(
        "Loading correspondence map ({} entries)...",
        map_manifest.entry_count
    );
    let entries = load_correspondence_entries(&store, &map_manifest)?;

    println!("Building diff report...");
    let report = build_diff_report(java, rust, &entries, &java_graph);
    println!(
        "  coverage: {:.1}% ({}/{} callables ported)",
        report.summary.coverage_pct,
        report.summary.ported_count,
        report.summary.total_java_callables
    );

    let markdown = render_markdown(&report);
    std::fs::write(out, &markdown)
        .map_err(|e| s4_core::S4Error::Other(format!("failed to write diff report {out}: {e}")))?;

    println!("Diff report written to {out}");
    Ok(())
}
