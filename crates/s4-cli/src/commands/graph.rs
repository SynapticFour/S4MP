use crate::workspace::{
    count_nodes, discover_parse_units, load_usir_modules, save_graph_projection, Workspace,
};
use s4_analysis::usir_to_graph;
use s4_core::Result;
use s4_graph::NodeKind;
use s4_parser::plugins::{parse_all_sequential, JavaParser, RustParser};
use s4_parser::{LanguageId, ParseContext};
use s4_project::{DefaultSourceIngestor, snapshot_physical, SourceIngestor};
use s4_storage::StoreWriter;
use std::path::Path;

/// Build a semantic graph for a registered source alias.
pub fn run(source: &str, out_dir: &str) -> Result<()> {
    let ws = Workspace::open(".")?;
    let source_ref = ws.find_source(source)?;
    let ingestor = DefaultSourceIngestor::new(ws.root().to_path_buf());

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

    let units = discover_parse_units(&resolved.local_root, &source_ref.language)?;
    println!(
        "Parsing {} {} files...",
        units.len(),
        source_ref.language.0
    );

    let mut ctx = ParseContext {
        source_root: &resolved.local_root,
        store: &mut store,
    };
    let module_ids = parse_with_language(&source_ref.language, &units, &mut ctx)?;
    let modules = load_usir_modules(&store, &module_ids)?;

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
    println!(
        "  {node_count} nodes ({callable_nodes} callables, {type_nodes} types)"
    );

    let out_path = Path::new(out_dir);
    std::fs::create_dir_all(out_path).map_err(|e| {
        s4_core::S4Error::Other(format!(
            "failed to create output directory {}: {e}",
            out_path.display()
        ))
    })?;

    let manifest = crate::workspace::GraphManifest {
        source_alias: source.to_string(),
        graph_artifact_id: graph_id.to_string(),
        snapshot_artifact_id: snapshot_id.to_string(),
        files_parsed: units.len(),
        callable_count: callable_nodes,
        type_count: type_nodes,
        node_count,
    };

    let manifest_path = out_path.join(format!("{source}.json"));
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| {
        s4_core::S4Error::Other(format!("failed to serialize graph manifest: {e}"))
    })?;
    std::fs::write(&manifest_path, bytes).map_err(|e| {
        s4_core::S4Error::Other(format!(
            "failed to write graph manifest {}: {e}",
            manifest_path.display()
        ))
    })?;

    if out_path != ws.graphs_dir().as_path() {
        ws.save_graph_manifest(&manifest)?;
        println!(
            "Graph manifest also written to {}",
            ws.graph_manifest_path(source).display()
        );
    }

    println!(
        "Graph manifest written to {} (artifact {graph_id})",
        manifest_path.display()
    );
    Ok(())
}

fn parse_with_language(
    language: &LanguageId,
    units: &[s4_parser::ParseUnit],
    ctx: &mut ParseContext<'_>,
) -> Result<Vec<s4_core::ArtifactId>> {
    match language.0.as_str() {
        "java" => parse_all_sequential(&JavaParser, units, ctx),
        "rust" => parse_all_sequential(&RustParser, units, ctx),
        other => Err(s4_core::S4Error::Other(format!(
            "no parser registered for language '{other}'"
        ))),
    }
}
