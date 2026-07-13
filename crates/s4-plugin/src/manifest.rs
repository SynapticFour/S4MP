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

/// Capability flags and metadata declared by a plugin.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Provides repository import.
    pub importer: bool,
    /// Provides source parsing.
    pub parser: bool,
    /// Provides IR linking.
    pub linker: bool,
    /// Provides analysis.
    pub analyzer: bool,
    /// Provides LLM reasoning (provider-specific impl).
    pub reasoner: bool,
    /// Provides verification rules.
    pub verifier: bool,
    /// Supported language identifiers.
    pub languages: Vec<String>,
    /// Glob file patterns handled by parser.
    pub file_patterns: Vec<String>,
}
