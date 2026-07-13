use serde::{Deserialize, Serialize};

/// Pinned plugin resolution for reproducible builds.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    /// Resolved plugin entries.
    pub plugins: Vec<LockedPlugin>,
}

/// Single locked plugin entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockedPlugin {
    /// Plugin name.
    pub name: String,
    /// Resolved version.
    pub version: String,
    /// Optional integrity checksum.
    pub checksum: Option<String>,
}
