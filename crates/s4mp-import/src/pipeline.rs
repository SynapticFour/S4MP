use crate::ImportConfig;
use s4mp_core::Result;
use s4mp_plugin_host::PluginHost;

pub struct ImportPipeline {
    config: ImportConfig,
}

impl ImportPipeline {
    pub fn new(config: ImportConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, _host: &PluginHost) -> Result<()> {
        let _ = &self.config;
        Ok(())
    }
}
