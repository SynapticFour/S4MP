use crate::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    pub from: EntityId,
    pub to: EntityId,
    pub kind: RelationKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Defines,
    Declares,
    References,
    Calls,
    Implements,
    DependsOn,
    Extension(String),
}
