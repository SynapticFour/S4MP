use thiserror::Error;

/// Platform-wide result type.
pub type Result<T> = std::result::Result<T, S4Error>;

/// Platform-wide error type.
#[derive(Debug, Error)]
pub enum S4Error {
    /// Invalid identifier string or bytes.
    #[error("invalid identifier: {0}")]
    InvalidId(String),

    /// Invalid user input, flags, or configuration.
    #[error("{0}")]
    InvalidInput(String),

    /// Schema version mismatch between producer and consumer.
    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch {
        /// Expected schema version.
        expected: String,
        /// Actual schema version.
        actual: String,
    },

    /// Error originating from a plugin invocation.
    #[error("plugin error ({plugin_id}): {message}")]
    Plugin {
        /// Plugin identifier.
        plugin_id: String,
        /// Human-readable message.
        message: String,
    },

    /// Storage layer error.
    #[error("storage error: {0}")]
    Storage(String),

    /// External process failure (git, etc.).
    #[error("external command: {0}")]
    External(String),

    /// A verification or certification check did not pass.
    #[error("{0}")]
    CheckFailed(String),
}
