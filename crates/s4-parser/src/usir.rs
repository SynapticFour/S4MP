use serde::{Deserialize, Serialize};

/// Universal Semantic IR module — stable interchange between parsers and analyzers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsirModule {
    /// Module name.
    pub name: String,
    /// Entities in this module.
    pub entities: Vec<UsirEntity>,
    /// Relations between entities.
    pub relations: Vec<UsirRelation>,
    /// Call sites whose callee was not declared in this module (cross-file linker input).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_calls: Vec<UnresolvedCall>,
}

/// USIR entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsirEntity {
    /// Local entity index.
    pub id: u64,
    /// Entity classification.
    pub kind: UsirEntityKind,
    /// Qualified or local name.
    pub name: String,
    /// Optional signature string (callables/types); additive in schema 0.1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Standard USIR entity kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsirEntityKind {
    /// Compilation module.
    Module,
    /// Named symbol.
    Symbol,
    /// Callable.
    Callable,
    /// Type definition.
    Type,
    /// Extension kind.
    Extension(String),
}

/// USIR relation between entities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsirRelation {
    /// Source entity index.
    pub from: u64,
    /// Target entity index.
    pub to: u64,
    /// Relation classification.
    pub kind: UsirRelationKind,
}

/// Standard USIR relation kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsirRelationKind {
    /// Defines relationship.
    Defines,
    /// References relationship.
    References,
    /// Calls relationship.
    Calls,
    /// Depends-on relationship.
    DependsOn,
    /// Extension kind.
    Extension(String),
}

/// A call whose callee name was not resolved inside the declaring module.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedCall {
    /// Local caller entity id.
    pub from: u64,
    /// Simple callee identifier (no package qualification in v1).
    pub callee_name: String,
}
