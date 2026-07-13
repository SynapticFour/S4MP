use serde::{Deserialize, Serialize};

/// Identifier for a programming or markup language.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct LanguageId(pub String);
