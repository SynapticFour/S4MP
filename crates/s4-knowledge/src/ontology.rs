use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Namespaced identifier for extensible kinds.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ExtensionKindId(pub String);

/// Registry of ontology extensions declared by plugins.
#[derive(Clone, Debug, Default)]
pub struct Ontology {
    kinds: HashMap<ExtensionKindId, ExtensionKindMeta>,
}

/// Metadata for a registered extension kind.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionKindMeta {
    /// Human-readable description.
    pub description: String,
    /// Declaring plugin name.
    pub declaring_plugin: String,
}

impl Ontology {
    /// Create an empty ontology.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an extension kind.
    pub fn register(&mut self, id: ExtensionKindId, meta: ExtensionKindMeta) {
        self.kinds.insert(id, meta);
    }

    /// Lookup extension kind metadata.
    #[must_use]
    pub fn get(&self, id: &ExtensionKindId) -> Option<&ExtensionKindMeta> {
        self.kinds.get(id)
    }
}
