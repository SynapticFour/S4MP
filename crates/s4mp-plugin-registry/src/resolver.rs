use crate::Registry;
use s4mp_core::{Result, S4mpError};
use s4mp_plugin_api::PluginManifest;

pub struct Resolver<'a> {
    registry: &'a Registry,
}

impl<'a> Resolver<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }

    pub fn resolve(&self, name: &str) -> Result<&PluginManifest> {
        self.registry
            .get(name)
            .ok_or_else(|| S4mpError::Plugin {
                plugin_id: name.to_string(),
                message: "plugin not found in registry".into(),
            })
    }
}
