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
            return Err(S4Error::InvalidInput(format!(
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
            .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", path.display())))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| S4Error::Storage(format!("failed to parse {}: {e}", path.display())))
    }

    /// Persist as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                S4Error::Storage(format!("failed to create {}: {e}", parent.display()))
            })?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| S4Error::Storage(format!("failed to serialize requirements: {e}")))?;
        std::fs::write(path, bytes)
            .map_err(|e| S4Error::Storage(format!("failed to write {}: {e}", path.display())))
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
            .map_err(|e| S4Error::Storage(format!("failed to read {}: {e}", path.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| S4Error::Storage(format!("failed to parse OpenAPI JSON: {e}")))?;
        let Some(paths) = value.get("paths").and_then(|p| p.as_object()) else {
            return Err(S4Error::InvalidInput(
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
    ///
    /// Uses whole-word (identifier-boundary) matching after lowercasing, not substring.
    #[must_use]
    pub fn suggest_traces_by_name(
        &self,
        callables: &[(EntityId, String)],
    ) -> Vec<(RequirementId, EntityId, String)> {
        let lowered: Vec<(EntityId, String, String)> = callables
            .iter()
            .map(|(id, label)| (id.clone(), label.clone(), label.to_ascii_lowercase()))
            .collect();
        let mut out = Vec::new();
        for req in self.requirements.values() {
            let stmt = req.statement.to_ascii_lowercase();
            for (entity, label, needle) in &lowered {
                if contains_ident_word(&stmt, needle) {
                    out.push((req.id, entity.clone(), label.clone()));
                }
            }
        }
        out
    }
}

fn contains_ident_word(haystack: &str, needle: &str) -> bool {
    if needle.len() < 3 {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let before_ok = abs == 0 || !is_ident_byte(bytes[abs - 1]);
        let end = abs + needle.len();
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
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
        doc.add_trace(id, EntityId::new("g", 7), TraceLinkKind::ImplementedBy)
            .unwrap();
        assert_eq!(doc.traces_from(id).unwrap().len(), 1);
    }

    #[test]
    fn suggest_by_name() {
        let mut doc = RequirementsDocument::new();
        let id = doc.add(RequirementKind::Functional, "add two integers");
        let suggestions = doc.suggest_traces_by_name(&[(EntityId::new("g", 1), "add".into())]);
        assert_eq!(suggestions[0].0, id);
        let extra = doc.suggest_traces_by_name(&[(EntityId::new("g", 2), "add".into())]);
        let additional = {
            let mut d = RequirementsDocument::new();
            d.add(RequirementKind::Functional, "additional coverage");
            d.suggest_traces_by_name(&[(EntityId::new("g", 1), "add".into())])
        };
        assert!(additional.is_empty(), "{additional:?}");
        assert_eq!(extra.len(), 1);
    }
}
