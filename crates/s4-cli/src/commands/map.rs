use crate::workspace::{load_correspondence_entries, load_graph_from_store, Workspace};
use s4_analysis::{
    entries_matching_id, entries_matching_name, merge_correspondences, save_correspondence_map,
    short_entry_id, suggest_correspondences, CorrespondenceEntry, CorrespondenceMethod,
    CorrespondenceStatus, GraphId,
};
use s4_core::{Result, S4Error};
use std::fmt::Write as _;

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

    let existing = if ws.map_manifest_path(java, rust).is_file() {
        load_existing_entries(&ws, java, rust)?
    } else {
        Vec::new()
    };
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
    println!("Next: s4 map show --java {java} --rust {rust}");
    println!("Then: s4 map confirm --name <symbol>   # or --id from the table");
    Ok(())
}

/// Print correspondence rows with short ids, pairings, and signatures.
pub fn run_show(java: Option<&str>, rust: Option<&str>, status: Option<&str>) -> Result<()> {
    let ws = Workspace::open(".")?;
    let filter = parse_status_filter(status)?;
    let manifests = selected_manifests(&ws, java, rust)?;
    if manifests.is_empty() {
        println!("No correspondence maps. Use `s4 map suggest` first.");
        return Ok(());
    }

    let store = ws.store()?;
    for manifest in &manifests {
        let mut entries = load_correspondence_entries(&store, manifest)?;
        if let Some(wanted) = filter {
            entries.retain(|e| e.status == wanted);
        }
        entries.sort_by(|a, b| {
            status_rank(a.status)
                .cmp(&status_rank(b.status))
                .then_with(|| {
                    a.display_name
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.display_name.as_deref().unwrap_or(""))
                })
        });

        println!(
            "{} -> {}  ({} rows{})",
            manifest.java_source,
            manifest.rust_source,
            entries.len(),
            filter.map_or(String::new(), |s| format!(", status {s:?}"))
        );
        if entries.is_empty() {
            println!("  (none)");
            continue;
        }
        println!("  {:<12} {:<10} {:>6}  PAIR / NAME", "ID", "STATUS", "CONF");
        for entry in &entries {
            println!(
                "  {:<12} {:<10} {:>6.2}  {}",
                short_entry_id(&entry.id),
                status_token(entry.status),
                entry.confidence,
                entry.display_name.as_deref().unwrap_or(&entry.id)
            );
            if let Some(sig) = entry.source_signature.as_deref() {
                println!("               java  {sig}");
            }
            if let Some(sig) = entry.target_signature.as_deref() {
                println!("               rust  {sig}");
            }
        }
        println!();
        println!(
            "Confirm a diverged pair:  s4 map confirm --id {} --java {} --rust {}",
            short_entry_id(&entries[0].id),
            manifest.java_source,
            manifest.rust_source
        );
        println!(
            "                     or:  s4 map confirm --name <symbol> --java {} --rust {}",
            manifest.java_source, manifest.rust_source
        );
    }
    Ok(())
}

/// Confirm a suggested correspondence as manually ported.
pub fn run_confirm(
    id: Option<&str>,
    name: Option<&str>,
    java: Option<&str>,
    rust: Option<&str>,
) -> Result<()> {
    let ws = Workspace::open(".")?;
    let (manifest, mut entries, index) = resolve_row(&ws, id, name, java, rust)?;
    let entry = &mut entries[index];
    match entry.status {
        CorrespondenceStatus::ExtraInTarget => {
            return Err(S4Error::InvalidInput(format!(
                "cannot confirm extra-in-target row {} ({}); extras have no Java counterpart",
                short_entry_id(&entry.id),
                entry.display_name.as_deref().unwrap_or("unnamed")
            )));
        },
        CorrespondenceStatus::MissingInTarget => {
            return Err(S4Error::InvalidInput(format!(
                "cannot confirm missing-in-target row {} ({}); there is no Rust pair to confirm",
                short_entry_id(&entry.id),
                entry.display_name.as_deref().unwrap_or("unnamed")
            )));
        },
        CorrespondenceStatus::Ported
        | CorrespondenceStatus::Diverged
        | CorrespondenceStatus::Unmapped => {},
    }
    if entry.target_node.is_none() {
        return Err(S4Error::InvalidInput(format!(
            "cannot confirm {} ({}); row has no Rust target node",
            short_entry_id(&entry.id),
            entry.display_name.as_deref().unwrap_or("unnamed")
        )));
    }
    entry.status = CorrespondenceStatus::Ported;
    entry.method = CorrespondenceMethod::Manual;
    entry.confidence = 1.0;
    entry.stale = false;
    entry.note = Some("manually confirmed via s4 map confirm".to_string());
    let display = entry
        .display_name
        .clone()
        .unwrap_or_else(|| entry.id.clone());
    let short = short_entry_id(&entry.id).to_string();
    persist_map(&ws, &manifest, &entries)?;
    let ported = entries
        .iter()
        .filter(|e| e.status == CorrespondenceStatus::Ported)
        .count();
    let diverged = entries
        .iter()
        .filter(|e| e.status == CorrespondenceStatus::Diverged)
        .count();
    println!("Confirmed {display} (`id={short}`) as Ported.");
    println!(
        "Map {} -> {}: {ported} ported, {diverged} diverged remaining.",
        manifest.java_source, manifest.rust_source
    );
    println!(
        "Next: s4 diff --java {} --rust {} && s4 verify --java {} --rust {} && s4 certify --java {} --rust {}",
        manifest.java_source,
        manifest.rust_source,
        manifest.java_source,
        manifest.rust_source,
        manifest.java_source,
        manifest.rust_source
    );
    Ok(())
}

/// Reject a suggested correspondence.
pub fn run_reject(
    id: Option<&str>,
    name: Option<&str>,
    java: Option<&str>,
    rust: Option<&str>,
) -> Result<()> {
    let ws = Workspace::open(".")?;
    let (manifest, mut entries, index) = resolve_row(&ws, id, name, java, rust)?;
    let removed = entries.remove(index);
    let short = short_entry_id(&removed.id).to_string();
    let display = removed
        .display_name
        .clone()
        .unwrap_or_else(|| removed.id.clone());

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
                display_name: removed.display_name,
                source_signature: removed.source_signature,
                target_signature: None,
                stale: false,
            });
        }
    }

    persist_map(&ws, &manifest, &entries)?;
    println!("Rejected {display} (`id={short}`)");
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
        println!(
            "    review rows: s4 map show --java {} --rust {}",
            manifest.java_source, manifest.rust_source
        );
    }
    Ok(())
}

fn resolve_row(
    ws: &Workspace,
    id: Option<&str>,
    name: Option<&str>,
    java: Option<&str>,
    rust: Option<&str>,
) -> Result<(
    crate::workspace::MapManifest,
    Vec<CorrespondenceEntry>,
    usize,
)> {
    let query_id = id.map(str::trim).filter(|s| !s.is_empty());
    let query_name = name.map(str::trim).filter(|s| !s.is_empty());
    match (query_id, query_name) {
        (None, None) => {
            return Err(S4Error::InvalidInput(
                "provide --id <prefix> or --name <symbol>".into(),
            ));
        },
        (Some(_), Some(_)) => {
            return Err(S4Error::InvalidInput(
                "provide either --id or --name, not both".into(),
            ));
        },
        _ => {},
    }

    let manifests = selected_manifests(ws, java, rust)?;
    if manifests.is_empty() {
        return Err(S4Error::InvalidInput(
            "no correspondence maps in this workspace — run `s4 map suggest` first".into(),
        ));
    }

    let store = ws.store()?;
    let mut hits: Vec<(
        crate::workspace::MapManifest,
        Vec<CorrespondenceEntry>,
        Vec<usize>,
    )> = Vec::new();
    for manifest in manifests {
        let entries = load_correspondence_entries(&store, &manifest)?;
        let matched: Vec<usize> = if let Some(q) = query_id {
            entries_matching_id(&entries, q)
                .into_iter()
                .filter_map(|hit| entries.iter().position(|e| e.id == hit.id))
                .collect()
        } else if let Some(q) = query_name {
            entries_matching_name(&entries, q)
                .into_iter()
                .filter_map(|hit| entries.iter().position(|e| e.id == hit.id))
                .collect()
        } else {
            Vec::new()
        };
        if !matched.is_empty() {
            hits.push((manifest, entries, matched));
        }
    }

    let total: usize = hits.iter().map(|(_, _, idx)| idx.len()).sum();
    match total {
        0 => Err(S4Error::InvalidId(format!(
            "no correspondence matches {} — run `s4 map show`{}",
            selector_label(query_id, query_name),
            java_rust_hint(java, rust)
        ))),
        1 => {
            let (manifest, entries, indices) = hits.remove(0);
            Ok((manifest, entries, indices[0]))
        },
        _ => {
            let mut detail = String::new();
            for (manifest, entries, indices) in &hits {
                for &index in indices {
                    let entry = &entries[index];
                    let _ = write!(
                        detail,
                        "\n  {} -> {}  id={}  {}  {:?}",
                        manifest.java_source,
                        manifest.rust_source,
                        short_entry_id(&entry.id),
                        entry.display_name.as_deref().unwrap_or(&entry.id),
                        entry.status
                    );
                }
            }
            Err(S4Error::InvalidInput(format!(
                "ambiguous {} — matches:{detail}\nUse a longer --id prefix, or pass --java/--rust to scope the map.",
                selector_label(query_id, query_name),
            )))
        },
    }
}

fn selected_manifests(
    ws: &Workspace,
    java: Option<&str>,
    rust: Option<&str>,
) -> Result<Vec<crate::workspace::MapManifest>> {
    match (java, rust) {
        (Some(j), Some(r)) => Ok(vec![ws.load_map_manifest(j, r)?]),
        (Some(_), None) | (None, Some(_)) => Err(S4Error::InvalidInput(
            "pass both --java and --rust, or neither (all maps)".into(),
        )),
        (None, None) => ws.load_all_map_manifests(),
    }
}

fn parse_status_filter(status: Option<&str>) -> Result<Option<CorrespondenceStatus>> {
    let Some(raw) = status.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let parsed = match raw.to_ascii_lowercase().as_str() {
        "ported" => CorrespondenceStatus::Ported,
        "diverged" => CorrespondenceStatus::Diverged,
        "missing" | "missing_in_target" => CorrespondenceStatus::MissingInTarget,
        "extra" | "extra_in_target" => CorrespondenceStatus::ExtraInTarget,
        "unmapped" => CorrespondenceStatus::Unmapped,
        other => {
            return Err(S4Error::InvalidInput(format!(
                "unknown --status '{other}' (ported|diverged|missing|extra|unmapped)"
            )));
        },
    };
    Ok(Some(parsed))
}

fn status_token(status: CorrespondenceStatus) -> &'static str {
    match status {
        CorrespondenceStatus::Ported => "ported",
        CorrespondenceStatus::Diverged => "diverged",
        CorrespondenceStatus::MissingInTarget => "missing",
        CorrespondenceStatus::ExtraInTarget => "extra",
        CorrespondenceStatus::Unmapped => "unmapped",
    }
}

fn status_rank(status: CorrespondenceStatus) -> u8 {
    match status {
        CorrespondenceStatus::Diverged => 0,
        CorrespondenceStatus::MissingInTarget => 1,
        CorrespondenceStatus::ExtraInTarget => 2,
        CorrespondenceStatus::Unmapped => 3,
        CorrespondenceStatus::Ported => 4,
    }
}

fn selector_label(id: Option<&str>, name: Option<&str>) -> String {
    if let Some(id) = id {
        format!("--id {id}")
    } else if let Some(name) = name {
        format!("--name {name}")
    } else {
        "selector".into()
    }
}

fn java_rust_hint(java: Option<&str>, rust: Option<&str>) -> String {
    match (java, rust) {
        (Some(j), Some(r)) => format!(" --java {j} --rust {r}"),
        _ => String::new(),
    }
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
