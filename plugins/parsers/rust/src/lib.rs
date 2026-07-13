//! Rust language parser plugin (skeleton).

use s4mp_core::{ApiVersion, PluginId, Result};
use s4mp_plugin_api::{
    CapabilitySet, InvocationContext, Parser, Plugin, PluginManifest,
};

pub struct RustParser {
    id: PluginId,
    manifest: PluginManifest,
}

impl RustParser {
    pub fn new() -> Self {
        Self {
            id: PluginId("s4mp/parser-rust".into()),
            manifest: PluginManifest {
                name: "s4mp-parser-rust".into(),
                version: "0.1.0".into(),
                api_version: "0.1".into(),
                capabilities: CapabilitySet {
                    parser: true,
                    languages: vec!["rust".into()],
                    file_patterns: vec!["**/*.rs".into()],
                    ..Default::default()
                },
                description: Some("Parse Rust source into USIR artifacts".into()),
            },
        }
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RustParser {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn api_version(&self) -> ApiVersion {
        ApiVersion::CURRENT
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

impl Parser for RustParser {
    fn parse(&self, _ctx: &mut InvocationContext<'_>, path: &str) -> Result<()> {
        let _ = path;
        Ok(())
    }
}
