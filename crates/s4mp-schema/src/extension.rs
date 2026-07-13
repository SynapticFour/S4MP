use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Namespaced identifier for extensible node/edge kinds.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ExtensionKindId(pub String);

/// Registry of extension kinds declared by plugins.
#[derive(Clone, Default, Debug)]
pub struct ExtensionRegistry {
    kinds: HashMap<ExtensionKindId, ExtensionKindMeta>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionKindMeta {
    pub description: String,
    pub declaring_plugin: String,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: ExtensionKindId, meta: ExtensionKindMeta) {
        self.kinds.insert(id, meta);
    }

    pub fn get(&self, id: &ExtensionKindId) -> Option<&ExtensionKindMeta> {
        self.kinds.get(id)
    }
}
