#[derive(Clone, Debug, Default)]
pub struct ImportConfig {
    pub source_uri: String,
    pub importer_plugin: String,
}
