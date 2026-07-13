use serde::{Deserialize, Serialize};

/// Top-level project configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Human-readable project name.
    pub name: String,
    /// Enabled plugins and versions.
    pub plugins: Vec<PluginRef>,
}

/// Reference to a plugin dependency.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginRef {
    /// Plugin package name.
    pub name: String,
    /// Requested semver range or exact version.
    pub version: String,
}
