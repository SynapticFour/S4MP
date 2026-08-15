//! Workspace paths, registry I/O, and shared pipeline helpers.

use s4_analysis::CorrespondenceEntry;
use s4_core::{ArtifactId, Result, S4Error, SchemaVersion, MATURITY};
use s4_graph::memory::InMemoryGraphView;
use s4_graph::{Edge, GraphView, Node, NodeKind};
use s4_parser::{LanguageId, ParseUnit};
use s4_project::{
    should_skip_snapshot_path, validate_git_ref, validate_git_subpath, validate_git_url,
    validate_source_alias, SourceOrigin, SourceRef,
};
use s4_storage::{Artifact, ArtifactKind, FileSystemStore, StoreReader, StoreWriter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Workspace root (typically the current directory).
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

/// On-disk source registry.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceRegistry {
    /// Registered sources keyed by alias order.
    pub sources: Vec<SourceRef>,
}

/// Workspace metadata written by `s4 init` (`.s4/workspace.json`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    /// Artifact / USIR schema version for this workspace.
    pub schema_version: SchemaVersion,
    /// Product maturity label (never stronger than shipped capabilities).
    pub maturity: String,
}

impl Default for WorkspaceMeta {
    fn default() -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            maturity: MATURITY.to_string(),
        }
    }
}

/// Manifest written by `s4 graph` under `.s4/graphs/<alias>.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphManifest {
    /// Source alias this graph was built from.
    pub source_alias: String,
    /// Content address of the [`ArtifactKind::GraphProjection`] payload.
    pub graph_artifact_id: String,
    /// Content address of the physical snapshot artifact.
    pub snapshot_artifact_id: String,
    /// Number of source files parsed.
    pub files_parsed: usize,
    /// Callable nodes in the graph.
    pub callable_count: usize,
    /// Type nodes in the graph.
    pub type_count: usize,
    /// Total nodes in the graph.
    pub node_count: usize,
}

/// Manifest for a Java↔Rust correspondence map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapManifest {
    /// Java source alias.
    pub java_source: String,
    /// Rust source alias.
    pub rust_source: String,
    /// Content address of the [`ArtifactKind::CorrespondenceMap`] artifact.
    pub artifact_id: String,
    /// Number of correspondence rows.
    pub entry_count: usize,
}

/// Serializable graph projection payload stored in the CAS.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphProjectionPayload {
    /// Source alias for traceability.
    pub source_alias: String,
    /// All nodes in the projection.
    pub nodes: Vec<Node>,
    /// All edges in the projection.
    pub edges: Vec<Edge>,
}

impl Workspace {
    /// Use `root` as the workspace directory (creates `.s4/` as needed).
    ///
    /// # Errors
    ///
    /// Returns an error if `.s4/` cannot be created.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join(".s4")).map_err(|e| {
            S4Error::Storage(format!(
                "failed to create workspace directory {}: {e}",
                root.join(".s4").display()
            ))
        })?;
        Ok(Self { root })
    }

    /// Workspace root path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to workspace metadata (`.s4/workspace.json`).
    #[must_use]
    pub fn meta_path(&self) -> PathBuf {
        self.root.join(".s4").join("workspace.json")
    }

    /// Ensure standard `.s4/` subdirectories exist.
    ///
    /// # Errors
    ///
    /// Returns an error if a directory cannot be created.
    pub fn ensure_layout(&self) -> Result<()> {
        for rel in [
            ".s4",
            ".s4/store",
            ".s4/cache",
            ".s4/graphs",
            ".s4/maps",
            ".s4/reports",
            ".s4/exports",
            ".s4/verification",
            ".s4/certificates",
            ".s4/knowledge",
            ".s4/proposals",
        ] {
            let path = self.root.join(rel);
            std::fs::create_dir_all(&path).map_err(|e| {
                S4Error::Storage(format!(
                    "failed to create workspace directory {}: {e}",
                    path.display()
                ))
            })?;
        }
        Ok(())
    }

    /// Load workspace metadata (defaults when missing).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load_meta(&self) -> Result<WorkspaceMeta> {
        let path = self.meta_path();
        if !path.is_file() {
            return Ok(WorkspaceMeta::default());
        }
        read_json(&path)
    }

    /// Persist workspace metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save_meta(&self, meta: &WorkspaceMeta) -> Result<()> {
        self.ensure_layout()?;
        write_json(&self.meta_path(), meta)
    }

    /// Open the file-backed artifact store.
    ///
    /// # Errors
    ///
    /// Returns an error if the store directory cannot be created.
    pub fn store(&self) -> Result<FileSystemStore> {
        FileSystemStore::workspace(&self.root)
    }

    /// Path to the source registry file.
    #[must_use]
    pub fn sources_path(&self) -> PathBuf {
        self.root.join(".s4").join("sources.json")
    }

    /// Path to graph manifests directory.
    #[must_use]
    pub fn graphs_dir(&self) -> PathBuf {
        self.root.join(".s4").join("graphs")
    }

    /// Path to a graph manifest for `alias`.
    #[must_use]
    pub fn graph_manifest_path(&self, alias: &str) -> PathBuf {
        self.graphs_dir().join(format!("{alias}.json"))
    }

    /// Path to correspondence map manifests directory.
    #[must_use]
    pub fn maps_dir(&self) -> PathBuf {
        self.root.join(".s4").join("maps")
    }

    /// Path to a map manifest for a Java/Rust alias pair.
    #[must_use]
    pub fn map_manifest_path(&self, java: &str, rust: &str) -> PathBuf {
        self.maps_dir().join(format!("{java}__{rust}.json"))
    }

    /// Load the source registry (empty when missing).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load_sources(&self) -> Result<SourceRegistry> {
        let path = self.sources_path();
        if !path.is_file() {
            return Ok(SourceRegistry::default());
        }
        let bytes = std::fs::read(&path)
            .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", path.display())))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| S4Error::Storage(format!("failed to parse {}: {e}", path.display())))
    }

    /// Persist the source registry.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    pub fn save_sources(&self, registry: &SourceRegistry) -> Result<()> {
        let path = self.sources_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                S4Error::Storage(format!(
                    "failed to create sources directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        let bytes = serde_json::to_vec_pretty(registry)
            .map_err(|e| S4Error::Storage(format!("failed to serialize source registry: {e}")))?;
        std::fs::write(&path, bytes)
            .map_err(|e| S4Error::Storage(format!("failed to write {}: {e}", path.display())))
    }

    /// Look up a registered source by alias.
    ///
    /// # Errors
    ///
    /// Returns an error when the alias is unknown.
    pub fn find_source(&self, alias: &str) -> Result<SourceRef> {
        let registry = self.load_sources()?;
        registry
            .sources
            .into_iter()
            .find(|s| s.alias == alias)
            .ok_or_else(|| S4Error::InvalidInput(format!("unknown source alias: {alias}")))
    }

    /// Load a graph manifest for `alias`.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest is missing or invalid.
    pub fn load_graph_manifest(&self, alias: &str) -> Result<GraphManifest> {
        let path = self.graph_manifest_path(alias);
        read_json(&path).map_err(|e| {
            S4Error::Storage(format!(
                "graph manifest for '{alias}' not found ({}): {e}",
                path.display()
            ))
        })
    }

    /// Save a graph manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save_graph_manifest(&self, manifest: &GraphManifest) -> Result<()> {
        std::fs::create_dir_all(self.graphs_dir()).map_err(|e| {
            S4Error::Storage(format!(
                "failed to create graphs directory {}: {e}",
                self.graphs_dir().display()
            ))
        })?;
        write_json(&self.graph_manifest_path(&manifest.source_alias), manifest)
    }

    /// Load a map manifest for a Java/Rust pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest is missing or invalid.
    pub fn load_map_manifest(&self, java: &str, rust: &str) -> Result<MapManifest> {
        let path = self.map_manifest_path(java, rust);
        read_json(&path).map_err(|e| {
            S4Error::Storage(format!(
                "correspondence map for '{java}' -> '{rust}' not found ({}): {e}",
                path.display()
            ))
        })
    }

    /// Save a map manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save_map_manifest(&self, manifest: &MapManifest) -> Result<()> {
        std::fs::create_dir_all(self.maps_dir()).map_err(|e| {
            S4Error::Storage(format!(
                "failed to create maps directory {}: {e}",
                self.maps_dir().display()
            ))
        })?;
        write_json(
            &self.map_manifest_path(&manifest.java_source, &manifest.rust_source),
            manifest,
        )
    }

    /// Load all map manifests in the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if a manifest file cannot be read or parsed.
    pub fn load_all_map_manifests(&self) -> Result<Vec<MapManifest>> {
        let dir = self.maps_dir();
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut manifests: Vec<MapManifest> = Vec::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| {
            S4Error::Storage(format!(
                "failed to read maps directory {}: {e}",
                dir.display()
            ))
        })? {
            let entry = entry.map_err(|e| {
                S4Error::Storage(format!("failed to read maps directory entry: {e}"))
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                manifests.push(read_json(&path)?);
            }
        }
        manifests.sort_by(|a, b| {
            (&a.java_source, &a.rust_source).cmp(&(&b.java_source, &b.rust_source))
        });
        Ok(manifests)
    }
}

/// Parse a CLI language flag into [`LanguageId`].
///
/// # Errors
///
/// Returns an error for unsupported language names.
pub fn parse_language(lang: &str) -> Result<LanguageId> {
    match lang.to_ascii_lowercase().as_str() {
        "java" => Ok(LanguageId("java".to_string())),
        "rust" => Ok(LanguageId("rust".to_string())),
        other => Err(S4Error::InvalidInput(format!(
            "unsupported language '{other}' (expected 'java' or 'rust')"
        ))),
    }
}

/// Build a [`SourceRef`] from CLI flags.
///
/// # Errors
///
/// Returns an error when neither `git` nor `local` is set, or both are set.
pub fn source_ref_from_flags(
    alias: &str,
    git: Option<&str>,
    local: Option<&str>,
    git_ref: Option<&str>,
    subpath: Option<&str>,
    lang: &str,
) -> Result<SourceRef> {
    let language = parse_language(lang)?;
    validate_source_alias(alias)?;
    if let Some(sub) = subpath {
        validate_git_subpath(sub)?;
    }
    let origin = match (git, local) {
        (Some(url), None) => {
            validate_git_url(url)?;
            if let Some(reference) = git_ref {
                validate_git_ref(reference)?;
            }
            SourceOrigin::Git {
                url: url.to_string(),
                git_ref: git_ref.map(str::to_string),
                subpath: subpath.map(str::to_string),
            }
        },
        (None, Some(path)) => SourceOrigin::Local {
            path: PathBuf::from(path),
        },
        (Some(_), Some(_)) => {
            return Err(S4Error::InvalidInput(
                "specify either --git or --local, not both".to_string(),
            ));
        },
        (None, None) => {
            return Err(S4Error::InvalidInput(
                "one of --git or --local is required".to_string(),
            ));
        },
    };
    Ok(SourceRef {
        alias: alias.to_string(),
        language,
        origin,
    })
}

/// Discover parseable source files under `root` for `language`.
///
/// When `hashes` contains a snapshot-relative unix path, the matching unit
/// carries `source_hash` so the parser can reuse cached USIR artifacts.
pub fn discover_parse_units(
    root: &Path,
    language: &LanguageId,
    hashes: &HashMap<String, String>,
) -> Result<Vec<ParseUnit>> {
    let extension = match language.0.as_str() {
        "java" => "java",
        "rust" => "rs",
        other => {
            return Err(S4Error::InvalidInput(format!(
                "no file extension mapping for language '{other}'"
            )));
        },
    };

    let mut units = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip_snapshot_path(e.path()))
    {
        let entry = entry
            .map_err(|e| S4Error::Storage(format!("failed to walk {}: {e}", root.display())))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == extension) {
            let relative = path.strip_prefix(root).ok().map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/")
            });
            let source_hash = relative.and_then(|rel| hashes.get(&rel).cloned());
            units.push(ParseUnit {
                path: path.to_string_lossy().into_owned(),
                language: language.clone(),
                content: None,
                source_text: None,
                source_hash,
            });
        }
    }
    units.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(units)
}

/// Serialize a [`GraphView`] into a storable payload.
#[must_use]
pub fn graph_to_payload(source_alias: &str, graph: &dyn GraphView) -> GraphProjectionPayload {
    GraphProjectionPayload {
        source_alias: source_alias.to_string(),
        nodes: graph.nodes().cloned().collect(),
        edges: graph.edges().cloned().collect(),
    }
}

/// Reconstruct an in-memory graph view from a payload.
#[must_use]
pub fn graph_from_payload(payload: GraphProjectionPayload) -> InMemoryGraphView {
    InMemoryGraphView::new(payload.nodes, payload.edges)
}

/// Persist a graph projection artifact.
///
/// # Errors
///
/// Returns an error if serialization or storage fails.
pub fn save_graph_projection(
    store: &mut dyn StoreWriter,
    source_alias: &str,
    graph: &dyn GraphView,
) -> Result<ArtifactId> {
    let payload = graph_to_payload(source_alias, graph);
    let value = serde_json::to_value(&payload)
        .map_err(|e| S4Error::Storage(format!("failed to serialize graph projection: {e}")))?;
    let artifact = Artifact {
        kind: ArtifactKind::GraphProjection,
        schema_version: SchemaVersion::CURRENT,
        payload: value,
    };
    store.write(&artifact)
}

/// Load a graph view from a graph projection artifact id string.
///
/// # Errors
///
/// Returns an error if the artifact is missing or not a graph projection.
pub fn load_graph_from_store(store: &dyn StoreReader, id: &str) -> Result<InMemoryGraphView> {
    let artifact_id = parse_artifact_id(id)?;
    let artifact = store
        .read(&artifact_id)?
        .ok_or_else(|| S4Error::Storage(format!("graph artifact not found: {id}")))?;
    artifact.expect_current_schema()?;
    if artifact.kind != ArtifactKind::GraphProjection {
        return Err(S4Error::Storage(format!(
            "expected graph_projection artifact, got {:?}",
            artifact.kind
        )));
    }
    let payload: GraphProjectionPayload = serde_json::from_value(artifact.payload)
        .map_err(|e| S4Error::Storage(format!("failed to deserialize graph projection: {e}")))?;
    Ok(graph_from_payload(payload))
}

/// Count nodes of a given kind in a graph view.
pub fn count_nodes(graph: &dyn GraphView, kind: &NodeKind) -> usize {
    graph.nodes().filter(|node| &node.kind == kind).count()
}

/// Parse a hex artifact id string.
///
/// # Errors
///
/// Returns an error when the string is not 64 hex characters.
pub fn parse_artifact_id(id: &str) -> Result<ArtifactId> {
    id.parse()
}

/// Load correspondence entries from a map manifest artifact id.
///
/// # Errors
///
/// Returns an error if loading fails.
pub fn load_correspondence_entries(
    store: &dyn StoreReader,
    manifest: &MapManifest,
) -> Result<Vec<CorrespondenceEntry>> {
    let id = parse_artifact_id(&manifest.artifact_id)?;
    s4_analysis::load_correspondence_map(store, &id)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path)
        .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| S4Error::Storage(format!("failed to parse {}: {e}", path.display())))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            S4Error::Storage(format!(
                "failed to create directory {}: {e}",
                parent.display()
            ))
        })?;
    }
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| S4Error::Storage(format!("failed to serialize {}: {e}", path.display())))?;
    std::fs::write(path, bytes)
        .map_err(|e| S4Error::Storage(format!("failed to write {}: {e}", path.display())))
}
