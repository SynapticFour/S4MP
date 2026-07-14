use crate::workspace::{
    load_correspondence_entries, load_graph_from_store, Workspace,
};
use s4_analysis::{
    merge_correspondences, save_correspondence_map, suggest_correspondences,
    CorrespondenceEntry, CorrespondenceMethod, CorrespondenceStatus, GraphId,
};
use s4_core::Result;

/// Suggest Java→Rust correspondences from built graphs.
pub fn run_suggest(java: &str, rust: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    ws.find_source(java)?;
    ws.find_source(rust)?;

    let java_manifest = ws.load_graph_manifest(java)?;
    let rust_manifest = ws.load_graph_manifest(rust)?;

    let store = ws.store()?;
    println!("Loading graphs for '{java}' and '{rust}'...");
    let java_graph = load_graph_from_store(&store, &java_manifest.graph_artifact_id)?;
    let rust_graph = load_graph_from_store(&store, &rust_manifest.graph_artifact_id)?;

    println!("Running name-heuristic correspondence suggestion...");
    let suggested = suggest_correspondences(
        &java_graph,
        &GraphId(java.to_string()),
        &rust_graph,
        &GraphId(rust.to_string()),
    );

    let above_threshold = suggested
        .iter()
        .filter(|e| e.confidence >= 0.5 && e.status == CorrespondenceStatus::Diverged)
        .count();
    let diverged = suggested
        .iter()
        .filter(|e| e.status == CorrespondenceStatus::Diverged)
        .count();
    let missing = suggested
        .iter()
        .filter(|e| e.status == CorrespondenceStatus::MissingInTarget)
        .count();
    let extra = suggested
        .iter()
        .filter(|e| e.status == CorrespondenceStatus::ExtraInTarget)
        .count();

    println!(
        "Suggested {} correspondences ({above_threshold} above 0.5 confidence, {diverged} diverged, {missing} missing, {extra} extra)",
        suggested.len()
    );

    let existing = load_existing_entries(&ws, java, rust).unwrap_or_default();
    let merged = merge_correspondences(existing, suggested);

    let mut store = ws.store()?;
    let artifact_id = save_correspondence_map(&mut store, &merged)?;
    let manifest = crate::workspace::MapManifest {
        java_source: java.to_string(),
        rust_source: rust.to_string(),
        artifact_id: artifact_id.to_string(),
        entry_count: merged.len(),
    };
    ws.save_map_manifest(&manifest)?;

    println!(
        "Saved correspondence map ({} entries, artifact {artifact_id})",
        merged.len()
    );
    Ok(())
}

/// Confirm a suggested correspondence as manually ported.
pub fn run_confirm(id: &str) -> Result<()> {
    update_entry(id, |entry| {
        entry.status = CorrespondenceStatus::Ported;
        entry.method = CorrespondenceMethod::Manual;
        entry.confidence = 1.0;
        entry.stale = false;
        entry.note = Some("manually confirmed via s4 map confirm".to_string());
    })
}

/// Reject a suggested correspondence.
pub fn run_reject(id: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    let (manifest, mut entries) = find_map_containing(&ws, id)?;
    let index = entries
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| s4_core::S4Error::Other(format!("correspondence id not found: {id}")))?;

    let removed = entries.remove(index);
    if let Some(source_node) = removed.source_node {
        if removed.status == CorrespondenceStatus::Diverged {
            entries.push(CorrespondenceEntry {
                id: removed.id,
                source_node: Some(source_node),
                target_node: None,
                status: CorrespondenceStatus::MissingInTarget,
                confidence: 0.0,
                method: CorrespondenceMethod::Manual,
                note: Some("rejected via s4 map reject".to_string()),
                stale: false,
            });
        }
    }

    persist_map(&ws, &manifest, &entries)?;
    println!("Rejected correspondence {id}");
    Ok(())
}

/// List correspondence maps in the workspace.
pub fn run_list() -> Result<()> {
    let ws = Workspace::open(".")?;
    let manifests = ws.load_all_map_manifests()?;
    if manifests.is_empty() {
        println!("No correspondence maps. Use `s4 map suggest` first.");
        return Ok(());
    }

    let store = ws.store()?;
    println!("Correspondence maps ({}):", manifests.len());
    for manifest in &manifests {
        let entries = load_correspondence_entries(&store, manifest)?;
        let ported = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::Ported)
            .count();
        let diverged = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::Diverged)
            .count();
        let missing = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::MissingInTarget)
            .count();
        let extra = entries
            .iter()
            .filter(|e| e.status == CorrespondenceStatus::ExtraInTarget)
            .count();
        println!(
            "  {} -> {}  ({} entries: {ported} ported, {diverged} diverged, {missing} missing, {extra} extra)  artifact {}",
            manifest.java_source,
            manifest.rust_source,
            entries.len(),
            manifest.artifact_id
        );
    }
    Ok(())
}

fn update_entry(id: &str, mutator: impl FnOnce(&mut CorrespondenceEntry)) -> Result<()> {
    let ws = Workspace::open(".")?;
    let (manifest, mut entries) = find_map_containing(&ws, id)?;
    let entry = entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| s4_core::S4Error::Other(format!("correspondence id not found: {id}")))?;
    mutator(entry);
    let status = entry.status;
    persist_map(&ws, &manifest, &entries)?;
    println!("Updated correspondence {id} -> {status:?}");
    Ok(())
}

fn find_map_containing(
    ws: &Workspace,
    id: &str,
) -> Result<(crate::workspace::MapManifest, Vec<CorrespondenceEntry>)> {
    let store = ws.store()?;
    for manifest in ws.load_all_map_manifests()? {
        let entries = load_correspondence_entries(&store, &manifest)?;
        if entries.iter().any(|e| e.id == id) {
            return Ok((manifest, entries));
        }
    }
    Err(s4_core::S4Error::Other(format!(
        "correspondence id not found in any map: {id}"
    )))
}

fn load_existing_entries(
    ws: &Workspace,
    java: &str,
    rust: &str,
) -> Result<Vec<CorrespondenceEntry>> {
    let manifest = ws.load_map_manifest(java, rust)?;
    let store = ws.store()?;
    load_correspondence_entries(&store, &manifest)
}

fn persist_map(
    ws: &Workspace,
    manifest: &crate::workspace::MapManifest,
    entries: &[CorrespondenceEntry],
) -> Result<()> {
    let mut store = ws.store()?;
    let artifact_id = save_correspondence_map(&mut store, entries)?;
    let updated = crate::workspace::MapManifest {
        java_source: manifest.java_source.clone(),
        rust_source: manifest.rust_source.clone(),
        artifact_id: artifact_id.to_string(),
        entry_count: entries.len(),
    };
    ws.save_map_manifest(&updated)
}
