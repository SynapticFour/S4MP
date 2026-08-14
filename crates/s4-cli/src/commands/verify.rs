//! Run Phase 5 verification over porting + requirements artifacts.

use crate::workspace::{load_correspondence_entries, load_graph_from_store, Workspace};
use s4_analysis::build_diff_report;
use s4_core::{Result, S4Error, MATURITY};
use s4_requirements::{RequirementsDocument, TraceLinkKind, TraceabilityGraph};
use s4_verification::{build_verification_run, VerificationInputs, VerificationThresholds};
use std::path::PathBuf;

/// Verify Java/Rust port artifacts against thresholds.
pub fn run(
    java: &str,
    rust: &str,
    min_coverage: f32,
    forbid_missing: bool,
    require_traced: bool,
) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.find_source(java)?;
    ws.find_source(rust)?;
    let java_manifest = ws.load_graph_manifest(java)?;
    let map_manifest = ws.load_map_manifest(java, rust)?;
    let store = ws.store()?;
    let java_graph = load_graph_from_store(&store, &java_manifest.graph_artifact_id)?;
    let entries = load_correspondence_entries(&store, &map_manifest)?;
    let report = build_diff_report(java, rust, &entries, &java_graph);

    let req_path = ws.root().join(".s4").join("requirements.json");
    let doc = RequirementsDocument::load(&req_path)?;
    let requirements_total = doc.requirements.len();
    let mut requirements_traced = 0_usize;
    for req_id in doc.requirements() {
        let traces = doc.traces_from(req_id)?;
        if traces
            .iter()
            .any(|t| t.kind == TraceLinkKind::ImplementedBy)
        {
            requirements_traced += 1;
        }
    }

    let thresholds = VerificationThresholds {
        min_coverage_pct: min_coverage,
        require_all_requirements_traced: require_traced,
        forbid_missing_in_target: forbid_missing,
    };

    let run = build_verification_run(
        &VerificationInputs {
            java_source: java,
            rust_source: rust,
            maturity: MATURITY,
            ported: report.summary.ported_count,
            diverged: report.summary.diverged_count,
            missing: report.summary.missing_count,
            extra: report.summary.extra_count,
            java_callables: report.summary.total_java_callables,
            coverage_pct: report.summary.coverage_pct,
            requirements_total,
            requirements_traced,
        },
        &thresholds,
    );

    let out_dir = ws.root().join(".s4").join("verification");
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| S4Error::Storage(format!("failed to create {}: {e}", out_dir.display())))?;
    let out: PathBuf = out_dir.join(format!("{java}__{rust}.json"));
    let bytes = serde_json::to_vec_pretty(&run)
        .map_err(|e| S4Error::Storage(format!("serialize verification run: {e}")))?;
    std::fs::write(&out, bytes)
        .map_err(|e| S4Error::Storage(format!("write {}: {e}", out.display())))?;

    println!("{}", run.summary);
    println!("wrote {}", out.display());
    println!(
        "note: this verifies coverage/trace thresholds only — not semantic equivalence ({MATURITY})"
    );
    if run.passed {
        Ok(())
    } else {
        Err(S4Error::CheckFailed(run.summary))
    }
}
