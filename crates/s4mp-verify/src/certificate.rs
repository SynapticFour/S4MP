use s4mp_core::ArtifactId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub id: ArtifactId,
    pub rule_set: String,
    pub passed: bool,
    pub message: Option<String>,
}
