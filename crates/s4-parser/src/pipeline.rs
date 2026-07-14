use crate::ParseUnit;
use s4_core::{ArtifactId, Result};
use s4_storage::Store;
use std::path::Path;

/// Execution context for parsing and persisting USIR artifacts.
pub struct ParseContext<'a> {
    /// Root directory used to compute module names relative to the source tree.
    pub source_root: &'a Path,
    /// Artifact store used to resolve optional inline content and persist USIR output.
    pub store: &'a mut dyn Store,
}

/// Orchestrates parser plugins over parse units.
pub trait ParsePipeline: Send + Sync {
    /// Parse a single unit and return emitted artifact IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    fn parse_unit(&self, unit: &ParseUnit, ctx: &mut ParseContext<'_>) -> Result<Vec<ArtifactId>>;

    /// Parse all units incrementally, reusing cached artifacts where valid.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    fn parse_all(
        &self,
        units: &[ParseUnit],
        ctx: &mut ParseContext<'_>,
    ) -> Result<Vec<ArtifactId>> {
        let mut ids = Vec::with_capacity(units.len());
        for unit in units {
            ids.extend(self.parse_unit(unit, ctx)?);
        }
        Ok(ids)
    }
}
