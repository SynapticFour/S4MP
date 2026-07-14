use crate::source::{SourceOrigin, SourceRef};
use s4_core::{Result, S4Error, SchemaVersion};
use s4_storage::{Artifact, ArtifactKind};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Result of resolving a [`SourceRef`] to a local directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSource {
    /// User-assigned alias copied from the source reference.
    pub alias: String,
    /// Local filesystem root for parsing (may be a subdirectory of a clone).
    pub local_root: PathBuf,
    /// Git commit SHA at `HEAD` when resolved from a remote repository.
    pub commit: Option<String>,
}

/// Resolves [`SourceRef`] definitions to local directory roots.
pub trait SourceIngestor: Send + Sync {
    /// Materialize the source at a local path ready for snapshotting or parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be resolved (missing path, git failure, etc.).
    fn resolve(&self, source: &SourceRef) -> Result<ResolvedSource>;
}

/// Default ingestor: local paths as-is, Git sources cloned into `.s4/cache/<alias>`.
#[derive(Clone, Debug)]
pub struct DefaultSourceIngestor {
    workspace_root: PathBuf,
}

impl DefaultSourceIngestor {
    /// Create an ingestor that caches Git clones under `<workspace_root>/.s4/cache/`.
    #[must_use]
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn resolve_local(alias: &str, path: &Path) -> Result<ResolvedSource> {
        if !path.exists() {
            return Err(S4Error::Other(format!(
                "local source path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(S4Error::Other(format!(
                "local source path is not a directory: {}",
                path.display()
            )));
        }
        Ok(ResolvedSource {
            alias: alias.to_string(),
            local_root: path.to_path_buf(),
            commit: None,
        })
    }

    fn resolve_git(
        &self,
        alias: &str,
        url: &str,
        git_ref: Option<&str>,
        subpath: Option<&str>,
    ) -> Result<ResolvedSource> {
        let cache_path = self.workspace_root.join(".s4").join("cache").join(alias);

        if cache_path.exists() {
            run_git(&["fetch"], &cache_path)?;
            if let Some(reference) = git_ref {
                run_git(&["checkout", reference], &cache_path)?;
            }
        } else {
            if let Some(parent) = cache_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    S4Error::Other(format!(
                        "failed to create cache directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }

            let dest = cache_path.to_string_lossy();
            let mut args = vec!["clone", "--depth", "1"];
            if let Some(reference) = git_ref {
                args.push("--branch");
                args.push(reference);
            }
            args.push(url);
            args.push(dest.as_ref());
            run_git(&args, self.workspace_root.as_path())?;
        }

        let local_root = match subpath {
            Some(sub) => cache_path.join(sub),
            None => cache_path.clone(),
        };

        if !local_root.is_dir() {
            return Err(S4Error::Other(format!(
                "resolved git source path is not a directory: {}",
                local_root.display()
            )));
        }

        Ok(ResolvedSource {
            alias: alias.to_string(),
            local_root,
            commit: git_head_commit(&cache_path),
        })
    }
}

impl SourceIngestor for DefaultSourceIngestor {
    fn resolve(&self, source: &SourceRef) -> Result<ResolvedSource> {
        match &source.origin {
            SourceOrigin::Local { path } => Self::resolve_local(&source.alias, path),
            SourceOrigin::Git {
                url,
                git_ref,
                subpath,
            } => self.resolve_git(
                &source.alias,
                url,
                git_ref.as_deref(),
                subpath.as_deref(),
            ),
        }
    }
}

/// One file entry in a physical snapshot payload.
#[derive(Clone, Debug, Serialize)]
struct PhysicalFileEntry {
    /// Path relative to the snapshot root, using `/` separators.
    path: String,
    /// Blake3 hash of file contents (hex-encoded).
    hash: String,
}

/// JSON payload stored in a physical snapshot artifact.
#[derive(Clone, Debug, Serialize)]
struct PhysicalSnapshotPayload {
    /// All regular files under the snapshot root.
    files: Vec<PhysicalFileEntry>,
}

/// Walk `root`, hash every regular file, and return a physical snapshot artifact.
///
/// Skips `.git/`, `target/`, `node_modules/`, and `.s4/` directories anywhere in the tree.
///
/// # Errors
///
/// Returns an error if `root` is missing, not a directory, or a file cannot be read.
pub fn snapshot_physical(root: &Path) -> Result<Artifact> {
    if !root.is_dir() {
        return Err(S4Error::Other(format!(
            "snapshot root is not a directory: {}",
            root.display()
        )));
    }

    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip_entry(e.path()))
    {
        let entry =
            entry.map_err(|e| S4Error::Other(format!("failed to walk {}: {e}", root.display())))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let full_path = entry.path();
        let relative = full_path.strip_prefix(root).map_err(|_| {
            S4Error::Other(format!("failed to relativize path {}", full_path.display()))
        })?;

        let content = std::fs::read(full_path)
            .map_err(|e| S4Error::Other(format!("failed to read {}: {e}", full_path.display())))?;
        let hash = blake3::hash(&content);
        files.push(PhysicalFileEntry {
            path: relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
            hash: hash.to_hex().to_string(),
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));

    let payload = PhysicalSnapshotPayload { files };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|e| S4Error::Other(format!("failed to serialize physical snapshot: {e}")))?;

    Ok(Artifact {
        kind: ArtifactKind::PhysicalSnapshot,
        schema_version: SchemaVersion::CURRENT,
        payload: payload_value,
    })
}

const SKIP_DIR_NAMES: &[&str] = &[".git", "target", "node_modules", ".s4"];

fn should_skip_entry(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(component, Component::Normal(name) if SKIP_DIR_NAMES.contains(&name.to_str().unwrap_or("")))
    })
}

fn run_git(args: &[&str], cwd: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| S4Error::Other(format!("failed to spawn git: {e}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(S4Error::Other(format!(
        "git {} failed in {}: {stderr}",
        args.join(" "),
        cwd.display()
    )))
}

fn git_head_commit(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        None
    } else {
        Some(commit)
    }
}
