use s4_core::{Result, S4Error};
use s4_graph::memory::InMemoryGraph;
use s4_graph::view::GraphBuilder;
use s4_graph::{Edge, EdgeKind, GraphView, Node, NodeId, NodeKind};
use s4_parser::{UsirEntityKind, UsirModule, UsirRelationKind};
use std::collections::HashMap;

/// Lower USIR modules into a semantic-layer in-memory graph view.
///
/// Node IDs are assigned sequentially across all modules. Entity indices inside each
/// [`UsirModule`] are mapped to these global IDs before relations are materialized.
///
/// # Errors
///
/// Returns an error if a relation references an unknown entity or graph construction fails.
pub fn usir_to_graph(modules: &[UsirModule]) -> Result<Box<dyn GraphView>> {
    let mut builder = InMemoryGraph::new();
    let mut next_node_id = 0_u64;

    for module in modules {
        let mut local_to_global: HashMap<u64, NodeId> = HashMap::new();

        for entity in &module.entities {
            let node_id = NodeId(next_node_id);
            next_node_id += 1;
            local_to_global.insert(entity.id, node_id);
            builder.add_node(Node {
                id: node_id,
                kind: map_entity_kind(&entity.kind),
                label: entity.name.clone(),
            })?;
        }

        for relation in &module.relations {
            let from = *local_to_global.get(&relation.from).ok_or_else(|| {
                S4Error::Other(format!(
                    "USIR relation source entity {} not found in module {}",
                    relation.from, module.name
                ))
            })?;
            let to = *local_to_global.get(&relation.to).ok_or_else(|| {
                S4Error::Other(format!(
                    "USIR relation target entity {} not found in module {}",
                    relation.to, module.name
                ))
            })?;
            builder.add_edge(Edge {
                from,
                to,
                kind: map_relation_kind(&relation.kind),
            })?;
        }
    }

    GraphBuilder::build(Box::new(builder))
}

fn map_entity_kind(kind: &UsirEntityKind) -> NodeKind {
    match kind {
        UsirEntityKind::Module => NodeKind::Module,
        UsirEntityKind::Symbol => NodeKind::Symbol,
        UsirEntityKind::Callable => NodeKind::Callable,
        UsirEntityKind::Type => NodeKind::Type,
        UsirEntityKind::Extension(name) => NodeKind::Extension(name.clone()),
    }
}

fn map_relation_kind(kind: &UsirRelationKind) -> EdgeKind {
    match kind {
        UsirRelationKind::Defines => EdgeKind::Defines,
        UsirRelationKind::References => EdgeKind::References,
        UsirRelationKind::Calls => EdgeKind::Calls,
        UsirRelationKind::DependsOn => EdgeKind::DependsOn,
        UsirRelationKind::Extension(name) => EdgeKind::Extension(name.clone()),
    }
}
