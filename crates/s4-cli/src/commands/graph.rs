use crate::graph_export::GraphExportFormat;
use crate::workspace::{count_nodes, discover_parse_units, save_graph_projection, Workspace};
use s4_analysis::usir_to_graph;
use s4_core::Result;
use s4_graph::NodeKind;
use s4_parser::plugins::{extract_for_language, parse_all_parallel, ParsedModules};
use s4_parser::{LanguageId, ParseContext, ParseUnit};
use s4_project::{snapshot_path_hashes, snapshot_physical, DefaultSourceIngestor, SourceIngestor};
use s4_storage::StoreWriter;
use std::path::{Path, PathBuf};

const DEFAULT_EXPORT_OUT: &str = ".s4/exports/graph";

/// Build a semantic graph for a registered source alias.
pub fn run_build(source: &str, out_dir: &str, force: bool, refresh: bool) -> Result<()> {
    let ws = Workspace::open(".")?;
    let source_ref = ws.find_source(source)?;
    let ingestor = DefaultSourceIngestor::new(ws.root().to_path_buf()).with_refresh(refresh);

    println!("Resolving source '{source}'...");
    let resolved = ingestor.resolve(&source_ref)?;
    println!("  root: {}", resolved.local_root.display());

    println!("Snapshotting source tree...");
    let snapshot = snapshot_physical(&resolved.local_root)?;
    let mut store = ws.store()?;
    let snapshot_id = store.write(&snapshot)?;
    let file_count = snapshot
        .payload
        .get("files")
        .and_then(|v| v.as_array())
        .map_or(0, std::vec::Vec::len);
    println!("  {file_count} files hashed (artifact {snapshot_id})");

    let out_path = Path::new(out_dir);
    std::fs::create_dir_all(out_path).map_err(|e| {
        s4_core::S4Error::Storage(format!(
            "failed to create output directory {}: {e}",
            out_path.display()
        ))
    })?;

    if !force {
        if let Ok(existing) = ws.load_graph_manifest(source) {
            if existing.snapshot_artifact_id == snapshot_id.to_string() {
                println!(
                    "Snapshot unchanged; reusing graph {} (pass --force to rebuild)",
                    existing.graph_artifact_id
                );
                write_graph_manifest(out_path, &ws, &existing)?;
                return Ok(());
            }
        }
    }

    let units = discover_parse_units(
        &resolved.local_root,
        &source_ref.language,
        &snapshot_path_hashes(&snapshot)?,
    )?;
    println!("Parsing {} {} files...", units.len(), source_ref.language.0);

    let mut ctx = ParseContext {
        source_root: &resolved.local_root,
        store: &mut store,
    };
    let parsed = parse_with_language(&source_ref.language, &units, &mut ctx)?;
    let modules = parsed.modules;

    let callable_count = modules
        .iter()
        .flat_map(|m| m.entities.iter())
        .filter(|e| matches!(e.kind, s4_parser::UsirEntityKind::Callable))
        .count();
    let type_count = modules
        .iter()
        .flat_map(|m| m.entities.iter())
        .filter(|e| matches!(e.kind, s4_parser::UsirEntityKind::Type))
        .count();
    println!(
        "Parsed {} files, {callable_count} callables, {type_count} types",
        units.len()
    );

    println!("Lowering USIR to graph...");
    let graph = usir_to_graph(&modules)?;
    let graph_id = save_graph_projection(&mut store, source, graph.as_ref())?;
    let node_count = s4_graph::GraphView::node_count(graph.as_ref());
    let callable_nodes = count_nodes(graph.as_ref(), &NodeKind::Callable);
    let type_nodes = count_nodes(graph.as_ref(), &NodeKind::Type);
    println!("  {node_count} nodes ({callable_nodes} callables, {type_nodes} types)");

    let manifest = crate::workspace::GraphManifest {
        source_alias: source.to_string(),
        graph_artifact_id: graph_id.to_string(),
        snapshot_artifact_id: snapshot_id.to_string(),
        files_parsed: units.len(),
        callable_count: callable_nodes,
        type_count: type_nodes,
        node_count,
    };

    write_graph_manifest(out_path, &ws, &manifest)?;
    println!(
        "Graph manifest written to {} (artifact {graph_id})",
        out_path.join(format!("{source}.json")).display()
    );
    Ok(())
}

fn write_graph_manifest(
    out_path: &Path,
    ws: &Workspace,
    manifest: &crate::workspace::GraphManifest,
) -> Result<()> {
    let manifest_path = out_path.join(format!("{}.json", manifest.source_alias));
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|e| {
        s4_core::S4Error::Storage(format!("failed to serialize graph manifest: {e}"))
    })?;
    std::fs::write(&manifest_path, bytes).map_err(|e| {
        s4_core::S4Error::Storage(format!(
            "failed to write graph manifest {}: {e}",
            manifest_path.display()
        ))
    })?;
    if out_path != ws.graphs_dir().as_path() {
        ws.save_graph_manifest(manifest)?;
        println!(
            "Graph manifest also written to {}",
            ws.graph_manifest_path(&manifest.source_alias).display()
        );
    }
    Ok(())
}

/// Export a built graph to DOT or JSON for visualization.
pub fn run_export(source: &str, format: &str, filter: &str, out: &str) -> Result<()> {
    use crate::graph_export::{parse_filter, render_export};
    use crate::workspace::{load_graph_from_store, Workspace};

    let ws = Workspace::open(".")?;
    ws.find_source(source)?;
    let manifest = ws.load_graph_manifest(source)?;

    println!("Loading graph for '{source}'...");
    let store = ws.store()?;
    let graph = load_graph_from_store(&store, &manifest.graph_artifact_id)?;
    let payload = crate::workspace::graph_to_payload(source, &graph);

    let export_format = GraphExportFormat::parse(format)?;
    let export_filter = parse_filter(filter);
    let rendered = render_export(&payload, &export_filter, export_format)?;
    let out_path = resolve_export_path(out, source, export_format);

    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                s4_core::S4Error::Storage(format!(
                    "failed to create output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }

    std::fs::write(&out_path, rendered).map_err(|e| {
        s4_core::S4Error::Storage(format!(
            "failed to write graph export {}: {e}",
            out_path.display()
        ))
    })?;

    println!(
        "Exported graph '{source}' to {} (format: {format}, filter: {filter})",
        out_path.display()
    );
    if export_format == GraphExportFormat::Dot {
        let svg_path = out_path.with_extension("svg");
        println!(
            "  Render SVG: dot -Tsvg {} -o {}",
            out_path.display(),
            svg_path.display()
        );
    }
    Ok(())
}

/// Diff two built graphs by `(kind, label)` identity.
pub fn run_diff(left: &str, right: &str) -> Result<()> {
    use crate::workspace::{load_graph_from_store, Workspace};
    use s4_graph::GraphDiff;

    let ws = Workspace::open(".")?;
    ws.find_source(left)?;
    ws.find_source(right)?;
    let left_manifest = ws.load_graph_manifest(left)?;
    let right_manifest = ws.load_graph_manifest(right)?;
    let store = ws.store()?;
    let left_graph = load_graph_from_store(&store, &left_manifest.graph_artifact_id)?;
    let right_graph = load_graph_from_store(&store, &right_manifest.graph_artifact_id)?;

    let diff = GraphDiff::from_views(&left_graph, &right_graph);
    println!("graph diff: {left} → {right}");
    println!("  shared: {}", diff.shared.len());
    println!("  only in {left}: {}", diff.only_left.len());
    for (kind, label) in &diff.only_left {
        println!("    - [{kind:?}] {label}");
    }
    println!("  only in {right}: {}", diff.only_right.len());
    for (kind, label) in &diff.only_right {
        println!("    - [{kind:?}] {label}");
    }
    println!(
        "  calls edges: {} vs {}",
        GraphDiff::call_edge_count(&left_graph),
        GraphDiff::call_edge_count(&right_graph)
    );
    Ok(())
}

fn resolve_export_path(out: &str, source: &str, format: GraphExportFormat) -> PathBuf {
    let ext = match format {
        GraphExportFormat::Dot => "dot",
        GraphExportFormat::Json => "json",
    };
    let path = Path::new(out);
    if path.extension().is_none() || out == DEFAULT_EXPORT_OUT {
        PathBuf::from(format!(".s4/exports/{source}.{ext}"))
    } else {
        path.to_path_buf()
    }
}

fn parse_with_language(
    language: &LanguageId,
    units: &[ParseUnit],
    ctx: &mut ParseContext<'_>,
) -> Result<ParsedModules> {
    parse_all_parallel(units, ctx.source_root, ctx.store, |unit, root| {
        let source = s4_parser::plugins::read_unit_source(unit)?;
        extract_for_language(&language.0, &source, &unit.path, root)
    })
}
