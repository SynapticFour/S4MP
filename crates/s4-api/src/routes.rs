use serde::{Deserialize, Serialize};

/// Known API routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApiRoute {
    /// Health check endpoint.
    Health,
    /// Query knowledge graph.
    Query,
    /// Trigger analysis pipeline.
    Analyze,
    /// List project snapshots.
    Snapshots,
}

/// Health check response payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status string.
    pub status: String,
    /// Workspace schema version.
    pub schema_version: String,
}
