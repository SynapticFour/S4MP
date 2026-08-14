//! Requirements CRUD, `OpenAPI` import, and name-based trace suggestions.

use crate::workspace::{load_graph_from_store, Workspace};
use s4_core::{Result, S4Error};
use s4_graph::{GraphView, NodeKind};
use s4_requirements::{RequirementKind, RequirementsDocument, TraceLinkKind, TraceabilityGraph};
use std::path::PathBuf;

fn requirements_path(ws: &Workspace) -> PathBuf {
    ws.root().join(".s4").join("requirements.json")
}

/// Add a requirement.
pub fn run_add(kind: &str, statement: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.ensure_layout()?;
    let path = requirements_path(&ws);
    let mut doc = RequirementsDocument::load(&path)?;
    let kind = parse_kind(kind)?;
    let id = doc.add(kind, statement);
    doc.save(&path)?;
    println!("added requirement {} — {statement}", id.0);
    Ok(())
}

/// List requirements and traces.
pub fn run_list() -> Result<()> {
    let ws = Workspace::open(".")?;
    let path = requirements_path(&ws);
    let doc = RequirementsDocument::load(&path)?;
    println!("requirements ({}):", doc.requirements.len());
    for req in doc.requirements.values() {
        println!("  [{}] {:?} — {}", req.id.0, req.kind, req.statement);
        for link in doc.traces_from(req.id)? {
            println!("    trace {:?} → entity {}", link.kind, link.target);
        }
    }
    Ok(())
}

/// Import `OpenAPI` paths as functional requirements.
pub fn run_import_openapi(openapi: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.ensure_layout()?;
    let path = requirements_path(&ws);
    let mut doc = RequirementsDocument::load(&path)?;
    let count = doc.import_openapi_paths(std::path::Path::new(openapi))?;
    doc.save(&path)?;
    println!("imported {count} OpenAPI path requirement(s)");
    Ok(())
}

/// Suggest and optionally confirm name-based traces for a source graph.
pub fn run_trace_suggest(source: &str, apply: bool) -> Result<()> {
    let ws = Workspace::open(".")?;
    let path = requirements_path(&ws);
    let mut doc = RequirementsDocument::load(&path)?;
    let manifest = ws.load_graph_manifest(source)?;
    let store = ws.store()?;
    let graph = load_graph_from_store(&store, &manifest.graph_artifact_id)?;

    let mut callables = Vec::new();
    for node in graph.nodes() {
        if node.kind == NodeKind::Callable {
            callables.push((node.id.as_entity_id(source), node.label.clone()));
        }
    }

    let suggestions = doc.suggest_traces_by_name(&callables);
    println!("suggested traces: {}", suggestions.len());
    for (req, entity, label) in &suggestions {
        println!("  req {} ↔ {label} (entity {entity})", req.0);
        if apply {
            doc.add_trace(*req, entity.clone(), TraceLinkKind::ImplementedBy)?;
        }
    }
    if apply {
        doc.save(&path)?;
        println!(
            "applied {} trace(s) to {}",
            suggestions.len(),
            path.display()
        );
    } else {
        println!("re-run with --apply to persist suggestions");
    }
    Ok(())
}

fn parse_kind(kind: &str) -> Result<RequirementKind> {
    match kind.to_ascii_lowercase().as_str() {
        "functional" | "fn" => Ok(RequirementKind::Functional),
        "non_functional" | "nfr" => Ok(RequirementKind::NonFunctional),
        "constraint" => Ok(RequirementKind::Constraint),
        "test" => Ok(RequirementKind::Test),
        other => Err(S4Error::InvalidInput(format!(
            "unknown requirement kind '{other}' (functional|non_functional|constraint|test)"
        ))),
    }
}
