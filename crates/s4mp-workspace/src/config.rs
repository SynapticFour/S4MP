use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub plugins: Vec<PluginRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginRef {
    pub name: String,
    pub version: String,
}
