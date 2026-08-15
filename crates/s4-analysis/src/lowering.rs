use s4_core::{Result, S4Error};
use s4_graph::memory::InMemoryGraph;
use s4_graph::view::GraphBuilder;
use s4_graph::{Edge, EdgeKind, GraphView, Node, NodeId, NodeKind};
use s4_parser::{UsirEntityKind, UsirLocalId, UsirModule, UsirRelationKind};
use std::collections::HashMap;

/// Lower USIR modules into a semantic-layer in-memory graph view.
///
/// Node IDs are assigned sequentially across all modules. Entity indices inside each
/// [`UsirModule`] are mapped to these global IDs before relations are materialized.
/// Unresolved cross-module calls are linked by simple callee name.
///
/// # Errors
///
/// Returns an error if a relation references an unknown entity or graph construction fails.
pub fn usir_to_graph(modules: &[UsirModule]) -> Result<Box<dyn GraphView>> {
    let mut builder = InMemoryGraph::new();
    let mut next_node_id = 0_u64;
    let mut module_maps: Vec<HashMap<UsirLocalId, NodeId>> = Vec::with_capacity(modules.len());
    let mut callables_by_name: HashMap<String, Vec<NodeId>> = HashMap::new();

    for module in modules {
        let mut local_to_global: HashMap<UsirLocalId, NodeId> = HashMap::new();

        for entity in &module.entities {
            let node_id = NodeId(next_node_id);
            next_node_id += 1;
            local_to_global.insert(entity.id, node_id);
            if entity.kind == UsirEntityKind::Callable {
                callables_by_name
                    .entry(entity.name.clone())
                    .or_default()
                    .push(node_id);
            }
            builder.add_node(Node {
                id: node_id,
                kind: map_entity_kind(&entity.kind),
                label: entity.name.clone(),
                signature: entity.signature.clone(),
                qualified: entity.qualified.clone(),
            })?;
        }

        module_maps.push(local_to_global);
    }

    for (module, local_to_global) in modules.iter().zip(&module_maps) {
        for relation in &module.relations {
            let from = *local_to_global.get(&relation.from).ok_or_else(|| {
                S4Error::InvalidId(format!(
                    "USIR relation source entity {} not found in module {}",
                    relation.from, module.name
                ))
            })?;
            let to = *local_to_global.get(&relation.to).ok_or_else(|| {
                S4Error::InvalidId(format!(
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

        for unresolved in &module.unresolved_calls {
            let Some(&from) = local_to_global.get(&unresolved.from) else {
                continue;
            };
            let Some(targets) = callables_by_name.get(&unresolved.callee_name) else {
                continue;
            };
            if targets.len() != 1 {
                continue;
            }
            let to = targets[0];
            if to == from {
                continue;
            }
            builder.add_edge(Edge {
                from,
                to,
                kind: EdgeKind::Calls,
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

#[cfg(test)]
mod tests {
    use super::*;
    use s4_parser::usir::{UnresolvedCall, UsirEntity, UsirLocalId, UsirRelation};

    #[test]
    fn cross_module_unresolved_call_becomes_edge() {
        let modules = vec![
            UsirModule {
                name: "a.java".into(),
                entities: vec![
                    UsirEntity {
                        id: UsirLocalId(0),
                        kind: UsirEntityKind::Module,
                        name: "a.java".into(),
                        signature: None,
                        qualified: None,
                    },
                    UsirEntity {
                        id: UsirLocalId(1),
                        kind: UsirEntityKind::Callable,
                        name: "caller".into(),
                        signature: Some("caller():void".into()),
                        qualified: None,
                    },
                ],
                relations: vec![UsirRelation {
                    from: UsirLocalId(0),
                    to: UsirLocalId(1),
                    kind: UsirRelationKind::Defines,
                }],
                unresolved_calls: vec![UnresolvedCall {
                    from: UsirLocalId(1),
                    callee_name: "scale".into(),
                }],
            },
            UsirModule {
                name: "b.java".into(),
                entities: vec![
                    UsirEntity {
                        id: UsirLocalId(0),
                        kind: UsirEntityKind::Module,
                        name: "b.java".into(),
                        signature: None,
                        qualified: None,
                    },
                    UsirEntity {
                        id: UsirLocalId(1),
                        kind: UsirEntityKind::Callable,
                        name: "scale".into(),
                        signature: Some("scale(int):int".into()),
                        qualified: None,
                    },
                ],
                relations: vec![UsirRelation {
                    from: UsirLocalId(0),
                    to: UsirLocalId(1),
                    kind: UsirRelationKind::Defines,
                }],
                unresolved_calls: vec![],
            },
        ];

        let graph = usir_to_graph(&modules).unwrap();
        let edges: Vec<_> = graph.edges().cloned().collect();
        assert!(
            edges.iter().any(|e| e.kind == EdgeKind::Calls),
            "expected cross-module Calls edge, got {edges:?}"
        );
    }
}
