//! Git repository importer plugin (skeleton).

use s4mp_core::{ApiVersion, PluginId, Result};
use s4mp_plugin_api::{
    CapabilitySet, Importer, InvocationContext, Plugin, PluginManifest,
};

pub struct GitImporter {
    id: PluginId,
    manifest: PluginManifest,
}

impl GitImporter {
    pub fn new() -> Self {
        Self {
            id: PluginId("s4mp/importer-git".into()),
            manifest: PluginManifest {
                name: "s4mp-importer-git".into(),
                version: "0.1.0".into(),
                api_version: "0.1".into(),
                capabilities: CapabilitySet {
                    importer: true,
                    ..Default::default()
                },
                description: Some("Import Git repositories into physical snapshots".into()),
            },
        }
    }
}

impl Default for GitImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GitImporter {
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

impl Importer for GitImporter {
    fn import(&self, _ctx: &mut InvocationContext<'_>, source_uri: &str) -> Result<()> {
        let _ = source_uri;
        Ok(())
    }
}
