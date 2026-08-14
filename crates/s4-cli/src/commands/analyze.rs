//! `s4 analyze` — run the Phase 2 porting pass pipeline over registered sources.

use crate::commands::{diff, graph, map};
use crate::workspace::{parse_artifact_id, Workspace};
use s4_analysis::{Pass, PassContext, PassOutcome, PassPipeline};
use s4_core::{ArtifactId, Result, S4Error};
use s4_events::{EventKind, RecordingEventSink};
use s4_parser::LanguageId;

/// Run graph builds for all sources, then optional map+diff for a Java/Rust pair.
pub fn run(
    java: Option<&str>,
    rust: Option<&str>,
    out: &str,
    force: bool,
    refresh: bool,
) -> Result<()> {
    let ws = Workspace::open(".")?;
    let registry = ws.load_sources()?;
    if registry.sources.is_empty() {
        return Err(S4Error::InvalidInput(
            "no sources registered — run `s4 init` and `s4 source add` first".to_string(),
        ));
    }

    let events = RecordingEventSink::new();
    let mut pipeline = PassPipeline::new();
    for source in &registry.sources {
        pipeline = pipeline.then(GraphBuildPass {
            alias: source.alias.clone(),
            force,
            refresh,
        });
    }

    let java_alias = java
        .map(str::to_string)
        .or_else(|| first_alias_for_lang(&registry.sources, "java"));
    let rust_alias = rust
        .map(str::to_string)
        .or_else(|| first_alias_for_lang(&registry.sources, "rust"));

    match (java_alias, rust_alias) {
        (Some(java), Some(rust)) => {
            pipeline = pipeline
                .then(SuggestMapPass {
                    java: java.clone(),
                    rust: rust.clone(),
                })
                .then(DiffReportPass {
                    java,
                    rust,
                    out: out.to_string(),
                });
        },
        _ => {
            println!(
                "Skipping map/diff (need one Java and one Rust source, or pass --java/--rust)."
            );
        },
    }

    println!("Pass order: {}", pipeline.names().join(" → "));
    let mut ctx = PassContext::new(Some(&events));
    pipeline.run(&mut ctx)?;
    for note in &ctx.notes {
        println!("{note}");
    }

    println!(
        "Analyze complete ({} events). Maturity: {}.",
        events.len(),
        s4_core::MATURITY
    );
    Ok(())
}

struct GraphBuildPass {
    alias: String,
    force: bool,
    refresh: bool,
}

impl Pass for GraphBuildPass {
    fn name(&self) -> &'static str {
        "graph_build"
    }

    fn run(&self, ctx: &mut PassContext<'_>) -> Result<PassOutcome> {
        graph::run_build(&self.alias, ".s4/graphs", self.force, self.refresh)?;
        let ws = Workspace::open(".")?;
        if let Ok(manifest) = ws.load_graph_manifest(&self.alias) {
            if let Ok(id) = parse_artifact_id(&manifest.graph_artifact_id) {
                ctx.emit(EventKind::GraphUpdated { projection: id });
                let mut artifacts = std::collections::BTreeMap::new();
                artifacts.insert(format!("graph:{}", self.alias), id);
                return Ok(PassOutcome {
                    notes: vec![format!("graph_build {}", self.alias)],
                    artifacts,
                });
            }
        }
        Ok(PassOutcome {
            notes: vec![format!("graph_build {}", self.alias)],
            artifacts: std::collections::BTreeMap::new(),
        })
    }
}

struct SuggestMapPass {
    java: String,
    rust: String,
}

impl Pass for SuggestMapPass {
    fn name(&self) -> &'static str {
        "suggest_map"
    }

    fn run(&self, _ctx: &mut PassContext<'_>) -> Result<PassOutcome> {
        map::run_suggest(&self.java, &self.rust)?;
        Ok(PassOutcome {
            notes: vec![format!("suggest_map {} → {}", self.java, self.rust)],
            artifacts: std::collections::BTreeMap::new(),
        })
    }
}

struct DiffReportPass {
    java: String,
    rust: String,
    out: String,
}

impl Pass for DiffReportPass {
    fn name(&self) -> &'static str {
        "diff_report"
    }

    fn run(&self, ctx: &mut PassContext<'_>) -> Result<PassOutcome> {
        diff::run(&self.java, &self.rust, &self.out)?;
        let findings = std::fs::read(&self.out).map_or_else(
            |_| ArtifactId::from_content(self.out.as_bytes()),
            |bytes| ArtifactId::from_content(&bytes),
        );
        ctx.emit(EventKind::AnalysisCompleted { findings });
        Ok(PassOutcome {
            notes: vec![format!("diff_report {}", self.out)],
            artifacts: std::collections::BTreeMap::new(),
        })
    }
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
