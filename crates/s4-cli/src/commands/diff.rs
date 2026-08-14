use crate::workspace::{load_correspondence_entries, load_graph_from_store, Workspace};
use s4_analysis::{build_diff_report, render_json, render_markdown};
use s4_core::Result;
use std::path::Path;

/// Render a Markdown (+ JSON sidecar) diff report for a Java/Rust port pair.
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
        report.summary.ported_callable_count,
        report.summary.total_java_callables
    );

    if let Some(parent) = Path::new(out).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            s4_core::S4Error::Storage(format!(
                "failed to create report directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let markdown = render_markdown(&report);
    std::fs::write(out, &markdown).map_err(|e| {
        s4_core::S4Error::Storage(format!("failed to write diff report {out}: {e}"))
    })?;

    let json_path = json_sidecar_path(out);
    let json = render_json(&report).map_err(|e| {
        s4_core::S4Error::Storage(format!("failed to serialize JSON diff report: {e}"))
    })?;
    std::fs::write(&json_path, json).map_err(|e| {
        s4_core::S4Error::Storage(format!(
            "failed to write JSON diff report {}: {e}",
            json_path.display()
        ))
    })?;

    println!("Diff report written to {out}");
    println!("JSON sidecar written to {}", json_path.display());
    Ok(())
}

fn json_sidecar_path(markdown_out: &str) -> std::path::PathBuf {
    Path::new(markdown_out).with_extension("json")
}
