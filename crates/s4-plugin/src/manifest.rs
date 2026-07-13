use serde::{Deserialize, Serialize};

/// Plugin package manifest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Package name.
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Required plugin API version.
    pub api_version: String,
    /// Declared capabilities.
    pub capabilities: CapabilitySet,
    /// Optional description.
    pub description: Option<String>,
}

/// Role a plugin may declare.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Provides repository import.
    Importer,
    /// Provides source parsing.
    Parser,
    /// Provides IR linking.
    Linker,
    /// Provides analysis.
    Analyzer,
    /// Provides LLM reasoning (provider-specific impl).
    Reasoner,
    /// Provides verification rules.
    Verifier,
}

/// Capability flags and metadata declared by a plugin.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Enabled plugin roles.
    pub roles: Vec<PluginCapability>,
    /// Supported language identifiers.
    pub languages: Vec<String>,
    /// Glob file patterns handled by parser.
    pub file_patterns: Vec<String>,
}
