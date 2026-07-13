use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    pub plugins: Vec<LockedPlugin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockedPlugin {
    pub name: String,
    pub version: String,
    pub checksum: Option<String>,
}
