//! Universal Semantic IR (USIR) — stable interchange between parsers and analyzers.

pub mod builder;
pub mod entity;
pub mod module;
pub mod relation;
pub mod validator;

pub use builder::IrBuilder;
pub use entity::{Entity, EntityId, EntityKind};
pub use module::IrModule;
pub use relation::{Relation, RelationKind};
pub use validator::IrValidator;
