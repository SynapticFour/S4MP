use crate::{Lockfile, ProjectConfig, SnapshotRef};
use s4_core::{ProjectId, Result};
use std::path::Path;

/// Mutable project workspace bound to a filesystem root.
pub trait Workspace: Send + Sync {
    /// Project identifier.
    fn id(&self) -> &ProjectId;

    /// Filesystem root of the workspace.
    fn root(&self) -> &Path;

    /// Current project configuration.
    fn config(&self) -> &ProjectConfig;

    /// Pinned plugin lockfile.
    fn lockfile(&self) -> &Lockfile;

    /// Active snapshot reference, if any.
    fn current_snapshot(&self) -> Option<&SnapshotRef>;

    /// Persist configuration changes.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be saved.
    fn save_config(&mut self) -> Result<()>;
}
