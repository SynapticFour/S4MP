use crate::ParseUnit;
use s4_core::{ArtifactId, Result};

/// Orchestrates parser plugins over parse units.
pub trait ParsePipeline: Send + Sync {
    /// Parse a single unit and return emitted artifact IDs.
    fn parse_unit(&self, unit: &ParseUnit) -> Result<Vec<ArtifactId>>;

    /// Parse all units incrementally, reusing cached artifacts where valid.
    fn parse_all(&self, units: &[ParseUnit]) -> Result<Vec<ArtifactId>>;
}
