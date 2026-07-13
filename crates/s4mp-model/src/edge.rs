use crate::{NodeId, Provenance};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Defines,
    References,
    Calls,
    Implements,
    DependsOn,
    TracesTo,
    Satisfies,
    Extension(String),
}
