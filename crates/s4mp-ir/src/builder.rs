use crate::{Entity, EntityId, EntityKind, IrModule, Relation, RelationKind};

/// Builder for constructing USIR modules.
pub struct IrBuilder {
    next_id: u64,
    module: IrModule,
}

impl IrBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            next_id: 0,
            module: IrModule::new(name),
        }
    }

    pub fn add_entity(&mut self, kind: EntityKind, name: impl Into<String>) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.module.entities.push(Entity {
            id,
            kind,
            name: name.into(),
            attributes: serde_json::Value::Null,
        });
        id
    }

    pub fn add_relation(&mut self, from: EntityId, to: EntityId, kind: RelationKind) {
        self.module.relations.push(Relation { from, to, kind });
    }

    pub fn build(self) -> IrModule {
        self.module
    }
}
