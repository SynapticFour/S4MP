use s4mp_core::{ApiVersion, ArtifactId, PluginId, Result};

/// Base trait implemented by all S4MP plugins.
pub trait Plugin: Send + Sync {
    fn id(&self) -> &PluginId;
    fn api_version(&self) -> ApiVersion;
    fn manifest(&self) -> &crate::PluginManifest;
}

/// Context passed to plugin invocations. Reads/writes go through artifact IDs.
pub struct InvocationContext<'a> {
    pub input_artifacts: &'a [ArtifactId],
    pub output_artifacts: &'a mut Vec<ArtifactId>,
}

impl<'a> InvocationContext<'a> {
    pub fn emit_output(&mut self, id: ArtifactId) {
        self.output_artifacts.push(id);
    }
}

/// Plugin invocation result with diagnostics.
pub struct PluginOutput {
    pub artifacts: Vec<ArtifactId>,
    pub diagnostics: Vec<String>,
}

pub type PluginFn = fn(&mut InvocationContext<'_>) -> Result<()>;
