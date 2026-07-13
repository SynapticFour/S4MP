use crate::PluginManifest;
use s4_core::{PluginId, Result};

/// Loads, validates, and invokes plugins.
pub trait PluginHost: Send + Sync {
    /// Register a plugin manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails.
    fn register(&mut self, manifest: PluginManifest) -> Result<()>;

    /// Lookup manifest by plugin ID.
    fn manifest(&self, id: &PluginId) -> Option<&PluginManifest>;

    /// Number of registered plugins.
    fn count(&self) -> usize;
}
