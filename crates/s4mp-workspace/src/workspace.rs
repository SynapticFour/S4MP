use crate::{Lockfile, ProjectConfig};
use s4mp_core::SnapshotId;
use std::path::{Path, PathBuf};

pub struct Workspace {
    pub root: PathBuf,
    pub config: ProjectConfig,
    pub lockfile: Lockfile,
    pub current_snapshot: Option<SnapshotId>,
}

impl Workspace {
    pub fn open(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            config: ProjectConfig::default(),
            lockfile: Lockfile::default(),
            current_snapshot: None,
        }
    }
}
