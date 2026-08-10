//! `s4 analyze` — run the Phase 2 porting pass pipeline over registered sources.

use crate::commands::{diff, graph, map};
use crate::workspace::{parse_artifact_id, Workspace};
use s4_analysis::PORTING_PASS_ORDER;
use s4_core::{ArtifactId, Result, S4Error};
use s4_events::{EventKind, RecordingEventSink};
use s4_parser::LanguageId;

/// Run graph builds for all sources, then optional map+diff for a Java/Rust pair.
pub fn run(java: Option<&str>, rust: Option<&str>, out: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    let registry = ws.load_sources()?;
    if registry.sources.is_empty() {
        return Err(S4Error::Other(
            "no sources registered — run `s4 init` and `s4 source add` first".to_string(),
        ));
    }

    println!("Pass order: {}", PORTING_PASS_ORDER.join(" → "));
    let events = RecordingEventSink::new();

    for source in &registry.sources {
        println!("Building graph for '{}'...", source.alias);
        graph::run_build(&source.alias, ".s4/graphs")?;
        if let Ok(manifest) = ws.load_graph_manifest(&source.alias) {
            if let Ok(id) = parse_artifact_id(&manifest.graph_artifact_id) {
                events.emit(EventKind::GraphUpdated { projection: id }, None);
            }
        }
    }

    let java_alias = java
        .map(str::to_string)
        .or_else(|| first_alias_for_lang(&registry.sources, "java"));
    let rust_alias = rust
        .map(str::to_string)
        .or_else(|| first_alias_for_lang(&registry.sources, "rust"));

    match (java_alias, rust_alias) {
        (Some(java), Some(rust)) => {
            println!("Suggesting correspondences {java} → {rust}...");
            map::run_suggest(&java, &rust)?;
            println!("Rendering diff report...");
            diff::run(&java, &rust, out)?;
            let findings = ArtifactId::from_content(out.as_bytes());
            events.emit(EventKind::AnalysisCompleted { findings }, None);
        },
        _ => {
            println!(
                "Skipping map/diff (need one Java and one Rust source, or pass --java/--rust)."
            );
        },
    }

    println!(
        "Analyze complete ({} events). Maturity: {}.",
        events.len(),
        s4_core::MATURITY
    );
    Ok(())
}

fn first_alias_for_lang(sources: &[s4_project::SourceRef], lang: &str) -> Option<String> {
    sources.iter().find_map(|s| {
        if s.language == LanguageId(lang.to_string()) {
            Some(s.alias.clone())
        } else {
            None
        }
    })
}
