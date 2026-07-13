use serde::{Deserialize, Serialize};
use std::fmt;

/// Semantic version for artifact and API schemas.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SchemaVersion {
    /// Major version — breaking changes.
    pub major: u32,
    /// Minor version — additive changes.
    pub minor: u32,
}

impl SchemaVersion {
    /// Current workspace schema version.
    pub const CURRENT: Self = Self { major: 0, minor: 1 };

    /// Create a new schema version.
    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Plugin API compatibility version.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ApiVersion {
    /// Major API version.
    pub major: u32,
    /// Minor API version.
    pub minor: u32,
}

impl ApiVersion {
    /// Current plugin API version.
    pub const CURRENT: Self = Self { major: 0, minor: 1 };

    /// Returns true when `self` satisfies `required`.
    #[must_use]
    pub const fn is_compatible_with(&self, required: &Self) -> bool {
        self.major == required.major && self.minor >= required.minor
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
