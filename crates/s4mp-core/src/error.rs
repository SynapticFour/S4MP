use thiserror::Error;

/// Platform-wide error type.
#[derive(Debug, Error)]
pub enum S4mpError {
    #[error("invalid artifact id: {0}")]
    InvalidArtifactId(String),

    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: String, actual: String },

    #[error("plugin error ({plugin_id}): {message}")]
    Plugin { plugin_id: String, message: String },

    #[error("store error: {0}")]
    Store(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, S4mpError>;
