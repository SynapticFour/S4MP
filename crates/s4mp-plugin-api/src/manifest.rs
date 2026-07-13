use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub capabilities: CapabilitySet,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub importer: bool,
    pub parser: bool,
    pub linker: bool,
    pub analyzer: bool,
    pub reasoner: bool,
    pub verifier: bool,
    pub languages: Vec<String>,
    pub file_patterns: Vec<String>,
}
