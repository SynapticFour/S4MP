use s4mp_core::{ApiVersion, PluginId, Result, S4mpError};
use s4mp_plugin_api::{Plugin, PluginManifest};
use std::collections::HashMap;

/// Manages loaded plugins and dispatches invocations.
pub struct PluginHost {
    plugins: HashMap<PluginId, LoadedPlugin>,
}

struct LoadedPlugin {
    manifest: PluginManifest,
    api_version: ApiVersion,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin_id: PluginId, manifest: PluginManifest) -> Result<()> {
        let api_version = parse_api_version(&manifest.api_version)?;
        self.plugins.insert(
            plugin_id,
            LoadedPlugin {
                manifest,
                api_version,
            },
        );
        Ok(())
    }

    pub fn get_manifest(&self, id: &PluginId) -> Option<&PluginManifest> {
        self.plugins.get(id).map(|p| &p.manifest)
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_api_version(s: &str) -> Result<ApiVersion> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        return Err(S4mpError::Other(format!("invalid api version: {s}")));
    }
    Ok(ApiVersion {
        major: parts[0].parse().map_err(|_| S4mpError::Other(format!("invalid api version: {s}")))?,
        minor: parts[1].parse().map_err(|_| S4mpError::Other(format!("invalid api version: {s}")))?,
    })
}

// Allow dead code until dynamic loading is implemented.
#[allow(dead_code)]
fn assert_plugin_compatible<P: Plugin>(plugin: &P) -> Result<()> {
    if !plugin.api_version().is_compatible_with(&ApiVersion::CURRENT) {
        return Err(S4mpError::Plugin {
            plugin_id: plugin.id().0.clone(),
            message: "incompatible API version".into(),
        });
    }
    Ok(())
}
