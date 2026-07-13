use crate::manifest::PluginManifest;
use s4_core::{ApiVersion, ArtifactId, PluginId, Result};
use serde::{Deserialize, Serialize};

/// Base trait implemented by every S4MP plugin.
pub trait Plugin: Send + Sync {
    /// Stable plugin identifier.
    fn id(&self) -> &PluginId;

    /// Plugin API version implemented by this plugin.
    fn api_version(&self) -> ApiVersion;

    /// Static plugin manifest.
    fn manifest(&self) -> &PluginManifest;
}

/// Context for a single plugin invocation. I/O is artifact-ID based.
pub struct InvocationContext<'a> {
    /// Input artifact identifiers.
    pub inputs: &'a [ArtifactId],
    /// Output artifact identifiers (append-only).
    pub outputs: &'a mut Vec<ArtifactId>,
}

impl InvocationContext<'_> {
    /// Record an output artifact identifier.
    pub fn emit(&mut self, id: ArtifactId) {
        self.outputs.push(id);
    }
}

/// Result of a plugin invocation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PluginOutput {
    /// Produced artifact identifiers.
    pub artifacts: Vec<ArtifactId>,
    /// Diagnostic messages.
    pub diagnostics: Vec<String>,
}

/// Type alias for plugin entry points loaded by the host.
pub type PluginEntrypoint = fn(&mut InvocationContext<'_>) -> Result<()>;
