use crate::{Entity, EntityId, Relation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrModule {
    pub name: String,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

impl IrModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
            relations: Vec::new(),
        }
    }

    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().map(|e| e.id)
    }
}
