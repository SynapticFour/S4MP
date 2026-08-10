//! Deterministic SKG concept extraction from code graphs (Phase 4 thin slice).

use crate::fact::{Confidence, Fact, FactKind, FactLifecycle, FactPayload};
use crate::provenance::{Provenance, SourceType};
use s4_graph::{GraphView, Node, NodeId, NodeKind};
use serde::{Deserialize, Serialize};

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
    for index in 0..view.node_count() as u64 {
        let Some(node) = view.node(NodeId(index)) else {
            continue;
        };
        if node.kind != NodeKind::Type {
            continue;
        }
        if node.label.starts_with('<') {
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

/// Materialize extracted concepts as accepted structural facts.
#[must_use]
pub fn concepts_to_facts(concepts: &[Concept], view: &dyn GraphView) -> Vec<Fact> {
    concepts
        .iter()
        .filter_map(|concept| {
            let node = concept.source_node.and_then(|id| view.node(NodeId(id)))?;
            Some(concept_fact(node))
        })
        .collect()
}

fn concept_fact(node: &Node) -> Fact {
    Fact {
        kind: FactKind::Structural,
        lifecycle: FactLifecycle::Accepted,
        confidence: Confidence::CERTAIN,
        provenance: Provenance {
            source_type: SourceType::Analysis,
            source_id: "naming_concept_extractor_v1".into(),
            artifact_id: s4_core::ArtifactId::from_content(node.label.as_bytes()),
            timestamp: "0".into(),
            schema_version: s4_core::SchemaVersion::CURRENT,
        },
        payload: FactPayload::Node(node.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4_graph::memory::InMemoryGraphView;

    #[test]
    fn extracts_type_concepts() {
        let view = InMemoryGraphView::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Type,
                    label: "Calculator".into(),
                    signature: None,
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Callable,
                    label: "add".into(),
                    signature: None,
                },
            ],
            vec![],
        );
        let concepts = extract_concepts_from_graph(&view);
        assert_eq!(concepts.len(), 1);
        assert_eq!(concepts[0].name, "Calculator");
    }
}
