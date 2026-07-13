use s4mp_core::ArtifactId;
use std::collections::{HashMap, HashSet};

/// Tracks artifact dependencies for incremental invalidation.
#[derive(Default)]
pub struct InvalidationGraph {
    dependents: HashMap<ArtifactId, HashSet<ArtifactId>>,
}

impl InvalidationGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dependency(&mut self, artifact: ArtifactId, depends_on: ArtifactId) {
        self.dependents
            .entry(depends_on)
            .or_default()
            .insert(artifact);
    }

    pub fn invalidate_from(&self, root: &ArtifactId) -> Vec<ArtifactId> {
        let mut result = Vec::new();
        if let Some(deps) = self.dependents.get(root) {
            result.extend(deps.iter().copied());
        }
        result
    }
}
