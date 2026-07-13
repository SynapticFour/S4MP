use s4mp_plugin_api::PluginManifest;
use std::collections::HashMap;

#[derive(Default)]
pub struct Registry {
    manifests: HashMap<String, PluginManifest>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, manifest: PluginManifest) {
        self.manifests.insert(manifest.name.clone(), manifest);
    }

    pub fn get(&self, name: &str) -> Option<&PluginManifest> {
        self.manifests.get(name)
    }

    pub fn list(&self) -> impl Iterator<Item = &PluginManifest> {
        self.manifests.values()
    }
}
