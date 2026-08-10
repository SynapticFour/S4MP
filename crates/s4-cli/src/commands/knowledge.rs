//! SKG concept extraction from built graphs.

use crate::workspace::{load_graph_from_store, Workspace};
use s4_core::Result;
use s4_knowledge::{concepts_to_facts, extract_concepts_from_graph};

/// Extract naming concepts from a source graph and print them.
pub fn run_extract(source: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    let manifest = ws.load_graph_manifest(source)?;
    let store = ws.store()?;
    let graph = load_graph_from_store(&store, &manifest.graph_artifact_id)?;
    let concepts = extract_concepts_from_graph(&graph);
    let facts = concepts_to_facts(&concepts, &graph);
    println!(
        "extracted {} concept(s) / {} fact(s) from '{source}'",
        concepts.len(),
        facts.len()
    );
    for concept in &concepts {
        match concept.source_node {
            Some(id) => println!("  {} (node {id})", concept.name),
            None => println!("  {}", concept.name),
        }
    }
    let out = ws
        .root()
        .join(".s4")
        .join("knowledge")
        .join(format!("{source}-concepts.json"));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            s4_core::S4Error::Other(format!("failed to create {}: {e}", parent.display()))
        })?;
    }
    let bytes = serde_json::to_vec_pretty(&concepts)
        .map_err(|e| s4_core::S4Error::Other(format!("serialize concepts: {e}")))?;
    std::fs::write(&out, bytes)
        .map_err(|e| s4_core::S4Error::Other(format!("write {}: {e}", out.display())))?;
    println!("wrote {}", out.display());
    Ok(())
}
