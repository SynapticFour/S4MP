//! Deterministic SKG concept extraction from code graphs (Phase 4 thin slice).

use crate::fact::{Confidence, Fact, FactKind, FactLifecycle, FactPayload};
use crate::provenance::{Provenance, SourceType};
use s4_core::utc_rfc3339;
use s4_graph::{GraphView, Node, NodeKind};
use serde::{Deserialize, Serialize};

/// Heuristic confidence for naming-derived concepts (not parse-certain).
const NAMING_CONCEPT_CONFIDENCE: f32 = 0.4;

/// A business/domain concept derived from type naming.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    /// Concept display name (from type label).
    pub name: String,
    /// Originating graph node id when known.
    pub source_node: Option<u64>,
}

/// Extract concepts from [`NodeKind::Type`] labels (deterministic naming heuristic).
#[must_use]
pub fn extract_concepts_from_graph(view: &dyn GraphView) -> Vec<Concept> {
    let mut concepts = Vec::new();
    for node in view.nodes() {
        if node.kind != NodeKind::Type {
            continue;
        }
        if node.label.starts_with('<') || node.label.is_empty() {
            continue;
        }
        concepts.push(Concept {
            name: node.label.clone(),
            source_node: Some(node.id.0),
        });
    }
    concepts.sort_by(|a, b| a.name.cmp(&b.name));
    concepts.dedup_by(|a, b| a.name == b.name);
    concepts
}

/// Materialize extracted concepts as **proposed** naming-heuristic facts.
#[must_use]
pub fn concepts_to_facts(concepts: &[Concept], view: &dyn GraphView) -> Vec<Fact> {
    concepts
        .iter()
        .filter_map(|concept| {
            let node = concept
                .source_node
                .and_then(|id| view.node(s4_graph::NodeId(id)))?;
            Some(concept_fact(node))
        })
        .collect()
}

fn concept_fact(node: &Node) -> Fact {
    Fact {
        kind: FactKind::Structural,
        lifecycle: FactLifecycle::Proposed,
        confidence: Confidence::clamped(NAMING_CONCEPT_CONFIDENCE),
        provenance: Provenance {
            source_type: SourceType::Analysis,
            source_id: "naming_concept_extractor_v1".into(),
            artifact_id: s4_core::ArtifactId::from_content(node.label.as_bytes()),
            timestamp: utc_rfc3339(),
            schema_version: s4_core::SchemaVersion::CURRENT,
        },
        payload: FactPayload::Node(node.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_graph::memory::InMemoryGraphView;
    use s4_graph::NodeId;

    #[test]
    fn extracts_type_concepts() {
        let view = InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Type,
                    label: "Calculator".into(),
                    signature: None,
                    qualified: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "add".into(),
                    signature: None,
                    qualified: None,
                },
            ],
            vec![],
        );
        let concepts = extract_concepts_from_graph(&view);
        assert_eq!(concepts.len(), 1);
        assert_eq!(concepts[0].name, "Calculator");
        let facts = concepts_to_facts(&concepts, &view);
        assert_eq!(facts[0].lifecycle, FactLifecycle::Proposed);
        assert!((facts[0].confidence.0 - NAMING_CONCEPT_CONFIDENCE).abs() < f32::EPSILON);
        assert!(facts[0].provenance.timestamp.ends_with('Z'));
    }

    #[test]
    fn sparse_ids_are_enumerated() {
        let view = InMemoryGraphView::new(
            vec![Node {
                id: NodeId(42),
                kind: NodeKind::Type,
                label: "Widget".into(),
                signature: None,
                qualified: None,
            }],
            vec![],
        );
        let concepts = extract_concepts_from_graph(&view);
        assert_eq!(concepts[0].source_node, Some(42));
    }
}
