use crate::source::{SourceOrigin, SourceRef};
use s4_core::{Result, S4Error, SchemaVersion};
use s4_storage::{Artifact, ArtifactKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
    refresh: bool,
}

impl DefaultSourceIngestor {
    /// Create an ingestor that caches Git clones under `<workspace_root>/.s4/cache/`.
    ///
    /// Existing clones are reused without `git fetch` unless [`Self::with_refresh`].
    #[must_use]
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            refresh: false,
        }
    }

    /// When true, `git fetch` existing cache directories on resolve.
    #[must_use]
    pub fn with_refresh(mut self, refresh: bool) -> Self {
        self.refresh = refresh;
        self
    }

    fn resolve_local(alias: &str, path: &Path) -> Result<ResolvedSource> {
        if !path.exists() {
            return Err(S4Error::InvalidInput(format!(
                "local source path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(S4Error::InvalidInput(format!(
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

    /// Resolve a Git remote into `.s4/cache/<alias>/`, optionally narrowing to `subpath`.
    ///
    /// When `subpath` is set and the cache directory does not yet exist, performs a sparse
    /// checkout clone (`--filter=blob:none`, cone mode) instead of a full shallow clone.
    /// That fetches tree and commit metadata completely but materializes file blobs only
    /// for paths in the sparse-checkout set — a large bandwidth and time saving for huge
    /// monorepos where most blobs live outside the target subtree. Requires Git >= 2.27 for
    /// `--cone` sparse-checkout; if the local Git is too old, [`run_git`] surfaces stderr.
    fn resolve_git(
        &self,
        alias: &str,
        url: &str,
        git_ref: Option<&str>,
        subpath: Option<&str>,
    ) -> Result<ResolvedSource> {
        validate_source_alias(alias)?;
        validate_git_url(url)?;
        if let Some(reference) = git_ref {
            validate_git_ref(reference)?;
        }
        if let Some(sub) = subpath {
            validate_git_subpath(sub)?;
        }
        let cache_path = self.workspace_root.join(".s4").join("cache").join(alias);

        if cache_path.exists() {
            if self.refresh {
                run_git(&["fetch"], &cache_path)?;
            }
            if let Some(reference) = git_ref {
                run_git(&["checkout", reference], &cache_path)?;
            }
            if let Some(sub) = subpath {
                run_git(&["sparse-checkout", "set", sub], &cache_path)?;
            }
        } else {
            if let Some(parent) = cache_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    S4Error::Storage(format!(
                        "failed to create cache directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }

            let dest = cache_path.to_string_lossy();
            if let Some(sub) = subpath {
                let mut clone_args = vec![
                    "clone",
                    "--filter=blob:none",
                    "--no-checkout",
                    "--depth",
                    "1",
                ];
                if let Some(reference) = git_ref {
                    clone_args.push("--branch");
                    clone_args.push(reference);
                }
                clone_args.push("--");
                clone_args.push(url);
                clone_args.push(dest.as_ref());
                run_git(&clone_args, self.workspace_root.as_path())?;

                run_git(&["sparse-checkout", "init", "--cone"], &cache_path)?;
                run_git(&["sparse-checkout", "set", sub], &cache_path)?;

                if let Some(reference) = git_ref {
                    run_git(&["checkout", reference], &cache_path)?;
                } else {
                    run_git(&["checkout"], &cache_path)?;
                }
            } else {
                let mut args = vec!["clone", "--depth", "1"];
                if let Some(reference) = git_ref {
                    args.push("--branch");
                    args.push(reference);
                }
                args.push("--");
                args.push(url);
                args.push(dest.as_ref());
                run_git(&args, self.workspace_root.as_path())?;
            }
        }

        let local_root = match subpath {
            Some(sub) => cache_path.join(sub),
            None => cache_path.clone(),
        };

        if !local_root.is_dir() {
            return Err(S4Error::InvalidInput(format!(
                "resolved git source path is not a directory: {}",
                local_root.display()
            )));
        }

        Ok(ResolvedSource {
            alias: alias.to_string(),
            local_root,
            commit: Some(git_head_commit(&cache_path)?),
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
            } => self.resolve_git(&source.alias, url, git_ref.as_deref(), subpath.as_deref()),
        }
    }
}

/// One file entry in a physical snapshot payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PhysicalFileEntry {
    /// Path relative to the snapshot root, using `/` separators.
    path: String,
    /// Blake3 hash of file contents (hex-encoded).
    hash: String,
}

/// JSON payload stored in a physical snapshot artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PhysicalSnapshotPayload {
    /// All regular files under the snapshot root.
    files: Vec<PhysicalFileEntry>,
}

/// Map snapshot-relative unix paths to Blake3 hex hashes.
///
/// # Errors
///
/// Returns an error if the artifact payload is not a physical snapshot.
pub fn snapshot_path_hashes(snapshot: &Artifact) -> Result<HashMap<String, String>> {
    let payload: PhysicalSnapshotPayload = serde_json::from_value(snapshot.payload.clone())
        .map_err(|e| S4Error::Storage(format!("failed to parse physical snapshot payload: {e}")))?;
    Ok(payload
        .files
        .into_iter()
        .map(|f| (f.path, f.hash))
        .collect())
}

/// Walk `root`, hash every regular file, and return a physical snapshot artifact.
///
/// Skips `.git/`, `target/`, `node_modules/`, and workspace metadata under `.s4/`
/// (store, graphs, maps, …). Paths under `.s4/cache/` — git source trees — are walked normally.
///
/// # Errors
///
/// Returns an error if `root` is missing, not a directory, or a file cannot be read.
pub fn snapshot_physical(root: &Path) -> Result<Artifact> {
    if !root.is_dir() {
        return Err(S4Error::InvalidInput(format!(
            "snapshot root is not a directory: {}",
            root.display()
        )));
    }

    let mut files = Vec::new();
    let mut total_bytes = 0_u64;

    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip_snapshot_path(e.path()))
    {
        let entry = entry
            .map_err(|e| S4Error::Storage(format!("failed to walk {}: {e}", root.display())))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if files.len() >= MAX_SNAPSHOT_FILES {
            return Err(S4Error::InvalidInput(format!(
                "snapshot exceeds {MAX_SNAPSHOT_FILES} files under {}",
                root.display()
            )));
        }

        let full_path = entry.path();
        let meta = std::fs::metadata(full_path).map_err(|e| {
            S4Error::Storage(format!("failed to stat {}: {e}", full_path.display()))
        })?;
        let len = meta.len();
        if len > MAX_SNAPSHOT_FILE_BYTES {
            return Err(S4Error::InvalidInput(format!(
                "snapshot file {} is {len} bytes (limit {MAX_SNAPSHOT_FILE_BYTES})",
                full_path.display()
            )));
        }
        total_bytes = total_bytes.saturating_add(len);
        if total_bytes > MAX_SNAPSHOT_TOTAL_BYTES {
            return Err(S4Error::InvalidInput(format!(
                "snapshot exceeds {MAX_SNAPSHOT_TOTAL_BYTES} bytes under {}",
                root.display()
            )));
        }

        let relative = full_path.strip_prefix(root).map_err(|_| {
            S4Error::Storage(format!("failed to relativize path {}", full_path.display()))
        })?;

        let content_hash = hash_file(full_path)?;
        files.push(PhysicalFileEntry {
            path: relative_unix_path(relative),
            hash: content_hash,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));

    let payload = PhysicalSnapshotPayload { files };
    let payload_value = serde_json::to_value(&payload)
        .map_err(|e| S4Error::Storage(format!("failed to serialize physical snapshot: {e}")))?;

    Ok(Artifact {
        kind: ArtifactKind::PhysicalSnapshot,
        schema_version: SchemaVersion::CURRENT,
        payload: payload_value,
    })
}

const SKIP_DIR_NAMES: &[&str] = &[".git", "target", "node_modules"];
/// Hard cap on files hashed into a physical snapshot.
const MAX_SNAPSHOT_FILES: usize = 50_000;
/// Hard cap on a single file hashed into a physical snapshot (32 MiB).
const MAX_SNAPSHOT_FILE_BYTES: u64 = 32 * 1024 * 1024;
/// Hard cap on total bytes hashed into a physical snapshot (512 MiB).
const MAX_SNAPSHOT_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

/// Whether `path` should be skipped while snapshotting or discovering sources.
#[must_use]
pub fn should_skip_snapshot_path(path: &Path) -> bool {
    let components: Vec<_> = path.components().collect();
    for (i, component) in components.iter().enumerate() {
        if let Component::Normal(name) = component {
            let name = name.to_str().unwrap_or("");
            if name == ".s4" {
                let next_is_cache =
                    components.get(i + 1).and_then(|c| c.as_os_str().to_str()) == Some("cache");
                if !next_is_cache {
                    return true;
                }
            } else if SKIP_DIR_NAMES.contains(&name) {
                return true;
            }
        }
    }
    false
}

/// Reject aliases that would escape `.s4/cache/<alias>/`.
///
/// # Errors
///
/// Returns [`S4Error::InvalidId`] when the alias is empty or contains path separators.
pub fn validate_source_alias(alias: &str) -> Result<()> {
    if alias.is_empty()
        || !alias
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(S4Error::InvalidId(format!(
            "source alias must match [A-Za-z0-9_-]+, got '{alias}'"
        )));
    }
    Ok(())
}

/// Allow only `https://`, `http://`, `ssh://`, and `git@host:path` Git remotes.
///
/// Rejects leading dashes (flag injection), `ext::`, `file://`, and other Git
/// transport tricks. The CLI is local, but Makefile defaults clone third-party
/// remotes.
///
/// # Errors
///
/// Returns [`S4Error::InvalidInput`] when the URL is empty, a flag, or an
/// unsupported scheme.
pub fn validate_git_url(url: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        return Err(S4Error::InvalidInput(format!(
            "git URL must not be empty or start with '-': '{url}'"
        )));
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("ext::")
        || lower.starts_with("file:")
        || lower.starts_with("fd::")
        || lower.contains('\n')
        || lower.contains('\0')
    {
        return Err(S4Error::InvalidInput(format!(
            "git URL scheme is not allowed: '{url}'"
        )));
    }
    let scp = trimmed.starts_with("git@") && trimmed.contains(':') && !trimmed.contains("://");
    if lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("ssh://")
        || scp
    {
        return Ok(());
    }
    Err(S4Error::InvalidInput(format!(
        "git URL must be https://, http://, ssh://, or git@host:path, got '{url}'"
    )))
}

/// Reject `git_ref` values that Git would treat as options or path traversal.
///
/// # Errors
///
/// Returns an error for empty refs, leading dashes, or illegal characters.
pub fn validate_git_ref(git_ref: &str) -> Result<()> {
    if git_ref.is_empty() || git_ref.starts_with('-') {
        return Err(S4Error::InvalidInput(format!(
            "git ref must not be empty or start with '-': '{git_ref}'"
        )));
    }
    if git_ref.contains("..") || git_ref.contains('\n') || git_ref.contains('\0') {
        return Err(S4Error::InvalidInput(format!(
            "git ref contains illegal characters: '{git_ref}'"
        )));
    }
    if !git_ref
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | '+'))
    {
        return Err(S4Error::InvalidInput(format!(
            "git ref must match [A-Za-z0-9._/+-]+, got '{git_ref}'"
        )));
    }
    Ok(())
}

/// Reject git subpaths that could escape the clone directory.
///
/// # Errors
///
/// Returns an error for empty, absolute, or `..` paths.
pub fn validate_git_subpath(subpath: &str) -> Result<()> {
    if subpath.is_empty() {
        return Err(S4Error::InvalidId("git subpath must not be empty".into()));
    }
    let path = Path::new(subpath);
    if path.is_absolute() {
        return Err(S4Error::InvalidId(format!(
            "git subpath must be relative, got '{subpath}'"
        )));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {},
            _ => {
                return Err(S4Error::InvalidId(format!(
                    "git subpath must not contain '..' or prefix components, got '{subpath}'"
                )));
            },
        }
    }
    Ok(())
}

fn relative_unix_path(relative: &Path) -> String {
    relative
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| S4Error::Storage(format!("failed to open {}: {e}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0_u8; 65_536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", path.display())))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

const GIT_TIMEOUT: Duration = Duration::from_secs(120);

fn run_git(args: &[&str], cwd: &Path) -> Result<()> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| S4Error::External(format!("failed to spawn git: {e}")))?;

    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| S4Error::External("failed to capture git stderr".to_string()))?;
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + GIT_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr_bytes = stderr_thread.join().unwrap_or_default();
                if status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&stderr_bytes);
                return Err(S4Error::External(format!(
                    "git {} failed in {}: {stderr}",
                    args.join(" "),
                    cwd.display()
                )));
            },
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(S4Error::External(format!(
                        "git {} timed out after {}s in {}",
                        args.join(" "),
                        GIT_TIMEOUT.as_secs(),
                        cwd.display()
                    )));
                }
                thread::sleep(Duration::from_millis(50));
            },
            Err(e) => {
                return Err(S4Error::External(format!("failed to wait for git: {e}")));
            },
        }
    }
}

fn git_head_commit(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| S4Error::External(format!("failed to spawn git rev-parse: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(S4Error::External(format!(
            "git rev-parse HEAD failed in {}: {stderr}",
            repo_root.display()
        )));
    }

    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        return Err(S4Error::External(format!(
            "git rev-parse HEAD returned empty in {}",
            repo_root.display()
        )));
    }
    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_rejects_path_escape() {
        assert!(validate_source_alias("gatk-java-hc").is_ok());
        assert!(validate_source_alias("../etc").is_err());
        assert!(validate_source_alias("foo/bar").is_err());
        assert!(validate_source_alias("").is_err());
    }

    #[test]
    fn subpath_rejects_parent_dir() {
        assert!(validate_git_subpath("src/main/java").is_ok());
        assert!(validate_git_subpath("../secret").is_err());
        assert!(validate_git_subpath("/abs").is_err());
    }

    #[test]
    fn git_url_allowlist() {
        assert!(validate_git_url("https://github.com/broadinstitute/gatk.git").is_ok());
        assert!(validate_git_url("git@github.com:broadinstitute/gatk.git").is_ok());
        assert!(validate_git_url("ssh://git@github.com/org/repo.git").is_ok());
        assert!(validate_git_url("--upload-pack=evil").is_err());
        assert!(validate_git_url("ext::sh -c id").is_err());
        assert!(validate_git_url("file:///etc/passwd").is_err());
        assert!(validate_git_url("").is_err());
    }

    #[test]
    fn git_ref_rejects_flags() {
        assert!(validate_git_ref("main").is_ok());
        assert!(validate_git_ref("feature/foo").is_ok());
        assert!(validate_git_ref("-ccore.sshCommand=oops").is_err());
        assert!(validate_git_ref("../secret").is_err());
        assert!(validate_git_ref("").is_err());
    }
}
