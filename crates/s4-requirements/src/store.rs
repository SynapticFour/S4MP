//! In-memory / JSON-backed requirements + traces (Phase 4 thin slice).

use crate::requirement::{Requirement, RequirementId, RequirementKind};
use crate::trace::{TraceLink, TraceLinkKind, TraceabilityGraph};
use s4_core::{EntityId, Result, S4Error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Persistable requirements workspace artifact.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RequirementsDocument {
    /// Requirements by id.
    pub requirements: BTreeMap<u64, Requirement>,
    /// Trace links.
    pub traces: Vec<TraceLink>,
    /// Next requirement id.
    pub next_id: u64,
}

impl RequirementsDocument {
    /// Create an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            requirements: BTreeMap::new(),
            traces: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a requirement and return its id.
    pub fn add(&mut self, kind: RequirementKind, statement: impl Into<String>) -> RequirementId {
        let id = RequirementId(self.next_id);
        self.next_id += 1;
        self.requirements.insert(
            id.0,
            Requirement {
                id,
                kind,
                statement: statement.into(),
            },
        );
        id
    }

    /// Add an implemented-by trace to a code entity id.
    ///
    /// # Errors
    ///
    /// Returns an error if the requirement id is unknown.
    pub fn add_trace(
        &mut self,
        requirement: RequirementId,
        target: EntityId,
        kind: TraceLinkKind,
    ) -> Result<()> {
        if !self.requirements.contains_key(&requirement.0) {
            return Err(S4Error::Other(format!(
                "unknown requirement id {}",
                requirement.0
            )));
        }
        self.traces.push(TraceLink {
            requirement,
            target,
            kind,
        });
        Ok(())
    }

    /// Load from JSON path (empty document if missing).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self::new());
        }
        let bytes = std::fs::read(path)
            .map_err(|e| S4Error::Other(format!("failed to read {}: {e}", path.display())))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| S4Error::Other(format!("failed to parse {}: {e}", path.display())))
    }

    /// Persist as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                S4Error::Other(format!("failed to create {}: {e}", parent.display()))
            })?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| S4Error::Other(format!("failed to serialize requirements: {e}")))?;
        std::fs::write(path, bytes)
            .map_err(|e| S4Error::Other(format!("failed to write {}: {e}", path.display())))
    }

    /// Import `OpenAPI` `paths` keys as functional API contract requirements.
    ///
    /// Accepts JSON `OpenAPI` 3 documents. Each path becomes one requirement statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as JSON.
    pub fn import_openapi_paths(&mut self, path: &Path) -> Result<usize> {
        let bytes = std::fs::read(path)
            .map_err(|e| S4Error::Other(format!("failed to read {}: {e}", path.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| S4Error::Other(format!("failed to parse OpenAPI JSON: {e}")))?;
        let Some(paths) = value.get("paths").and_then(|p| p.as_object()) else {
            return Err(S4Error::Other(
                "OpenAPI document has no object 'paths' field".to_string(),
            ));
        };
        let mut count = 0_usize;
        for (api_path, _) in paths {
            self.add(
                RequirementKind::Functional,
                format!("API path `{api_path}` must remain available"),
            );
            count += 1;
        }
        Ok(count)
    }

    /// Suggest traces by matching requirement statement tokens to callable labels.
    #[must_use]
    pub fn suggest_traces_by_name(
        &self,
        callables: &[(EntityId, String)],
    ) -> Vec<(RequirementId, EntityId, String)> {
        let mut out = Vec::new();
        for req in self.requirements.values() {
            let stmt = req.statement.to_ascii_lowercase();
            for (entity, label) in callables {
                let needle = label.to_ascii_lowercase();
                if needle.len() >= 3 && stmt.contains(&needle) {
                    out.push((req.id, *entity, label.clone()));
                }
            }
        }
        out
    }
}

impl TraceabilityGraph for RequirementsDocument {
    fn requirements(&self) -> Box<dyn Iterator<Item = RequirementId> + '_> {
        Box::new(self.requirements.keys().copied().map(RequirementId))
    }

    fn traces_from(&self, requirement: RequirementId) -> Result<Vec<TraceLink>> {
        Ok(self
            .traces
            .iter()
            .filter(|t| t.requirement == requirement)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_trace() {
        let mut doc = RequirementsDocument::new();
        let id = doc.add(RequirementKind::Functional, "Calculator must add");
        doc.add_trace(id, EntityId(7), TraceLinkKind::ImplementedBy)
            .unwrap();
        assert_eq!(doc.traces_from(id).unwrap().len(), 1);
    }

    #[test]
    fn suggest_by_name() {
        let mut doc = RequirementsDocument::new();
        let id = doc.add(RequirementKind::Functional, "add two integers");
        let suggestions = doc.suggest_traces_by_name(&[(EntityId(1), "add".into())]);
        assert_eq!(suggestions[0].0, id);
    }
}
